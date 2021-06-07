pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
mod source;
#[cfg(test)]
mod tests;

use std::{cell::RefCell, fmt::{self, Debug}};

use crate::symbol;
pub use ast::Ast;
pub use error::{DisplayErrors, Error};
use lexer::Lexer;
use parser::Parser;
pub use source::{Source, SourcePos};


pub struct Analysis {
	/// The produced AST, possibly partial if there were errors.
	pub ast: Ast,
	/// Syntax errors.
	pub errors: Box<[Error]>,
}


impl Analysis {
	/// Perform syntax analysis in the given source.
	pub fn analyze(source: Source, interner: &mut symbol::Interner) -> Self {
		let cursor = lexer::Cursor::from(source.contents.as_ref());
		let lexer = Lexer::new(cursor, interner);

		// Errors will be produced by the lexer and the parser alternatively.
		// There won't be borrow issues here because the lexer will always run a complete
		// iteration (producing a token or an error) before yielding to the parser.
		let errors = RefCell::new(Vec::new());

		let tokens = lexer.filter_map(|result| match result {
			Ok(token) => Some(token),
			Err(error) => {
				errors.borrow_mut().push(Error::Lexer(error));
				None
			}
		});

		let parser = Parser::new(tokens, |error| {
			errors.borrow_mut().push(Error::Parser(error))
		});

		let statements = parser.parse();

		Analysis {
			ast: Ast { path: source.path, statements },
			errors: errors.into_inner().into(),
		}
	}
}


impl Debug for Analysis {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		writeln!(f, "Analysis for {}", self.ast.path.display())?;

		for error in self.errors.iter() {
			writeln!(f, "Error: {}", error)?;
		}

		if f.alternate() {
			writeln!(f, "AST:\n{:#?}", self.ast.statements)
		} else {
			writeln!(f, "AST:\n{:?}", self.ast.statements)
		}
	}
}
