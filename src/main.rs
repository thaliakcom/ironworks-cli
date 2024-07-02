use std::process::ExitCode;
use crate::err::Err;

use clap::Parser;
use cli::{Cli, Command, IconArgs, Id, JobActionsCommandArgs, RoleActionsCommandArgs, SheetCommandArgs};
use data::job_actions;

mod cli;
mod data;
mod err;

fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Err(e) = process(cli) {
        eprintln!("{}", e);

        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn process(cli: Cli) -> Result<(), Err> {
    match cli.command {
        Command::Icon(IconArgs { id }) => data::icons::extract(id, &cli),
        Command::JobActions(JobActionsCommandArgs { ref base, names }) => {
            data::job_actions::get(&job_actions::Input::ClassJob(base.id.clone()), &cli, names, base.pretty)
        },
        Command::RoleActions(RoleActionsCommandArgs { role, names, pretty }) => data::role_actions::get(role, &cli, names, pretty),
        Command::ContentFinderCondition(SheetCommandArgs { ref id, pretty })
      | Command::Action(SheetCommandArgs { ref id, pretty })
      | Command::Status(SheetCommandArgs { ref id, pretty }) => {
            match id {
                Id::Name(name) => data::sheet_extractor::search(cli.command.sheet(), name, &cli),
                Id::Index(index) => data::sheet_extractor::extract(cli.command.sheet(), *index, &cli, pretty),
            }
        }
    }
}
