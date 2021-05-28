// #![feature(backtrace)]
#![allow(dead_code)] // This is temporarily used for the inital development.

mod io;
mod symbol;
mod syntax;

use std::path::Path;


fn main() -> std::io::Result<()> {
	let source = syntax::Source::from_reader(Path::new("<stdin>"), std::io::stdin().lock())?;

	syntax(source);

	Ok(())
}


fn syntax(source: syntax::Source) {
	use syntax::Analysis;

	let mut interner = symbol::Interner::new();
	let analysis = Analysis::analyze(source, &mut interner);

	for error in analysis.errors.iter() {
		eprintln!("{}", error);
	}

	println!("{:#?}", analysis.ast);
}


fn lexer(source: syntax::Source) {
	use syntax::lexer::{Cursor, Lexer};

	let mut interner = symbol::Interner::new();
	let cursor = Cursor::from(source.contents.as_ref());
	let lexer = Lexer::new(cursor, &mut interner);

	let mut current_line = 0;
	for result in lexer {
		match result {
			Ok(token) => {
				if token.pos.line != current_line {
					current_line = token.pos.line;
					println!();
				}

				print!("{:?} ", token);
			}
			Err(error) => {
				if error.pos.line != current_line {
					current_line = error.pos.line;
				}

				eprintln!("\n{}: {}", source.path.display(), error)
			}
		}
	}
}
