pub mod command;
pub mod cursor;
#[cfg(test)]
mod tests;

use crate::symbol::{self, Symbol};
use cursor::{Cursor, SourcePos};


#[derive(Debug)]
pub struct Token<Kind> {
	kind: Kind,
	pos: SourcePos,
}


impl<K> Token<K> {
	pub fn new(kind: K, pos: SourcePos) -> Self {
		Self { kind, pos }
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

	// Commands require a different lexer mode:
	Command(command::Lexer),        // {}
	CaptureCommand(command::Lexer), // ${}
	AsyncCommand(command::Lexer),   // &{}

	Unexpected(u8), // Lex error.
}


#[derive(Debug)]
pub struct Lexer;


impl Lexer {
	pub fn read_token<'a>(
		cursor: &mut Cursor<'a>,
		interner: &mut symbol::Interner,
	) -> Option<Token<TokenKind>> {
		loop {
			let token_pos = cursor.pos();

			macro_rules! token {
				($kind:expr) => {
					Some(Token::new($kind, token_pos))
				};
			}

			macro_rules! operator {
				($op:expr) => {
					token!(TokenKind::Operator($op))
				};
			}

			macro_rules! command {
				($cmd:expr) => {
					token!($cmd(command::Lexer))
				};
			}

			macro_rules! try_eat {
				($pattern:expr, $val:expr) => {
					if cursor.eat($pattern) {
						return $val;
					}
				};
			}

			match cursor.peek()? {
				c if c.is_ascii_whitespace() => {
					cursor
						.take(|c| c.is_ascii_whitespace())
						.expect("at least one whitespace character should have been read");
				}

				b'#' => {
					cursor
						.skip_line()
						.expect("at least one character should have been read");
				}

				c => {
					// Identifiers, keywords and keyword literals.
					if Self::is_identifier(c) {
						return Some(
							Self::read_word(cursor, interner)
								.expect("at least one identifier character should have been read"),
						);
					}

					// TODO: literals

					// Operators and symbols:
					try_eat!(b"++", operator!(Operator::Concat));
					try_eat!(b"+", operator!(Operator::Plus));
					try_eat!(b"-", operator!(Operator::Minus));
					try_eat!(b"*", operator!(Operator::Times));
					try_eat!(b"/", operator!(Operator::Div));
					try_eat!(b"%", operator!(Operator::Mod));

					try_eat!(b">=", operator!(Operator::GreaterEquals));
					try_eat!(b"<=", operator!(Operator::LowerEquals));
					try_eat!(b">", operator!(Operator::Greater));
					try_eat!(b"<", operator!(Operator::Lower));

					try_eat!(b"!=", operator!(Operator::NotEquals));
					try_eat!(b"==", operator!(Operator::Equals));
					try_eat!(b"=", operator!(Operator::Assign));

					try_eat!(b".", operator!(Operator::Dot));
					try_eat!(b":", token!(TokenKind::Colon));
					try_eat!(b",", token!(TokenKind::Comma));

					try_eat!(b"(", token!(TokenKind::OpenParens));
					try_eat!(b")", token!(TokenKind::CloseParens));

					try_eat!(b"@[", token!(TokenKind::OpenDict));
					try_eat!(b"[", token!(TokenKind::OpenBracket));
					try_eat!(b"]", token!(TokenKind::CloseBracket));

					// Command blocks:
					try_eat!(b"{", command!(TokenKind::Command));
					try_eat!(b"${", command!(TokenKind::CaptureCommand));
					try_eat!(b"&{", command!(TokenKind::AsyncCommand));

					// Unexpected token:
					cursor.skip();
					return token!(TokenKind::Unexpected(c));
				}
			}
		}
	}


	pub fn read_word<'a>(
		cursor: &mut Cursor<'a>,
		interner: &mut symbol::Interner,
	) -> Option<Token<TokenKind>> {
		let token_pos = cursor.pos();
		let word = cursor.take(Self::is_identifier)?;

		macro_rules! token {
			($kind:expr) => {
				Some(Token::new($kind, token_pos))
			};
		}

		match word {
			// Keywords:
			b"let" => token!(TokenKind::Keyword(Keyword::Let)),
			b"if" => token!(TokenKind::Keyword(Keyword::If)),
			b"then" => token!(TokenKind::Keyword(Keyword::Then)),
			b"else" => token!(TokenKind::Keyword(Keyword::Else)),
			b"end" => token!(TokenKind::Keyword(Keyword::End)),
			b"for" => token!(TokenKind::Keyword(Keyword::For)),
			b"in" => token!(TokenKind::Keyword(Keyword::In)),
			b"do" => token!(TokenKind::Keyword(Keyword::Do)),
			b"while" => token!(TokenKind::Keyword(Keyword::While)),
			b"function" => token!(TokenKind::Keyword(Keyword::Function)),
			b"return" => token!(TokenKind::Keyword(Keyword::Return)),
			b"break" => token!(TokenKind::Keyword(Keyword::Break)),
			b"self" => token!(TokenKind::Keyword(Keyword::Self_)),

			// Literals:
			b"nil" => token!(TokenKind::Literal(Literal::Nil)),
			b"true" => token!(TokenKind::Literal(Literal::True)),
			b"false" => token!(TokenKind::Literal(Literal::False)),

			// Operators:
			b"not" => token!(TokenKind::Operator(Operator::Not)),
			b"and" => token!(TokenKind::Operator(Operator::And)),
			b"or" => token!(TokenKind::Operator(Operator::Or)),

			// Identifier:
			ident => {
				let ident = unsafe {
					// As idents are composed only of alphanumeric or underscore characters, and we
					// have checked those, they are valid utf8.
					std::str::from_utf8_unchecked(ident)
				};
				let symbol = interner.get_or_intern(ident);
				token!(TokenKind::Identifier(symbol))
			}
		}
	}


	fn is_identifier(c: u8) -> bool {
		c.is_ascii_alphanumeric() || c == b'_'
	}
}
