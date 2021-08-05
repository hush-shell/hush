// Hashmaps are used for the dict implementation, which allows mutable keys.
#![allow(clippy::mutable_key_type)]


mod command;
mod flow;
mod lib;
mod mem;
mod panic;
mod source;
pub mod value;
#[cfg(test)]
mod tests;

use std::{
	collections::HashMap,
	ops::Deref,
};

use crate::symbol;
use super::semantic::program;
use value::{
	keys,
	Array,
	CallContext,
	Dict,
	Error,
	Float,
	Function,
	HushFun,
	NativeFun,
	Str,
	Value,
};
pub use panic::Panic;
pub use source::SourcePos;
use flow::Flow;
use mem::Stack;


/// A runtime instance to execute Hush programs.
#[derive(Debug)]
pub struct Runtime<'a> {
	stack: Stack,
	arguments: Vec<Value>,
	interner: &'a mut symbol::Interner,
}


impl<'a> Runtime<'a> {
	/// Execute the given program.
	pub fn eval(
		program: &'static program::Program,
		interner: &'a mut symbol::Interner
	) -> Result<Value, Panic> {
		let mut runtime = Self {
			stack: Stack::default(),
			arguments: Vec::new(),
			interner,
		};

		// Global variables.
		let slots: mem::SlotIx = program.root_slots.into();

		runtime.stack
			.extend(slots.copy())
			.map_err(|_| Panic::stack_overflow(SourcePos::file(program.source)))?;

		// Stdlib.
		let std = lib::new();
		runtime.stack.store(mem::SlotIx(0), std);

		// Execute the program.
		let value = match runtime.eval_block(&program.statements)? {
			Flow::Regular(value) => value,
			flow => panic!("invalid flow in root state: {:#?}", flow)
		};

		// Drop global variables.
		runtime.stack.shrink(slots);

		debug_assert!(runtime.stack.is_empty());
		debug_assert!(runtime.arguments.is_empty());

		Ok(value)
	}


	/// Execute a block, returning the value of the last statement, or the corresponding
	/// control flow if returns or breaks are reached.
	fn eval_block(&mut self, block: &'static program::Block) -> Result<Flow, Panic> {
		self.eval_tail_block(block, |_| ())
	}


	/// Execute a block, returning the value of the last statement, or the corresponding
	/// control flow if returns or breaks are reached.
	fn eval_tail_block<F>(
		&mut self,
		block: &'static program::Block,
		tail_call: F,
	) -> Result<Flow, Panic>
	where
		F: FnOnce(&mut Self),
	{
		let mut iter = block.0.iter();

		let last_statement = iter.next_back();

		for statement in iter {
			match self.eval_statement(statement)? {
				Flow::Regular(_) => (),
				flow => return Ok(flow),
			}
		}

		let flow =
			if let Some(last_statement) = last_statement {
				self.eval_tail_statement(last_statement, tail_call)?
			} else {
				Flow::Regular(Value::default())
			};

		Ok(flow)
	}


