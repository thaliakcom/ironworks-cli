use std::process::ExitCode;
use crate::err::Err;

use clap::Parser;
use cli::{Cli, Command, CommandArgs};

mod cli;
mod extractor;
mod err;
mod sheets;
mod icons;

fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Err(e) = process(cli) {
        eprintln!("{}", e);

        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn process(mut cli: Cli) -> Result<(), Err> {
    if let Command::Icon(CommandArgs { id }) = cli.command {
        icons::extract(&mut cli.file, id, &cli.game_dir)
    } else {
        extractor::extract(&mut cli.file, cli.command.sheet(), cli.command.id(), &cli.game_dir)
    }
}
