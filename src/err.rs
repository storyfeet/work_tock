use crate::pesto;
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
    ParseErr(pest::error::Error<pesto::Rule>),
    #[fail(display = "Cannot parse int")]
    ParseIntErr,
    #[fail(display = "Unexepected Parsing Rule {:?}", 0)]
    UnexpectedRule(pesto::Rule),
    #[fail(display = "No Token")]
    NoToken,
    #[fail(display = "Cannot work for negative time")]
    NegativeTime,
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

impl From<pest::error::Error<pesto::Rule>> for TokErr {
    fn from(e: pest::error::Error<pesto::Rule>) -> Self {
        TokErr::ParseErr(e)
    }
}

impl From<std::num::ParseIntError> for TokErr {
    fn from(_: std::num::ParseIntError) -> Self {
        TokErr::ParseIntErr
    }
}
