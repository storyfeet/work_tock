//! Work Tock: A Time Tracking System.
//!
//! The Primary purpose of this program is to easily track your hours
//! for various different Jobs.
//!
//! I use it as a Freelancer to keep tabs on each job I have, and to track my own hours.
//!
//! The program has two main functions.
//! The primary function is to parse a Clockin file and print various records based on that.
//! The second is to make it easy to add a clocking in and out to that file.
//!
//! To tell the program where that file is, put the following in $HOME/.config/init:
//! ```bash
//! config:
//!     file:{HOME}/<path>/<to>/<intended>/<file>
//!
//! ```
//! (The "{}" Is for environment variables)
//!
//!
//! ```bash
//! #Clockin
//! work_tock -i <JobName>
//! #or
//! work_tock -i "<JobName>,<time>,<date>"
//!
//! #Clockout
//! work_tock -o
//! #or
//! work_tock --outat 13:37
//!
//! ```
//! Times are always in 24:00 hr notation
//!
//! work_tock will append your actions to the END of the file
//! and not change anything else within so everything you have written
//! is safe
//!
//! An Example File
//! ```bash
//! #Comments begin with a hash
//! #entries are separated by a comma or newline
//! #whitespace is otherwise ignored
//! 23/01/2019
//!     Carwashing,12:30-13:50
//!     15:00,#Carwashing is implied by previous Job
//!     Programming,16:00,#Clockout for Carwash is implied by new Job
//!     Eating,17:00
//!   -18:00,#Clockout
//!
//! 24/01/2019
//!     _breakfast,#Tags can be added with underscore
//!     15:00,#Eating is implied as it was the last job
//!     __,#clears all current tags.
//!   -16:00
//!
//! ```
//!
//! running work_tock on the above file will produce:
//! ```bash
//! {"Carwashing": 02:20, "Eating": 02:00, "Programming": 01:00}
//!
//! Total Time = 05:20
//! ```
//!
//!

extern crate work_tock_lib;

use work_tock_lib::{
    clockin, ClockAction, Clockin, LineClockAction, Pestable, Rule, STime, TokErr,
};

use std::collections::BTreeMap;
use std::io::Write;
use std::str::FromStr;

use chrono::naive::NaiveDate;
use chrono::offset::Local;
use chrono::{Datelike, Weekday};

use clap_conf::*;

fn append_to(fname: &str) -> Result<std::fs::File, failure::Error> {
    std::fs::OpenOptions::new()
        .append(true)
        .open(&fname)
        .map_err(|e| e.into())
}

