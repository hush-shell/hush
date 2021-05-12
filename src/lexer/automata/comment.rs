use super::{
	Cursor,
	Root,
	State,
	Transition
};

#[derive(Debug)]
pub struct Comment;


impl Comment {
	pub fn visit<'a>(self, cursor: &Cursor<'a>) -> Transition<'a> {
		match cursor.peek() {
			Some(b'\n') => Transition::revisit(Root),
			_ => Transition::step(self),
		}
	}
}


impl<'a> From<Comment> for State<'a> {
	fn from(state: Comment) -> State<'a> {
		State::Comment(state)
	}
}
