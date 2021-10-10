use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(Args) }

#[derive(Trace, Finalize)]
struct Args;

impl NativeFun for Args {
	fn name(&self) -> &'static str { "std.args" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[] => Ok(context.runtime.args.copy()),
			args => Err(Panic::invalid_args(args.len() as u32, 0, context.pos))
		}
	}
}
