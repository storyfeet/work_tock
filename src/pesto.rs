use num::Num;
use std::str::FromStr;

use pest::iterators::Pair;
use pest::{Parser, RuleType};
use pest_derive::*;

use crate::err::{LineErr, TokErr};

pub trait LineNum {
    fn line_num(&self) -> usize;
}
impl<'a, R: RuleType> LineNum for Pair<'a, R> {
    fn line_num(&self) -> usize {
        self.as_span().start_pos().line_col().0
    }
}

#[derive(Parser)]
#[grammar = "clock.pest"]
pub struct TimeFile;

pub trait Pestable: Sized {
    fn from_pesto(r: Pair<Rule>) -> Result<Self, LineErr>;
    fn from_pestopt(po: Option<Pair<Rule>>) -> Result<Self, LineErr> {
        Self::from_pesto(po.ok_or(TokErr::NoToken.on_line(0))?)
    }

    fn pest_parse(r: Rule, s: &str) -> Result<Self, LineErr> {
        let ps = TimeFile::parse(r, s).map_err(|_| TokErr::from(format!("could not parse '{}' as time",s)).on_line(0) )?.next();
        Self::from_pestopt(ps)
    }
}

impl<T> Pestable for T
where
    T: Num + FromStr,
    TokErr: From<<T as FromStr>::Err>,
{
    fn from_pesto(r: Pair<Rule>) -> Result<Self, LineErr> {
        match r.as_rule() {
            Rule::Num | Rule::Plusnum => r.as_str().parse::<T>().map_err(|e| {
                let te: TokErr = e.into();
                te.on_line(r.line_num())
            }),
            other => Err(TokErr::UnexpectedRule(other).on_line(r.line_num())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pest::Parser;
    #[test]
    pub fn test_clock() {
        let t = TimeFile::parse(Rule::Time, "25:34")
            .unwrap()
            .next()
            .expect("Not sure what this one does");
        assert_eq!(t.as_rule(), Rule::Time);
        assert_eq!(t.as_str(), "25:34");

        println!("T is {}", t);

        let mut rc = t.into_inner();
        println!("RC is {:?}", rc);

        assert_eq!(rc.next().expect("Or this one").as_str(), "25");
    }
}
