use failure_derive::*;

#[derive(Debug, PartialEq, Fail)]
#[fail(display = "Err on line {}: {}", line, err)]
pub struct LineErr {
    pub line: usize,
    pub err: TokErr,
}

#[derive(Debug, PartialEq, Fail)]
pub enum TokErr {
    #[fail(display = "{}", 0)]
    Mess(String),
    #[fail(display = "Not Set : {}", 0)]
    NotSet(&'static str),
    #[fail(display = "{}", 0)]
    ParseErr(gobble::ParseError),
    #[fail(display = "Cannot parse int")]
    ParseIntErr,
    #[fail(display = "No Token")]
    NoToken,
    #[fail(display = "Cannot work for negative time")]
    NegativeTime,
    #[fail(display = "Processing errors {:?}", 0)]
    Lines(Vec<LineErr>),
}

impl TokErr {
    pub fn on_line(self, n: usize) -> LineErr {
        LineErr { line: n, err: self }
    }
}

impl From<&str> for TokErr {
    fn from(s: &str) -> Self {
        TokErr::Mess(s.to_string())
    }
}

impl From<String> for TokErr {
    fn from(s: String) -> Self {
        TokErr::Mess(s.to_string())
    }
}

impl From<std::num::ParseIntError> for TokErr {
    fn from(_: std::num::ParseIntError) -> Self {
        TokErr::ParseIntErr
    }
}

impl From<gobble::ParseError> for TokErr {
    fn from(e: gobble::ParseError) -> Self {
        TokErr::ParseErr(e)
    }
}
