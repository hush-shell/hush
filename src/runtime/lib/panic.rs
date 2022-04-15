use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(UserPanic) }

#[derive(Trace, Finalize)]
struct UserPanic;

impl NativeFun for UserPanic {
	fn name(&self) -> &'static str { "std.panic" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ value ] => Err(Panic::user(value.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
