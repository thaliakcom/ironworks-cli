use std::io::Stdout;
use std::process::ExitCode;
use ironworks_cli::err::Err;

use clap::Parser;
use cli::{Cli, Command, IconArgs, JobActionsCommandArgs, RoleActionsCommandArgs, SheetCommandArgs};
use ironworks_cli::data::{self, Id};
use ironworks_cli::err::ToUnknownErr;
use ironworks::sqpack::Resource;

mod cli;

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
        Command::Icon(IconArgs { id }) => data::icons::extract(id, &mut cli.into()),
        Command::JobActions(JobActionsCommandArgs { ref base, names }) => {
            data::job_actions::get(&data::job_actions::Input::ClassJob(base.id.clone()), &mut (&cli).into(), names, base.pretty)
        },
        Command::RoleActions(RoleActionsCommandArgs { role, names, pretty }) => data::role_actions::get(role, &mut cli.into(), names, pretty),
        Command::ContentFinderCondition(SheetCommandArgs { ref id, pretty })
      | Command::Action(SheetCommandArgs { ref id, pretty })
      | Command::Status(SheetCommandArgs { ref id, pretty }) => {
            match id {
                Id::Name(name) => data::sheet_extractor::search(cli.command.sheet(), name, &mut (&cli).into()),
                Id::Index(index) => data::sheet_extractor::extract(cli.command.sheet(), *index, &mut cli.into(), pretty),
            }
        },
        Command::Version => {
            let game_res = data::Init::get_game_resource(&Into::<ironworks_cli::data::Args<Stdout>>::into(cli).game_path.as_deref()).to_unknown_err()?;
            let version = game_res.version(0).to_unknown_err()?;
            println!("{}", version);

            Ok(())
        }
    }
}
