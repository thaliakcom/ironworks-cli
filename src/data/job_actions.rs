use clio::ClioPath;
use ironworks::excel::Excel;
use ironworks_schema::{saint_coinach::Version, Node, Schema};
use crate::err::{Err, ToUnknownErr};
use super::Init;

const SHEET_NAME: &'static str = "Action";
const CLASS_JOB_SHEET_NAME: &'static str = "ClassJob";

fn get_base_class<'a>(id: u32, excel: Excel<'a>, version: Version) -> Result<u8, Err> {
    let class_jobs = excel.sheet(CLASS_JOB_SHEET_NAME).to_unknown_err()?;
    let schema = version.sheet(CLASS_JOB_SHEET_NAME).to_unknown_err()?;
    
    if let Node::Struct(columns) = schema.node {
        let base_class_column = columns.iter().find(|x| x.name == "ClassJob{Parent}").to_unknown_err()?;
        let index = base_class_column.offset as usize;

        let class_job = class_jobs.row(id).map_err(|_| Err::JobNotFound(id))?;

        Ok(class_job.field(index).to_unknown_err()?.into_u8().to_unknown_err()?)
    } else {
        return Err(Err::UnsupportedSheet(SHEET_NAME));
    }
}

/// Searches for a given string in the given sheet and prints a list of all matching row IDs
/// to [`stdout`].
///
/// Note that this function does not search through _all_ columns; instead
/// only the columns specified in `sheets.rs` are searched.
pub fn get(id: u32, game_dir: &Option<ClioPath>) -> Result<(), Err> {
    let Init { schema, sheet: actions, excel, version } = Init::new(SHEET_NAME, game_dir)?;
    let base_class = get_base_class(id, excel, version)?;
    let mut matches: Vec<String> = Vec::new();

    if let Node::Struct(columns) = schema.node {
        let class_job_column = columns.iter().find(|x| x.name == "ClassJob").to_unknown_err()?;
        let index = class_job_column.offset as usize;

        for row in actions.iter() {
            let class_job_id = row.field(index).to_unknown_err()?.into_i8().to_unknown_err()?;

            if class_job_id == id as i8 || class_job_id == base_class as i8 {
                matches.push(row.row_id().to_string());
            }
        }
    } else {
        return Err(Err::UnsupportedSheet(SHEET_NAME));
    }

    println!("[{}]", matches.join(", "));

    Ok(())
}
