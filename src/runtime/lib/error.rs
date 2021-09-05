use gc::{Finalize, Trace};

use super::{
	CallContext,
	Error,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(ErrorFun) }

#[derive(Trace, Finalize)]
struct ErrorFun;

impl NativeFun for ErrorFun {
	fn name(&self) -> &'static str { "std.error" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::String(ref string), context ] => Ok(
				Error
					::new(string.copy(), context.copy())
					.into()
			),

			[ other, _ ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 2, context.pos))
		}
	}
}
