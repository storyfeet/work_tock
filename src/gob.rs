use crate::clockin::{ClockAction, LineClockAction};
use crate::s_time::STime;
use gobble::*;

/*
#[derive(Debug)]
pub enum ClockAction {
    AddTag(String),            //TODO deprecate
    ClearTags(Option<String>), //replacement tag
    In(STime),
    Out(STime),
    InOut(STime, STime),
    SetJob(String),
    SetDate(u32, u32, Option<i32>),
    SetNum(String, i32),
}
*/

pub fn str_val() -> impl Parser<String> {
    or(common_str(), read_fs(is_alpha_num, 1))
}

pub fn s_time() -> impl Parser<STime> {
    (common_int, ":", common_int).map(|(a, _, b)| STime::new(a, b))
}

pub fn clock_action() -> impl Parser<ClockAction> {
    or3(
        ('_', str_val()).map(|(_, s)| ClockAction::AddTag(s)),
        ("__", maybe(str_val())).map(|(_, os)| ClockAction::ClearTags(os)),
        ('-', s_time()).map(|(_, t)| ClockAction::Out(t)),
    )
}
