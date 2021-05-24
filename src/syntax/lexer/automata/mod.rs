mod argument;
mod command;
mod comment;
mod number;
mod root;
mod string;
mod symbol;
mod word;

use self::{
	argument::{Argument, DoubleQuoted, SingleQuoted},
	command::Command,
	comment::Comment,
	number::NumberLiteral,
	root::Root,
	string::{ByteLiteral, StringLiteral},
	symbol::{CommandSymbol, Symbol},
	word::Word,
};
use super::{
	ArgPart,
	ArgUnit,
	CommandOperator,
	Cursor,
	Error,
	ErrorKind,
	Keyword,
	Literal,
	Operator,
	SourcePos,
	Token,
	TokenKind,
};
use crate::symbol::Interner as SymbolInterner;


/// The automata may produce a token, or an error.
type Output = Result<Token, Error>;


/// The transition to be made after a character in the input has been visited.
#[derive(Debug)]
struct Transition {
	/// The next state.
	state: State,
	/// Whether to consume the visited input character.
	consume: bool,
	/// The produced output, if any.
	output: Option<Output>,
}


impl Transition {
	/// Consume the character while updating the machine state, but not producing a token
	/// yet.
	pub fn step<S: Into<State>>(state: S) -> Self {
		Self { state: state.into(), consume: true, output: None }
	}

	/// Consume the input character and produce a token.
	pub fn produce<S: Into<State>>(state: S, token: Token) -> Self {
		Self {
			state: state.into(),
			consume: true,
			output: Some(Ok(token)),
		}
	}

	/// Consume the input character and produce an error.
	pub fn error<S: Into<State>>(state: S, error: Error) -> Self {
		Self {
			state: state.into(),
			consume: true,
			output: Some(Err(error)),
		}
	}

	/// Don't consume the input character, updating the machine state instead.
	pub fn resume<S: Into<State>>(state: S) -> Self {
		Self { state: state.into(), consume: false, output: None }
	}

	/// Don't consume the input character, but produce a token.
	pub fn resume_produce<S: Into<State>>(state: S, output: Token) -> Self {
		Self {
			state: state.into(),
			consume: false,
			output: Some(Ok(output)),
		}
	}

	/// Don't consume the input character and produce an error.
	pub fn resume_error<S: Into<State>>(state: S, error: Error) -> Self {
		Self {
			state: state.into(),
			consume: false,
			output: Some(Err(error)),
		}
	}
}


/// All states in the automata.
#[derive(Debug)]
enum State {
	// Top level lexer states:
	Root(Root),
	Comment(Comment<Root>),
	NumberLiteral(NumberLiteral),
	ByteLiteral(ByteLiteral),
	StringLiteral(StringLiteral),
	Word(Word),
	Symbol(Symbol),

	// Command block lexer states:
	Command(Command),
	CommandComment(Comment<Command>),
	Argument(Argument),
	SingleQuoted(SingleQuoted),
	DoubleQuoted(DoubleQuoted),
	UnquotedWord(argument::Word<Argument>),
	SingleQuotedWord(argument::Word<SingleQuoted>),
	DoubleQuotedWord(argument::Word<DoubleQuoted>),
	Dollar(argument::Dollar<Argument>),
	QuotedDollar(argument::Dollar<DoubleQuoted>),
	CommandSymbol(CommandSymbol),
}


impl Default for State {
	fn default() -> Self {
		Root.into()
	}
}


impl State {
	pub fn visit(self, cursor: &Cursor, interner: &mut SymbolInterner) -> Transition {
		match self {
			State::Root(state) => state.visit(cursor),
			State::Comment(state) => state.visit(cursor),
			State::NumberLiteral(state) => state.visit(cursor),
			State::ByteLiteral(state) => state.visit(cursor),
			State::StringLiteral(state) => state.visit(cursor),
			State::Word(state) => state.visit(cursor, interner),
			State::Symbol(state) => state.visit(cursor),

			State::Command(state) => state.visit(cursor),
			State::CommandComment(state) => state.visit(cursor),
			State::Argument(state) => state.visit(cursor),
			State::SingleQuoted(state) => state.visit(cursor),
			State::DoubleQuoted(state) => state.visit(cursor),
			State::UnquotedWord(state) => state.visit(cursor),
			State::SingleQuotedWord(state) => state.visit(cursor),
			State::DoubleQuotedWord(state) => state.visit(cursor),
			State::Dollar(state) => state.visit(cursor, interner),
			State::QuotedDollar(state) => state.visit(cursor, interner),
			State::CommandSymbol(state) => state.visit(cursor),
		}
	}
}


/// The automata instance.
#[derive(Debug)]
pub(super) struct Automata<'a, 'b> {
	state: State,
	cursor: Cursor<'a>,
	interner: &'b mut SymbolInterner,
}


impl<'a, 'b> Automata<'a, 'b> {
	pub fn new(cursor: Cursor<'a>, interner: &'b mut SymbolInterner) -> Self {
		Self { state: State::default(), cursor, interner }
	}
}


impl<'a, 'b> Iterator for Automata<'a, 'b> {
	type Item = Output;

	fn next(&mut self) -> Option<Output> {
		loop {
			// We must temporarily take the state so that we can consume it.
			let state = std::mem::take(&mut self.state);

			let transition = state.visit(&self.cursor, self.interner);

			self.state = transition.state;

			// Check EOF *before* stepping.
			let eof = self.cursor.is_eof();

			if transition.consume {
				self.cursor.step();
			}

			if let Some(output) = transition.output {
				return Some(output);
			}

			if eof {
				return None;
			}
		}
	}
}
