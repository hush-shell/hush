use std::fmt::Display as _;

use super::Error;
use crate::{
	fmt::Display,
	symbol,
};


impl<'a> Display<'a> for Error {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::Lexer(error) => error.fmt(f),
			Self::Parser(error) => Display::fmt(error, f, context),
		}
	}
}


/// We need this in order to be able to implement std::error::Error.
impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		Display::fmt(self, f, &symbol::Interner::new())
	}
}
