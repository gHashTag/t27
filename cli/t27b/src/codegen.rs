//! AArch64 code generation from the t27b IR.
//!
//! One pass per function, directly to machine words (no assembler text):
//!
//! * Canonical register form. A value of a type of 32 bits or less lives in
//!   the low 32 bits of its register: u8/u16/bool zero-extended to 32 bits,
//!   i8/i16 sign-extended to 32 bits, the upper 32 bits undefined. A 64-bit
//!   value uses the whole X register. All arithmetic on narrow types is done in
//!   W registers and re-normalised (or overflow-checked) afterwards.
//! * Registers. x0-x7 arguments and result (and variable homes in leaf
//!   functions); x8 operation scratch; x9-x15 the expression temp stack (temp
//!   7 and deeper spill to the frame); x16/x17 scratch for spilled temps and
//!   materialised constants; x18 never touched (Apple platform register);
//!   x19-x28 variable homes, assigned by loop-weighted use count; x29/x30
//!   frame pointer and link register.
//! * Frame. `stp x29, x30, [sp, #-16]!; mov x29, sp; sub sp, sp, #N`, then
//!   from sp upward: variable slots, saved callee-saved registers, temp slots
//!   (spilled temps and temps saved across calls); and, directly below x29,
//!   the aggregate area holding `Func::slots`, addressed from x29 so its
//!   offsets do not depend on how many temps the body needs. Leaf functions
//!   that need none of this get no frame at all.
//! * Memory. Loads and stores take the narrowest instruction for the type
//!   (`ldrsb w` for i8, `ldrh` for u16, ...), so a loaded value is already in
//!   canonical form. A slot access close to x29 is one `ldur`/`stur`; read-only
//!   data is reached with `adrp` + `add`, resolved by the JIT loader or by
//!   PAGE21/PAGEOFF12 relocations in the object file. Large copies loop with
//!   x30 as the end pointer, which forces a frame.
//! * Traps. Every check branches to an out-of-line stub at the end of the
//!   function. JIT stubs load the site number (and the assert_eq operands) and
//!   jump to the shared `trap_common` routine of the JIT image; object-file
//!   stubs are `brk #kind`.
//! * Calls inside the module are `bl` with the offset filled in by `link`.

use crate::a64::{self, Cond, Ext, LogOp, Reg, Shift, SP, ZR};
use crate::ir::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrapStyle {
    /// In-process JIT: stubs record the site and branch to `trap_common`.
    Jit,
    /// Object file: stubs are `brk #kind`.
    Brk,
}

/// Machine code of one function before linking.
#[derive(Clone, Debug)]
pub struct FuncCode {
    pub func: FuncId,
    pub code: Vec<u32>,
    /// Word index of every `bl`, with its callee.
    pub calls: Vec<(usize, FuncId)>,
    /// Word index of every `b trap_common`.
    pub trap_jumps: Vec<usize>,
    /// Word index of every `adrp` (followed by its `add`), with the blob.
    pub data_refs: Vec<(usize, u32)>,
}

const GLOBAL_IN_OBJECT: &str = "module-level var in an object file";

/// A function the generator cannot encode (frame or branch range limits).
#[derive(Clone, Debug)]
pub struct CodegenError {
    pub func: String,
    pub line: u32,
    pub construct: &'static str,
    pub detail: String,
}

const X0: Reg = 0;
const X8: Reg = 8;
const X16: Reg = 16;
const X17: Reg = 17;
const TEMP_REGS: usize = 7; // x9..x15
const CALLEE_SAVED: [Reg; 10] = [19, 20, 21, 22, 23, 24, 25, 26, 27, 28];
const MAX_SLOT_BYTES: u32 = 32760;
/// Largest aggregate area of one frame. Kept below the stack guard size so a
/// frame never reaches past the guard page without touching it.
pub const MAX_AGG_BYTES: u32 = 16384;
const LR: Reg = 30;
const FP: Reg = 29;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Home {
    None,
    Reg(Reg),
    /// Byte offset from sp.
    Slot(u32),
}

/// Where an evaluated value is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum V {
    /// A register owned by somebody else (a variable home, or the requested
    /// destination). Read-only for the consumer.
    Reg(Reg),
    /// Temp stack entry (x9+i for i < 7, else a frame slot).
    Temp(usize),
    /// A constant not yet materialised.
    Const(i128),
}

type Label = usize;

enum Stub {
    Site(SiteId),
    AssertEq(SiteId, Reg, Reg),
    Brk(TrapKind),
}

/// Register image of a constant of type `ty` (see canonical form above).
pub fn canon(c: i128, ty: Ty) -> u64 {
    if ty.is64() {
        c as u64
    } else {
        (c as i64 as u64) & 0xffff_ffff
    }
}

fn narrow_ext(ty: Ty) -> Option<Ext> {
    match ty {
        Ty::U8 | Ty::Bool => Some(Ext::Uxtb),
        Ty::U16 => Some(Ext::Uxth),
        Ty::I8 => Some(Ext::Sxtb),
        Ty::I16 => Some(Ext::Sxth),
        _ => None,
    }
}

fn cond_for(op: CmpOp, ty: Ty) -> Cond {
    if ty == Ty::F64 {
        return fcond_for(op);
    }
    let s = ty.signed();
    match op {
        CmpOp::Eq => Cond::Eq,
        CmpOp::Ne => Cond::Ne,
        CmpOp::Lt => {
            if s {
                Cond::Lt
            } else {
                Cond::Lo
            }
        }
        CmpOp::Le => {
            if s {
                Cond::Le
            } else {
                Cond::Ls
            }
        }
        CmpOp::Gt => {
            if s {
                Cond::Gt
            } else {
                Cond::Hi
            }
        }
        CmpOp::Ge => {
            if s {
                Cond::Ge
            } else {
                Cond::Hs
            }
        }
    }
}

/// The condition that holds after `fcmp dn, dm` when `dn op dm` is true.
/// An unordered compare (a NaN operand) sets NZCV to 0011: none of these
/// but NE holds then, and `Cond::invert` of each is its exact negation.
fn fcond_for(op: CmpOp) -> Cond {
    match op {
        CmpOp::Eq => Cond::Eq,
        CmpOp::Ne => Cond::Ne,
        CmpOp::Lt => Cond::Mi,
        CmpOp::Le => Cond::Ls,
        CmpOp::Gt => Cond::Gt,
        CmpOp::Ge => Cond::Ge,
    }
}

/// d registers used as scratch for one floating point operation. Values
/// live in X registers as bit patterns between operations; d8-d15 (callee
/// saved) are never touched.
const D16: Reg = 16;
const D17: Reg = 17;

struct Gen<'a> {
    prog: &'a Program,
    f: &'a Func,
    style: TrapStyle,
    code: Vec<u32>,
    labels: Vec<Option<usize>>,
    fixes: Vec<(usize, Label)>,
    calls: Vec<(usize, FuncId)>,
    trap_jumps: Vec<usize>,
    homes: Vec<Home>,
    depth: usize,
    temp_base: u32,
    max_slot: usize,
    loops: Vec<(Label, Label)>,
    stubs: Vec<(Label, Stub)>,
    stub_by_site: HashMap<SiteId, Label>,
    brk_by_kind: HashMap<u32, Label>,
    ret_label: Label,
    ret_jumps: Vec<usize>,
    used_callee: Vec<Reg>,
    nvar_slots: u32,
    has_call: bool,
    error: Option<String>,
    /// Offset of each `Func::slots` entry from the bottom of the aggregate area.
    agg_off: Vec<u32>,
    /// Size of the aggregate area (a multiple of 16).
    agg_bytes: u32,
    /// x30 is used as scratch (copy loops): the function needs a frame.
    uses_lr: bool,
    /// Some parameter arrives on the stack (read from x29, so a frame).
    stack_params: bool,
    data_refs: Vec<(usize, u32)>,
}

/// Compile every function of `prog` (tests too when `with_tests`).
pub fn compile(prog: &Program, style: TrapStyle, with_tests: bool) -> Result<Vec<FuncCode>, CodegenError> {
    let mut out = Vec::with_capacity(prog.funcs.len());
    for (i, f) in prog.funcs.iter().enumerate() {
        if f.is_test && !with_tests {
            continue;
        }
        out.push(compile_func(prog, i as FuncId, style)?);
    }
    Ok(out)
}

