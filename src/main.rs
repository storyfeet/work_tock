use std::io::Read;

mod clockin;
mod parse;
mod s_time;

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

    let fname = fname.ok_or("No Filename provided use -f")?;

    let mut f = std::fs::File::open(fname).map_err(|_| "Could not read file")?;

    let mut s: String = String::new();

    f.read_to_string(&mut s)
        .map_err(|_| "Could not read file")?;

    let (clocks, errs) = clockin::read_string(&s);

    println!("CLOCK INS");
    for c in clocks {
        println!("{:?}", c);
    }
    println!("\n\nERRS  \n{:?}", errs);

    Ok(())
}
