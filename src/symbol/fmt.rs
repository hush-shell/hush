use std::{borrow::Cow, fmt::Display as _};

use super::{Interner, Symbol};
use crate::{
	syntax::ast::fmt::ILL_FORMED,
	fmt::Display,
	term::color,
};


impl<'a> Display<'a> for Symbol {
	type Context = &'a Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: Self::Context) -> std::fmt::Result {
		if *self == Self::default() {
			ILL_FORMED.fmt(f)
		} else {
			let ident: Cow<str> = match context.resolve(*self) {
				Some(id) => id.into(),
				None => format!("<unresolved id #{}>", Into::<usize>::into(*self)).into(),
			};

			color::Fg(color::Green, ident).fmt(f)
		}
	}
}
