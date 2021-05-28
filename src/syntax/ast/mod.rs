mod command;
mod debug;

use std::{collections::HashMap, path::Path};

use super::{lexer, SourcePos};
pub use crate::symbol::Symbol;
use command::{Command, CommandBlockKind};


/// A block is a list of statements, constituting a new scope.
#[derive(Default)]
pub struct Block(Box<[Statement]>);


impl From<Box<[Statement]>> for Block {
	fn from(block: Box<[Statement]>) -> Self {
		Self(block)
	}
}


/// Literals of all types in the language.
/// Note that there are no literals for the error type.
pub enum Literal {
	Nil,
	Bool(bool),
	Int(i64),
	Float(f64),
	Byte(u8),
	String(Box<[u8]>),
	Array(Box<[Expr]>),
	Dict(HashMap<Symbol, Expr>),
	Function {
		/// A list of arguments (identifiers).
		args: Box<[Symbol]>,
		body: Block,
	},
}


impl From<lexer::Literal> for Literal {
	fn from(op: lexer::Literal) -> Self {
		match op {
			lexer::Literal::Nil => Literal::Nil,
			lexer::Literal::True => Literal::Bool(true),
			lexer::Literal::False => Literal::Bool(false),
			lexer::Literal::Int(int) => Literal::Int(int),
			lexer::Literal::Float(float) => Literal::Float(float),
			lexer::Literal::Byte(byte) => Literal::Byte(byte),
			lexer::Literal::String(string) => Literal::String(string),
		}
	}
}


/// Unary operators.
pub enum UnaryOp {
	Minus, // -
	Not,   // not
}


/// Warning, the following instance may panic if used with unmapped operators.
impl From<lexer::Operator> for UnaryOp {
	fn from(op: lexer::Operator) -> Self {
		match op {
			lexer::Operator::Minus => UnaryOp::Minus,
			lexer::Operator::Not => UnaryOp::Not,
			_ => panic!("invalid operator"),
		}
	}
}


/// Binary operators.
/// Assignment/Access are not represented as operators, but directly as
/// statements/expressions instead.
pub enum BinaryOp {
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

	And, // and
	Or,  // or

	Concat, // ++
}


/// Warning, the following instance may panic if used with unmapped operators.
impl From<lexer::Operator> for BinaryOp {
	fn from(op: lexer::Operator) -> Self {
		match op {
			lexer::Operator::Plus => BinaryOp::Plus,
			lexer::Operator::Minus => BinaryOp::Minus,
			lexer::Operator::Times => BinaryOp::Times,
			lexer::Operator::Div => BinaryOp::Div,
			lexer::Operator::Mod => BinaryOp::Mod,
			lexer::Operator::Equals => BinaryOp::Equals,
			lexer::Operator::NotEquals => BinaryOp::NotEquals,
			lexer::Operator::Greater => BinaryOp::Greater,
			lexer::Operator::GreaterEquals => BinaryOp::GreaterEquals,
			lexer::Operator::Lower => BinaryOp::Lower,
			lexer::Operator::LowerEquals => BinaryOp::LowerEquals,
			lexer::Operator::And => BinaryOp::And,
			lexer::Operator::Or => BinaryOp::Or,
			lexer::Operator::Concat => BinaryOp::Concat,
			_ => panic!("invalid operator"),
		}
	}
}


/// Expressions of all kinds in the language.
pub enum Expr {
	/// The `self` keyword.
	Self_ {
		pos: SourcePos,
	},
	Identifier {
		identifier: Symbol,
		pos: SourcePos,
	},
	Literal {
		literal: Literal,
		pos: SourcePos,
	},
	UnaryOp {
		op: UnaryOp,
		operand: Box<Expr>,
		pos: SourcePos,
	},
	BinaryOp {
		left: Box<Expr>,
		op: BinaryOp,
		right: Box<Expr>,
		pos: SourcePos,
	},
	/// If-else expression.
	If {
		condition: Box<Expr>,
		then: Block,
		otherwise: Block,
		pos: SourcePos,
	},
	/// Field access ([]) operator.
	Access {
		object: Box<Expr>,
		field: Box<Expr>,
		pos: SourcePos,
	},
	FunctionCall {
		function: Box<Expr>,
		params: Box<[Expr]>,
		pos: SourcePos,
	},
	CommandBlock {
		kind: CommandBlockKind,
		commands: Box<[Command]>,
		pos: SourcePos,
	},
}


/// Statements of all kinds in the language.
pub enum Statement {
	/// Introduces an identifier.
	Let {
		identifier: Symbol,
		pos: SourcePos,
	},
	Assign {
		left: Expr,
		right: Expr,
		pos: SourcePos,
	},
	Return {
		expr: Expr,
		pos: SourcePos,
	},
	Break {
		pos: SourcePos,
	},
	/// While loop.
	While {
		condition: Expr,
		block: Block,
		pos: SourcePos,
	},
	/// For loop. Also introduces an identifier.
	For {
		identifier: Symbol,
		expr: Expr,
		block: Block,
		pos: SourcePos,
	},
	Expr(Expr),
}


/// The abstract syntax tree for a source file.
pub struct Ast {
	/// The source path. May be something fictional, like "<stdin>".
	pub path: Box<Path>,
	/// The program.
	pub statements: Block,
}
