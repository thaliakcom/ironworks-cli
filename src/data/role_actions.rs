#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[clap(rename_all = "kebab_case")]
pub enum Role {
    Tank,
    Healer,
    Melee,
    PhysicalRanged,
    Caster
}

impl Role {
    pub fn get_class_categories(&self) -> &[u8] {
        match self {
            Role::Tank => &[113, 161],
            Role::Healer => &[117, 120],
            Role::Melee => &[114, 161, 118],
            Role::PhysicalRanged => &[115, 161, 118],
            Role::Caster => &[116, 120]
        }
    }
}
