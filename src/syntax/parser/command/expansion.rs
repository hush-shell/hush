use super::ast;


/// A literal parsed for expansions.
/// As parsing expansions never fails, we have either a successfully parsed expansion, or
/// the input literal.
pub enum Expanded<'a> {
	Literal(&'a [u8]),
	Expansion(ast::ArgExpansion),
}


/// Parser for argument expansions.
/// As parsing expansions is infallible, this parser is implemented as an iterator.
pub struct Parser<'a> {
	input: &'a [u8],
	/// Whether to allow expanding home (~/)
	allow_home: bool,
}


impl<'a> Parser<'a> {
	/// Create a new expansions parser for the given input.
	/// This won't expand home (~/) unless explicitly set with `allow_home`.
	pub fn new(input: &'a [u8]) -> Self {
		Self {
			input,
			allow_home: false,
		}
	}


	/// Whether to allow expanding home (~/)
	pub fn allow_home(mut self, allow_home: bool) -> Self {
		self.allow_home = allow_home;
		self
	}
}


impl<'a> Iterator for Parser<'a> {
	type Item = Expanded<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		use crate::slice::SplitOnce;

		if self.input.is_empty() {
			return None;
		}

		// Home:
		if self.allow_home {
			self.allow_home = false; // Only allow home in the beggining.
			if let Some(rest) = self.input.strip_prefix(b"~/") {
				self.input = rest;
				return Some(Expanded::Expansion(ast::ArgExpansion::Home));
			}
		}

		// Star:
		if let Some(rest) = self.input.strip_prefix(b"*") {
			self.input = rest;
			return Some(Expanded::Expansion(ast::ArgExpansion::Star));
		}

		// Question:
		if let Some(rest) = self.input.strip_prefix(b"?") {
			self.input = rest;
			return Some(Expanded::Expansion(ast::ArgExpansion::Question));
		}

		// Char class:
		if let Some(rest) = self.input.strip_prefix(b"[") {
			if let Some((class, rest)) = rest.split_once(|c| *c == b']') {
				self.input = rest;
				return Some(Expanded::Expansion(ast::ArgExpansion::CharClass(class.into())));
			}
		}

		// TODO: range, collection

		// Take until expansion or end:
		let (part, rest) = self.input
			.split_once(|c| b"*?{[".contains(c))
			.unwrap_or((self.input, b""));

		self.input = rest;

		Some(Expanded::Literal(part))
	}
}