fn main() -> Result<(), failure::Error> {
    let clap = clap_app!(
        work_tock=>
            (version: crate_version!())
            (author: "Matthew Stoodley")
            (about: "Clock in and out of work")
            (@arg config: -c "Config File") //allow clap_conf config loader to work
            (@arg file: -f --file +takes_value "Filename")
            (@arg week:  --week +takes_value "Filter by Week.")
            (@arg this_week: -w "Filter by this week")
            //(@arg on_date: --date +takes_value "Filter by date.")
            (@arg today: -d "Filter by Today")
            (@arg month: --month +takes_value "Filter by Month 1--12.")
            (@arg this_month: -m "Filter by this month")
            (@arg print: -p "Print Filtered Results nicely")
            (@arg clockin: -i --in +takes_value "Clock in")
            (@arg clockout: -o --out "Clock out Now")
            (@arg clockoutat: --outat +takes_value "Clock out at given time")
            (@arg long_day: -l --long_day "Acknowledge working past midnight")
            (@arg same_day:-s --same_day "Clockout on the same day as the clockin")

            (@arg since: --since +takes_value "Filter Since given date (inclusive)")
            (@arg until: --until +takes_value "Filter until given date (inclusive)")
            (@arg job: --job +takes_value "Filter by Job")
            (@arg jobstart: --job_s +takes_value "Filter by Job Starts with")
            (@arg tag: --tag +takes_value "Filter by Tag")
    )
    .get_matches();

    let cfg = clap_conf::with_toml_env(&clap, &["{HOME}/.config/work_tock/init.toml"]);
    //core options
    let fname = cfg
        .grab()
        .arg("file")
        .conf("config.file")
        .rep_env()
        .expect("No File given");

    let s = std::fs::read_to_string(&fname)?; //.map_err(|_| format!("Could not read file: {}", fname))?;

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
                    Some(data) => {
                        if cout < data.time {
                            return Err(TokErr::NegativeTime.on_line(data.line).into());
                        }
                        c_io.push((data, cout));
                    }
                    None => println!("Two Out's in a row"),
                }
                curr = None;
            }
        }
    }

    //Data all created time to check things
    let today = Local::now().date().naive_local();

    if let Some(data) = curr.clone() {
        let since_last = STime::since(data.time, &data.date);
        let sdate = if today == data.date {
            "".to_string()
        } else {
            data.date.to_string()
        };
        println!(
            "You have been clocked in for {} since {} {} for {} hours",
            data.job, sdate, data.time, since_last,
        );
        let otime = since_last + data.time;
        c_io.push((data, otime));
    }

    let last_entry = c_io.get(c_io.len() - 1).map(|x| x.clone());

    //filter.

    if cfg.bool_flag("this_week", Filter::Arg) {
        let dt = Local::today();
        let wk = dt.iso_week().week();
        let st = NaiveDate::from_isoywd(dt.year(), dt.iso_week().week(), Weekday::Mon);
        let fin = NaiveDate::from_isoywd(dt.year(), dt.iso_week().week(), Weekday::Sun);
        println!("Filtering by week {}", wk);
        c_io.retain(|(ind, _)| ind.date >= st && ind.date <= fin);
    }

    if let Some(wks) = cfg.grab().arg("week").done() {
        let dt = Local::today();
        let wk = wks.parse::<u32>()?;
        //.map_err(|_| "Could not parse week value")?;
        let st = NaiveDate::from_isoywd(dt.year(), wk, Weekday::Mon);
        let fin = NaiveDate::from_isoywd(dt.year(), wk, Weekday::Sun);
        println!("Filtering by week {}", wk);
        c_io.retain(|(ind, _)| ind.date >= st && ind.date <= fin);
    }

    //local closure for month filter
    let month_s_fin = |yr, m| {
        (
            NaiveDate::from_ymd(yr, m, 1),
            match m {
                12 => NaiveDate::from_ymd(yr + 1, 1, 1),
                _ => NaiveDate::from_ymd(yr, m + 1, 1),
            },
        )
    };

    if cfg.bool_flag("this_month", Filter::Arg) {
        let dt = Local::today();
        let (st, fin) = month_s_fin(dt.year(), dt.month());
        c_io.retain(|(ind, _)| ind.date >= st && ind.date < fin);
    }

    if let Some(mth) = cfg.grab().arg("month").done() {
        let dt = Local::today();
        let (st, fin) = month_s_fin(dt.year(), mth.parse()?);
        c_io.retain(|(ind, _)| ind.date >= st && ind.date < fin);
    }

    //TODO filter by given date
    if cfg.bool_flag("today", Filter::Arg) {
        let dt = Local::today().naive_local();
        println!("Filtering by Today");
        c_io.retain(|(ind, _)| ind.date == dt);
    }

    if let Some(d) = cfg.grab().arg("since").done() {
        let dt = LineClockAction::pest_parse(Rule::Date, &d)?
            .action
            .as_date()
            .ok_or(TokErr::from("Could not read since date"))?;
        c_io.retain(|(ind, _)| ind.date >= dt);
    }

    if let Some(d) = cfg.grab().arg("until").done() {
        let lc = LineClockAction::pest_parse(Rule::Date, &d)?;
        let dt = lc
            .action
            .as_date()
            .ok_or(TokErr::from("Could not read since date"))?;
        c_io.retain(|(ind, _)| ind.date <= dt);
    }

    if let Some(jb) = cfg.grab().arg("job").done() {
        c_io.retain(|(ind, _)| ind.job == jb);
    }

    if let Some(jbs) = cfg.grab().arg("jobstart").done() {
        c_io.retain(|(ind, _)| ind.job.starts_with(&jbs));
    }

    if let Some(tg) = cfg.grab().arg("tag").done() {
        c_io.retain(|(ind, _)| ind.tags.contains(&tg.to_string()));
    }

    //build report
    let mut r_times: BTreeMap<String, STime> = BTreeMap::new();
    let mut t_time = STime::new(0, 0);
    let mut last_dat = NaiveDate::from_ymd(1, 1, 1);
    for (idat, otime) in c_io {
        let tt = r_times
            .get(&idat.job)
            .map(|x| *x)
            .unwrap_or(STime::new(0, 0));
        t_time += otime - idat.time;
        if cfg.bool_flag("print", Filter::Arg) {
            //maybe move out later
            if last_dat != idat.date {
                println!("{}", idat.date.format("%d/%m/%Y"));
                last_dat = idat.date;
            }
            println!(
                "  {}: {}-{} = {}   => {}",
                idat.job,
                idat.time,
                otime,
                otime - idat.time,
                t_time
            );
        }
        r_times.insert(idat.job, tt + otime - idat.time);
    }

    println!("\n{:?}\n", r_times);
    println!("Total Time = {}", t_time);

    if cfg.bool_flag("clockout", Filter::Arg) {
        let c_data = curr
            .as_ref()
            .ok_or(TokErr::from("Cannot clock out if not clocked in"))?;

        if today > last_dat && !cfg.bool_flag("long_day", Filter::Arg) {
            return Err(TokErr::from(
                "Last Clockin was not today please use -l to confirm long day",
            )
            .into());
        }

        let since = STime::since(c_data.time, &c_data.date);
        let mut f = append_to(&fname)?;
        writeln!(f, "  -{}", since + c_data.time)?; //.map_err(|e| format!("{:?}", e))?;
        println!("You are now Clocked out at {}", since + c_data.time);
    }

    if let Some(tm) = cfg.grab().arg("clockoutat").done() {
        let c_data = curr
            .as_ref()
            .ok_or(TokErr::from("Cannot clock out if not clocked in"))?;
        let otime = STime::from_str(&tm)?;

        if c_data.date != today
            && otime < STime::new(24, 0)
            && !cfg.bool_flag("same_day", Filter::Arg)
        {
            return Err(TokErr::from("The clockin was on a different date, please use -s to confirm clockout on that day, or add n*(24 hours) to time for long day clockout").into());
        }
        let mut f = append_to(&fname)?;
        writeln!(f, "  -{}", otime)?; //.map_err(|e| format!("{:?}", e))?;
        println!("You are now Clocked out at {}", otime);
    }

    if let Some(istr) = cfg.grab().arg("clockin").done() {
        //first check that we are not clockedin on a different date
        if let Some(c_data) = curr.as_ref() {
            if c_data.date != today {
                return Err(TokErr::from("You are currently clocked in from a different date, Please clockout from that before clocking in").into());
            }
        }

        //then build the query
        let mut new_date = Local::today().naive_local();
        let mut new_time = STime::now();
        let mut new_job: Option<String> = None;
        let (acs, errs) = clockin::read_clock_actions(&istr);
        if errs.len() > 0 {
            println!("Clockin format errors : {:?}", errs);
        } else {
            for ac in acs {
                match ac.action {
                    ClockAction::In(t) => new_time = t,
                    ClockAction::SetJob(j) => new_job = Some(j),
                    ClockAction::SetDate(d, m, Some(y)) => new_date = NaiveDate::from_ymd(y, m, d),
                    ClockAction::SetDate(d, m, None) => {
                        new_date = match last_entry.as_ref() {
                            Some(ref l) => NaiveDate::from_ymd(l.0.date.year(), m, d),
                            None => NaiveDate::from_ymd(new_date.year(), m, d),
                        }
                    }
                    other => println!("Option not handled {:?}", other),
                }
            }
            let mut line = new_date.format("%d/%m/%Y\n\t").to_string();

            if let Some(ref l) = last_entry.as_ref() {
                if new_date == l.0.date {
                    line = "\t".to_string();
                }
                if let Some(ref nj) = new_job {
                    if *nj != l.0.job {
                        line.push_str(nj);
                        line.push(',');
                    }
                }
            } else {
                if let Some(ref nj) = new_job {
                    line.push_str(nj);
                    line.push(',');
                }
            }

            line.push_str(&new_time.to_string());
            println!("Adding: {}", line);
            let mut f = append_to(&fname)?;
            writeln!(f, "{}", line)? //.map_err(|e| format!("{:?}", e))?;
        }
    }

    Ok(())
}
