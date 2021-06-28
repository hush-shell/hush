mod error;
mod scope;
pub mod program;
#[cfg(test)]
mod tests;

use std::collections::HashSet;

use crate::{
	symbol::{self, Symbol},
	syntax::ast::IllFormed,
};
use super::syntax::{ast, lexer, SourcePos};
use program::{
	mem,
	ArgPart,
	ArgUnit,
	Argument,
	BasicCommand,
	Block,
	Command,
	CommandBlock,
	Expr,
	Literal,
	Lvalue,
	Program,
	Redirection,
	RedirectionTarget,
	Statement,
};
pub use error::{Error, ErrorKind, Errors};


/// Static semantic analyzer.
#[derive(Debug)]
pub struct Analyzer<'a> {
	/// Collected errors.
	errors: &'a mut Errors,
	/// Scope stack to check declared variables.
	scope: &'a mut scope::Stack,
	/// Hashset to check duplicate symbols in dict keys.
	dict_keys: &'a mut HashSet<Symbol>,
	/// The symbol interner.
	interner: &'a mut symbol::Interner,
	/// Whether the analyzer is inside a function.
	in_function: bool,
	/// Whether the analyzer is inside a loop.
	in_loop: bool,
	/// Whether the analyzer is inside an async context.
	in_async: bool,
	/// Whether the scope has been manually dropped.
	dropped: bool,
}


impl<'a> Analyzer<'a> {
	/// Perform static semantic analysis in the given AST.
	/// If the AST contains any ill-formed parts, the Program won't be produced, but no
	/// error will be reported for such parts, as those errors were already reported by the
	/// syntactic analysis.
	pub fn analyze(ast: ast::Ast, interner: &mut symbol::Interner) -> Result<Program, Errors> {
		let mut scope = scope::Stack::default();
		let mut dict_keys = HashSet::default();
		let mut errors = Errors::default();

		let (result, root_frame) = {
			let mut analyzer = Analyzer::new(interner, &mut scope, &mut dict_keys, &mut errors);
			let result = analyzer.analyze_block(ast.statements);
			let root_frame = analyzer.exit_frame();
			(result, root_frame)
			// Drop analyzer before proceeding, making sure everything is clean.
		};

		match result {
			Some(statements) if errors.0.is_empty() => Ok(
				Program {
					source: ast.path,
					statements,
					root_slots: root_frame.slots,
				}
			),

			_ => Err(errors)
		}
	}


	/// Analyze a block.
	/// None is returned if any error is detected.
	fn analyze_block(&mut self, block: ast::Block) -> Option<Block> {
		match block {
			ast::Block::IllFormed => None,

			ast::Block::Block(block) => {
				let block = self.analyze_items(
					Self::analyze_statement,
					block.into_vec(), // Use vec's owned iterator.
				)?;

				Some(block.into())
			}
		}
	}


