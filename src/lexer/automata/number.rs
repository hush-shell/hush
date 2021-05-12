use super::{Cursor, SourcePos, State, Transition};


#[derive(Debug)]
pub struct NumberLiteral {
	start_offset: usize,
	consumed_dot: bool,
	pos: SourcePos,
}


impl NumberLiteral {
	pub fn visit<'a>(self, cursor: &Cursor<'a>) -> Transition<'a> {
		todo!()
	}
}


impl<'a> From<NumberLiteral> for State<'a> {
	fn from(state: NumberLiteral) -> State<'a> {
		State::NumberLiteral(state)
	}
}
