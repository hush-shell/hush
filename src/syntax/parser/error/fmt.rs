use std::fmt::Display as _;

use super::{Error, Expected, Token};
use crate::{
	fmt::{self, Display},
	symbol,
};


impl<'a> Display<'a> for Expected {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::Token(token) => {
				"'".fmt(f)?;
				token.fmt(f, context)?;
				"'".fmt(f)
			}

			Self::Message(msg) => msg.fmt(f),
		}
	}
}


impl<'a> Display<'a> for Error {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::UnexpectedEof => "unexpected end of file".fmt(f),

			Self::Unexpected { token: Token { kind, pos }, expected } => {
				write!(f, "{} - unexpected '", fmt::Show(pos, context))?;
				kind.fmt(f, context)?;
				"', expected ".fmt(f)?;
				expected.fmt(f, context)
			},

			Self::EmptyCommandBlock { pos } => {
				write!(f, "{} - empty command block", fmt::Show(pos, context))
			}
		}
	}
}


/// We need this in order to be able to implement std::error::Error.
impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		Display::fmt(self, f, &symbol::Interner::new())
	}
}