pub fn compile_func(prog: &Program, id: FuncId, style: TrapStyle) -> Result<FuncCode, CodegenError> {
    let f = &prog.funcs[id as usize];
    let mut g = Gen {
        prog,
        f,
        style,
        code: Vec::with_capacity(64),
        labels: Vec::new(),
        fixes: Vec::new(),
        calls: Vec::new(),
        trap_jumps: Vec::new(),
        homes: vec![Home::None; f.vars.len()],
        depth: 0,
        temp_base: 0,
        max_slot: 0,
        loops: Vec::new(),
        stubs: Vec::new(),
        stub_by_site: HashMap::new(),
        brk_by_kind: HashMap::new(),
        ret_label: 0,
        ret_jumps: Vec::new(),
        used_callee: Vec::new(),
        nvar_slots: 0,
        has_call: false,
        error: None,
        agg_off: Vec::with_capacity(f.slots.len()),
        agg_bytes: 0,
        uses_lr: false,
        stack_params: false,
        data_refs: Vec::new(),
    };
    let mut top = 0u32;
    for sl in &f.slots {
        let a = sl.align.clamp(1, 8);
        let off = (top + a - 1) / a * a;
        g.agg_off.push(off);
        top = off + sl.size;
    }
    g.agg_bytes = (top + 15) & !15;
    if g.agg_bytes > MAX_AGG_BYTES {
        return Err(CodegenError {
            func: f.name.clone(),
            line: f.line,
            construct: "FnDecl(frame size)",
            detail: format!(
                "{} bytes of local aggregates; at most {} per frame",
                g.agg_bytes, MAX_AGG_BYTES
            ),
        });
    }
    g.ret_label = g.new_label();
    g.assign_homes();
    g.stmts(&f.body);
    if f.ret.is_some() && may_fall_through(&f.body) {
        // Falling off the end of a non-void fn.
        let l = g.stub_site(f.noreturn_site);
        g.jump(l);
    }
    let code = g.finish(id);
    if let Some(e) = g.error.take() {
        return Err(CodegenError {
            func: f.name.clone(),
            line: f.line,
            construct: if e.starts_with(GLOBAL_IN_OBJECT) {
                "VarDecl(module, object file)"
            } else {
                "FnDecl(frame or branch range)"
            },
            detail: e,
        });
    }
    Ok(code)
}

impl<'a> Gen<'a> {
    // ----------------------------------------------------------- emission

    fn emit(&mut self, w: u32) {
        self.code.push(w);
    }

    fn new_label(&mut self) -> Label {
        self.labels.push(None);
        self.labels.len() - 1
    }

    fn bind(&mut self, l: Label) {
        self.labels[l] = Some(self.code.len());
    }

    fn jump(&mut self, l: Label) {
        self.fixes.push((self.code.len(), l));
        self.emit(a64::b(0));
    }

    fn bcond(&mut self, c: Cond, l: Label) {
        self.fixes.push((self.code.len(), l));
        self.emit(a64::b_cond(c, 0));
    }

    fn cbz(&mut self, sf: bool, r: Reg, nz: bool, l: Label) {
        self.fixes.push((self.code.len(), l));
        self.emit(if nz { a64::cbnz(sf, r, 0) } else { a64::cbz(sf, r, 0) });
    }

    fn mov_imm(&mut self, sf: bool, rd: Reg, v: u64) {
        let mut buf = Vec::with_capacity(4);
        a64::mov_imm(sf, rd, v, &mut buf);
        self.code.extend_from_slice(&buf);
    }

    fn fail(&mut self, msg: String) {
        if self.error.is_none() {
            self.error = Some(msg);
        }
    }

    // ------------------------------------------------------- trap stubs

    fn stub_site(&mut self, site: SiteId) -> Label {
        match self.style {
            TrapStyle::Jit => {
                if let Some(&l) = self.stub_by_site.get(&site) {
                    return l;
                }
                let l = self.new_label();
                self.stubs.push((l, Stub::Site(site)));
                self.stub_by_site.insert(site, l);
                l
            }
            TrapStyle::Brk => {
                let kind = self.prog.sites[site as usize].kind;
                self.stub_brk(kind)
            }
        }
    }

    fn stub_brk(&mut self, kind: TrapKind) -> Label {
        let k = kind as u32;
        if let Some(&l) = self.brk_by_kind.get(&k) {
            return l;
        }
        let l = self.new_label();
        self.stubs.push((l, Stub::Brk(kind)));
        self.brk_by_kind.insert(k, l);
        l
    }

    // --------------------------------------------------- homes and frame

    fn assign_homes(&mut self) {
        let f = self.f;
        // w counts reads, wr writes; a variable that is never read gets no
        // home at all (its assignments are evaluated for effect only).
        let mut w = vec![0u64; f.vars.len()];
        let mut wr = vec![0u64; f.vars.len()];
        let mut has_call = false;
        weigh_stmts(&f.body, 0, &mut w, &mut wr, &mut has_call);
        for (r, x) in w.iter_mut().zip(&wr) {
            if *r > 0 {
                *r += x;
            }
        }
        self.has_call = has_call;
        self.stack_params = Program::stack_args(&Program::arg_regs(f)).1 > 0;
        let mut order: Vec<usize> = (0..f.vars.len()).collect();
        order.sort_by(|&a, &b| w[b].cmp(&w[a]).then(a.cmp(&b)));
        let mut callee = CALLEE_SAVED.iter().copied();
        let mut slots = 0u32;
        if !has_call {
            // Leaf: parameters stay in their argument registers; the free
            // argument registers are homes too.
            // An F64 parameter arrives in a d register; it gets a home like
            // any other variable and the prologue moves it there.
            let regs = Program::arg_regs(f);
            let nint = regs.iter().filter(|r| !r.0).count();
            let mut caller: Vec<Reg> = ((nint as Reg)..8).collect();
            caller.reverse();
            for (p, &(fp, k)) in regs.iter().enumerate() {
                if !fp && k < 8 {
                    self.homes[p] = Home::Reg(k as Reg);
                }
            }
            for &v in &order {
                if (v < f.nparams && !regs[v].0 && regs[v].1 < 8) || w[v] == 0 {
                    continue;
                }
                self.homes[v] = if let Some(r) = caller.pop() {
                    Home::Reg(r)
                } else if let Some(r) = callee.next() {
                    self.used_callee.push(r);
                    Home::Reg(r)
                } else {
                    slots += 1;
                    Home::Slot(8 * (slots - 1))
                };
            }
        } else {
            for &v in &order {
                if w[v] == 0 {
                    continue;
                }
                self.homes[v] = if let Some(r) = callee.next() {
                    self.used_callee.push(r);
                    Home::Reg(r)
                } else {
                    slots += 1;
                    Home::Slot(8 * (slots - 1))
                };
            }
        }
        self.nvar_slots = slots;
        if 8 * slots > MAX_SLOT_BYTES {
            self.fail(format!("more than {} variables spill to the frame", MAX_SLOT_BYTES / 8));
        }
        self.temp_base = 8 * (slots + self.used_callee.len() as u32);
    }

    fn slot_of_temp(&mut self, i: usize) -> u32 {
        if i + 1 > self.max_slot {
            self.max_slot = i + 1;
        }
        let off = self.temp_base + 8 * i as u32;
        if off > MAX_SLOT_BYTES {
            self.fail(format!("frame larger than {} bytes", MAX_SLOT_BYTES));
            return 0;
        }
        off
    }

    fn ldr_slot(&mut self, rt: Reg, off: u32) {
        self.emit(a64::ldr_x(rt, SP, off));
    }

    fn str_slot(&mut self, rt: Reg, off: u32) {
        self.emit(a64::str_x(rt, SP, off));
    }

