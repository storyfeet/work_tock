use pest_derive::*;
use pest::Parser;

#[derive(Parser)]
#[grammar = "clock.pest"]
pub struct TimeFile;


#[cfg(test)]
mod test{
    use super::*;
    #[test]
    pub fn test_clock(){
        let t = TimeFile::parse(Rule::Time,"25:34").unwrap().next().expect("Not sure what this one does");
        assert_eq!(t.as_rule(),Rule::Time);
        assert_eq!(t.as_str(),"25:34");

        println!("T is {}",t);

        let mut rc = t.into_inner();   
        println!("RC is {:?}",rc);

        assert_eq!(rc.next().expect("Or this one").as_str(),"25");
        
    }
}


