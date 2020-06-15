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

parser! {
    (Date->(usize,usize,Option<isize>))
    (CommonUInt,last(s_('/'),CommonUInt),maybe(last(s_('/'),CommonInt)))
}

/*pub fn date() -> impl Parser<Out = (usize, usize, Option<isize>)> {
    (
        common_uint,
        s_('/').ig_then(common_uint),
        maybe(s_('/').ig_then(common_int)),
    )
}*/

parser! {
    (StrVal -> String)
    or(CommonStr,CommonIdent)
}

parser! {
    (Comment ->())
    ('#',Any.except("\n\r,").skip_star()).ig()
}

pub fn next_<P: Parser>(p: P) -> impl Parser<Out = P::Out> {
    sep(", \t\n\r".skip_star(), Comment).ig_then(p)
}

parser! {
    (ToEnd->()),
    (" ,\t\n\r".skip_star(),eoi).ig()
}

parser! {
    (STIME -> STime),
    (CommonInt, ":", CommonInt).map(|(a, _, b)| STime::new(a, b))
}

pub fn line_clock_actions() -> impl Parser<Out = Vec<LineClockAction>> {
    repeat_until_ig(
        next_((line_col, ClockACTION)).map(|((line, col), action)| LineClockAction {
            line,
            col,
            action,
        }),
        ToEnd,
    )
}

parser! {
    (Group->ClockAction)
    (
        '$',
        StrVal,
        s_('['),
        repeat_until_ig(next_(StrVal), next_("]")),
    )
        .map(|(_, k, _, v)| ClockAction::DefGroup(k, v))
}

parser! {
    (ClockACTION -> ClockAction)
    or5(
        or(
            //handle tags
            ('_', StrVal).map(|(_, s)| ClockAction::AddTag(s)),
            ("__", maybe(StrVal)).map(|(_, os)| ClockAction::ClearTags(os)),
        ),
        or3(
            //handle time
            ('-', STIME).map(|(_, t)| ClockAction::Out(t)),
            Date.map(|(d, m, yop)| ClockAction::SetDate(d, m, yop)),
            (STIME, maybe(('-', STIME))).map(|(i, op)| match op {
                Some((_, out)) => ClockAction::InOut(i, out),
                None => ClockAction::In(i),
            }),
        ),
        ('=', StrVal, s_(':'), CommonInt).map(|(_, k, _, v)| ClockAction::SetNum(k, v)),
        Group,
        (StrVal, maybe((s_('='), CommonInt))).map(|(k, set)| match set {
            Some((_, v)) => ClockAction::SetNum(k, v),
            None => ClockAction::SetJob(k),
        }),
    )
}

#[cfg(test)]
pub mod test {
    use super::*;
    #[test]
    pub fn str_val_parses_dashes() {
        assert_eq!(StrVal.parse_s("hello "), Ok("hello".to_string()));
        assert_eq!(StrVal.parse_s("hel_p_me@52"), Ok("hel_p_me".to_string()));
        assert_eq!(
            StrVal.parse_s(r#""hello\tworld"poo "#),
            Ok("hello\tworld".to_string())
        );
        assert!(StrVal.parse_s("_hello").is_err());
    }
}
