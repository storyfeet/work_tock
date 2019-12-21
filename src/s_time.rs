use chrono::naive::NaiveDate;
use chrono::offset::Local;
use chrono::Timelike;
use derive_more::*;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use pest::Parser;

use crate::err::{LineErr, TokErr};
use crate::pesto::{LineNum, Pestable, Rule, TimeFile};

#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Add, Sub, AddAssign, SubAssign)]
pub struct STime(i32); //minutes

impl STime {
    pub fn new(hr: i32, min: i32) -> Self {
        STime(hr * 60 + min)
    }
    pub fn now() -> Self {
        let t = Local::now();
        STime::new(t.time().hour() as i32, t.time().minute() as i32)
    }

    pub fn since(&self, now_date: &NaiveDate, then_time: Self, then_date: &NaiveDate) -> Self {
        let days_between = (*now_date - *then_date).num_days() as i32;

        *self + STime::new(24 * days_between, 0) - then_time
    }
}

impl Pestable for STime {
    fn from_pesto(p: pest::iterators::Pair<Rule>) -> Result<Self, LineErr> {
        match p.as_rule() {
            Rule::Time => {
                let mut rc = p.into_inner();
                Ok(STime::new(
                    i32::from_pestopt(rc.next())?,
                    i32::from_pestopt(rc.next())?,
                ))
            }
            Rule::Clockout => STime::from_pestopt(p.into_inner().next()),
            v => Err(TokErr::UnexpectedRule(v).on_line(p.line_num())),
        }
    }
}

impl FromStr for STime {
    type Err = TokErr;
    fn from_str(s: &str) -> Result<Self, TokErr> {
        let p = TimeFile::parse(crate::pesto::Rule::Time, s)?
            .next()
            .unwrap();
        Self::from_pesto(p).map_err(|e| e.err)
    }
}

impl Debug for STime {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:02}:{:02}", self.0 / 60, self.0 % 60)
    }
}

impl Display for STime {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn test_stime_parse() {
        assert!("243430343090349309309430334390:54"
            .parse::<STime>()
            .is_err());
        assert_eq!("24:54".parse(), Ok(STime::new(24, 54)));
    }
}
