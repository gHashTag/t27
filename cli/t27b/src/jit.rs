//! In-process JIT for arm64 macOS.
//!
//! The linked image is `[enter trampoline][trap_common][functions...]` in one
//! MAP_JIT region. Writing follows Apple's protocol: the region is mapped
//! RWX with MAP_JIT, the calling thread turns write protection off with
//! `pthread_jit_write_protect_np(0)`, copies the code, turns it back on with
//! `pthread_jit_write_protect_np(1)`, and invalidates the instruction cache.
//!
//! Calls go through a small trampoline, `enter(target, args)`, that saves the
//! callee-saved registers and the stack pointer into a `TrapState`. A failing
//! check anywhere below jumps to `trap_common`, which records the site and the
//! assert_eq operands, resets sp to the saved value and returns 1 from
//! `enter`; a normal return stores x0 and returns 0. No signals are involved.

use crate::a64::{self, SP};
use crate::codegen::{self, FuncCode, Linked};
use crate::ir::{FuncId, SiteId};
use std::ffi::c_void;

extern "C" {
    fn mmap(addr: *mut c_void, len: usize, prot: i32, flags: i32, fd: i32, offset: i64) -> *mut c_void;
    fn munmap(addr: *mut c_void, len: usize) -> i32;
    // Declared for completeness: a MAP_JIT region is switched between
    // writable and executable per thread with pthread_jit_write_protect_np,
    // so mprotect is not needed on this path.
    #[allow(dead_code)]
    fn mprotect(addr: *mut c_void, len: usize, prot: i32) -> i32;
}

/// True where the JIT can run: MAP_JIT and the two calls below are Apple
/// arm64 only. Elsewhere the crate still builds (the encoder, interpreter
/// and Mach-O writer are portable) and `Jit::load` returns an error.
pub const JIT_SUPPORTED: bool = cfg!(all(target_os = "macos", target_arch = "aarch64"));

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
extern "C" {
    fn pthread_jit_write_protect_np(enabled: i32);
    fn sys_icache_invalidate(start: *mut c_void, len: usize);
}

// Never reached: `Jit::load` returns before mapping anything on these hosts.
#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
unsafe fn pthread_jit_write_protect_np(_enabled: i32) {}
#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
unsafe fn sys_icache_invalidate(_start: *mut c_void, _len: usize) {}

const PROT_READ: i32 = 0x1;
const PROT_WRITE: i32 = 0x2;
const PROT_EXEC: i32 = 0x4;
const MAP_PRIVATE: i32 = 0x0002;
const MAP_ANON: i32 = 0x1000;
const MAP_JIT: i32 = 0x0800;
const MAP_FAILED: *mut c_void = !0usize as *mut c_void;

/// Shared between generated code and Rust. Field offsets are hard-coded in
/// the trampoline: saved_sp 0, ret 8, site 16, a 24, b 32.
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct TrapState {
    saved_sp: u64,
    ret: u64,
    site: u64,
    a: u64,
    b: u64,
}

/// A trap raised by generated code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trap {
    pub site: SiteId,
    /// assert_eq operands (raw register images); zero for other traps.
    pub a: u64,
    pub b: u64,
}

pub struct Jit {
    base: *mut u8,
    len: usize,
    offsets: Vec<Option<usize>>,
    /// Owned (from Box::into_raw, freed in Drop). Generated code writes it,
    /// so Rust only touches it through volatile raw-pointer accesses.
    state: *mut TrapState,
    pub code_words: usize,
}

type Enter = unsafe extern "C" fn(target: *const u8, args: *const u64) -> u64;

fn mov_addr(rd: u8, addr: u64, out: &mut Vec<u32>) {
    // Always four instructions so the layout does not depend on the address.
    for hw in 0..4u32 {
        let imm = ((addr >> (16 * hw)) & 0xffff) as u32;
        out.push(if hw == 0 { a64::movz(true, rd, imm, 0) } else { a64::movk(true, rd, imm, hw) });
    }
}

