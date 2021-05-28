mod error;
#[cfg(test)]
mod tests;

use super::{ast, lexer::Operator};
use super::lexer::{Token, TokenKind, Keyword};
pub use error::Error;


/// The parser may report multiple errors before finishing. Instead of allocating those in
/// an vector, we delegate such handling to the caller.
pub trait ErrorReporter {
	fn report(&mut self, error: Error);
}


impl<F> ErrorReporter for F
where
	F: FnMut(Error),
{
	fn report(&mut self, error: Error) {
		self(error)
	}
}


/// The parser for Hush syntax.
#[derive(Debug)]
pub struct Parser<I, E>
where
	I: Iterator<Item = Token>
{
	// We don't use a std::iter::Peekable instead of a (Iterator, Option<Token>) pair
	// because we must be able to move from `token`, but Peekable only returns a reference.
	cursor: I,
	token: Option<Token>,
	error_reporter: E,
}


impl<I, E> Parser<I, E>
where
	I: Iterator<Item = Token>,
	E: ErrorReporter,
{
	/// Create a new parser for the given input.
	pub fn new(mut cursor: I, error_reporter: E) -> Self {
		let token = cursor.next();

		Self {
			cursor,
			token,
			error_reporter
		}
	}


	/// Parse the input, producing a top-level block.
	pub fn parse(mut self) -> ast::Block {
		loop {
			match self.parse_block() {
				Ok(block) => return block,
				Err(error) => self.error_reporter.report(error),
			};
		}
	}


	/// Step the cursor, placing the next token on self.token.
	fn step(&mut self) {
		self.token = self.cursor.next();
	}


	/// Try and eat a token.
	fn eat<F, T>(&mut self, eat: F) -> Result<T, Error>
	where
		F: FnOnce(Token) -> Result<T, (Error, Token)>
	{
		if let Some(token) = self.token.take() {
			match eat(token) {
				Ok(value) => { // Token successfully consumed.
					self.step();
					Ok(value)
				},

				Err((error, token)) => { // Fail, rollback the token and produce an error.
					self.token = Some(token);
					Err(error)
				}
			}
		} else {
			Err(Error::unexpected_eof())
		}
	}


	/// Consume the expected token, or produce an error.
	fn expect(&mut self, expected: TokenKind) -> Result<TokenKind, Error> {
		self.eat(
			|token| match token {
				Token { token, .. } if token == expected => Ok(token),
				token => Err((Error::unexpected(token.clone(), expected), token)),
			}
		)
	}


	/// Parse a block of statements, stopping when END of EOF are reached, or after a return
	/// is parsed. The lua-like grammar requires stopping after such conditions.
	fn parse_block(&mut self) -> Result<ast::Block, Error> {
		let mut block = Vec::new();

		loop {
			match &self.token {
				// Break on end of block.
				Some(Token { token: TokenKind::Keyword(Keyword::End), .. }) => break,

				Some(_) => {
					let statement = self.parse_statement()?;
					let is_return = matches!(statement, ast::Statement::Return { .. });

					block.push(statement);

					if is_return {
						// There may be no statements following a return in a block.
						break;
					}
				},

				// Break on eof.
				None => break,
			}
		}

		Ok(
			block.into_boxed_slice().into()
		)
	}


	/// Parse a single statement.
	fn parse_statement(&mut self) -> Result<ast::Statement, Error> {
		match self.token.take() {
			// Let.
			Some(Token { token: TokenKind::Keyword(Keyword::Let), pos }) => {
				self.step();

				let identifier = self.parse_identifier()?;

				Ok(
					ast::Statement::Let {
						identifier,
						pos: pos.into(),
					}
				)
			},

			// TODO: assignment.
			// TODO: let function

			// Return.
			Some(Token { token: TokenKind::Keyword(Keyword::Return), pos }) => {
				self.step();

				let expr = self.parse_expression()?;

				Ok(
					ast::Statement::Return {
						expr,
						pos: pos.into(),
					}
				)
			},

			// Break.
			Some(Token { token: TokenKind::Keyword(Keyword::Break), pos }) => {
				self.step();

				Ok(
					ast::Statement::Break {
						pos: pos.into()
					}
				)
			},

			// While.
			Some(Token { token: TokenKind::Keyword(Keyword::While), pos }) => {
				self.step();

				let condition = self.parse_expression()?;
				self.expect(TokenKind::Keyword(Keyword::Do))?;
				let block = self.parse_block()?;
				self.expect(TokenKind::Keyword(Keyword::End))?;

				Ok(
					ast::Statement::While {
						condition,
						block,
						pos: pos.into()
					}
				)
			},

			// For.
			Some(Token { token: TokenKind::Keyword(Keyword::For), pos }) => {
				self.step();

				let identifier = self.parse_identifier()?;
				self.expect(TokenKind::Keyword(Keyword::In))?;
				let expr = self.parse_expression()?;
				self.expect(TokenKind::Keyword(Keyword::Do))?;
				let block = self.parse_block()?;
				self.expect(TokenKind::Keyword(Keyword::End))?;

				Ok(
					ast::Statement::For {
						identifier,
						expr,
						block,
						pos: pos.into()
					}
				)
			},

			// Expr.
			Some(token) => {
				self.token = Some(token);

				let expr = self.parse_expression()?;

				Ok(
					ast::Statement::Expr(expr)
				)
			},

			// EOF.
			None => Err(Error::unexpected_eof()),
		}
	}


	/// Parse a single expression.
	fn parse_expression(&mut self) -> Result<ast::Expr, Error> {
		// TODO: Function call and subscript operator
		// TODO: dot access
		self.parse_or()
	}


	/// Parse a higher precedence expression, optionally ending as a logical OR.
	fn parse_or(&mut self) -> Result<ast::Expr, Error> {
		let mut expr = self.parse_and()?;

		loop {
			match self.token.take() {
				Some(Token { token: TokenKind::Operator(op), pos }) if op == Operator::Or => {
					self.step();

					let right = self.parse_and()?;

					expr = ast::Expr::BinaryOp {
						left: expr.into(),
						op: op.into(),
						right: right.into(),
						pos: pos.into(),
					};
				},

				token => {
					self.token = token;
					break;
				},
			}
		}

		Ok(expr)
	}


	/// Parse a higher precedence expression, optionally ending as a logical AND.
	fn parse_and(&mut self) -> Result<ast::Expr, Error> {
		let mut expr = self.parse_equality()?;

		loop {
			match self.token.take() {
				Some(Token { token: TokenKind::Operator(op), pos }) if op == Operator::And => {
					self.step();

					let right = self.parse_equality()?;

					expr = ast::Expr::BinaryOp {
						left: expr.into(),
						op: op.into(),
						right: right.into(),
						pos: pos.into(),
					};
				},

				token => {
					self.token = token;
					break;
				},
			}
		}

		Ok(expr)
	}


	/// Parse a higher precedence expression, optionally ending as a logical equality.
	fn parse_equality(&mut self) -> Result<ast::Expr, Error> {
		let mut expr = self.parse_comparison()?;

		loop {
			match self.token.take() {
				Some(Token { token: TokenKind::Operator(op), pos }) if op.is_equality() => {
					self.step();

					let right = self.parse_comparison()?;

					expr = ast::Expr::BinaryOp {
						left: expr.into(),
						op: op.into(),
						right: right.into(),
						pos: pos.into(),
					};
				},

				token => {
					self.token = token;
					break;
				},
			}
		}

		Ok(expr)
	}


	/// Parse a higher precedence expression, optionally ending as a logical comparison.
	fn parse_comparison(&mut self) -> Result<ast::Expr, Error> {
		let mut expr = self.parse_concat()?;

		loop {
			match self.token.take() {
				Some(Token { token: TokenKind::Operator(op), pos }) if op.is_comparison() => {
					self.step();

					let right = self.parse_concat()?;

					expr = ast::Expr::BinaryOp {
						left: expr.into(),
						op: op.into(),
						right: right.into(),
						pos: pos.into(),
					};
				},

				token => {
					self.token = token;
					break;
				},
			}
		}

		Ok(expr)
	}


	/// Parse a higher precedence expression, optionally ending as a string concat.
	fn parse_concat(&mut self) -> Result<ast::Expr, Error> {
		let mut expr = self.parse_term()?;

		loop {
			match self.token.take() {
				Some(Token { token: TokenKind::Operator(op), pos }) if op == Operator::Concat => {
					self.step();

					let right = self.parse_term()?;

					expr = ast::Expr::BinaryOp {
						left: expr.into(),
						op: op.into(),
						right: right.into(),
						pos: pos.into(),
					};
				},

				token => {
					self.token = token;
					break;
				},
			}
		}

		Ok(expr)
	}


	/// Parse a higher precedence expression, optionally ending as a additive arithmetic
	/// expression.
	fn parse_term(&mut self) -> Result<ast::Expr, Error> {
		let mut expr = self.parse_factor()?;

		loop {
			match self.token.take() {
				Some(Token { token: TokenKind::Operator(op), pos }) if op.is_term() => {
					self.step();

					let right = self.parse_factor()?;

					expr = ast::Expr::BinaryOp {
						left: expr.into(),
						op: op.into(),
						right: right.into(),
						pos: pos.into(),
					};
				},

				token => {
					self.token = token;
					break;
				},
			}
		}

		Ok(expr)
	}


	/// Parse a higher precedence expression, optionally ending as a multiplicative
	/// arithmetic expression.
	fn parse_factor(&mut self) -> Result<ast::Expr, Error> {
		let mut expr = self.parse_unary()?;

		loop {
			match self.token.take() {
				Some(Token { token: TokenKind::Operator(op), pos }) if op.is_factor() => {
					self.step();

					let right = self.parse_unary()?;

					expr = ast::Expr::BinaryOp {
						left: expr.into(),
						op: op.into(),
						right: right.into(),
						pos: pos.into(),
					};
				},

				token => {
					self.token = token;
					break;
				},
			}
		}

		Ok(expr)
	}


	/// Parse a higher precedence expression, optionally starting with a unary operator.
	fn parse_unary(&mut self) -> Result<ast::Expr, Error> {
		match self.token.take() {
			Some(Token { token: TokenKind::Operator(op), pos }) if op.is_unary() => {
				self.step();

				let operand = self.parse_unary()?;

				Ok(
					ast::Expr::UnaryOp {
						op: op.into(),
						operand: operand.into(),
						pos: pos.into(),
					}
				)
			},

			token => {
				self.token = token;
				self.parse_primary()
			},
		}
	}


	/// Parse a higher precedence expression.
	fn parse_primary(&mut self) -> Result<ast::Expr, Error> {
		match self.token.take() {
			// Identifier.
			Some(Token { token: TokenKind::Identifier(identifier), pos }) => {
				self.step();

				Ok(
					ast::Expr::Identifier {
						identifier,
						pos: pos.into(),
					}
				)
			}

			// Self.
			Some(Token { token: TokenKind::Keyword(Keyword::Self_), pos }) => {
				self.step();

				Ok(
					ast::Expr::Self_ {
						pos: pos.into(),
					}
				)
			}

			// Basic literal.
			Some(Token { token: TokenKind::Literal(literal), pos }) => {
				self.step();

				Ok(
					ast::Expr::Literal {
						literal: literal.into(),
						pos: pos.into(),
					}
				)
			},

			// Array literal.
			Some(Token { token: TokenKind::OpenBracket, pos }) => {
				self.step();

				let items = self.comma_sep(Self::parse_expression)?;
				self.expect(TokenKind::CloseBracket)?;

				Ok(
					ast::Expr::Literal {
						literal: ast::Literal::Array(items.into()),
						pos: pos.into(),
					}
				)
			},

			// Dict literal.
			Some(Token { token: TokenKind::OpenDict, pos }) => {
				self.step();

				let items = self.comma_sep(
					|parser| {
						let key = parser.parse_identifier()?;
						parser.expect(TokenKind::Colon)?;
						let value = parser.parse_expression()?;

						Ok((key, value))
					}
				)?;
				self.expect(TokenKind::CloseBracket)?;

				// TODO: warn on duplicate item.
				let dict = items
					.into_vec() // Slice has no owned iterator.
					.into_iter()
					.collect();

				Ok(
					ast::Expr::Literal {
						literal: ast::Literal::Dict(dict),
						pos: pos.into(),
					}
				)
			},

			// Function literal.
			Some(Token { token: TokenKind::Keyword(Keyword::Function), pos }) => {
				self.step();
				let function = self.parse_function()?;

				Ok(
					ast::Expr::Literal {
						literal: function,
						pos: pos.into(),
					}
				)
			},

			// TODO: command block

			// If conditional.
			Some(Token { token: TokenKind::Keyword(Keyword::If), pos }) => {
				self.step();

				let condition = self.parse_expression()?;
				self.expect(TokenKind::Keyword(Keyword::Then))?;
				let then = self.parse_block()?;
				let otherwise = {
					let has_else = self.eat(
						|token| match token {
							Token { token: TokenKind::Keyword(Keyword::End), .. } => Ok(false),
							Token { token: TokenKind::Keyword(Keyword::Else), .. } => Ok(true),
							token => Err((Error::unexpected_msg(token.clone(), "end or else"), token)),
						}
					)?;

					if has_else {
						let block = self.parse_block()?;
						self.expect(TokenKind::Keyword(Keyword::End))?;
						block
					} else {
						ast::Block::default()
					}
				};

				Ok(
					ast::Expr::If {
						condition: condition.into(),
						then,
						otherwise,
						pos: pos.into(),
					}
				)
			}

			// Parenthesis.
			Some(Token { token: TokenKind::OpenParens, .. }) => {
				self.step();

				let expr = self.parse_expression()?;
				self.expect(TokenKind::CloseParens)?;

				Ok(expr)
			}

			// Some other unexpected token.
			Some(token) => {
				// We need to restore the token because it may be some delimiter.
				self.token = Some(token.clone());
				Err(Error::unexpected_msg(token, "expression"))
			},

			None => Err(Error::unexpected_eof()),
		}
	}


	/// Parse a identifier.
	fn parse_identifier(&mut self) -> Result<ast::Symbol, Error> {
		self.eat(
			|token| match token {
				Token { token: TokenKind::Identifier(symbol), .. } => Ok(symbol),
				token => Err((Error::unexpected_msg(token.clone(), "identifier"), token)),
			}
		)
	}


	/// Parse a function literal after the function keyword.
	fn parse_function(&mut self) -> Result<ast::Literal, Error> {
		self.expect(TokenKind::OpenParens)?;
		let args = self.comma_sep(Self::parse_identifier)?;
		self.expect(TokenKind::CloseParens)?;
		let body = self.parse_block()?;
		self.expect(TokenKind::Keyword(Keyword::End))?;

		Ok(
			ast::Literal::Function { args, body }
		)
	}


	/// Comma-separated items.
	fn comma_sep<P, R>(&mut self, mut parse: P) -> Result<Box<[R]>, Error>
	where
		P: FnMut(&mut Self) -> Result<R, Error>,
	{
		let mut items = Vec::new();

		while let Ok(item) = parse(self) {
			items.push(item);

			match self.token {
				Some(Token { token: TokenKind::Comma, .. }) => self.step(),
				_ => break,
			}
		}

		Ok(items.into())
	}
}
