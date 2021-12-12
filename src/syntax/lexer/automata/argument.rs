use super::{
	word::{self, IsWord},
	expansion::{self, Expansion, ExpansionContext},
	ArgPart,
	ArgUnit,
	Command,
	Cursor,
	Checkpoint,
	Error,
	SourcePos,
	State,
	SymbolInterner,
	Token,
	TokenKind,
	Transition,
};
use crate::symbol::Symbol;


/// The state context for the Word state.
/// The Word state is generic in the sense that it returns to the previous state once it
/// is finished. Such previous state is the WordContext.
pub(super) trait WordContext: Sized {
	/// The transition to make when the argument has been consumed.
	fn resume_produce(self, value: Vec<u8>) -> Transition;
	/// Check if a character starts an expansion.
	fn expansion_start(state: Word<Self>, cursor: &Cursor, value: u8) -> Result<Transition, Word<Self>>;
	/// Check if a character should be consumed.
	fn is_word(value: u8) -> bool;
	/// Check if a character is a valid escape sequence, and return it's corresponding
	/// value.
	fn validate_escape(value: u8) -> Option<u8>;
}


/// The state for lexing raw argument words.
#[derive(Debug)]
pub(super) struct Word<C> {
	/// The parsed bytes, if any.
	value: Vec<u8>,
	/// The position of the current escape sequence, if any.
	escaping: Option<(usize, SourcePos)>,
	/// Whether to allow expansion start in the next character.
	allow_expansion_start: bool,
	/// The argument context.
	context: C,
}


impl<C> Word<C>
where
	C: WordContext,
	State: From<Self>,
{
	pub fn visit(mut self, cursor: &Cursor) -> Transition {
		match (&self, cursor.peek()) {
			// Escaped character.
			(&Self { escaping: Some((offset, pos)), .. }, Some(value)) => {
				if let Some(c) = C::validate_escape(value) {
					self.value.push(c);
					self.escaping = None;
					Transition::step(self)
				} else {
					// Invalid escape sequence.
					let escape_sequence = &cursor.slice()[offset ..= cursor.offset()];
					Transition::error(self, Error::invalid_escape_sequence(escape_sequence, pos))
				}
			}

			// Begin of escape sequence.
			(_, Some(b'\\')) => {
				self.escaping = Some((cursor.offset(), cursor.pos()));
				Transition::step(self)
			}

			// Word character, try expansion.
			(_, Some(c)) if C::is_word(c) && self.allow_expansion_start => {
				match C::expansion_start(self, cursor, c) {
					Ok(transition) => transition,
					Err(mut state) => {
						state.value.push(c);
						Transition::step(state)
					},
				}
			}

			// Word character, no expansion.
			(_, Some(c)) if C::is_word(c) => {
				self.value.push(c);
				self.allow_expansion_start = true;
				Transition::step(self)
			}

			// End of word. Let the context deal with EOF.
			_ => self.context.resume_produce(self.value),
		}
	}
}


impl<C: WordContext> From<C> for Word<C> {
	fn from(context: C) -> Self {
		Self {
			value: Vec::with_capacity(8), // We expect most literals not to be empty.
			allow_expansion_start: true,
			escaping: None,
			context,
		}
	}
}


impl From<Word<Argument>> for State {
	fn from(state: Word<Argument>) -> State {
		Self::UnquotedWord(state)
	}
}


impl From<Word<SingleQuoted>> for State {
	fn from(state: Word<SingleQuoted>) -> State {
		Self::SingleQuotedWord(state)
	}
}


impl From<Word<DoubleQuoted>> for State {
	fn from(state: Word<DoubleQuoted>) -> State {
		Self::DoubleQuotedWord(state)
	}
}


/// The state context for the Dollar state.
/// The Dollar state is generic in the sense that it returns to the previous state once it
/// is finished. Such previous state is the DollarContext.
pub(super) trait DollarContext {
	/// The transition to make when the symbol has been consumed.
	fn produce(self, symbol: Symbol, pos: SourcePos) -> Transition;
	/// The transition to make when the symbol is invalid.
	fn error(self, error: Error) -> Transition;
	/// Non-consuming variant of produce.
	fn resume(self, symbol: Symbol, pos: SourcePos) -> Transition;
	/// Non-consuming variant of error.
	fn resume_error(self, error: Error) -> Transition;
}


impl DollarContext for Argument {
	fn produce(mut self, symbol: Symbol, pos: SourcePos) -> Transition {
		self.parts.push(ArgPart::Unquoted(ArgUnit::Dollar { symbol, pos }));

		Transition::step(self)
	}

