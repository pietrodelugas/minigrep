use std::{fmt::Display, collections::VecDeque};

#[derive(Clone,Debug)]
pub struct OneLine(pub String, bool, bool, i32);
impl OneLine {
    pub fn new(line: String, prints: bool, matches: bool, linenum: i32) -> Self{
        Self{0: line, 1: prints, 2: matches, 3: linenum} 
    }
    pub fn prints(&self) -> bool{
        self.1
    } 
    pub fn toggle_prints(&mut self) {
        if !self.prints(){
            //Self::new(self.0, true, self.2)
            self.1 = true 
        } 
    }
    
    pub fn lnum(&self) -> i32 {
        self.3
    }
    
}

impl Default for OneLine {
    fn default() -> Self {
        Self::new("".to_string(),false, false,0)
    }
}

impl Display for OneLine{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = &self.0[..];
        write!(f, "{content}")
    }
}



pub fn popfront(mvec: Vec<OneLine>) -> (Option<OneLine>,Vec<OneLine>) {
    let mut dqvec = VecDeque::from(mvec); 
    let value = dqvec.pop_front(); 
    (value, Vec::from(dqvec))
    
}