    /// Assemble prologue + body + epilogue + stubs, and resolve local branches.
    fn finish(&mut self, id: FuncId) -> FuncCode {
        let f = self.f;
        let frame = self.has_call
            || !self.used_callee.is_empty()
            || self.nvar_slots > 0
            || self.max_slot > 0
            || self.agg_bytes > 0
            || self.uses_lr
            || self.stack_params;
        // Drop a trailing `b ret` that would jump to the very next word.
        if let Some(&last) = self.ret_jumps.last() {
            if last + 1 == self.code.len() {
                let old_len = self.code.len();
                self.code.pop();
                self.ret_jumps.pop();
                self.fixes.retain(|&(p, _)| p != last);
                for l in self.labels.iter_mut() {
                    if *l == Some(old_len) {
                        *l = Some(old_len - 1);
                    }
                }
            }
        }
        // Epilogue.
        self.bind(self.ret_label);
        let ncallee = self.used_callee.len();
        let nbytes = 8 * (self.nvar_slots as usize + ncallee + self.max_slot);
        let frame_bytes = ((nbytes + 15) & !15) + self.agg_bytes as usize;
        if frame {
            let base = 8 * self.nvar_slots;
            let mut k = 0;
            while k < ncallee {
                let off = base + 8 * k as u32;
                if k + 1 < ncallee && off <= 504 {
                    self.emit(a64::ldp_x(self.used_callee[k], self.used_callee[k + 1], SP, off as i32));
                    k += 2;
                } else {
                    self.emit(a64::ldr_x(self.used_callee[k], SP, off));
                    k += 1;
                }
            }
            self.emit(a64::mov_sp(SP, 29));
            self.emit(a64::ldp_x_post(29, 30, SP, 16));
            self.emit(a64::ret());
        } else {
            // Frameless: every `b ret` becomes `ret` itself.
            let jumps = std::mem::take(&mut self.ret_jumps);
            for p in jumps {
                self.code[p] = a64::ret();
                self.fixes.retain(|&(q, _)| q != p);
            }
            self.emit(a64::ret());
        }
        // Trap stubs.
        let stubs = std::mem::take(&mut self.stubs);
        for (l, stub) in stubs {
            self.bind(l);
            match stub {
                Stub::Site(site) => {
                    self.mov_imm(false, 1, site as u64);
                    self.trap_jumps.push(self.code.len());
                    self.emit(a64::b(0));
                }
                Stub::AssertEq(site, ra, rb) => {
                    self.emit(a64::mov(true, X8, rb));
                    self.emit(a64::mov(true, 2, ra));
                    self.emit(a64::mov(true, 3, X8));
                    self.mov_imm(false, 1, site as u64);
                    self.trap_jumps.push(self.code.len());
                    self.emit(a64::b(0));
                }
                Stub::Brk(kind) => self.emit(a64::brk(kind as u32)),
            }
        }
        // Resolve local branches.
        let fixes = std::mem::take(&mut self.fixes);
        for (pos, l) in fixes {
            let target = match self.labels[l] {
                Some(t) => t,
                None => {
                    self.fail("internal: unbound label".into());
                    continue;
                }
            };
            let off = target as i64 - pos as i64;
            let w = self.code[pos];
            if w & 0x7C00_0000 == 0x1400_0000 {
                if !(-(1 << 25)..(1 << 25)).contains(&off) {
                    self.fail("branch out of range".into());
                }
                self.code[pos] = a64::b(off as i32);
            } else {
                if !(-(1 << 18)..(1 << 18)).contains(&off) {
                    self.fail("conditional branch out of range (function too large)".into());
                }
                self.code[pos] = (w & !(0x7ffff << 5)) | ((off as u32 & 0x7ffff) << 5);
            }
        }
        // Prologue.
        let mut pro: Vec<u32> = Vec::new();
        if frame {
            pro.push(a64::stp_x_pre(29, 30, SP, -16));
            pro.push(a64::mov_sp(29, SP));
            if frame_bytes > 0 {
                if frame_bytes < 4096 {
                    pro.push(a64::sub_imm(true, SP, SP, frame_bytes as u32));
                } else if frame_bytes <= (MAX_SLOT_BYTES + 16 + self.agg_bytes) as usize {
                    a64::mov_imm(true, X16, frame_bytes as u64, &mut pro);
                    pro.push(a64::addsub_ext(true, true, false, SP, SP, X16, Ext::Uxtx, 0));
                } else {
                    self.fail(format!("frame larger than {} bytes", MAX_SLOT_BYTES));
                }
            }
            let base = 8 * self.nvar_slots;
            let mut k = 0;
            while k < ncallee {
                let off = base + 8 * k as u32;
                if k + 1 < ncallee && off <= 504 {
                    pro.push(a64::stp_x(self.used_callee[k], self.used_callee[k + 1], SP, off as i32));
                    k += 2;
                } else {
                    pro.push(a64::str_x(self.used_callee[k], SP, off));
                    k += 1;
                }
            }
        }
        // Parameters: normalise narrow ones and move them to their homes.
        let regs = Program::arg_regs(f);
        let (stack, _) = Program::stack_args(&regs);
        for p in 0..f.nparams {
            let ty = f.vars[p].ty;
            let (fp, k) = regs[p];
            if let Some(s) = stack[p] {
                // A stack parameter: one full word above the saved x29/x30
                // (an F64 is its bit pattern, homed in an x register too).
                let at = 16 + 8 * s;
                let ext = if fp { None } else { narrow_ext(ty) };
                match self.homes[p] {
                    Home::None => {}
                    Home::Reg(h) => {
                        pro.push(a64::ldr_x(h, FP, at));
                        if let Some(e) = ext {
                            pro.push(extend(h, h, e));
                        }
                    }
                    Home::Slot(off) => {
                        pro.push(a64::ldr_x(X16, FP, at));
                        if let Some(e) = ext {
                            pro.push(extend(X16, X16, e));
                        }
                        pro.push(a64::str_x(X16, SP, off));
                    }
                }
                continue;
            }
            if fp {
                // AAPCS64: the k-th F64 argument is in d_k.
                match self.homes[p] {
                    Home::None => {}
                    Home::Reg(h) => pro.push(a64::fmov_x_d(h, k as Reg)),
                    Home::Slot(off) => pro.push(a64::str_d(k as Reg, SP, off)),
                }
                continue;
            }
            let src = k as Reg;
            let ext = narrow_ext(ty);
            match self.homes[p] {
                Home::None => {}
                Home::Reg(h) => {
                    if let Some(e) = ext {
                        pro.push(extend(h, src, e));
                    } else if h != src {
                        pro.push(a64::mov(true, h, src));
                    }
                }
                Home::Slot(off) => {
                    if let Some(e) = ext {
                        pro.push(extend(src, src, e));
                    }
                    pro.push(a64::str_x(src, SP, off));
                }
            }
        }
        let shift = pro.len();
        let mut code = pro;
        code.extend_from_slice(&self.code);
        FuncCode {
            func: id,
            code,
            calls: self.calls.iter().map(|&(p, c)| (p + shift, c)).collect(),
            trap_jumps: self.trap_jumps.iter().map(|&p| p + shift).collect(),
            data_refs: self.data_refs.iter().map(|&(p, k)| (p + shift, k)).collect(),
        }
    }

    // ------------------------------------------------------- temp stack

    fn alloc(&mut self) -> usize {
        self.depth += 1;
        self.depth - 1
    }

    fn release(&mut self, v: V) {
        if let V::Temp(i) = v {
            debug_assert_eq!(i + 1, self.depth, "temp stack is LIFO");
            self.depth -= 1;
        }
    }

    /// The register holding `v`; spilled temps and constants go to `scratch`.
    /// A zero constant is the zero register.
    fn use_(&mut self, v: V, scratch: Reg, ty: Ty) -> Reg {
        match v {
            V::Reg(r) => r,
            V::Temp(i) if i < TEMP_REGS => 9 + i as Reg,
            V::Temp(i) => {
                let off = self.slot_of_temp(i);
                self.ldr_slot(scratch, off);
                scratch
            }
            V::Const(0) => ZR,
            V::Const(c) => {
                self.mov_imm(ty.is64(), scratch, canon(c, ty));
                scratch
            }
        }
    }

    /// Like `use_` but never the zero register (for forms where 31 is SP).
    fn use_nz(&mut self, v: V, scratch: Reg, ty: Ty) -> Reg {
        if v == V::Const(0) {
            self.emit(a64::movz(true, scratch, 0, 0));
            return scratch;
        }
        self.use_(v, scratch, ty)
    }

    /// Destination register: the requested one, or a new temp.
    fn dest(&mut self, dst: Option<Reg>) -> (Reg, Option<usize>) {
        match dst {
            Some(r) => (r, None),
            None => {
                let i = self.alloc();
                if i < TEMP_REGS {
                    (9 + i as Reg, Some(i))
                } else {
                    (X16, Some(i))
                }
            }
        }
    }

    fn done(&mut self, d: Reg, t: Option<usize>) -> V {
        match t {
            None => V::Reg(d),
            Some(i) => {
                if i >= TEMP_REGS {
                    let off = self.slot_of_temp(i);
                    self.str_slot(X16, off);
                }
                V::Temp(i)
            }
        }
    }

    // ------------------------------------------------------- statements

    fn stmts(&mut self, ss: &[Stmt]) {
        for s in ss {
            self.stmt(s);
            debug_assert_eq!(self.depth, 0);
        }
    }

