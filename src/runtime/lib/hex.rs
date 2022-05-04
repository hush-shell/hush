use gc::{Finalize, Trace};

use super::{
	Error,
	NativeFun,
	Panic,
	RustFun,
	Value,
	Str,
	CallContext,
};


inventory::submit! { RustFun::from(Encode) }
inventory::submit! { RustFun::from(Decode) }

#[derive(Trace, Finalize)]
struct Encode;

impl NativeFun for Encode {
	fn name(&self) -> &'static str { "std.hex.encode" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::String(ref string) ] => Ok(hex::encode(string).into()),

			[ other ] => Err(Panic::type_error(other.copy(), "string", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}

#[derive(Trace, Finalize)]
struct Decode;

impl NativeFun for Decode {
	fn name(&self) -> &'static str { "std.hex.decode" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ value @ Value::String(ref string) ] => Ok(
				hex::decode(string)
					.map(|value| Str::from(value).into())
					.unwrap_or_else(
						|error| Error::new(error.to_string().into(), value.copy()).into()
					)
			),

			[ other ] => Err(Panic::type_error(other.copy(), "string", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
