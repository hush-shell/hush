use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(Bytes) }

#[derive(Trace, Finalize)]
struct Bytes;

impl NativeFun for Bytes {
	fn name(&self) -> &'static str { "std.bytes" }

	fn call(&self, mut context: CallContext) -> Result<Value, Panic> {
		match context.args_mut() {
			[ Value::String(ref string) ] => Ok(
				string
					.as_bytes()
					.iter()
					.copied()
					.map(Value::Byte)
					.collect::<Vec<_>>()
					.into()
			),

			[ other ] => Err(Panic::type_error(other.copy(), "string", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
