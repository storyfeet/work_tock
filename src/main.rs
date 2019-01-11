use std::collections::BTreeMap;

mod clockin;
use crate::clockin::Clockin;
mod parse;
mod s_time;
use crate::s_time::STime;

mod err;

fn main() -> Result<(), String> {
    let mut cfg = lazy_conf::config("-c", &["{HOME}/.config/work_tock/init"])
        .map_err(|_| "Wierd Arguments")?;

    let fname = cfg
        .grab()
        .fg("-f")
        .cf("config.file")
        .help("Filename: what file to look at")
        .s();

    if cfg.help("Work Tock") {
        return Ok(());
    }

    let fname = lazy_conf::env::replace_env(&fname.ok_or("No Filename provided use -f")?).map_err(|_|"could not env replace")?;
    

    let s = std::fs::read_to_string(&fname).map_err(|_| format!("Could not read file: {}",fname))?;



    let (clocks, errs) = clockin::read_string(&s);

    println!("\n\nERRS  \n{:?}", errs);

    let mut curr= None;
    let mut c_io = Vec::new();
    //Get outs with ins so filter makes sense
    for c in clocks {
        match c {
            Clockin::In(data)=>{
                if let Some(cin) = curr{
                    c_io.push((cin,data.time));
                }
                curr = Some(data);
            }
            Clockin::Out(cout)=>{
                match curr {
                    Some(data)=>c_io.push((data,cout)),
                    None=>println!("Two Out's in a row"),
                }
                curr = None;
            }
        }
    }
    if let Some(data) = curr{
        c_io.push((data,STime::now()));
    }

    //TODO add filters.

    //build report
    let mut r_times:BTreeMap<String,STime> = BTreeMap::new();
    let mut t_time = STime::new(0,0);
    for (idat,otime) in c_io{
        let tt = r_times.get(&idat.job).map(|x|*x).unwrap_or(STime::new(0,0));
        r_times.insert(idat.job,tt + otime - idat.time);
        t_time += otime - idat.time;
    }


    println!("{:?}",r_times);
    println!("Total Time = {}",t_time);


    Ok(())
}
