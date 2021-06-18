use std::{io, path::Path};

use crate::{fmt, symbol, tests};
use super::{Analysis, Source};


fn test_dir<P, F>(path: P, mut check: F) -> io::Result<()>
where
	P: AsRef<Path>,
	F: FnMut(&Analysis) -> bool,
{
	let mut interner = symbol::Interner::new();

	tests::util::test_dir(
		path,
		move |path, file| {
			let source = Source::from_reader(path, file)?;
			let analysis = Analysis::analyze(source, &mut interner);

			if !check(&analysis) {
				panic!("{}", fmt::Show(analysis, &interner));
			}

			Ok(())
		}
	)
}


#[test]
fn test_examples() -> io::Result<()> {
	test_dir(
		"examples/hush",
		|analysis| analysis.errors.is_empty(),
	)
}


#[test]
fn test_positive() -> io::Result<()> {
	test_dir(
		"src/syntax/tests/data/positive",
		|analysis| analysis.errors.is_empty(),
	)
}


#[test]
fn test_negative() -> io::Result<()> {
	test_dir(
		"src/syntax/tests/data/negative",
		|analysis| !analysis.errors.is_empty(),
	)
}
