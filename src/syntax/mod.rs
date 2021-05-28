pub mod ast;
pub mod lexer;
pub mod parser;
pub mod error;
mod source;

use std::cell::RefCell;

pub use ast::Ast;
pub use source::{Source, SourcePos};
pub use error::Error;
use lexer::Lexer;
use parser::Parser;
use crate::symbol;


#[derive(Debug)]
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

		let tokens = lexer.filter_map(
			|result| match result {
				Ok(token) => Some(token),
				Err(error) => {
					errors.borrow_mut().push(Error::Lexer(error));
					None
				},
			}
		);

		let parser = Parser::new(
			tokens,
			|error| errors.borrow_mut().push(Error::Parser(error))
		);

		let statements = parser.parse();

		Analysis {
			ast: Ast {
				path: source.path,
				statements,
			},
			errors: errors.into_inner().into(),
		}
	}
}
