use chrono::naive::NaiveDate;
use chrono::offset::Local;
use chrono::Datelike;
//use pest::iterators::Pair;
//use pest::Parser;
use crate::gob;
use gobble::Parser;
use std::collections::BTreeMap;
use std::fmt::Display;

use crate::err::TokErr;
//use crate::pesto::{LineNum, Pestable, Rule, TimeFile};
use crate::s_time::STime;

#[derive(Debug)]
pub struct LineClockAction {
    pub line: usize,
    pub col: usize,
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
    SetDate(usize, usize, Option<isize>),
    SetNum(String, isize),
    DefGroup(String, Vec<String>),
}

use self::ClockAction::*;

pub fn read_date(s: &str) -> Result<NaiveDate, TokErr> {
    let (d, m, yop) = gob::date()
        .parse_s(s)
        .map_err(|_| TokErr::Mess("Could not read date".to_string()))?;
    Ok(NaiveDate::from_ymd(
        yop.map(|y| y as i32).unwrap_or(Local::today().year()),
        m as u32,
        d as u32,
    ))
}

impl ClockAction {
    pub fn as_date(&self) -> Option<NaiveDate> {
        match self {
            ClockAction::SetDate(d, m, Some(y)) => {
                Some(NaiveDate::from_ymd(*y as i32, *m as u32, *d as u32))
            }
            ClockAction::SetDate(d, m, None) => {
                let date = Local::today();
                Some(NaiveDate::from_ymd(date.year(), *m as u32, *d as u32))
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Clockin {
    In(InData),
    Out(STime),
}

pub struct AllData {
    pub clocks: Vec<Clockin>,
    pub groups: BTreeMap<String, Vec<String>>,
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

pub fn read_string(s: &str) -> Result<AllData, TokErr> {
    let mut job = "General".to_string();
    let mut tags = Vec::new();
    let mut date = NaiveDate::from_ymd(1, 1, 1); //consider changing
    let mut year: Option<isize> = None;

    let mut c_res = Vec::new();
    let mut groups = BTreeMap::new();

    let c_ac = gob::line_clock_actions().parse_s(s)?;
    let mut errs = Vec::new();

    for ac in c_ac {
        match ac.action {
            SetJob(j) => job = j,
            SetDate(d, m, Some(y)) => date = NaiveDate::from_ymd(y as i32, m as u32, d as u32),
            SetDate(d, m, None) => match year {
                Some(y) => date = NaiveDate::from_ymd(y as i32, m as u32, d as u32),
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
            In(time) => c_res.push(Clockin::In(InData {
                time,
                job: job.clone(),
                tags: tags.clone(),
                date,
                line: ac.line,
            })),

            Out(time) => c_res.push(Clockin::Out(time)),
            InOut(tin, tout) => {
                c_res.push(Clockin::In(InData {
                    time: tin,
                    job: job.clone(),
                    tags: tags.clone(),
                    date,
                    line: ac.line,
                }));
                c_res.push(Clockin::Out(tout));
            }
            DefGroup(k, v) => {
                groups.insert(k, v);
            }
        }
    }

    if errs.len() > 0 {
        Err(TokErr::Lines(errs))
    } else {
        Ok(AllData {
            clocks: c_res,
            groups: groups,
        })
    }
}
