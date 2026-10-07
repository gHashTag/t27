//! Reference interpreter over the t27b IR.
//!
//! Exact integer semantics in i128, written independently of the code
//! generator so the two can be compared. This is the oracle of the randomized
//! differential test: every value and every trap site the JIT produces must be
//! the one this interpreter produces.
//!
//! Memory (frame slots and read-only data) is a checked model, not a byte
//! array the program can wander through: every load and store must fall
//! inside one live object, a load must read only bytes already written, and a
//! store into read-only data is refused. Any such access is `Stop::Fault` --
//! never a value -- because it means the IR itself is wrong, and the JIT would
//! silently read whatever the stack held.

use crate::ir::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stop {
    /// A trap at the given site; for AssertEq, the two compared values.
    Trap { site: SiteId, a: i128, b: i128 },
    /// The step budget ran out (likely an infinite loop).
    Fuel,
    /// Call depth limit (runaway recursion).
    Depth,
    /// An access outside every live object, a read of unwritten bytes, or a
    /// store into read-only data: an IR defect, not a program outcome.
    Fault(String),
}

/// Base address of frame slots in the model. Chosen so that no address in
/// the model is a plausible small integer.
pub const STACK_BASE: u64 = 0x1000_0000_0000;
/// Base address of `Program::data` in the model.
pub const DATA_BASE: u64 = 0x2000_0000_0000;

/// One function activation: variable values and the addresses of its slots.
struct Frame {
    vars: Vec<i128>,
    slots: Vec<u64>,
}

/// The checked memory model.
struct Mem {
    stack: Vec<u8>,
    written: Vec<bool>,
    /// Live stack objects as (start offset, size), in allocation order.
    objects: Vec<(usize, usize)>,
    data: Vec<u8>,
    /// Read-only objects as (start offset, size), sorted.
    data_objects: Vec<(usize, usize)>,
    /// Module-level vars: the first stack objects, never freed, with their
    /// initial bytes (restored on every host entry, see `Interp::call`).
    globals: Vec<(u64, Vec<u8>)>,
}

impl Mem {
    fn new(prog: &Program) -> Mem {
        let mut data = Vec::new();
        let mut data_objects = Vec::new();
        for blob in &prog.data {
            while data.len() % 8 != 0 {
                data.push(0);
            }
            data_objects.push((data.len(), blob.len()));
            data.extend_from_slice(blob);
        }
        let mut m = Mem { stack: Vec::new(), written: Vec::new(), objects: Vec::new(), data, data_objects, globals: Vec::new() };
        for g in &prog.globals {
            let a = m.alloc(&SlotInfo { size: g.len() as u32, align: 8 });
            m.globals.push((a, g.clone()));
        }
        m.reset_globals();
        m
    }

    /// Give every module-level var its initial bytes (all written).
    fn reset_globals(&mut self) {
        for (a, init) in &self.globals {
            let off = (a - STACK_BASE) as usize;
            self.stack[off..off + init.len()].copy_from_slice(init);
            for w in &mut self.written[off..off + init.len()] {
                *w = true;
            }
        }
    }

    /// Model address of data blob `k`.
    fn data_addr(&self, k: u32) -> u64 {
        DATA_BASE + self.data_objects[k as usize].0 as u64
    }

    fn alloc(&mut self, s: &SlotInfo) -> u64 {
        let align = s.align.max(1) as usize;
        let mut at = self.stack.len();
        at = (at + align - 1) / align * align;
        // A one-byte gap between objects: an access running off the end of
        // one object never lands in the next.
        let end = at + s.size as usize + 1;
        self.stack.resize(end, 0xAA);
        self.written.resize(end, false);
        self.objects.push((at, s.size as usize));
        STACK_BASE + at as u64
    }

    fn free_to(&mut self, nobjects: usize) {
        self.objects.truncate(nobjects);
        let top = self.objects.last().map_or(0, |&(a, n)| a + n + 1);
        self.stack.truncate(top);
        self.written.truncate(top);
    }

