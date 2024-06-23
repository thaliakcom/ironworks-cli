use std::process::ExitCode;
use crate::err::Err;

use clap::Parser;
use cli::{Cli, Command, CommandArgs, Id};

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
        Command::Icon(CommandArgs { id }) => {
            match id {
                Id::Name(_) => Err(Err::SearchNotSupported),
                Id::Index(id) => data::icons::extract(id, &cli.game)
            }
        },
        Command::JobActions(CommandArgs { id }) => {
            match id {
                Id::Name(_) => Err(Err::SearchNotSupported),
                Id::Index(id) => data::job_actions::get(id, &cli.game)
            }
        },
        _ => {
            match cli.command.id() {
                Id::Name(name) => data::sheet_extractor::search(cli.command.sheet(), name, &cli.game),
                Id::Index(index) => data::sheet_extractor::extract(cli.command.sheet(), *index, &cli.game),
            }
        }
    }
}
