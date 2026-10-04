//! t27b IR: a typed, name-resolved, desugared tree.
//!
//! Produced by `lower` from the t27c AST, consumed by `eval` (the reference
//! interpreter) and by `codegen` (AArch64). Every operation that can trap
//! carries a `site`, an index into `Program::sites`, so the interpreter and
//! the generated code can be compared trap-for-trap, not only value-for-value.

/// Integer and boolean types of the supported subset. `usize` lowers to `U64`,
/// `isize` to `I64`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Ty {
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    /// A 64-bit address into a frame slot or the read-only data image (memory
    /// lane). Never a value of the language: lowering makes it for aggregates,
    /// passes it as a hidden parameter, and only ever offsets and loads it.
    Ptr,
}

impl Ty {
    pub const INTS: [Ty; 8] = [
        Ty::U8,
        Ty::U16,
        Ty::U32,
        Ty::U64,
        Ty::I8,
        Ty::I16,
        Ty::I32,
        Ty::I64,
    ];

    pub fn from_name(s: &str) -> Option<Ty> {
        Some(match s {
            "bool" => Ty::Bool,
            "u8" => Ty::U8,
            "u16" => Ty::U16,
            "u32" => Ty::U32,
            "u64" | "usize" => Ty::U64,
            "i8" => Ty::I8,
            "i16" => Ty::I16,
            "i32" => Ty::I32,
            "i64" | "isize" => Ty::I64,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Ty::Bool => "bool",
            Ty::U8 => "u8",
            Ty::U16 => "u16",
            Ty::U32 => "u32",
            Ty::U64 => "u64",
            Ty::I8 => "i8",
            Ty::I16 => "i16",
            Ty::I32 => "i32",
            Ty::I64 => "i64",
            Ty::Ptr => "ptr",
        }
    }

    pub fn is_int(self) -> bool {
        !matches!(self, Ty::Bool | Ty::Ptr)
    }

    /// Size in memory: a load or store of this type moves this many bytes.
    pub fn bytes(self) -> u32 {
        match self {
            Ty::Bool => 1,
            _ => self.bits() / 8,
        }
    }

    /// Width in bits. `Bool` reports 8 (it is stored as a 0/1 byte value).
    pub fn bits(self) -> u32 {
        match self {
            Ty::Bool | Ty::U8 | Ty::I8 => 8,
            Ty::U16 | Ty::I16 => 16,
            Ty::U32 | Ty::I32 => 32,
            Ty::U64 | Ty::I64 | Ty::Ptr => 64,
        }
    }

    pub fn signed(self) -> bool {
        matches!(self, Ty::I8 | Ty::I16 | Ty::I32 | Ty::I64)
    }

    /// True when the value occupies a full 64-bit X register.
    pub fn is64(self) -> bool {
        self.bits() == 64
    }

    pub fn min(self) -> i128 {
        match self {
            Ty::Bool => 0,
            _ if self.signed() => -(1i128 << (self.bits() - 1)),
            _ => 0,
        }
    }

    pub fn max(self) -> i128 {
        match self {
            Ty::Bool => 1,
            _ if self.signed() => (1i128 << (self.bits() - 1)) - 1,
            _ => (1i128 << self.bits()) - 1,
        }
    }

    pub fn fits(self, v: i128) -> bool {
        v >= self.min() && v <= self.max()
    }

    /// Two's-complement reduction of an arbitrary integer to this type's range.
    pub fn wrap(self, v: i128) -> i128 {
        let bits = self.bits();
        let m = (v as u128) & ((1u128 << bits) - 1);
        if self.signed() && (m >> (bits - 1)) & 1 == 1 {
            (m as i128) - (1i128 << bits)
        } else {
            m as i128
        }
    }

    /// Interpret the low `bits` of a raw 64-bit register value as this type.
    pub fn from_raw(self, raw: u64) -> i128 {
        if self == Ty::Bool {
            return (raw & 0xff) as i128;
        }
        self.wrap(raw as i128)
    }

    /// `self` can hold every value of `from`: the lossless implicit widening
    /// the subset allows (Zig's rule: same signedness and not narrower, or
    /// unsigned into a strictly wider signed type).
    pub fn can_widen_from(self, from: Ty) -> bool {
        if self == from {
            return true;
        }
        if !self.is_int() || !from.is_int() {
            return false;
        }
        if self.signed() == from.signed() {
            self.bits() >= from.bits()
        } else {
            self.signed() && self.bits() > from.bits()
        }
    }
}

/// How the plain `+ - * /` and the shifts behave on overflow. The language
/// default is `Trap`; `Wrap` exists for C-compatible differential testing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OverflowMode {
    Trap,
    Wrap,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ArithOp {
    /// Checked: traps when the exact result does not fit.
    Add,
    Sub,
    Mul,
    /// Two's-complement wrapping (`+%`, `-%`, `*%`, or plain ops in Wrap mode).
    AddW,
    SubW,
    MulW,
    /// Truncating division. Traps on a zero divisor and on MIN / -1.
    Div,
    /// Truncating division; zero divisor traps, MIN / -1 wraps to MIN.
    DivW,
    /// Truncating remainder (sign of the dividend). Zero divisor traps;
    /// MIN % -1 is 0.
    Rem,
    And,
    Or,
    Xor,
    /// Shifts whose amount must lie in 0..bits, else trap.
    Shl,
    Shr,
    /// Shifts whose amount is reduced modulo the width.
    ShlW,
    ShrW,
}

impl ArithOp {
    pub fn symbol(self) -> &'static str {
        match self {
            ArithOp::Add => "+",
            ArithOp::Sub => "-",
            ArithOp::Mul => "*",
            ArithOp::AddW => "+%",
            ArithOp::SubW => "-%",
            ArithOp::MulW => "*%",
            ArithOp::Div | ArithOp::DivW => "/",
            ArithOp::Rem => "%",
            ArithOp::And => "&",
            ArithOp::Or => "|",
            ArithOp::Xor => "^",
            ArithOp::Shl | ArithOp::ShlW => "<<",
            ArithOp::Shr | ArithOp::ShrW => ">>",
        }
    }

