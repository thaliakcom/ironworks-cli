use std::ops::{Deref, DerefMut};

use crate::{data::sheet_extractor::SheetColumn, err::{Err, ToUnknownErr}};
use super::{role_actions::Role, Id, IronworksCli, WritableResult};

const SHEET_NAME: &str = "Action";
const CLASS_JOB_SHEET_NAME: &str = "ClassJob";

/// Represents an action, as returned by [`IronworksCli::get_job_actions()`] or [`IronworksCli::get_role_actions()`].
/// Use [`IronworksCli::get()`] to retrieve additional information about the action.
#[derive(Debug)]
pub struct Action {
    pub id: u32,
    pub name: String
}

/// Represents a list of actions returned by the [`IronworksCli::get_job_actions()`]
/// or [`IronworksCli::get_role_actions()`] functions.
#[derive(Debug)]
pub struct Actions(Vec<Action>);

/// Same as [`Actions`], but configured to be written to a [`std::io::Write`] instance.
#[derive(Debug)]
pub struct WritableActions<'a> {
    names: bool,
    actions: &'a [Action]
}

impl Deref for Actions {
    type Target = [Action];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Actions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Actions {
    /// Gets a formatted representation of the contained actions, configured to be
    /// written to a [`std::io::Write`] instance.
    pub fn writable(&self, names: bool) -> WritableActions<'_> {
        WritableActions { names, actions: &self.0 }
    }
}

impl IronworksCli {
    /// Gets the job actions for a given class ID or abbreviation.
    pub fn get_job_actions(&self, class_id: Id) -> Result<Actions, Err> {
        let mut matches: Vec<Action> = Vec::new();
        let sheet_info = self.get_sheet(SHEET_NAME)?;
        let (class_id, base_class_id) = self.get_class_id(class_id)?;
    
        let columns: Vec<SheetColumn> = sheet_info.filtered_columns(&[CLASS_JOB_SHEET_NAME, "Name"])?.collect();
        let class_job_column = &columns.iter().find(|x| x.name == CLASS_JOB_SHEET_NAME).to_unknown_err()?.column;
        let name_column = &columns.iter().find(|x| &x.name == "Name").to_unknown_err()?.column;
    
        for row in sheet_info.sheet.into_iter() {
            let class_job_id = row.field(class_job_column).to_unknown_err()?.into_i8().to_unknown_err()?;
    
            if class_job_id == class_id as i8 || class_job_id == base_class_id as i8 {
                matches.push(Action {
                    id: row.row_id(),
                    name: row.field(name_column).unwrap().as_string().to_unknown_err()?.to_string()
                });
            }
        }
    
        Ok(Actions(matches))
    }
    
    /// Gets the role actions for a given role.
    pub fn get_role_actions(&self, role: Role) -> Result<Actions, Err> {
        let mut matches: Vec<Action> = Vec::new();
        let sheet_info = self.get_sheet(SHEET_NAME)?;
        let categories = role.get_class_categories();
    
        let columns: Vec<SheetColumn> = sheet_info.filtered_columns(&["ClassJobCategory", "Name"])?.collect();
        let class_job_column = &columns.iter().find(|x| &x.name == "ClassJobCategory").to_unknown_err()?.column;
        let name_column = &columns.iter().find(|x| &x.name == "Name").to_unknown_err()?.column;
    
        for row in sheet_info.sheet.into_iter() {
            let class_job_id = row.field(class_job_column).to_unknown_err()?.into_u8().to_unknown_err()?;
    
            if categories.contains(&class_job_id) {
                matches.push(Action {
                    id: row.row_id(),
                    name: row.field(name_column).unwrap().as_string().to_unknown_err()?.to_string()
                });
            }
        }
    
        Ok(Actions(matches))
    }

    fn get_class_id(&self, id: Id) -> Result<(u8, u8), Err> {
        let sheet_info = self.get_sheet(CLASS_JOB_SHEET_NAME)?;
        let columns: Vec<SheetColumn> = sheet_info.filtered_columns(&["Abbreviation", "ClassJobParent"])?.collect();
        
        let class_id = match id {
            Id::Index(id) => id,
            Id::Name(abbreviation) => {
                let abbreviation_column = &columns.iter().find(|x| x.name == "Abbreviation").to_unknown_err()?.column;
                self.sheet_iter(CLASS_JOB_SHEET_NAME)?
                    .find(|x| x.field(abbreviation_column).unwrap().into_string().unwrap().to_string() == abbreviation)
                    .ok_or_else(|| Err::JobAcronymNotFound(abbreviation.clone()))?.row_id()
            }
        };
    
        let base_class_column = &columns.iter().find(|x| x.name == "ClassJobParent").to_unknown_err()?.column;
        let class_job = sheet_info.sheet.row(class_id).map_err(|_| Err::JobNotFound(class_id))?;
    
        Ok((class_id as u8, class_job.field(base_class_column).to_unknown_err()?.into_u8().to_unknown_err()?))
    }
}

impl <'a> WritableResult for WritableActions<'a> {
    fn write(&self, mut w: impl std::io::Write) -> std::io::Result<()> {
        write!(w, "[")?;
    
        if self.names {
            for (i, action) in self.actions.iter().enumerate() {
                if i != 0 {
                    write!(w, ",")?;
                }
    
                write!(w, "{{\"id\":{},\"name\":{}}}", &action.id, &action.name)?;
            }
        } else {
            for (i, action) in self.actions.iter().enumerate() {
                if i != 0 {
                    write!(w, ",")?;
                }
    
                write!(w, "{}", action.id)?;
            }
        }
    
        writeln!(w, "]")?;

        Ok(())
    }
    
    fn pretty_write(&self, mut w: impl std::io::Write) -> std::io::Result<()> {
        writeln!(w, "[")?;
    
        if self.names {
            for (i, action) in self.actions.iter().enumerate() {
                if i != 0 {
                    writeln!(w, ",")?;
                }
    
                write!(w, "  {{ \"id\": {}, \"name\": {} }}", &action.id, &action.name)?;
            }
        } else {
            for (i, action) in self.actions.iter().enumerate() {
                if i != 0 {
                    writeln!(w, ",")?;
                }
    
                write!(w, "  {}", action.id)?;
            }
        }
    
        writeln!(w, "\n]")?;

        Ok(())
    }
}