	/// Analyze a statement.
	/// None is returned if any error is detected.
	fn analyze_statement(&mut self, statement: ast::Statement) -> Option<Statement> {
		match statement {
			// Let.
			ast::Statement::Let { identifier, init, pos } => {
				let slot_ix = self.scope
					.declare(identifier, pos)
					.map_err(
						|error| self.report(error)
					)
					.ok();

				let init = self.analyze_expr(init);

				let (slot_ix, right) = slot_ix.zip(init)?;

				let left = Lvalue::Identifier { slot_ix, pos };

				Some(Statement::Assign { left, right })
			}

			// Assign.
			ast::Statement::Assign { left, right, pos } => {
				let left = self
					.analyze_lvalue(left)
					.map_err(
						|lvalue| if !lvalue {
							self.report(Error::invalid_assignment(pos));
						}
					)
					.ok();

				let right = self.analyze_expr(right);

				let (left, right) = left.zip(right)?;

				Some(Statement::Assign { left, right })
			}

			// Return.
			ast::Statement::Return { expr, pos } => {
				let ret =
					if self.in_function {
						Some(())
					} else {
						self.report(Error::return_outside_function(pos));
						None
					};

				let expr = self.analyze_expr(expr);

				let (_, expr) = ret.zip(expr)?;

				Some(Statement::Return { expr })
			}

			// Break.
			ast::Statement::Break { pos } => {
				if self.in_loop {
					Some(Statement::Break)
				} else {
					self.report(Error::break_outside_loop(pos));
					None
				}
			}

			// While.
			ast::Statement::While { condition, block, .. } => {
				let condition = self.analyze_expr(condition);
				let block = {
					self.enter_loop().analyze_block(block)
				};

				let (condition, block) = condition.zip(block)?;

				Some(Statement::While { condition, block })
			}

			// For.
			ast::Statement::For { identifier, expr, block, pos } => {
				let expr = self.analyze_expr(expr);
				let id_block = {
					let mut analyzer = self.enter_loop();

					let slot_ix =
						if identifier.is_ill_formed() {
							None
						} else {
							analyzer.scope
								.declare(identifier, pos)
								.map_err(
									|error| analyzer.report(error)
								)
								.ok()
						};

					let block = analyzer.analyze_block(block);

					slot_ix.zip(block)
				};

				let (expr, (slot_ix, block)) = expr.zip(id_block)?;

				Some(Statement::For { slot_ix, expr, block })
			}

			// Expr.
			ast::Statement::Expr(expr) => {
				let expr = self.analyze_expr(expr)?;
				Some(Statement::Expr(expr))
			}

			// Ill-formed.
			ast::Statement::IllFormed => None,
		}
	}


	/// Analyze an expression.
	/// None is returned if any error is detected.
	fn analyze_expr(&mut self, expr: ast::Expr) -> Option<Expr> {
		match expr {
			// Self
			ast::Expr::Self_ { pos } => {
				if self.in_function {
					let slot_ix = self.scope.resolve_or_insert_self();
					Some(Expr::Identifier { slot_ix, pos })
				} else {
					self.report(Error::self_outside_function(pos));
					None
				}
			}

			// Identifier.
			ast::Expr::Identifier { identifier, pos } => {
				let slot_ix =
					if identifier.is_ill_formed() {
						None
					} else {
						self.scope
							.resolve(identifier, pos, self.interner)
							.map_err(
								|error| self.report(error)
							)
							.ok()
					}?;

				Some(Expr::Identifier { slot_ix, pos })
			}

			// Literal.
			ast::Expr::Literal { literal, pos } => {
				let literal = self.analyze_literal(literal)?;
				Some(Expr::Literal { literal, pos })
			}

			// UnaryOp.
			ast::Expr::UnaryOp { op, operand, pos } => {
				let operand = self.analyze_expr(*operand)?;
				Some(
					Expr::UnaryOp {
						op: op.into(),
						operand: Box::new(operand),
						pos,
					}
				)
			}

			// BinaryOp.
			ast::Expr::BinaryOp { left, op, right, pos } => {
				let left = self.analyze_expr(*left);
				let right = self.analyze_expr(*right);

				let (left, right) = left.zip(right)?;

				Some(
					Expr::BinaryOp {
						left: Box::new(left),
						op: op.into(),
						right: Box::new(right),
						pos,
					}
				)
			}

			// If.
			ast::Expr::If { condition, then, otherwise, pos } => {
				let condition = self.analyze_expr(*condition);
				let then = {
					self.enter_block().analyze_block(then)
				};
				let otherwise = {
					self.enter_block().analyze_block(otherwise)
				};

				let (condition, (then, otherwise)) = condition.zip(then.zip(otherwise))?;

				Some(
					Expr::If {
						condition: Box::new(condition),
						then,
						otherwise,
						pos
					}
				)
			}

			// Access.
			ast::Expr::Access { object, field, pos } => {
				let object = self.analyze_expr(*object);
				let field = self.analyze_expr(*field);

				let (object, field) = object.zip(field)?;

				Some(
					Expr::Access {
						object: Box::new(object),
						field: Box::new(field),
						pos
					}
				)
			}

			// Call.
			ast::Expr::Call { function, args, pos } => {
				let function = self.analyze_expr(*function);

				let args = self.analyze_items(
					Self::analyze_expr,
					args.into_vec(), // Use vec's owned iterator.
				);

				let (function, args) = function.zip(args)?;

				Some(
					Expr::Call {
						function: Box::new(function),
						args,
						pos,
					}
				)
			}

			// Command block.
			ast::Expr::CommandBlock { block, pos } => {
				let block = self.analyze_command_block(block)?;

				Some(Expr::CommandBlock { block, pos })
			},

			// Ill-formed.
			ast::Expr::IllFormed => None,
		}
	}


