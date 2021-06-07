use std::{io, fs, path::{Path, PathBuf}};

use crate::symbol;
use super::{Analysis, Source, DisplayErrors};


#[test]
fn test_examples() -> io::Result<()> {
	let mut examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	examples_dir.push("examples/hush");

	fn run(dir: &Path, interner: &mut symbol::Interner) -> io::Result<()> {
		for entry in fs::read_dir(dir)? {
			let path = entry?.path();

			if path.is_dir() {
				run(&path, interner)?;
			} else {
				let source = Source::from_path(path)?;
				let analysis = Analysis::analyze(source, interner);

				if !analysis.errors.is_empty() {
					panic!("{:#?}", analysis);
				}
			}
		}

		Ok(())
	}

	let mut interner = symbol::Interner::new();

	run(&examples_dir, &mut interner)
}

#[test]
fn test_ill_statement() {
	let mut interner = symbol::Interner::new();

  let source = Source {
		path: Path::new("<stdin>").into(),
		contents: [
			".",
		].join("\n").as_bytes().into(),
	};

	let analysis = Analysis::analyze(source, &mut interner);

	assert!(!analysis.errors.is_empty());
}


#[test]
fn test_funcall() {
	let mut interner = symbol::Interner::new();

  let source = Source {
		path: Path::new("<stdin>").into(),
		contents: [
			"call()",
			"dict[field](arg)",
			"fun(many)(chained)[field](calls)",
			"fun(many).field(chained).field.calls()",
		].join("\n").as_bytes().into(),
	};

	let analysis = Analysis::analyze(source, &mut interner);

	assert!(analysis.errors.is_empty(), "\n{}", DisplayErrors(&analysis.errors));
}


#[test]
fn test_ill_funcall_1() {
	let mut interner = symbol::Interner::new();

  let source = Source {
		path: Path::new("<stdin>").into(),
		contents: [
			"call()",
			"dict[field](arg",
			"fun(many)(chained)[field](calls)",
			"fun(many).field(chained).field.calls()"
		].join("\n").as_bytes().into(),
	};

	let analysis = Analysis::analyze(source, &mut interner);

	assert!(!analysis.errors.is_empty());
}


#[test]
fn test_ill_funcall_2() {
	let mut interner = symbol::Interner::new();

  let source = Source {
		path: Path::new("<stdin>").into(),
		contents: [
			"call()",
			"dict[field](arg)",
			"fun(many)(chained)[field](calls.)",
			"fun(many).field(chained).field.calls()"
		].join("\n").as_bytes().into(),
	};

	let analysis = Analysis::analyze(source, &mut interner);

	assert!(!analysis.errors.is_empty());
}


#[test]
fn test_ill_expr_1() {
	let mut interner = symbol::Interner::new();

  let source = Source {
		path: Path::new("<stdin>").into(),
		contents: [
			"@[ ",
			"	hai: 1,",
			"	b: [ 5 * ],",
			"	c: 3",
			"]",
			"function b()",
			"	std.print(hai)",
			"end",
		].join("\n").as_bytes().into(),
	};

	let analysis = Analysis::analyze(source, &mut interner);

	assert!(!analysis.errors.is_empty());
}


#[test]
fn test_ill_expr_2() {
	let mut interner = symbol::Interner::new();

  let source = Source {
		path: Path::new("<stdin>").into(),
		contents: [
			"fun(1, hello., 3,)",
			"5 + 5"
		].join("\n").as_bytes().into(),
	};

	let analysis = Analysis::analyze(source, &mut interner);

	assert!(!analysis.errors.is_empty());
}


#[test]
fn test_command_block() {
	let mut interner = symbol::Interner::new();

  let source = Source {
		path: Path::new("<stdin>").into(),
		contents: [
			"{",
			"	hey you;",
			"	out there on the wall 2>1 1>2 >> file > 'some file' ?;",
			"}",
		].join("\n").as_bytes().into(),
	};

	let analysis = Analysis::analyze(source, &mut interner);

	assert!(analysis.errors.is_empty(), "\n{}", DisplayErrors(&analysis.errors));
}
