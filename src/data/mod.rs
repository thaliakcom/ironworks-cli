mod icons;
mod init;
mod job_actions;
mod role_actions;
mod sheet_extractor;
mod sheets;

pub use init::*;
pub use icons::extract as extract_icon;
use ironworks::{excel::Field, sestring::SeString};
pub use job_actions::*;
pub use role_actions::*;
pub use sheet_extractor::*;
pub use sheets::*;

/// Either the name or numerical ID of the desired entity.
#[derive(Debug, Clone)]
pub enum Id {
    Name(String),
    Index(u32)
}

/// A result of any of [`IronworksCli`]'s functions that can be written
/// (either in prettified or minified) form to an [`std::io::Write`] stream.
pub trait WritableResult {
    /// Writes a minified representation of the result to the [`std::io::Write`] stream.
    fn write(&self, w: impl std::io::Write) -> std::io::Result<()>;
    /// Writes a prettified representation (with whitespace) of the result to the [`std::io::Write`] stream.
    fn pretty_write(&self, w: impl std::io::Write) -> std::io::Result<()>;
}

impl WritableResult for Field {
    fn write(&self, mut w: impl std::io::Write) -> std::io::Result<()> {
        match self {
            Field::String(s) => s.write(w),
            Field::Bool(b) => write!(w, "{}", b),
            Field::I8(num) => write!(w, "{}", num),
            Field::I16(num) => write!(w, "{}", num),
            Field::I32(num) => write!(w, "{}", num),
            Field::I64(num) => write!(w, "{}", num),
            Field::U8(num) => write!(w, "{}", num),
            Field::U16(num) => write!(w, "{}", num),
            Field::U32(num) => write!(w, "{}", num),
            Field::U64(num) => write!(w, "{}", num),
            Field::F32(num) => write!(w, "{}", num)
        }
    }

    fn pretty_write(&self, w: impl std::io::Write) -> std::io::Result<()> {
        self.write(w)
    }
}

impl <'a> WritableResult for SeString<'a> {
    fn write(&self, mut w: impl std::io::Write) -> std::io::Result<()> {
        write!(w, "\"{}\"", self.to_string().replace('\n', "\\n").replace('"', "\\\""))
    }

    fn pretty_write(&self, w: impl std::io::Write) -> std::io::Result<()> {
        self.write(w)
    }
}