	fn error(self, error: Error) -> Transition {
		Transition::error(self, error)
	}

	fn resume(mut self, symbol: Symbol, pos: SourcePos) -> Transition {
		self.parts.push(ArgPart::Unquoted(ArgUnit::Dollar { symbol, pos }));

		Transition::resume(self)
	}

	fn resume_error(self, error: Error) -> Transition {
		Transition::resume_error(self, error)
	}
}


impl DollarContext for DoubleQuoted {
	fn produce(mut self, symbol: Symbol, pos: SourcePos) -> Transition {
		self.parts.push(ArgUnit::Dollar { symbol, pos });

		Transition::step(self)
	}

	fn error(self, error: Error) -> Transition {
		Transition::error(self, error)
	}

	fn resume(mut self, symbol: Symbol, pos: SourcePos) -> Transition {
		self.parts.push(ArgUnit::Dollar { symbol, pos });

		Transition::resume(self)
	}

	fn resume_error(self, error: Error) -> Transition {
		Transition::resume_error(self, error)
	}
}


/// The state for lexing dollar identifiers.
#[derive(Debug)]
pub(super) struct Dollar<C> {
	/// The start offset of the identifier.
	start_offset: Option<usize>,
	/// Whether the identifier is enclosed in braces. None indicates unknown.
	braces: Option<bool>,
	/// Whether the identifier is invalid.
	error: bool,
	/// The position of the dollar.
	pos: SourcePos,
	/// The argument context.
	context: C,
}


impl<C> Dollar<C>
where
	C: DollarContext + std::fmt::Debug,
	State: From<Self>,
{
	pub fn at(cursor: &Cursor, context: C) -> Self {
		Self {
			start_offset: None,
			braces: None,
			error: false,
			pos: cursor.pos(),
			context,
		}
	}


	pub fn visit(mut self, cursor: &Cursor, interner: &mut SymbolInterner) -> Transition {
		macro_rules! produce {
			($consume:expr) => {{
				// If no characters have been read, the identifier is empty, which is an error.
				let offset = self.start_offset.unwrap_or(cursor.offset());
				let identifier = &cursor.slice()[offset .. cursor.offset()];

				if identifier.is_empty() || self.error {
					return self.context
						.error(Error::invalid_identifier(identifier, self.pos))
				}

				match word::to_token(identifier, interner) {
					TokenKind::Identifier(symbol) => {
						if $consume {
							self.context.produce(symbol, self.pos)
						} else {
							self.context.resume(symbol, self.pos)
						}
					}

					_ => {
						let error = Error::invalid_identifier(identifier, self.pos);

						if $consume {
							self.context.error(error)
						} else {
							self.context.resume_error(error)
						}
					}
				}
			}};
		}

		match (&self, cursor.peek()) {
			// Open brace:
			(&Self { start_offset: None, braces: None, .. }, Some(b'{')) => {
				self.braces = Some(true);
				Transition::step(self)
			}

			// Close brace:
			(&Self { braces: Some(true), .. }, Some(b'}')) => produce!(true),

			// Head character:
			(&Self { start_offset: None, .. }, Some(c)) => {
				self.start_offset = Some(cursor.offset());
				if !c.is_word_start() {
					self.error = true;
				}
				if self.braces == None {
					self.braces = Some(false);
				}

				Transition::step(self)
			}

			// Tail character
			(&Self { start_offset: Some(_), braces: Some(false), .. }, Some(c)) => {
				if !c.is_word() {
					produce!(false)
				} else {
					Transition::step(self)
				}
			}

			// Tail character when braces
			(&Self { start_offset: Some(_), .. }, Some(c)) => {
				if !c.is_word() {
					self.error = true;
				}

				Transition::step(self)
			}

			// EOF before close brace.
			(&Self { braces: Some(true), .. }, None) => {
				self.context.error(Error::unexpected_eof(cursor.pos()))
			}

			// EOF when no braces.
			(_, None) => produce!(true),
		}
	}
}


impl From<Dollar<Argument>> for State {
	fn from(state: Dollar<Argument>) -> State {
		Self::Dollar(state)
	}
}


impl From<Dollar<DoubleQuoted>> for State {
	fn from(state: Dollar<DoubleQuoted>) -> State {
		Self::QuotedDollar(state)
	}
}


/// The state for lexing argument literals enclosed in single quotes.
#[derive(Debug)]
pub(super) struct SingleQuoted {
	/// The parsed bytes, if any.
	value: Vec<u8>,
	/// The parent state.
	parent: Argument,
}


