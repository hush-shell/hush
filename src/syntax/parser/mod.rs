mod command;
mod error;
mod sync;

use std::iter::Peekable;

use super::{
	SourcePos,
	ast,
	lexer::{
		ArgPart,
		ArgUnit,
		Keyword,
		Token,
		TokenKind,
		Operator,
		CommandOperator
	}
};
use sync::{ResultExt, WithSync, Synchronizable};
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
	I: Iterator<Item = Token>,
{
	// We don't use a std::iter::Peekable instead of a (Iterator, Option<Token>) pair
	// because we must be able to move from `token`, but Peekable only returns a reference.
	cursor: Peekable<I>,
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

		Self { cursor: cursor.peekable(), token, error_reporter }
	}


	/// Peek the next token.
	fn peek(&mut self) -> Option<&Token> {
		self.cursor.peek()
	}


	/// Step the cursor, placing the next token on self.token.
	fn step(&mut self) {
		self.token = self.cursor.next();
	}


	/// Try and eat a token.
	fn eat<F, T>(&mut self, eat: F) -> Result<T, Error>
	where
		F: FnOnce(Token) -> Result<T, (Error, Token)>,
	{
		if let Some(token) = self.token.take() {
			match eat(token) {
				Ok(value) => {
					// Token successfully consumed.
					self.step();
					Ok(value)
				}

				Err((error, token)) => {
					// Fail, rollback the token and produce an error.
					self.token = Some(token);
					Err(error)
				}
			}
		} else {
			Err(Error::unexpected_eof())
		}
	}


	/// Consume the expected token, or produce an error.
	fn expect(&mut self, expected: TokenKind) -> Result<SourcePos, Error> {
		self.eat(|token| match token {
			Token { kind: token, pos } if token == expected => Ok(pos),
			token => Err((Error::unexpected(token.clone(), expected), token)),
		})
	}


	/// Items divided by a separator.
	/// A ending trailing separator is optional.
	fn sep_by<P, R, Sep, End>(&mut self, mut parse: P, mut sep: Sep, end: End) -> Box<[R]>
	where
		P: FnMut(&mut Self) -> sync::Result<R, Error>,
		R: ast::IllFormed,
		Sep: FnMut(&TokenKind) -> bool,
		End: Fn(&TokenKind) -> bool,
	{
		let mut items = Vec::new();

		loop {
			if let Some(Token { kind: token, .. }) = &self.token {
				if end(token) {
					break;
				}
			}

			let item = parse(self)
				.synchronize(self);

			items.push(item);

			match &self.token {
				Some(Token { kind: token, .. }) if sep(token) => self.step(),
				_ => break,
			}
		}

		items.into()
	}


	/// Comma-separated items.
	fn comma_sep<P, R, End>(&mut self, parse: P, end: End) -> Box<[R]>
	where
		P: FnMut(&mut Self) -> sync::Result<R, Error>,
		R: ast::IllFormed,
		End: Fn(&TokenKind) -> bool,
	{
		self.sep_by(parse, |token| *token == TokenKind::Comma, end)
	}


	/// Semicolon-separated items.
	fn semicolon_sep<P, R, End>(&mut self, parse: P, end: End) -> Box<[R]>
	where
		P: FnMut(&mut Self) -> sync::Result<R, Error>,
		R: ast::IllFormed,
		End: Fn(&TokenKind) -> bool,
	{
		self.sep_by(parse, |token| *token == TokenKind::Semicolon, end)
	}
}


impl<I, E> Synchronizable<Error> for Parser<I, E>
where
	I: Iterator<Item = Token>,
	E: ErrorReporter,
{
	fn synchronize(&mut self, error: Error, mut strategy: sync::Strategy) {
		self.error_reporter.report(error);

		while let Some(Token { kind: token, .. }) = &self.token {
			if strategy.synchronized(token) {
				break;
			} else {
				self.step();
			}
		}
	}
}


