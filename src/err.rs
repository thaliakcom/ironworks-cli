use std::{backtrace::Backtrace, fmt::Display};

#[derive(Debug)]
pub enum Err {
    GameNotFound,
    VersionNotFound(String),
    SheetNotFound(&'static str),
    RowNotFound(&'static str, u32),
    ColumnNotFound(&'static str, &'static str),
    NoIndex(&'static str, &'static str),
    IconNotFound(String),
    JobNotFound(u32),
    JobAcronymNotFound(String),
    UnsupportedIconFormat(u32, String),
    UnsupportedSheet(&'static str),
    Unknown(Option<Backtrace>)
}

impl Display for Err {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GameNotFound => writeln!(f, "No game path found. You can specify the game path automatically by using the \"-d\" option."),
            Self::VersionNotFound(s) => writeln!(f, "No schema for game version {} found. You may need to wait for schemas to be updated", s),
            Self::SheetNotFound(s) => writeln!(f, "Sheet {} not found", s),
            Self::RowNotFound(sheet, row) => writeln!(f, "Sheet {} has no row {}", sheet, row),
            Self::ColumnNotFound(sheet, column) => writeln!(f, "Sheet {} has no column {}", sheet, column),
            Self::NoIndex(sheet, column) => writeln!(f, "Column {}::{} cannot be coerced to a u32", sheet, column),
            Self::IconNotFound(path) => writeln!(f, "No icon found at path \"{}\"", path),
            Self::JobNotFound(job) => writeln!(f, "There is no class or job with ID \"{}\"", job),
            Self::JobAcronymNotFound(job) => writeln!(f, "There is no class or job with abbreviation \"{}\"", job),
            Self::UnsupportedIconFormat(format, path) => writeln!(f, "Unsupported icon format {:#04x} at \"{}\"", format, path),
            Self::UnsupportedSheet(sheet) => writeln!(f, "Unsupported sheet type {}", sheet),
            Self::Unknown(trace) => if let Some(trace) = trace {
                writeln!(f, "An unknown error occurred at:\n{}", trace)
            } else {
                writeln!(f, "An unknown error occurred")
            }
        }
    }
}

pub trait ToUnknownErr<T> {
    fn to_unknown_err(self) -> Result<T, Err>;
}

impl <T, E> ToUnknownErr<T> for Result<T, E> {
    /// Converts the [`Result<T, E>`] into a `Result<T, Err::Unknown>`.
    fn to_unknown_err(self) -> Result<T, Err> {
        self.map_err(|_| {
            if cfg!(debug_assertions) {
                Err::Unknown(Some(Backtrace::force_capture()))
            } else {
                Err::Unknown(None)
            }
        })
    }
}

impl <T> ToUnknownErr<T> for Option<T> {
    /// Converts the [`Option<T>`] into a `Result<T, Err::Unknown>`.
    fn to_unknown_err(self) -> Result<T, Err> {
        self.ok_or_else(|| {
            if cfg!(debug_assertions) {
                Err::Unknown(Some(Backtrace::force_capture()))
            } else {
                Err::Unknown(None)
            }
        })
    }
}
