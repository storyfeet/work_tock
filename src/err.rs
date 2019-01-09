use crate::parse::Token;

#[derive(Debug)]
pub enum TokErr {
    Mess(String),
    NotNum(Token),
    NotAsExpected(Token, Token),
    UnexpectedEOF,
    EOF, //Don't like this,
    MustFail(String),
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

pub fn must(b:bool,s:&str)-> Result<(),TokErr>{
    if b {Ok(())} else {Err(TokErr::MustFail(s.to_string()))}
}

pub fn must_pass<T,F>(t:T,f:F,s:&str)->Result<(),TokErr>
    where F:FnOnce(T)->bool
{
    if f(t) {Ok(())} else{ (Err(TokErr::MustFail(s.to_string())))}
}
