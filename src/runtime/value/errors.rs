use std::fmt::{self, Display};


/// Array or Dict index out of bounds.
#[derive(Debug)]
pub struct IndexOutOfBounds;


impl Display for IndexOutOfBounds {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "index out of bounds")
  }
}


impl std::error::Error for IndexOutOfBounds { }
