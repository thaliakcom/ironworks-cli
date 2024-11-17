mod icons;
mod init;
mod job_actions;
mod role_actions;
mod sheet_extractor;
mod sheets;

use std::{io::{stdout, Stdout}, path::PathBuf};

pub use init::*;
pub use icons::extract as extract_icon;
pub use job_actions::*;
pub use role_actions::*;
pub use sheet_extractor::*;
pub use sheets::*;

pub struct Args<O : std::io::Write> {
    /// The path to the game directory.
    pub game_path: Option<PathBuf>,
    /// Whether or not to refresh the repository data.
    pub refresh: bool,
    /// Whether the output should be pretty printed.
    /// Only applicable when [`Args::out`] is [`Some`].
    pub pretty_print: bool,
    /// The output stream the data should be written to.
    pub out: Option<O>
}

impl Args<Stdout> {
    /// Creates a new Args instance from the given game path that
    /// doesn't output its results to any output stream.
    pub fn from_path(path: PathBuf) -> Self {
        Self { game_path: Some(path), refresh: false, out: None, pretty_print: false }
    }

    /// Creates a new Args instance from the given game path that
    /// outputs its results to [`stdout`].
    pub fn from_path_stdout(path: PathBuf) -> Self {
        Self::from_path_and_stream(path, stdout())
    }
}

impl <O : std::io::Write> Args<O> {
    /// Creates a new Args instance from the given game path that
    /// outputs its results to the given output stream.
    pub fn from_path_and_stream(path: PathBuf, out: O) -> Self {
        Self { game_path: Some(path), refresh: false, out: Some(out), pretty_print: false }
    }
}

#[derive(Debug, Clone)]
pub enum Id {
    Name(String),
    Index(u32)
}
