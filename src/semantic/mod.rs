mod error;
mod scope;
pub mod program;


use std::collections::HashSet;

use crate::symbol::Symbol;
use super::syntax::{ast, lexer, SourcePos};
pub use error::{Error, ErrorKind, Errors};
pub use program::{Program, Statement, Block, Literal, Expr, UnaryOp, BinaryOp};


/// Perform semantic analysis in the given AST.
pub fn analyze(ast: ast::Ast) -> Result<Program, Errors> {
	let mut scope = scope::Stack::default();
	let mut dict_keys = HashSet::default();
	let mut errors = Errors::default();

	let result = {
		let mut context = Context::new(&mut scope, &mut dict_keys, &mut errors);
		analyze_block(ast.statements, &mut context)
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


fn analyze_block(block: ast::Block, context: &mut Context) -> Option<Block> {
	let block = analyze_items(
		analyze_statement,
		block.0.into_vec(), // Use vec's owned iterator.
		context
	)?;

	Some(block.into())
}


fn analyze_statement(statement: ast::Statement, context: &mut Context) -> Option<Statement> {
	match statement {
		ast::Statement::Let { identifier, init, pos } => {
			let id = context.scope
				.declare(identifier, pos)
				.map_err(
					|error| context.report(error)
				)
				.ok();

			let init = analyze_expr(init, context);

			let (_, init) = id.zip(init)?;

			Some(Statement::Let { identifier, init })
		}

		ast::Statement::Assign { left, right } => {
			// TODO: Check if left is a valid lvalue.
			let left = analyze_expr(left, context);
			let right = analyze_expr(right, context);

			let (left, right) = left.zip(right)?;

			Some(Statement::Assign { left, right })
		}

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

		ast::Statement::Break { pos } => {
			if context.in_loop {
				Some(Statement::Break)
			} else {
				context.report(Error::break_outside_loop(pos));
				None
			}
		}

		ast::Statement::While { condition, block, .. } => {
			let condition = analyze_expr(condition, context);
			let block = {
				let mut context = context.enter_loop();
				analyze_block(block, &mut context)
			};

			let (condition, block) = condition.zip(block)?;

			Some(Statement::While { condition, block })
		}

		ast::Statement::For { identifier, expr, block, pos } => {
			let expr = analyze_expr(expr, context);
			let id_block = {
				let mut context = context.enter_loop();

				let id = context.scope
					.declare(identifier, pos)
					.map_err(
						|error| context.report(error)
					)
					.ok();

				let block = analyze_block(block, &mut context);

				id.zip(block)
			};

			let (expr, (_, block)) = expr.zip(id_block)?;

			Some(Statement::For { identifier, expr, block })
		}

		ast::Statement::Expr(expr) => {
			let expr = analyze_expr(expr, context)?;
			Some(Statement::Expr(expr))
		}

		ast::Statement::IllFormed => panic!("ill-formed statement in semantic analysis"),
	}
}


fn analyze_expr(expr: ast::Expr, context: &mut Context) -> Option<Expr> {
	match expr {
		ast::Expr::Self_ { pos } => {
			if context.in_function {
				Some(Expr::Self_ { pos })
			} else {
				context.report(Error::self_outside_function(pos));
				None
			}
		}

		ast::Expr::Identifier { identifier, pos } => {
			context.scope
				.resolve(identifier, pos)
				.map_err(
					|error| context.report(error)
				)
				.ok()?;

			Some(Expr::Identifier { identifier, pos })
		}

		ast::Expr::Literal { literal, pos } => {
			let literal = analyze_literal(literal, context)?;
			Some(Expr::Literal { literal, pos })
		}

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

		ast::Expr::CommandBlock { .. } => todo!(),

		ast::Expr::IllFormed => panic!("ill-formed statement in semantic analysis"),
	}
}


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


fn analyze_literal(literal: ast::Literal, context: &mut Context) -> Option<Literal> {
	match literal {
		ast::Literal::Nil => Some(Literal::Nil),

		ast::Literal::Bool(b) => Some(Literal::Bool(b)),

		ast::Literal::Int(i) => Some(Literal::Int(i)),

		ast::Literal::Float(f) => Some(Literal::Float(f)),

		ast::Literal::Byte(b) => Some(Literal::Byte(b)),

		ast::Literal::String(s) => Some(Literal::String(s)),

		ast::Literal::Array(array) => {
			let array = analyze_items(
				analyze_expr,
				array.into_vec(), // Use vec's owned iterator.
				context
			)?;

			Some(Literal::Array(array))
		}

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

			let (params, body) = params.zip(body)?;

			Some(
				Literal::Function { params, body }
			)
		}

		ast::Literal::Identifier(identifier) => Some(Literal::Identifier(identifier)),
	}
}


/// Static semantic analysis.
#[derive(Debug)]
struct Context<'a> {
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
}


impl<'a> Context<'a> {
	/// Create a new context.
	fn new(
		scope: &'a mut scope::Stack,
		dict_keys: &'a mut HashSet<Symbol>,
		errors: &'a mut Errors
	) -> Self {
		scope.enter();

		Self {
			scope,
			dict_keys,
			errors,
			in_function: false,
			in_loop: false,
		}
	}


	fn enter_block<'b>(&'b mut self) -> Context<'b> {
		self.scope.enter();

		Context {
			scope: self.scope,
			dict_keys: self.dict_keys,
			errors: self.errors,
			in_function: self.in_function,
			in_loop: self.in_loop,
		}
	}


	fn enter_loop<'b>(&'b mut self) -> Context<'b> {
		self.scope.enter();

		Context {
			scope: self.scope,
			dict_keys: self.dict_keys,
			errors: self.errors,
			in_function: self.in_function,
			in_loop: true,
		}
	}


	fn enter_function<'b>(&'b mut self) -> Context<'b> {
		self.scope.enter();

		Context {
			scope: self.scope,
			dict_keys: self.dict_keys,
			errors: self.errors,
			in_function: true,
			in_loop: false,
		}
	}


	fn report(&mut self, error: Error) {
		self.errors.0.push(error);
	}
}


impl<'a> Drop for Context<'a> {
	fn drop(&mut self) {
		self.scope.exit();
	}
}
