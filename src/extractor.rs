use clio::ClioPath;
use ironworks::{excel::{Excel, Field, Language}, sestring::SeString, sqpack::{Install, Resource, SqPack}, Ironworks};
use ironworks_schema::{saint_coinach::Provider, Node, Schema};
use crate::{err::{Err, ToUnknownErr}, sheets::{LinkSource, SHEET_COLUMNS}};

/// Extracts a single row from the given sheet and prints a
/// JSON representation of the result to [`stdout`].
pub fn extract(sheet_name: &'static str, id: u32, game_dir: &Option<ClioPath>) -> Result<(), Err> {
    let game_resource = if let Some(game_dir) = game_dir {
        Some(Install::at(game_dir.path()))
    } else {
        Install::search()
    }.ok_or(Err::GameNotFound)?;

    // There's currently an error in ironworks where search() always returns
    // Some(), even if no path was found. We do this check to ensure the path
    // actually points to the game.
    game_resource.version(0).map_err(|_| Err::GameNotFound)?;

    let ironworks = Ironworks::new().with_resource(SqPack::new(game_resource));
    let excel = Excel::with().language(Language::English).build(&ironworks);
    let values = get_values(excel, sheet_name, id)?;

    print_values(values)?;

    Ok(())
}

/// Searches for a given string in the given sheet and prints a list of all matching row IDs
/// to [`stdout`].
///
/// Note that this function does not search through _all_ columns; instead
/// only the columns specified in `sheets.rs` are searched.
pub fn search(sheet_name: &'static str, search_str: &str, game_dir: &Option<ClioPath>) -> Result<(), Err> {
    let game_resource = if let Some(game_dir) = game_dir {
        Some(Install::at(game_dir.path()))
    } else {
        Install::search()
    }.ok_or(Err::GameNotFound)?;

    // There's currently an error in ironworks where search() always returns
    // Some(), even if no path was found. We do this check to ensure the path
    // actually points to the game.
    game_resource.version(0).map_err(|_| Err::GameNotFound)?;

    let ironworks = Ironworks::new().with_resource(SqPack::new(game_resource));
    let excel = Excel::with().language(Language::English).build(&ironworks);
    let provider = Provider::new().to_unknown_err()?;
    let version = provider.version("HEAD").to_unknown_err()?;
    let schema = version.sheet(sheet_name).to_unknown_err()?;
    let sheet = excel.sheet(sheet_name).map_err(|_| Err::SheetNotFound(sheet_name))?;
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
    key: String,
    value: Field
}

/// Gets a [`Vec`] of the field values and their field names
/// from the given row in the given sheet.
///
/// Note that this function does not extract _all_ fields. Instead only
/// the fields specified in `sheets.rs` are extracted.
fn get_values(excel: Excel, sheet_name: &'static str, row_id: u32) -> Result<Vec<KeyValue>, Err> {
    let provider = Provider::new().to_unknown_err()?;
    let version = provider.version("HEAD").to_unknown_err()?;
    let schema = version.sheet(sheet_name).to_unknown_err()?;
    let sheet = excel.sheet(sheet_name).map_err(|_| Err::SheetNotFound(sheet_name))?;
    let row = sheet.row(row_id).map_err(|_| Err::RowNotFound(sheet_name, row_id))?;
    let sheet_data = SHEET_COLUMNS.get(sheet_name);

    if let Node::Struct(columns) = schema.node {
        let fallible_result: Result<Vec<KeyValue>, Err> = columns.iter()
        .filter(|column| if let Some(data) = sheet_data { data.columns.contains(&column.name.as_ref()) } else { true })
        .map(|column| {
            let index = column.offset as usize;
            Ok(KeyValue { key: column.name.clone(), value: row.field(index).to_unknown_err()? })
        })
        .collect();
        let mut result: Vec<KeyValue> = fallible_result?;
    
        if let Some(data) = sheet_data {
            for link in data.links {
                let linked_sheet = excel.sheet(link.sheet).map_err(|_| Err::SheetNotFound(link.sheet))?;
                let linked_row_id = if let LinkSource::Field(column_name) = link.source {
                    let column = columns.iter().find(|x| x.name == column_name).ok_or(Err::ColumnNotFound(sheet_name, column_name))?;
                    let index = column.offset as usize;
                    get_u32(row.field(index).to_unknown_err()?).ok_or(Err::NoIndex(sheet_name, column_name))?
                } else {
                    row_id
                };

                let linked_row = linked_sheet.row(linked_row_id).map_err(|_| Err::RowNotFound(link.sheet, linked_row_id))?;
                let linked_schema = version.sheet(link.sheet).to_unknown_err()?;
                let linked_columns = if let Node::Struct(columns) = linked_schema.node { Ok(columns) } else { Err(Err::UnsupportedSheet(link.sheet)) }?;
                let column_data = linked_columns.into_iter().filter_map(|column| {
                    let link_data = link.columns.iter().find(|x| x.source == column.name)?;

                    Some((link_data, column))
                });

                for (link_data, column) in column_data {
                    result.push(KeyValue { key: link_data.target.to_owned(), value: linked_row.field(column.offset as usize).to_unknown_err()? });
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

    for (i, column) in values.into_iter().enumerate() {
        print!("\"{}\":", &column.key);
        print_value(&column.value);

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
