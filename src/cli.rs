use std::fmt::Display;

use clap::{Args, Parser, Subcommand};
use clio::{ClioPath, Output};
use strum::IntoStaticStr;

/// A command line utility that can extract data from FFXIV's internal Excel sheets.
#[derive(Parser, Debug)]
#[command(version, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    /// The file that the output should be written to.
    /// If not specified, writes to stdout.
    #[clap(global = true, long, short, value_parser, default_value = "-")]
    pub file: Output,
    /// Path to the game's directory.
    /// If not specified, attempts to find the game directory automatically.
    #[clap(global = true, long, short = 'd', value_parser, default_value = "Option::None")]
    pub game_dir: Option<ClioPath>
}

#[derive(Subcommand, Debug, IntoStaticStr)]
#[clap(rename_all = "verbatim")]
pub enum Command {
    /// Retrieves information about a specific duty.
    ContentFinderCondition(CommandArgs),
    /// Retrieves information about a specific action.
    Action(CommandArgs),
    /// Retrieves information about a specific status effect.
    Status(CommandArgs),
    /// Retrieves a specific icon.
    #[clap(name = "icon")]
    Icon(CommandArgs)
}

impl Command {
    pub fn id(&self) -> &Id {
        let (Command::ContentFinderCondition(args) | Command::Action(args) | Command::Status(args) | Command::Icon(args)) = self;
        &args.id
    }

    /// Gets the name of the game sheet corresponding to this command.
    pub fn sheet(&self) -> &'static str {
        self.into()
    }
}

#[derive(Args, Debug)]
pub struct CommandArgs {
    /// The ID of the item that information should be retrieved about.
    /// Can also be a string to search for an item by name.
    #[clap(value_parser = parse_id)]
    pub id: Id
}

#[derive(Debug, Clone)]
pub enum Id {
    Name(String),
    Index(u32)
}

#[derive(Debug)]
struct Never;

impl std::error::Error for Never {}
impl Display for Never {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

fn parse_id(input: &str) -> Result<Id, Never> {
    Ok(input.parse::<u32>().map_or(Id::Name(input.to_owned()), |v| Id::Index(v)))
}
