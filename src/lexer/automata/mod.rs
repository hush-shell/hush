mod argument;
mod comment;
mod command;
mod number;
mod root;
mod string;
mod symbol;
mod word;

use self::{
	argument::{Arg, SingleQuoted, DoubleQuoted},
	comment::Comment,
	command::Command,
	number::NumberLiteral,
	root::Root,
	string::{ByteLiteral, StringLiteral},
	symbol::{CommandSymbol, Symbol},
	word::Word,
};
use super::{
	Argument,
	BasicArgument,
	Cursor,
	Error,
	ErrorKind,
	InvalidEscapeCode,
	InvalidLiteral,
	Keyword,
	Literal,
	Operator,
	CommandOperator,
	SourcePos,
	Token,
	TokenKind,
};
use crate::symbol::Interner as SymbolInterner;


type Output<'a> = Result<Token, Error<'a>>;


#[derive(Debug)]
struct Transition<'a> {
	/// The currect state.
	state: State<'a>,
	/// Whether to consume the visited input character.
	consume: bool,
	/// The produced output, if any.
	output: Option<Output<'a>>,
}


impl<'a> Transition<'a> {
	/// Consume the character while updating the machine state, but not producing a token
	/// yet.
	pub fn step<S: Into<State<'a>>>(state: S) -> Self {
		Self { state: state.into(), consume: true, output: None }
	}

	/// Consume the input character and produce a token.
	pub fn produce<S: Into<State<'a>>>(state: S, token: Token) -> Self {
		Self {
			state: state.into(),
			consume: true,
			output: Some(Ok(token)),
		}
	}

	/// Consume the input character and produce an error.
	pub fn error<S: Into<State<'a>>>(state: S, error: Error<'a>) -> Self {
		Self {
			state: state.into(),
			consume: true,
			output: Some(Err(error)),
		}
	}

	/// Don't consume the input character, updating the machine state instead.
	pub fn resume<S: Into<State<'a>>>(state: S) -> Self {
		Self { state: state.into(), consume: false, output: None }
	}

	/// Don't consume the input character, but produce a token.
	pub fn resume_produce<S: Into<State<'a>>>(state: S, output: Token) -> Self {
		Self {
			state: state.into(),
			consume: false,
			output: Some(Ok(output)),
		}
	}

	/// Don't consume the input character and produce an error.
	pub fn resume_error<S: Into<State<'a>>>(state: S, error: Error<'a>) -> Self {
		Self {
			state: state.into(),
			consume: false,
			output: Some(Err(error)),
		}
	}
}


#[derive(Debug)]
enum State<'a> {
	// Top level lexer states:
	Root(Root),
	Comment(Comment<Root>),
	NumberLiteral(NumberLiteral),
	ByteLiteral(ByteLiteral<'a>),
	StringLiteral(StringLiteral<'a>),
	Word(Word),
	Symbol(Symbol),
	// Command block lexer states:
	Command(Command),
	CommandComment(Comment<Command>),
	Argument(Arg<'a>),
	SingleQuoted(SingleQuoted<'a>),
	DoubleQuoted(DoubleQuoted<'a>),
	UnquotedWord(argument::Word<'a, Arg<'a>>),
	SingleQuotedWord(argument::Word<'a, SingleQuoted<'a>>),
	DoubleQuotedWord(argument::Word<'a, DoubleQuoted<'a>>),
	CommandSymbol(CommandSymbol),
}


impl<'a> Default for State<'a> {
	fn default() -> Self {
		Root.into()
	}
}


impl<'a> State<'a> {
	pub fn visit(self, cursor: &Cursor<'a>, interner: &mut SymbolInterner) -> Transition<'a> {
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
			State::CommandSymbol(state) => state.visit(cursor),
		}
	}
}


#[derive(Debug)]
pub(super) struct Automata<'a, 'b> {
	state: State<'a>,
	cursor: Cursor<'a>,
	interner: &'b mut SymbolInterner,
}


impl<'a, 'b> Automata<'a, 'b> {
	pub fn new(cursor: Cursor<'a>, interner: &'b mut SymbolInterner) -> Self {
		Self { state: State::default(), cursor, interner }
	}
}


impl<'a, 'b> Iterator for Automata<'a, 'b> {
	type Item = Output<'a>;

	fn next(&mut self) -> Option<Output<'a>> {
		loop {
			// We must temporarily take the state so that we can consume it.
			let state = std::mem::take(&mut self.state);

			let transition = state.visit(&self.cursor, self.interner);

			self.state = transition.state;

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
