use std::io::stdout;
use std::process::ExitCode;
use clio::ClioPath;
use ironworks_cli::err::Err;

use clap::{crate_name, crate_version, Parser};
use cli::{Cli, Command, IconArgs, JobActionsCommandArgs, RoleActionsCommandArgs, SheetCommandArgs};
use ironworks_cli::{self, Id};
use ironworks_cli::err::ToUnknownErr;
use ironworks_cli::{IronworksBuilder, IronworksCli, Sheet, WritableResult};

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
    let path = &cli.game;

    if cli.version {
        print_version(path);
        return Ok(());
    }

    match cli.command.ok_or(Err::NoSubcommand)? {
        Command::Icon(IconArgs { id }) => ironworks_cli::extract_icon(id, cli.game.as_deref(), stdout()),
        Command::JobActions(JobActionsCommandArgs { base, names }) => print(ironworks(path)?.get_job_actions(base.id)?.writable(names), base.pretty),
        Command::RoleActions(RoleActionsCommandArgs { role, names, pretty }) => print(ironworks(path)?.get_role_actions(role)?.writable(names), pretty),
        Command::ContentFinderCondition(SheetCommandArgs { ref id, pretty }) => process_sheet_command(Sheet::ContentFinderCondition, id, path, pretty),
        Command::Action(SheetCommandArgs { ref id, pretty }) => process_sheet_command(Sheet::Action, id, path, pretty),
        Command::Status(SheetCommandArgs { ref id, pretty }) => process_sheet_command(Sheet::Status, id, path, pretty)
    }
}

fn ironworks(game_path: &Option<ClioPath>) -> Result<IronworksCli, Err> {
    let mut builder = IronworksBuilder::new();

    if let Some(game_path) = game_path {
        builder = builder.game_path(game_path.to_path_buf());
    }

    builder.build()
}

fn print(input: impl WritableResult, pretty: bool) -> Result<(), Err> {
    if pretty {
        input.pretty_write(stdout()).to_unknown_err(29)
    } else {
        input.write(stdout()).to_unknown_err(30)
    }
}

fn process_sheet_command(sheet: Sheet, id: &Id, game_path: &Option<ClioPath>, pretty: bool) -> Result<(), Err> {
    let ironworks = ironworks(game_path)?;

    match id {
        Id::Name(name) => print(ironworks.search(sheet, name)?, pretty),
        Id::Index(index) => print(ironworks.get(sheet, *index)?, pretty),
    }
}

fn print_version(game_path: &Option<ClioPath>) {
    let ironworks = ironworks(game_path);

    println!("{} v{}", crate_name!(), crate_version!());

    if let Ok(ironworks) = ironworks {
        println!("Final Fantasy XIV v{}", ironworks.version());
    }
}