    /// Resolve `[addr, addr + n)` to (is_stack, offset), or fault.
    fn locate(&self, addr: u64, n: usize, store: bool) -> Result<(bool, usize), Stop> {
        if addr >= STACK_BASE && addr < DATA_BASE {
            let off = (addr - STACK_BASE) as usize;
            for &(a, size) in self.objects.iter().rev() {
                if off >= a && off + n <= a + size {
                    return Ok((true, off));
                }
            }
            return Err(Stop::Fault(format!("{}-byte access at stack+{:#x} is outside every live slot", n, off)));
        }
        if addr >= DATA_BASE {
            let off = (addr - DATA_BASE) as usize;
            if store {
                return Err(Stop::Fault(format!("store into read-only data at data+{:#x}", off)));
            }
            for &(a, size) in &self.data_objects {
                if off >= a && off + n <= a + size {
                    return Ok((false, off));
                }
            }
            return Err(Stop::Fault(format!("{}-byte read at data+{:#x} is outside every blob", n, off)));
        }
        Err(Stop::Fault(format!("{}-byte access at {:#x}, which is not an address", n, addr)))
    }

    fn read(&self, addr: u64, n: usize) -> Result<&[u8], Stop> {
        let (stack, off) = self.locate(addr, n, false)?;
        if stack {
            if self.written[off..off + n].iter().any(|w| !w) {
                return Err(Stop::Fault(format!("{}-byte read at stack+{:#x} of bytes never written", n, off)));
            }
            Ok(&self.stack[off..off + n])
        } else {
            Ok(&self.data[off..off + n])
        }
    }

    fn write(&mut self, addr: u64, bytes: &[u8]) -> Result<(), Stop> {
        let (_, off) = self.locate(addr, bytes.len(), true)?;
        self.stack[off..off + bytes.len()].copy_from_slice(bytes);
        for w in &mut self.written[off..off + bytes.len()] {
            *w = true;
        }
        Ok(())
    }

    fn load(&self, addr: u64, ty: Ty) -> Result<i128, Stop> {
        let n = ty.bytes() as usize;
        let b = self.read(addr, n)?;
        let mut raw = [0u8; 8];
        raw[..n].copy_from_slice(b);
        let v = u64::from_le_bytes(raw);
        if ty == Ty::Bool && v > 1 {
            return Err(Stop::Fault(format!("bool load of {} at {:#x}", v, addr)));
        }
        Ok(if ty == Ty::Ptr { v as i128 } else { ty.from_raw(v) })
    }

    fn store(&mut self, addr: u64, ty: Ty, v: i128) -> Result<(), Stop> {
        let n = ty.bytes() as usize;
        let raw = (v as u64).to_le_bytes();
        self.write(addr, &raw[..n])
    }
}

enum Flow {
    Next,
    Break,
    Continue,
    Return(Option<i128>),
}

pub struct Interp<'p> {
    pub prog: &'p Program,
    pub fuel: u64,
    pub max_depth: u32,
    /// Runtime asserts executed so far (#6115): every `assert` / `assert_eq`
    /// statement reached whose operands are not all compile-time constants.
    /// An assert the lowering folded to a constant is not counted: the code
    /// generator emits nothing for `assert(true)`, so it checks nothing at
    /// run time. A test that passes with this at 0 is a vacuous pass.
    pub asserts: u64,
    depth: u32,
    mem: Mem,
}

/// True when the IR expression is a compile-time constant.
fn is_const(e: &Expr) -> bool {
    matches!(e.kind, ExprKind::Const(_))
}