    pub fn is_shift(self) -> bool {
        matches!(self, ArithOp::Shl | ArithOp::Shr | ArithOp::ShlW | ArithOp::ShrW)
    }

    pub fn commutative(self) -> bool {
        matches!(
            self,
            ArithOp::Add
                | ArithOp::Mul
                | ArithOp::AddW
                | ArithOp::MulW
                | ArithOp::And
                | ArithOp::Or
                | ArithOp::Xor
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl CmpOp {
    pub fn symbol(self) -> &'static str {
        match self {
            CmpOp::Eq => "==",
            CmpOp::Ne => "!=",
            CmpOp::Lt => "<",
            CmpOp::Le => "<=",
            CmpOp::Gt => ">",
            CmpOp::Ge => ">=",
        }
    }

    pub fn holds(self, a: i128, b: i128) -> bool {
        match self {
            CmpOp::Eq => a == b,
            CmpOp::Ne => a != b,
            CmpOp::Lt => a < b,
            CmpOp::Le => a <= b,
            CmpOp::Gt => a > b,
            CmpOp::Ge => a >= b,
        }
    }

    pub fn negate(self) -> CmpOp {
        match self {
            CmpOp::Eq => CmpOp::Ne,
            CmpOp::Ne => CmpOp::Eq,
            CmpOp::Lt => CmpOp::Ge,
            CmpOp::Le => CmpOp::Gt,
            CmpOp::Gt => CmpOp::Le,
            CmpOp::Ge => CmpOp::Lt,
        }
    }

    /// The comparison with operands exchanged: `a op b` == `b op.swap() a`.
    pub fn swap(self) -> CmpOp {
        match self {
            CmpOp::Eq => CmpOp::Eq,
            CmpOp::Ne => CmpOp::Ne,
            CmpOp::Lt => CmpOp::Gt,
            CmpOp::Le => CmpOp::Ge,
            CmpOp::Gt => CmpOp::Lt,
            CmpOp::Ge => CmpOp::Le,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TrapKind {
    Overflow = 1,
    DivZero = 2,
    ShiftRange = 3,
    Assert = 4,
    AssertEq = 5,
    NoReturn = 6,
    /// `x as T` whose value is outside `T` (Zig's checked `@intCast`).
    Cast = 7,
    // Memory lane: numbered from 16 so the scalar lane can add kinds below.
    /// An index at or past the length of an array, slice or string.
    Bounds = 16,
}

impl TrapKind {
    pub fn describe(self) -> &'static str {
        match self {
            TrapKind::Overflow => "integer overflow",
            TrapKind::DivZero => "division by zero",
            TrapKind::ShiftRange => "shift amount out of range",
            TrapKind::Assert => "assert failed",
            TrapKind::AssertEq => "assert_eq failed",
            TrapKind::NoReturn => "reached the end of a non-void fn without return",
            TrapKind::Cast => "integer cast out of range",
            TrapKind::Bounds => "index out of bounds",
        }
    }
}

/// One place in the program that can trap.
#[derive(Clone, Debug)]
pub struct Site {
    pub kind: TrapKind,
    pub line: u32,
    /// Short description, e.g. the operator and its type.
    pub what: String,
    /// For AssertEq: the type of the compared values (to print them).
    pub ty: Ty,
}

pub type VarId = u32;
pub type FuncId = u32;
pub type SiteId = u32;

#[derive(Clone, Debug)]
pub struct Var {
    pub name: String,
    pub ty: Ty,
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub ty: Ty,
    pub kind: ExprKind,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    /// A value inside `ty`'s range (bool: 0 or 1).
    Const(i128),
    Var(VarId),
    /// `lhs` has type `ty`. `rhs` has type `ty` too, except for shifts, whose
    /// amount keeps its own integer type. `site` is 0 for ops that cannot trap.
    Arith {
        op: ArithOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        site: SiteId,
    },
    /// `ty` is Bool; both operands share one type (an integer type, or Bool
    /// for `==` and `!=`).
    Cmp {
        op: CmpOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    BitNot(Box<Expr>),
    Call {
        func: FuncId,
        args: Vec<Expr>,
    },
    /// Lossless conversion from the operand's narrower integer type to `ty`
    /// (or from bool, as 0 / 1).
    Widen(Box<Expr>),
    /// `x as T` that can lose information: the operand (an integer type) is
    /// converted to integer `ty`. `site` 0 truncates (two's complement, Zig's
    /// `@truncate`); otherwise a value outside `ty` traps at `site` (Zig's
    /// checked `@intCast`).
    Cast {
        arg: Box<Expr>,
        site: SiteId,
    },

    // ---- Memory lane: addresses, loads and bounds. Every address is a
    // `Ty::Ptr` value that points into a frame slot or into `Program::data`.
    /// Address of frame slot `k` of the enclosing function (`Func::slots`).
    Slot(u32),
    /// Address of read-only blob `k` (`Program::data`).
    Data(u32),
    /// Load a `ty` (a scalar, `ty.bytes()` wide) from `addr + off`.
    Load { addr: Box<Expr>, off: u32 },
    /// `base + idx * scale`: `ty` and `base` are Ptr, `idx` is U64. Cannot
    /// trap; an index reaching it is already bounds-checked or constant.
    Offset {
        base: Box<Expr>,
        idx: Box<Expr>,
        scale: u32,
    },
    /// `idx` when `idx < len` (unsigned, both U64), else trap at `site`.
    Bounds {
        idx: Box<Expr>,
        len: Box<Expr>,
        site: SiteId,
    },
    /// Run `stmts` -- only `Store`, `Copy` and `Eval` -- then evaluate
    /// `value`. How an aggregate temporary (a struct literal passed as an
    /// argument) is built at the exact point the expression is evaluated.
    Seq { stmts: Vec<Stmt>, value: Box<Expr> },
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Assign {
        var: VarId,
        value: Expr,
    },
    If {
        cond: Expr,
        then: Vec<Stmt>,
        els: Vec<Stmt>,
    },
    /// `while (cond) : (step) { body }`; `continue` runs `step`.
    While {
        cond: Expr,
        body: Vec<Stmt>,
        step: Vec<Stmt>,
    },
    Break,
    Continue,
    Return(Option<Expr>),
    /// A call evaluated for effect (its value, if any, is discarded).
    Eval(Expr),
    Assert {
        cond: Expr,
        site: SiteId,
    },
    AssertEq {
        lhs: Expr,
        rhs: Expr,
        site: SiteId,
    },

    // ---- Memory lane. `addr` is evaluated first, then the other operand.
    /// Store `value` (`value.ty.bytes()` wide) at `addr + off`.
    Store {
        addr: Expr,
        off: u32,
        value: Expr,
    },
    /// Copy `size` bytes from `src` to `dst`. The two ranges are either
    /// disjoint or identical; lowering never makes a partial overlap.
    Copy {
        dst: Expr,
        src: Expr,
        size: u32,
    },
}

/// One frame slot: a stack object of a fixed size (an aggregate local, or a
/// temporary for a struct literal or a by-value result).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotInfo {
    pub size: u32,
    /// 1, 2, 4 or 8.
    pub align: u32,
}

#[derive(Clone, Debug)]
pub struct Func {
    pub name: String,
    /// Parameters are vars 0..nparams.
    pub nparams: usize,
    pub ret: Option<Ty>,
    pub vars: Vec<Var>,
    pub body: Vec<Stmt>,
    pub line: u32,
    /// A `test` or `invariant` block: no parameters, no return value, never
    /// callable.
    pub is_test: bool,
    /// An `invariant` block (also `is_test`). t27c's Zig backend emits one as
    /// a `comptime { ... }` block, so a broken invariant is a compile error
    /// there; t27b runs it at test time, through the same trap machinery as a
    /// test, and reports it separately.
    pub is_invariant: bool,
    /// Site used when control falls off the end of a non-void fn.
    pub noreturn_site: SiteId,
    /// Frame slots, addressed by `ExprKind::Slot`.
    pub slots: Vec<SlotInfo>,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub module: String,
    pub funcs: Vec<Func>,
    /// Site 0 is a placeholder meaning "cannot trap".
    pub sites: Vec<Site>,
    pub mode: OverflowMode,
    /// Invariants whose body the front-end discarded (a `forall`, or a clause
    /// it cannot parse): nothing to run, and reported as NOT CHECKED rather
    /// than as held.
    pub unchecked: Vec<String>,
    /// Read-only data blobs (string bytes, constant aggregates), addressed by
    /// `ExprKind::Data`. Each is placed 8-byte aligned.
    pub data: Vec<Vec<u8>>,
    /// Functions that pass or return an aggregate. They use t27b's own
    /// convention (a pointer to the caller's copy; a hidden last pointer
    /// parameter for the result, which is also returned), not AAPCS64, so an
    /// object file does not export them.
    pub internal_abi: Vec<FuncId>,
}

impl Program {
    pub fn func_index(&self, name: &str) -> Option<usize> {
        self.funcs.iter().position(|f| !f.is_test && f.name == name)
    }

    pub fn tests(&self) -> impl Iterator<Item = (usize, &Func)> {
        self.funcs.iter().enumerate().filter(|(_, f)| f.is_test)
    }
}
