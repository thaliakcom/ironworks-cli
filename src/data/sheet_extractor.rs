use std::borrow::Cow;
use ironworks::excel::Field;
use ironworks_schema::{Node, Schema};
use crate::err::{Err, ToUnknownErr};
use super::sheets::{Column, LinkCondition, LinkSource, SHEET_COLUMNS};
use super::{Args, Init};

/// Extracts a single row from the given sheet and prints a
/// JSON representation of the result to [`stdout`].
pub fn extract(sheet_name: &'static str, id: u32, args: &mut Args<impl std::io::Write>, pretty_print: bool) -> Result<(), Err> {
    let values = get_values(sheet_name, id, args)?;

    if pretty_print {
        pretty_print_values(&mut args.out, values)?;
    } else {
        print_values(&mut args.out, values)?;
    }

    Ok(())
}

struct SearchMatch {
    id: u32,
    name: Field,
    field: Option<KeyValue>
}

/// Searches for a given string in the given sheet and prints a list of all matching row IDs
/// to [`stdout`].
///
/// Note that this function does not search through _all_ columns; instead
/// only the columns specified in `sheets.rs` are searched.
pub fn search(sheet_name: &'static str, search_str: &str, args: &mut Args<impl std::io::Write>) -> Result<(), Err> {
    let Init { schema, sheet, .. } = Init::new(sheet_name, args)?;
    let sheet_data = SHEET_COLUMNS.get(sheet_name).to_unknown_err()?;
    let mut matches: Vec<SearchMatch> = Vec::new();

    if let Node::Struct(columns) = schema.node {
        let name_column_offset = columns.iter().find(|x| x.name == sheet_data.identifier).unwrap().offset as usize;
        let filtered_columns: Vec<_> = columns.iter().filter(|x| sheet_data.search_columns.contains(&x.name.as_ref())).collect();

        for row in sheet.iter() {
            let name = row.field(name_column_offset).to_unknown_err()?;

            for column in filtered_columns.iter() {
                let index = column.offset as usize;
                let field = row.field(index).to_unknown_err()?;
                let sestring = field.as_string().to_unknown_err()?;

                if sestring.to_string().contains(search_str) {
                    if name_column_offset == index {
                        matches.push(SearchMatch { id: row.row_id(), name, field: None });
                    } else {
                        matches.push(SearchMatch { id: row.row_id(), name, field: Some(KeyValue { key: Cow::Owned(column.name.clone()), value: field }) });
                    }

                    break;
                }
            }
        }
    } else {
        return Err(Err::UnsupportedSheet(sheet_name));
    }

    if matches.is_empty() {
        writeln!(args.out, "No matches found").unwrap();
    } else {
        writeln!(args.out, "{} matches found:", matches.len()).unwrap();

        for SearchMatch { id, name, field } in matches {
            write!(args.out, "  at {: >5}: ", id).unwrap();
            print_value(&mut args.out, &name);

            if let Some(key_value) = field {
                write!(args.out, " -> {{ \"{}\": ", key_value.key).unwrap();
                print_value(&mut args.out, &key_value.value);
                write!(args.out, " }}").unwrap();
            }

            writeln!(args.out, ).unwrap();
        }
    }

    Ok(())
}

struct KeyValue {
    key: Cow<'static, str>,
    value: Field
}

/// Gets a [`Vec`] of the field values and their field names
/// from the given row in the given sheet.
///
/// Note that this function does not extract _all_ fields. Instead only
/// the fields specified in `sheets.rs` are extracted.
fn get_values(sheet_name: &'static str, row_id: u32, args: &mut Args<impl std::io::Write>) -> Result<Vec<KeyValue>, Err> {
    let Init { excel, schema, sheet, version, .. } = Init::new(sheet_name, args)?;
    // For some reason calling `sheet.row()` on the Action sheet
    // takes longer than any other sheet by a magnitude of about 4x.
    // Since this is a bug in the dependency, we can't fix it.
    let row = sheet.row(row_id).map_err(|_| Err::RowNotFound(sheet_name, row_id))?;
    let sheet_data = SHEET_COLUMNS.get(sheet_name);

    if let Node::Struct(columns) = schema.node {
        let mut result: Vec<KeyValue> = columns.iter()
            .filter_map(|column| {
                if let Some(data) = sheet_data {
                    data.columns.iter().find(|x| x.name() == column.name).map(|matching_column| match matching_column {
                            Column::AsIs(name) => KeyValue { key: Cow::Borrowed(name), value: row.field(column.offset as usize).unwrap() },
                            Column::Alias(_, alias) => KeyValue { key: Cow::Borrowed(alias), value: row.field(column.offset as usize).unwrap() }
                        })
                } else {
                    Some(KeyValue { key: Cow::Owned(column.name.to_owned()), value: row.field(column.offset as usize).unwrap() })
                }
            })
            .collect();
    
        if let Some(data) = sheet_data {
            for link in data.links {
                if match link.condition {
                    LinkCondition::Always => false,
                    LinkCondition::IfNot(condition_col, ref condition_val) => compare_fields(&row.field(columns.iter().find(|x| x.name == condition_col).unwrap().offset as usize).unwrap(), condition_val)
                } {
                    continue;
                }

                let linked_sheet = excel.sheet(link.sheet).map_err(|_| Err::SheetNotFound(link.sheet))?;
                let linked_row_id = if let LinkSource::Field(column_name) = link.source {
                    let column = columns.iter().find(|x| x.name == column_name).ok_or(Err::ColumnNotFound(sheet_name, column_name))?;
                    get_u32(row.field(column.offset as usize).to_unknown_err()?).ok_or(Err::NoIndex(sheet_name, column_name))?
                } else {
                    row_id
                };

                let linked_row = linked_sheet.row(linked_row_id).map_err(|_| Err::RowNotFound(link.sheet, linked_row_id))?;
                let linked_schema = version.sheet(link.sheet).to_unknown_err()?;
                let linked_columns = if let Node::Struct(columns) = linked_schema.node { Ok(columns) } else { Err(Err::UnsupportedSheet(link.sheet)) }?;
                let column_data = linked_columns.into_iter().filter_map(|column| {
                    let link_data = link.columns.iter().find(|x| x.source == column.name)?;

                    Some((link_data, column.offset))
                });

                for (link_data, offset) in column_data {
                    result.push(KeyValue { key: Cow::Borrowed(link_data.target), value: linked_row.field(offset as usize).to_unknown_err()? });
                }
            }
        }
    
        Ok(result)
    } else {
        Err(Err::UnsupportedSheet(sheet_name))
    }
}

