use super::{Command, Cursor, Root, State, Transition};

/// The state for lexing comments.
/// This state is generic in the sense that it returns to the previous state once the
/// entire comment is consumed.
#[derive(Debug)]
pub(super) struct Comment<S>(S);


impl<S> Comment<S>
where
	S: Into<State>,
	State: From<Self>,
{
	pub fn visit(self, cursor: &Cursor) -> Transition {
		match cursor.peek() {
			// Newline marks the end of the comment.
			Some(b'\n') => Transition::resume(self.0),

			// Otherwise, eat everything.
			_ => Transition::step(self),
		}
	}
}


impl<S> From<S> for Comment<S> {
	fn from(state: S) -> Self {
		Self(state)
	}
}


impl From<Comment<Root>> for State {
	fn from(state: Comment<Root>) -> State {
		Self::Comment(state)
	}
}


impl From<Comment<Command>> for State {
	fn from(state: Comment<Command>) -> State {
		Self::CommandComment(state)
	}
}
