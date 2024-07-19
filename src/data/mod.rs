pub mod icons;
mod init;
pub mod job_actions;
pub mod role_actions;
pub mod sheet_extractor;
pub mod sheets;

use std::{io::{stdout, Stdout}, path::PathBuf};

pub use init::*;

pub struct Args<O : std::io::Write> {
    /// The path to the game directory.
    pub game_path: Option<PathBuf>,
    /// Whether or not to refresh the repository data.
    pub refresh: bool,
    /// The output stream the data should be written to.
    pub out: O
}

impl Args<Stdout> {
    pub fn from_path(path: PathBuf) -> Self {
        Self::from_path_to_stream(path, stdout())
    }
}

impl <O : std::io::Write> Args<O> {
    pub fn from_path_to_stream(path: PathBuf, out: O) -> Self {
        Self { game_path: Some(path), refresh: false, out }
    }
}

#[derive(Debug, Clone)]
pub enum Id {
    Name(String),
    Index(u32)
}
