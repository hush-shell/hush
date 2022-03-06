use std::{
	borrow::Cow,
	ffi::OsString,
	os::unix::ffi::OsStringExt,
};

use super::exec;


pub type Arg = Vec<u8>;


pub enum Args {
	Patterns(Vec<Arg>),
	Literals(Vec<Arg>),
}


impl Args {
	pub fn push_literal(&mut self, literal: &[u8]) {
		match self {
			Self::Patterns(patterns) => {
				if patterns.is_empty() {
					patterns.push(Arg::default());
				}

				let escaped = Self::pattern_escape(literal);
				let escaped = escaped.as_ref();

				for pattern in patterns.iter_mut() {
					pattern.extend(escaped);
				}
			}

			Self::Literals(literals) => {
				if literals.is_empty() {
					literals.push(Arg::default());
				}

				for lit in literals.iter_mut() {
					lit.extend(literal);
				}
			}
		}
	}


	pub fn push_pattern(&mut self, pattern: &[u8]) {
		match self {
			Self::Patterns(patterns) => {
				if patterns.is_empty() {
					patterns.push(Arg::default());
				}

				for rx in patterns.iter_mut() {
					rx.extend(pattern);
				}
			}

			Self::Literals(literals) => {
				if literals.is_empty() {
					literals.push(Arg::default());
				}

				let mut patterns = std::mem::take(literals);

				for literal in patterns.iter_mut() {
					if let Cow::Owned(mut lit) = Self::pattern_escape(literal) {
						std::mem::swap(&mut lit, literal);
					};

					literal.extend(pattern);
				}

				*self = Self::Patterns(patterns);
			}
		}
	}


	/// Push many literals in a cartesian product style.
	pub fn push_literals<I, B>(&mut self, mut iter: I)
	where
		I: Iterator<Item = B>,
		B: AsRef<[u8]>,
	{
		if let Some(first) = iter.next() {
			let first = first.as_ref();

			let (args, escape) = match self {
				Args::Patterns(patterns) => (patterns, true),
				Args::Literals(literals) => (literals, false),
			};

			if args.is_empty() {
				args.push(Arg::default());
			}

			let original_len = args.len();

			for lit in iter {
				let lit = lit.as_ref();

				let previous_len = args.len();
				args.extend_from_within(..original_len);

				let lit =
					if escape {
						Self::pattern_escape(lit)
					} else {
						lit.into()
					};

				for arg in args[previous_len..].iter_mut() {
					arg.extend(lit.as_ref());
				}
			}

			let first =
				if escape {
					Self::pattern_escape(first)
				} else {
					first.into()
				};

			for arg in args[..original_len].iter_mut() {
				arg.extend(first.as_ref());
			}
		}
	}


	fn pattern_escape(literal: &[u8]) -> Cow<[u8]> {
		let has_meta = literal
			.iter()
			.copied()
			.any(Self::is_pattern_meta);

		if has_meta {
			let mut escaped = Vec::with_capacity(literal.len());

			for character in literal.iter().copied() {
				if Self::is_pattern_meta(character) {
					escaped.push(b'[');
					escaped.push(character);
					escaped.push(b']');
				} else {
					escaped.push(character)
				}
			}

			Cow::Owned(escaped)
		} else {
			Cow::Borrowed(literal)
		}
	}


	fn is_pattern_meta(c: u8) -> bool {
		matches!(c, b'?' | b'*' | b'[' | b']')
	}
}


impl Default for Args {
	fn default() -> Self {
		Self::Literals(Vec::new())
	}
}


impl From<Args> for Box<[exec::Argument]> {
	fn from(args: Args) -> Box<[exec::Argument]> {
		match args {
			Args::Patterns(patterns) => {
				patterns
					.into_iter()
					.map(
						|pattern| exec::Argument::Pattern(
							OsString::from_vec(pattern).into_boxed_os_str()
						)
					)
					.collect()
			}

			Args::Literals(literals) => {
				literals
					.into_iter()
					.map(
						|lit| exec::Argument::Literal(
							OsString::from_vec(lit).into_boxed_os_str()
						)
					)
					.collect()
			}
		}
	}
}
