use std::fmt::{self, Debug, Display};

use termion::color as term;
pub use termion::color::{Black, Red, Green, Blue};


/// Paint the foreground with a given color when formatting the value.
pub struct Fg<C, T>(pub C, pub T);


impl<C, T> Debug for Fg<C, T>
where
	C: term::Color + Copy,
	T: Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", term::Fg(self.0))?;
		self.1.fmt(f)?;
		write!(f, "{}", term::Fg(term::Reset))
	}
}


impl<C, T> Display for Fg<C, T>
where
	C: term::Color + Copy,
	T: Display,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}{}{}", term::Fg(self.0), self.1, term::Fg(term::Reset))
	}
}
