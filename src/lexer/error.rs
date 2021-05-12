use std::fmt::{self, Display};

use super::SourcePos;


pub type InvalidEscapeCode<'a> = &'a [u8];


#[derive(Debug)]
pub enum InvalidLiteral<'a> {
	InvalidEscapeCodes(Box<[InvalidEscapeCode<'a>]>),
	InvalidNumber(&'a [u8]),
}


impl<'a> Display for InvalidLiteral<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			InvalidLiteral::InvalidEscapeCodes(codes) => {
				for code in codes.iter() {
					writeln!(f, "Invalid escape code: {}", String::from_utf8_lossy(code))?;
				}
			}

			InvalidLiteral::InvalidNumber(literal) => {
				writeln!(f, "Invalid number: {}", String::from_utf8_lossy(literal))?;
			}
		};

		Ok(())
	}
}


#[derive(Debug)]
pub enum ErrorKind<'a> {
	UnexpectedEof,
	Unexpected(u8),
	InvalidLiteral(InvalidLiteral<'a>),
}


impl<'a> Display for ErrorKind<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			ErrorKind::UnexpectedEof => writeln!(f, "Unexpected end of file.")?,
			ErrorKind::Unexpected(value) => writeln!(f, "Unexpected '{}'.", value)?,
			ErrorKind::InvalidLiteral(literal) => {
				writeln!(f, "Invalid literal.")?;
				writeln!(f, "{}", literal)?;
			}
		};

		Ok(())
	}
}


#[derive(Debug)]
pub struct Error<'a> {
	pub error: ErrorKind<'a>,
	pub pos: SourcePos,
}


impl<'a> Display for Error<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		writeln!(f, "Error at {}: {}", self.pos, self.error)
	}
}


impl<'a> Error<'a> {
	pub fn unexpected(input: u8, pos: SourcePos) -> Self {
		Self {
			error: ErrorKind::Unexpected(input),
			pos
		}
	}
}
