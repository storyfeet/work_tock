

fn main()-> Result<(),String>{
    let cfg = lazy_conf::config("-c",&["{HOME}/.config/work_tock/init"]).map_err(|_|"Wierd Arguments")?;

    
    if cfg.help("Work Tock") {return Ok(())}

    Ok(())

}