/// Exact arithmetic of one IR operation on in-range operands.
/// Returns Err(site offset 0 or 1) for a trap; the caller maps it to a site.
pub fn arith(op: ArithOp, ty: Ty, a: i128, b: i128, amt_ty: Ty) -> Result<i128, TrapKind> {
    let bits = ty.bits() as i128;
    let fit = |r: i128| if ty.fits(r) { Ok(r) } else { Err(TrapKind::Overflow) };
    match op {
        ArithOp::Add => fit(a + b),
        ArithOp::Sub => fit(a - b),
        ArithOp::Mul => match a.checked_mul(b) {
            Some(r) => fit(r),
            None => Err(TrapKind::Overflow),
        },
        ArithOp::AddW => Ok(ty.wrap(a + b)),
        ArithOp::SubW => Ok(ty.wrap(a - b)),
        ArithOp::MulW => Ok(ty.wrap(((a as u128).wrapping_mul(b as u128)) as i128)),
        ArithOp::Div => {
            if b == 0 {
                return Err(TrapKind::DivZero);
            }
            fit(a / b)
        }
        ArithOp::DivW => {
            if b == 0 {
                return Err(TrapKind::DivZero);
            }
            Ok(ty.wrap(a / b))
        }
        ArithOp::Rem => {
            if b == 0 {
                return Err(TrapKind::DivZero);
            }
            Ok(a % b)
        }
        ArithOp::And => Ok(a & b),
        ArithOp::Or => Ok(a | b),
        ArithOp::Xor => Ok(a ^ b),
        ArithOp::Shl | ArithOp::Shr | ArithOp::ShlW | ArithOp::ShrW => {
            let _ = amt_ty;
            let amt = if matches!(op, ArithOp::Shl | ArithOp::Shr) {
                if b < 0 || b >= bits {
                    return Err(TrapKind::ShiftRange);
                }
                b
            } else if (0..bits).contains(&b) {
                b
            } else {
                // Wrap mode: the low bits of the amount (a power-of-two
                // width; lowering refuses a runtime one on an odd width).
                b & (bits - 1)
            };
            if matches!(op, ArithOp::Shl | ArithOp::ShlW) {
                Ok(ty.wrap(((a as u128) << amt) as i128))
            } else {
                Ok(a >> amt)
            }
        }
    }
}

impl<'p> Interp<'p> {
    pub fn new(prog: &'p Program) -> Self {
        Interp {
            prog,
            fuel: 50_000_000,
            max_depth: 2000,
            asserts: 0,
            depth: 0,
            mem: Mem::new(prog),
        }
    }

