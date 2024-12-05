use std::borrow::Cow;
use std::collections::HashMap;
use ironworks::excel::{Field, Sheet};
use ironworks::file::exh::ColumnDefinition;
use ironworks::sestring::SeString;
use ironworks_schema::{Node, Order, Schema};
use crate::err::{Err, ToUnknownErr};
use super::sheets::{LinkCondition, LinkSource, SHEET_COLUMNS};
use super::{Args, Init};

/// Extracts a single row from the given sheet and prints a
/// JSON representation of the result to the given output stream.
pub fn extract(sheet: super::sheets::Sheet, id: u32, args: &mut Args<impl std::io::Write>) -> Result<KeyValues, Err> {
    let values = get_values(sheet, id, args)?;

    if let Some(ref mut out) = args.out {
        if args.pretty_print {
            pretty_print_values(out, &values)?;
        } else {
            print_values(out, &values)?;
        }
    }

    Ok(values)
}

pub type KeyValues = HashMap<Cow<'static, str>, Field>;

pub struct KeyValue {
    pub key: Cow<'static, str>,
    pub value: Field
}

pub struct SearchMatch {
    pub id: u32,
    pub name: SeString<'static>,
    pub field: Option<KeyValue>
}

/// Searches for a given string in the given sheet and prints a list of all matching row IDs
/// to [`stdout`].
///
/// Note that this function does not search through _all_ columns; instead
/// only the columns specified in `sheets.rs` are searched.
pub fn search(sheet: super::sheets::Sheet, search_str: &str, args: &mut Args<impl std::io::Write>) -> Result<Vec<SearchMatch>, Err> {
    let sheet_name: &'static str = sheet.into();
    let Init { schema, sheet, .. } = Init::new(sheet_name, args)?;
    let sheet_data = SHEET_COLUMNS.get(sheet_name).to_unknown_err()?;

    let mut matches: Vec<SearchMatch> = Vec::new();
    let filtered_columns: Vec<SheetColumn> = filtered_column_iter(&sheet, schema, Some(sheet_data.columns))?.collect();
    let name_column = filtered_columns.iter().find(|x| &x.name == sheet_data.identifier).to_unknown_err()?;
    let search_columns: Vec<_> = filtered_columns.iter().filter(|x| sheet_data.search_columns.contains(&x.name.as_ref())).collect();

    for row in sheet.into_iter() {
        let name = row.field(&name_column.column).to_unknown_err()?.into_string().to_unknown_err()?;

        for column in search_columns.iter() {
            let field = row.field(&column.column).to_unknown_err()?;
            let sestring = field.as_string().to_unknown_err()?;

            if sestring.to_string().to_lowercase().contains(&search_str.to_lowercase()) {
                if column.name == name_column.name {
                    matches.push(SearchMatch { id: row.row_id(), name, field: None });
                } else {
                    matches.push(SearchMatch { id: row.row_id(), name, field: Some(KeyValue { key: Cow::Owned(column.name.clone()), value: field }) });
                }

                break;
            }
        }
    }

    
    if let Some(ref mut out) = args.out {
        if matches.is_empty() {
            writeln!(out, "No matches found").unwrap();
        } else {
            writeln!(out, "{} matches found:", matches.len()).unwrap();
    
            for SearchMatch { id, name, field } in matches.iter() {
                write!(out, "  at {: >5}: ", id).unwrap();
                print_string(out, name).unwrap();
    
                if let Some(key_value) = field {
                    write!(out, " -> {{ \"{}\": ", key_value.key).unwrap();
                    print_value(out, &key_value.value);
                    write!(out, " }}").unwrap();
                }
    
                writeln!(out, ).unwrap();
            }
        }
    }

    Ok(matches)
}

pub(crate) struct SheetColumn {
    pub name: String,
    pub column: ColumnDefinition
}

pub(crate) fn column_iter(sheet: &Sheet<&str>, schema: ironworks_schema::Sheet) -> Result<impl Iterator<Item = SheetColumn>, Err> {
    filtered_column_iter(sheet, schema, None)
}

pub(crate) fn filtered_column_iter(sheet: &Sheet<&str>, schema: ironworks_schema::Sheet, filter_columns: Option<&'static [&'static str]>) -> Result<impl Iterator<Item = SheetColumn>, Err> {
    let mut columns = sheet.columns().map_err(|_| Err::UnsupportedSheet(Cow::Owned(sheet.name())))?;
    let fields = if let Node::Struct(columns) = schema.node { columns } else { Err(Err::UnsupportedSheet(Cow::Owned(sheet.name())))? };

    match schema.order {
        Order::Index => (),
        Order::Offset => columns.sort_by_key(|column| column.offset()),
    };

    Ok(fields.into_iter()
        .filter(move |x| {
            if let Some(filter_columns) = filter_columns {
                filter_columns.contains(&x.name.as_ref())
            } else {
                true
            }
        })
        .map(move |x| SheetColumn { name: x.name, column: columns[x.offset as usize].clone() }))
}