    fn stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Assign { var, value } => match self.homes[*var as usize] {
                Home::Reg(h) => self.eval_into(value, h),
                Home::Slot(off) => {
                    let v = self.eval(value);
                    let r = self.use_(v, X16, value.ty);
                    self.str_slot(r, off);
                    self.release(v);
                }
                Home::None => {
                    let v = self.eval(value);
                    self.release(v);
                }
            },
            Stmt::If { cond, then, els } => {
                let l_else = self.new_label();
                self.branch(cond, l_else, false);
                self.stmts(then);
                if els.is_empty() {
                    self.bind(l_else);
                } else {
                    let l_end = self.new_label();
                    if !ends_in_jump(then) {
                        self.jump(l_end);
                    }
                    self.bind(l_else);
                    self.stmts(els);
                    self.bind(l_end);
                }
            }
            Stmt::While { cond, body, step } => {
                let l_body = self.new_label();
                let l_step = self.new_label();
                let l_cond = self.new_label();
                let l_end = self.new_label();
                self.jump(l_cond);
                self.bind(l_body);
                self.loops.push((l_step, l_end));
                self.stmts(body);
                self.bind(l_step);
                self.stmts(step);
                self.loops.pop();
                self.bind(l_cond);
                self.branch(cond, l_body, true);
                self.bind(l_end);
            }
            Stmt::Break => {
                let (_, l_end) = *self.loops.last().expect("break outside a loop is rejected by lowering");
                self.jump(l_end);
            }
            Stmt::Continue => {
                let (l_step, _) = *self.loops.last().expect("continue outside a loop is rejected by lowering");
                self.jump(l_step);
            }
            Stmt::Return(e) => {
                if let Some(e) = e {
                    self.eval_into(e, X0);
                    if e.ty == Ty::F64 {
                        // AAPCS64 returns an F64 in d0.
                        self.emit(a64::fmov_d_x(0, X0));
                    }
                }
                self.ret_jumps.push(self.code.len());
                let l = self.ret_label;
                self.jump(l);
            }
            Stmt::Eval(e) => {
                if let ExprKind::Call { func, args } = &e.kind {
                    self.call(*func, args, None, false);
                } else {
                    let v = self.eval(e);
                    self.release(v);
                }
            }
            Stmt::Assert { cond, site } => {
                let l = self.stub_site(*site);
                self.branch(cond, l, false);
            }
            Stmt::AssertEq { lhs, rhs, site } => {
                let ty = lhs.ty;
                let a = self.eval(lhs);
                let b = self.eval(rhs);
                let ra = self.use_(a, X16, ty);
                let rb = self.use_(b, X17, ty);
                if ty == Ty::F64 {
                    // IEEE equality: NaN is unequal to itself, -0.0 == 0.0.
                    self.emit(a64::fmov_d_x(D16, ra));
                    self.emit(a64::fmov_d_x(D17, rb));
                    self.emit(a64::fcmp(D16, D17));
                } else {
                    self.emit(a64::cmp(ty.is64(), ra, rb));
                }
                self.release(b);
                self.release(a);
                let l = match self.style {
                    TrapStyle::Jit => {
                        let l = self.new_label();
                        self.stubs.push((l, Stub::AssertEq(*site, ra, rb)));
                        l
                    }
                    TrapStyle::Brk => self.stub_brk(TrapKind::AssertEq),
                };
                self.bcond(Cond::Ne, l);
            }
            Stmt::Store { addr, off, value } => {
                let ty = value.ty;
                if let ExprKind::Slot(k) = addr.kind {
                    if let Some(disp) = self.fp_disp(k, *off) {
                        let v = self.eval(value);
                        let rv = self.use_(v, X17, ty);
                        let (size, opc) = ldst_op(ty, false);
                        self.emit(a64::ldst_unscaled(size, opc, rv, FP, disp));
                        self.release(v);
                        return;
                    }
                }
                let a = self.eval(addr);
                let v = self.eval(value);
                let ra = self.use_nz(a, X16, Ty::Ptr);
                let rv = self.use_(v, X17, ty);
                self.ldst(ty, false, rv, ra, *off);
                self.release(v);
                self.release(a);
            }
            Stmt::Copy { dst, src, size } => {
                let d = self.eval(dst);
                let s = self.eval(src);
                let rd = self.use_nz(d, X16, Ty::Ptr);
                let rs = self.use_nz(s, X17, Ty::Ptr);
                self.release(s);
                self.release(d);
                // rd is never x17 and rs never x16 (homes and temps are
                // neither), so these two moves cannot clobber each other.
                if rd != X16 {
                    self.emit(a64::mov(true, X16, rd));
                }
                if rs != X17 {
                    self.emit(a64::mov(true, X17, rs));
                }
                self.copy_x16_x17(*size);
            }
        }
    }

    // ----------------------------------------------------------- memory

    /// x29-relative displacement of `off` bytes into slot `k`, when it fits
    /// the unscaled form.
    fn fp_disp(&self, k: u32, off: u32) -> Option<i32> {
        let disp = self.agg_off[k as usize] as i64 + off as i64 - self.agg_bytes as i64;
        if (-256..256).contains(&disp) {
            Some(disp as i32)
        } else {
            None
        }
    }

    /// `d = address of slot k` (x29 minus its distance below the frame pointer).
    fn slot_addr(&mut self, d: Reg, k: u32) {
        let n = self.agg_bytes - self.agg_off[k as usize];
        if n < 4096 {
            self.emit(a64::sub_imm(true, d, FP, n));
        } else {
            self.emit(a64::addsub_imm(true, true, false, d, FP, n >> 12, true));
            if n & 0xfff != 0 {
                self.emit(a64::sub_imm(true, d, d, n & 0xfff));
            }
        }
    }

    /// One load into, or store from, `rt` of a `ty` at `base + off`.
    fn ldst(&mut self, ty: Ty, load: bool, rt: Reg, base: Reg, off: u32) {
        let (size, opc) = ldst_op(ty, load);
        if a64::uimm_fits(size, off) {
            self.emit(a64::ldst_uimm(size, opc, rt, base, off));
        } else if off < 256 {
            self.emit(a64::ldst_unscaled(size, opc, rt, base, off as i32));
        } else {
            self.mov_imm(true, X8, off as u64);
            self.emit(a64::ldst_reg(size, opc, rt, base, X8, false));
        }
    }

    /// Copy `size` bytes from [x17] to [x16]; clobbers x8, x16, x17 (and x30
    /// for a loop).
    fn copy_x16_x17(&mut self, size: u32) {
        let mut rem = size;
        if size > 64 {
            let n8 = size & !7;
            if n8 < 4096 {
                self.emit(a64::add_imm(true, LR, X16, n8));
            } else {
                self.mov_imm(true, LR, n8 as u64);
                self.emit(a64::add(true, LR, X16, LR));
            }
            let top = self.new_label();
            self.bind(top);
            self.emit(a64::ldst_post(3, a64::LD, X8, X17, 8));
            self.emit(a64::ldst_post(3, a64::ST, X8, X16, 8));
            self.emit(a64::cmp(true, X16, LR));
            self.bcond(Cond::Ne, top);
            self.uses_lr = true;
            rem = size - n8;
        }
        let mut o = 0u32;
        for chunk in [8u32, 4, 2, 1] {
            while rem >= chunk {
                let sz = a64::size_log2(chunk);
                self.emit(a64::ldst_uimm(sz, a64::LD, X8, X17, o));
                self.emit(a64::ldst_uimm(sz, a64::ST, X8, X16, o));
                o += chunk;
                rem -= chunk;
            }
        }
    }

    // ------------------------------------------------------ expressions

    fn eval(&mut self, e: &Expr) -> V {
        match &e.kind {
            ExprKind::Const(c) => V::Const(*c),
            ExprKind::Var(v) => match self.homes[*v as usize] {
                Home::Reg(r) => V::Reg(r),
                Home::Slot(off) => {
                    let (d, t) = self.dest(None);
                    self.ldr_slot(d, off);
                    self.done(d, t)
                }
                Home::None => V::Const(0),
            },
            _ => self.compute(e, None),
        }
    }

    fn eval_into(&mut self, e: &Expr, r: Reg) {
        match &e.kind {
            ExprKind::Const(c) => self.mov_imm(e.ty.is64(), r, canon(*c, e.ty)),
            ExprKind::Var(v) => match self.homes[*v as usize] {
                Home::Reg(h) => {
                    if h != r {
                        self.emit(a64::mov(true, r, h));
                    }
                }
                Home::Slot(off) => self.ldr_slot(r, off),
                Home::None => self.emit(a64::movz(true, r, 0, 0)),
            },
            _ => {
                let v = self.compute(e, Some(r));
                debug_assert_eq!(v, V::Reg(r));
            }
        }
    }

    fn compute(&mut self, e: &Expr, dst: Option<Reg>) -> V {
        let ty = e.ty;
        match &e.kind {
            ExprKind::Const(_) | ExprKind::Var(_) => {
                let v = self.eval(e);
                match dst {
                    None => v,
                    Some(r) => {
                        let ra = self.use_(v, X16, ty);
                        self.release(v);
                        if ra != r {
                            self.emit(a64::mov(true, r, ra));
                        }
                        V::Reg(r)
                    }
                }
            }
            ExprKind::Arith { op, lhs, rhs, site } => self.arith(ty, *op, lhs, rhs, *site, dst),
            ExprKind::Cmp { op, lhs, rhs } => {
                let c = self.compare(*op, lhs, rhs);
                let (d, t) = self.dest(dst);
                self.emit(a64::cset(false, d, c));
                self.done(d, t)
            }
            ExprKind::And(..) | ExprKind::Or(..) => {
                let l_false = self.new_label();
                let l_end = self.new_label();
                self.branch(e, l_false, false);
                let (d, t) = self.dest(dst);
                self.emit(a64::movz(false, d, 1, 0));
                if t.map_or(false, |i| i >= TEMP_REGS) {
                    let off = self.slot_of_temp(t.unwrap());
                    self.str_slot(X16, off);
                }
                self.jump(l_end);
                self.bind(l_false);
                self.emit(a64::movz(false, d, 0, 0));
                let v = self.done(d, t);
                self.bind(l_end);
                v
            }
            ExprKind::Not(x) => {
                let v = self.eval(x);
                let ra = self.use_(v, X16, Ty::Bool);
                self.release(v);
                let (d, t) = self.dest(dst);
                self.emit(a64::logic_imm(false, LogOp::Eor, d, ra, 1).expect("1 is a bitmask immediate"));
                self.done(d, t)
            }
            ExprKind::BitNot(x) => {
                let v = self.eval(x);
                let ra = self.use_(v, X16, ty);
                self.release(v);
                let (d, t) = self.dest(dst);
                match ty {
                    Ty::U8 => self.emit(a64::logic_imm(false, LogOp::Eor, d, ra, 0xff).unwrap()),
                    Ty::U16 => self.emit(a64::logic_imm(false, LogOp::Eor, d, ra, 0xffff).unwrap()),
                    _ => self.emit(a64::mvn(ty.is64(), d, ra)),
                }
                self.done(d, t)
            }
            ExprKind::Call { func, args } => self.call(*func, args, dst, true),
            ExprKind::FArith { op, lhs, rhs } => {
                let a = self.eval(lhs);
                let b = self.eval(rhs);
                let ra = self.use_(a, X16, ty);
                let rb = self.use_(b, X17, ty);
                self.emit(a64::fmov_d_x(D16, ra));
                self.emit(a64::fmov_d_x(D17, rb));
                self.emit(match op {
                    FOp::Add => a64::fadd(D16, D16, D17),
                    FOp::Sub => a64::fsub(D16, D16, D17),
                    FOp::Mul => a64::fmul(D16, D16, D17),
                    FOp::Div => a64::fdiv(D16, D16, D17),
                });
                self.release(b);
                self.release(a);
                let (d, t) = self.dest(dst);
                self.emit(a64::fmov_x_d(d, D16));
                self.done(d, t)
            }
            ExprKind::FNeg(x) => {
                let v = self.eval(x);
                let ra = self.use_(v, X16, ty);
                self.emit(a64::fmov_d_x(D16, ra));
                self.emit(a64::fneg(D16, D16));
                self.release(v);
                let (d, t) = self.dest(dst);
                self.emit(a64::fmov_x_d(d, D16));
                self.done(d, t)
            }
            ExprKind::IntToFloat(x) => {
                // A narrow integer is canonical in its W register (sign- or
                // zero-extended to 32 bits), so the W form converts it.
                let from = x.ty;
                let v = self.eval(x);
                let ra = self.use_(v, X16, from);
                if from.signed() {
                    self.emit(a64::scvtf(from.is64(), D16, ra));
                } else {
                    self.emit(a64::ucvtf(from.is64(), D16, ra));
                }
                self.release(v);
                let (d, t) = self.dest(dst);
                self.emit(a64::fmov_x_d(d, D16));
                self.done(d, t)
            }
            ExprKind::FloatToInt { arg, site } => {
                // Traps when x <= lo (x < lo for i64, whose lower bound
                // -2^63 is itself a double) or x >= hi; both bounds are exact
                // doubles. Each trap condition (MI, LS, GE) is false after an
                // unordered FCMP (NZCV = 0011), so a NaN does not trap and
                // FCVTZ* turns it into 0 -- what Zig 0.16 Debug does. FCVTZ*
                // truncates toward zero, giving the canonical W form.
                let v = self.eval(arg);
                let ra = self.use_(v, X16, Ty::F64);
                self.emit(a64::fmov_d_x(D16, ra));
                self.release(v);
                let l = self.stub_site(*site);
                let (lo, lo_incl) = if ty == Ty::I64 {
                    (-9223372036854775808.0f64, true)
                } else {
                    (ty.min() as f64 - 1.0, false)
                };
                let hi = (ty.max() + 1) as f64;
                self.mov_imm(true, X17, lo.to_bits());
                self.emit(a64::fmov_d_x(D17, X17));
                self.emit(a64::fcmp(D16, D17));
                let below = if lo_incl { Cond::Mi } else { Cond::Ls };
                self.bcond(below, l);
                self.mov_imm(true, X17, hi.to_bits());
                self.emit(a64::fmov_d_x(D17, X17));
                self.emit(a64::fcmp(D16, D17));
                self.bcond(Cond::Ge, l);
                let (d, t) = self.dest(dst);
                if ty.signed() {
                    self.emit(a64::fcvtzs(ty.is64(), d, D16));
                } else {
                    self.emit(a64::fcvtzu(ty.is64(), d, D16));
                }
                self.done(d, t)
            }
            ExprKind::Widen(x) => {
                let from = x.ty;
                let v = self.eval(x);
                if ty.is64() && !from.is64() {
                    let ra = self.use_(v, X16, from);
                    self.release(v);
                    let (d, t) = self.dest(dst);
                    if from.signed() {
                        self.emit(a64::sxtw(d, ra));
                    } else {
                        self.emit(a64::mov(false, d, ra));
                    }
                    self.done(d, t)
                } else {
                    match dst {
                        None => v,
                        Some(r) => {
                            let ra = self.use_(v, X16, from);
                            self.release(v);
                            if ra != r {
                                self.emit(a64::mov(true, r, ra));
                            }
                            V::Reg(r)
                        }
                    }
                }
            }
            ExprKind::Cast { arg, site } => {
                let from = arg.ty;
                let wide = from.is64();
                let v = self.eval(arg);
                let ra = self.use_nz(v, X16, from);
                if *site != 0 && !ty.can_widen_from(from) {
                    // The operand is canonical in a W register (32 bits or
                    // less) or an X register; it fits in `ty` when extending
                    // its low `ty.bits()` bits gives it back, and, if `ty` is
                    // signed and the operand a full-register unsigned value
                    // (or the widths match, so only the sign differs), when it
                    // is not negative read as signed.
                    let l = self.stub_site(*site);
                    let reg_bits = if wide { 64 } else { 32 };
                    if ty.bits() < reg_bits {
                        let ext = match ty {
                            Ty::U8 => Ext::Uxtb,
                            Ty::U16 => Ext::Uxth,
                            Ty::U32 => Ext::Uxtw,
                            Ty::I8 => Ext::Sxtb,
                            Ty::I16 => Ext::Sxth,
                            Ty::I32 => Ext::Sxtw,
                            _ => unreachable!("cast to {}", ty.name()),
                        };
                        self.emit(a64::cmp_ext(wide, ra, ra, ext));
                        self.bcond(Cond::Ne, l);
                        if !from.signed() && ty.signed() && from.bits() == reg_bits {
                            self.emit(a64::cmp_imm(wide, ra, 0));
                            self.bcond(Cond::Lt, l);
                        }
                    } else {
                        self.emit(a64::cmp_imm(wide, ra, 0));
                        self.bcond(Cond::Lt, l);
                    }
                }
                self.release(v);
                let (d, t) = self.dest(dst);
                if ty.is64() {
                    if wide {
                        self.emit(a64::mov(true, d, ra));
                    } else if from.signed() {
                        self.emit(a64::sxtw(d, ra));
                    } else {
                        self.emit(a64::mov(false, d, ra));
                    }
                } else if let Some(e) = narrow_ext(ty) {
                    self.emit(extend(d, ra, e));
                } else {
                    self.emit(a64::mov(false, d, ra));
                }
                self.done(d, t)
            }
            ExprKind::Slot(k) => {
                let (d, t) = self.dest(dst);
                self.slot_addr(d, *k);
                self.done(d, t)
            }
            ExprKind::Data(k) => {
                let (d, t) = self.dest(dst);
                self.data_refs.push((self.code.len(), *k));
                self.emit(a64::adrp(d, 0));
                self.emit(a64::add_imm(true, d, d, 0));
                self.done(d, t)
            }
            ExprKind::Global(k) => {
                // The JIT appends one more blob after `Program::data`: a
                // table of the absolute address of every global, which live
                // in writable memory outside the executable image. An object
                // file has no such table (no __DATA section yet).
                if self.style != TrapStyle::Jit {
                    self.fail(format!("{} (no writable data section)", GLOBAL_IN_OBJECT));
                }
                if *k >= 4096 {
                    self.fail("more than 4096 module-level vars".into());
                }
                let (d, t) = self.dest(dst);
                self.data_refs.push((self.code.len(), self.prog.data.len() as u32));
                self.emit(a64::adrp(d, 0));
                self.emit(a64::add_imm(true, d, d, 0));
                self.emit(a64::ldr_x(d, d, (*k).min(4095) * 8));
                self.done(d, t)
            }
            ExprKind::Load { addr, off } => {
                if let ExprKind::Slot(k) = addr.kind {
                    if let Some(disp) = self.fp_disp(k, *off) {
                        let (d, t) = self.dest(dst);
                        let (size, opc) = ldst_op(ty, true);
                        self.emit(a64::ldst_unscaled(size, opc, d, FP, disp));
                        return self.done(d, t);
                    }
                }
                let a = self.eval(addr);
                let ra = self.use_nz(a, X16, Ty::Ptr);
                self.release(a);
                let (d, t) = self.dest(dst);
                self.ldst(ty, true, d, ra, *off);
                self.done(d, t)
            }
            ExprKind::Offset { base, idx, scale } => {
                let b = self.eval(base);
                if let ExprKind::Const(c) = idx.kind {
                    let bytes = (c as u64).wrapping_mul(*scale as u64);
                    let rb = self.use_nz(b, X16, Ty::Ptr);
                    self.release(b);
                    let (d, t) = self.dest(dst);
                    if bytes < 4096 {
                        if bytes != 0 || d != rb {
                            self.emit(a64::add_imm(true, d, rb, bytes as u32));
                        }
                    } else {
                        self.mov_imm(true, X17, bytes);
                        self.emit(a64::add(true, d, rb, X17));
                    }
                    return self.done(d, t);
                }
                let i = self.eval(idx);
                let rb = self.use_(b, X16, Ty::Ptr);
                let ri = self.use_(i, X17, Ty::U64);
                self.release(i);
                self.release(b);
                let (d, t) = self.dest(dst);
                if scale.is_power_of_two() {
                    self.emit(a64::addsub_reg(true, false, false, d, rb, ri, Shift::Lsl, scale.trailing_zeros()));
                } else {
                    self.mov_imm(true, X8, *scale as u64);
                    self.emit(a64::madd(true, d, ri, X8, rb));
                }
                self.done(d, t)
            }
            ExprKind::Bounds { idx, len, site } => {
                let a = self.eval(idx);
                let b = self.eval(len);
                let ra = self.use_(a, X16, Ty::U64);
                match b {
                    V::Const(c) if (0..4096).contains(&c) && ra != ZR => self.emit(a64::cmp_imm(true, ra, c as u32)),
                    _ => {
                        let rb = self.use_(b, X17, Ty::U64);
                        self.emit(a64::cmp(true, ra, rb));
                    }
                }
                let l = self.stub_site(*site);
                self.bcond(Cond::Hs, l);
                self.release(b);
                self.release(a);
                let (d, t) = self.dest(dst);
                if d != ra {
                    self.emit(a64::mov(true, d, ra));
                }
                self.done(d, t)
            }
            ExprKind::Seq { stmts, value } => {
                // Store, Copy and Eval only use x8/x16/x17/x30 as scratch and
                // allocate temps above the live ones; a call saves those.
                for s in stmts {
                    self.stmt(s);
                }
                match dst {
                    Some(r) => {
                        self.eval_into(value, r);
                        V::Reg(r)
                    }
                    None => self.eval(value),
                }
            }
            ExprKind::Select { cond, then, els } => {
                // Like And/Or: the destination is taken after the condition,
                // and each arm is evaluated aside (its temps sit above the
                // destination), then moved in; a spilled destination (x16)
                // is stored on both paths.
                let l_else = self.new_label();
                let l_end = self.new_label();
                self.branch(cond, l_else, false);
                let (d, t) = self.dest(dst);
                let spilled = t.map_or(false, |i| i >= TEMP_REGS);
                self.select_arm(then, d, ty);
                if spilled {
                    let off = self.slot_of_temp(t.unwrap());
                    self.str_slot(X16, off);
                }
                self.jump(l_end);
                self.bind(l_else);
                self.select_arm(els, d, ty);
                let v = self.done(d, t);
                self.bind(l_end);
                v
            }
        }
    }

    /// One arm of a `Select`, evaluated and moved into `d`.
    fn select_arm(&mut self, e: &Expr, d: Reg, ty: Ty) {
        let v = self.eval(e);
        let r = self.use_(v, X16, ty);
        self.release(v);
        if r != d {
            self.emit(a64::mov(true, d, r));
        }
    }

    /// Emit the flag-setting compare of `lhs op rhs` and return the condition
    /// that holds when the comparison is true.
    fn compare(&mut self, op: CmpOp, lhs: &Expr, rhs: &Expr) -> Cond {
        let ty = lhs.ty;
        let s = ty.is64();
        let a = self.eval(lhs);
        let b = self.eval(rhs);
        if ty == Ty::F64 {
            let rx = self.use_(a, X16, ty);
            let ry = self.use_(b, X17, ty);
            self.emit(a64::fmov_d_x(D16, rx));
            self.emit(a64::fmov_d_x(D17, ry));
            self.emit(a64::fcmp(D16, D17));
            self.release(b);
            self.release(a);
            return fcond_for(op);
        }
        let (mut x, mut y, mut op2) = (a, b, op);
        if matches!(x, V::Const(_)) && !matches!(y, V::Const(_)) {
            std::mem::swap(&mut x, &mut y);
            op2 = op2.swap();
        }
        match y {
            V::Const(c) if !matches!(x, V::Const(_)) && (0..4096).contains(&c) => {
                let rx = self.use_(x, X16, ty);
                self.emit(a64::cmp_imm(s, rx, c as u32));
            }
            V::Const(c) if !matches!(x, V::Const(_)) && (-4095..0).contains(&c) => {
                let rx = self.use_(x, X16, ty);
                self.emit(a64::cmn_imm(s, rx, (-c) as u32));
            }
            _ => {
                let rx = self.use_(x, X16, ty);
                let ry = self.use_(y, X17, ty);
                self.emit(a64::cmp(s, rx, ry));
            }
        }
        self.release(b);
        self.release(a);
        cond_for(op2, ty)
    }

    /// Jump to `target` when `e` evaluates to `when`.
    fn branch(&mut self, e: &Expr, target: Label, when: bool) {
        match &e.kind {
            ExprKind::Const(c) => {
                if (*c != 0) == when {
                    self.jump(target);
                }
            }
            ExprKind::Not(x) => self.branch(x, target, !when),
            ExprKind::And(x, y) => {
                if !when {
                    self.branch(x, target, false);
                    self.branch(y, target, false);
                } else {
                    let skip = self.new_label();
                    self.branch(x, skip, false);
                    self.branch(y, target, true);
                    self.bind(skip);
                }
            }
            ExprKind::Or(x, y) => {
                if when {
                    self.branch(x, target, true);
                    self.branch(y, target, true);
                } else {
                    let skip = self.new_label();
                    self.branch(x, skip, true);
                    self.branch(y, target, false);
                    self.bind(skip);
                }
            }
            ExprKind::Cmp { op, lhs, rhs } => {
                let c = self.compare(*op, lhs, rhs);
                self.bcond(if when { c } else { c.invert() }, target);
            }
            _ => {
                let v = self.eval(e);
                let r = self.use_(v, X16, Ty::Bool);
                self.release(v);
                self.cbz(false, r, when, target);
            }
        }
    }

    /// Call `func`; result into `dst` (or a temp) when `want`.
    fn call(&mut self, func: FuncId, args: &[Expr], dst: Option<Reg>, want: bool) -> V {
        let base = self.depth;
        let mut vals: Vec<(V, Ty)> = Vec::with_capacity(args.len());
        for a in args {
            let v = self.eval(a);
            vals.push((v, a.ty));
        }
        let nsave = base.min(TEMP_REGS);
        for i in 0..nsave {
            let off = self.slot_of_temp(i);
            self.str_slot(9 + i as Reg, off);
        }
        let callee = &self.prog.funcs[func as usize];
        let regs = Program::arg_regs(callee);
        let (stack, nstack) = Program::stack_args(&regs);
        let ret_f64 = callee.ret == Some(Ty::F64);
        // Register arguments first (they touch only x0-x7, d0-d7 and x16),
        // then the stack ones, below sp: x9-x15 still hold the temps.
        for (j, &(v, ty)) in vals.iter().enumerate() {
            let (fp, k) = regs[j];
            if stack[j].is_some() {
                continue;
            }
            if fp {
                // AAPCS64: the k-th F64 argument goes in d_k.
                let r = self.use_(v, X16, ty);
                self.emit(a64::fmov_d_x(k as Reg, r));
                continue;
            }
            let rj = k as Reg;
            match v {
                V::Temp(i) if i < TEMP_REGS => self.emit(a64::mov(true, rj, 9 + i as Reg)),
                V::Temp(i) => {
                    let off = self.slot_of_temp(i);
                    self.ldr_slot(rj, off);
                }
                V::Reg(r) => self.emit(a64::mov(true, rj, r)),
                V::Const(c) => self.mov_imm(ty.is64(), rj, canon(c, ty)),
            }
        }
        // Outgoing stack words, 16-byte aligned at the `bl`.
        let out_bytes = (8 * nstack + 15) & !15;
        if out_bytes > 0 {
            if out_bytes >= 4096 {
                self.fail(format!("{} stack argument words; at most 510", nstack));
            }
            self.emit(a64::sub_imm(true, SP, SP, out_bytes));
            for (j, &(v, ty)) in vals.iter().enumerate() {
                let Some(s) = stack[j] else { continue };
                let r = match v {
                    V::Temp(i) if i < TEMP_REGS => 9 + i as Reg,
                    V::Temp(i) => {
                        // A spilled temp's slot is sp-relative: sp moved.
                        let off = self.slot_of_temp(i) + out_bytes;
                        if off > MAX_SLOT_BYTES {
                            self.fail(format!("frame larger than {} bytes", MAX_SLOT_BYTES));
                        }
                        self.ldr_slot(X16, off);
                        X16
                    }
                    V::Reg(r) => r,
                    V::Const(c) => {
                        self.mov_imm(ty.is64(), X16, canon(c, ty));
                        X16
                    }
                };
                self.str_slot(r, 8 * s);
            }
        }
        for &(v, _) in vals.iter().rev() {
            self.release(v);
        }
        self.calls.push((self.code.len(), func));
        self.emit(a64::bl(0));
        if out_bytes > 0 {
            self.emit(a64::add_imm(true, SP, SP, out_bytes));
        }
        let out = if want {
            let (d, t) = self.dest(dst);
            if ret_f64 {
                self.emit(a64::fmov_x_d(d, 0));
            } else if d != X0 {
                self.emit(a64::mov(true, d, X0));
            }
            self.done(d, t)
        } else {
            V::Const(0)
        };
        for i in 0..nsave {
            let off = self.slot_of_temp(i);
            self.ldr_slot(9 + i as Reg, off);
        }
        out
    }

    fn normalize(&mut self, d: Reg, ty: Ty) {
        if let Some(e) = narrow_ext(ty) {
            self.emit(extend(d, d, e));
        }
    }

    /// Trap unless the 32-bit value in `d` is a canonical value of narrow `ty`.
    fn check_narrow(&mut self, d: Reg, ty: Ty, site: SiteId) {
        let e = narrow_ext(ty).expect("narrow type");
        self.emit(a64::cmp_ext(false, d, d, e));
        let l = self.stub_site(site);
        self.bcond(Cond::Ne, l);
    }

    fn arith(&mut self, ty: Ty, op: ArithOp, lhs: &Expr, rhs: &Expr, site: SiteId, dst: Option<Reg>) -> V {
        let s = ty.is64();
        let narrow = ty.bits() < 32;
        let a = self.eval(lhs);
        let b = self.eval(rhs);
        let (mut x, mut y) = (a, b);
        if op.commutative() && matches!(x, V::Const(_)) && !matches!(y, V::Const(_)) {
            std::mem::swap(&mut x, &mut y);
        }
        let xc = matches!(x, V::Const(_));
        match (op, y) {
            // ---------------------------------------------- add/sub immediate
            (ArithOp::Add | ArithOp::AddW | ArithOp::Sub | ArithOp::SubW, V::Const(c)) if !xc && (0..4096).contains(&c) => {
                let rx = self.use_(x, X16, ty);
                self.release(b);
                self.release(a);
                let (d, t) = self.dest(dst);
                let sub = matches!(op, ArithOp::Sub | ArithOp::SubW);
                let checked = matches!(op, ArithOp::Add | ArithOp::Sub);
                if checked && !narrow {
                    self.emit(a64::addsub_imm(s, sub, true, d, rx, c as u32, false));
                    let l = self.stub_site(site);
                    self.bcond(overflow_cond(ty, sub), l);
                } else {
                    self.emit(a64::addsub_imm(s, sub, false, d, rx, c as u32, false));
                    if narrow {
                        if checked {
                            self.check_narrow(d, ty, site);
                        } else {
                            self.normalize(d, ty);
                        }
                    }
                }
                return self.done(d, t);
            }
            // ---------------------------------------------- logical immediate
            (ArithOp::And | ArithOp::Or | ArithOp::Xor, V::Const(c))
                if !xc && a64::bitmask_imm(canon(c, ty), s).is_some() =>
            {
                let rx = self.use_(x, X16, ty);
                self.release(b);
                self.release(a);
                let (d, t) = self.dest(dst);
                let lop = match op {
                    ArithOp::And => LogOp::And,
                    ArithOp::Or => LogOp::Orr,
                    _ => LogOp::Eor,
                };
                self.emit(a64::logic_imm(s, lop, d, rx, canon(c, ty)).unwrap());
                return self.done(d, t);
            }
            // ---------------------------------------------- constant shift amount
            (ArithOp::Shl | ArithOp::Shr | ArithOp::ShlW | ArithOp::ShrW, V::Const(c)) => {
                let bits = ty.bits() as i128;
                let checked = matches!(op, ArithOp::Shl | ArithOp::Shr);
                let rx = self.use_(x, X16, ty);
                self.release(b);
                self.release(a);
                let (d, t) = self.dest(dst);
                if checked && !(0..bits).contains(&c) {
                    let l = self.stub_site(site);
                    self.jump(l);
                    self.emit(a64::movz(true, d, 0, 0));
                    return self.done(d, t);
                }
                let amt = (c & (bits - 1)) as u32;
                let left = matches!(op, ArithOp::Shl | ArithOp::ShlW);
                if left {
                    self.emit(a64::lsl_imm(s, d, rx, amt));
                    if narrow {
                        self.normalize(d, ty);
                    }
                } else if ty.signed() {
                    self.emit(a64::asr_imm(s, d, rx, amt));
                } else {
                    self.emit(a64::lsr_imm(s, d, rx, amt));
                }
                return self.done(d, t);
            }
            // ---------------------------------------------- constant divisor
            (ArithOp::Div | ArithOp::DivW | ArithOp::Rem, V::Const(0)) => {
                let _ = self.use_(x, X16, ty);
                self.release(b);
                self.release(a);
                let (d, t) = self.dest(dst);
                let l = self.stub_site(site);
                self.jump(l);
                self.emit(a64::movz(true, d, 0, 0));
                return self.done(d, t);
            }
            _ => {}
        }

        // -------------------------------------------------- register forms
        let rty = if op.is_shift() { rhs.ty } else { ty };
        let (xty, yty) = if op.is_shift() { (ty, rty) } else { (ty, ty) };
        let rx = self.use_(x, X16, xty);
        let ry = if op.is_shift() || matches!(op, ArithOp::Div | ArithOp::DivW | ArithOp::Rem) {
            self.use_nz(y, X17, yty)
        } else {
            self.use_(y, X17, yty)
        };
        self.release(b);
        self.release(a);
        let yc = match y {
            V::Const(c) => Some(c),
            _ => None,
        };
        let (d, t) = self.dest(dst);
        match op {
            ArithOp::AddW | ArithOp::SubW | ArithOp::Add | ArithOp::Sub => {
                let sub = matches!(op, ArithOp::Sub | ArithOp::SubW);
                let checked = matches!(op, ArithOp::Add | ArithOp::Sub);
                if checked && !narrow {
                    self.emit(a64::addsub_reg(s, sub, true, d, rx, ry, Shift::Lsl, 0));
                    let l = self.stub_site(site);
                    self.bcond(overflow_cond(ty, sub), l);
                } else {
                    self.emit(a64::addsub_reg(s, sub, false, d, rx, ry, Shift::Lsl, 0));
                    if narrow {
                        if checked {
                            self.check_narrow(d, ty, site);
                        } else {
                            self.normalize(d, ty);
                        }
                    }
                }
            }
            ArithOp::MulW => {
                self.emit(a64::mul(s, d, rx, ry));
                if narrow {
                    self.normalize(d, ty);
                }
            }
            ArithOp::Mul => {
                if narrow {
                    self.emit(a64::mul(false, d, rx, ry));
                    self.check_narrow(d, ty, site);
                } else {
                    match ty {
                        Ty::U32 => {
                            self.emit(a64::umull(d, rx, ry));
                            self.emit(a64::cmp_ext(true, d, d, Ext::Uxtw));
                            let l = self.stub_site(site);
                            self.bcond(Cond::Ne, l);
                        }
                        Ty::I32 => {
                            self.emit(a64::smull(d, rx, ry));
                            self.emit(a64::cmp_ext(true, d, d, Ext::Sxtw));
                            let l = self.stub_site(site);
                            self.bcond(Cond::Ne, l);
                        }
                        Ty::U64 => {
                            self.emit(a64::umulh(X8, rx, ry));
                            self.emit(a64::mul(true, d, rx, ry));
                            let l = self.stub_site(site);
                            self.cbz(true, X8, true, l);
                        }
                        _ => {
                            // I64
                            self.emit(a64::smulh(X8, rx, ry));
                            self.emit(a64::mul(true, d, rx, ry));
                            self.emit(a64::cmp_shifted(true, X8, d, Shift::Asr, 63));
                            let l = self.stub_site(site);
                            self.bcond(Cond::Ne, l);
                        }
                    }
                }
            }
            ArithOp::Div | ArithOp::DivW | ArithOp::Rem => {
                if yc.is_none() {
                    let l = self.stub_site(site);
                    self.cbz(s, ry, false, l);
                }
                if op == ArithOp::Rem {
                    if ty.signed() {
                        self.emit(a64::sdiv(s, X8, rx, ry));
                    } else {
                        self.emit(a64::udiv(s, X8, rx, ry));
                    }
                    self.emit(a64::msub(s, d, X8, ry, rx));
                } else if !ty.signed() {
                    self.emit(a64::udiv(s, d, rx, ry));
                } else if narrow {
                    self.emit(a64::sdiv(false, d, rx, ry));
                    if op == ArithOp::Div {
                        self.check_narrow(d, ty, site + 1);
                    } else {
                        self.normalize(d, ty);
                    }
                } else {
                    if op == ArithOp::Div && yc.map_or(true, |c| c == -1) {
                        // MIN / -1 overflows.
                        let ok = self.new_label();
                        if yc.is_none() {
                            self.emit(a64::cmn_imm(s, ry, 1));
                            self.bcond(Cond::Ne, ok);
                        }
                        self.emit(a64::subs(s, ZR, ZR, rx));
                        let l = self.stub_site(site + 1);
                        self.bcond(Cond::Vs, l);
                        self.bind(ok);
                    }
                    self.emit(a64::sdiv(s, d, rx, ry));
                }
            }
            ArithOp::And => self.emit(a64::logic_reg(s, LogOp::And, false, d, rx, ry)),
            ArithOp::Or => self.emit(a64::logic_reg(s, LogOp::Orr, false, d, rx, ry)),
            ArithOp::Xor => self.emit(a64::logic_reg(s, LogOp::Eor, false, d, rx, ry)),
            ArithOp::Shl | ArithOp::Shr | ArithOp::ShlW | ArithOp::ShrW => {
                let bits = ty.bits();
                let checked = matches!(op, ArithOp::Shl | ArithOp::Shr);
                let mut amt = ry;
                if checked {
                    self.emit(a64::cmp_imm(rty.is64(), ry, bits));
                    let l = self.stub_site(site);
                    self.bcond(Cond::Hs, l);
                } else if narrow {
                    self.emit(a64::logic_imm(false, LogOp::And, X8, ry, (bits - 1) as u64).unwrap());
                    amt = X8;
                }
                let left = matches!(op, ArithOp::Shl | ArithOp::ShlW);
                if left {
                    self.emit(a64::lslv(s, d, rx, amt));
                    if narrow {
                        self.normalize(d, ty);
                    }
                } else if ty.signed() {
                    self.emit(a64::asrv(s, d, rx, amt));
                } else {
                    self.emit(a64::lsrv(s, d, rx, amt));
                }
            }
        }
        self.done(d, t)
    }
}