	/// Analyze an l-value expression.
	/// Err is returned if any error is detected. The boolean indicates if the expression is
	/// a valid l-value.
	fn analyze_lvalue(&mut self, expr: ast::Expr) -> Result<Lvalue, bool> {
		match expr {
			// Identifier.
			ast::Expr::Identifier { identifier, pos } => {
				let slot_ix =
					if identifier.is_ill_formed() {
						Err(false)
					} else {
						self.scope
							.resolve(identifier, pos, self.interner)
							.map_err(
								|error| {
									self.report(error);
									true
								}
							)
					}?;

				Ok(Lvalue::Identifier { slot_ix, pos })
			}

			// Access.
			ast::Expr::Access { object, field, pos } => {
				let object = self.analyze_expr(*object);
				let field = self.analyze_expr(*field);

				let (object, field) = object
					.zip(field)
					.ok_or(true)?;

				Ok(
					Lvalue::Access {
						object: Box::new(object),
						field: Box::new(field),
						pos
					}
				)
			}

			// Ill-formed.
			ast::Expr::IllFormed => Err(false),

			// Other.
			_ => Err(false), // Error will be reported in the assignment node.
		}
	}


	/// Analyze a literal.
	/// None is returned if any error is detected.
	fn analyze_literal(&mut self, literal: ast::Literal) -> Option<Literal> {
		match literal {
			// Nil.
			ast::Literal::Nil => Some(Literal::Nil),

			// Bool.
			ast::Literal::Bool(b) => Some(Literal::Bool(b)),

			// Int.
			ast::Literal::Int(i) => Some(Literal::Int(i)),

			// Float.
			ast::Literal::Float(f) => Some(Literal::Float(f)),

			// Byte.
			ast::Literal::Byte(b) => Some(Literal::Byte(b)),

			// String.
			ast::Literal::String(s) => Some(Literal::String(s)),

			// Array.
			ast::Literal::Array(array) => {
				let array = self.analyze_items(
					Self::analyze_expr,
					array.into_vec(), // Use vec's owned iterator.
				)?;

				Some(Literal::Array(array))
			}

			// Dict.
			ast::Literal::Dict(items) => {
				self.dict_keys.clear();

				let items = self.analyze_items(
					|analyzer, ((symbol, pos), expr)| {
						let symbol =
							if symbol.is_ill_formed() {
								None
							} else if analyzer.dict_keys.insert(symbol) {
								Some(symbol)
							} else { // Duplicate symbol.
								analyzer.report(Error::duplicate_key(symbol, pos));
								None
							};

						let expr = analyzer.analyze_expr(expr);

						let (symbol, expr) = symbol.zip(expr)?;

						Some((symbol, expr))
					},
					items.into_vec(), // Use vec's owned iterator.
				)?;

				Some(Literal::Dict(items))
			}

			// Function.
			ast::Literal::Function { params, body } => {
				let mut analyzer = self.enter_frame();

				let params_result = params
					.iter()
					.fold(
						Some(()),
						|acc, &(symbol ,pos)| {
							let result = if symbol.is_ill_formed() {
								None
							} else {
								analyzer.scope
									.declare(symbol, pos)
									.map_err(
										|error| analyzer.report(error)
									)
									.ok()
									.map(|_| ())
							};

							acc.and(result)
						}
					);

				let body = analyzer.analyze_block(body);

				let frame_info = analyzer.exit_frame();

				let (_, body) = params_result.zip(body)?;

				Some(
					Literal::Function {
						params: params.len() as u32,
						frame_info,
						body
					}
				)
			}

			// Literal identifier.
			ast::Literal::Identifier(identifier) => {
				if identifier.is_ill_formed() {
					None
				} else {
					Some(Literal::Identifier(identifier))
				}
			}
		}
	}


