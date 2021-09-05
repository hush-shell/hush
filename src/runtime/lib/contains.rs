use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(Contains) }

#[derive(Trace, Finalize)]
struct Contains;

impl NativeFun for Contains {
	fn name(&self) -> &'static str { "std.contains" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::Array(ref array), item ] => Ok(array.contains(item).into()),

			[ Value::Dict(ref dict), key ] => Ok(dict.contains(key).into()),

			[ Value::String(ref string), Value::Byte(byte) ] => Ok(string.contains(*byte).into()),
			[ Value::String(_), other ] => Err(Panic::type_error(other.copy(), context.pos)),

			[ other, _ ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 2, context.pos))
		}
	}
}
