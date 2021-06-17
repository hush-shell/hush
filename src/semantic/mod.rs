mod error;
mod mem;
mod scope;
pub mod program;


use std::collections::HashSet;

use crate::symbol::{self, Symbol};
use super::syntax::{ast, lexer, SourcePos};
pub use error::{Error, ErrorKind, Errors};
pub use program::{Program, Statement, Block, Literal, Lvalue, Expr, UnaryOp, BinaryOp};


/// Static semantic analyzer.
#[derive(Debug)]
pub struct Analyzer<'a> {
	/// The symbol interner.
	interner: &'a mut symbol::Interner,
	/// Scope stack to check declared variables.
	scope: &'a mut scope::Stack,
	/// Hashset to check duplicate symbols in dict keys.
	dict_keys: &'a mut HashSet<Symbol>,
	/// Collected errors.
	errors: &'a mut Errors,
	/// Whether the analyzer is inside a function.
	in_function: bool,
	/// Whether the analyzer is inside a loop.
	in_loop: bool,
	/// Whether the scope has been manually dropped.
	dropped: bool,
	/// The symbol for the self keyword.
	self_symbol: Symbol,
}


impl<'a> Analyzer<'a> {
	/// Perform static semantic analysis in the given AST.
	pub fn analyze(ast: ast::Ast, interner: &mut symbol::Interner) -> Result<Program, Errors> {
		let mut scope = scope::Stack::default();
		let mut dict_keys = HashSet::default();
		let mut errors = Errors::default();

		let result = {
			let mut analyzer = Analyzer::new(interner, &mut scope, &mut dict_keys, &mut errors);
			analyzer.analyze_block(ast.statements)
			// Drop self before proceeding, closing opened scopes.
		};

		match result {
			Some(statements) if errors.0.is_empty() => Ok(
				Program {
					source: ast.path,
					statements
				}
			),

			_ => Err(errors)
		}
	}


	/// Analyze a block.
	/// None is returned if any error ocurred.
	fn analyze_block(&mut self, block: ast::Block) -> Option<Block> {
		let block = self.analyze_items(
			Self::analyze_statement,
			block.0.into_vec(), // Use vec's owned iterator.
		)?;

		Some(block.into())
	}


	/// Analyze a statement.
	/// None is returned if any error ocurred.
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
				let left = self.analyze_lvalue(left);

				if left.is_none() {
					self.report(Error::invalid_assignment(pos));
				}

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

					let slot_ix = analyzer.scope
						.declare(identifier, pos)
						.map_err(
							|error| analyzer.report(error)
						)
						.ok();

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
			ast::Statement::IllFormed => panic!("ill-formed statement in semantic analysis"),
		}
	}


	/// Analyze an expression.
	/// None is returned if any error ocurred.
	fn analyze_expr(&mut self, expr: ast::Expr) -> Option<Expr> {
		match expr {
			// Self
			ast::Expr::Self_ { pos } => {
				if self.in_function {
					let slot_ix = self.scope.resolve_or_declare(self.self_symbol);
					Some(Expr::Identifier { slot_ix, pos })
				} else {
					self.report(Error::self_outside_function(pos));
					None
				}
			}

			// Identifier.
			ast::Expr::Identifier { identifier, pos } => {
				let slot_ix = self.scope
					.resolve(identifier, pos, self.interner)
					.map_err(
						|error| self.report(error)
					)
					.ok()?;

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
			ast::Expr::CommandBlock { .. } => todo!(),

			// Ill-formed.
			ast::Expr::IllFormed => panic!("ill-formed expression in semantic analysis"),
		}
	}


	/// Analyze a statement.
	/// None is returned if any error ocurred.
	fn analyze_lvalue(&mut self, expr: ast::Expr) -> Option<Lvalue> {
		match expr {
			// Identifier.
			ast::Expr::Identifier { identifier, pos } => {
				let slot_ix = self.scope
					.resolve(identifier, pos, self.interner)
					.map_err(
						|error| self.report(error)
					)
					.ok()?;

				Some(Lvalue::Identifier { slot_ix, pos })
			}

			// Access.
			ast::Expr::Access { object, field, pos } => {
				let object = self.analyze_expr(*object);
				let field = self.analyze_expr(*field);

				let (object, field) = object.zip(field)?;

				Some(
					Lvalue::Access {
						object: Box::new(object),
						field: Box::new(field),
						pos
					}
				)
			}

			// Ill-formed.
			ast::Expr::IllFormed => panic!("ill-formed expression in semantic analysis"),

			// Other.
			_ => None, // Error will be reported in the assignment node.
		}
	}


	/// Analyze a literal.
	/// None is returned if any error ocurred.
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
						let duplicate_key =
							if analyzer.dict_keys.insert(symbol) {
								Some(())
							} else {
								analyzer.report(Error::duplicate_key(symbol, pos));
								None
							};

						let expr = analyzer.analyze_expr(expr);

						let (_, expr) = duplicate_key.zip(expr)?;

						Some((symbol, expr))
					},
					items.into_vec(), // Use vec's owned iterator.
				)?;

				Some(Literal::Dict(items))
			}

			// Function.
			ast::Literal::Function { params, body } => {
				let mut analyzer = self.enter_function();

				let params = analyzer.analyze_items(
					|analyzer, (symbol, pos)| analyzer.scope
						.declare(symbol, pos)
						.map_err(
							|error| analyzer.report(error)
						)
						.ok(),
					params.into_vec(), // Use vec's owned iterator.
				);

				let body = analyzer.analyze_block(body);

				let frame_info = analyzer.exit_function();

				let (params, body) = params.zip(body)?;

				Some(
					Literal::Function { params, frame_info, body }
				)
			}

			// Literal identifier.
			ast::Literal::Identifier(identifier) => Some(Literal::Identifier(identifier)),
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
		scope.enter_frame();

		let self_symbol = interner.get_or_intern("self");

		Self {
			interner,
			scope,
			dict_keys,
			errors,
			in_function: false,
			in_loop: false,
			dropped: false,
			self_symbol,
		}
	}


	/// Enter a new block scope.
	fn enter_block<'b>(&'b mut self) -> Analyzer<'b> {
		self.scope.enter_block();

		Analyzer {
			interner: self.interner,
			scope: self.scope,
			dict_keys: self.dict_keys,
			errors: self.errors,
			in_function: self.in_function,
			in_loop: self.in_loop,
			dropped: false,
			self_symbol: self.self_symbol,
		}
	}


	/// Enter a loop, including block scope.
	fn enter_loop<'b>(&'b mut self) -> Analyzer<'b> {
		self.scope.enter_block();

		Analyzer {
			interner: self.interner,
			scope: self.scope,
			dict_keys: self.dict_keys,
			errors: self.errors,
			in_function: self.in_function,
			in_loop: true,
			dropped: false,
			self_symbol: self.self_symbol,
		}
	}


	/// Enter a function, including block scope.
	fn enter_function<'b>(&'b mut self) -> Analyzer<'b> {
		self.scope.enter_frame();

		Analyzer {
			interner: self.interner,
			scope: self.scope,
			dict_keys: self.dict_keys,
			errors: self.errors,
			in_function: true,
			in_loop: false,
			dropped: false,
			self_symbol: self.self_symbol,
		}
	}


	/// Exit a function, dropping it's scope and returning the generated FrameInfo.
	fn exit_function<'b>(mut self) -> mem::FrameInfo {
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
