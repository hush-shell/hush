pub mod command;
pub mod cursor;
mod state;
#[cfg(test)]
mod tests;

use std::fmt::{self, Display};

use crate::symbol::{self, Symbol};
use cursor::{Cursor, SourcePos};


#[derive(Debug)]
pub enum Token<Kind, Error> {
	Eof,
	Token {
		kind: Kind,
		pos: SourcePos,
	},
	Error {
		error: Error,
		pos: SourcePos,
	}
}


impl<K, E> Token<K, E> {
	pub fn new(kind: K, pos: SourcePos) -> Self {
		Self::Token { kind, pos }
	}

	pub fn error(error: E, pos: SourcePos) -> Self {
		Self::Error { error, pos }
	}


	pub fn map<U, F>(self, op: F) -> Token<U, E>
	where
		F: FnOnce(K) -> U,
	{
		match self {
			Token::Eof => Token::Eof,
			Token::Token { kind, pos } => Token::Token { kind: op(kind), pos },
			Token::Error { error, pos } => Token::Error { error, pos },
		}
	}
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
	Let,
	If,
	Then,
	Else,
	End,
	For,
	In,
	Do,
	While,
	Function,
	Return,
	Break,
	Self_,
}


#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
	Nil,
	True,
	False,
	Int(i64),
	Float(f64),
	Byte(u8),
	// String literals are not interned because they probably won't be repeated very often.
	String(Box<[u8]>),
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
	Plus,  // +
	Minus, // -
	Times, // *
	Div,   // /
	Mod,   // %

	Equals,        // ==
	NotEquals,     // !=
	Greater,       // >
	GreaterEquals, // >=
	Lower,         // <
	LowerEquals,   // <=

	Not, // not
	And, // and
	Or,  // or

	Concat, // ++
	Dot,    // .

	Assign, // =
}


#[derive(Debug)]
pub enum TokenKind {
	Identifier(Symbol),
	Keyword(Keyword),
	Literal(Literal),
	Operator(Operator),

	Colon, // :
	Comma, // ,

	OpenParens,  // (
	CloseParens, // )

	OpenBracket,  // [
	OpenDict,     // @[
	CloseBracket, // ]

	// Commands require a different lexer mode. When any of these tokens are produced, a
	// command lexer should be used to consume all command tokens, including the close block
	// token.
	Command,        // {}
	CaptureCommand, // ${}
	AsyncCommand,   // &{}
}


#[derive(Debug)]
pub enum Error<'a> {
	// Lexical errors:
	Unexpected(u8),
	UnexpectedEof,
	InvalidEscapeCode(u8),
	InvalidEscapeCodes(Box<[u8]>),
	InvalidLiteral(&'a[u8]),
}


impl<'a> Error<'a> {
	pub fn unexpected_eof<T>(pos: SourcePos) -> Token<T, Self> {
		Token::error(Self::UnexpectedEof, pos)
	}
}


impl<'a> Display for Error<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut invalid_escape_code = |code: u8| {
			write!(f, "invalid escape code '\\{}' ({:#02x})", code as char, code)
		};

		match self {
			Error::Unexpected(c) => write!(f, "unexpected character '{}' ({:#02x})", *c as char, c)?,
			Error::UnexpectedEof => write!(f, "unexpected end of file")?,
			Error::InvalidEscapeCode(code) => invalid_escape_code(*code)?,
			Error::InvalidEscapeCodes(codes) => {
				for code in codes.iter() {
					invalid_escape_code(*code)?;
				}
			}
			Error::InvalidLiteral(lit) => write!(f, "invalid literal: {}", String::from_utf8_lossy(lit))?,
		}

		Ok(())
	}
}


impl<'a> std::error::Error for Error<'a> { }


#[derive(Debug)]
pub struct Lexer;


