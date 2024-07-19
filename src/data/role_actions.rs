use crate::err::Err;
use super::Args;

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

pub fn get(role: Role, args: &mut Args<impl std::io::Write>, names: bool, pretty_print: bool) -> Result<(), Err> {
    super::job_actions::get(&super::job_actions::Input::Role(role), args, names, pretty_print)
}
