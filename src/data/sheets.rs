use ironworks::excel::Field;
use phf::phf_map;

/// A source for a [`SheetLink`], i.e. the type of column in the source
/// sheet that is used to join to another sheet.
pub enum LinkSource {
    /// The [`SheetLink`] links to another sheet directly by cross-referencing both IDs.
    ID,
    /// The [`SheetLink`] links to another sheet by cross-referencing the column defined
    /// by this variant in the source sheet with the ID of the target sheet.
    #[allow(dead_code)]
    Field(&'static str)
}

pub enum Column {
    AsIs(&'static str),
    Alias(&'static str, &'static str)
}

impl Column {
    pub fn name(&self) -> &str {
        match self {
            Column::AsIs(s) => s,
            Column::Alias(s, ..) => s
        }
    }
}

/// Represents which columns in the linked sheet are used.
pub struct SheetLinkColumn {
    /// The name of a column in the target sheet to print.
    pub source: &'static str,
    /// What this field should be called in the output.
    pub target: &'static str
}

/// Represents a link to another sheet.
pub struct SheetLink {
    /// The link source. Describes how the two sheets should be connected.
    pub source: LinkSource,
    /// The sheet to link to.
    pub sheet: &'static str,
    /// Which columns in the linked sheet are used, and whether they should be aliased.
    pub columns: &'static [SheetLinkColumn],
    /// Whether this link should be established at all.
    pub condition: LinkCondition
}

pub enum LinkCondition {
    /// Condition always evaluates to `true`.
    Always,
    #[allow(dead_code)]
    /// Condition evaluates to `true` if the given column in the source row does _not_ contain the value.
    IfNot(&'static str, Field)
}

/// Data for a sheet.
pub struct SheetData {
    /// Name of the column that contains an identifier for the column.
    pub identifier: &'static str,
    /// Which columns to add to the output.
    pub columns: &'static [Column],
    /// Whether data from another sheet should be added to the output.
    pub links: &'static [SheetLink],
    /// Which columns to search in.
    pub search_columns: &'static [&'static str]
}

pub static SHEET_COLUMNS: phf::Map<&'static str, SheetData> = phf_map! {
    "Action" => SheetData {
        identifier: "Name",
        columns: &[
            Column::AsIs("Name"),
            Column::AsIs("Icon"),
            Column::AsIs("ActionCategory"),
            Column::AsIs("ClassJob"),
            Column::AsIs("ClassJobLevel"),
            Column::AsIs("IsRoleAction"),
            Column::AsIs("CanTargetSelf"),
            Column::AsIs("CanTargetParty"),
            Column::AsIs("CanTargetFriendly"),
            Column::AsIs("CanTargetHostile"),
            Column::AsIs("CanTargetDead"),
            Column::AsIs("TargetArea"),
            Column::AsIs("CastType"),
            Column::AsIs("BehaviourType"),
            Column::AsIs("Range"),
            Column::AsIs("EffectRange"),
            Column::Alias("Action{Combo}", "ActionCombo"),
            Column::AsIs("PreservesCombo"),
            Column::Alias("Cast<100ms>", "Cast100ms"),
            Column::Alias("Recast<100ms>", "Recast100ms"),
            Column::AsIs("CooldownGroup"),
            Column::AsIs("MaxCharges"),
            Column::AsIs("AttackType"),
            Column::AsIs("Aspect"),
            Column::AsIs("ClassJobCategory"),
            Column::AsIs("IsPlayerAction")
        ],
        search_columns: &[
            "Name"
        ],
        links: &[
            SheetLink {
                source: LinkSource::ID,
                sheet: "ActionTransient",
                columns: &[SheetLinkColumn { source: "Description", target: "Description" }],
                condition: LinkCondition::Always
            }
        ]
    },
    "Status" => SheetData {
        identifier: "Name",
        columns: &[
            Column::AsIs("Name"),
            Column::AsIs("Description"),
            Column::AsIs("Icon"),
            Column::AsIs("MaxStacks"),
            Column::AsIs("StatusCategory"),
            Column::AsIs("HitEffect"),
            Column::AsIs("Transfiguration"),
            Column::AsIs("IsGaze"),
            Column::AsIs("CanDispel"),
            Column::AsIs("InflictedByActor"),
            Column::AsIs("IsPermanent"),
        ],
        search_columns: &[
            "Name",
            "Description"
        ],
        links: &[]
    },
    "ContentFinderCondition" => SheetData {
        identifier: "Name",
        columns: &[
            Column::AsIs("TerritoryType"),
            Column::Alias("ClassJobLevel{Required}", "ClassJobLevelRequired"),
            Column::Alias("ClassJobLevel{Sync}", "ClassJobLevelSync"),
            Column::Alias("ItemLevel{Required}", "ItemLevelRequired"),
            Column::Alias("ItemLevel{Sync}", "ItemLevelSync"),
            Column::AsIs("AllowUndersized"),
            Column::AsIs("AllowExplorerMode"),
            Column::AsIs("HighEndDuty"),
            Column::AsIs("Name"),
            Column::AsIs("NameShort"),
            Column::AsIs("ShortCode"),
            Column::AsIs("ContentType"),
            Column::AsIs("Image"),
            Column::AsIs("Icon")
        ],
        search_columns: &[
            "Name",
            "ShortCode"
        ],
        links: &[]
    }
};

#[cfg(test)]
mod tests {
    use ironworks_schema::Node;
    use super::super::Init;
    use super::*;

    #[test]
    fn sheets_data_valid() {
        let mut non_matching_columns: Vec<String> = Vec::new();

        for (sheet_name, data) in SHEET_COLUMNS.entries() {
            let (schema, ..) = Init::get_schema(sheet_name, "2024.03.27.0000.0000").unwrap();

            if let Node::Struct(columns) = schema.node {
                let column_names: Vec<_> = columns.iter().map(|x| x.name.to_owned()).collect();

                for column in data.columns {
                    let name = column.name();
                    if !column_names.iter().any(|y| y == name) {
                        non_matching_columns.push(format!("{}::{}", sheet_name, name));
                    }
                }
            } else {
                panic!("Schema {} of incompatible type", sheet_name);
            }
        }

        assert!(non_matching_columns.is_empty(), "Columns {:#?} are defined in sheets.rs but do not actually exist.", non_matching_columns);
    }
}
