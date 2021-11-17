use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(Length) }

#[derive(Trace, Finalize)]
struct Length;

impl NativeFun for Length {
	fn name(&self) -> &'static str { "std.len" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::Array(ref array) ] => Ok(Value::Int(array.len())),
			[ Value::Dict(ref dict) ] => Ok(Value::Int(dict.len())),
			[ Value::String(ref string) ] => Ok(Value::Int(string.len() as i64)),
			[ other ] => Err(Panic::type_error(other.copy(), "string, array or dict", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
