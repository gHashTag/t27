//! AST Expression nodes
//!
//! Defines all expression types in the Trinity language.
//! Part of the modular compiler architecture (Ring-018).

use serde::{Deserialize, Serialize};

/// Expression in the Trinity language
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// Literal value: integer, float, string, trit
    Literal(Literal),

    /// Variable reference
    Var(String),

    /// Binary operation: lhs op rhs
    BinOp {
        lhs: Box<Expr>,
        op: BinOp,
        rhs: Box<Expr>,
    },

    /// Unary operation: op expr
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    /// Function call: func(arg1, arg2, ...)
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },

    /// Field access: expr.field
    Field {
        expr: Box<Expr>,
        field: String,
    },

    /// Index access: expr[index]
    Index {
        expr: Box<Expr>,
        index: Box<Expr>,
    },

    /// Block: { stmt1; stmt2; expr }
    Block(Vec<Stmt>, Option<Box<Expr>>),

    /// If expression: if cond { then } else { else }
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },

    /// Loop expression: loop { body }
    Loop(Box<Expr>),

    /// Break expression: break [value]
    Break(Option<Box<Expr>>),

    /// Match expression: match expr { pat => expr, ... }
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },

    /// Tuple expression: (expr1, expr2, ...)
    Tuple(Vec<Expr>),

    /// List literal: [elem1, elem2, ...]
    List(Vec<Expr>),

    /// Struct literal: Type { field: value, ... }
    Struct {
        type_name: String,
        fields: Vec<(String, Expr)>,
    },
}

/// Literal values
// Temporarily disable derive to fix chrono::format::Item::Literal conflict
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Trit literal: -1, 0, +1
    Trit(TritValue),

    /// Integer literal
    Integer(i64),

    /// Float literal (f32 or f64)
    Float(f64),

    /// String literal
    String(String),

    /// Boolean literal
    Bool(bool),

    /// PHI constant literal
    Phi,
}

/// Trit value (ternary logic)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TritValue {
    Neg = -1,
    Zero = 0,
    Pos = 1,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical (ternary/Kleene)
    And,
    Or,
    Xor,
    Implies,

    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    // Arithmetic
    Neg,
    Not,

    // Ternary-specific
    TritNeg,
}

/// Statement (for blocks)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    /// Let binding: let name[: type] [= expr];
    Let {
        name: String,
        type_annotation: Option<Type>,
        value: Option<Expr>,
    },

    /// Assignment: expr = value;
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },

    /// Expression statement: expr;
    Expr(Expr),

    /// Return statement: return [expr];
    Return(Option<Box<Expr>>),
}

/// Type annotation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    /// Primitive types
    Primitive(PrimitiveType),

    /// Tuple type: (T1, T2, ...)
    Tuple(Vec<Type>),

    /// List type: [T]
    List(Box<Type>),

    /// Function type: (T1, T2) -> T
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },

    /// Struct type: Name { fields... }
    Struct {
        name: String,
        fields: Vec<(String, Type)>,
    },

    /// Type variable or reference
    Var(String),
}

/// Primitive types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveType {
    // Trit types
    Trit,
    Trit3,

    // Integer types
    I8, I16, I32, I64,
    U8, U16, U32, U64,

    // Float types
    F32, F64,

    // GoldenFloat family
    GF4, GF8, GF16, GF32, GF64,

    // TF3 (ternary float)
    TF3,

    // Boolean
    Bool,

    // String
    String,

    // PHI ratio
    Phi,

    // Any/type erasure
    Any,
}

/// Match arm in match expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

/// Pattern (for match, let, function params)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    /// Wildcard: _
    Wildcard,

    /// Literal pattern
    Literal(Literal),

    /// Variable binding: name
    Var(String),

    /// Tuple pattern: (pat1, pat2, ...)
    Tuple(Vec<Pattern>),

    /// Struct pattern: Type { field: pat, ... }
    Struct {
        type_name: String,
        fields: Vec<(String, Pattern)>,
    },

    /// Or pattern: pat1 | pat2
    Or(Box<Pattern>, Box<Pattern>),
}

impl Expr {
    /// Create a literal expression
    pub fn lit(lit: Literal) -> Self {
        Expr::Literal(lit)
    }

    /// Create a variable reference
    pub fn var(name: impl Into<String>) -> Self {
        Expr::Var(name.into())
    }

    /// Create a binary operation
    pub fn binop(lhs: Expr, op: BinOp, rhs: Expr) -> Self {
        Expr::BinOp {
            lhs: Box::new(lhs),
            op,
            rhs: Box::new(rhs),
        }
    }
}

impl std::fmt::Display for TritValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TritValue::Neg => write!(f, "-1"),
            TritValue::Zero => write!(f, "0"),
            TritValue::Pos => write!(f, "+1"),
        }
    }
}