/// Prints the list of named values to [`stdout`] in JSON format.
fn print_values(out: &mut impl std::io::Write, values: Vec<KeyValue>) -> Result<(), Err> {
    write!(out, "{{").unwrap();
    let len = values.len();

    for (i, field) in values.into_iter().enumerate() {
        write!(out, "\"{}{}\":", &field.key.chars().next().unwrap().to_lowercase(), &field.key[1..]).unwrap();
        print_value(out, &field.value);

        if i < len - 1 {
            write!(out, ",").unwrap();
        }
    }
    writeln!(out, "}}").unwrap();

    Ok(())
}

/// Prints the list of named values to [`stdout`] in JSON format.
fn pretty_print_values(out: &mut impl std::io::Write, values: Vec<KeyValue>) -> Result<(), Err> {
    writeln!(out, "{{").unwrap();
    let len = values.len();

    for (i, field) in values.into_iter().enumerate() {
        write!(out, "  \"{}{}\": ", &field.key.chars().next().unwrap().to_lowercase(), &field.key[1..]).unwrap();
        print_value(out, &field.value);

        if i < len - 1 {
            writeln!(out, ",").unwrap();
        }
    }
    writeln!(out, "\n}}").unwrap();

    Ok(())
}

/// Prints the value contained in the field to [`stdout`].
pub fn print_value(out: &mut impl std::io::Write, field: &Field) {
    match field {
        Field::String(s) => write!(out, "\"{}\"", s.to_string().replace('\n', "\\n").replace('"', "\\\"")),
        Field::Bool(b) => write!(out, "{}", b),
        Field::I8(num) => write!(out, "{}", num),
        Field::I16(num) => write!(out, "{}", num),
        Field::I32(num) => write!(out, "{}", num),
        Field::I64(num) => write!(out, "{}", num),
        Field::U8(num) => write!(out, "{}", num),
        Field::U16(num) => write!(out, "{}", num),
        Field::U32(num) => write!(out, "{}", num),
        Field::U64(num) => write!(out, "{}", num),
        Field::F32(num) => write!(out, "{}", num)
    }.unwrap();
}

/// Attempts to convert the value contained in the field to [`u32`].
fn get_u32(field: Field) -> Option<u32> {
    match field {
        Field::I8(num) => Some(num as u32),
        Field::I16(num) => Some(num as u32),
        Field::I32(num) => Some(num as u32),
        Field::I64(num) => Some(num as u32),
        Field::U8(num) => Some(num as u32),
        Field::U16(num) => Some(num as u32),
        Field::U32(num) => Some(num),
        Field::U64(num) => Some(num as u32),
        Field::F32(num) => Some(num as u32),
        _ => None
    }
}

fn compare_fields(a: &Field, b: &Field) -> bool {
    match (a, b) {
        (Field::String(a), Field::String(b)) => a.to_string() == b.to_string(),
        (Field::Bool(a), Field::Bool(b)) => a == b,
        (Field::I8(a), Field::I8(b)) => a == b,
        (Field::I16(a), Field::I16(b)) => a == b,
        (Field::I32(a), Field::I32(b)) => a == b,
        (Field::I64(a), Field::I64(b)) => a == b,
        (Field::U8(a), Field::U8(b)) => a == b,
        (Field::U16(a), Field::U16(b)) => a == b,
        (Field::U32(a), Field::U32(b)) => a == b,
        (Field::U64(a), Field::U64(b)) => a == b,
        (Field::F32(a), Field::F32(b)) => a == b,
        _ => false
    }
}
