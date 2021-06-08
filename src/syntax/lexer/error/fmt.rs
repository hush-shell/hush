use std::fmt::{self, Display};

use super::{Error, ErrorKind};


impl Display for ErrorKind {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::UnexpectedEof => "unexpected end of file".fmt(f)?,

			Self::Unexpected(value) => write!(f, "unexpected '{}'", *value as char)?,

			Self::EmptyByteLiteral => "empty char literal".fmt(f)?,

			Self::InvalidEscapeSequence(sequence) => {
				write!(
					f,
					"invalid escape sequence: {}",
					String::from_utf8_lossy(sequence)
				)?;
			}

			Self::InvalidNumber(number) => {
				write!(f, "invalid number: {}", String::from_utf8_lossy(number))?;
			}

			Self::InvalidIdentifier(ident) => {
				write!(f, "invalid identifier: {}", String::from_utf8_lossy(ident))?;
			}
		};

		Ok(())
	}
}


impl Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{} - {}.", self.pos, self.error)
	}
}
