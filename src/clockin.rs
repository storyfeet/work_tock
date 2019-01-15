use chrono::naive::NaiveDate;
use pest::iterators::Pair;
use pest::Parser;

use crate::err::TokErr;
use crate::pesto::{Pestable, Rule, TimeFile};
use crate::s_time::STime;

#[derive(Debug)]
pub enum ClockAction {
    AddTag(String),
    ClearTags(Option<String>), //replacement tag
    AddClockin(STime),         // bool = is_in
    AddClockout(STime),
    AddClockIO(STime, STime),
    SetJob(String),
    SetDate(u32, u32, Option<i32>),
    SetNum(String, i32),
}
use self::ClockAction::*;

impl Pestable for ClockAction {
    fn from_pesto(r: Pair<Rule>) -> Result<Self, TokErr> {
        match r.as_rule() {
            Rule::Time => Ok(AddClockin(STime::from_pesto(r)?)),
            Rule::Clockout => Ok(AddClockout(STime::from_pesto(r)?)),
            Rule::ClockIO => {
                let mut rc = r.into_inner();
                Ok(AddClockIO(
                    STime::from_pestopt(rc.next())?,
                    STime::from_pestopt(rc.next())?,
                ))
            }
            Rule::Date => {
                let mut rc = r.into_inner();
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
            Rule::Tag => Ok(AddTag(
                r.into_inner()
                    .next()
                    .expect("Tag should always have an inner")
                    .as_str()
                    .to_string(),
            )),
            Rule::ClearTags => Ok(ClearTags(
                r.into_inner().next().map(|p| p.as_str().to_string()),
            )),
            Rule::Job => {
                let inner = r
                    .into_inner()
                    .next()
                    .expect("Job should always have an Inner");
                //Consider de-escaping should not be an issue
                Ok(SetJob(inner.as_str().to_string()))
            }
            Rule::NumSetter => {
                let mut rc = r.into_inner();
                Ok(SetNum(
                    rc.next()
                        .expect("NumSet should always have 2 children")
                        .as_str()
                        .to_string(),
                    i32::from_pestopt(rc.next())?,
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

    let mut job = "General".to_string();
    let mut tags = Vec::new();
    let mut date = NaiveDate::from_ymd(1, 1, 1); //consider changing

    let p = match TimeFile::parse(Rule::Main, s) {
        Ok(mut p) => p.next().expect("Root should always have one child"),
        Err(e) => return (Vec::new(), vec![TokErr::ParseErr(e)]),
    };

    let mut res = Vec::new();
    let mut errs = Vec::new();

    for record in p.into_inner() {
        if record.as_rule() == Rule::EOI {
            continue;
        }
        println!("Record {:?}", record);
        let pr = record
            .into_inner()
            .next()
            .expect("Record should always have exactly one child");
        match pr.as_rule() {
            Rule::Word => job = pr.as_str().to_string(),
            Rule::Time => match STime::from_pesto(pr) {
                Ok(time) => res.push(Clockin::In(InData {
                    time,
                    job: job.clone(),
                    tags: tags.clone(),
                    date,
                })),
                Err(e) => errs.push(e),
            },

            other => errs.push(TokErr::UnexpectedRule(other)),
        }
    }

    (res, errs)
}
