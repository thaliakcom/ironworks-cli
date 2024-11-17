use std::convert::Infallible;
use std::io::{stdout, Stdout};
use clap::{Args, Parser, Subcommand};
use clio::ClioPath;
use ironworks_cli::data::Role;
use ironworks_cli::data::Id;

/// A command line utility that can extract data from FFXIV's internal Excel sheets.
#[derive(Parser, Debug)]
#[command(version, propagate_version = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Command,
    /// Path to the game's directory.
    /// If not specified, attempts to find the game directory automatically.
    #[clap(global = true, long, short, value_parser, default_value = "Option::None")]
    pub game: Option<ClioPath>,
    /// If set, the header data for the game files is forcibly updated from
    /// an upstream source. This requires an internet connection.
    /// 
    /// Note that, by default, the header data is automatically updated
    /// whenever the game is updated. However, in some cases after a game updated
    /// you may accidentally run this program before the upstream data is updated,
    /// in which case this flag is required to manually update the header data.
    #[clap(global = true, long, short)]
    pub refresh: bool
}

impl Cli {
    pub fn pretty_print(&self) -> bool {
        match &self.command {
            | Command::ContentFinderCondition(command_args) => command_args.pretty,
            | Command::Action(command_args) => command_args.pretty,
            | Command::Status(command_args) => command_args.pretty,
            | Command::JobActions(command_args) => command_args.base.pretty,
            | Command::RoleActions(command_args) => command_args.pretty,
            _ => false
        }
    }
}

impl From<Cli> for ironworks_cli::Args<Stdout> {
    fn from(value: Cli) -> Self {
        Self {
            game_path: value.game.as_ref().map(|x| x.to_path_buf()),
            refresh: value.refresh,
            out: Some(stdout()),
            pretty_print: value.pretty_print()
        }
    }
}

impl From<&Cli> for ironworks_cli::Args<Stdout> {
    fn from(value: &Cli) -> Self {
        Self {
            game_path: value.game.as_ref().map(|x| x.to_path_buf()),
            refresh: value.refresh,
            out: Some(stdout()),
            pretty_print: value.pretty_print()
        }
    }
}

#[derive(Subcommand, Debug)]
#[clap(rename_all = "verbatim")]
pub(crate) enum Command {
    /// Retrieves JSON information about a specific duty.
    ContentFinderCondition(SheetCommandArgs),
    /// Retrieves JSON information about a specific action.
    Action(SheetCommandArgs),
    /// Retrieves JSON information about a specific status effect.
    Status(SheetCommandArgs),
    /// Prints an array of the numerical IDs of all job actions for a specific class or job.
    #[clap(name = "job-actions")]
    JobActions(JobActionsCommandArgs),
    /// Prints an array of the numerical IDs of all role actions for a specific role.
    #[clap(name = "role-actions")]
    RoleActions(RoleActionsCommandArgs),
    /// Retrieves a specific icon and prints its binary data.
    #[clap(name = "icon")]
    Icon(IconArgs),
    /// Prints the game's installed version.
    #[clap(name = "version")]
    Version
}

#[derive(Args, Debug)]
pub(crate) struct SheetCommandArgs {
    /// The ID of the item that information should be retrieved about.
    /// Can also be a string to search for an item by name.
    #[clap(value_parser = parse_id)]
    pub id: Id,
    /// Whether to pretty-print the result.
    #[clap(short, long)]
    pub pretty: bool
}

#[derive(Args, Debug)]
pub(crate) struct JobActionsCommandArgs {
    #[clap(flatten)]
    pub base: SheetCommandArgs,
    /// Prints an array of JSON objects containing each action's ID and name.
    #[clap(short, long)]
    pub names: bool
}

#[derive(Args, Debug)]
pub(crate) struct RoleActionsCommandArgs {
    #[clap(value_enum)]
    pub role: Role,
    /// Prints an array of JSON objects containing each action's ID and name.
    #[clap(short, long)]
    pub names: bool,
    /// Whether to pretty-print the result.
    #[clap(short, long)]
    pub pretty: bool
}

#[derive(Args, Debug)]
pub(crate) struct IconArgs {
    /// The ID of the item that information should be retrieved about.
    pub id: u32
}

fn parse_id(input: &str) -> Result<Id, Infallible> {
    Ok(input.parse::<u32>().map_or(Id::Name(input.to_owned()), Id::Index))
}
