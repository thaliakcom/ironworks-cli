use std::fmt::Display;

#[derive(Debug)]
pub enum Err {
    GameNotFound,
    SheetNotFound(&'static str),
    RowNotFound(&'static str, u32),
    UnsupportedSheet(&'static str)
}

impl Display for Err {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GameNotFound => writeln!(f, "No game path found. You can specify the game path automatically by using the \"-d\" option."),
            Self::SheetNotFound(s) => writeln!(f, "Sheet {} not found", s),
            Self::RowNotFound(sheet, row) => writeln!(f, "Sheet {} has no row {}", sheet, row),
            Self::UnsupportedSheet(sheet) => writeln!(f, "Unsupported sheet type {}", sheet)
        }
    }
}
