use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
	Type,
};


inventory::submit! { RustFun::from(StdType) }

#[derive(Trace, Finalize)]
pub struct StdType;


impl StdType {
	/// Get the type string as a value.
	pub fn get_type(value: &Value) -> Value {
		thread_local! {
			pub static NIL: Value = Type::Nil.display().into();
			pub static BOOL: Value = Type::Bool.display().into();
			pub static INT: Value = Type::Int.display().into();
			pub static FLOAT: Value = Type::Float.display().into();
			pub static BYTE: Value = Type::Byte.display().into();
			pub static STRING: Value = Type::String.display().into();
			pub static ARRAY: Value = Type::Array.display().into();
			pub static DICT: Value = Type::Dict.display().into();
			pub static FUNCTION: Value = Type::Function.display().into();
			pub static ERROR: Value = Type::Error.display().into();
		}

		let typename = match value {
			Value::Nil => &NIL,
			Value::Bool(_) => &BOOL,
			Value::Int(_) => &INT,
			Value::Float(_) => &FLOAT,
			Value::Byte(_) => &BYTE,
			Value::String(_) => &STRING,
			Value::Array(_) => &ARRAY,
			Value::Dict(_) => &DICT,
			Value::Function(_) => &FUNCTION,
			Value::Error(_) => &ERROR,
		};

		typename.with(Value::copy)
	}
}


impl NativeFun for StdType {
	fn name(&self) -> &'static str { "std.type" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ value ] => Ok(Self::get_type(value)),

			args @ [] | args @ [_, _, ..] => Err(
				Panic::invalid_args(args.len() as u32, 1, context.pos)
			),
		}
	}
}
