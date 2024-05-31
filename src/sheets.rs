use phf::phf_map;

pub static SHEET_COLUMNS: phf::Map<&'static str, &'static [&'static str]> = phf_map! {
    "Action" => &[
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
    "Status" => &[
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
    "ContentFinderCondition" => &[
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
    ]
};
