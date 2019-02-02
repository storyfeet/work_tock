use num::Num;
use std::str::FromStr;

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::*;

use crate::err::TokErr;

#[derive(Parser)]
#[grammar = "clock.pest"]
pub struct TimeFile;

pub trait Pestable: Sized {
    fn from_pesto(r: Pair<Rule>) -> Result<Self, TokErr>;

    fn from_pestopt(or: Option<Pair<Rule>>) -> Result<Self, TokErr> {
        Self::from_pesto(or.ok_or(TokErr::NoToken)?)
    }

    fn pest_parse(r:Rule,s:&str)->Result<Self,TokErr>{
        let ps = TimeFile::parse(r,s).unwrap().next();
        Self::from_pestopt(ps)
    }
}

impl<T> Pestable for T
where
    T: Num + FromStr,
    TokErr: From<<T as FromStr>::Err>,
{
    fn from_pesto(r: Pair<Rule>) -> Result<Self, TokErr> {
        match r.as_rule() {
            Rule::Num | Rule::Plusnum => Ok(r.as_str().parse::<T>()?),
            other => Err(TokErr::UnexpectedRule(other)),
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
