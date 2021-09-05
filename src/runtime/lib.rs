automod::dir!("src/runtime/lib");

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
