use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(IsEmpty) }

#[derive(Trace, Finalize)]
struct IsEmpty;

impl NativeFun for IsEmpty {
	fn name(&self) -> &'static str { "std.is_empty" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::Array(ref array) ] => Ok(array.is_empty().into()),

			[ Value::Dict(ref dict) ] => Ok(dict.is_empty().into()),

			[ Value::String(ref string) ] => Ok(string.is_empty().into()),

			[ other ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
