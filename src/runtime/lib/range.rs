use std::collections::HashMap;

use gc::{Finalize, GcCell, Trace};

use super::{
	util,
	keys,
	CallContext,
	Dict,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(Range) }

#[derive(Trace, Finalize)]
struct Range;

impl NativeFun for Range {
	fn name(&self) -> &'static str { "std.range" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ from, to, step ] => {
				let numbers = util::Numbers
					::promote([from.copy(), to.copy(), step.copy()])
					.map_err(|value| Panic::type_error(value, "int or float", context.pos))?;

				Ok(
					match numbers {
						util::Numbers::Ints([ from, to, step ]) => RangeImpl {
							from: GcCell::new(from),
							to,
							step
						}.into(),

						util::Numbers::Floats([ from, to, step ]) => RangeImpl {
							from: GcCell::new(from),
							to,
							step
						}.into(),
					}
				)
			},

			args => Err(Panic::invalid_args(args.len() as u32, 3, context.pos))
		}
	}
}


#[derive(Trace, Finalize)]
struct RangeImpl<T: 'static> {
	from: GcCell<T>,
	to: T,
	step: T,
}

impl<T> NativeFun for RangeImpl<T>
where
	T: Trace + Finalize + 'static,
	T: Clone + Default + Ord + std::ops::Add<Output = T>,
	T: Into<Value>,
{
	fn name(&self) -> &'static str { "std.range<impl>" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		let args = context.args();
		if !args.is_empty() {
			return Err(Panic::invalid_args(args.len() as u32, 0, context.pos));
		}

		let mut from = self.from.borrow_mut();
		let mut iteration = HashMap::new();

		let finished =
			if self.step > T::default() { // Step is positive.
				*from >= self.to
			} else { // Step is negative.
				*from <= self.to
			};

		let next = if finished {
			None
		} else {
			let value = from.clone();
			*from = from.clone() + self.step.clone();
			Some(value)
		};

		keys::FINISHED.with(
			|finished| iteration.insert(finished.copy(), next.is_none().into())
		);

		if let Some(next) = next {
			keys::VALUE.with(
				|value| iteration.insert(value.copy(), next.into())
			);
		}

		Ok(Dict::new(iteration).into())
	}
}