	/// Execute a literal.
	/// For trivial types, this basically instatiates a corresponding value.
	/// For compound types, sub-expressions are evaluated.
	/// For function types, closed-over variables are captured, if any.
	/// For identifiers, their string is resolved.
	fn eval_literal(
		&mut self,
		literal: &'static program::Literal,
		pos: program::SourcePos
	) -> Result<Flow, Panic> {
		match literal {
			// Nil.
			program::Literal::Nil => Ok(Flow::Regular(Value::Nil)),

			// Bool.
			program::Literal::Bool(b) => Ok(Flow::Regular((*b).into())),

			// Int.
			program::Literal::Int(int) => Ok(Flow::Regular((*int).into())),

			// Float.
			program::Literal::Float(float) => Ok(Flow::Regular((*float).into())),

			// Byte.
			program::Literal::Byte(byte) => Ok(Flow::Regular((*byte).into())),

			// String.
			program::Literal::String(string) => Ok(Flow::Regular(string.as_ref().into())),

			// Array.
			program::Literal::Array(exprs) => {
				let mut array = Vec::new();

				for expr in exprs.iter() {
					match self.eval_expr(expr)?.0 {
						Flow::Regular(value) => array.push(value),
						flow => return Ok(flow),
					}
				}

				Ok(Flow::Regular(Array::new(array).into()))
			},

			// Dict.
			program::Literal::Dict(exprs) => {
				let mut dict = HashMap::new();

				for (symbol, expr) in exprs.iter() {
					let key: Value = self.interner
						.resolve(*symbol)
						.expect("unresolved symbol")
						.into();

					match self.eval_expr(expr)?.0 {
						Flow::Regular(value) => dict.insert(key, value),
						flow => return Ok(flow),
					};
				}

				Ok(Flow::Regular(Dict::new(dict).into()))
			}

			// Function.
			program::Literal::Function { params, frame_info, body } => {
				let context = frame_info
					.captures
					.iter()
					.map(
						|capture| (
							self.stack.capture(capture.from.into()),
							capture.to.into(),
						)
					)
					.collect();

				Ok(
					Flow::Regular(
						HushFun::new(*params, frame_info, body, context, pos.into()).into()
					)
				)
			},

			// Identifier.
			program::Literal::Identifier(symbol) => Ok(
				Flow::Regular(
					self.interner
						.resolve(*symbol)
						.expect("unresolved symbol")
						.into()
				)
			),
		}
	}


	/// Execute an expression.
	/// Returns a triple of (flow, expr pos, self value) or panic.
	fn eval_expr(
		&mut self,
		expr: &'static program::Expr
	) -> Result<(Flow, SourcePos, Value), Panic> {
		self.eval_tail_expr(expr, |_| ())
	}


	/// Execute an expression.
	/// Returns a triple of (flow, expr pos, self value) or panic.
	fn eval_tail_expr<F>(
		&mut self,
		expr: &'static program::Expr,
		tail_call: F,
	) -> Result<(Flow, SourcePos, Value), Panic>
	where
		F: FnOnce(&mut Self),
	{
		macro_rules! regular_expr {
			($expr: expr, $pos: expr) => {
				match self.eval_expr($expr)? {
					(Flow::Regular(value), pos, _) => (value, pos),
					(flow, _, _) => return Ok((flow, $pos, Value::default()))
				};
			}
		}

		match expr {
			// Identifier.
			program::Expr::Identifier { slot_ix, pos } => {
				let value = self.stack.fetch(slot_ix.into());
				Ok((Flow::Regular(value), pos.into(), Value::default()))
			},

			// Literal.
			program::Expr::Literal { literal, pos } => {
				let flow = self.eval_literal(literal, *pos)?;
				Ok((flow, pos.into(), Value::default()))
			},

			// UnaryOp.
			program::Expr::UnaryOp { op, operand, pos } => {
				let pos = pos.into();

				let flow = self.unary_op(op, operand)?;

				Ok((flow, pos, Value::default()))
			}

			// BinaryOp.
			program::Expr::BinaryOp { left, op, right, pos } => {
				let pos = pos.into();

				let flow = self.binary_op(left, op, right, &pos)?;

				Ok((flow, pos, Value::default()))
			}

			// If.
			program::Expr::If { condition, then, otherwise, pos } => {
				let pos = pos.into();

				let condition = match self.eval_expr(condition)? {
					(Flow::Regular(Value::Bool(b)), _, _) => b,
					(Flow::Regular(value), pos, _) => return Err(Panic::invalid_condition(value, pos)),
					(flow, _, _) => return Ok((flow, pos, Value::default()))
				};

				let value = if condition {
					self.eval_block(then)
				} else {
					self.eval_block(otherwise)
				}?;

				Ok((value, pos, Value::default()))
			}

			// Access.
			program::Expr::Access { object, field, pos } => {
				let pos = pos.into();

				let (obj, obj_pos) = regular_expr!(object, pos);
				let (field, field_pos) = regular_expr!(field, pos);

				let value = match (&obj, field) {
					(Value::Dict(ref dict), field) => dict
						.get(&field)
						.map_err(|_| Panic::index_out_of_bounds(field, field_pos)),

					(Value::Array(ref array), Value::Int(ix)) => array
						.index(ix)
						.map_err(|_| Panic::index_out_of_bounds(Value::Int(ix), field_pos)),

					(Value::Array(_), field) => Err(Panic::type_error(field, field_pos)),

					(Value::String(ref string), Value::Int(ix)) => string
						.index(ix)
						.map_err(|_| Panic::index_out_of_bounds(Value::Int(ix), field_pos)),

					(Value::String(_), field) => Err(Panic::type_error(field, field_pos)),

					(Value::Error(ref error), field) => error
						.get(&field)
						.map_err(|_| Panic::index_out_of_bounds(field, field_pos)),

					(_, _) => return Err(Panic::type_error(obj, obj_pos)),
				}?;

				Ok((Flow::Regular(value), pos, obj))
			}

			// Call.
			program::Expr::Call { function, args, pos } => {
				let pos = pos.into();

				// Eval function.
				let (function, obj) = match self.eval_expr(function)? {
					(Flow::Regular(Value::Function(ref fun)), _, obj) => (fun.copy(), obj),
					(Flow::Regular(value), pos, _) => return Err(Panic::invalid_call(value, pos)),
					(flow, _, _) => return Ok((flow, pos, Value::default())),
				};

				// Eval arguments.
				// While evaluating arguments, we may need to call other functions, so we must
				// keep track of when our arguments start.
				let args_start = self.arguments.len();

				for expr in args.iter() {
					match self.eval_expr(expr)? {
						(Flow::Regular(value), _, _) => self.arguments.push(value),
						(flow, _, _) => {
							self.arguments.truncate(args_start);
							return Ok((flow, pos, Value::default()));
						}
					}
				}

				tail_call(self);

				let value = self.call(obj, &function, args_start, pos.copy())?;

				Ok((Flow::Regular(value), pos, Value::default()))
			}

			// CommandBlock.
			program::Expr::CommandBlock { block, pos } => {
				let value = self.eval_command_block(block)?;
				Ok((Flow::Regular(value), pos.into(), Value::default()))
			}
		}
	}


