use crate::parse::Token;

#[derive(Debug)]
pub enum TokErr {
    Mess(String),
    NotNum(Token),
    NotAsExpected(Token, Token),
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
