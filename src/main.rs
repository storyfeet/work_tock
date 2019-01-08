
use chrono::naive::NaiveDate;

mod s_time;
use crate::s_time::STime;
mod parse;

mod err;


fn main()-> Result<(),String>{
    let mut cfg = lazy_conf::config("-c",&["{HOME}/.config/work_tock/init"]).map_err(|_|"Wierd Arguments")?;

    let fname = cfg.grab().fg("-f").cf("config.file").help("Filename: what file to look at").s();
    
    if cfg.help("Work Tock") {return Ok(())}

    let fname = fname.ok_or("No Filename provided use -f")?;

    
    
    

    Ok(())

}