/// (size, opc) of the load or store of one `ty`: loads sign-extend i8/i16
/// to 32 bits and zero-extend u8/u16/bool, which is the canonical form.
fn ldst_op(ty: Ty, load: bool) -> (u32, u32) {
    let size = a64::size_log2(ty.bytes());
    let opc = if !load {
        a64::ST
    } else if ty.signed() && size < 2 {
        a64::LDS32
    } else {
        a64::LD
    };
    (size, opc)
}

fn overflow_cond(ty: Ty, sub: bool) -> Cond {
    if ty.signed() {
        Cond::Vs
    } else if sub {
        Cond::Lo
    } else {
        Cond::Hs
    }
}

fn extend(d: Reg, n: Reg, e: Ext) -> u32 {
    match e {
        Ext::Uxtb => a64::uxtb(d, n),
        Ext::Uxth => a64::uxth(d, n),
        Ext::Sxtb => a64::sxtb(d, n),
        Ext::Sxth => a64::sxth(d, n),
        _ => unreachable!("only narrow extends"),
    }
}

/// Conservative: false only when control certainly cannot reach the end.
fn may_fall_through(ss: &[Stmt]) -> bool {
    match ss.last() {
        Some(Stmt::Return(_)) => false,
        Some(Stmt::If { then, els, .. }) => may_fall_through(then) || els.is_empty() || may_fall_through(els),
        _ => true,
    }
}

