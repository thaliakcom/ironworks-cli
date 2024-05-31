use phf::phf_map;

pub enum SheetLinkTarget {
    ID,
    Field(&'static str)
}

pub struct SheetLinkColumn {
    pub source: &'static str,
    pub target: &'static str
}

pub struct SheetLink {
    pub target: SheetLinkTarget,
    pub sheet: &'static str,
    pub columns: &'static [SheetLinkColumn]
}

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
                target: SheetLinkTarget::ID,
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
