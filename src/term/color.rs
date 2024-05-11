use std::{
	io,
	fmt::{self, Debug, Display},
};

pub use termion::color::{Blue, Green, Red, Yellow};


thread_local! {
	static IS_TTY: bool = termion::is_tty(&io::stdout())
		&& termion::is_tty(&io::stderr());
}


macro_rules! tty_fmt {
	($f: expr, $open: expr, $value: expr, $close: expr) => {
		IS_TTY.with(
			|&is_tty| if is_tty {
				write!($f, "{}", $open)?;
				$value.fmt($f)?;
				write!($f, "{}", $close)
			} else {
				$value.fmt($f)
			}
		)
	}
}


/// Paint the foreground with a given color when formatting the value.
pub struct Fg<C, T>(pub C, pub T);


impl<C, T> Debug for Fg<C, T>
where
	C: termion::color::Color + Copy,
	T: Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		tty_fmt!(
			f,
			termion::color::Fg(self.0),
			self.1,
			termion::color::Fg(termion::color::Reset)
		)
	}
}


impl<C, T> Display for Fg<C, T>
where
	C: termion::color::Color + Copy,
	T: Display,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		tty_fmt!(
			f,
			termion::color::Fg(self.0),
			self.1,
			termion::color::Fg(termion::color::Reset)
		)
	}
}


/// Paint with a given style.
pub struct Style<S, T>(pub S, pub T);


impl<S, T> Debug for Style<S, T>
where
	S: Display,
	T: Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		tty_fmt!(
			f,
			self.0,
			self.1,
			termion::style::Reset
		)
	}
}


impl<S, T> Display for Style<S, T>
where
	S: Display,
	T: Display,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		tty_fmt!(
			f,
			self.0,
			self.1,
			termion::style::Reset
		)
	}
}


/// Paint with a bold style.
pub struct Bold<T>(pub T);


impl<T> Debug for Bold<T>
where
	T: Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		Style(termion::style::Bold, &self.0).fmt(f)
	}
}


impl<T> Display for Bold<T>
where
	T: Display,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		Style(termion::style::Bold, &self.0).fmt(f)
	}
}
