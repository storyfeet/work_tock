use derive_more::*;
use std::fmt::{Display,Debug,Formatter};


#[derive(Copy, Clone, PartialEq, Eq,  Add, Sub, AddAssign, SubAssign)]
pub struct STime(i32);//minutes

impl STime{
    pub fn new(hr:i32,min:i32)->Self{
        STime(hr*60 + min) 
    }
}

impl Debug for STime{
    fn fmt(&self,f:&mut Formatter)->std::fmt::Result{
        write!(f,"{:02}:{:02}",self.0/60,self.0%60)
    }
}

impl Display for STime{
    fn fmt(&self,f:&mut Formatter)->std::fmt::Result{
        write!(f,"{:?}",self)
    }
}



