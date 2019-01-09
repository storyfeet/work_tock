use chrono::naive::NaiveDate;
use derive_more::*;

use crate::err::{must, TokErr};
use crate::parse::{Token, Tokeniser};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Constructor, Add, Sub, AddAssign, SubAssign)]
pub struct STime {
    hour: i32,
    min: i32,
}

#[derive(Debug)]
pub enum ClockAction {
    NoAction,
    AddTag(String, bool), //clear on true
    ClearTags,
    AddClockin(STime, bool), // bool = is_in
    SetJob(String),
    SetDate(NaiveDate),
}

use self::ClockAction::*;

#[derive(Clone,Debug, PartialEq, Eq)]
pub struct Clockin {
    time: STime,
    is_in: bool,
    date: NaiveDate,
    job: String,
    tags: Vec<String>,
}

//Note TokErr::EOF is used when file ends is not a bad error.
fn read_break(p: &mut Tokeniser) -> Result<ClockAction, TokErr> {
    use self::TokErr::UnexpectedEOF as U_EOF;
    match p.next().ok_or(TokErr::EOF)? {
        Token::Break => Ok(NoAction),
        Token::Str(s) => Ok(SetJob(s)),
        Token::Dash => {
            let hr = p.next().ok_or(U_EOF)?.as_num()?;
            must(p.next() == Some(Token::Colon), "Use ':' to set minutes")?;
            let min = p.next().ok_or(U_EOF)?.as_num()?;
            Ok(AddClockin(STime::new(hr, min), false))
        }
        Token::Num(n) => {
            match p.next().ok_or(TokErr::UnexpectedEOF)? {
                Token::Colon => {
                    let n2 = p.next().ok_or(TokErr::UnexpectedEOF)?.as_num()?;
                    Ok(AddClockin(STime::new(n, n2), true))
                }
                Token::Slash => {
                    let month = p.next().ok_or(TokErr::UnexpectedEOF)?.as_num()?;
                    must(
                        p.next() == Some(Token::Slash),
                        "Use '/' to separate date items dd/mm/yyyy",
                    )?;
                    let year = p.next().ok_or(TokErr::UnexpectedEOF)?.as_num()?; //replace with test on =year
                    Ok(SetDate(NaiveDate::from_ymd(year, month as u32, n as u32)))
                    //date
                }
                other => Err(TokErr::Mess(format!(
                    "use / for date, and : for time. not '{:?}'",
                    other
                ))),
            }
        }
        Token::DUScore => match p.next().ok_or(TokErr::EOF)? {
            Token::Str(s) => Ok(AddTag(s, true)),
            Token::Break => Ok(ClearTags),
            other => Err(TokErr::Mess(format!("unexpected {:?}", other))),
        },
        Token::UScore => match p.next().ok_or(TokErr::UnexpectedEOF)? {
            Token::Str(s) => Ok(AddTag(s, false)),
            other => Err(TokErr::Mess(format!("unexpected {:?}", other))),
        },
        other => Err(TokErr::Mess(format!("Failed at {:?}", other))),
    }
}

pub fn read_string(s: &str) -> (Vec<Clockin>, Vec<TokErr>) {
    let mut pk = Tokeniser::new(s);

    let mut job: Option<String> = None;
    let mut date: Option<NaiveDate> = None;
    let mut tags: Vec<String> = Vec::new();

    let mut res = Vec::new();
    let mut errs = Vec::new();

    loop {
        match read_break(&mut pk) {
            Ok(SetJob(s)) => job = Some(s),
            Ok(AddTag(t, fresh)) => {
                if fresh {
                    tags.clear();
                }
                tags.push(t);
            }
            Ok(SetDate(d)) => date = Some(d),
            Ok(NoAction) => {}
            Ok(ClearTags) => tags.clear(),
            Ok(AddClockin(time, is_in)) => {
                if let Some(date) = date {
                    res.push(Clockin {
                        time,
                        is_in,
                        date,
                        job: job.clone().unwrap_or("GENERAL".to_string()),
                        tags: tags.clone(),
                    });
                }
            }
            //Ok(a)=> errs.push(TokErr::Mess(format!("Not yet coded {:?}",a))),
            Err(TokErr::EOF) => {
                break;
            }
            Err(e) => {
                errs.push(e);
            }
        }
    }
    (res, errs)
}
