use std::{
	borrow::Cow,
	ffi::OsString,
	os::unix::ffi::OsStringExt,
};

use super::Argument;

use regex::bytes::Regex;


pub type Arg = Vec<u8>;


pub enum Args {
	Regexes(Vec<Arg>),
	Literals(Vec<Arg>),
}


impl Args {
	pub fn push_literal(&mut self, literal: &[u8]) {
		match self {
			Self::Regexes(regexes) => {
				let escaped = Self::regex_escape(literal);
				let escaped = escaped.as_ref();

				for regex in regexes.iter_mut() {
					regex.extend(escaped);
				}
			}

			Self::Literals(literals) => {
				for lit in literals.iter_mut() {
					lit.extend(literal);
				}
			}
		}
	}


	pub fn push_regex(&mut self, regex: &[u8]) {
		match self {
			Self::Regexes(regexes) => {
				for rx in regexes.iter_mut() {
					rx.extend(regex);
				}
			}

			Self::Literals(literals) => {
				let mut regexes = std::mem::take(literals);

				for literal in regexes.iter_mut() {
					if let Cow::Owned(mut lit) = Self::regex_escape(literal) {
						std::mem::swap(&mut lit, literal);
					};

					literal.extend(regex);
				}

				*self = Self::Regexes(regexes);
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
				Args::Regexes(regexes) => (regexes, true),
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
						Self::regex_escape(lit)
					} else {
						lit.into()
					};

				for arg in args[previous_len..].iter_mut() {
					arg.extend(lit.as_ref());
				}
			}

			let first =
				if escape {
					Self::regex_escape(first)
				} else {
					first.into()
				};

			for arg in args[..original_len].iter_mut() {
				arg.extend(first.as_ref());
			}
		}
	}


	fn regex_escape(literal: &[u8]) -> Cow<[u8]> {
		let has_meta = literal
			.iter()
			.copied()
			.any(Self::is_regex_meta);

		if has_meta {
			let mut escaped = Vec::with_capacity(literal.len());

			for character in literal.iter().copied() {
				if Self::is_regex_meta(character) {
					escaped.push(b'\\');
				}

				escaped.push(character)
			}

			Cow::Owned(escaped)
		} else {
			Cow::Borrowed(literal)
		}
	}


	fn is_regex_meta(c: u8) -> bool {
		match c {
			b'\\' | b'.' | b'+' | b'*' | b'?' | b'(' | b')' | b'|' | b'[' | b']' | b'{'
				| b'}' | b'^' | b'$' | b'#' | b'&' | b'-' | b'~' => true,
			_ => false,
		}
	}
}


impl Default for Args {
	fn default() -> Self {
		Self::Literals(Vec::new())
	}
}


impl Into<Box<[Argument]>> for Args {
	fn into(self) -> Box<[Argument]> {
		match self {
			Self::Regexes(regexes) => {
				regexes
					.into_iter()
					.map(
						|regex| Argument::Regex(
							Regex
								::new(
									&String::from_utf8(regex).expect("invalid utf8 regex argument")
								)
								.expect("invalid regex in argument")
						)
					)
					.collect()
			}

			Self::Literals(literals) => {
				literals
					.into_iter()
					.map(
						|lit| Argument::Literal(
							OsString::from_vec(lit).into_boxed_os_str()
						)
					)
					.collect()
			}
		}
	}
}
