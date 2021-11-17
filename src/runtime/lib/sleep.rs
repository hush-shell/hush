use std::time::Duration;

use gc::{Finalize, Trace};

use super::{
	CallContext,
	NativeFun,
	RustFun,
	Panic,
	Value,
};


inventory::submit!{ RustFun::from(Sleep) }

#[derive(Trace, Finalize)]
struct Sleep;

impl NativeFun for Sleep {
	fn name(&self) -> &'static str { "std.sleep" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::Int(i) ] if *i < 0 => Err(Panic::value_error(Value::Int(*i), "positive integer", context.pos)),

			[ Value::Int(i) ] => {
				std::thread::sleep(Duration::from_millis(*i as u64));
				Ok(Value::default())
			},

			[ other ] => Err(Panic::type_error(other.copy(), "int", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