fn ends_in_jump(ss: &[Stmt]) -> bool {
    matches!(ss.last(), Some(Stmt::Return(_)) | Some(Stmt::Break) | Some(Stmt::Continue))
}

fn weigh_stmts(ss: &[Stmt], depth: u32, w: &mut [u64], wr: &mut [u64], has_call: &mut bool) {
    let unit = 1u64 << (3 * depth.min(6));
    for s in ss {
        match s {
            Stmt::Assign { var, value } => {
                wr[*var as usize] += unit;
                weigh_expr(value, unit, w, has_call);
            }
            Stmt::If { cond, then, els } => {
                weigh_expr(cond, unit, w, has_call);
                weigh_stmts(then, depth, w, wr, has_call);
                weigh_stmts(els, depth, w, wr, has_call);
            }
            Stmt::While { cond, body, step } => {
                weigh_expr(cond, unit << 3, w, has_call);
                weigh_stmts(body, depth + 1, w, wr, has_call);
                weigh_stmts(step, depth + 1, w, wr, has_call);
            }
            Stmt::Break | Stmt::Continue | Stmt::Return(None) => {}
            Stmt::Return(Some(e)) | Stmt::Eval(e) | Stmt::Assert { cond: e, .. } => weigh_expr(e, unit, w, has_call),
            Stmt::AssertEq { lhs, rhs, .. } => {
                weigh_expr(lhs, unit, w, has_call);
                weigh_expr(rhs, unit, w, has_call);
            }
            Stmt::Store { addr, value, .. } => {
                weigh_expr(addr, unit, w, has_call);
                weigh_expr(value, unit, w, has_call);
            }
            Stmt::Copy { dst, src, .. } => {
                weigh_expr(dst, unit, w, has_call);
                weigh_expr(src, unit, w, has_call);
            }
        }
    }
}

