#![allow(dead_code)] // This is temporarily used for the inital development.

mod fmt;
mod io;
mod semantic;
mod symbol;
mod syntax;
mod term;

use std::path::Path;

use term::color;


fn main() -> std::io::Result<()> {
	let mut interner = symbol::Interner::new();
	let source = syntax::Source::from_reader(Path::new("<stdin>"), std::io::stdin().lock())?;

	semantic(source, &mut interner);

	Ok(())
}


fn lexer(source: syntax::Source, interner: &mut symbol::Interner) {
	use syntax::lexer::{Cursor, Lexer};

	let cursor = Cursor::from(source.contents.as_ref());
	let lexer = Lexer::new(cursor, interner);
	let tokens: Vec<_> = lexer.collect();

	for result in tokens {
		match result {
			Ok(token) => println!("{}", fmt::Show(token, &*interner)),
			Err(error) => {
				eprintln!("\n{}: {}", source.path.display(), error)
			}
		}
	}
}


fn syntax(source: syntax::Source, interner: &mut symbol::Interner) -> syntax::Analysis {
	let analysis = syntax::Analysis::analyze(source, interner);

	println!("{}", color::Fg(color::Yellow, "syntactic analysis"));

	for error in analysis.errors.iter().take(20) {
		eprintln!(
			"{}: {}",
			color::Fg(color::Red, "Error"),
			fmt::Show(error, &*interner)
		);
	}

	println!(
		"{}",
		fmt::Show(
			&analysis.ast,
			syntax::ast::fmt::Context::from(&*interner)
		)
	);

	analysis
}


fn semantic(source: syntax::Source, interner: &mut symbol::Interner) {
	let syntactic_analysis = syntax(source, interner);

	println!("{}", color::Fg(color::Yellow, "semantic analysis"));

	match semantic::analyze(syntactic_analysis.ast) {
		Err(errors) => for error in errors.into_iter().take(20) {
			eprintln!(
				"{}: {}",
				color::Fg(color::Red, "Error"),
				fmt::Show(error, &*interner)
			);
		},

		Ok(program) => println!(
			"{}",
			fmt::Show(
				&program,
				semantic::program::fmt::Context::from(&*interner)
			)
		),
	}
}
