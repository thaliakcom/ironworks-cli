use ironworks::excel::Field;
use phf::phf_map;
use strum::IntoStaticStr;

#[derive(Debug, Clone, PartialEq, Eq, IntoStaticStr)]
pub enum Sheet {
    Action,
    Status,
    ContentFinderCondition
}

/// A source for a [`SheetLink`], i.e. the type of column in the source
/// sheet that is used to join to another sheet.
pub(crate) enum LinkSource {
    /// The [`SheetLink`] links to another sheet directly by cross-referencing both IDs.
    ID,
    /// The [`SheetLink`] links to another sheet by cross-referencing the column defined
    /// by this variant in the source sheet with the ID of the target sheet.
    #[allow(dead_code)]
    Field(&'static str)
}

/// Represents which columns in the linked sheet are used.
pub(crate) struct SheetLinkColumn {
    /// The name of a column in the target sheet to print.
    pub source: &'static str,
    /// What this field should be called in the output.
    pub target: &'static str
}

/// Represents a link to another sheet.
pub(crate) struct SheetLink {
    /// The link source. Describes how the two sheets should be connected.
    pub source: LinkSource,
    /// The sheet to link to.
    pub sheet: &'static str,
    /// Which columns in the linked sheet are used, and whether they should be aliased.
    pub columns: &'static [SheetLinkColumn],
    /// Whether this link should be established at all.
    pub condition: LinkCondition
}

pub(crate) enum LinkCondition {
    /// Condition always evaluates to `true`.
    #[allow(dead_code)]
    Always,
    /// Condition evaluates to `true` if the given column in the source row satisfies the condition.
    Predicate(&'static str, fn(&Field) -> bool)
}

/// Data for a sheet.
pub(crate) struct SheetData {
    /// Name of the column that contains an identifier for the column.
    pub identifier: &'static str,
    /// Which columns to add to the output.
    pub columns: &'static [&'static str],
    /// Whether data from another sheet should be added to the output.
    pub links: &'static [SheetLink],
    /// Which columns to search in.
    pub search_columns: &'static [&'static str]
}

pub(crate) static SHEET_COLUMNS: phf::Map<&'static str, SheetData> = phf_map! {
    "Action" => SheetData {
        identifier: "Name",
        columns: &[
            "Name",
            "Icon",
            "ActionCategory",
            "ClassJob",
            "ClassJobLevel",
            "IsRoleAction",
            "CanTargetSelf",
            "CanTargetParty",
            "CanTargetFriendly",
            "CanTargetHostile",
            "CanTargetDead",
            "TargetArea",
            "CastType",
            "BehaviourType",
            "Range",
            "EffectRange",
            "ActionCombo",
            "PreservesCombo",
            "Cast100ms",
            "Recast100ms",
            "CooldownGroup",
            "MaxCharges",
            "AttackType",
            "Aspect",
            "ClassJobCategory",
            "IsPlayerAction"
        ],
        search_columns: &[
            "Name"
        ],
        links: &[
            SheetLink {
                source: LinkSource::ID,
                sheet: "ActionTransient",
                columns: &[SheetLinkColumn { source: "Description", target: "Description" }],
                condition: LinkCondition::Predicate("ClassJob", |x| *x.as_i8().unwrap() != -1)
            }
        ]
    },
    "Status" => SheetData {
        identifier: "Name",
        columns: &[
            "Name",
            "Description",
            "Icon",
            "MaxStacks",
            "StatusCategory",
            "HitEffect",
            "Transfiguration",
            "IsGaze",
            "CanDispel",
            "InflictedByActor",
            "IsPermanent",
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
            "Name",
            "NameShort",
            "TerritoryType",
            "ClassJobLevelRequired",
            "ClassJobLevelSync",
            "ItemLevelRequired",
            "ItemLevelSync",
            "AllowUndersized",
            "AllowExplorerMode",
            "HighEndDuty",
            "ShortCode",
            "ContentType",
            "Image",
            "Icon"
        ],
        search_columns: &[
            "Name",
            "ShortCode"
        ],
        links: &[]
    }
};
