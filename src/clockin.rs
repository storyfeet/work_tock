use chrono::naive::NaiveDate;
use chrono::Datelike;
use pest::iterators::Pair;
use pest::Parser;
use std::fmt::Display;

use crate::err::{LineErr, TokErr};
use crate::pesto::{LineNum, Pestable, Rule, TimeFile};
use crate::s_time::STime;

#[derive(Debug)]
pub struct LineClockAction {
    pub line: usize,
    pub action: ClockAction,
}

#[derive(Debug)]
pub enum ClockAction {
    AddTag(String),
    ClearTags(Option<String>), //replacement tag
    In(STime),
    Out(STime),
    InOut(STime, STime),
    SetJob(String),
    SetDate(u32, u32, Option<i32>),
    SetNum(String, i32),
}
use self::ClockAction::*;

pub fn read_date(s: &str) -> Result<NaiveDate,TokErr> {
    LineClockAction::pest_parse(Rule::Date, &s).map_err(|e|e.err)?
        .action
        .as_date()
            .ok_or(TokErr::from("Could not read since date"))
}

impl ClockAction {
    pub fn as_date(&self) -> Option<NaiveDate> {
        match self {
            ClockAction::SetDate(d, m, Some(y)) => Some(NaiveDate::from_ymd(*y, *m, *d)),
            ClockAction::SetDate(d, m, None) => {
                let date = chrono::offset::Local::today();
                Some(NaiveDate::from_ymd(date.year(), *m, *d))
            }
            _ => None,
        }
    }
}

impl Pestable for LineClockAction {
    fn from_pesto(r: Pair<Rule>) -> Result<Self, LineErr> {
        let (line, _) = r.as_span().start_pos().line_col();
        ClockAction::from_pesto(r).map(|action| LineClockAction { line, action })
    }
}

impl Pestable for ClockAction {
    fn from_pesto(r: Pair<Rule>) -> Result<Self, LineErr> {
        match r.as_rule() {
            Rule::Time => Ok(In(STime::from_pesto(r)?)),
            Rule::Clockout => Ok(Out(STime::from_pesto(r)?)),
            Rule::ClockIO => {
                let mut rc = r.into_inner();
                Ok(InOut(
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

            other => Err(TokErr::UnexpectedRule(other).on_line(r.line_num())),
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
    pub line: usize,
}

impl Display for InData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}({} - {})", self.job, self.date, self.time)
    }
}

pub fn read_clock_actions(s: &str) -> (Vec<LineClockAction>, Vec<LineErr>) {
    let p = match TimeFile::parse(Rule::Main, s) {
        Ok(mut p) => p.next().expect("Root should always have one child"),
        Err(e) => return (Vec::new(), vec![TokErr::ParseErr(e).on_line(0)]),
    };

    let mut res = Vec::new();
    let mut errs = Vec::new();

    for record in p.into_inner() {
        if record.as_rule() == Rule::EOI {
            continue;
        }
        match LineClockAction::from_pestopt(record.into_inner().next()) {
            Ok(v) => res.push(v),
            Err(e) => errs.push(e),
        }
    }
    (res, errs)
}

pub fn read_string(s: &str) -> (Vec<Clockin>, Vec<LineErr>) {
    let mut job = "General".to_string();
    let mut tags = Vec::new();
    let mut date = NaiveDate::from_ymd(1, 1, 1); //consider changing
    let mut year: Option<i32> = None;

    let mut res = Vec::new();

    let (c_ac, mut errs) = read_clock_actions(s);

    for ac in c_ac {
        match ac.action {
            SetJob(j) => job = j,
            SetDate(d, m, Some(y)) => date = NaiveDate::from_ymd(y, m, d),
            SetDate(d, m, None) => match year {
                Some(y) => date = NaiveDate::from_ymd(y, m, d),
                None => errs.push(TokErr::NotSet("date").on_line(ac.line)),
            },
            AddTag(s) => tags.push(s.clone()),
            ClearTags(Some(s)) => tags = vec![s],
            ClearTags(None) => tags.clear(),
            SetNum(k, v) => {
                if &k == "year" {
                    year = Some(v);
                }
            }
            In(time) => res.push(Clockin::In(InData {
                time,
                job: job.clone(),
                tags: tags.clone(),
                date,
                line: ac.line,
            })),

            Out(time) => res.push(Clockin::Out(time)),
            InOut(tin, tout) => {
                res.push(Clockin::In(InData {
                    time: tin,
                    job: job.clone(),
                    tags: tags.clone(),
                    date,
                    line: ac.line,
                }));
                res.push(Clockin::Out(tout));
            }
        }
    }

    (res, errs)
}
