use gc::{Finalize, Trace};

use crate::fmt::FmtString;
use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(ToString) }

#[derive(Trace, Finalize)]
struct ToString;

impl NativeFun for ToString {
	fn name(&self) -> &'static str { "std.to_string" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::String(ref string) ] => Ok(string.copy().into()),
			[ value ] => Ok(value.fmt_string(context.interner()).into()),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