    /// Call function `f` with argument values (already in range).
    pub fn call(&mut self, f: usize, args: &[i128]) -> Result<Option<i128>, Stop> {
        let func = &self.prog.funcs[f];
        if self.depth >= self.max_depth {
            return Err(Stop::Depth);
        }
        if self.depth == 0 {
            // Each host entry sees fresh module-level vars, as each test of
            // the reference runs in its own process.
            self.mem.reset_globals();
        }
        self.depth += 1;
        let mark = self.mem.objects.len();
        let mut env = Frame {
            vars: vec![0i128; func.vars.len()],
            slots: func.slots.iter().map(|s| self.mem.alloc(s)).collect(),
        };
        env.vars[..args.len()].copy_from_slice(args);
        let r = self.block(func, &func.body, &mut env);
        self.mem.free_to(mark);
        self.depth -= 1;
        match r? {
            Flow::Return(v) => Ok(v),
            _ => {
                if func.ret.is_some() {
                    Err(Stop::Trap { site: func.noreturn_site, a: 0, b: 0 })
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn tick(&mut self) -> Result<(), Stop> {
        if self.fuel == 0 {
            return Err(Stop::Fuel);
        }
        self.fuel -= 1;
        Ok(())
    }

    fn block(&mut self, f: &Func, ss: &[Stmt], env: &mut Frame) -> Result<Flow, Stop> {
        for s in ss {
            match self.stmt(f, s, env)? {
                Flow::Next => {}
                other => return Ok(other),
            }
        }
        Ok(Flow::Next)
    }

    fn stmt(&mut self, f: &Func, s: &Stmt, env: &mut Frame) -> Result<Flow, Stop> {
        self.tick()?;
        match s {
            Stmt::Assign { var, value } => {
                let v = self.expr(value, env)?;
                env.vars[*var as usize] = v;
                Ok(Flow::Next)
            }
            Stmt::If { cond, then, els } => {
                if self.expr(cond, env)? != 0 {
                    self.block(f, then, env)
                } else {
                    self.block(f, els, env)
                }
            }
            Stmt::While { cond, body, step } => {
                loop {
                    self.tick()?;
                    if self.expr(cond, env)? == 0 {
                        break;
                    }
                    match self.block(f, body, env)? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Next | Flow::Continue => {}
                    }
                    match self.block(f, step, env)? {
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        _ => {}
                    }
                }
                Ok(Flow::Next)
            }
            Stmt::Break => Ok(Flow::Break),
            Stmt::Continue => Ok(Flow::Continue),
            Stmt::Return(e) => {
                let v = match e {
                    Some(e) => Some(self.expr(e, env)?),
                    None => None,
                };
                Ok(Flow::Return(v))
            }
            Stmt::Eval(_) | Stmt::Store { .. } | Stmt::Copy { .. } => {
                self.effect(s, env)?;
                Ok(Flow::Next)
            }
            Stmt::Assert { cond, site } => {
                if !is_const(cond) {
                    self.asserts += 1;
                }
                if self.expr(cond, env)? == 0 {
                    return Err(Stop::Trap { site: *site, a: 0, b: 0 });
                }
                Ok(Flow::Next)
            }
            Stmt::AssertEq { lhs, rhs, site } => {
                if !(is_const(lhs) && is_const(rhs)) {
                    self.asserts += 1;
                }
                let a = self.expr(lhs, env)?;
                let b = self.expr(rhs, env)?;
                // Floats compare as IEEE values: NaN never equals, -0.0 == 0.0.
                let differ = if lhs.ty.is_float() { float_of(a, lhs.ty) != float_of(b, lhs.ty) } else { a != b };
                if differ {
                    return Err(Stop::Trap { site: *site, a, b });
                }
                Ok(Flow::Next)
            }
        }
    }

    /// The statements an `ExprKind::Seq` may hold, which have no control
    /// flow: `Eval`, `Store` and `Copy`.
    fn effect(&mut self, s: &Stmt, env: &mut Frame) -> Result<(), Stop> {
        match s {
            Stmt::Eval(e) => {
                if let ExprKind::Call { func, args } = &e.kind {
                    let mut vals = Vec::with_capacity(args.len());
                    for a in args {
                        vals.push(self.expr(a, env)?);
                    }
                    self.call(*func as usize, &vals)?;
                } else {
                    self.expr(e, env)?;
                }
                Ok(())
            }
            Stmt::Store { addr, off, value } => {
                let p = self.expr(addr, env)? as u64;
                let v = self.expr(value, env)?;
                self.mem.store(p.wrapping_add(*off as u64), value.ty, v)?;
                Ok(())
            }
            Stmt::Copy { dst, src, size } => {
                let d = self.expr(dst, env)? as u64;
                let s = self.expr(src, env)? as u64;
                let n = *size as usize;
                if n > 0 {
                    // Partially written sources (a struct whose padding was
                    // never stored) copy byte by byte, written-ness included.
                    let (ss, so) = self.mem.locate(s, n, false)?;
                    let bytes: Vec<(u8, bool)> = if ss {
                        (so..so + n).map(|i| (self.mem.stack[i], self.mem.written[i])).collect()
                    } else {
                        self.mem.data[so..so + n].iter().map(|&b| (b, true)).collect()
                    };
                    let (_, dofs) = self.mem.locate(d, n, true)?;
                    for (i, (b, w)) in bytes.into_iter().enumerate() {
                        self.mem.stack[dofs + i] = b;
                        self.mem.written[dofs + i] = w;
                    }
                }
                Ok(())
            }
            _ => Err(Stop::Fault("a statement with control flow inside an expression".into())),
        }
    }

    fn expr(&mut self, e: &Expr, env: &mut Frame) -> Result<i128, Stop> {
        match &e.kind {
            ExprKind::Const(c) => Ok(*c),
            ExprKind::Var(v) => Ok(env.vars[*v as usize]),
            ExprKind::Arith { op, lhs, rhs, site } => {
                let a = self.expr(lhs, env)?;
                let b = self.expr(rhs, env)?;
                match arith(*op, e.ty, a, b, rhs.ty) {
                    Ok(r) => Ok(r),
                    Err(kind) => {
                        // Div has two sites: zero divisor first, overflow next.
                        let s = if *op == ArithOp::Div && kind == TrapKind::Overflow {
                            site + 1
                        } else {
                            *site
                        };
                        Err(Stop::Trap { site: s, a: 0, b: 0 })
                    }
                }
            }
            ExprKind::Cmp { op, lhs, rhs } => {
                let a = self.expr(lhs, env)?;
                let b = self.expr(rhs, env)?;
                if lhs.ty.is_float() {
                    // A binary32 widens to binary64 exactly: same order.
                    return Ok(op.holds_f64(float_of(a, lhs.ty), float_of(b, lhs.ty)) as i128);
                }
                Ok(op.holds(a, b) as i128)
            }
            ExprKind::FArith { op, lhs, rhs } => {
                let a = self.expr(lhs, env)?;
                let b = self.expr(rhs, env)?;
                Ok(op.apply_bits(e.ty, a, b))
            }
            ExprKind::FNeg(a) => {
                // Flips the sign bit, of a NaN too.
                let v = self.expr(a, env)?;
                Ok(v ^ (1i128 << (e.ty.bits() - 1)))
            }
            ExprKind::FSqrt(a) => {
                let v = self.expr(a, env)?;
                Ok(if e.ty == Ty::F32 { f32_bits(f32_of(v).sqrt()) } else { f64_bits(f64_of(v).sqrt()) })
            }
            ExprKind::IntToFloat(a) => {
                // i128 to f64 / f32 rounds to nearest, ties to even; every
                // integer operand is below 2^64, so this is the one rounding.
                let v = self.expr(a, env)?;
                Ok(if e.ty == Ty::F32 { f32_bits(v as f32) } else { f64_bits(v as f64) })
            }
            ExprKind::FloatCast(a) => {
                let v = self.expr(a, env)?;
                Ok(if e.ty == Ty::F32 { f32_bits(f64_of(v) as f32) } else { f64_bits(f32_of(v) as f64) })
            }
            ExprKind::FloatToInt { arg, site } => {
                let v = self.expr(arg, env)?;
                match float_to_int(float_of(v, arg.ty), e.ty) {
                    Some(r) => Ok(r),
                    None => Err(Stop::Trap { site: *site, a: 0, b: 0 }),
                }
            }
            ExprKind::And(a, b) => {
                if self.expr(a, env)? == 0 {
                    Ok(0)
                } else {
                    self.expr(b, env)
                }
            }
            ExprKind::Or(a, b) => {
                if self.expr(a, env)? != 0 {
                    Ok(1)
                } else {
                    self.expr(b, env)
                }
            }
            ExprKind::Not(a) => Ok(1 - self.expr(a, env)?),
            ExprKind::BitNot(a) => {
                let v = self.expr(a, env)?;
                Ok(e.ty.wrap(!v))
            }
            ExprKind::Call { func, args } => {
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(self.expr(a, env)?);
                }
                match self.call(*func as usize, &vals)? {
                    Some(v) => Ok(v),
                    None => Ok(0),
                }
            }
            ExprKind::Widen(a) => self.expr(a, env),
            ExprKind::Cast { arg, site } => {
                let v = self.expr(arg, env)?;
                if *site == 0 {
                    Ok(e.ty.wrap(v))
                } else if e.ty.fits(v) {
                    Ok(v)
                } else {
                    Err(Stop::Trap { site: *site, a: 0, b: 0 })
                }
            }
            ExprKind::Slot(k) => Ok(env.slots[*k as usize] as i128),
            ExprKind::Data(k) => Ok(self.mem.data_addr(*k) as i128),
            ExprKind::Global(k) => Ok(self.mem.globals[*k as usize].0 as i128),
            ExprKind::Load { addr, off } => {
                let p = self.expr(addr, env)? as u64;
                self.mem.load(p.wrapping_add(*off as u64), e.ty)
            }
            ExprKind::Offset { base, idx, scale } => {
                let b = self.expr(base, env)? as u64;
                let i = self.expr(idx, env)? as u64;
                Ok(b.wrapping_add(i.wrapping_mul(*scale as u64)) as i128)
            }
            ExprKind::Bounds { idx, len, site } => {
                let i = self.expr(idx, env)?;
                let n = self.expr(len, env)?;
                if i < n {
                    Ok(i)
                } else {
                    Err(Stop::Trap { site: *site, a: i, b: n })
                }
            }
            ExprKind::Seq { stmts, value } => {
                for s in stmts {
                    self.tick()?;
                    self.effect(s, env)?;
                }
                self.expr(value, env)
            }
            ExprKind::Select { cond, then, els } => {
                if self.expr(cond, env)? != 0 {
                    self.expr(then, env)
                } else {
                    self.expr(els, env)
                }
            }
        }
    }
}
