use std::fmt::Display;

#[derive(Debug)]
pub enum Err {
    GameNotFound,
    SheetNotFound(&'static str),
    RowNotFound(&'static str, u32),
    ColumnNotFound(&'static str, &'static str),
    NoIndex(&'static str, &'static str),
    IconNotFound(String),
    UnsupportedIconFormat(u32, String),
    UnsupportedSheet(&'static str),
    NoSearchForIcon,
    Unknown
}

impl Display for Err {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GameNotFound => writeln!(f, "No game path found. You can specify the game path automatically by using the \"-d\" option."),
            Self::SheetNotFound(s) => writeln!(f, "Sheet {} not found", s),
            Self::RowNotFound(sheet, row) => writeln!(f, "Sheet {} has no row {}", sheet, row),
            Self::ColumnNotFound(sheet, column) => writeln!(f, "Sheet {} has no column {}", sheet, column),
            Self::NoIndex(sheet, column) => writeln!(f, "Column {}::{} cannot be coerced to a u32", sheet, column),
            Self::IconNotFound(path) => writeln!(f, "No icon found at path \"{}\"", path),
            Self::UnsupportedIconFormat(format, path) => writeln!(f, "Unsupported icon format {:#04x} at \"{}\"", format, path),
            Self::UnsupportedSheet(sheet) => writeln!(f, "Unsupported sheet type {}", sheet),
            Self::NoSearchForIcon => writeln!(f, "Using the search feature is only supported for excel sheet subcommands"),
            Self::Unknown => writeln!(f, "An unknown error occurred")
        }
    }
}

pub trait ToUnknownErr<T> {
    fn to_unknown_err(self) -> Result<T, Err>;
}

impl <T, E> ToUnknownErr<T> for Result<T, E> {
    fn to_unknown_err(self) -> Result<T, Err> {
        self.map_err(|_| Err::Unknown)
    }
}

impl <T> ToUnknownErr<T> for Option<T> {
    fn to_unknown_err(self) -> Result<T, Err> {
        self.ok_or(Err::Unknown)
    }
}
