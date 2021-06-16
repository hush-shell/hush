mod fmt;

use super::{Symbol, SourcePos};


/// A semantic error.
#[derive(Debug, Default)]
pub struct Errors(pub Vec<Error>);


impl IntoIterator for Errors {
	type Item = Error;
	type IntoIter = std::vec::IntoIter<Error>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}


impl Extend<Error> for Errors {
	fn extend<T>(&mut self, iter: T)
	where
		T : IntoIterator<Item = Error>,
	{
		self.0.extend(iter)
	}
}


// impl From<Vec<Error>> for Errors {
// 	fn from(errors: Vec<Error>) -> Self {
// 		Errors(errors)
// 	}
// }


/// The kind of semantic error.
#[derive(Debug)]
pub enum ErrorKind {
	/// Variable usage before variable declaration.
	UndeclaredVariable(Symbol),
	/// Variable declared with the same name twice in the same scope.
	/// Includes function parameters.
	DuplicateVariable(Symbol),
	/// Duplicate keys in dict literal.
	DuplicateKey(Symbol),
	/// Return statement outside function.
	ReturnOutsideFunction,
	/// Self keyword outside function.
	SelfOutsideFunction,
	/// Break statement outside loop.
	BreakOutsideLoop,
	/// Invalid assignment l-value.
	InvalidAssignment,
	/// Built-in command used in async context.
	/// Async contexts include pipes, redirections and capture or async blocks.
	AsyncBuiltin,
}


/// A semantic error.
#[derive(Debug)]
pub struct Error {
	pub kind: ErrorKind,
	pub pos: SourcePos,
}


impl Error {
	/// Variable usage before variable declaration.
	pub fn undeclared_variable(symbol: Symbol, pos: SourcePos) -> Self {
		Self {
			kind: ErrorKind::UndeclaredVariable(symbol),
			pos
		}
	}


	/// Variable declared with the same name twice in the same scope.
	/// Includes function parameters.
	pub fn duplicate_variable(symbol: Symbol, pos: SourcePos) -> Self {
		Self {
			kind: ErrorKind::DuplicateVariable(symbol),
			pos
		}
	}


	/// Duplicate keys in dict literal.
	pub fn duplicate_key(symbol: Symbol, pos: SourcePos) -> Self {
		Self {
			kind: ErrorKind::DuplicateKey(symbol),
			pos
		}
	}


	/// Return statement outside function.
	pub fn return_outside_function(pos: SourcePos) -> Self {
		Self {
			kind: ErrorKind::ReturnOutsideFunction,
			pos
		}
	}


	/// Return statement outside function.
	pub fn self_outside_function(pos: SourcePos) -> Self {
		Self {
			kind: ErrorKind::SelfOutsideFunction,
			pos
		}
	}


	/// Break statement outside loop.
	pub fn break_outside_loop(pos: SourcePos) -> Self {
		Self {
			kind: ErrorKind::BreakOutsideLoop,
			pos
		}
	}


	/// Invalid assignment l-value.
	pub fn invalid_assignment(pos: SourcePos) -> Self {
		Self {
			kind: ErrorKind::InvalidAssignment,
			pos
		}
	}


	/// Built-in command used in async context.
	/// Async contexts include pipes redirections and capture or async blocks.
	pub fn async_builtin(pos: SourcePos) -> Self {
		Self {
			kind: ErrorKind::AsyncBuiltin,
			pos
		}
	}
}
