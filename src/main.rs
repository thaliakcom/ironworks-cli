use std::process::ExitCode;

use clap::Parser;
use cli::Cli;

mod cli;
mod extractor;
mod err;
mod sheets;

fn main() -> ExitCode {
    let mut cli = Cli::parse();

    if let Err(e) = extractor::extract(&mut cli.file, cli.command.sheet(), cli.command.id(), &cli.game_dir) {
        eprintln!("{}", e);

        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
