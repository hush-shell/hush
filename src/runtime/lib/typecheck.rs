use gc::{Finalize, Trace};

use crate::fmt;

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
	Type,
	Error,
};


inventory::submit! { RustFun::from(Typecheck) }
inventory::submit! { RustFun::from(TryTypecheck) }


/// A typecheck result.
enum TypeChecked {
	Valid(Value),
	Invalid {
		value: Value,
		expected_type: Type,
	}
}


#[derive(Trace, Finalize)]
struct Typecheck;


impl Typecheck {
	fn typecheck(context: &CallContext) -> Result<TypeChecked, Panic> {
		match context.args() {
			[ value, Value::String(expected) ] => {
				let expected_type = Type
					::parse(&expected)
					.ok_or_else(
						|| Panic::value_error(
							Value::String(expected.copy()),
							"valid type",
							context.pos.copy()
						)
					)?;

				let value = value.copy();

				Ok(
					if value.get_type() == expected_type {
						TypeChecked::Valid(value)
					}
					else {
						TypeChecked::Invalid { value, expected_type }
					}
				)
			}

			[ _, other ] => Err(Panic::type_error(other.copy(), "string", context.pos.copy())),
			args @ [] | args @ [_] | args @ [_, _, ..] => Err(
				Panic::invalid_args(args.len() as u32, 2, context.pos.copy())
			),
		}
	}
}


impl NativeFun for Typecheck {
	fn name(&self) -> &'static str { "std.typecheck" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match Self::typecheck(&context)? {
			// No problem in returning the value here, as typecheck errors are signaled as panics.
			TypeChecked::Valid(value) => Ok(value),

			TypeChecked::Invalid { value, expected_type } => Err(
				Panic::type_error(
					value,
					expected_type.display(),
					context.pos
				)
			),
		}
	}
}


#[derive(Trace, Finalize)]
struct TryTypecheck;


impl NativeFun for TryTypecheck {
	fn name(&self) -> &'static str { "std.try_typecheck" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match Typecheck::typecheck(&context)? {
			// We can't return the value here, because it would be impossible to distinguish a
			// typechecked error value from a type error.
			TypeChecked::Valid(_) => Ok(Value::default()),

			TypeChecked::Invalid { value, expected_type } => {
				let description = format!(
					"type error: expected {}, got {} ({})",
					expected_type,
					value.get_type(),
					fmt::Show(&value, context.interner()),
				);

				Ok(
					Error
						::new(description.into(), value)
						.into()
				)
			}
		}
	}
}
