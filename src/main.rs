//!
//!
//!   Work Tock
//!   ==========
//!
//!   A command line work tracking program.
//!
//!   Use --help to get basic usage on the command line
//!
//!   Basic Usage
//!   ----------
//!
//!   You can use this program to clockin
//!
//!       work_tock -i <JobName>
//!
//!   clockout
//!
//!       work_tock -o
//!
//!   Or print a record of recent clock entries with flags
//!
//!       work_tock --job_s dothing -p --since 03/04/2020
//!
//!
//!   The program works with a single text file that is easy to edit if needed. The program will never overwrite your file, only read and append, so all data in that file will otherwise remain untouched.
//!
//!   To set the location of the core file, the default config for your program can be found in "$HOME/.config/work\_tock/init.toml 
//!
//!   ```toml
//!   [config]
//!       # Set path the the current working file 
//!       # anything within "{}" is read as an environment variable
//!       file="{HOME}/<path>/<to>/<file>"
//!
//!       #Optional require all job entries to be snake_case
//!       snake=true  
//!       
//!       #camel=true  #if you prefer camelCase
//!   ```
//!
//!   A standard file looks like this :
//!
//!   ```toml
//!   $home_jobs[car_wash,eat]
//!   23/01/2019
//!       car_wash,12:30-13:50
//!       15:00,#car_wash is implied by previous Job
//!       programming,16:00,#Clockout for car_wash is implied by new Job
//!       eat,17:00
//!     -18:00,#Clockout
//!
//!   24/01/2019
//!       12:00,#Eating is implied as it was the last job
//!     -13:00
//!       programming,14:00
//!     -16:00
//!   ```
//!
//!   * Commas and newlines begin a new item
//!   * Whitespace is ignored next to either side or a break (",","\n")
//!   * Jobs are Letters only
//!   * Clockins are "hh:mm"
//!   * Clockouts are  "-hh:mm"
//!   * Tags begin with an "\_" and can be cleared with "\_\_"
//!   * Dates are dd/mm/yyyy, but if you set year=2019, dates can be dd/mm after that.
//!   * Groups are defined by a ```$group_name[list,of,jobs,in,group]```
//!
//!   Every Clockin will use the most recent Job,Date, and Tags for the clocking, 
//!
//!   So given the example file ```work_tock``` will produce:
//!
//!   ```toml
//!   {"car_wash": 02:20, "eat": 04:00, "programming": 01:00}
//!
//!   Total Time = 07:20
//!
//!   ```
//!
//!   Printing and Filters
//!   ------------------
//!
//!   Using "-p" Will print all entries, but if you want to be morse spcific you can apply a filter and -p will print only entries that pass that filter.
//!
//!   > NOTE: -t for today used to be -d which is now used to specify a date to work on
//!
//!   To get more relevent data you can use filters such as "-t" :Today, or "--day 3/1/2019", or by job 
//!
//!   eg: ```work_tock -p --job car_wash``` will return
//!
//!   ```toml
//!   23/01/2019
//!     car_wash: 12:30-13:50 = 01:20   => 01:20
//!     car_wash: 15:00-16:00 = 01:00   => 02:20
//!
//!   {"car_wash": 02:20}
//!
//!   Total Time = 02:20
//!
//!   ```
//!
//!   or ```work_tock -p --group home_jobs``` will produce:
//!
//!   ```toml
//!   Filtering by group home_jobs
//!   23/01/2019
//!     car_wash: 12:30-13:50 = 01:20   => 01:20
//!     car_wash: 15:00-16:00 = 01:00   => 02:20
//!     eat: 17:00-18:00 = 01:00   => 03:20
//!   24/01/2019
//!     eat: 12:00-13:00 = 01:00   => 04:20
//!
//!   {"car_wash": 02:20, "eat": 02:00}
//!
//!   Total Time = 04:20
//!
//!   ```
//!
//!
//!
//!
//!   For more information use ```work_tock --help```
//!


extern crate work_tock_lib;

use work_tock_lib::{
    clockin,  Clockin, ClockAction,    STime, TokErr, gob
};

