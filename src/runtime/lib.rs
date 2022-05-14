automod::dir!("src/runtime/lib");

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
	Type,
};


inventory::collect!(RustFun);


/// Instantiate the stdlib.
pub fn new() -> Value {
	let mut dict = Dict::default();

	for fun in inventory::iter::<RustFun> {
		let path = fun
			.name()
			.strip_prefix("std.")
			.expect("Builtin function name missing std prefix.");

		insert(path, fun.copy().into(), &mut dict);
	}

	dict.into()
}


fn insert(path: &str, value: Value, dict: &mut Dict) {
	match path.split_once('.') {
		None => dict.insert(path.into(), value),
		Some((key, path)) => {
			let mut dict = dict.borrow_mut();
			let dict = dict.entry(key.into()).or_insert_with(|| Dict::default().into());

			match dict {
				Value::Dict(dict) => insert(path, value, dict),
				_ => panic!("invalid value in std initialization"),
			}
		},
	}
}
