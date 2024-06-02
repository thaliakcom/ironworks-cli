use std::{borrow::Cow, time::Instant};

use clio::ClioPath;
use ironworks::{excel::Field, sestring::SeString};
use ironworks_schema::{Node, Schema};
use crate::{err::{Err, ToUnknownErr}, init::Init, sheets::{Column, LinkCondition, LinkSource, SHEET_COLUMNS}};

/// Extracts a single row from the given sheet and prints a
/// JSON representation of the result to [`stdout`].
pub fn extract(sheet_name: &'static str, id: u32, game_dir: &Option<ClioPath>) -> Result<(), Err> {
    let values = get_values(sheet_name, id, game_dir)?;

    print_values(values)?;

    Ok(())
}

/// Searches for a given string in the given sheet and prints a list of all matching row IDs
/// to [`stdout`].
///
/// Note that this function does not search through _all_ columns; instead
/// only the columns specified in `sheets.rs` are searched.
pub fn search(sheet_name: &'static str, search_str: &str, game_dir: &Option<ClioPath>) -> Result<(), Err> {
    let Init { schema, sheet, .. } = Init::new(sheet_name, game_dir)?;
    let sheet_data = SHEET_COLUMNS.get(sheet_name).to_unknown_err()?;
    let mut matches: Vec<(u32, SeString)> = Vec::new();

    if let Node::Struct(columns) = schema.node {
        let filtered_columns: Vec<_> = columns.iter().filter(|x| sheet_data.search_columns.contains(&x.name.as_ref())).collect();

        for row in sheet.iter() {
            for column in filtered_columns.iter() {
                let index = column.offset as usize;
                let sestring = row.field(index).to_unknown_err()?.into_string().to_unknown_err()?;

                if sestring.to_string().contains(search_str) {
                    matches.push((row.row_id(), sestring));
                }
            }
        }
    } else {
        return Err(Err::UnsupportedSheet(sheet_name));
    }

    if matches.is_empty() {
        println!("No matches found");
    } else {
        println!("{} matches found:", matches.len());

        for match_ in matches {
            println!("  at {: >5}: {}", match_.0, match_.1);
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
fn get_values(sheet_name: &'static str, row_id: u32, game_dir: &Option<ClioPath>) -> Result<Vec<KeyValue>, Err> {
    let Init { excel, schema, sheet, version, .. } = Init::new(sheet_name, game_dir)?;
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
fn print_values(values: Vec<KeyValue>) -> Result<(), Err> {
    print!("{{");
    let len = values.len();

    for (i, field) in values.into_iter().enumerate() {
        print!("\"{}{}\":", &field.key.chars().nth(0).unwrap().to_lowercase(), &field.key[1..]);
        print_value(&field.value);

        if i < len - 1 {
            print!(",");
        }
    }
    println!("}}");

    Ok(())
}

/// Prints the value contained in the field to [`stdout`].
fn print_value(field: &Field) {
    match field {
        Field::String(s) => print!("\"{}\"", s),
        Field::Bool(b) => print!("{}", b),
        Field::I8(num) => print!("{}", num),
        Field::I16(num) => print!("{}", num),
        Field::I32(num) => print!("{}", num),
        Field::I64(num) => print!("{}", num),
        Field::U8(num) => print!("{}", num),
        Field::U16(num) => print!("{}", num),
        Field::U32(num) => print!("{}", num),
        Field::U64(num) => print!("{}", num),
        Field::F32(num) => print!("{}", num)
    }
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
