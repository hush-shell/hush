use std::fmt::Display as _;

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
			let ident = context.resolve(*self).expect("invalid symbol");

			color::Fg(color::Green, ident).fmt(f)
		}
	}
}