impl SingleQuoted {
	pub fn visit(mut self, cursor: &Cursor) -> Transition {
		match cursor.peek() {
			// Closing quote.
			Some(b'\'') => {
				self.parent
					.parts
					.push(ArgPart::SingleQuoted(self.value.into_boxed_slice()));

				Transition::step(self.parent)
			}

			// This must be the start of the literal, because the WordContext instance for
			// SingleQuoted guarantees that the only non-word character is the closing quote.
			Some(_) => Transition::resume(Word::from(self)),

			// Eof.
			None => Transition::error(self.parent, Error::unexpected_eof(cursor.pos())),
		}
	}
}


impl From<Argument> for SingleQuoted {
	fn from(parent: Argument) -> Self {
		Self {
			value: Vec::with_capacity(8), // We expect most literals not to be empty.
			parent,
		}
	}
}


impl WordContext for SingleQuoted {
	fn resume_produce(mut self, value: Vec<u8>) -> Transition {
		self.value = value;
		Transition::resume(self)
	}

	fn is_word(value: u8) -> bool {
		// Comments, double quotes, symbols, dollars and whitespace are literals in single
		// quotes.
		value != b'\''
	}

	fn expansion_start(state: Word<Self>, _: &Cursor, _: u8) -> Result<Transition, Word<Self>> {
		Err(state) // No expansions inside single quotes.
	}

	fn validate_escape(value: u8) -> Option<u8> {
		match value {
			// Syntactical escape sequences:
			b'\'' => Some(value), // Escaped quotes.

			// Additional escape sequences:
			b'n' => Some(b'\n'),
			b't' => Some(b'\t'),
			b'0' => Some(b'\0'),
			b'\\' => Some(b'\\'),

			// Invalid escape sequence:
			_ => None,
		}
	}
}


impl From<SingleQuoted> for State {
	fn from(state: SingleQuoted) -> State {
		Self::SingleQuoted(state)
	}
}


/// The state for lexing argument literals enclosed in double quotes.
#[derive(Debug)]
pub(super) struct DoubleQuoted {
	/// The parts of the literal.
	parts: Vec<ArgUnit>,
	/// The parent state.
	parent: Argument,
}


impl DoubleQuoted {
	pub fn visit(mut self, cursor: &Cursor) -> Transition {
		match cursor.peek() {
			// Closing quote.
			Some(b'\"') => {
				self.parent
					.parts
					.push(ArgPart::DoubleQuoted(self.parts.into_boxed_slice()));

				Transition::step(self.parent)
			}

			// Dollar.
			Some(b'$') => Transition::step(Dollar::at(cursor, self)),

			// This must be the start of the literal, because the WordContext instance for
			// DoubleQuoted guarantees that the only non-word characters are the closing quote
			// and the dollar.
			Some(_) => Transition::resume(Word::from(self)),

			// Eof.
			None => Transition::error(self.parent, Error::unexpected_eof(cursor.pos())),
		}
	}
}


impl From<Argument> for DoubleQuoted {
	fn from(parent: Argument) -> Self {
		Self {
			parts: Vec::with_capacity(1), // We expect most literals not to be empty.
			parent,
		}
	}
}


impl WordContext for DoubleQuoted {
	fn resume_produce(mut self, value: Vec<u8>) -> Transition {
		self.parts.push(ArgUnit::Literal(value.into_boxed_slice()));

		Transition::resume(self)
	}

	fn is_word(value: u8) -> bool {
		// Comments, single quotes, symbols and whitespace are literals in double quotes.
		value != b'"' && value != b'$'
	}

	fn expansion_start(state: Word<Self>, _: &Cursor, _: u8) -> Result<Transition, Word<Self>> {
		Err(state) // No expansions inside double quotes.
	}

	fn validate_escape(value: u8) -> Option<u8> {
		match value {
			// Syntactical escape sequences:
			b'"' => Some(value), // Escaped quotes.
			b'$' => Some(value), // Escaped dollar.

			// Additional escape sequences:
			b'n' => Some(b'\n'),
			b't' => Some(b'\t'),
			b'0' => Some(b'\0'),
			b'\\' => Some(b'\\'),

			// Invalid escape sequence:
			_ => None,
		}
	}
}


impl From<DoubleQuoted> for State {
	fn from(state: DoubleQuoted) -> State {
		Self::DoubleQuoted(state)
	}
}


