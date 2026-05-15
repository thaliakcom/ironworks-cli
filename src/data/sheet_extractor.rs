use std::borrow::Cow;
use ironworks::excel::{Field, Row};
use ironworks::file::exh::ColumnDefinition;
use ironworks::sestring::SeString;
use crate::SheetData;
use crate::err::{Err, ToUnknownErr};
use super::sheets::{LinkCondition, LinkSource, SHEET_COLUMNS};
use super::{IronworksCli, WritableResult};

impl IronworksCli {
    /// Extracts a single row from the given sheet and prints a
    /// JSON representation of the result to the given output stream.
    pub fn get(&self, sheet: super::sheets::Sheet, id: u32) -> Result<KeyValues<'_>, Err> {
        self.get_values(sheet, id)
    }

    /// Gets a [`Vec`] of the field values and their field names
    /// from the given row in the given sheet.
    ///
    /// Note that this function does not extract _all_ fields. Instead only
    /// the fields specified in `sheets.rs` are extracted.
    fn get_values(&self, sheet: super::sheets::Sheet, row_id: u32) -> Result<KeyValues<'_>, Err> {
        let sheet_name: &'static str = sheet.into();
        let sheet_info = self.get_sheet(sheet_name)?;
        // For some reason calling `sheet.row()` on the Action sheet
        // takes longer than any other sheet by a magnitude of about 4x.
        // Since this is a bug in the dependency, we can't fix it.
        let row = sheet_info.sheet.row(row_id).map_err(|_| Err::RowNotFound(sheet_name, row_id))?;
        let sheet_data = SHEET_COLUMNS.get(sheet_name);
        let filtered_columns: Vec<SheetColumn> = if let Some(sheet_data) = sheet_data {
            sheet_info.filtered_columns(sheet_data.columns)?.collect()
        }  else {
            sheet_info.columns()?.collect()
        };

        let mut result: KeyValues = filtered_columns.iter()
            .map(|column| (Cow::Owned(column.name.to_owned()), row.field(&column.column).unwrap()))
            .collect();

        if let Some(data) = sheet_data {
            for (field_name, field) in self.linked_fields_iter(data, &row, &filtered_columns) {
                result.push((Cow::Owned(field_name), field));
            }
        }

        if let Some(sheet_data) = sheet_data {
            result.sort_by(|(a, _), (b, _)| sheet_data.columns.iter().position(|x| x == a).cmp(&sheet_data.columns.iter().position(|x| x == b)));
        }

        Ok(result)
    }

    fn linked_fields_iter(&self, data: &SheetData, row: &Row, columns: &[SheetColumn]) -> impl Iterator<Item = (String, Field)> {
        data.links.iter().filter_map(|link| {
            if match link.condition {
                LinkCondition::Always => false,
                LinkCondition::Predicate(condition_col, predicate) => !predicate(&row.field(&columns.iter().find(|x| x.name == condition_col).to_unknown_err(21).ok()?.column).ok()?)
            } {
                return None;
            }

            let linked_sheet_info = self.get_sheet(link.sheet).ok()?;
            let linked_row_id = if let LinkSource::Field(column_name) = link.source {
                let column_definition = &columns.iter().find(|x| x.name == column_name).ok_or(Err::ColumnNotFound(data.identifier, column_name)).ok()?.column;
                let value = &row.field(column_definition).ok()?;
                get_u32(value).ok_or(Err::NoIndex(data.identifier, column_name)).ok()?
            } else {
                row.row_id()
            };

            let linked_row = linked_sheet_info.sheet.row(linked_row_id).map_err(|_| Err::RowNotFound(link.sheet, linked_row_id)).ok()?;

            Some(linked_sheet_info.columns().ok()?.filter_map(|column| {
                let link_data = link.columns.iter().find(|x| x.source == column.name);

                if let Some(link_data) = link_data {
                    return Some((link_data.target.to_owned(), linked_row.field(&column.column).to_unknown_err(22).ok()?));
                }

                None
            }).collect::<Vec<_>>())
        }).flatten()
    }

    /// Searches for a given string in the given sheet and prints a list of all matching row IDs
    /// to [`stdout`].
    ///
    /// Note that this function does not search through _all_ columns; instead
    /// only the columns specified in `sheets.rs` are searched.
    pub fn search<'a>(&'a self, sheet: super::sheets::Sheet, search_str: &str, descriptions: bool) -> Result<SearchMatches<'a>, Err> {
        let sheet_name: &'static str = sheet.into();
        let sheet_info = self.get_sheet(sheet_name)?;
        let sheet_data = SHEET_COLUMNS.get(sheet_name).to_unknown_err(23)?;
    
        let mut matches: SearchMatches<'a> = Vec::new();
        let filtered_columns: Vec<SheetColumn> = sheet_info.filtered_columns(sheet_data.columns)?.collect();
        let name_column = filtered_columns.iter().find(|x| x.name == sheet_data.identifier).to_unknown_err(24)?;
        let description_column = filtered_columns.iter().find(|x| x.name == "Description");

        'outer: for row in sheet_info.sheet.into_iter() {
            let row = row?;
            let name = row.field(&name_column.column).to_unknown_err(26)?.into_string().to_unknown_err(25)?;
            
            if !name.to_string().to_lowercase().contains(&search_str.to_lowercase()) {
                continue;
            }

            if descriptions {
                if let Some(description_column) = description_column {
                    let description = row.field(&description_column.column).to_unknown_err(27)?;
                    matches.push(SearchMatch { id: row.row_id(), name, field: Some(KeyValue { key: Cow::Owned(description_column.name.clone()), value: description }) });
                    continue;
                }

                for (field_name, field) in self.linked_fields_iter(sheet_data, &row, &filtered_columns) {
                    if field_name == "Description" {
                        matches.push(SearchMatch { id: row.row_id(), name, field: Some(KeyValue { key: Cow::Owned(field_name), value: field }) });
                        continue 'outer;
                    }
                }
            }

            matches.push(SearchMatch { id: row.row_id(), name, field: None });
        }
    
        Ok(matches)
    }
}

