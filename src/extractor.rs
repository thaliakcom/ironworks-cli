use clio::{ClioPath, Output};
use ironworks::{excel::{Excel, Field, Language}, file::exh::ColumnKind, sestring::SeString, sqpack::{Install, Resource, SqPack}, Ironworks};
use ironworks_schema::{saint_coinach::Provider, Node, Schema};
use crate::{err::{Err, ToUnknownErr}, sheets::{SheetLinkTarget, SHEET_COLUMNS}};

pub fn extract(mut output: &mut Output, sheet_name: &'static str, id: u32, game_dir: &Option<ClioPath>) -> Result<(), Err> {
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

    write_values(&mut output, values)?;

    Ok(())
}

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

    if matches.len() == 0 {
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
    value: Field,
    kind: ColumnKind
}

fn get_values(excel: Excel, sheet_name: &'static str, row_id: u32) -> Result<Vec<KeyValue>, Err> {
    let provider = Provider::new().to_unknown_err()?;
    let version = provider.version("HEAD").to_unknown_err()?;
    let schema = version.sheet(sheet_name).to_unknown_err()?;
    let sheet = excel.sheet(sheet_name).map_err(|_| Err::SheetNotFound(sheet_name))?;
    let row = sheet.row(row_id).map_err(|_| Err::RowNotFound(sheet_name, row_id))?;
    let column_defs = sheet.columns().to_unknown_err()?;
    let sheet_data = SHEET_COLUMNS.get(sheet_name);

    if let Node::Struct(columns) = schema.node {
        let fallible_result: Result<Vec<KeyValue>, Err> = columns.iter()
        .filter(|column| if let Some(data) = sheet_data { data.columns.contains(&column.name.as_ref()) } else { true })
        .map(|column| {
            let index = column.offset as usize;
            let definition = column_defs.get(index).to_unknown_err()?;
            Ok(KeyValue { key: column.name.clone(), value: row.field(index).to_unknown_err()?, kind: definition.kind() })
        })
        .collect();
        let mut result: Vec<KeyValue> = fallible_result?;
    
        if let Some(data) = sheet_data {
            for link in data.links {
                let linked_sheet = excel.sheet(link.sheet).map_err(|_| Err::SheetNotFound(link.sheet))?;
                let linked_row_id = if let SheetLinkTarget::Field(column_name) = link.target {
                    let column = columns.iter().find(|x| x.name == column_name).ok_or(Err::ColumnNotFound(sheet_name, column_name))?;
                    let index = column.offset as usize;
                    let definition = column_defs.get(index).to_unknown_err()?;
                    get_u32(row.field(index).to_unknown_err()?, definition.kind()).ok_or(Err::NoIndex(sheet_name, column_name))?
                } else {
                    row_id
                };

                let linked_row = linked_sheet.row(linked_row_id).map_err(|_| Err::RowNotFound(link.sheet, linked_row_id))?;
                let linked_schema = version.sheet(link.sheet).to_unknown_err()?;
                let linked_columns = if let Node::Struct(columns) = linked_schema.node { Ok(columns) } else { Err(Err::UnsupportedSheet(link.sheet)) }?;
                let linked_defs = linked_sheet.columns().to_unknown_err()?;
                let column_data = linked_columns.into_iter().filter_map(|column| {
                    let link_data = link.columns.iter().find(|x| x.source == column.name)?;

                    Some((link_data, column))
                });

                for (link_data, column) in column_data {
                    let index = column.offset as usize;
                    let definition = linked_defs.get(index).to_unknown_err()?;
                    result.push(KeyValue { key: link_data.target.to_owned(), value: linked_row.field(index).to_unknown_err()?, kind: definition.kind() });
                }
            }
        }
    
        Ok(result)
    } else {
        return Err(Err::UnsupportedSheet(sheet_name));
    }
}

fn write_values(mut output: impl std::io::Write, values: Vec<KeyValue>) -> Result<(), Err> {
    write!(&mut output, "{{").to_unknown_err()?;
    let len = values.len();

    for (i, column) in values.into_iter().enumerate() {
        write!(&mut output, "\"{}\":", &column.key).to_unknown_err()?;
        write_value(&mut output, column.value, column.kind);

        if i < len - 1 {
            write!(&mut output, ",").to_unknown_err()?;
        }
    }
    write!(&mut output, "}}\n").to_unknown_err()?;

    Ok(())
}

fn write_value(mut w: impl std::io::Write, field: Field, kind: ColumnKind) {
    match kind {
        ColumnKind::String => write!(w, "\"{}\"", field.into_string().unwrap()),
        ColumnKind::Bool => write!(w, "{}", field.into_bool().unwrap()),
        ColumnKind::Int8 => write!(w, "{}", field.into_i8().unwrap()),
        ColumnKind::UInt8 => write!(w, "{}", field.into_u8().unwrap()),
        ColumnKind::Int16 => write!(w, "{}", field.into_i16().unwrap()),
        ColumnKind::UInt16 => write!(w, "{}", field.into_u16().unwrap()),
        ColumnKind::Int32 => write!(w, "{}", field.into_i32().unwrap()),
        ColumnKind::UInt32 => write!(w, "{}", field.into_u32().unwrap()),
        ColumnKind::Float32 => write!(w, "{}", field.into_f32().unwrap()),
        ColumnKind::Int64 => write!(w, "{}", field.into_i64().unwrap()),
        ColumnKind::UInt64 => write!(w, "{}", field.into_u64().unwrap()),
        ColumnKind::PackedBool0 => write!(w, "{}", field.into_bool().unwrap()),
        ColumnKind::PackedBool1 => write!(w, "{}", field.into_bool().unwrap()),
        ColumnKind::PackedBool2 => write!(w, "{}", field.into_bool().unwrap()),
        ColumnKind::PackedBool3 => write!(w, "{}", field.into_bool().unwrap()),
        ColumnKind::PackedBool4 => write!(w, "{}", field.into_bool().unwrap()),
        ColumnKind::PackedBool5 => write!(w, "{}", field.into_bool().unwrap()),
        ColumnKind::PackedBool6 => write!(w, "{}", field.into_bool().unwrap()),
        ColumnKind::PackedBool7 => write!(w, "{}", field.into_bool().unwrap())
    }.unwrap()
}

fn get_u32(field: Field, kind: ColumnKind) -> Option<u32> {
    match kind {
        ColumnKind::Int8 => Some(field.into_i8().unwrap() as u32),
        ColumnKind::UInt8 => Some(field.into_u8().unwrap() as u32),
        ColumnKind::Int16 => Some(field.into_i16().unwrap() as u32),
        ColumnKind::UInt16 => Some(field.into_u16().unwrap() as u32),
        ColumnKind::Int32 => Some(field.into_i32().unwrap() as u32),
        ColumnKind::UInt32 => Some(field.into_u32().unwrap() as u32),
        ColumnKind::Float32 => Some(field.into_f32().unwrap() as u32),
        ColumnKind::Int64 => Some(field.into_i64().unwrap() as u32),
        ColumnKind::UInt64 => Some(field.into_u64().unwrap() as u32),
        _ => None
    }
}
