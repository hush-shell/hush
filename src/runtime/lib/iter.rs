use std::collections::HashMap;

use gc::{Finalize, GcCell, Trace};

use super::{
	keys,
	Array,
	CallContext,
	Dict,
	RustFun,
	NativeFun,
	Panic,
	Str,
	Value,
};


inventory::submit! { RustFun::from(Iter) }

#[derive(Trace, Finalize)]
struct Iter;

impl NativeFun for Iter {
	fn name(&self) -> &'static str { "std.iter" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::Array(ref array) ] => Ok(
				IterImpl::Array {
					array: array.copy(),
					ix: GcCell::new(0),
				}.into()
			),

			[ Value::Dict(ref dict) ] => Ok(
				IterImpl::Dict {
					entries: GcCell::new(
						dict
							.borrow()
							.iter()
							.map(|(k, v)| (k.copy(), v.copy()))
							.collect()
					)
				}.into()
			),

			[ Value::String(ref string) ] => Ok(
				IterImpl::String {
					string: string.copy(),
					ix: GcCell::new(0),
				}.into()
			),

			[ other ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}


#[derive(Trace, Finalize)]
enum IterImpl {
	Array {
		array: Array,
		ix: GcCell<i64>,
	},
	String {
		string: Str,
		ix: GcCell<i64>,
	},
	Dict {
		entries: GcCell<Vec<(Value, Value)>>,
	}
}

impl NativeFun for IterImpl {
	fn name(&self) -> &'static str { "std.iter<impl>" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		let args = context.args();
		if !args.is_empty() {
			return Err(Panic::invalid_args(args.len() as u32, 0, context.pos));
		}

		let mut iteration = HashMap::new();

		let next = match self {
			IterImpl::Array { array, ix } => {
				let mut ix = ix.borrow_mut();
				if let Ok(value) = array.index(*ix) {
					*ix += 1;
					Some(value)
				} else {
					None
				}
			}

			IterImpl::String { string, ix } => {
				let mut ix = ix.borrow_mut();
				if let Ok(value) = string.index(*ix) {
					*ix += 1;
					Some(value)
				} else {
					None
				}
			}

			IterImpl::Dict { entries } => entries
				.borrow_mut()
				.pop()
				.map(
					|(k, v)| {
						let mut entry = HashMap::new();

						keys::KEY.with(
							|key| entry.insert(key.copy(), k)
						);

						keys::VALUE.with(
							|value| entry.insert(value.copy(), v)
						);

						Dict::new(entry).into()
					}
				)
		};

		keys::FINISHED.with(
			|finished| iteration.insert(finished.copy(), next.is_none().into())
		);

		if let Some(next) = next {
			keys::VALUE.with(
				|value| iteration.insert(value.copy(), next)
			);
		}

		Ok(Dict::new(iteration).into())
	}
}
