use chrono::naive::NaiveDate;
use pest::iterators::Pair;

use crate::err::TokErr;
use crate::pesto::{Pestable, Rule};
use crate::s_time::STime;

#[derive(Debug)]
pub enum ClockAction {
    NoAction,
    AddTag(String),
    ClearTags(Option<String>), //replacement tag
    AddClockin(STime),         // bool = is_in
    AddClockout(STime),
    AddClockIO(STime, STime),
    SetJob(String),
    SetDate(u32, u32, Option<i32>),
    SetNum(String, u32),
}
use self::ClockAction::*;

impl Pestable for ClockAction {
    fn from_pesto(r: Pair<Rule>) -> Result<Self, TokErr> {
        match r.as_rule() {
            Rule::Time => Ok(AddClockin(STime::from_pesto(r)?)),
            Rule::Clockout => Ok(AddClockout(STime::from_pesto(r)?)),
            Rule::ClockIO => {
                let rc = r.into_inner();
                Ok(AddClockIO(
                    STime::from_pestopt(rc.next())?,
                    STime::from_pestopt(rc.next())?,
                ))
            }
            Rule::Date => {
                let rc = r.into_inner();
                Ok(SetDate(
                    u32::from_pestopt(rc.next())?,
                    u32::from_pestopt(rc.next())?,
                    match rc.next() {
                        //third part of date optional
                        Some(v) => Some(i32::from_pesto(v)?),
                        None => None,
                    },
                ))
            }

            other => Err(TokErr::UnexpectedRule(other)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Clockin {
    In(InData),
    Out(STime),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InData {
    pub time: STime,
    pub date: NaiveDate,
    pub job: String,
    pub tags: Vec<String>,
}

pub fn read_string(s: &str) -> (Vec<Clockin>, Vec<TokErr>) {
    //TODO, seriously
    (Vec::new(), Vec::new())
}
