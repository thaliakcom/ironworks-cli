use std::io::Write;

use clio::{ClioPath, Output};
use ironworks::{excel::{Excel, Field, Language}, file::exh::ColumnKind, sqpack::{Install, Resource, SqPack}, Ironworks};
use ironworks_schema::{saint_coinach::Provider, Node, Schema};
use crate::{err::Err, sheets::SHEET_COLUMNS};

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
    let provider = Provider::new().unwrap();
    let version = provider.version("HEAD").unwrap();
    let schema = version.sheet(sheet_name).unwrap();
    let sheet = excel.sheet(sheet_name).map_err(|_| Err::SheetNotFound(sheet_name))?;
    let row = sheet.row(id).map_err(|_| Err::RowNotFound(sheet_name, id))?;
    let column_definitions = sheet.columns().unwrap();
    let accepted_columns = SHEET_COLUMNS.get(sheet_name);

    if let Node::Struct(columns) = schema.node {
        write!(&mut output, "{{").unwrap();
        let filtered_columns: Vec<_> = columns.iter()
            .filter(|column| if let Some(accepted_columns) = accepted_columns { accepted_columns.contains(&column.name.as_ref()) } else { true })
            .collect();
        let len = filtered_columns.len();
        for (i, column) in filtered_columns.iter().enumerate() {
            let index = column.offset as usize;
            let definition = column_definitions.get(index).unwrap();
            let kind = definition.kind();
            write!(&mut output, "\"{}\":", &column.name).unwrap();
            write_value(&mut output, row.field(index).unwrap(), kind);

            if i < len - 1 {
                write!(&mut output, ",").unwrap();
            }
        }
        write!(&mut output, "}}\n").unwrap();
    } else {
        return Err(Err::UnsupportedSheet(sheet_name));
    }

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