/// The trampoline and `trap_common`. Returns the words and the word index of
/// `trap_common`.
fn prefix(state: u64) -> (Vec<u32>, usize) {
    let mut c = Vec::new();
    // enter(x0 = target, x1 = pointer to 8 argument words)
    c.push(a64::stp_x_pre(29, 30, SP, -96));
    c.push(a64::mov_sp(29, SP));
    for (k, r) in [19u8, 21, 23, 25, 27].iter().enumerate() {
        c.push(a64::stp_x(*r, r + 1, SP, 16 + 16 * k as i32));
    }
    mov_addr(9, state, &mut c);
    c.push(a64::mov_sp(10, SP));
    c.push(a64::str_x(10, 9, 0));
    c.push(a64::mov(true, 16, 0));
    c.push(a64::mov(true, 17, 1));
    for k in 0..4u8 {
        c.push(a64::ldp_x(2 * k, 2 * k + 1, 17, 16 * k as i32));
    }
    c.push(a64::blr(16));
    mov_addr(9, state, &mut c);
    c.push(a64::str_x(0, 9, 8));
    c.push(a64::movz(true, 0, 0, 0));
    let restore = c.len();
    for (k, r) in [19u8, 21, 23, 25, 27].iter().enumerate() {
        c.push(a64::ldp_x(*r, r + 1, SP, 16 + 16 * k as i32));
    }
    c.push(a64::ldp_x_post(29, 30, SP, 96));
    c.push(a64::ret());
    // trap_common(w1 = site, x2 = a, x3 = b)
    let trap_common = c.len();
    mov_addr(9, state, &mut c);
    c.push(a64::str_x(1, 9, 16));
    c.push(a64::str_x(2, 9, 24));
    c.push(a64::str_x(3, 9, 32));
    c.push(a64::ldr_x(10, 9, 0));
    c.push(a64::mov_sp(SP, 10));
    c.push(a64::movz(true, 0, 1, 0));
    let here = c.len();
    c.push(a64::b(restore as i32 - here as i32));
    (c, trap_common)
}

impl Jit {
    /// Link `funcs` behind the trampoline and map the result executable.
    pub fn load(funcs: &[FuncCode], nfuncs: usize) -> Result<Jit, String> {
        if !JIT_SUPPORTED {
            return Err("the t27b JIT runs only on arm64 macOS; use `t27b build` for an object file".into());
        }
        let (pre, trap_common) = prefix(0);
        let Linked { mut code, offsets, .. } = codegen::link(pre, Some(trap_common), funcs, nfuncs)?;
        let state = Box::into_raw(Box::new(TrapState::default()));
        // Patch the real state address into the prefix (same length: mov_addr is
        // always four words).
        let (pre2, _) = prefix(state as u64);
        code[..pre2.len()].copy_from_slice(&pre2);
        let bytes = code.len() * 4;
        let page = 16384;
        let len = ((bytes + page - 1) / page).max(1) * page;
        // SAFETY: anonymous private mapping; the result is checked below.
        let p = unsafe {
            mmap(
                std::ptr::null_mut(),
                len,
                PROT_READ | PROT_WRITE | PROT_EXEC,
                MAP_PRIVATE | MAP_ANON | MAP_JIT,
                -1,
                0,
            )
        };
        if p == MAP_FAILED || p.is_null() {
            // SAFETY: allocated above by Box::into_raw, not shared yet.
            unsafe { drop(Box::from_raw(state)) };
            return Err(format!(
                "mmap(MAP_JIT) of {} bytes failed: {}",
                len,
                std::io::Error::last_os_error()
            ));
        }
        let base = p as *mut u8;
        // SAFETY: the region is ours and `bytes <= len`; write protection is
        // lifted only for this thread and only around the copy.
        unsafe {
            pthread_jit_write_protect_np(0);
            let dst = base as *mut u32;
            for (i, w) in code.iter().enumerate() {
                dst.add(i).write(w.to_le());
            }
            pthread_jit_write_protect_np(1);
            sys_icache_invalidate(p, bytes);
        }
        Ok(Jit {
            base,
            len,
            offsets,
            state,
            code_words: code.len(),
        })
    }

    pub fn has(&self, f: FuncId) -> bool {
        self.offsets.get(f as usize).map_or(false, |o| o.is_some())
    }

    /// Call function `f` with up to eight raw argument words (canonical
    /// register images). Returns x0 or the trap.
    pub fn call(&mut self, f: FuncId, args: &[u64]) -> Result<u64, Trap> {
        assert!(args.len() <= 8, "at most 8 arguments");
        let off = self.offsets[f as usize].expect("function not in the JIT image");
        let mut regs = [0u64; 8];
        regs[..args.len()].copy_from_slice(args);
        // SAFETY: `state` is a live allocation owned by `self`.
        unsafe { std::ptr::write_volatile(self.state, TrapState::default()) };
        // SAFETY: `base` is the start of our executable mapping, word 0 is
        // the `enter` trampoline with the C signature `Enter`, `off` is the
        // start of a function compiled for this image, and `regs` outlives
        // the call. Generated code only touches its own stack frame and the
        // TrapState owned by `self`.
        let flag = unsafe {
            let enter: Enter = std::mem::transmute::<*mut u8, Enter>(self.base);
            enter(self.base.add(off * 4), regs.as_ptr())
        };
        // SAFETY: as above; generated code has finished writing it.
        let st = unsafe { std::ptr::read_volatile(self.state) };
        if flag == 0 {
            Ok(st.ret)
        } else {
            Err(Trap {
                site: st.site as SiteId,
                a: st.a,
                b: st.b,
            })
        }
    }
}

impl Drop for Jit {
    fn drop(&mut self) {
        // SAFETY: unmapping exactly the region mapped in `load`.
        unsafe {
            munmap(self.base as *mut c_void, self.len);
            drop(Box::from_raw(self.state));
        }
    }
}
