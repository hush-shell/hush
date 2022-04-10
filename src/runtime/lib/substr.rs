use gc::{Finalize, Trace};

use super::{
    CallContext,
    RustFun,
    NativeFun,
    Panic,
    Value,
    Error,
};


inventory::submit! { RustFun::from(Substr) }

#[derive(Trace, Finalize)]
struct Substr;

impl Substr {
    fn substr(string: &[u8], start: &i64, len: &i64) -> Result<Value, Error> {
        let string = std::str::from_utf8(string).map_err(|_| Error::new("Invalid string".into(), Value::default()))?;
        let substr: String = string.chars().skip(*start as usize).take(*len as usize).collect();
        Ok(substr.into())
    }
}

impl NativeFun for Substr {
    fn name(&self) -> &'static str { "std.substr" }

    fn call(&self, context: CallContext) -> Result<Value, Panic> {
        match context.args() {
            [ Value::String(ref string), Value::Int(start), Value::Int(len) ] => {
                let result = Self::substr(string.as_ref(), start, len);
                Ok(result.unwrap_or_else(Into::into))
            },
            [ other, Value::Int(_), Value::Int(_) ] => Err(Panic::type_error(other.copy(), "string", context.pos)),
            [ Value::String(_), other, Value::Int(_) ] => Err(Panic::type_error(other.copy(), "int", context.pos)),
            [ Value::String(_), Value::Int(_), other ] => Err(Panic::type_error(other.copy(), "int", context.pos)),
            args => Err(Panic::invalid_args(args.len() as u32, 3, context.pos))
        }
    }
}

