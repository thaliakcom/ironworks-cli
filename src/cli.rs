use std::convert::Infallible;
use clap::{Args, Parser, Subcommand};
use clio::ClioPath;
use ironworks_cli::Role;
use ironworks_cli::Id;

/// A command line utility that can extract data from FFXIV's internal Excel sheets.
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
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
    pub refresh: bool,
    /// Prints the version of the application and the game directory (if specified or found).
    #[clap(global = true, long, short, num_args = 0..2, require_equals = true, default_missing_value = "", default_value = "Option::None")]
    pub version: Option<String>
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
    Icon(IconArgs)
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
