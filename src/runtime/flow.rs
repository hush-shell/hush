use super::value::Value;


#[derive(Debug)]
pub enum Flow {
	Regular(Value),
	Return(Value),
	Break,
}
