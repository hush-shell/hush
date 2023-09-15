//! This module uses the `ChaCha8Rng` pseudo-random generator from the rand
//! crate. It is suitable for tasks such as simulation, but should not be used
//! for applications like criptography or gambling games.
//! see: <https://rust-random.github.io/rand/rand_chacha/struct.ChaCha8Rng.html>

use std::cell::RefCell;

use gc::{Finalize, Trace};
use rand::{Rng, SeedableRng, thread_rng};
use rand_chacha::ChaCha8Rng;

use super::{
	Float,
	NativeFun,
	Panic,
	RustFun,
	Value,
	CallContext,
};

inventory::submit! { RustFun::from(Rand) }
inventory::submit! { RustFun::from(RandInt) }
inventory::submit! { RustFun::from(RandSeed) }

thread_local!(static RNG: RefCell<ChaCha8Rng> = RefCell::new(ChaCha8Rng::from_rng(thread_rng()).unwrap()));

#[derive(Trace, Finalize)]
struct Rand;

#[derive(Trace, Finalize)]
struct RandInt;

#[derive(Trace, Finalize)]
struct RandSeed;

impl NativeFun for Rand {
	fn name(&self) -> &'static str { "std.rand" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		let args = context.args();
		if args.is_empty() {
			Ok(Value::Float(Float(RNG.with(|rng| rng.borrow_mut().gen::<f64>()))))
		} else {
			Err(Panic::invalid_args(args.len() as u32, 0, context.pos))
		}
	}
}

impl NativeFun for RandInt {
	fn name(&self) -> &'static str { "std.randint" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::Int(m), Value::Int(n) ] => Ok(Value::Int(
				RNG.with(|rng| rng.borrow_mut().gen_range(*m..=*n))
			)),
			[ other, _ ] => Err(Panic::type_error(other.copy(), "int", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 2, context.pos))
		}
	}
}

impl NativeFun for RandSeed {
	fn name(&self) -> &'static str { "std.randseed" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::Int(n) ] => {
				RNG.with(|rng| *rng.borrow_mut() = ChaCha8Rng::seed_from_u64(*n as u64));
				Ok(Value::default())
			},
			[ other ] => Err(Panic::type_error(other.copy(), "int", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
