use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(Type) }

#[derive(Trace, Finalize)]
struct Type;

impl NativeFun for Type {
	fn name(&self) -> &'static str { "std.type" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		thread_local! {
			pub static NIL: Value = "nil".into();
			pub static BOOL: Value = "bool".into();
			pub static INT: Value = "int".into();
			pub static FLOAT: Value = "float".into();
			pub static BYTE: Value = "char".into();
			pub static STRING: Value = "string".into();
			pub static ARRAY: Value = "array".into();
			pub static DICT: Value = "dict".into();
			pub static FUNCTION: Value = "function".into();
			pub static ERROR: Value = "error".into();
		}

		let typename = match context.args() {
			[ Value::Nil ] => &NIL,
			[ Value::Bool(_) ] => &BOOL,
			[ Value::Int(_) ] => &INT,
			[ Value::Float(_) ] => &FLOAT,
			[ Value::Byte(_) ] => &BYTE,
			[ Value::String(_) ] => &STRING,
			[ Value::Array(_) ] => &ARRAY,
			[ Value::Dict(_) ] => &DICT,
			[ Value::Function(_) ] => &FUNCTION,
			[ Value::Error(_) ] => &ERROR,
			args @ [] | args @ [_, _, ..] => return Err(
				Panic::invalid_args(args.len() as u32, 1, context.pos)
			),
		};

		Ok(typename.with(Value::copy))
	}
}
