use crate::pesto;

#[derive(Debug, PartialEq)]
pub enum TokErr {
    Mess(String),
    NotSet(&'static str),
    ParseErr(pest::error::Error<pesto::Rule>),
    ParseIntErr,
    UnexpectedRule(pesto::Rule),
    NoToken,
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
