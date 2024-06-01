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
    pub columns: &'static [SheetLinkColumn]
}

/// Data for a sheet.
pub struct SheetData {
    /// Which columns to add to the output.
    pub columns: &'static [&'static str],
    /// Whether data from another sheet should be added to the output.
    pub links: &'static [SheetLink],
    /// Which columns to search in.
    pub search_columns: &'static [&'static str]
}

pub static SHEET_COLUMNS: phf::Map<&'static str, SheetData> = phf_map! {
    "Action" => SheetData {
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
            "Action{Combo}",
            "PreservesCombo",
            "Cast<100ms>",
            "Recast<100ms>",
            "CooldownGroup",
            "MaxCharges",
            "AttackType",
            "Aspect",
            "IsPlayerAction"
        ],
        search_columns: &[
            "Name"
        ],
        links: &[
            SheetLink {
                source: LinkSource::ID,
                sheet: "ActionTransient",
                columns: &[SheetLinkColumn { source: "Description", target: "Description" }]
            }
        ]
    },
    "Status" => SheetData {
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
            "IsPermanent"
        ],
        search_columns: &[
            "Name",
            "Description"
        ],
        links: &[]
    },
    "ContentFinderCondition" => SheetData {
        columns: &[
            "TerritoryType",
            "ClassJobLevel{Required}",
            "ClassJobLevel{Sync}",
            "ItemLevel{Required}",
            "ItemLevel{Sync}",
            "AllowUndersized",
            "AllowExplorerMode",
            "HighEndDuty",
            "Name",
            "NameShort",
            "ContentType",
            "Image",
            "Icon"
        ],
        search_columns: &[
            "Name"
        ],
        links: &[]
    }
};