/// Gets a [`Vec`] of the field values and their field names
/// from the given row in the given sheet.
///
/// Note that this function does not extract _all_ fields. Instead only
/// the fields specified in `sheets.rs` are extracted.
fn get_values(sheet: super::sheets::Sheet, row_id: u32, args: &mut Args<impl std::io::Write>) -> Result<KeyValues, Err> {
    let sheet_name: &'static str = sheet.into();
    let Init { excel, schema, sheet, version, .. } = Init::new(sheet_name, args)?;
    // For some reason calling `sheet.row()` on the Action sheet
    // takes longer than any other sheet by a magnitude of about 4x.
    // Since this is a bug in the dependency, we can't fix it.
    let row = sheet.row(row_id).map_err(|_| Err::RowNotFound(sheet_name, row_id))?;
    let sheet_data = SHEET_COLUMNS.get(sheet_name);
    let filtered_columns: Vec<SheetColumn> = filtered_column_iter(&sheet, schema, sheet_data.map(|x| x.columns))?.collect();

    let mut result: KeyValues = filtered_columns.iter()
        .map(|column| (Cow::Owned(column.name.to_owned()), row.field(&column.column).unwrap()))
        .collect();

    if let Some(data) = sheet_data {
        for link in data.links {
            if match link.condition {
                LinkCondition::Always => false,
                LinkCondition::IfNot(condition_col, ref condition_val) => compare_fields(result.iter().find(|x| x.0 == condition_col).to_unknown_err()?.1, condition_val)
            } {
                continue;
            }

            let linked_sheet = excel.sheet(link.sheet).map_err(|_| Err::SheetNotFound(link.sheet))?;
            let linked_row_id = if let LinkSource::Field(column_name) = link.source {
                let value = &result.iter().find(|x| x.0 == column_name).ok_or(Err::ColumnNotFound(sheet_name, column_name))?.1;
                get_u32(value).ok_or(Err::NoIndex(sheet_name, column_name))?
            } else {
                row_id
            };

            let linked_row = linked_sheet.row(linked_row_id).map_err(|_| Err::RowNotFound(link.sheet, linked_row_id))?;
            let linked_schema = version.sheet(link.sheet).to_unknown_err()?;

            for column in column_iter(&linked_sheet, linked_schema)? {
                let link_data = link.columns.iter().find(|x| x.source == column.name);

                if let Some(link_data) = link_data {
                    result.insert(Cow::Borrowed(link_data.target), linked_row.field(&column.column).to_unknown_err()?);
                }
            }
        }
    }

    Ok(result)
}

/// Prints the list of named values to [`stdout`] in JSON format.
fn print_values(out: &mut impl std::io::Write, values: &KeyValues) -> Result<(), Err> {
    write!(out, "{{").unwrap();
    let len = values.len();

    for (i, (key, field)) in values.iter().enumerate() {
        write!(out, "\"{}{}\":", &key.chars().next().unwrap().to_lowercase(), &key[1..]).unwrap();
        print_value(out, field);

        if i < len - 1 {
            write!(out, ",").unwrap();
        }
    }
    writeln!(out, "}}").unwrap();

    Ok(())
}

/// Prints the list of named values to [`stdout`] in JSON format.
fn pretty_print_values(out: &mut impl std::io::Write, values: &KeyValues) -> Result<(), Err> {
    writeln!(out, "{{").unwrap();
    let len = values.len();

    for (i, (key, value)) in values.iter().enumerate() {
        write!(out, "  \"{}{}\": ", &key.chars().next().unwrap().to_lowercase(), &key[1..]).unwrap();
        print_value(out, value);

        if i < len - 1 {
            writeln!(out, ",").unwrap();
        }
    }
    writeln!(out, "\n}}").unwrap();

    Ok(())
}

/// Prints the value contained in the field to [`stdout`].
pub(crate) fn print_value(out: &mut impl std::io::Write, field: &Field) {
    match field {
        Field::String(s) => print_string(out, s),
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

fn print_string(out: &mut impl std::io::Write, string: &SeString<'static>) -> std::io::Result<()> {
    write!(out, "\"{}\"", string.to_string().replace('\n', "\\n").replace('"', "\\\""))
}

/// Attempts to convert the value contained in the field to [`u32`].
fn get_u32(field: &Field) -> Option<u32> {
    match field {
        Field::I8(num) => Some(*num as u32),
        Field::I16(num) => Some(*num as u32),
        Field::I32(num) => Some(*num as u32),
        Field::I64(num) => Some(*num as u32),
        Field::U8(num) => Some(*num as u32),
        Field::U16(num) => Some(*num as u32),
        Field::U32(num) => Some(*num),
        Field::U64(num) => Some(*num as u32),
        Field::F32(num) => Some(*num as u32),
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
