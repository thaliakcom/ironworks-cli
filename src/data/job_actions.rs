use ironworks::excel::Excel;
use ironworks_schema::{exdschema::Version, Schema};
use crate::{data::sheet_extractor::{filtered_column_iter, SheetColumn}, err::{Err, ToUnknownErr}};
use super::{role_actions::Role, Args, Id, Init};

const SHEET_NAME: &str = "Action";
const CLASS_JOB_SHEET_NAME: &str = "ClassJob";

#[derive(Debug)]
enum Input {
    Role(Role),
    ClassJob(Id)
}

#[derive(Debug)]
pub struct Action {
    pub id: u32,
    pub name: String
}

/// Gets the job actions for a given class ID or abbreviation.
pub fn get_job_actions(class_id: Id, args: &mut Args<impl std::io::Write>, names: bool) -> Result<Vec<Action>, Err> {
    get(Input::ClassJob(class_id), args, names)
}

/// Gets the role actions for a given role.
pub fn get_role_actions(role: Role, args: &mut Args<impl std::io::Write>, names: bool) -> Result<Vec<Action>, Err> {
    get(Input::Role(role), args, names)
}

/// Gets all the job actions of a specific job by ID or acronym.
/// Or: Gets all the role actions of a specific role.
fn get(input: Input, args: &mut Args<impl std::io::Write>, names: bool) -> Result<Vec<Action>, Err> {
    let init = Init::new(SHEET_NAME, args)?;
    let mut matches: Vec<Action> = Vec::new();

    match input {
        Input::Role(role) => accumulate_role_actions(role, init, &mut matches),
        Input::ClassJob(id) => accumulate_job_actions(id, init, &mut matches)
    }?;

    if let Some(ref mut out) = args.out {
        if args.pretty_print {
            pretty_print_values(&matches, names, out)
        } else {
            print_values(&matches, names, out);
        }
    }

    Ok(matches)
}

fn accumulate_job_actions(id: Id, init: Init, matches: &mut Vec<Action>) -> Result<(), Err> {
    let (class_id, base_class_id) = get_class_id(id, init.excel, init.version)?;

    let columns: Vec<SheetColumn> = filtered_column_iter(&init.sheet, init.schema, Some(&[CLASS_JOB_SHEET_NAME, "Name"]))?.collect();
    let class_job_column = &columns.iter().find(|x| x.name == CLASS_JOB_SHEET_NAME).to_unknown_err()?.column;
    let name_column = &columns.iter().find(|x| &x.name == "Name").to_unknown_err()?.column;

    for row in init.sheet.into_iter() {
        let class_job_id = row.field(class_job_column).to_unknown_err()?.into_i8().to_unknown_err()?;

        if class_job_id == class_id as i8 || class_job_id == base_class_id as i8 {
            matches.push(Action {
                id: row.row_id(),
                name: row.field(name_column).unwrap().as_string().to_unknown_err()?.to_string()
            });
        }
    }

    Ok(())
}

fn accumulate_role_actions(role: Role, init: Init, matches: &mut Vec<Action>) -> Result<(), Err> {
    let categories = role.get_class_categories();

    let columns: Vec<SheetColumn> = filtered_column_iter(&init.sheet, init.schema, Some(&["ClassJobCategory", "Name"]))?.collect();
    let class_job_column = &columns.iter().find(|x| &x.name == "ClassJobCategory").to_unknown_err()?.column;
    let name_column = &columns.iter().find(|x| &x.name == "Name").to_unknown_err()?.column;

    for row in init.sheet.into_iter() {
        let class_job_id = row.field(class_job_column).to_unknown_err()?.into_u8().to_unknown_err()?;

        if categories.contains(&class_job_id) {
            matches.push(Action {
                id: row.row_id(),
                name: row.field(name_column).unwrap().as_string().to_unknown_err()?.to_string()
            });
        }
    }

    Ok(())
}

fn get_class_id(id: Id, excel: Excel, version: Version) -> Result<(u8, u8), Err> {
    let sheet = excel.sheet(CLASS_JOB_SHEET_NAME).to_unknown_err()?;
    let schema = version.sheet(CLASS_JOB_SHEET_NAME).to_unknown_err()?;
    let columns: Vec<SheetColumn> = filtered_column_iter(&sheet, schema, Some(&["Abbreviation", "ClassJobParent"]))?.collect();
    
    let class_id = match id {
        Id::Index(id) => id,
        Id::Name(abbreviation) => {
            let abbreviation_column = &columns.iter().find(|x| x.name == "Abbreviation").to_unknown_err()?.column;
            sheet.into_iter().find(|x| x.field(abbreviation_column).unwrap().into_string().unwrap().to_string() == abbreviation).ok_or_else(|| Err::JobAcronymNotFound(abbreviation.clone()))?.row_id()
        }
    };

    let base_class_column = &columns.iter().find(|x| x.name == "ClassJobParent").to_unknown_err()?.column;
    let class_job = excel.sheet(CLASS_JOB_SHEET_NAME).to_unknown_err()?.row(class_id).map_err(|_| Err::JobNotFound(class_id))?;

    Ok((class_id as u8, class_job.field(base_class_column).to_unknown_err()?.into_u8().to_unknown_err()?))
}

fn print_values(matches: &[Action], names: bool, out: &mut impl std::io::Write) {
    write!(out, "[").unwrap();

    if names {
        for (i, action) in matches.iter().enumerate() {
            if i != 0 {
                write!(out, ",").unwrap();
            }

            write!(out, "{{\"id\":{},\"name\":{}}}", &action.id, &action.name).unwrap();
        }
    } else {
        for (i, action) in matches.iter().enumerate() {
            if i != 0 {
                write!(out, ",").unwrap();
            }

            write!(out, "{}", action.id).unwrap();
        }
    }

    writeln!(out, "]").unwrap();
}

fn pretty_print_values(matches: &[Action], names: bool, out: &mut impl std::io::Write) {
    writeln!(out, "[").unwrap();

    if names {
        for (i, action) in matches.iter().enumerate() {
            if i != 0 {
                writeln!(out, ",").unwrap();
            }

            write!(out, "  {{ \"id\": {}, \"name\": {} }}", &action.id, &action.name).unwrap();
        }
    } else {
        for (i, action) in matches.iter().enumerate() {
            if i != 0 {
                writeln!(out, ",").unwrap();
            }

            write!(out, "  {}", action.id).unwrap();
        }
    }

    writeln!(out, "\n]").unwrap();
}
