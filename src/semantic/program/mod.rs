pub mod command;
pub mod fmt;
pub mod mem;

use std::path::Path;

use super::{ast, lexer};
pub use crate::{
	syntax::SourcePos,
	symbol::Symbol,
};
pub use command::{
	ArgPart,
	ArgUnit,
	Argument,
	BasicCommand,
	Command,
	CommandBlock,
	CommandBlockKind,
	Redirection,
	RedirectionTarget,
};


/// A block is a list of statements.
#[derive(Debug, Default)]
pub struct Block(pub Box<[Statement]>);


impl From<Box<[Statement]>> for Block {
	fn from(block: Box<[Statement]>) -> Self {
		Self(block)
	}
}


/// Literals of all types in the language.
/// Note that there are no literals for the error type.
#[derive(Debug)]
pub enum Literal {
	Nil,
	Bool(bool),
	Int(i64),
	Float(f64),
	Byte(u8),
	String(Box<[u8]>),
	Array(Box<[Expr]>),
	Dict(Box<[(Symbol, Expr)]>),
	Function {
		/// The number of parameters.
		params: u32,
		frame_info: mem::FrameInfo,
		body: Block,
	},
	/// For the dot access operator, we want to be able to have identifiers as literal
	/// strings instead of names for variables. This variant should only be used in such
	/// case.
	Identifier(Symbol),
}


/// Unary operators.
#[derive(Debug)]
pub enum UnaryOp {
	Minus, // -
	Not,   // not
}


impl From<ast::UnaryOp> for UnaryOp {
	fn from(op: ast::UnaryOp) -> Self {
		match op {
			ast::UnaryOp::Minus => UnaryOp::Minus,
			ast::UnaryOp::Not => UnaryOp::Not,
		}
	}
}


/// Binary operators.
/// Assignment/Access are not represented as operators, but directly as
/// statements/expressions instead.
#[derive(Debug)]
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


impl BinaryOp {
	/// Check if the operator is arithmetic (+, -, *, /, %).
	pub fn is_arithmetic(&self) -> bool {
		matches!(
			self,
			Self::Plus | Self::Minus | Self::Times | Self::Div | Self::Mod
		)
	}


	/// Check if the operator is relational (==, !=, >, >=, <, <=).
	pub fn is_relational(&self) -> bool {
		matches!(
			self,
			Self::Equals | Self::NotEquals
				| Self::Greater | Self::GreaterEquals
				| Self::Lower | Self::LowerEquals
		)
	}


	/// Check if the operator is boolean (and, or).
	pub fn is_boolean(&self) -> bool {
		matches!(self, Self::And | Self::Or)
	}
}


impl From<ast::BinaryOp> for BinaryOp {
	fn from(op: ast::BinaryOp) -> Self {
		match op {
			ast::BinaryOp::Plus => BinaryOp::Plus,
			ast::BinaryOp::Minus => BinaryOp::Minus,
			ast::BinaryOp::Times => BinaryOp::Times,
			ast::BinaryOp::Div => BinaryOp::Div,
			ast::BinaryOp::Mod => BinaryOp::Mod,
			ast::BinaryOp::Equals => BinaryOp::Equals,
			ast::BinaryOp::NotEquals => BinaryOp::NotEquals,
			ast::BinaryOp::Greater => BinaryOp::Greater,
			ast::BinaryOp::GreaterEquals => BinaryOp::GreaterEquals,
			ast::BinaryOp::Lower => BinaryOp::Lower,
			ast::BinaryOp::LowerEquals => BinaryOp::LowerEquals,
			ast::BinaryOp::And => BinaryOp::And,
			ast::BinaryOp::Or => BinaryOp::Or,
			ast::BinaryOp::Concat => BinaryOp::Concat,
		}
	}
}


/// Expressions of all kinds in the language, except for l-values.
#[derive(Debug)]
pub enum Expr {
	Identifier {
		/// Frame index of the local variable.
		/// Closures are inserted on the frame on function call.
		slot_ix: mem::SlotIx,
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
	/// Function call (()) operator.
	Call {
		function: Box<Expr>,
		args: Box<[Expr]>,
		pos: SourcePos,
	},
	CommandBlock {
		block: CommandBlock,
		pos: SourcePos,
	},
}


/// L-value expressions.
#[derive(Debug)]
pub enum Lvalue {
	Identifier {
		/// Frame index of the local variable.
		/// Closures are inserted on the frame on function call.
		slot_ix: mem::SlotIx,
		pos: SourcePos,
	},
	/// Field access ([]) operator.
	Access {
		object: Box<Expr>,
		field: Box<Expr>,
		pos: SourcePos,
	},
}


/// Statements of all kinds in the language.
#[derive(Debug)]
pub enum Statement {
	Assign {
		left: Lvalue,
		right: Expr,
	},
	Return {
		expr: Expr,
	},
	Break,
	/// While loop.
	While {
		condition: Expr,
		block: Block,
	},
	/// For loop. Also introduces an identifier.
	For {
		slot_ix: mem::SlotIx,
		expr: Expr,
		block: Block,
	},
	Expr(Expr),
}


/// A statically correct (syntactically and semantically) Hush program.
#[derive(Debug)]
pub struct Program {
	/// The source path. May be something fictional, like "<stdin>".
	pub source: Box<Path>,
	/// The program.
	pub statements: Block,
	/// How many slots in the root scope.
	pub root_slots: mem::SlotIx,
}
