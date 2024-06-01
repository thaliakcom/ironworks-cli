use std::process::ExitCode;
use crate::err::Err;

use clap::Parser;
use cli::{Cli, Command, CommandArgs, Id};

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

fn process(cli: Cli) -> Result<(), Err> {
    if let Command::Icon(CommandArgs { id }) = cli.command {
        match id {
            Id::Name(_) => Err(Err::NoSearchForIcon),
            Id::Index(id) => icons::extract(id, &cli.game_dir)
        }
    } else {
        match cli.command.id() {
            Id::Name(name) => extractor::search(cli.command.sheet(), &name, &cli.game_dir),
            Id::Index(index) => extractor::extract(cli.command.sheet(), *index, &cli.game_dir),
        }
    }
}
