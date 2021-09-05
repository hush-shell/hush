use bstr::ByteSlice;

use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(Split) }

#[derive(Trace, Finalize)]
struct Split;

impl NativeFun for Split {
	fn name(&self) -> &'static str { "std.split" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::String(ref string), Value::String(ref pattern) ] => Ok(
				string
					.as_bytes()
					.split_str(pattern)
					.map(Value::from)
					.collect::<Vec<Value>>()
					.into()
			),

			[ Value::String(_), other ] => Err(Panic::type_error(other.copy(), context.pos)),
			[ other, _ ] => Err(Panic::type_error(other.copy(), context.pos)),

			args => Err(Panic::invalid_args(args.len() as u32, 2, context.pos))
		}
	}
}
