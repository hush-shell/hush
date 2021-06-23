use std::fmt::{self, Display};


#[derive(Debug)]
pub struct StackOverflow;


impl Display for StackOverflow {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "stack overflow")
  }
}


impl std::error::Error for StackOverflow { }
