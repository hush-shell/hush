use gc::{Finalize, Trace};

use super::{
	Float,
	NativeFun,
	Panic,
	RustFun,
	Value,
	CallContext,
};
use rand::{Rng, thread_rng};

inventory::submit! { RustFun::from(Rand) }
inventory::submit! { RustFun::from(RandInt) }

#[derive(Trace, Finalize)]
struct Rand;

#[derive(Trace, Finalize)]
struct RandInt;

impl NativeFun for Rand {
	fn name(&self) -> &'static str { "std.rand" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		let mut rng = thread_rng();
		let args = context.args();
		if args.is_empty() {
			Ok(Value::Float(Float(rng.gen::<f64>())))
		} else {
			Err(Panic::invalid_args(args.len() as u32, 0, context.pos))
		}
	}
}

impl NativeFun for RandInt {
	fn name(&self) -> &'static str { "std.randint" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		let mut rng = thread_rng();
		match context.args() {
			[ Value::Int(m), Value::Int(n) ] => Ok(Value::Int(rng.gen_range(*m..=*n))),
			[ other, _ ] => Err(Panic::type_error(other.copy(), "int", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 2, context.pos))
		}
	}
}
