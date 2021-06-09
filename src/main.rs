#![allow(dead_code)] // This is temporarily used for the inital development.

mod io;
mod symbol;
mod syntax;
mod term;
mod fmt;

use std::path::Path;

use term::color;


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
		println!(
			"{}: {}",
			color::Fg(color::Red, "Error"),
			fmt::Show(error, &interner)
		);
	}

	println!(
		"{}",
		fmt::Show(
			analysis.ast,
			syntax::ast::fmt::Context::from(&interner)
		)
	);
}


fn lexer(source: syntax::Source) {
	use syntax::lexer::{Cursor, Lexer};

	let mut interner = symbol::Interner::new();
	let cursor = Cursor::from(source.contents.as_ref());
	let lexer = Lexer::new(cursor, &mut interner);
	let tokens: Vec<_> = lexer.collect();

	for result in tokens {
		match result {
			Ok(token) => println!("{}", fmt::Show(token, &interner)),
			Err(error) => {
				eprintln!("\n{}: {}", source.path.display(), error)
			}
		}
	}
}
