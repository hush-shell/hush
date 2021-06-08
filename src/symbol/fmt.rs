use std::{borrow::Cow, fmt::Display as _};

use super::{Interner, Symbol, SymbolExt};
use crate::{
	fmt::Display,
	term::color,
};


impl<'a> Display<'a> for Symbol {
	type Context = &'a Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: Self::Context) -> std::fmt::Result {
		let ident: Cow<str> = match context.resolve(*self) {
			Some(id) => id.into(),
			None => format!("<unresolved id #{}>", self.to_usize()).into(),
		};

		color::Fg(color::Green, ident).fmt(f)
	}
}