/// The return type of the [`IronworksCli::extract`] function
/// that can also be written to an [`std::io::Write`] stream.
///
/// Internally it's simply a key value vector, similar to a JSON object.
pub type KeyValues<'a> = Vec<(Cow<'a, str>, Field)>;

fn write_uncapitalized<'a>(mut w: impl std::io::Write, text: &Cow<'a, str>) -> std::io::Result<()> {
    if text.chars().take(2).all(|x| x.is_uppercase()) {
        write!(w, "{}", text)?;
        return Ok(());
    }

    let mut iterator = text.chars();
    let first = iterator.next();

    if let Some(first) = first {
        write!(w, "{}", first.to_lowercase())?;

        for char in iterator {
            write!(w, "{}", char)?;
        }

        Ok(())
    } else {
        write!(w, "unknown")
    }
}

impl <'a> WritableResult for KeyValues<'a> {
    fn write(&self, mut w: impl std::io::Write) -> std::io::Result<()> {
        write!(w, "{{")?;
        let len = self.len();
    
        for (i, (key, field)) in self.iter().enumerate() {
            write!(w, "\"")?;
            write_uncapitalized(&mut w, &key)?;
            write!(w, "\":")?;
            field.write(&mut w)?;
    
            if i < len - 1 {
                write!(w, ",")?;
            }
        }
        writeln!(w, "}}")?;
    
        Ok(())
    }

    fn pretty_write(&self, mut w: impl std::io::Write) -> std::io::Result<()> {
        writeln!(w, "{{")?;
        let len = self.len();
    
        for (i, (key, value)) in self.iter().enumerate() {
            write!(w, "  \"")?;
            write_uncapitalized(&mut w, &key)?;
            write!(w, "\": ")?;
            value.pretty_write(&mut w)?;
    
            if i < len - 1 {
                writeln!(w, ",")?;
            }
        }
        writeln!(w, "\n}}")?;
    
        Ok(())
    }
}

/// A key value pair for an Excel field.
#[derive(Debug)]
pub struct KeyValue<'a> {
    pub key: Cow<'a, str>,
    pub value: Field
}

/// A search match that contains references to data that lives
/// within [`IronworksCli`].
#[derive(Debug)]
pub struct SearchMatch<'a> {
    /// The ID (or row index) of the found entity.
    pub id: u32,
    /// The name of the entity as retrieved from its "name" column.
    pub name: SeString<'a>,
    /// A key-value pair of a related column that should be printed
    /// alongside the name.
    pub field: Option<KeyValue<'a>>
}

/// The matches of the [`IronworksCli::search()`] function.
pub type SearchMatches<'a> = Vec<SearchMatch<'a>>;

impl <'a> WritableResult for SearchMatches<'a> {
    fn write(&self, w: impl std::io::Write) -> std::io::Result<()> {
        self.pretty_write(w)
    }

    fn pretty_write(&self, mut w: impl std::io::Write) -> std::io::Result<()> {
        if self.is_empty() {
            writeln!(w, "No matches found")?;
        } else {
            writeln!(w, "{} matches found:", self.len())?;
    
            for SearchMatch { id, name, field } in self.iter() {
                write!(w, "  at {: >5}: ", id)?;
                name.pretty_write(&mut w)?;
    
                if let Some(key_value) = field {
                    write!(w, " -> {{ \"{}\": ", key_value.key)?;
                    key_value.value.pretty_write(&mut w)?;
                    write!(w, " }}")?;
                }
    
                writeln!(w, )?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct SheetColumn {
    pub name: String,
    pub column: ColumnDefinition
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
