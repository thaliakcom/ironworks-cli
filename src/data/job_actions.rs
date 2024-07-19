use ironworks::excel::{Excel, Field};
use ironworks_schema::{saint_coinach::Version, Node, Schema};
use crate::{data::sheet_extractor::print_value, err::{Err, ToUnknownErr}};
use super::{role_actions::Role, Args, Id, Init};

const SHEET_NAME: &str = "Action";
const CLASS_JOB_SHEET_NAME: &str = "ClassJob";

#[derive(Debug)]
pub enum Input {
    Role(Role),
    ClassJob(Id)
}

/// Gets all the job actions of a specific job by ID or acronym.
/// Or: Gets all the role actions of a specific role.
pub fn get(input: &Input, args: &mut Args<impl std::io::Write>, names: bool, pretty_print: bool) -> Result<(), Err> {
    let init = Init::new(SHEET_NAME, args)?;
    let mut matches: Vec<Field> = Vec::new();

    match input {
        Input::Role(role) => get_role_actions(*role, init, &mut matches, names),
        Input::ClassJob(id) => get_job_actions(id, init, &mut matches, names)
    }?;

    if pretty_print {
        pretty_print_values(matches, names, &mut args.out)
    } else {
        print_values(matches, names, &mut args.out);
    }

    Ok(())
}

fn get_job_actions(id: &Id, init: Init, matches: &mut Vec<Field>, names: bool) -> Result<(), Err> {
    let (class_id, base_class_id) = get_class_id(id, init.excel, init.version)?;
    let columns = if let Node::Struct(columns) = &init.schema.node { Ok(columns) } else { Err(Err::UnsupportedSheet(SHEET_NAME)) }?;
    let class_job_column = columns.iter().find(|x| x.name == CLASS_JOB_SHEET_NAME).to_unknown_err()?.offset as usize;
    let name_column = if names { columns.iter().find(|x| x.name == "Name").to_unknown_err()?.offset as usize } else { 0 };

    for row in init.sheet.iter() {
        let class_job_id = row.field(class_job_column).to_unknown_err()?.into_i8().to_unknown_err()?;

        if class_job_id == class_id as i8 || class_job_id == base_class_id as i8 {
            matches.push(Field::U32(row.row_id()));

            if names {
                matches.push(row.field(name_column).to_unknown_err()?);
            }
        }
    }

    Ok(())
}

fn get_role_actions(role: Role, init: Init, matches: &mut Vec<Field>, names: bool) -> Result<(), Err> {
    let categories = role.get_class_categories();
    let columns = if let Node::Struct(columns) = &init.schema.node { Ok(columns) } else { Err(Err::UnsupportedSheet(SHEET_NAME)) }?;
    let class_job_column = columns.iter().find(|x| x.name == "ClassJobCategory").to_unknown_err()?.offset as usize;
    let name_column = if names { columns.iter().find(|x| x.name == "Name").to_unknown_err()?.offset as usize } else { 0 };

    for row in init.sheet.iter() {
        let class_job_id = row.field(class_job_column).to_unknown_err()?.into_u8().to_unknown_err()?;

        if categories.contains(&class_job_id) {
            matches.push(Field::U32(row.row_id()));

            if names {
                matches.push(row.field(name_column).unwrap());
            }
        }
    }

    Ok(())
}

fn get_class_id(id: &Id, excel: Excel<'_>, version: Version) -> Result<(u8, u8), Err> {
    let class_jobs = excel.sheet(CLASS_JOB_SHEET_NAME).to_unknown_err()?;
    let schema = version.sheet(CLASS_JOB_SHEET_NAME).to_unknown_err()?;
    
    if let Node::Struct(columns) = schema.node {
        let class_id = match id {
            Id::Index(id) => *id,
            Id::Name(abbreviation) => {
                let abbreviation_column = columns.iter().find(|x| x.name == "Abbreviation").to_unknown_err()?.offset as usize;
                class_jobs.iter().find(|x| &x.field(abbreviation_column).unwrap().into_string().unwrap().to_string() == abbreviation).ok_or_else(|| Err::JobAcronymNotFound(abbreviation.clone()))?.row_id()
            }
        };

        let base_class_column = columns.iter().find(|x| x.name == "ClassJob{Parent}").to_unknown_err()?.offset as usize;
        let class_job = class_jobs.row(class_id).map_err(|_| Err::JobNotFound(class_id))?;

        Ok((class_id as u8, class_job.field(base_class_column).to_unknown_err()?.into_u8().to_unknown_err()?))
    } else {
        Err(Err::UnsupportedSheet(SHEET_NAME))
    }
}

fn print_values(matches: Vec<Field>, names: bool, out: &mut impl std::io::Write) {
    write!(out, "[").unwrap();

    if names {
        for (i, m) in matches.chunks_exact(2).enumerate() {
            if i != 0 {
                write!(out, ",").unwrap();
            }

            write!(out, "{{\"id\":").unwrap();
            print_value(out, &m[0]);
            write!(out, ",\"name\":").unwrap();
            print_value(out, &m[1]);
            write!(out, "}}").unwrap();
        }
    } else {
        for (i, index) in matches.iter().enumerate() {
            if i != 0 {
                write!(out, ",").unwrap();
            }

            print_value(out, index);
        }
    }

    writeln!(out, "]").unwrap();
}

fn pretty_print_values(matches: Vec<Field>, names: bool, out: &mut impl std::io::Write) {
    writeln!(out, "[").unwrap();

    if names {
        for (i, m) in matches.chunks_exact(2).enumerate() {
            if i != 0 {
                writeln!(out, ",").unwrap();
            }

            write!(out, "  {{ \"id\": ").unwrap();
            print_value(out, &m[0]);
            write!(out, ", \"name\": ").unwrap();
            print_value(out, &m[1]);
            write!(out, " }}").unwrap();
        }
    } else {
        for (i, index) in matches.iter().enumerate() {
            if i != 0 {
                writeln!(out, ",").unwrap();
            }

            write!(out, "  ").unwrap();
            print_value(out, index);
        }
    }

    writeln!(out, "\n]").unwrap();
}