impl<I, E> Parser<I, E>
where
	I: Iterator<Item = Token>,
	E: ErrorReporter,
{
	/// Parse the input, producing a top-level block.
	pub fn parse(mut self) -> ast::Block {
		loop {
			let block = self.parse_block();

			match self.token.take() {
				// If the token is a block terminator, the parse_block won't parse anything.
				// We must then prevent an infinite loop here.
				Some(token) if token.kind.is_block_terminator() => {
					Err(Error::unexpected_msg(token, "statement"))
						.with_sync(sync::Strategy::skip_one())
						.synchronize(&mut self)
				}

				// Stop on EOF.
				None => return block,

				token => {
					self.token = token;
				},
			}
		}
	}


	/// Parse a block of statements, stopping when ELSE, END of EOF are reached, or after a
	/// return is parsed. The Lua-like grammar requires stopping after such conditions.
	/// This method synchronizes on all errors, producing an empty block if no statements
	/// can be parsed.
	fn parse_block(&mut self) -> ast::Block {
		let mut block = Vec::new();

		loop {
			match &self.token {
				// Break on eof.
				None => break,

				// Break on end of block.
				Some(Token { kind: token, .. }) if token.is_block_terminator() => break,

				Some(_) => {
					let statement = self
						.parse_statement()
						.force_sync_skip() // Prevent the parser from getting stuck.
						.synchronize(self);

					let is_return = matches!(statement, ast::Statement::Return { .. });

					block.push(statement);

					if is_return {
						// There may be no statements following a return in a block.
						break;
					}
				}
			}
		}

		block.into_boxed_slice().into()
	}


	/// Parse a single statement.
	fn parse_statement(&mut self) -> sync::Result<ast::Statement, Error> {
		match self.token.take() {
			// Let.
			Some(Token { kind: TokenKind::Keyword(Keyword::Let), .. }) => {
				self.step();

				let (identifier, pos) = self
					.parse_identifier()
					.synchronize(self);

				let init =
					if matches!(self.token, Some(Token { kind: TokenKind::Operator(Operator::Assign), .. })) {
						self.step();
						// Don't synchronize here because this expression is the last part of the statement.
						self.parse_expression()?
					} else {
						ast::Expr::Literal {
							literal: ast::Literal::default(),
							pos,
						}
					};

				Ok(ast::Statement::Let { identifier, init, pos })
			}

			// Let function.
			Some(Token { kind: TokenKind::Keyword(Keyword::Function), pos })
				if matches!(self.peek(), Some(Token { kind: TokenKind::Identifier(_), .. })) => {
					self.step();

					// This should not fail because we have just peeked an identifier.
					let (identifier, id_pos) = self
						.parse_identifier()
						.expect("there should be an identifier");

					let (params, body) = self.parse_function()?;

					Ok(
						ast::Statement::Let {
							identifier,
							init: ast::Expr::Literal { literal: ast::Literal::Function { params, body }, pos },
							pos: id_pos,
						}
					)
				}

			// Return.
			Some(Token { kind: TokenKind::Keyword(Keyword::Return), pos }) => {
				self.step();

				// Don't synchronize here because this expression is the last part of the statement.
				let expr = match &self.token {
					Some(Token { kind, .. }) if kind.is_block_terminator() => ast::Expr::Literal {
						literal: ast::Literal::Nil,
						pos,
					},

					_ => self.parse_expression()?,
				};

				Ok(ast::Statement::Return { expr, pos })
			}

			// Break.
			Some(Token { kind: TokenKind::Keyword(Keyword::Break), pos }) => {
				self.step();

				Ok(ast::Statement::Break { pos })
			}

			// While.
			Some(Token { kind: TokenKind::Keyword(Keyword::While), pos }) => {
				self.step();

				let condition = self.parse_expression()
					.synchronize(self);

				self.expect(TokenKind::Keyword(Keyword::Do))
					.with_sync(sync::Strategy::keep())
					.synchronize(self);

				let block = self.parse_block();

				self.expect(TokenKind::Keyword(Keyword::End))
					.with_sync(sync::Strategy::keyword(Keyword::End))?;

				Ok(ast::Statement::While { condition, block, pos })
			}

			// For.
			Some(Token { kind: TokenKind::Keyword(Keyword::For), .. }) => {
				self.step();

				let (identifier, pos) = self.parse_identifier()
					.synchronize(self);

				self.expect(TokenKind::Keyword(Keyword::In))
					.with_sync(sync::Strategy::skip_one())
					.synchronize(self);

				let expr = self.parse_expression()
					.synchronize(self);

				self.expect(TokenKind::Keyword(Keyword::Do))
					.with_sync(sync::Strategy::keep())
					.synchronize(self);

				let block = self.parse_block();

				self.expect(TokenKind::Keyword(Keyword::End))
					.with_sync(sync::Strategy::keyword(Keyword::End))?;

				Ok(ast::Statement::For { identifier, expr, block, pos })
			}

			// Expr.
			Some(token) => {
				self.token = Some(token);

				// Don't synchronize here because this expression may be the last part of the statement.
				let expr = self.parse_expression()?;

				let pos = match &self.token {
					Some(Token { kind: TokenKind::Operator(Operator::Assign), pos }) => Some(*pos),
					_ => None
				};

				if let Some(pos) = pos {
					self.step();

					// Don't synchronize here because this expression is the last part of the statement.
					let right = self.parse_expression()?;

					Ok(
						ast::Statement::Assign { left: expr, right, pos }
					)
				} else {
					Ok(ast::Statement::Expr(expr))
				}
			}

			// EOF.
			None => Err(Error::unexpected_eof())
				.with_sync(sync::Strategy::eof()),
		}
	}


	/// Parse a single expression.
	fn parse_expression(&mut self) -> sync::Result<ast::Expr, Error> {
		macro_rules! binop {
			($parse_higher_prec:expr, $check:expr) => {
				move |parser: &mut Self| parser.parse_binop($parse_higher_prec, $check)
			}
		}

		let parse_factor     = binop!(Self::parse_prefix, Operator::is_factor);
		let parse_term       = binop!(parse_factor,     Operator::is_term);
		let parse_concat     = binop!(parse_term,       |&op| op == Operator::Concat);
		let parse_comparison = binop!(parse_concat,     Operator::is_comparison);
		let parse_equality   = binop!(parse_comparison, Operator::is_equality);
		let parse_and        = binop!(parse_equality,   |&op| op == Operator::And);
		let parse_or         = binop!(parse_and,        |&op| op == Operator::Or);

		parse_or(self)
	}


	/// Parse a higher precedence expression, optionally ending as a logical OR.
	fn parse_binop<P, F>(
		&mut self,
		mut parse_higher_prec_op: P,
		mut check: F,
	) -> sync::Result<ast::Expr, Error>
	where
		P: FnMut(&mut Self) -> sync::Result<ast::Expr, Error>,
		F: FnMut(&Operator) -> bool,
	{
		let mut expr = parse_higher_prec_op(self)?;

		loop {
			match self.token.take() {
				Some(Token { kind: TokenKind::Operator(op), pos }) if check(&op) => {
					self.step();

					let right = parse_higher_prec_op(self)?;

					expr = ast::Expr::BinaryOp {
						left: expr.into(),
						op: op.into(),
						right: right.into(),
						pos,
					};
				}

				token => {
					self.token = token;
					break;
				}
			}
		}

		Ok(expr)
	}


	/// Parse a higher precedence expression, optionally starting with a prefix operator.
	fn parse_prefix(&mut self) -> sync::Result<ast::Expr, Error> {
		match self.token.take() {
			Some(Token { kind: TokenKind::Operator(op), pos }) if op.is_prefix() => {
				self.step();

				let operand = self.parse_prefix()?;

				Ok(ast::Expr::UnaryOp {
					op: op.into(),
					operand: operand.into(),
					pos,
				})
			}

			token => {
				self.token = token;
				self.parse_postfix()
			}
		}
	}


	/// Parse a primary expression followed by a postfix operator.
	fn parse_postfix(&mut self) -> sync::Result<ast::Expr, Error> {
		let mut expr = self.parse_primary()?;

		loop {
			match self.token.take() {
				// Function call.
				Some(Token { kind: TokenKind::OpenParens, pos }) => {
					self.step();

					let args = self.comma_sep(
						Self::parse_expression,
						|token| *token == TokenKind::CloseParens,
					);

					self.expect(TokenKind::CloseParens)
						.with_sync(sync::Strategy::token(TokenKind::CloseParens))?;

					expr = ast::Expr::Call {
						function: expr.into(),
						args,
						pos,
					}
				},

				// Subscript operator.
				Some(Token { kind: TokenKind::OpenBracket, pos }) => {
					self.step();

					let field = self.parse_expression()
						.synchronize(self);

					self.expect(TokenKind::CloseBracket)
						.with_sync(sync::Strategy::token(TokenKind::CloseBracket))?;

					expr = ast::Expr::Access {
						object: expr.into(),
						field: field.into(),
						pos,
					}
				},

				// Dot access operator.
				Some(Token { kind: TokenKind::Operator(Operator::Dot), pos }) => {
					self.step();

					// Here, the identifier is a literal, and not a variable name. Hence, `var.id`
					// is equivalent to `var["id"]`, and not from `var[id]`.
					let (identifier, id_pos) = self.parse_identifier()?;

					let field = ast::Expr::Literal {
						literal: ast::Literal::Identifier(identifier),
						pos: id_pos,
					};

					expr = ast::Expr::Access {
						object: expr.into(),
						field: field.into(),
						pos,
					}
				},

				// Try operator.
				Some(Token { kind: TokenKind::Operator(Operator::Try), pos }) => {
					self.step();

					expr = ast::Expr::UnaryOp {
						op: ast::UnaryOp::Try,
						operand: Box::new(expr),
						pos,
					}
				},

				token => {
					self.token = token;
					break;
				}
			}
		}

		Ok(expr)
	}


	/// Parse a primary (highest precedence) expression.
	fn parse_primary(&mut self) -> sync::Result<ast::Expr, Error> {
		match self.token.take() {
			// Identifier.
			Some(Token { kind: TokenKind::Identifier(identifier), pos }) => {
				self.step();

				Ok(ast::Expr::Identifier { identifier, pos })
			}

			// Self.
			Some(Token { kind: TokenKind::Keyword(Keyword::Self_), pos }) => {
				self.step();

				Ok(ast::Expr::Self_ { pos })
			}

			// Basic literal.
			Some(Token { kind: TokenKind::Literal(literal), pos }) => {
				self.step();

				Ok(ast::Expr::Literal { literal: literal.into(), pos })
			}

			// Array literal.
			Some(Token { kind: TokenKind::OpenBracket, pos }) => {
				self.step();

				let items = self.comma_sep(
					Self::parse_expression,
					|token| *token == TokenKind::CloseBracket,
				);

				self.expect(TokenKind::CloseBracket)
					.with_sync(sync::Strategy::token(TokenKind::CloseBracket))?;

				Ok(ast::Expr::Literal {
					literal: ast::Literal::Array(items),
					pos,
				})
			}

			// Dict literal.
			Some(Token { kind: TokenKind::OpenDict, pos }) => {
				self.step();

				let items = self.comma_sep(
					|parser| {
						let key = parser.parse_identifier()
							.with_sync(sync::Strategy::skip_one())
							.synchronize(parser);

						parser.expect(TokenKind::Colon)
							.with_sync(sync::Strategy::keep())
							.synchronize(parser);

						let value = parser.parse_expression()?;

						Ok((key, value))
					},
					|token| *token == TokenKind::CloseBracket,
				);

				self.expect(TokenKind::CloseBracket)
					.with_sync(sync::Strategy::token(TokenKind::CloseBracket))?;

				Ok(ast::Expr::Literal { literal: ast::Literal::Dict(items), pos })
			}

			// Function literal.
			Some(Token { kind: TokenKind::Keyword(Keyword::Function), pos }) => {
				self.step();

				let (params, body) = self.parse_function()?;

				Ok(ast::Expr::Literal { literal: ast::Literal::Function { params, body }, pos })
			}

			// Command blocks.
			Some(token) if token.kind.is_command_block_starter() => {
				let pos = token.pos;
				self.token = Some(token);

				let block = self.parse_command_block()
					.synchronize(self);

				Ok(
					ast::Expr::CommandBlock {
						block,
						pos
					}
				)
			}

			// If conditional.
			Some(Token { kind: TokenKind::Keyword(Keyword::If), pos }) => {
				self.step();

				let (condition, then, otherwise) = self.parse_condblock()?;

				self.expect(TokenKind::Keyword(Keyword::End))
					.with_sync(sync::Strategy::keyword(Keyword::End))?;

				Ok(ast::Expr::If {
					condition: condition.into(),
					then,
					otherwise,
					pos,
				})
			}

			// Parenthesis.
			Some(Token { kind: TokenKind::OpenParens, .. }) => {
				self.step();

				let expr = self.parse_expression()
					.synchronize(self);

				self.expect(TokenKind::CloseParens)
					.with_sync(sync::Strategy::token(TokenKind::CloseParens))?;

				Ok(expr)
			}

			// Some other unexpected token.
			Some(token) => {
				self.token = Some(token.clone());
				Err(Error::unexpected_msg(token, "expression"))
					.with_sync(sync::Strategy::keep())
			}

			None => Err(Error::unexpected_eof())
				.with_sync(sync::Strategy::eof()),
		}
	}


	/// Parse a identifier.
	fn parse_identifier(&mut self) -> sync::Result<(ast::Symbol, SourcePos), Error> {
		self
			.eat(
				|token| match token {
					Token { kind: TokenKind::Identifier(symbol), pos } => Ok((symbol, pos)),
					token => Err((Error::unexpected_msg(token.clone(), "identifier"), token)),
				}
			)
			.with_sync(sync::Strategy::keep())
	}


	/// Parse a function literal after the function keyword.
	/// Returns a pair of parameters and body.
	#[allow(clippy::type_complexity)]
	fn parse_function(
		&mut self
	) -> sync::Result<(Box<[(ast::Symbol, SourcePos)]>, ast::Block), Error> {
		let result = self.expect(TokenKind::OpenParens)
			.with_sync(sync::Strategy::keep());

		let open_parens = result.is_ok();

		result.synchronize(self);

		let params = self.comma_sep(
			Self::parse_identifier,
			|token| *token == TokenKind::CloseParens,
		);

		self.expect(TokenKind::CloseParens)
			.with_sync(
				if open_parens {
					sync::Strategy::token(TokenKind::CloseParens)
				} else {
					sync::Strategy::keep()
				}
			)
			.synchronize(self);

		let body = self.parse_block();

		self.expect(TokenKind::Keyword(Keyword::End))
			.with_sync(sync::Strategy::keyword(Keyword::End))?;

		Ok((params, body))
	}


	/// Parse an if-else expression after the if keyword
	/// Returns the if condition and the it+else blocks
	fn parse_condblock(&mut self) -> sync::Result<(ast::Expr, ast::Block, ast::Block), Error> {
		let condition = self.parse_expression()
			.synchronize(self);

		self.expect(TokenKind::Keyword(Keyword::Then))
			.with_sync(sync::Strategy::keep())
			.synchronize(self);

		let then = self.parse_block();

		let otherwise = match self.token.take() {
			Some(token) => match token {
				Token { kind: TokenKind::Keyword(Keyword::End), .. } => {
					self.token = Some(token);
					Ok(ast::Block::default())
				},
				Token { kind: TokenKind::Keyword(Keyword::ElseIf), pos, .. } => {
					self.step();

					let (condition, then, otherwise) = self.parse_condblock()?;

					let stmt = ast::Statement::Expr(ast::Expr::If {
						condition: condition.into(),
						then: then,
						otherwise: otherwise,
						pos: pos,
					});

					Ok(ast::Block::Block(Box::new([stmt])))
				},
				Token { kind: TokenKind::Keyword(Keyword::Else), .. } => {
					self.step();
					let block = self.parse_block();

					Ok(block)
				},
				token => Err(Error::unexpected_msg(token.clone(), "end or else"))
			}.with_sync(sync::Strategy::block_terminator())?,
			None => Err(Error::unexpected_eof())
				.with_sync(sync::Strategy::eof())?
		};

		Ok((condition, then, otherwise))
	}
}