	/// Analyze a command block.
	/// None is returned if any error is detected.
	fn analyze_command_block(&mut self, block: ast::CommandBlock) -> Option<CommandBlock> {
		self.in_async = !block.kind.is_sync();

		let head = self.analyze_command(block.head);
		let tail = self.analyze_items(
			Self::analyze_command,
			block.tail.into_vec(), // Use vec's owned iterator.
		);

		self.in_async = false;

		let (head, tail) = head.zip(tail)?;

		Some(
			CommandBlock {
				kind: block.kind.into(),
				head,
				tail,
			}
		)
	}


	/// Analyze a command.
	/// None is returned if any error is detected.
	fn analyze_command(&mut self, command: ast::Command) -> Option<Command> {
		let previous_async = self.in_async;
		self.in_async |= !command.tail.is_empty();

		let head = self.analyze_basic_command(command.head);

		let tail = self.analyze_items(
			Self::analyze_basic_command,
			command.tail.into_vec(), // Use vec's owned iterator.
		);

		self.in_async = previous_async;

		let (head, tail) = head.zip(tail)?;

		Some(Command { head, tail })
	}


	/// Analyze a basic command.
	/// None is returned if any error is detected.
	fn analyze_basic_command(&mut self, command: ast::BasicCommand) -> Option<BasicCommand> {
		let is_builtin = matches!(
			&command.program,
			ast::Argument { parts, .. }
			if matches!(
				parts.as_ref(),
				&[ ast::ArgPart::Unit(ast::ArgUnit::Literal(ref program)) ]
				if program::command::builtin::is_builtin(program)
			)
		);

		let previous_async = self.in_async;
		self.in_async |= !command.redirections.is_empty();

		let error = self.in_async && is_builtin;
		if error {
			self.report(Error::async_builtin(command.pos));
		}

		let program = self.analyze_argument(command.program);
		let arguments = self.analyze_items(
			Self::analyze_argument,
			command.arguments.into_vec(), // Use vec's owned iterator.
		);
		let redirections = self.analyze_items(
			Self::analyze_redirection,
			command.redirections.into_vec(), // Use vec's owned iterator.
		);

		self.in_async = previous_async;

		let (program, (arguments, redirections)) = program.zip(arguments.zip(redirections))?;

		if error {
			None
		} else {
			Some(
				BasicCommand {
					program,
					arguments,
					redirections,
					abort_on_error: command.abort_on_error,
					pos: command.pos,
				}
			)
		}
	}


	/// Analyze a command argument.
	/// None is returned if any error is detected.
	fn analyze_argument(&mut self, argument: ast::Argument) -> Option<Argument> {
		if argument.is_ill_formed() {
			None
		} else {
			let parts = self.analyze_items(
				Self::analyze_arg_part,
				argument.parts.into_vec(), // Use vec's owned iterator.
			)?;

			Some(
				Argument {
					parts,
					pos: argument.pos,
				}
			)
		}
	}


	/// Analyze a command argument part.
	/// None is returned if any error is detected.
	fn analyze_arg_part(&mut self, part: ast::ArgPart) -> Option<ArgPart> {
		match part {
			ast::ArgPart::Unit(unit) => self
				.analyze_arg_unit(unit)
				.map(ArgPart::Unit),
			ast::ArgPart::Home => Some(ArgPart::Home),
			ast::ArgPart::Range(from, to) => Some(ArgPart::Range(from, to)),
			ast::ArgPart::Collection(items) => {
				let items = self.analyze_items(
					Self::analyze_arg_unit,
					items.into_vec() // Use vec's owned iterator.
				)?;

				Some(ArgPart::Collection(items))
			},
			ast::ArgPart::Star => Some(ArgPart::Star),
			ast::ArgPart::Question => Some(ArgPart::Question),
			ast::ArgPart::CharClass(chars) => Some(ArgPart::CharClass(chars)),
		}
	}