	/// Execute a statement.
	fn eval_statement(&mut self, statement: &'static program::Statement) -> Result<Flow, Panic> {
		self.eval_tail_statement(statement, |_| ())
	}


	/// Execute a statement.
	fn eval_tail_statement<F>(
		&mut self,
		statement: &'static program::Statement,
		tail_call: F,
	) -> Result<Flow, Panic>
	where
		F: FnOnce(&mut Self),
	{
		match statement {
			// Assign.
			program::Statement::Assign { left, right } => {
				let value = match self.eval_expr(right)?.0 {
					Flow::Regular(value) => value,
					flow => return Ok(flow),
				};

				match left {
					program::Lvalue::Identifier { slot_ix, .. } => self.stack.store(slot_ix.into(), value),

					program::Lvalue::Access { object, field, pos } => {
						let (obj, obj_pos) = match self.eval_expr(object)? {
							(Flow::Regular(obj), pos, _) => (obj, pos),
							(flow, _, _) => return Ok(flow),
						};

						let (field, field_pos) = match self.eval_expr(field)? {
							(Flow::Regular(field), pos, _) => (field, pos),
							(flow, _, _) => return Ok(flow),
						};

						match (obj, field) {
							(Value::Dict(ref dict), field) => dict.insert(field, value),

							(Value::Array(ref array), Value::Int(ix)) if ix >= array.len() => return Err(
								Panic::index_out_of_bounds(Value::Int(ix), field_pos)
							),

							(Value::Array(ref array), Value::Int(ix)) => array
								.deref()
								.set(ix, value)
								.map_err(|_| Panic::index_out_of_bounds(Value::Int(ix), pos.into()))?,

							(Value::Array(_), field) => return Err(Panic::type_error(field, field_pos)),

							(Value::Error(_), field) => return Err(Panic::assign_to_readonly_field(field, field_pos)),

							(obj, _) => return Err(Panic::type_error(obj, obj_pos)),
						};
					}
				}

				Ok(Flow::Regular(Value::default()))
			}

			// Return.
			program::Statement::Return { expr } => {
				match self.eval_tail_expr(expr, tail_call)?.0 {
					Flow::Regular(value) => Ok(Flow::Return(value)),
					flow => Ok(flow),
				}
			}

			// Break.
			program::Statement::Break => Ok(Flow::Break),

			// While.
			program::Statement::While { condition, block } => {
				loop {
					let condition = match self.eval_expr(condition)? {
						(Flow::Regular(Value::Bool(b)), _, _) => b,
						(Flow::Regular(value), pos, _) => return Err(Panic::invalid_condition(value, pos)),
						(flow, _, _) => return Ok(flow)
					};

					if !condition {
						break;
					}

					match self.eval_block(block)? {
						Flow::Regular(_) => (),
						flow @ Flow::Return(_) => return Ok(flow),
						Flow::Break => break,
					}
				}

				Ok(Flow::Regular(Value::default()))
			}

			// For.
			program::Statement::For { slot_ix, expr, block } => {
				let slot_ix: mem::SlotIx = slot_ix.into();

				let (iter, pos) = match self.eval_expr(expr)? {
					(Flow::Regular(Value::Function(ref iter)), pos, _) => (iter.copy(), pos),
					(Flow::Regular(value), pos, _) => return Err(Panic::type_error(value, pos)),
					(flow, _, _) => return Ok(flow)
				};

				loop {
					// While evaluating arguments, we may need to call other functions, so we must
					// keep track of when our arguments start.
					let args_start = self.arguments.len();
					match self.call(Value::default(), &iter, args_start, pos.copy())? {
						Value::Dict(ref dict) => {
							let finished = keys::FINISHED.with(
								|finished| dict
									.get(finished)
									.map_err(|_| Panic::index_out_of_bounds(finished.copy(), pos.copy()))
							)?;

							match finished {
								Value::Bool(false) => {
									let value = keys::VALUE.with(
										|value| dict
											.get(value)
											.map_err(|_| Panic::index_out_of_bounds(value.copy(), pos.copy()))
									)?;

									self.stack.store(slot_ix.copy(), value);
								},

								Value::Bool(true) => break,

								other => return Err(Panic::type_error(other, pos))
							}

							Value::Nil
						},

						other => return Err(Panic::type_error(other, pos)),
					};

					match self.eval_block(block)? {
						Flow::Regular(_) => (),
						flow @ Flow::Return(_) => return Ok(flow),
						Flow::Break => break,
					}
				}

				Ok(Flow::Regular(Value::default()))
			}

			// Expr.
			program::Statement::Expr(expr) => self
				.eval_tail_expr(expr, tail_call)
				.map(|(flow, _, _)| flow)
		}
	}


