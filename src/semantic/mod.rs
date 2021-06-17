mod error;
mod mem;
mod scope;
pub mod program;


use std::collections::HashSet;

use crate::symbol::{self, Symbol};
use super::syntax::{ast, lexer, SourcePos};
pub use error::{Error, ErrorKind, Errors};
pub use program::{Program, Statement, Block, Literal, Expr, UnaryOp, BinaryOp};


/// Perform static semantic analysis in the given AST.
pub fn analyze(ast: ast::Ast, interner: &mut symbol::Interner) -> Result<Program, Errors> {
	let mut scope = scope::Stack::default();
	let mut dict_keys = HashSet::default();
	let mut errors = Errors::default();

	let result = {
		let mut context = Context::new(interner, &mut scope, &mut dict_keys, &mut errors);
		analyze_block(ast.statements, &mut context)
		// Drop context before proceeding, closing opened scopes.
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
/// Errors are reported through the context, and None is returned if any error ocurred.
fn analyze_block(block: ast::Block, context: &mut Context) -> Option<Block> {
	let block = analyze_items(
		analyze_statement,
		block.0.into_vec(), // Use vec's owned iterator.
		context
	)?;

	Some(block.into())
}


/// Analyze a statement.
/// Errors are reported through the context, and None is returned if any error ocurred.
fn analyze_statement(statement: ast::Statement, context: &mut Context) -> Option<Statement> {
	match statement {
		// Let.
		ast::Statement::Let { identifier, init, pos } => {
			let slot_ix = context.scope
				.declare(identifier, pos)
				.map_err(
					|error| context.report(error)
				)
				.ok();

			let init = analyze_expr(init, context);

			let (slot_ix, right) = slot_ix.zip(init)?;

			let left = Expr::Identifier { slot_ix, pos };

			Some(Statement::Assign { left, right })
		}

		// Assign.
		ast::Statement::Assign { left, right } => {
			// TODO: Check if left is a valid l-value.
			let left = analyze_expr(left, context);
			let right = analyze_expr(right, context);

			let (left, right) = left.zip(right)?;

			Some(Statement::Assign { left, right })
		}

		// Return.
		ast::Statement::Return { expr, pos } => {
			let ret =
				if context.in_function {
					Some(())
				} else {
					context.report(Error::return_outside_function(pos));
					None
				};

			let expr = analyze_expr(expr, context);

			let (_, expr) = ret.zip(expr)?;

			Some(Statement::Return { expr })
		}

		// Break.
		ast::Statement::Break { pos } => {
			if context.in_loop {
				Some(Statement::Break)
			} else {
				context.report(Error::break_outside_loop(pos));
				None
			}
		}

		// While.
		ast::Statement::While { condition, block, .. } => {
			let condition = analyze_expr(condition, context);
			let block = {
				let mut context = context.enter_loop();
				analyze_block(block, &mut context)
			};

			let (condition, block) = condition.zip(block)?;

			Some(Statement::While { condition, block })
		}

		// For.
		ast::Statement::For { identifier, expr, block, pos } => {
			let expr = analyze_expr(expr, context);
			let id_block = {
				let mut context = context.enter_loop();

				let slot_ix = context.scope
					.declare(identifier, pos)
					.map_err(
						|error| context.report(error)
					)
					.ok();

				let block = analyze_block(block, &mut context);

				slot_ix.zip(block)
			};

			let (expr, (slot_ix, block)) = expr.zip(id_block)?;

			Some(Statement::For { slot_ix, expr, block })
		}

		// Expr.
		ast::Statement::Expr(expr) => {
			let expr = analyze_expr(expr, context)?;
			Some(Statement::Expr(expr))
		}

		// Ill-formed.
		ast::Statement::IllFormed => panic!("ill-formed statement in semantic analysis"),
	}
}


/// Analyze an expression.
/// Errors are reported through the context, and None is returned if any error ocurred.
fn analyze_expr(expr: ast::Expr, context: &mut Context) -> Option<Expr> {
	match expr {
		// Self
		ast::Expr::Self_ { pos } => {
			if context.in_function {
				let slot_ix = context.scope.resolve_or_declare(context.self_symbol);
				Some(Expr::Identifier { slot_ix, pos })
			} else {
				context.report(Error::self_outside_function(pos));
				None
			}
		}

		// Identifier.
		ast::Expr::Identifier { identifier, pos } => {
			let slot_ix = context.scope
				.resolve(identifier, pos, context.interner)
				.map_err(
					|error| context.report(error)
				)
				.ok()?;

			Some(Expr::Identifier { slot_ix, pos })
		}

		// Literal.
		ast::Expr::Literal { literal, pos } => {
			let literal = analyze_literal(literal, context)?;
			Some(Expr::Literal { literal, pos })
		}

		// UnaryOp.
		ast::Expr::UnaryOp { op, operand, pos } => {
			let operand = analyze_expr(*operand, context)?;
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
			let left = analyze_expr(*left, context);
			let right = analyze_expr(*right, context);

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
			let condition = analyze_expr(*condition, context);
			let then = {
				let mut context = context.enter_block();
				analyze_block(then, &mut context)
			};
			let otherwise = {
				let mut context = context.enter_block();
				analyze_block(otherwise, &mut context)
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
			let object = analyze_expr(*object, context);
			let field = analyze_expr(*field, context);

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
			let function = analyze_expr(*function, context);

			let args = analyze_items(
				analyze_expr,
				args.into_vec(), // Use vec's owned iterator.
				context
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


/// Analyze a literal.
/// Errors are reported through the context, and None is returned if any error ocurred.
fn analyze_literal(literal: ast::Literal, context: &mut Context) -> Option<Literal> {
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
			let array = analyze_items(
				analyze_expr,
				array.into_vec(), // Use vec's owned iterator.
				context
			)?;

			Some(Literal::Array(array))
		}

		// Dict.
		ast::Literal::Dict(items) => {
			context.dict_keys.clear();

			let items = analyze_items(
				|((symbol, pos), expr), context| {
					let duplicate_key =
						if context.dict_keys.insert(symbol) {
							Some(())
						} else {
							context.report(Error::duplicate_key(symbol, pos));
							None
						};

					let expr = analyze_expr(expr, context);

					let (_, expr) = duplicate_key.zip(expr)?;

					Some((symbol, expr))
				},
				items.into_vec(), // Use vec's owned iterator.
				context,
			)?;

			Some(Literal::Dict(items))
		}

		// Function.
		ast::Literal::Function { params, body } => {
			let mut context = context.enter_function();

			let params = analyze_items(
				|(symbol, pos), context| context.scope
						.declare(symbol, pos)
						.map_err(
							|error| context.report(error)
						)
						.ok(),
				params.into_vec(), // Use vec's owned iterator.
				&mut context
			);

			let body = analyze_block(body, &mut context);

			let frame_info = context.exit_function();

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
fn analyze_items<T, U, F, I>(
	mut analyze: F,
	iter: I,
	context: &mut Context,
) -> Option<Box<[U]>>
where
	I: IntoIterator<Item = T>,
	F: FnMut(T, &mut Context) -> Option<U>,
{
	let mut error = false;
	let mut items = Vec::new();

	for item in iter { // Use vec's owned iterator.
		if let Some(item) = analyze(item, context) {
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


/// Static semantic analysis context.
#[derive(Debug)]
struct Context<'a> {
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


impl<'a> Context<'a> {
	/// Create a new context.
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
	fn enter_block<'b>(&'b mut self) -> Context<'b> {
		self.scope.enter_block();

		Context {
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
	fn enter_loop<'b>(&'b mut self) -> Context<'b> {
		self.scope.enter_block();

		Context {
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
	fn enter_function<'b>(&'b mut self) -> Context<'b> {
		self.scope.enter_frame();

		Context {
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


impl<'a> Drop for Context<'a> {
	fn drop(&mut self) {
		if !self.dropped {
			self.scope.exit_block();
		}
	}
}
