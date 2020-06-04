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

pub fn date() -> impl Parser<Out = (usize, usize, Option<isize>)> {
    (
        common_uint,
        s_('/').ig_then(common_uint),
        maybe(s_('/').ig_then(common_int)),
    )
}

pub fn str_val() -> impl Parser<Out = String> {
    or(
        common_str,
        (Alpha.min_n(1), (Alpha, NumDigit, "_").any()).map(|(mut a, b)| {
            a.push_str(&b);
            a
        }),
    )
}

pub fn comment() -> impl Parser<Out = ()> {
    '#'.ig_then(skip_while(|c| !"\n\r,".contains(c), 0))
}

pub fn next_<P: Parser>(p: P) -> impl Parser<Out = P::Out> {
    sep(", \t\n\r".skip(), comment(), 1).ig_then(p)
}

pub fn to_end() -> impl Parser<Out = ()> {
    " ,\t\n\r".skip().ig_then(eoi)
    // read_fs(|c| ", \t\n\r".contains(c), 0).ig_then(eoi)
}

pub fn s_time() -> impl Parser<Out = STime> {
    (common_int, s_(":"), common_int).map(|(a, _, b)| STime::new(a, b))
}

pub fn line_clock_actions() -> impl Parser<Out = Vec<LineClockAction>> {
    repeat_until_ig(
        next_((line_col, clock_action())).map(|((line, col), action)| LineClockAction {
            line,
            col,
            action,
        }),
        to_end(),
    )
}

pub fn group() -> impl Parser<Out = ClockAction> {
    (
        '$',
        str_val(),
        s_('['),
        repeat_until_ig(next_(str_val()), next_("]")),
    )
        .map(|(_, k, _, v)| ClockAction::DefGroup(k, v))
}

pub fn clock_action() -> impl Parser<Out = ClockAction> {
    or5(
        or(
            //handle tags
            ('_', str_val()).map(|(_, s)| ClockAction::AddTag(s)),
            ("__", maybe(str_val())).map(|(_, os)| ClockAction::ClearTags(os)),
        ),
        or3(
            //handle time
            ('-', s_time()).map(|(_, t)| ClockAction::Out(t)),
            date().map(|(d, m, yop)| ClockAction::SetDate(d, m, yop)),
            (s_time(), maybe(('-', s_time()))).map(|(i, op)| match op {
                Some((_, out)) => ClockAction::InOut(i, out),
                None => ClockAction::In(i),
            }),
        ),
        ('=', str_val(), s_(':'), common_int).map(|(_, k, _, v)| ClockAction::SetNum(k, v)),
        group(),
        (str_val(), maybe((s_('='), common_int))).map(|(k, set)| match set {
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
        let p = str_val();
        assert_eq!(p.parse_s("hello "), Ok("hello".to_string()));
        assert_eq!(p.parse_s("hel_p_me@52"), Ok("hel_p_me".to_string()));
        assert_eq!(
            p.parse_s(r#""hello\tworld"poo "#),
            Ok("hello\tworld".to_string())
        );
        assert!(p.parse_s("_hello").is_err());
    }
}
