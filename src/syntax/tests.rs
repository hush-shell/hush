use std::path::Path;

use crate::symbol;
use super::{Analysis, Source, DisplayErrors};


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
