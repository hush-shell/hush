use std::collections::HashSet;

use super::{Error, Symbol, SourcePos};


/// A scope stack.
#[derive(Debug, Default)]
pub struct Stack(Vec<HashSet<Symbol>>);


impl Stack {
	/// Enter a new empty scope.
	pub fn enter(&mut self) {
		self.0.push(HashSet::new());
	}


	/// Exit the current scope.
	/// Panics if the stack is empty.
	pub fn exit(&mut self) -> HashSet<Symbol> {
		self.0
			.pop()
			.expect("attempt to exit empty stack")
	}


	/// Declares a symbol in the current scope.
	/// Returns false if the symbol was already declared.
	/// Panics if the stack is empty.
	pub fn declare(&mut self, symbol: Symbol, pos: SourcePos) -> Result<Symbol, Error> {
		let scope = self.0.last_mut().expect("empty scope stack");

		if scope.insert(symbol) {
			Ok(symbol)
		} else {
			Err(Error::duplicate_variable(symbol, pos))
		}
	}


	/// Check if a symbol is declared in the stack.
	pub fn resolve(&self, symbol: Symbol, pos: SourcePos) -> Result<Symbol, Error> {
		let found = self.0
			.iter()
			.rev()
			.any(
				|scope| scope.contains(&symbol)
			);

		if found {
			Ok(symbol)
		} else {
			Err(Error::undeclared_variable(symbol, pos))
		}
	}
}
