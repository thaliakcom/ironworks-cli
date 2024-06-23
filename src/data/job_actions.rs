use clio::ClioPath;
use ironworks::excel::{Excel, Field};
use ironworks_schema::{saint_coinach::Version, Node, Schema};
use crate::{cli::Id, data::sheet_extractor::print_value, err::{Err, ToUnknownErr}};
use super::Init;

const SHEET_NAME: &'static str = "Action";
const CLASS_JOB_SHEET_NAME: &'static str = "ClassJob";

fn get_class_id<'a>(id: &Id, excel: Excel<'a>, version: Version) -> Result<(u8, u8), Err> {
    let class_jobs = excel.sheet(CLASS_JOB_SHEET_NAME).to_unknown_err()?;
    let schema = version.sheet(CLASS_JOB_SHEET_NAME).to_unknown_err()?;
    
    if let Node::Struct(columns) = schema.node {
        let class_id = match id {
            Id::Index(id) => *id,
            Id::Name(abbreviation) => {
                let abbreviation_column = columns.iter().find(|x| x.name == "Abbreviation").to_unknown_err()?.offset as usize;
                class_jobs.iter().find(|x| &x.field(abbreviation_column).unwrap().into_string().unwrap().to_string() == abbreviation).to_unknown_err()?.row_id()
            }
        };

        let base_class_column = columns.iter().find(|x| x.name == "ClassJob{Parent}").to_unknown_err()?.offset as usize;
        let class_job = class_jobs.row(class_id).map_err(|_| Err::JobNotFound(class_id))?;

        Ok((class_id as u8, class_job.field(base_class_column).to_unknown_err()?.into_u8().to_unknown_err()?))
    } else {
        return Err(Err::UnsupportedSheet(SHEET_NAME));
    }
}

/// Searches for a given string in the given sheet and prints a list of all matching row IDs
/// to [`stdout`].
///
/// Note that this function does not search through _all_ columns; instead
/// only the columns specified in `sheets.rs` are searched.
pub fn get(id: &Id, game_dir: &Option<ClioPath>, names: bool, pretty_print: bool) -> Result<(), Err> {
    let Init { schema, sheet: actions, excel, version } = Init::new(SHEET_NAME, game_dir)?;
    let (class_id, base_class_id) = get_class_id(id, excel, version)?;
    let mut matches: Vec<Field> = Vec::new();

    if let Node::Struct(columns) = schema.node {
        let class_job_column = columns.iter().find(|x| x.name == "ClassJob").to_unknown_err()?.offset as usize;
        let name_column = if names { columns.iter().find(|x| x.name == "Name").to_unknown_err()?.offset as usize } else { 0 };

        for row in actions.iter() {
            let class_job_id = row.field(class_job_column).to_unknown_err()?.into_i8().to_unknown_err()?;

            if class_job_id == class_id as i8 || class_job_id == base_class_id as i8 {
                matches.push(Field::U32(row.row_id()));

                if names {
                    matches.push(row.field(name_column).to_unknown_err()?);
                }
            }
        }
    } else {
        return Err(Err::UnsupportedSheet(SHEET_NAME));
    }

    if pretty_print {
        pretty_print_values(matches, names)
    } else {
        print_values(matches, names);
    }

    Ok(())
}

fn print_values(matches: Vec<Field>, names: bool) {
    print!("[");

    if names {
        for (i, m) in matches.chunks_exact(2).enumerate() {
            if i != 0 {
                print!(",");
            }

            print!("{{\"id\":");
            print_value(&m[0]);
            print!(",\"name\":");
            print_value(&m[1]);
            print!("}}");
        }
    } else {
        for (i, index) in matches.iter().enumerate() {
            if i != 0 {
                print!(",");
            }

            print_value(index);
        }
    }

    println!("]");
}

fn pretty_print_values(matches: Vec<Field>, names: bool) {
    println!("[");

    if names {
        for (i, m) in matches.chunks_exact(2).enumerate() {
            if i != 0 {
                println!(",");
            }

            print!("  {{ \"id\": ");
            print_value(&m[0]);
            print!(", \"name\": ");
            print_value(&m[1]);
            print!(" }}");
        }
    } else {
        for (i, index) in matches.iter().enumerate() {
            if i != 0 {
                println!(",");
            }

            print!("  ");
            print_value(index);
        }
    }

    println!("\n]");
}