impl Lexer {
	pub fn read_token<'a>(
		cursor: &mut Cursor<'a>,
		interner: &mut symbol::Interner,
	) -> Token<TokenKind, Error<'a>> {
		loop {
			let token_pos = cursor.pos();

			let token = |kind| Token::new(kind, token_pos);

			let operator = |op| token(TokenKind::Operator(op));

			macro_rules! try_eat {
				($pattern:expr, $val:expr) => {
					if cursor.eat($pattern) {
						return $val;
					}
				};
			}

			match cursor.peek() {
				Some(c) if c.is_ascii_whitespace() => {
					cursor
						.take(|c| c.is_ascii_whitespace())
						.expect("at least one whitespace character should have been read");
				}

				Some(b'#') => {
					cursor
						.skip_line()
						.expect("at least one character should have been read");
				}

				Some(b'\'') => {
					return Self
						::read_char_literal(cursor)
						.map(|c| TokenKind::Literal(Literal::Byte(c)));
				}

				Some(b'\"') => {
					return Self
						::read_string_literal(cursor)
						.map(|s| TokenKind::Literal(Literal::String(s)));
				}

				Some(c) => {
					// Identifiers, keywords and keyword literals.
					if Self::is_identifier(c) {
						return Self::read_word(cursor, interner);
					}

					// TODO: literals

					// Operators and symbols:
					try_eat!(b"++", operator(Operator::Concat));
					try_eat!(b"+", operator(Operator::Plus));
					try_eat!(b"-", operator(Operator::Minus));
					try_eat!(b"*", operator(Operator::Times));
					try_eat!(b"/", operator(Operator::Div));
					try_eat!(b"%", operator(Operator::Mod));

					try_eat!(b">=", operator(Operator::GreaterEquals));
					try_eat!(b"<=", operator(Operator::LowerEquals));
					try_eat!(b">", operator(Operator::Greater));
					try_eat!(b"<", operator(Operator::Lower));

					try_eat!(b"!=", operator(Operator::NotEquals));
					try_eat!(b"==", operator(Operator::Equals));
					try_eat!(b"=", operator(Operator::Assign));

					try_eat!(b".", operator(Operator::Dot));
					try_eat!(b":", token(TokenKind::Colon));
					try_eat!(b",", token(TokenKind::Comma));

					try_eat!(b"(", token(TokenKind::OpenParens));
					try_eat!(b")", token(TokenKind::CloseParens));

					try_eat!(b"@[", token(TokenKind::OpenDict));
					try_eat!(b"[", token(TokenKind::OpenBracket));
					try_eat!(b"]", token(TokenKind::CloseBracket));

					// Command blocks:
					try_eat!(b"{", token(TokenKind::Command));
					try_eat!(b"${", token(TokenKind::CaptureCommand));
					try_eat!(b"&{", token(TokenKind::AsyncCommand));

					// Unexpected token:
					cursor.skip();
					return Token::error(Error::Unexpected(c), token_pos);
				}

				None => return Token::Eof,
			}
		}
	}


	pub fn read_word<'a>(
		cursor: &mut Cursor<'a>,
		interner: &mut symbol::Interner,
	) -> Token<TokenKind, Error<'a>> {
		let token_pos = cursor.pos();

		let token = |kind| Token::new(kind, token_pos);

		let word = match cursor.take(Self::is_identifier) {
			Some(word) => word,
			None => return Token::Eof,
		};

		match word {
			// Keywords:
			b"let" => token(TokenKind::Keyword(Keyword::Let)),
			b"if" => token(TokenKind::Keyword(Keyword::If)),
			b"then" => token(TokenKind::Keyword(Keyword::Then)),
			b"else" => token(TokenKind::Keyword(Keyword::Else)),
			b"end" => token(TokenKind::Keyword(Keyword::End)),
			b"for" => token(TokenKind::Keyword(Keyword::For)),
			b"in" => token(TokenKind::Keyword(Keyword::In)),
			b"do" => token(TokenKind::Keyword(Keyword::Do)),
			b"while" => token(TokenKind::Keyword(Keyword::While)),
			b"function" => token(TokenKind::Keyword(Keyword::Function)),
			b"return" => token(TokenKind::Keyword(Keyword::Return)),
			b"break" => token(TokenKind::Keyword(Keyword::Break)),
			b"self" => token(TokenKind::Keyword(Keyword::Self_)),

			// Literals:
			b"nil" => token(TokenKind::Literal(Literal::Nil)),
			b"true" => token(TokenKind::Literal(Literal::True)),
			b"false" => token(TokenKind::Literal(Literal::False)),

			// Operators:
			b"not" => token(TokenKind::Operator(Operator::Not)),
			b"and" => token(TokenKind::Operator(Operator::And)),
			b"or" => token(TokenKind::Operator(Operator::Or)),

			// Identifier:
			ident => {
				let ident = unsafe {
					// As idents are composed only of alphanumeric or underscore characters, and we
					// have checked those, they are valid utf8.
					std::str::from_utf8_unchecked(ident)
				};
				let symbol = interner.get_or_intern(ident);
				token(TokenKind::Identifier(symbol))
			}
		}
	}


	pub fn read_string_literal<'a>(cursor: &mut Cursor<'a>,) -> Token<Box<[u8]>, Error<'a>> {
		let token_pos = cursor.pos();

		return match cursor.skip() {
			Some(b'"') => {
				let mut literal = Vec::with_capacity(10); // We expect most string literals not to be empty.
				let mut invalid_escapes = Vec::new(); // Don't allocate until we meet an error.

				while cursor.peek() != Some(b'"') {
					match Self::read_quoted_char(cursor) {
						Token::Eof => return Error::unexpected_eof(cursor.pos()),
						Token::Token { kind: c, .. } => literal.push(c),
						Token::Error { error: Error::InvalidEscapeCode(c), .. } => invalid_escapes.push(c),
						Token::Error { error, pos } => return Token::error(error, pos),
					}
				}

				cursor.skip(); // Closing quote.

				if invalid_escapes.len() > 0 {
					Token::error(
						Error::InvalidEscapeCodes(invalid_escapes.into_boxed_slice()),
						token_pos
					)
				} else {
					Token::new(literal.into_boxed_slice(), token_pos)
				}
			},
			Some(c) => return Token::error(Error::Unexpected(c), token_pos),
			None => return Token::Eof,
		}
	}


	pub fn read_char_literal<'a>(cursor: &mut Cursor<'a>,) -> Token<u8, Error<'a>> {
		let token_pos = cursor.pos();

		return match cursor.skip() {
			Some(b'\'') => {
				let c = match Self::read_quoted_char(cursor) {
					Token::Eof => return Error::unexpected_eof(cursor.pos()),
					Token::Token { kind: c, .. } => c,
					Token::Error { error, pos } => return Token::error(error, pos),
				};

				let closing_pos = cursor.pos();

				match cursor.skip() {
					Some(b'\'') => (),
					Some(c) => return Token::error(Error::Unexpected(c), closing_pos),
					None => return Error::unexpected_eof(cursor.pos()),
				}

				Token::new(c, token_pos)
			},
			Some(c) => return Token::error(Error::Unexpected(c), token_pos),
			None => return Token::Eof,
		}
	}


	pub fn read_quoted_char<'a>(cursor: &mut Cursor<'a>,) -> Token<u8, Error<'a>> {
		let token_pos = cursor.pos();

		if cursor.eat(&[b'\\']) { // Escape sequence.
			let c = match cursor.skip() {
				Some(b'"') => b'"',
				Some(b'\'') => b'\'',
				Some(b'n') => b'\n',
				Some(b't') => b'\t',
				Some(b'0') => b'\0',
				// TODO: hex literals
				Some(c) => return Token::error(Error::InvalidEscapeCode(c), token_pos),
				None => return Error::unexpected_eof(cursor.pos()),
			};

			return Token::new(c, token_pos);
		}

		match cursor.skip() {
			Some(c) => Token::new(c, token_pos),
			None => Token::Eof,
		}
	}



	fn is_identifier(c: u8) -> bool {
		c.is_ascii_alphanumeric() || c == b'_'
	}
}
