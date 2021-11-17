use bstr::ByteSlice;

use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(Trim) }

#[derive(Trace, Finalize)]
struct Trim;

impl NativeFun for Trim {
	fn name(&self) -> &'static str { "std.trim" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::String(ref string) ] => Ok(
				string
					.as_bytes()
					.trim()
					.into()
			),

			[ other ] => Err(Panic::type_error(other.copy(), "string", context.pos)),

			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
