mod assert;
mod bind;
mod cd;
mod contains;
mod cwd;
mod env;
mod error;
mod has_error;
mod import;
mod is_empty;
mod iter;
mod length;
mod pop;
mod print;
mod push;
mod range;
mod replace;
mod sort;
mod split;
mod to_string;
mod trim;
mod type_;
mod util;

use std::collections::HashMap;

use super::{
	keys,
	Array,
	CallContext,
	Dict,
	Error,
	Float,
	Function,
	NativeFun,
	RustFun,
	Panic,
	Str,
	Value,
};


/// Instantiate the stdlib.
pub fn new() -> Value {
	let mut dict = HashMap::new();

	for fun in inventory::iter::<RustFun> {
		let name = fun
			.name()
			.strip_prefix("std.")
			.expect("Builtin function name missing std prefix.");

		dict.insert(name.into(), fun.copy().into());
	}

	Dict::new(dict).into()
}


inventory::collect!(RustFun);
