use std::collections::BTreeMap;
use std::io::Write;

use chrono::naive::NaiveDate;
use chrono::offset::Local;
use chrono::{Datelike, Weekday};

mod clockin;
use crate::clockin::Clockin;
mod parse;
mod s_time;
use crate::s_time::STime;
mod pesto;

mod err;

fn main() -> Result<(), String> {
    let mut cfg = lazy_conf::config("-c", &["{HOME}/.config/work_tock/init"])
        .map_err(|_| "Wierd Arguments")?;

    //core options
    let fname = cfg
        .grab()
        .fg("-f")
        .cf("config.file")
        .help("Filename: what file to look at")
        .s();

    //general options

    let week = cfg
        .grab()
        .fg("-wk")
        .help("Week Of Year: 1 to 53 or use '-' for this week")
        .s();

    let out = cfg.grab().fg("-out").help("Clock Out").s();

    if cfg.help("Work Tock") {
        return Ok(());
    }

    let fname = lazy_conf::env::replace_env(&fname.ok_or("No Filename provided use -f")?)
        .map_err(|_| "could not env replace")?;

    let s =
        std::fs::read_to_string(&fname).map_err(|_| format!("Could not read file: {}", fname))?;

    let (clocks, errs) = clockin::read_string(&s);

    if errs.len() > 0 {
        println!("\n\nERRS  \n{:?}", errs);
    }

    let mut curr = None;
    let mut c_io = Vec::new();
    //Get outs with ins so filter makes sense
    for c in clocks {
        match c {
            Clockin::In(data) => {
                if let Some(cin) = curr {
                    c_io.push((cin, data.time));
                }
                curr = Some(data);
            }
            Clockin::Out(cout) => {
                match curr {
                    Some(data) => c_io.push((data, cout)),
                    None => println!("Two Out's in a row"),
                }
                curr = None;
            }
        }
    }
    if let Some(data) = curr.clone() {
        println!(
            "You are clocked in for '{}'.\n You have been since {} for {} hours",
            &data.job,
            data.time,
            STime::now() - data.time
        );
        c_io.push((data.clone(), STime::now()));
    }

    //filter.

    if let Some(wks) = week {
        let dt = Local::today();
        let (st, fin) = match wks.parse::<u32>() {
            Ok(n) => (
                NaiveDate::from_isoywd(dt.year(), n, Weekday::Mon),
                NaiveDate::from_isoywd(dt.year(), n, Weekday::Sun),
            ),
            Err(_) => (
                NaiveDate::from_isoywd(dt.year(), dt.iso_week().week(), Weekday::Mon),
                NaiveDate::from_isoywd(dt.year(), dt.iso_week().week(), Weekday::Sun),
            ),
        };
        c_io.retain(|(ind, _)| ind.date >= st && ind.date <= fin);
    }

    //build report
    let mut r_times: BTreeMap<String, STime> = BTreeMap::new();
    let mut t_time = STime::new(0, 0);
    for (idat, otime) in c_io {
        let tt = r_times
            .get(&idat.job)
            .map(|x| *x)
            .unwrap_or(STime::new(0, 0));
        r_times.insert(idat.job, tt + otime - idat.time);
        t_time += otime - idat.time;
    }

    println!("{:?}", r_times);
    println!("Total Time = {}", t_time);

    if let Some(_) = out {
        if let Some(_data) = curr {
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&fname)
                .map_err(|e| format!("{:?}", e))?;
            let now = STime::now();
            write!(f, "-{}", now).map_err(|e| format!("{:?}", e))?;
            println!("You are now Clocked out at {}", now);
        } else {
            println!("Cannot clock out, if not clocked in");
        }
    }

    Ok(())
}
