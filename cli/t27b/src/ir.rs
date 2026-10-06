//! t27b IR: a typed, name-resolved, desugared tree.
//!
//! Produced by `lower` from the t27c AST, consumed by `eval` (the reference
//! interpreter) and by `codegen` (AArch64). Every operation that can trap
//! carries a `site`, an index into `Program::sites`, so the interpreter and
//! the generated code can be compared trap-for-trap, not only value-for-value.

/// Integer, boolean and float types of the supported subset. `usize` lowers
/// to `U64`, `isize` to `I64`.
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
    /// IEEE-754 binary64. A value is carried as its 64-bit pattern (a
    /// non-negative `i128` below 2^64 in the IR and the interpreter, the whole
    /// X register in generated code); arithmetic happens in D registers, and
    /// AAPCS64 passes and returns it in d0-d7.
    F64,
    /// IEEE-754 binary32: its 32-bit pattern, canonical like a u32 (zero-
    /// extended in the X register); arithmetic happens in S registers, and
    /// AAPCS64 passes and returns it in s0-s7.
    F32,
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
            "f64" => Ty::F64,
            "f32" => Ty::F32,
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
            Ty::F64 => "f64",
            Ty::F32 => "f32",
        }
    }

    pub fn is_int(self) -> bool {
        !matches!(self, Ty::Bool | Ty::Ptr | Ty::F64 | Ty::F32)
    }

    pub fn is_float(self) -> bool {
        matches!(self, Ty::F64 | Ty::F32)
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
            Ty::U32 | Ty::I32 | Ty::F32 => 32,
            Ty::U64 | Ty::I64 | Ty::Ptr | Ty::F64 => 64,
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

/// The binary64 value of an F64 IR value (its bit pattern).
pub fn f64_of(v: i128) -> f64 {
    f64::from_bits(v as u64)
}

/// The IR value (bit pattern, as a non-negative i128) of a binary64.
pub fn f64_bits(x: f64) -> i128 {
    x.to_bits() as i128
}

/// The binary32 value of an F32 IR value (its bit pattern).
pub fn f32_of(v: i128) -> f32 {
    f32::from_bits(v as u32)
}

/// The IR value (bit pattern, as a non-negative i128) of a binary32.
pub fn f32_bits(x: f32) -> i128 {
    x.to_bits() as i128
}

/// The value of a float IR constant of type `ty` (F32 or F64) as a binary64:
/// exact, since every binary32 is a binary64.
pub fn float_of(v: i128, ty: Ty) -> f64 {
    if ty == Ty::F32 {
        f32_of(v) as f64
    } else {
        f64_of(v)
    }
}

/// `@intFromFloat(x)` to integer `ty`: `x` truncated toward zero, or None
/// when that is outside `ty` (the infinities included). A NaN gives 0, as
/// Zig 0.16 Debug does (it panics on infinities, not on NaN).
pub fn float_to_int(x: f64, ty: Ty) -> Option<i128> {
    if x.is_nan() {
        return Some(0);
    }
    let t = x.trunc();
    // |t| < 2^100 is finite, not NaN, and converts to i128 exactly.
    if !(t.abs() < 1.0e30) {
        return None;
    }
    let v = t as i128;
    if ty.fits(v) {
        Some(v)
    } else {
        None
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

/// IEEE-754 binary operations on F64.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum FOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl FOp {
    pub fn symbol(self) -> &'static str {
        match self {
            FOp::Add => "+",
            FOp::Sub => "-",
            FOp::Mul => "*",
            FOp::Div => "/",
        }
    }

    /// The operation on host binary64 values (round to nearest, ties to even).
    pub fn apply(self, a: f64, b: f64) -> f64 {
        match self {
            FOp::Add => a + b,
            FOp::Sub => a - b,
            FOp::Mul => a * b,
            FOp::Div => a / b,
        }
    }

    /// The operation on host binary32 values (round to nearest, ties to even).
    pub fn apply_f32(self, a: f32, b: f32) -> f32 {
        match self {
            FOp::Add => a + b,
            FOp::Sub => a - b,
            FOp::Mul => a * b,
            FOp::Div => a / b,
        }
    }

    /// The operation on two `ty` (F32 or F64) bit patterns.
    pub fn apply_bits(self, ty: Ty, a: i128, b: i128) -> i128 {
        if ty == Ty::F32 {
            f32_bits(self.apply_f32(f32_of(a), f32_of(b)))
        } else {
            f64_bits(self.apply(f64_of(a), f64_of(b)))
        }
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

    /// The comparison on two binary64 values: false for every op but `!=`
    /// when either is NaN.
    pub fn holds_f64(self, a: f64, b: f64) -> bool {
        match self {
            CmpOp::Eq => a == b,
            CmpOp::Ne => a != b,
            CmpOp::Lt => a < b,
            CmpOp::Le => a <= b,
            CmpOp::Gt => a > b,
            CmpOp::Ge => a >= b,
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
    /// `@enumFromInt(x)` whose value is no tag of the enum (Zig's safety
    /// check on the conversion).
    EnumTag = 8,
    /// `@intFromFloat(x)` whose integer part is outside the result type, or
    /// x is NaN or infinite (Zig's safety check on the conversion).
    FloatToInt = 9,
    /// `undefined;` or an ignored value (`expr;`) in a fn the reference
    /// never analyzes: no test reaches it, so this trap is never expected
    /// to fire.
    Stub = 10,
    /// `x.?` where `x` is `null` (Zig's safety check on the unwrap).
    Null = 11,
    // Memory lane: numbered from 16 so the scalar lane can add kinds below.
    /// An index at or past the length of an array, slice or string.
    Bounds = 16,
    /// `for (a, b) |x, y|` over objects whose lengths differ at run time
    /// (Zig's safety check before the first iteration).
    ForLength = 17,
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
            TrapKind::EnumTag => "invalid enum value",
            TrapKind::FloatToInt => "integer part of floating point value out of bounds",
            TrapKind::Bounds => "index out of bounds",
            TrapKind::ForLength => "for loop over objects with non-equal lengths",
            TrapKind::Null => "attempt to use null value",
            TrapKind::Stub => "reached a statement the reference never compiles (`undefined;` or an ignored value)",
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
    /// `ty` is Bool; both operands share one type (an integer type, Bool
    /// for `==` and `!=`, or F64 / F32, compared as IEEE values: NaN is unordered,
    /// so only `!=` holds for it, and `-0.0 == 0.0`).
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
    /// IEEE-754 `+ - * /` of two `ty` (F64 or F32) operands, rounding to nearest,
    /// ties to even. Never traps: overflow gives an infinity, 0/0 a NaN.
    FArith {
        op: FOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// Float negation: flips the sign bit (so `-0.0` and NaNs too).
    FNeg(Box<Expr>),
    /// `@floatFromInt`: the integer operand converted to float `ty`, rounding
    /// to nearest, ties to even (once: an i64 goes straight to F32).
    IntToFloat(Box<Expr>),
    /// The float operand converted to the other float type `ty`: F32 to F64
    /// is exact (Zig's implicit widening), F64 to F32 rounds to nearest,
    /// ties to even, overflowing to an infinity (`@floatCast`); a NaN stays
    /// a quiet NaN.
    FloatCast(Box<Expr>),
    /// `@intFromFloat`: the F64 or F32 operand truncated toward zero to integer
    /// `ty`; traps at `site` when that does not fit `ty` (NaN and the
    /// infinities never fit).
    FloatToInt {
        arg: Box<Expr>,
        site: SiteId,
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
    /// Address of module-level `var` `k` (`Program::globals`): writable, and
    /// back at its initial bytes whenever the host enters a function, as each
    /// test of `t27c test-report` runs in a process of its own.
    Global(u32),
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
    /// `if (cond) then else els` used as a value: `cond` (Bool) first, then
    /// exactly one of the two arms (both `ty`); the other is never evaluated,
    /// so its traps, calls and stores do not happen.
    Select {
        cond: Box<Expr>,
        then: Box<Expr>,
        els: Box<Expr>,
    },
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
    /// Initial bytes of each module-level `var`, addressed by
    /// `ExprKind::Global`. Each is placed 8-byte aligned in writable memory.
    pub globals: Vec<Vec<u8>>,
    /// Functions that pass or return an aggregate, or take more parameters
    /// of one class than AAPCS64 has argument registers. They use t27b's own
    /// convention (a pointer to the caller's copy; a hidden last pointer
    /// parameter for the result, which is also returned; one full word per
    /// stack parameter, see `Program::stack_args`), not the platform's, so an
    /// object file does not export them.
    pub internal_abi: Vec<FuncId>,
}

impl Program {
    pub fn func_index(&self, name: &str) -> Option<usize> {
        self.funcs.iter().position(|f| !f.is_test && f.name == name)
    }

    /// AAPCS64 argument locations of `f`'s parameters: for each, true when
    /// it is a float (passed in the next of d0-d7 / s0-s7) and its index among the
    /// parameters of its own class (x registers or d registers).
    pub fn arg_regs(f: &Func) -> Vec<(bool, usize)> {
        let (mut xi, mut di) = (0, 0);
        f.vars[..f.nparams]
            .iter()
            .map(|v| {
                if v.ty.is_float() {
                    di += 1;
                    (true, di - 1)
                } else {
                    xi += 1;
                    (false, xi - 1)
                }
            })
            .collect()
    }

    /// Stack slot of each parameter that AAPCS64 would not place in a
    /// register (the ninth and later of its class), in parameter order; and
    /// how many there are. Slot `s` is the 8-byte word at `[sp + 8*s]` of the
    /// caller at the `bl`, i.e. `[x29 + 16 + 8*s]` in the callee. Every slot is
    /// a full word holding the register image (t27b's own convention: Apple's
    /// arm64 packs narrow stack arguments, so a function with stack
    /// parameters is never exported, see `internal_abi`).
    pub fn stack_args(regs: &[(bool, usize)]) -> (Vec<Option<u32>>, u32) {
        let mut n = 0u32;
        let v = regs
            .iter()
            .map(|&(_, k)| {
                if k >= 8 {
                    n += 1;
                    Some(n - 1)
                } else {
                    None
                }
            })
            .collect();
        (v, n)
    }

    pub fn tests(&self) -> impl Iterator<Item = (usize, &Func)> {
        self.funcs.iter().enumerate().filter(|(_, f)| f.is_test)
    }
}
