use super::{
	Cursor,
	Error,
	ErrorKind,
	InvalidLiteral,
	Literal,
	Root,
	SourcePos,
	State,
	Token,
	TokenKind,
	Transition,
};


#[derive(Debug)]
pub(super) struct NumberLiteral {
	start_offset: usize,
	consumed_decimal: Option<bool>,
	consumed_exponent: Option<bool>,
	pos: SourcePos,
}


impl NumberLiteral {
	pub fn at(cursor: &Cursor) -> Self {
		Self {
			start_offset: cursor.offset(),
			consumed_decimal: None,
			consumed_exponent: None,
			pos: cursor.pos(),
		}
	}


	pub fn visit<'a>(mut self, cursor: &Cursor<'a>) -> Transition<'a> {
		let error = |error| Transition::error(Root, Error { error, pos: self.pos });

		match (&self, cursor.peek()) {
			// There must be a single dot, and it must precede the exponent if any.
			(
				&Self {
					consumed_decimal: None, consumed_exponent: None, ..
				},
				Some(b'.'),
			) => {
				self.consumed_decimal = Some(false);
				Transition::step(self)
			}

			// Exponent may be present regardless of dot.
			(&Self { consumed_exponent: None, .. }, Some(b'e')) => {
				self.consumed_exponent = Some(false);
				Transition::step(self)
			}

			// Consume digits.
			(_, Some(value)) if value.is_ascii_digit() => {
				// If a dot or an exponent preceded, then set the according flag.
				if self.consumed_decimal == Some(false) {
					self.consumed_decimal = Some(true);
				}
				if self.consumed_exponent == Some(false) {
					self.consumed_exponent = Some(true);
				}

				Transition::step(self)
			}

			// A dot or an exponent must be followed by a digit.
			(&Self { consumed_decimal: Some(false), .. }, value)
			| (&Self { consumed_exponent: Some(false), .. }, value) => {
				if let Some(value) = value {
					error(ErrorKind::Unexpected(value))
				} else {
					error(ErrorKind::UnexpectedEof)
				}
			}

			// Stop and produce if a non-digit is found, including EOF.
			(_, _) => match self.parse(cursor) {
				Ok(token) => Transition::revisit_produce(Root, token),
				Err(error) => Transition::error(Root, error),
			},
		}
	}


	fn parse<'a>(&self, cursor: &Cursor<'a>) -> Result<Token, Error<'a>> {
		let number = &cursor.slice()[self.start_offset .. cursor.offset()];

		let literal = |literal| Ok(Token { token: TokenKind::Literal(literal), pos: self.pos });

		let error = Err(Error {
			error: ErrorKind::InvalidLiteral(InvalidLiteral::InvalidNumber(number)),
			pos: self.pos,
		});

		// There is no method in std to parse a number from a byte array.
		let number_str = std::str::from_utf8(number)
			.expect("number literals should be valid ascii, which should be valid utf8");

		if self.is_float() {
			match number_str.parse() {
				Ok(float) => literal(Literal::Float(float)),
				Err(_) => error,
			}
		} else {
			match number_str.parse() {
				Ok(int) => literal(Literal::Int(int)),
				Err(_) => error,
			}
		}
	}


	fn is_float(&self) -> bool {
		self.consumed_decimal.is_some() || self.consumed_exponent.is_some()
	}
}


impl<'a> From<NumberLiteral> for State<'a> {
	fn from(state: NumberLiteral) -> State<'a> {
		State::NumberLiteral(state)
	}
}
