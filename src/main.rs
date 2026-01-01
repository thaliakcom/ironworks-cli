use std::io::stdout;
use std::process::ExitCode;
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
    if cli.version.as_ref().is_some_and(|x| x.is_empty()) {
        print_version(&cli);
        return Ok(());
    }

    match cli.command.as_ref().ok_or(Err::NoSubcommand)? {
        Command::Icon(IconArgs { id }) => ironworks_cli::extract_icon(*id, cli.game.as_deref(), stdout()),
        Command::JobActions(JobActionsCommandArgs { base, names }) => print(ironworks(&cli)?.get_job_actions(base.id.clone())?.writable(*names), base.pretty),
        Command::RoleActions(RoleActionsCommandArgs { role, names, pretty }) => print(ironworks(&cli)?.get_role_actions(*role)?.writable(*names), *pretty),
        Command::ContentFinderCondition(SheetCommandArgs { id, pretty }) => process_sheet_command(Sheet::ContentFinderCondition, id, &cli, *pretty),
        Command::Action(SheetCommandArgs { id, pretty }) => process_sheet_command(Sheet::Action, id, &cli, *pretty),
        Command::Status(SheetCommandArgs { id, pretty }) => process_sheet_command(Sheet::Status, id, &cli, *pretty)
    }
}

fn ironworks(cli: &Cli) -> Result<IronworksCli, Err> {
    let mut builder = IronworksBuilder::new();

    if let Some(game_path) = &cli.game {
        builder = builder.game_path(game_path.to_path_buf());
    }

    if let Some(requested_version) = &cli.version {
        if !requested_version.is_empty() {
            builder = builder.force_refresh_with_version(requested_version.to_string());
        }
    }

    if cli.refresh {
        builder = builder.force_refresh()
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

fn process_sheet_command(sheet: Sheet, id: &Id, cli: &Cli, pretty: bool) -> Result<(), Err> {
    let ironworks = ironworks(cli)?;

    match id {
        Id::Name(name) => print(ironworks.search(sheet, name)?, pretty),
        Id::Index(index) => print(ironworks.get(sheet, *index)?, pretty),
    }
}

fn print_version(cli: &Cli) {
    let ironworks = ironworks(cli);

    println!("{} v{}", crate_name!(), crate_version!());

    match ironworks {
        Ok(ironworks) => println!("Final Fantasy XIV v{}", ironworks.version()),
        Result::Err(err) => eprintln!("Failed to find FFXIV version (error: \"{}\")", err),
    }
}