/// The state for lexing command arguments.
/// This state should be introduced only when the next character is a valid argument
/// starter: a quote or a word character.
#[derive(Debug)]
pub(super) struct Argument {
	/// The parts of the argument. This can be empty if only errors are produced when lexing
	/// the argument.
	parts: Vec<ArgPart>,
	/// Whether to allow home expansion. We should only allow that in the start of the argument.
	allow_home: bool,
	pos: SourcePos,
}


impl Argument {
	pub fn at(cursor: &Cursor) -> Self {
		Self {
			parts: Vec::with_capacity(1), // Any arg should have at least one part.
			allow_home: true, // Allow home in argument start.
			pos: cursor.pos(),
		}
	}


	pub fn visit(mut self, cursor: &Cursor) -> Transition {
		let allow_home = self.allow_home;
		self.allow_home = false;

		match cursor.peek() {
			// Dollar.
			Some(b'$') => Transition::step(Dollar::at(cursor, self)),

			// Single quotes.
			Some(b'\'') => Transition::step(SingleQuoted::from(self)),

			// Double quotes.
			Some(b'"') => Transition::step(DoubleQuoted::from(self)),

			// Expansion.
			Some(c) if expansion::is_start(c) => Transition::resume(
				Expansion::at(cursor, allow_home, self)
			),

			// Unquoted.
			Some(c) if Self::is_word(c) => Transition::resume(Word::from(self)),

			// End of argument.
			_ => Transition::resume_produce(
				Command,
				Token {
					kind: TokenKind::Argument(self.parts.into_boxed_slice()),
					pos: self.pos,
				},
			),
		}
	}
}


impl WordContext for Argument {
	fn resume_produce(mut self, value: Vec<u8>) -> Transition {
		self.parts.push(ArgPart::Unquoted(ArgUnit::Literal(
			value.into_boxed_slice(),
		)));

		Transition::resume(self)
	}

	fn is_word(value: u8) -> bool {
		match value {
			b'#' => false,                         // Comments.
			b'\'' | b'"' => false,                 // Quotes.
			b'>' | b'<' | b'?' | b';' => false,    // Symbols.
			b'$' => false,                         // Dollar.
			b'}' => false,                         // Close command.
			c if c.is_ascii_whitespace() => false, // Whitespace.
			_ => true,
		}
	}

	fn expansion_start(state: Word<Self>, cursor: &Cursor, value: u8) -> Result<Transition, Word<Self>> {
		// Allow expansions in unquoted.
		if expansion::is_start(value) {
			Ok(
				Transition::resume(
					Expansion::at(cursor, false, state)
				)
			)
		} else {
			Err(state)
		}
	}

	fn validate_escape(value: u8) -> Option<u8> {
		match value {
			// Syntactical escape sequences:
			b'#' => Some(value),                         // Escaped comment starter.
			b'\'' | b'"' => Some(value),                 // Escaped quotes.
			b'>' | b'<' | b'?' | b';' => Some(value),    // Escaped symbols.
			b'$' => Some(value),                         // Escaped dollar.
			c if c.is_ascii_whitespace() => Some(value), // Escaped whitespace.

			// Additional escape sequences:
			b'n' => Some(b'\n'),
			b't' => Some(b'\t'),
			b'0' => Some(b'\0'),
			b'\\' => Some(b'\\'),

			// Invalid escape sequence:
			_ => None,
		}
	}
}


impl ExpansionContext for Argument {
	fn produce(mut self, expansion: crate::syntax::lexer::ArgExpansion) -> Transition {
		self.parts.push(
			ArgPart::Expansion(expansion)
		);

		Transition::step(self)
	}

	fn rollback(self, checkpoint: Checkpoint) -> Transition {
		// If expansion parsing fails, handle it like a word.
		Transition::rollback(checkpoint, Word::from(self))
	}

	fn is_expansion_word(value: u8) -> bool {
		Self::is_word(value)
	}
}


impl ExpansionContext for Word<Argument> {
	fn produce(self, expansion: crate::syntax::lexer::ArgExpansion) -> Transition {
		let mut argument_state = self.context;

		argument_state.parts.push(ArgPart::Unquoted(ArgUnit::Literal(
			self.value.into_boxed_slice(),
		)));

		argument_state.parts.push(
			ArgPart::Expansion(expansion)
		);

		Transition::step(argument_state)
	}

	fn rollback(mut self, checkpoint: Checkpoint) -> Transition {
		self.allow_expansion_start = false;
		// If expansion parsing fails, handle it like a word.
		Transition::rollback(checkpoint, self)
	}

	fn is_expansion_word(value: u8) -> bool {
		Argument::is_word(value)
	}
}



impl From<Argument> for State {
	fn from(state: Argument) -> State {
		Self::Argument(state)
	}
}