fn weigh_expr(e: &Expr, unit: u64, w: &mut [u64], has_call: &mut bool) {
    match &e.kind {
        ExprKind::Const(_) => {}
        ExprKind::Var(v) => w[*v as usize] += unit,
        ExprKind::Arith { lhs, rhs, .. } | ExprKind::Cmp { lhs, rhs, .. } => {
            weigh_expr(lhs, unit, w, has_call);
            weigh_expr(rhs, unit, w, has_call);
        }
        ExprKind::And(a, b) | ExprKind::Or(a, b) => {
            weigh_expr(a, unit, w, has_call);
            weigh_expr(b, unit, w, has_call);
        }
        ExprKind::Not(a) | ExprKind::BitNot(a) | ExprKind::Widen(a) => weigh_expr(a, unit, w, has_call),
        ExprKind::FNeg(a) | ExprKind::IntToFloat(a) => weigh_expr(a, unit, w, has_call),
        ExprKind::FArith { lhs, rhs, .. } => {
            weigh_expr(lhs, unit, w, has_call);
            weigh_expr(rhs, unit, w, has_call);
        }
        ExprKind::Cast { arg, .. } | ExprKind::FloatToInt { arg, .. } => weigh_expr(arg, unit, w, has_call),
        ExprKind::Call { args, .. } => {
            *has_call = true;
            for a in args {
                weigh_expr(a, unit, w, has_call);
            }
        }
        ExprKind::Slot(_) | ExprKind::Data(_) | ExprKind::Global(_) => {}
        ExprKind::Load { addr, .. } => weigh_expr(addr, unit, w, has_call),
        ExprKind::Offset { base: a, idx: b, .. } | ExprKind::Bounds { idx: a, len: b, .. } => {
            weigh_expr(a, unit, w, has_call);
            weigh_expr(b, unit, w, has_call);
        }
        ExprKind::Seq { stmts, value } => {
            for s in stmts {
                match s {
                    Stmt::Store { addr: a, value: b, .. } | Stmt::Copy { dst: a, src: b, .. } => {
                        weigh_expr(a, unit, w, has_call);
                        weigh_expr(b, unit, w, has_call);
                    }
                    Stmt::Eval(a) => weigh_expr(a, unit, w, has_call),
                    _ => {}
                }
            }
            weigh_expr(value, unit, w, has_call);
        }
        ExprKind::Select { cond, then, els } => {
            weigh_expr(cond, unit, w, has_call);
            weigh_expr(then, unit, w, has_call);
            weigh_expr(els, unit, w, has_call);
        }
    }
}

