use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(Env) }

#[derive(Trace, Finalize)]
struct Env;

impl NativeFun for Env {
	fn name(&self) -> &'static str { "std.env" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::String(ref string) ] => Ok(
				std::env
					::var_os(string)
					.map(Value::from)
					.unwrap_or(Value::Nil)
			),

			[ other ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
