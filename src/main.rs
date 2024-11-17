use std::io::Stdout;
use std::process::ExitCode;
use ironworks_cli::err::Err;

use clap::Parser;
use cli::{Cli, Command, IconArgs, JobActionsCommandArgs, RoleActionsCommandArgs, SheetCommandArgs};
use ironworks_cli::data::{self, Id};
use ironworks_cli::err::ToUnknownErr;
use ironworks::sqpack::Resource;
use ironworks_cli::Sheet;

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
        Command::Icon(IconArgs { id }) => data::extract_icon(id, cli.into()),
        Command::JobActions(JobActionsCommandArgs { ref base, names }) => data::get_job_actions(base.id.clone(), cli.into(), names).map(|_| ()),
        Command::RoleActions(RoleActionsCommandArgs { role, names, .. }) => data::get_role_actions(role, cli.into(), names).map(|_| ()),
        Command::ContentFinderCondition(SheetCommandArgs { ref id, .. }) => process_sheet_command(Sheet::ContentFinderCondition, id, &cli),
      | Command::Action(SheetCommandArgs { ref id, .. }) => process_sheet_command(Sheet::Action, id, &cli),
      | Command::Status(SheetCommandArgs { ref id, .. }) => process_sheet_command(Sheet::Status, id, &cli),
        Command::Version => {
            let game_res = data::get_game_resource(&Into::<ironworks_cli::data::Args<Stdout>>::into(cli).game_path.as_deref()).to_unknown_err()?;
            let version = game_res.version(0).to_unknown_err()?;
            println!("{}", version);

            Ok(())
        }
    }
}

fn process_sheet_command(sheet: Sheet, id: &Id, cli: &Cli) -> Result<(), Err> {
    match id {
        Id::Name(name) => data::search(sheet, name, cli.into()).map(|_| ()),
        Id::Index(index) => data::extract(sheet, *index, cli.into()).map(|_| ()),
    }
}
