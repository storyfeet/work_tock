use chrono::Timelike;
use derive_more::*;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use pest::Parser;

use crate::err::TokErr;
use crate::pesto::TimeFile;

#[derive(Copy, Clone, PartialEq, Eq, Add, Sub, AddAssign, SubAssign)]
pub struct STime(i32); //minutes

impl STime {
    pub fn new(hr: i32, min: i32) -> Self {
        STime(hr * 60 + min)
    }
    pub fn now() -> Self {
        let t = chrono::offset::Local::now();
        STime::new(t.time().hour() as i32, t.time().minute() as i32)
    }
}

impl FromStr for STime {
    type Err = TokErr;
    fn from_str(s: &str) -> Result<Self, TokErr> {
        let p = TimeFile::parse(crate::pesto::Rule::Time, s)?
            .next()
            .unwrap();

        let mut rc = p.into_inner();
        Ok(STime::new(rc.next().unwrap().as_str().parse()?, rc.next().unwrap().as_str().parse()?))
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
        assert!("243430343090349309309430334390:54".parse::<STime>().is_err());
        assert_eq!("24:54".parse(), Ok(STime::new(24, 54)));
    }
}
