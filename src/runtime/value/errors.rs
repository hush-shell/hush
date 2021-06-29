use std::fmt::{self, Display};


/// Collection index out of bounds.
#[derive(Debug)]
pub struct IndexOutOfBounds;


impl Display for IndexOutOfBounds {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "index out of bounds")
  }
}


impl std::error::Error for IndexOutOfBounds { }


/// Collection is empty.
#[derive(Debug)]
pub struct EmptyCollection;


impl Display for EmptyCollection {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "collection is empty")
  }
}


impl std::error::Error for EmptyCollection { }
