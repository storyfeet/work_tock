use std::collections::BTreeMap;
use std::io::Write;
use std::str::FromStr;

use chrono::naive::NaiveDate;
use chrono::offset::Local;
use chrono::{Datelike, Weekday};

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

    //core options
    let fname = cfg
        .grab()
        .fg("-f")
        .cf("config.file")
        .help("Filename: what file to look at")
        .s();

    //general options

    let week_fil = cfg
        .grab()
        .fg("-wk")
        .help("Filter -- Week Of Year: 1 to 53 or use '-' for this week")
        .s();

    let month_fil = cfg
        .grab()
        .fg("-month")
        .fg("-mth")
        .help("Filter -- Month of Year: 1 to 12")
        .s();

    let day_fil = cfg
        .grab()
        .fg("-day")
        .help("Filter -- Day: as dd/mm/yy? use '-' for today")
        .s();

    let job_fil = cfg.grab().fg("-job").help("Filter -- Job").s();

    let tag_fil = cfg.grab().fg("-tag").help("Filter -- tag").s();

    let out = cfg.grab().fg("-out").help("Clock Out").s();

    let c_in = cfg.grab().fg("-in").help("Clock In").s();

    let do_print = cfg
        .grab()
        .fg("-p")
        .help("Print out filtered results NO_ARGS")
        .is_present();

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

    let last = c_io.get(c_io.len() - 1).map(|x| x.clone());

    //filter.

    if let Some(wks) = week_fil {
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


    //local closure for month filter
    let month_s_fin = |yr,m|{
        (
            NaiveDate::from_ymd(yr,m,1),
        match m{
            12=>NaiveDate::from_ymd(yr+1,1,1),
            _=>NaiveDate::from_ymd(yr,m+1,1),
        })
    };

    if let Some(mth) = month_fil{
        let dt = Local::today();
        let (st,fin) = match mth.parse::<u32>(){
            Ok(n)=>month_s_fin(dt.year(),n),
            Err(_)=>month_s_fin(dt.year(),dt.month()),
        };
        c_io.retain(|(ind, _)| ind.date >= st && ind.date < fin);
    }

    if let Some(dt) = day_fil {
        if dt == "-" {
            let dt = Local::today().naive_local();
            c_io.retain(|(ind, _)| ind.date == dt);
        }
        //Todo
    }

    if let Some(jb) = job_fil {
        c_io.retain(|(ind, _)| ind.job == jb);
    }

    if let Some(tg) = tag_fil {
        c_io.retain(|(ind, _)| ind.tags.contains(&tg));
    }

    //build report
    let mut r_times: BTreeMap<String, STime> = BTreeMap::new();
    let mut t_time = STime::new(0, 0);
    for (idat, otime) in c_io {
        let tt = r_times
            .get(&idat.job)
            .map(|x| *x)
            .unwrap_or(STime::new(0, 0));
        t_time += otime - idat.time;
        if do_print {
            //maybe move out later
            println!("{}: {}-{} = {} => {}", idat.job, idat.time, otime, otime - idat.time, t_time);
        }
        r_times.insert(idat.job, tt + otime - idat.time);
    }

    println!("\n{:?}\n", r_times);
    println!("Total Time = {}", t_time);

    if cfg.grab().fg("-out").is_present() {
        let _data = curr.ok_or("Cannot clock out if not clocked it")?;
        let otime = match out.as_ref().map(|s| s.as_str()) {
            Some("-") | None => STime::now(),

            Some(v) => STime::from_str(v)?,
        };
        let mut f = append_to(&fname)?;
        writeln!(f, "-{}", otime).map_err(|e| format!("{:?}", e))?;
        println!("You are now Clocked out at {}", otime);
    }

    if let Some(istr) = c_in {
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
