use chrono::naive::NaiveDate;
use derive_more::*;

use crate::parse::{Peeker, Tokeniser,Token};
use crate::err::TokErr;


#[derive(Copy, Clone, PartialEq, Eq, Constructor, Add, Sub, AddAssign, SubAssign)]
pub struct STime {
    hour: u32,
    min: u32,
}

pub struct ClockinBuilder {
    job: Option<String>,
    date: Option<NaiveDate>,
    tags: Vec<String>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Clockin {
    c_in: STime,
    c_out: Option<STime>,
    date: NaiveDate,
    job: String,
    tags: Vec<String>,
}

pub fn read_string(s: &str) -> Result<Vec<Clockin>,TokErr> {
    let mut pk = Peeker::from_iter(Tokeniser::new(s));

    let mut bld = ClockinBuilder {
        job: None,
        date: None,
        tags: Vec::new(),
    };
    let mut res = Vec::new();

    
    while let Some(tk) = pk.next(){
        use self::Token::*;
        match tk{
            Break=>continue,
            Str(s)=>bld.job = Some(s),
            Num(n)=>{
                match pk.next().ok_or("num without context")?{
                    Colon=>{
                        let n2 = pk.next().ok_or("EarlyEOF")?.as_num()?;
                        pk.next().ok_or("EarlyEOF")?.must_be(Dash)?;
                        let o1 = pk.next().ok_or("EarlyEOF")?.as_num()?;
                        pk.next().ok_or("EarlyEOF")?.must_be(Colon)?;
                        let o2 = pk.next().ok_or("EarlyEOF")?.as_num()?;
                        //TODO create clockitem 
                        //time
                    }
                    Slash=>{
                        //date
                    }
                    other=>Err(format!("use / for date, and : for time. not '{:?}'",other))?,
                }
            }
            other=>{
                Err(format!("Failed at {:?}",other))?;
                
            }
        }
    }


    Ok(res)
}