	/// Analyze a command argument unit.
	/// None is returned if any error is detected.
	fn analyze_arg_unit(&mut self, unit: ast::ArgUnit) -> Option<ArgUnit> {
		match unit {
			ast::ArgUnit::Literal(lit) => Some(ArgUnit::Literal(lit)),
			ast::ArgUnit::Dollar { symbol, pos } => {
				if symbol.is_ill_formed() {
					None
				} else {
					let slot_ix = self.scope
						.resolve(symbol, pos, self.interner)
						.map_err(
							|error| self.report(error)
						)
						.ok()?;

					Some(ArgUnit::Dollar { slot_ix, pos })
				}
			}
		}
	}


	/// Analyze a redirection.
	/// None is returned if any error is detected.
	fn analyze_redirection(&mut self, redirection: ast::Redirection) -> Option<Redirection> {
		match redirection {
			ast::Redirection::IllFormed => None,

			ast::Redirection::Output { source, target } => {
				let target = match target {
					ast::RedirectionTarget::Fd(fd) => Some(RedirectionTarget::Fd(fd)),

					ast::RedirectionTarget::Overwrite(arg) => self
						.analyze_argument(arg)
						.map(RedirectionTarget::Overwrite),

					ast::RedirectionTarget::Append(arg) => self
						.analyze_argument(arg)
						.map(RedirectionTarget::Append),
				}?;

				Some(Redirection::Output { source, target })
			},

			ast::Redirection::Input { literal, source } => {
				let source = self.analyze_argument(source)?;

				Some(Redirection::Input { literal, source })
			}
		}
	}


	/// Analyze a collection of items.
	fn analyze_items<T, U, F, I>(&mut self, mut analyze: F, iter: I) -> Option<Box<[U]>>
	where
		I: IntoIterator<Item = T>,
		F: FnMut(&mut Self, T) -> Option<U>,
	{
		let mut error = false;
		let mut items = Vec::new();

		for item in iter { // Use vec's owned iterator.
			if let Some(item) = analyze(self, item) {
				items.push(item);
			} else {
				error = true;
			}
		}

		if error {
			None
		} else {
			Some(
				items.into_boxed_slice()
			)
		}
	}
}


impl<'a> Analyzer<'a> {
	/// Create a new analyzer.
	fn new(
		interner: &'a mut symbol::Interner,
		scope: &'a mut scope::Stack,
		dict_keys: &'a mut HashSet<Symbol>,
		errors: &'a mut Errors
	) -> Self {
		let std_symbol = interner.get_or_intern("std");

		scope.enter_frame();
		scope
			.declare(std_symbol, SourcePos::default())
			.expect("failed to insert std symbol");

		Self {
			errors,
			scope,
			dict_keys,
			interner,
			in_function: false,
			in_loop: false,
			in_async: false,
			dropped: false,
		}
	}


	/// Enter a new block scope.
	fn enter_block(&mut self) -> Analyzer {
		self.scope.enter_block();

		Analyzer {
			errors: self.errors,
			scope: self.scope,
			dict_keys: self.dict_keys,
			interner: self.interner,
			in_function: self.in_function,
			in_loop: self.in_loop,
			in_async: self.in_async,
			dropped: false,
		}
	}


	/// Enter a loop, including block scope.
	fn enter_loop(&mut self) -> Analyzer {
		self.scope.enter_block();

		Analyzer {
			errors: self.errors,
			scope: self.scope,
			dict_keys: self.dict_keys,
			interner: self.interner,
			in_function: self.in_function,
			in_loop: true,
			in_async: self.in_async,
			dropped: false,
		}
	}


	/// Enter a function, including block scope.
	fn enter_frame(&mut self) -> Analyzer {
		self.scope.enter_frame();

		Analyzer {
			errors: self.errors,
			scope: self.scope,
			dict_keys: self.dict_keys,
			interner: self.interner,
			in_function: true,
			in_loop: false,
			in_async: self.in_async,
			dropped: false,
		}
	}


	/// Exit a function, dropping it's scope and returning the generated FrameInfo.
	fn exit_frame(mut self) -> mem::FrameInfo {
		self.dropped = true;
		self.scope.exit_frame()
	}


	/// Report an error.
	fn report(&mut self, error: Error) {
		self.errors.0.push(error);
	}
}


impl<'a> Drop for Analyzer<'a> {
	fn drop(&mut self) {
		if !self.dropped {
			self.scope.exit_block();
		}
	}
}
