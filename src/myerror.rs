use std::fmt; 
use std::error::Error;
pub static READING_STDIN:&str = "Reading from stdin";
#[derive(Debug)]
pub(crate) struct NotImplementedError{
   pub feature:&'static str, 
}

impl fmt::Display for NotImplementedError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let feature = self.feature; 
        write!(f, "feature {feature} not yet implemented")
    }
}

impl Error for NotImplementedError{}

