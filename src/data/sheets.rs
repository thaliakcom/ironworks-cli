use ironworks::excel::Field;
use phf::phf_map;
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr, EnumIter)]
pub enum Sheet {
    Action,
    Status,
    ContentFinderCondition,
    ClassJob,
    ClassJobCategory,
    BNpcBase,
    ModelSkeleton,
    Lockon,
    Channeling
}

impl Sheet {
    /// The version of this sheet's JSON output, which every row starts with as `schemaVersion`.
    /// See [`SheetData::schema_version`].
    pub fn schema_version(&self) -> u16 {
        SHEET_COLUMNS.get(<&'static str>::from(*self)).expect("every sheet has an entry in SHEET_COLUMNS").schema_version
    }
}

/// The name and [`Sheet::schema_version`] of every sheet.
pub fn schema_versions() -> Vec<(&'static str, u16)> {
    Sheet::iter().map(|sheet| (sheet.into(), sheet.schema_version())).collect()
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
    Always,
    /// Condition evaluates to `true` if the given column in the source row satisfies the condition.
    Predicate(&'static str, fn(&Field) -> bool)
}

/// Data for a sheet.
pub(crate) struct SheetData {
    /// Name of the column that contains an identifier for the column.
    /// Empty if the sheet has no name column and can therefore not be searched.
    pub identifier: &'static str,
    /// Which columns to add to the output. An empty array indicates that all columns should be used, except those with Unknown in their name.
    pub columns: &'static [&'static str],
    /// Whether data from another sheet should be added to the output.
    pub links: &'static [SheetLink],
    /// The version of this sheet's JSON output. Consumers cache rows and refetch those whose `schemaVersion` is
    /// older than this, so it **must** be incremented whenever the JSON output of this sheet can change for any row:
    /// columns, links, derived fields (such as `AnimationEndDuration` in `sheet_extractor.rs`) or how values are written.
    /// A change to output logic shared by all sheets (such as `KeyValues::write`, key casing or ordering) increments
    /// every sheet. At most one increment per commit; the `schema_versions_track_output` test enforces this for columns
    /// and links. A change may only add fields or change values: consumers also accept rows newer than their own
    /// version, so removing or renaming a field needs every consumer (thaliak, thaliak-timeline-linter,
    /// thaliak-diagram-editor) updated as well. See `AGENTS.md`.
    pub schema_version: u16
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
                source: LinkSource::Field("AnimationEnd"),
                sheet: "ActionTimeline",
                columns: &[SheetLinkColumn { source: "Key", target: "AnimationEndKey" }],
                condition: LinkCondition::Predicate("AnimationEnd", |x| x.as_u16().is_some_and(|v| *v != 0) || x.as_i16().is_some_and(|v| *v != 0))
            },
            SheetLink {
                source: LinkSource::Field("ActionTimelineHit"),
                sheet: "ActionTimeline",
                columns: &[SheetLinkColumn { source: "Key", target: "ActionTimelineHitKey" }],
                condition: LinkCondition::Predicate("ActionTimelineHit", |x| x.as_u16().is_some_and(|v| *v != 0) || x.as_i16().is_some_and(|v| *v != 0))
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
        ],
        schema_version: 8
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
        links: &[],
        schema_version: 3
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
        links: &[],
        schema_version: 3
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
        links: &[],
        schema_version: 2
    },
    "ClassJobCategory" => SheetData {
        identifier: "Name",
        columns: &[],
        links: &[],
        schema_version: 1
    },
    "BNpcBase" => SheetData {
        identifier: "",
        columns: &[
            "Scale",
            "ModelChara",
            "IsOmnidirectional"
        ],
        links: &[
            SheetLink {
                source: LinkSource::Field("ModelChara"),
                sheet: "ModelChara",
                columns: &[
                    SheetLinkColumn { source: "Radius", target: "ModelCharaRadius" },
                    SheetLinkColumn { source: "Model", target: "ModelCharaModel" },
                    SheetLinkColumn { source: "Type", target: "ModelCharaType" }
                ],
                condition: LinkCondition::Always
            }
        ],
        schema_version: 1
    },
    "ModelSkeleton" => SheetData {
        identifier: "",
        columns: &[
            "Radius",
            "Height",
            "VFXScale"
        ],
        links: &[],
        schema_version: 1
    },
    "Lockon" => SheetData {
        identifier: "",
        columns: &[
            "IconName"
        ],
        links: &[],
        schema_version: 1
    },
    "Channeling" => SheetData {
        identifier: "",
        columns: &[
            "File"
        ],
        links: &[],
        schema_version: 1
    }
};

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;
    use std::process::Command;
    use serde_json::{json, Value};
    use super::*;

    /// The committed record of each sheet's schema version and output keys, which the test regenerates.
    const BASELINE: &str = "schema-versions.json";

    /// What decides a sheet's output keys, as a stable string.
    fn descriptor(data: &SheetData) -> String {
        let links: Vec<String> = data.links.iter().map(|link| {
            let source = match link.source {
                LinkSource::ID => "ID".to_string(),
                LinkSource::Field(field) => format!("Field({field})")
            };
            let columns: Vec<String> = link.columns.iter().map(|column| format!("{} -> {}", column.source, column.target)).collect();

            format!("{source} {}: {}", link.sheet, columns.join(", "))
        }).collect();

        format!("identifier: {}; columns: {}; links: {}", data.identifier, data.columns.join(", "), links.join("; "))
    }

    /// The baseline as of the last commit, or `None` if there is none (or git isn't available).
    fn committed_baseline(root: &Path) -> Option<Value> {
        let output = Command::new("git").args(["show", &format!("HEAD:{BASELINE}")]).current_dir(root).output().ok()?;

        if !output.status.success() {
            return None;
        }

        serde_json::from_slice(&output.stdout).ok()
    }

    /// Compares every sheet's columns and links with the last commit: a change needs exactly one increment of
    /// `schema_version`, an increment without one is allowed (for value changes), and there is never more than one
    /// increment per commit, so that iterating on uncommitted changes doesn't force a second one.
    /// Then writes the current state to the baseline, which is committed with the change.
    #[test]
    fn schema_versions_track_output() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let committed = committed_baseline(root);
        let mut current = BTreeMap::new();
        let mut problems = Vec::new();

        for (name, version) in schema_versions() {
            let keys = descriptor(SHEET_COLUMNS.get(name).unwrap());

            if let Some(committed) = &committed {
                match committed.get(name) {
                    // A sheet added since the last commit starts at 1.
                    None if version != 1 => problems.push(format!("{name} is new: its schema_version must be 1, not {version}")),
                    None => (),
                    Some(entry) => {
                        let last = entry["version"].as_u64().unwrap() as u16;
                        let changed = entry["keys"].as_str() != Some(keys.as_str());

                        if version < last {
                            problems.push(format!("{name}: schema_version {version} is lower than the committed {last}"));
                        } else if version > last + 1 {
                            problems.push(format!("{name}: schema_version {version} is more than one above the committed {last}; set it to {}", last + 1));
                        } else if changed && version == last {
                            problems.push(format!("{name}: its columns or links changed; bump its schema_version to {}", last + 1));
                        }
                    }
                }
            }

            current.insert(name, json!({ "version": version, "keys": keys }));
        }

        std::fs::write(root.join(BASELINE), serde_json::to_string_pretty(&current).unwrap() + "\n").unwrap();

        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }
}