// ------------------------------------------------------------------ linking

/// Linked machine code: `prefix` followed by the functions.
pub struct Linked {
    pub code: Vec<u32>,
    /// Word offset of each function in `code`, indexed by FuncId.
    pub offsets: Vec<Option<usize>>,
    /// Size in words of each function.
    pub sizes: Vec<usize>,
    /// Word index in `code` of every `adrp` + `add` pair, with its blob.
    pub data_refs: Vec<(usize, u32)>,
}

/// Lay the functions out after `prefix`, resolve every `bl` and every
/// `b trap_common` (`trap_common` is a word index into `prefix`).
pub fn link(prefix: Vec<u32>, trap_common: Option<usize>, funcs: &[FuncCode], nfuncs: usize) -> Result<Linked, String> {
    let mut code = prefix;
    let mut offsets = vec![None; nfuncs];
    let mut sizes = vec![0; nfuncs];
    let mut starts = Vec::with_capacity(funcs.len());
    let mut data_refs = Vec::new();
    for fc in funcs {
        // Keep function starts 16-byte aligned, like the system toolchain.
        while code.len() % 4 != 0 {
            code.push(a64::nop());
        }
        starts.push(code.len());
        offsets[fc.func as usize] = Some(code.len());
        sizes[fc.func as usize] = fc.code.len();
        code.extend_from_slice(&fc.code);
    }
    for (fc, &start) in funcs.iter().zip(&starts) {
        for &(p, callee) in &fc.calls {
            let at = start + p;
            let target = offsets[callee as usize].ok_or_else(|| format!("call to function #{} that was not emitted", callee))?;
            let off = target as i64 - at as i64;
            if !(-(1 << 25)..(1 << 25)).contains(&off) {
                return Err("bl out of range (module larger than 128 MiB)".into());
            }
            code[at] = a64::bl(off as i32);
        }
        data_refs.extend(fc.data_refs.iter().map(|&(p, k)| (start + p, k)));
        for &p in &fc.trap_jumps {
            let at = start + p;
            let tc = trap_common.ok_or("trap stub without trap_common")?;
            let off = tc as i64 - at as i64;
            if !(-(1 << 25)..(1 << 25)).contains(&off) {
                return Err("trap branch out of range".into());
            }
            code[at] = a64::b(off as i32);
        }
    }
    Ok(Linked { code, offsets, sizes, data_refs })
}

/// Byte offset of each blob of `data` when laid out from offset 0, each
/// 8-byte aligned (the interpreter's layout too), and the total size.
pub fn data_layout(data: &[Vec<u8>]) -> (Vec<usize>, usize) {
    let mut offs = Vec::with_capacity(data.len());
    let mut at = 0usize;
    for b in data {
        at = (at + 7) & !7;
        offs.push(at);
        at += b.len();
    }
    (offs, at)
}

/// Point every `adrp` + `add` pair of `code` at its blob, for an image whose
/// first word sits on a 4 KiB boundary and whose data starts at byte
/// `data_at` of the image.
pub fn resolve_data(code: &mut [u32], refs: &[(usize, u32)], blob_offs: &[usize], data_at: usize) -> Result<(), String> {
    for &(p, k) in refs {
        let target = data_at + *blob_offs.get(k as usize).ok_or_else(|| format!("reference to missing data blob #{}", k))?;
        let pages = (target >> 12) as i64 - ((p * 4) >> 12) as i64;
        if !(-(1 << 20)..(1 << 20)).contains(&pages) {
            return Err("data out of adrp range (image larger than 4 GiB)".into());
        }
        code[p] = a64::adrp_patch(code[p], pages as i32);
        code[p + 1] = a64::add_imm_patch(code[p + 1], (target & 0xfff) as u32);
    }
    Ok(())
}
