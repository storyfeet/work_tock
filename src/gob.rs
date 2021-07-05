use crate::clockin::{ClockAction, LineClockAction};
use crate::s_time::STime;
use gobble::*;

parser! {
    (Date->(usize,usize,Option<isize>))
    (common::UInt,last(ws__('/'),common::UInt),maybe(last(ws__('/'),common::Int)))
}

parser! {
    (StrVal -> String)
    or(common::Quoted,common::Ident)
}

parser! {
    (Comment ->())
    ('#',Any.except("\n\r,").istar()).ig()
}

pub fn next_<P: Parser>(p: P) -> impl Parser<Out = P::Out> {
    sep_star(", \t\n\r".istar(), Comment).ig_then(p)
}

parser! {
    (ToEnd->()),
    (" ,\t\n\r".istar(),eoi).ig()
}

parser! {
    (STIME -> STime),
    (common::Int, ":", common::Int).map(|(a, _, b)| STime::new(a, b))
}

pub fn line_clock_actions() -> impl Parser<Out = Vec<LineClockAction>> {
    star_until_ig(
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
        ws__('['),
        star_until_ig(next_(StrVal), next_("]")),
    )
        .map(|(_, k, _, v)| ClockAction::DefGroup(k, v))
}

parser! {
    (ClockACTION -> ClockAction)
    or!(
        //handle tags
        ('_', StrVal).map(|(_, s)| ClockAction::AddTag(s)),
        ("__", maybe(StrVal)).map(|(_, os)| ClockAction::ClearTags(os)),
        //handle time
        ('-', STIME).map(|(_, t)| ClockAction::Out(t)),
        Date.map(|(d, m, yop)| ClockAction::SetDate(d, m, yop)),
        (STIME, maybe(('-', STIME))).map(|(i, op)| match op {
            Some((_, out)) => ClockAction::InOut(i, out),
            None => ClockAction::In(i),
        }),
        ('=', StrVal, ws__(':'), common::Int).map(|(_, k, _, v)| ClockAction::SetNum(k, v)),
        Group,
        (StrVal, maybe((ws__('='), common::Int))).map(|(k, set)| match set {
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
