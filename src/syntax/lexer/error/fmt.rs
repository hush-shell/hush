use crate::{
	fmt::{self, Display},
	symbol,
};
use super::{Error, ErrorKind};


impl std::fmt::Display for ErrorKind {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::UnexpectedEof => "unexpected end of file".fmt(f)?,

			Self::Unexpected(value) => write!(f, "unexpected '{}'", (*value as char).escape_debug())?,

			Self::EmptyByteLiteral => "empty char literal".fmt(f)?,

			Self::InvalidEscapeSequence(sequence) => {
				write!(
					f,
					"invalid escape sequence '{}'",
					String::from_utf8_lossy(sequence)
				)?;
			}

			Self::InvalidNumber(number) => {
				write!(f, "invalid number '{}'", String::from_utf8_lossy(number))?;
			}

			Self::InvalidIdentifier(ident) => {
				write!(f, "invalid identifier '{}'", String::from_utf8_lossy(ident))?;
			}
		};

		Ok(())
	}
}


impl<'a> Display<'a> for Error {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		write!(f, "{} - {}.", fmt::Show(self.pos, context), self.error)
	}
}


impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "line {}, column {} - {}.", self.pos.line, self.pos.column, self.error)
	}
}
