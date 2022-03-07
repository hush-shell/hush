use std::{
	io,
	path::Path,
	os::unix::ffi::OsStrExt,
};

use crate::{fmt, symbol, syntax::AnalysisDisplayContext, tests};
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
			let path_symbol = interner.get_or_intern(path.as_os_str().as_bytes());
			let source = Source::from_reader(path_symbol, file)?;
			let analysis = Analysis::analyze(&source, &mut interner);

			if !check(&analysis) {
				panic!("{}", fmt::Show(
					analysis,
					AnalysisDisplayContext {
						max_errors: None,
						interner: &interner,
					}
				));
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