use gobble::Parser;

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
            (@arg attime:-a +takes_value "perform activity at given time")
            (@arg ondate:-d +takes_value "perform activity on given date")
            (@arg file: -f --file +takes_value "Filename")
            (@arg week:  --week +takes_value "Filter by Week.")
            (@arg this_week: -w "Filter by this week")
            //(@arg on_date: --date +takes_value "Filter by date.")
            (@arg today: -t "Filter by Today")
            (@arg month: --month +takes_value "Filter by Month 1--12.")
            (@arg this_month: -m "Filter by this month")
            (@arg print: -p "Print Filtered Results nicely")
            (@arg clockin: -i --in +takes_value "Clock in to named job, (comma separate clockin time if not for now)")
            (@arg quickin: -q "Clock in now to previous job")
            (@arg clockout: -o --out "Clock out Now")
            (@arg clockoutat: --outat +takes_value "Clock out at given time")
            (@arg long_day: -l --long_day "Acknowledge working past midnight")
            (@arg yesterday: -y --yesterday "go back one day equivilat to -d <the day before>")
            (@arg same_day:-s --same_day "Clockout on the same day as the clockin")

            (@arg since: --since +takes_value "Filter Since given date (inclusive)")
            (@arg until: --until +takes_value "Filter until given date (inclusive)")
            (@arg job: --job +takes_value "Filter by Job")
            (@arg group: -g --group + takes_value "Filter by group")
            (@arg jobstart: --job_s +takes_value "Filter by Job Starts with")
            (@arg tag: --tag +takes_value "Filter by Tag")
            (@arg camel: --camel "Force Camel Case on job input")
            (@arg snake: --snake "Force Camel Case on job input")
            (@subcommand complete  => )
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

    //Tab Complete list
    if let Some(_sc) = clap.subcommand_matches("complete"){
        let mut mp = std::collections::BTreeSet::new();
        let line_actions = gob::line_clock_actions().parse_s(&s).map_err(|e|e.strung())?;
        for a in line_actions {
            match a.action{
                ClockAction::SetJob(d)=>{
                    mp.insert(d);
            
                }
                _=>{}
            }
        }
        for k in mp {
            print!(" {}",k);
        }
        println!("");
        return Ok(()) ;
    }



    let clock_data = match clockin::read_string(&s){
        Ok(c)=>c,
        Err(e)=> {
            println!("\n\n Errs : \n");
            return Err(e.into());
        }
    };
   


    let mut curr = None;
    let mut c_io = Vec::new();
    //Get outs with ins so filter makes sense
    //If currently clocked in leaves curr as an option to be added later
    for c in clock_data.clocks {
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

    let today = match cfg.grab().arg("ondate").done(){
        Some(s)=>clockin::read_date(&s)?,
        None=>Local::now().date().naive_local(),
    };
    let today = match cfg.bool_flag("yesterday",Filter::Arg){
        true => today- chrono::Duration::days(1),
        false => today,
    };
    
    
    let now = match cfg.grab().arg("attime").done(){
        Some(s)=>STime::from_str(&s)?,
        None=>STime::now(),
    };
        

    if cfg.bool_flag("clockout", Filter::Arg) {
        let c_data = curr
            .take()
            .ok_or(TokErr::from("Cannot clock out if not clocked in"))?;

        if today > c_data.date && !cfg.bool_flag("long_day", Filter::Arg) {
            return Err(TokErr::from(format!(
                "Last Clockin was not today: {}. Please use -l to confirm long day",
                c_data
            ))
            .into());
        }

        let since = now.since(&today,c_data.time, &c_data.date);
        if since < STime::new(0,0) {
            return Err(TokErr::from(format!("Cannot clockout before clockin")).into());
        }
        let mut f = append_to(&fname)?;
        let otime = since + c_data.time;

        writeln!(f, "  -{}", otime)?; //.map_err(|e| format!("{:?}", e))?;
        println!("You are now clocking out from {} at {}", c_data, otime);
        c_io.push((c_data, otime));
    }

    if let Some(_tm) = cfg.grab().arg("clockoutat").done() {
        println!(r#""--outat <time>" has been replaced by "-o -a <time>""#)
    }

    let mut clockin = None;

    if cfg.bool_flag("quickin", Filter::Arg) {
        clockin = Some(c_io.get(c_io.len() - 1).map(|x|x.0.job.clone()).ok_or(TokErr::from("no previous job"))?);
    }

    if let Some(istr) = cfg.grab().arg("clockin").done() {
        //camel case required
        let f_camel = cfg.bool_flag("camel",Filter::Arg) || cfg.bool_flag("config.camel",Filter::Conf);
        if f_camel && istr.contains("_"){
            return Err(TokErr::from("You have required CamelCase job entires").into());
        }
        //snake case required{
        let f_snake = cfg.bool_flag("snake",Filter::Arg) || cfg.bool_flag("config.snake",Filter::Conf);
        if f_snake && istr.chars().find(|c|c.is_uppercase()).is_some(){
            return Err(TokErr::from("You have required snake_case job entires").into());

        }
        clockin = Some(istr);

    }

    if let Some(job) = clockin{
        //first check that we are not clockedin on a different date
        if let Some(c_data) = curr.take() {
            if c_data.date != today {
                return Err(TokErr::from("You are currently clocked in from a different date, Please clockout from that before clocking in.").into());
            }
            let since = now.since(&today,c_data.time, &c_data.date);
            if since < STime::new(0,0) {
                return Err(TokErr::from("You are currently clocked in since after the given time. Cannot clockout before clocking in").into());

            }
            println!("You are now clocking out from {} at {} ({}hrs)", c_data, now,since);
            c_io.push((c_data, now));
        }
        
        let real_today = Local::now().date().naive_local();
        let date_str = if real_today != today {
            today.format("on %d/%m/%Y").to_string()
        } else {
            "today".to_string()
        };
        println!(// message 
            "You are now clocking in {} at {} for \"{}\"",
            date_str,
            now,
            job
        );

        let lastjob = c_io.get(c_io.len() - 1).map(|x| x.clone().0);//Option
        let f_line = match lastjob {
            Some(lj) =>{
                let mut f = if lj.date != today{
                    today.format("%d/%m/%Y\n\t").to_string()
                }else { "\t".to_string()};
                if lj.job != job {
                    f.push_str(&format!("{},",do_quotes(&job)));
                }
                f.push_str(&now.to_string());
                f
            }
            None=>{
                format!("{}\n\t{},{}",today.format("%d/%m/%Y"),job,now)
            }
        };
         

        let mut f = append_to(&fname)?;
        writeln!(f, "{}", f_line)?
    }
    if let Some(c_data) = curr {
        let since_last = now.since(&today,c_data.time, &c_data.date);
        println!(
            "You have been clocked in for {} for {} hours",
            c_data, since_last,
        );
        let otime = since_last + c_data.time;
        c_io.push((c_data, otime));
    }

    //filter.

    if cfg.bool_flag("this_week", Filter::Arg) {
        let dt = Local::today();
        let wk = dt.iso_week().week();
        let st = NaiveDate::from_isoywd(dt.year(), dt.iso_week().week(), Weekday::Mon);
        let fin = NaiveDate::from_isoywd(dt.year(), dt.iso_week().week(), Weekday::Sun);
        println!("Filtering by week {}", wk);
        c_io.retain(|(ind, _)| ind.date >= st && ind.date <= fin);
    }

    if let Some(grp) = cfg.grab().arg("group").done(){
        println!("Filtering by group {}",grp); 
        let group = clock_data.groups.get(&grp).ok_or(TokErr::Mess(format!("Group not defined \"{}\"",grp) ))?;
        c_io.retain(|(ind,_)| group.contains(&ind.job));
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
        let dt = clockin::read_date(&d)?;
        c_io.retain(|(ind, _)| ind.date >= dt);
    }

    if let Some(d) = cfg.grab().arg("until").done() {
        let dt = clockin::read_date(&d)?;
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

    Ok(())
}

fn do_quotes(s:&str)->String{
    for c in " \n\t".chars(){
        if s.contains(c){
            return format!("\"{}\"",s);
        }
    }
    s.to_string()
}
