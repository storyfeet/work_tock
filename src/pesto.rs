use pest_derive::*;
use pest::Parser;
use pest::iterators::Pair;

use crate::err::TokErr;

#[derive(Parser)]
#[grammar = "clock.pest"]
pub struct TimeFile;


pub trait Pestable where Self:Sized{
    fn from_pesto(r:Pair<Rule>)->Result<Self,TokErr>;
}

impl Pestable for i32{
    fn from_pesto(r:Pair<Rule>)->Result<Self,TokErr>{
        match r.as_rule(){
            Rule::Plusnum=>Ok(r.as_str().parse::<i32>()?),
            other=>Err(TokErr::UnexpectedRule(other)),
        }
    }
}



#[cfg(test)]
mod test{
    use super::*;
    #[test]
    pub fn test_clock(){
        let t = TimeFile::parse(Rule::Time,"25:34").unwrap().next().expect("Not sure what this one does");
        assert_eq!(t.as_rule(),Rule::Time);
        assert_eq!(t.as_str(),"25:34");
        assert_eq!(t,"hello");

        println!("T is {}",t);

        let mut rc = t.into_inner();   
        println!("RC is {:?}",rc);

        assert_eq!(rc.next().expect("Or this one").as_str(),"25");
        
    }
}


