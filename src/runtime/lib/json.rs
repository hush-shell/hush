use std::{fmt, collections::HashMap, convert::TryFrom};

use gc::{Finalize, Trace};
use serde::{
	Deserialize,
	Serialize,
	de::{self, Visitor, SeqAccess, MapAccess},
	ser::{self, SerializeMap},
	Serializer,
	Deserializer
};

use super::{
	Dict,
	Error,
	Float,
	NativeFun,
	Panic,
	RustFun,
	Value,
	CallContext,
};


inventory::submit! { RustFun::from(Encode) }
inventory::submit! { RustFun::from(Decode) }

#[derive(Trace, Finalize)]
struct Encode;

impl NativeFun for Encode {
	fn name(&self) -> &'static str { "std.json.encode" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ value ] => serde_json::to_string_pretty(value)
					.map(Into::into)
					.map_err(
						|_| Panic::value_error(
							value.copy(),
							"nil, bool, byte, int, float, string, array or dict",
							context.pos.copy()
						)
					),

			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}

#[derive(Trace, Finalize)]
struct Decode;

impl NativeFun for Decode {
	fn name(&self) -> &'static str { "std.json.decode" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ value @ Value::String(ref string) ] => Ok(
				serde_json::from_slice(string.as_bytes())
					.unwrap_or_else(
						|error| Error::new(error.to_string().into(), value.copy()).into()
					)
			),

			[ other ] => Err(Panic::type_error(other.copy(), "string", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}

impl<'de> Deserialize<'de> for Value {
	fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct ValueVisitor;

		impl<'de> Visitor<'de> for ValueVisitor {
			type Value = Value;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("any valid JSON value")
			}

			fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
				Ok(Value::Bool(value))
			}

			fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
				Ok(Value::Int(value))
			}

			fn visit_u64<E>(self, value: u64) -> Result<Value, E>
			where
				E: de::Error,
			{
				i64::try_from(value)
					.map(Into::into)
					.map_err(|error| de::Error::custom(error))
			}

			fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
				Ok(Value::Float(value.into()))
			}

			fn visit_str<E>(self, value: &str) -> Result<Value, E>
			where
				E: de::Error,
			{
				self.visit_string(String::from(value))
			}

			fn visit_string<E>(self, value: String) -> Result<Value, E> {
				Ok(value.into())
			}

			fn visit_none<E>(self) -> Result<Value, E> {
				Ok(Value::Nil)
			}

			fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
			where
				D: Deserializer<'de>,
			{
				Deserialize::deserialize(deserializer)
			}

			fn visit_unit<E>(self) -> Result<Value, E> {
				Ok(Value::Nil)
			}

			fn visit_seq<V>(self, mut visitor: V) -> Result<Value, V::Error>
			where
				V: SeqAccess<'de>,
			{
				let mut vec = Vec::new();

				while let Some(elem) = visitor.next_element()? {
					vec.push(elem);
				}

				Ok(vec.into())
			}

			fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
			where
				V: MapAccess<'de>,
			{
				match visitor.next_key()? {
					Some(key) => {
						let mut values = HashMap::new();

						values.insert(key, visitor.next_value()?);
						while let Some((key, value)) = visitor.next_entry()? {
							values.insert(key, value);
						}

						Ok(Dict::new(values).into())
					}

					None => Ok(Dict::default().into()),
				}
			}
		}

		deserializer.deserialize_any(ValueVisitor)
	}
}

impl Serialize for Value {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		match self {
			Value::Nil => serializer.serialize_unit(),
			Value::Bool(b) => serializer.serialize_bool(*b),
			Value::Byte(b) => serializer.serialize_str(&String::from_utf8_lossy(&[*b])),
			Value::Int(n) => n.serialize(serializer),
			Value::Float(Float(n)) => n.serialize(serializer),
			Value::String(s) => serializer.serialize_str(&String::from_utf8_lossy(s.as_bytes())),
			Value::Array(v) => v.borrow().serialize(serializer),
			Value::Dict(m) => {
				let mut map = serializer.serialize_map(Some(m.borrow().len()))?;
				for (key, value) in m.borrow().iter() {
					match key {
						Value::String(_) => map.serialize_entry(key, value)?,
						_ => return Err(ser::Error::custom("json object key must be string")),
					}
				}
				map.end()
			}

			Value::Function(_) => Err(ser::Error::custom("can't serialize function")),
			Value::Error(_) => Err(ser::Error::custom("can't serialize error")),
		}
	}
}
