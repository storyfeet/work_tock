use std::collections::BTreeMap;
use std::io::Write;
use std::str::FromStr;

use chrono::naive::NaiveDate;
use chrono::offset::Local;
use chrono::{Datelike, Weekday};

use clap::clap_app;
mod clockin;
use crate::clockin::{ClockAction, Clockin};
mod s_time;
use crate::s_time::STime;
mod pesto;
//use crate::pesto::{TimeFile,Rule};

mod err;
use err::TokErr;

fn append_to(fname: &str) -> Result<std::fs::File, String> {
    std::fs::OpenOptions::new()
        .append(true)
        .open(&fname)
        .map_err(|e| format!("{:?}", e))
}

fn main() -> Result<(), TokErr> {
    let mut cfg = lazy_conf::config("-c", &["{HOME}/.config/work_tock/init"])
        .map_err(|_| "Wierd Arguments")?;

    //note this currently runs as filter will eventually replace lazyconf, but only bit at a time
    let matches = clap_app!(
        work_tock=>
            (version: clap::crate_version!())
            (author: "Matthew Stoodley")
            (about: "Clock in and out of work")
            (@arg conf: -c "Config File") //allow lazyconf config loader to work
            (@arg file: -f --file +takes_value "Filename")
            (@arg week:  --week +takes_value "Filter by Week.")
            (@arg this_week: -w "Filter by this week")
            //(@arg on_date: --date +takes_value "Filter by date.")
            (@arg today: -d "Filter by Today")
            (@arg print: -p "Print Filtered Results nicely")
            (@arg clockin: -i --in +takes_value "Clock in")
            (@arg clockout: -o --out "Clock out Now")
            (@arg clockoutat: --outat +takes_value "Clock out at given time")
            (@arg job: --job +takes_value "Filter by Job")
            (@arg tag: --tag +takes_value "Filter by Tag")
    )
    .get_matches();

    //core options
    let fname = cfg.grab().cf("config.file").s();

    let month_fil = cfg
        .grab()
        .fg("-month")
        .fg("-mth")
        .help("Filter -- Month of Year: 1 to 12")
        .s();

    //mashing two systems together not so fun
    let fname = matches.value_of("file").map(|s| s.to_string()).unwrap_or(
        lazy_conf::env::replace_env(&fname.ok_or("No Filename provided use -f")?)
            .map_err(|_| "no env")?,
    );

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

    let last = c_io.get(c_io.len() - 1).map(|x| x.clone());

    //filter.

    if matches.is_present("this_week") {
        let dt = Local::today();
        let wk = dt.iso_week().week();
        let st = NaiveDate::from_isoywd(dt.year(), dt.iso_week().week(), Weekday::Mon);
        let fin = NaiveDate::from_isoywd(dt.year(), dt.iso_week().week(), Weekday::Sun);
        println!("Filtering by week {}", wk);
        c_io.retain(|(ind, _)| ind.date >= st && ind.date <= fin);
    }

    if let Some(wks) = matches.value_of("week") {
        let dt = Local::today();
        let wk = wks
            .parse::<u32>()
            .map_err(|_| "Could not parse week value")?;
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

    if let Some(mth) = month_fil {
        let dt = Local::today();
        let (st, fin) = match mth.parse::<u32>() {
            Ok(n) => month_s_fin(dt.year(), n),
            Err(_) => month_s_fin(dt.year(), dt.month()),
        };
        c_io.retain(|(ind, _)| ind.date >= st && ind.date < fin);
    }

    //TODO filter by given date
    if matches.is_present("today") {
        let dt = Local::today().naive_local();
        println!("Filtering by Today");
        c_io.retain(|(ind, _)| ind.date == dt);
    }

    if let Some(jb) = matches.value_of("job") {
        c_io.retain(|(ind, _)| ind.job == jb);
    }

    if let Some(tg) = matches.value_of("tag") {
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
        if matches.is_present("print") {
            //maybe move out later
            if last_dat != idat.date {
                println!("{}", idat.date.format("%Y/%m/%d"));
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

    if matches.is_present("clockout") {
        let _data = curr.as_ref().ok_or("Cannot clock out if not clocked it")?;
        let otime = STime::now();
        let mut f = append_to(&fname)?;
        writeln!(f, "-{}", otime).map_err(|e| format!("{:?}", e))?;
        println!("You are now Clocked out at {}", otime);
    }

    if let Some(tm) = matches.value_of("clockoutat") {
        let _data = curr.ok_or("Cannot clock out if not clocked in")?;
        let otime = STime::from_str(tm)?;
        let mut f = append_to(&fname)?;
        writeln!(f, "-{}", otime).map_err(|e| format!("{:?}", e))?;
        println!("You are now Clocked out at {}", otime);
    }

    if let Some(istr) = matches.value_of("clockin") {
        let mut new_date = Local::today().naive_local();
        let mut new_time = STime::now();
        let mut new_job: Option<String> = None;
        let (acs, errs) = clockin::read_clock_actions(&istr);
        if errs.len() > 0 {
            println!("Clockin format errors : {:?}", errs);
        } else {
            for ac in acs {
                match ac {
                    ClockAction::In(t) => new_time = t,
                    ClockAction::SetJob(j) => new_job = Some(j),
                    ClockAction::SetDate(d, m, Some(y)) => new_date = NaiveDate::from_ymd(y, m, d),
                    ClockAction::SetDate(d, m, None) => {
                        new_date = match last {
                            Some(ref l) => NaiveDate::from_ymd(l.0.date.year(), m, d),
                            None => NaiveDate::from_ymd(new_date.year(), m, d),
                        }
                    }
                    other => println!("Option not handled {:?}", other),
                }
            }
            let mut line = "".to_string();
            if let Some(ref l) = last {
                if new_date != l.0.date {
                    line.push_str(&new_date.format("%d/%m/%Y,").to_string());
                }
                if let Some(ref nj) = new_job {
                    if *nj != l.0.job {
                        line.push_str(nj);
                        line.push(',');
                    }
                }
            } else {
                line.push_str(&new_date.format("%d/%m/%Y,").to_string());
                if let Some(ref nj) = new_job {
                    line.push_str(nj);
                    line.push(',');
                }
            }

            line.push_str(&new_time.to_string());
            println!("Adding: {}", line);
            let mut f = append_to(&fname)?;
            writeln!(f, "{}", line).map_err(|e| format!("{:?}", e))?;
        }
    }

    Ok(())
}
