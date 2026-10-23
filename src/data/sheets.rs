use ironworks::excel::Field;
use phf::phf_map;
use strum::IntoStaticStr;

#[derive(Debug, Clone, PartialEq, Eq, IntoStaticStr)]
pub enum Sheet {
    Action,
    Status,
    ContentFinderCondition,
    ClassJob,
    ClassJobCategory
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
    /// Which columns to add to the output. An empty array indicates that all columns should be used, except those with Unknown in their name.
    pub columns: &'static [&'static str],
    /// Whether data from another sheet should be added to the output.
    pub links: &'static [SheetLink]
}

pub(crate) static SHEET_COLUMNS: phf::Map<&'static str, SheetData> = phf_map! {
    "Action" => SheetData {
        identifier: "Name",
        columns: &[
            "Name",
            "Description",
            "Icon",
            "ActionCategory",
            "Cast100ms",
            "Recast100ms",
            "MaxCharges",
            "Range",
            "EffectRange",
            "AttackType",
            "Aspect",
            "ClassJob",
            "ClassJobLevel",
            "ClassJobCategory",
            "IsRoleAction",
            "CanTargetSelf",
            "CanTargetParty",
            "CanTargetFriendly",
            "CanTargetHostile",
            "CanTargetDead",
            "TargetArea",
            "XAxisModifier",
            "CastType",
            "BehaviourType",
            "ActionCombo",
            "PreservesCombo",
            "CooldownGroup",
            "IsPlayerAction",
            "ExtraCastTime100ms",
            "AnimationStart",
            "AnimationEnd",
            "ActionTimelineHit"
        ],
        links: &[
            SheetLink {
                source: LinkSource::ID,
                sheet: "ActionTransient",
                columns: &[SheetLinkColumn { source: "Description", target: "Description" }],
                condition: LinkCondition::Predicate("ClassJob", |x| *x.as_i8().unwrap() != -1)
            },
            SheetLink {
                source: LinkSource::Field("Omen"),
                sheet: "Omen",
                columns: &[
                    SheetLinkColumn { source: "Path", target: "OmenPath" },
                    SheetLinkColumn { source: "Type", target: "OmenType" }
                ],
                condition: LinkCondition::Predicate("Omen", |x| *x.as_u16().unwrap() != 0)
            },
            SheetLink {
                source: LinkSource::Field("OmenAlt"),
                sheet: "Omen",
                columns: &[
                    SheetLinkColumn { source: "Path", target: "OmenAltPath" },
                    SheetLinkColumn { source: "Type", target: "OmenAltType" }
                ],
                condition: LinkCondition::Predicate("OmenAlt", |x| *x.as_u16().unwrap() != 0)
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
        links: &[]
    },
    "ClassJob" => SheetData {
        identifier: "Name",
        columns: &[
            "Name",
            "Abbreviation",
            "ClassJobCategory",
            "Role",
            "StartingLevel",
            "PartyBonus",
            "JobType",
            "IsLimitedJob",
            "CanQueueForDuty",
            "ModifierHitPoints",
            "ModifierManaPoints",
            "ModifierStrength",
            "ModifierVitality",
            "ModifierDexterity",
            "ModifierIntelligence",
            "ModifierMind",
            "ModifierPiety",
            "ClassJobParent",
            "PrimaryStat",
            "LimitBreak1",
            "LimitBreak2",
            "LimitBreak3"
        ],
        links: &[]
    },
    "ClassJobCategory" => SheetData {
        identifier: "Name",
        columns: &[],
        links: &[]
    }
};