	/// Call the given function.
	/// The arguments are expected to be on the self.arguments vector.
	fn call(
		&mut self,
		obj: Value,
		function: &Function,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args_count = self.arguments.len() as u32;

		let value = match function {
			Function::Hush(HushFun { params, frame_info, body, context, .. }) => {
				// Make sure we clean the arguments vector even when early returning.
				let arguments = self.arguments.drain(args_start..);

				if args_count != *params {
					return Err(Panic::invalid_args(args_count, *params, pos));
				}

				let slots: mem::SlotIx = frame_info.slots.into();
				self.stack.extend(slots.copy())
					.map_err(|_| Panic::stack_overflow(pos))?;

				// Place arguments
				for (ix, value) in arguments.enumerate() {
					self.stack.store(mem::SlotIx(ix as u32), value);
				}

				// Place captured variables.
				for (value, slot_ix) in context.iter().cloned() {
					self.stack.place(slot_ix, value);
				}

				// Place self.
				if let Some(slot_ix) = frame_info.self_slot {
					self.stack.store(slot_ix.into(), obj);
				}

				let mut shrinked = false;

				let flow = self.eval_tail_block(
					body,
					|runtime| { // Shrink stack before tail calling.
						runtime.stack.shrink(slots.copy());
						shrinked = true;
					}
				)?;

				let value = match flow {
					Flow::Regular(value) => value,
					Flow::Return(value) => value,
					Flow::Break => panic!("break outside loop"),
				};

				if !shrinked { // Only shrink the stack if there was no tail call.
					self.stack.shrink(slots);
				}

				value
			}

			Function::Rust(fun) => {
				let result = fun.call(
					CallContext {
						runtime: self,
						obj,
						args_start,
						pos
					}
				);

				self.arguments.truncate(args_start);

				result?
			}
		};

		Ok(value)
	}


