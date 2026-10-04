//! Reference interpreter over the t27b IR.
//!
//! Exact integer semantics in i128, written independently of the code
//! generator so the two can be compared. This is the oracle of the randomized
//! differential test: every value and every trap site the JIT produces must be
//! the one this interpreter produces.

use crate::ir::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stop {
    /// A trap at the given site; for AssertEq, the two compared values.
    Trap { site: SiteId, a: i128, b: i128 },
    /// The step budget ran out (likely an infinite loop).
    Fuel,
    /// Call depth limit (runaway recursion).
    Depth,
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
    depth: u32,
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
            } else {
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
            depth: 0,
        }
    }

    /// Call function `f` with argument values (already in range).
    pub fn call(&mut self, f: usize, args: &[i128]) -> Result<Option<i128>, Stop> {
        let func = &self.prog.funcs[f];
        if self.depth >= self.max_depth {
            return Err(Stop::Depth);
        }
        self.depth += 1;
        let mut env = vec![0i128; func.vars.len()];
        env[..args.len()].copy_from_slice(args);
        let r = self.block(func, &func.body, &mut env);
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

    fn block(&mut self, f: &Func, ss: &[Stmt], env: &mut Vec<i128>) -> Result<Flow, Stop> {
        for s in ss {
            match self.stmt(f, s, env)? {
                Flow::Next => {}
                other => return Ok(other),
            }
        }
        Ok(Flow::Next)
    }

    fn stmt(&mut self, f: &Func, s: &Stmt, env: &mut Vec<i128>) -> Result<Flow, Stop> {
        self.tick()?;
        match s {
            Stmt::Assign { var, value } => {
                let v = self.expr(value, env)?;
                env[*var as usize] = v;
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
                Ok(Flow::Next)
            }
            Stmt::Assert { cond, site } => {
                if self.expr(cond, env)? == 0 {
                    return Err(Stop::Trap { site: *site, a: 0, b: 0 });
                }
                Ok(Flow::Next)
            }
            Stmt::AssertEq { lhs, rhs, site } => {
                let a = self.expr(lhs, env)?;
                let b = self.expr(rhs, env)?;
                if a != b {
                    return Err(Stop::Trap { site: *site, a, b });
                }
                Ok(Flow::Next)
            }
        }
    }

    pub fn expr(&mut self, e: &Expr, env: &mut Vec<i128>) -> Result<i128, Stop> {
        match &e.kind {
            ExprKind::Const(c) => Ok(*c),
            ExprKind::Var(v) => Ok(env[*v as usize]),
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
                Ok(op.holds(a, b) as i128)
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
        }
    }
}
