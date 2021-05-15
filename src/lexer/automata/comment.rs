use super::{Cursor, Command, Root, State, Transition};

#[derive(Debug)]
pub(super) struct Comment<S>(S);


impl<'a, S> Comment<S>
where
	S: Into<State<'a>>,
	State<'a>: From<Self>
{
	pub fn visit(self, cursor: &Cursor<'a>) -> Transition<'a> {
		match cursor.peek() {
			Some(b'\n') => Transition::resume(self.0),
			_ => Transition::step(self),
		}
	}
}


impl<S> From<S> for Comment<S> {
	fn from(state: S) -> Self {
		Self(state)
	}
}


impl<'a> From<Comment<Root>> for State<'a> {
	fn from(state: Comment<Root>) -> State<'a> {
		State::Comment(state)
	}
}


impl<'a> From<Comment<Command>> for State<'a> {
	fn from(state: Comment<Command>) -> State<'a> {
		State::CommandComment(state)
	}
}