	/// Execute a unary operator expression.
	fn unary_op(
		&mut self,
		op: &'static program::UnaryOp,
		operand: &'static program::Expr,
	) -> Result<Flow, Panic> {
		use program::UnaryOp::{Minus, Not};

		let (value, operand_pos) = match self.eval_expr(operand)? {
			(Flow::Regular(value), pos, _) => (value, pos),
			(flow, _, _) => return Ok(flow),
		};

		let value = match (op, value) {
			(Minus, Value::Float(ref f)) => Ok((-f).into()),
			(Minus, Value::Int(i)) => Ok((-i).into()),
			(Minus, value) => Err(Panic::type_error(value, operand_pos)),

			(Not, Value::Bool(b)) => Ok((!b).into()),
			(Not, value) => Err(Panic::type_error(value, operand_pos)),
		}?;

		Ok(Flow::Regular(value))
	}


	/// Execute a binary operator expression.
	fn binary_op(
		&mut self,
		left: &'static program::Expr,
		op: &'static program::BinaryOp,
		right: &'static program::Expr,
		pos: &SourcePos,
	) -> Result<Flow, Panic> {
		use program::BinaryOp::*;

		macro_rules! regular_expr {
			($expr: expr) => {
				match self.eval_expr($expr)? {
					(Flow::Regular(value), pos, _) => (value, pos),
					(flow, _, _) => return Ok(flow)
				};
			}
		}

		let (left, left_pos) = regular_expr!(left);

		let value = match op {
			And | Or => match (left, op) {
				(Value::Bool(false), And) => Value::Bool(false),
				(Value::Bool(true), Or) => Value::Bool(true),

				(Value::Bool(_), _) => {
					let (right, right_pos) = regular_expr!(right);
					match right {
						right @ Value::Bool(_) => right,
						right => return Err(Panic::type_error(right, right_pos)),
					}
				}

				(left, _) => return Err(Panic::type_error(left, left_pos)),
			}

			Plus | Minus | Times | Div | Mod => {
				let (right, right_pos) = regular_expr!(right);

				self.arithmetic_op(left, left_pos, op, &pos, right, right_pos)?
			}

			Greater | GreaterEquals | Lower | LowerEquals => {
				let (right, right_pos) = regular_expr!(right);

				self.ord_op(left, left_pos, op, right, right_pos)?
			}

			Equals => Value::Bool(left == regular_expr!(right).0),
			NotEquals => Value::Bool(left != regular_expr!(right).0),

			Concat => {
				let (right, right_pos) = regular_expr!(right);

				match (left, right) {
					(Value::String(ref str1), Value::String(ref str2)) => {
						let string =
							[
								AsRef::<[u8]>::as_ref(str1),
								AsRef::<[u8]>::as_ref(str2),
							]
							.concat::<u8>();

						string.into_boxed_slice().into()
					}

					(Value::String(_), right) => return Err(Panic::type_error(right, right_pos)),
					(left, _) => return Err(Panic::type_error(left, left_pos)),
				}
			}
		};

		Ok(Flow::Regular(value))
	}


