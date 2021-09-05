use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Str,
	Value,
};


inventory::submit! { RustFun::from(Replace) }

#[derive(Trace, Finalize)]
struct Replace;

impl NativeFun for Replace {
	fn name(&self) -> &'static str { "std.replace" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		use bstr::ByteSlice;

		match context.args() {
			[ Value::String(ref string), Value::String(ref pattern), Value::String(ref replace) ] => Ok(
				Str::from(
					string
						.as_bytes()
						.replace(pattern, replace)
				).into()
			),

			[ Value::String(_), Value::String(_), other ] => Err(Panic::type_error(other.copy(), context.pos)),
			[ Value::String(_), other, _ ] => Err(Panic::type_error(other.copy(), context.pos)),
			[ other, _, _ ] => Err(Panic::type_error(other.copy(), context.pos)),

			args => Err(Panic::invalid_args(args.len() as u32, 3, context.pos))
		}
	}
}