	/// Execute a binary arithmetic operator expression.
	/// Panics if op is not arithmetic (+, -, *, /, %).
	fn arithmetic_op(
		&mut self,
		left: Value,
		left_pos: SourcePos,
		op: &'static program::BinaryOp,
		pos: &SourcePos,
		right: Value,
		right_pos: SourcePos,
	) -> Result<Value, Panic> {
		use program::BinaryOp::*;
		use std::ops::{Add, Sub, Mul, Div, Rem};

		macro_rules! arith_operator {
			($op_float: expr, $op_int: ident, $err_int: expr) => {
				match (left, right) {
					// int . int
					(Value::Int(int1), Value::Int(int2)) => {
						let val = int1.$op_int(int2).ok_or($err_int)?;
						Ok(Value::Int(val))
					},

					// float . float
					(Value::Float(ref float1), Value::Float(ref float2)) => {
						let val = $op_float(float1.copy(), float2.copy());
						Ok(Value::Float(val))
					},

					// float . int, int . float
					(Value::Int(int), Value::Float(ref float)) => {
						let val = $op_float(Float::from(int), float.copy());
						Ok(Value::Float(val))
					},

					(Value::Float(ref float), Value::Int(int)) => {
						let val = $op_float(float.copy(), int.into());
						Ok(Value::Float(val))
					},

					// ? . ?
					(left, right) => Err(
						if matches!(left, Value::Int(_) | Value::Float(_)) {
							Panic::type_error(right, right_pos)
						} else {
							Panic::type_error(left, left_pos)
						}
					),
				}
			}
		}

		match op {
			Plus => arith_operator!(
				Add::add,
				checked_add,
				Panic::integer_overflow(pos.copy())
			),

			Minus => arith_operator!(
				Sub::sub,
				checked_sub,
				Panic::integer_overflow(pos.copy())
			),

			Times => arith_operator!(
				Mul::mul,
				checked_mul,
				Panic::integer_overflow(pos.copy())
			),

			Div => arith_operator!(
				Div::div,
				checked_div,
				Panic::division_by_zero(pos.copy()) // TODO: this can be caused by overflow too.
			),

			Mod => arith_operator!(
				Rem::rem,
				checked_rem,
				Panic::division_by_zero(pos.copy()) // TODO: this can be caused by overflow too.
			),

			_ => unreachable!("operator is not arithmetic"),
		}
	}


	/// Execute a binary ord operator expression.
	/// Panics if op is not ord (<, <=, >, >=).
	fn ord_op(
		&mut self,
		left: Value,
		left_pos: SourcePos,
		op: &'static program::BinaryOp,
		right: Value,
		right_pos: SourcePos,
	) -> Result<Value, Panic> {
		use program::BinaryOp::*;
		use std::cmp::Ordering;

		let ord_operator = |order: fn(Ordering) -> bool| {
			match (left, right) {
				// int . int
				(Value::Int(int1), Value::Int(int2)) => Ok(
					Value::Bool(
						order(int1.cmp(&int2))
					)
				),

				// float . float
				(Value::Float(ref float1), Value::Float(ref float2)) => Ok(
					Value::Bool(
						order(float1.cmp(float2))
					)
				),

				// float . int, int . float
				(Value::Int(int), Value::Float(ref float)) => Ok(
					Value::Bool(
						order(float.cmp(&int.into()))
					)
				),

				(Value::Float(ref float), Value::Int(int)) => Ok(
					Value::Bool(
						order(Float::from(int).cmp(float))
					)
				),

				// char . char
				(Value::Byte(b1), Value::Byte(b2)) => Ok(
					Value::Bool(
						order(b1.cmp(&b2))
					)
				),

				// string . string
				(Value::String(ref str1), Value::String(ref str2)) => Ok(
					Value::Bool(
						order(str1.cmp(&str2))
					)
				),

				// ? + ?
				(left, right) => Err(
					if matches!(left, Value::Int(_) | Value::Float(_) | Value::Byte(_) | Value::String(_)) {
						Panic::type_error(right, right_pos)
					} else {
						Panic::type_error(left, left_pos)
					}
				),
			}
		};

		match op {
			Lower => ord_operator(|ordering| ordering == Ordering::Less),
			LowerEquals => ord_operator(|ordering| ordering != Ordering::Greater),
			Greater => ord_operator(|ordering| ordering == Ordering::Greater),
			GreaterEquals => ord_operator(|ordering| ordering != Ordering::Less),
			_ => unreachable!("operator is not ord"),
		}
	}
}
