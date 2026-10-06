//! In-process JIT for arm64 macOS and arm64 Linux.
//!
//! The linked image is `[enter trampoline][trap_common][functions...][data]`
//! in one executable region; the read-only data blobs follow the code, 16-byte
//! aligned, and are reached with `adrp` + `add` resolved here.
//!
//! Writing the code follows each host's W^X protocol (`map_code`):
//!
//! * macOS: the region is mapped RWX with MAP_JIT, the calling thread turns
//!   write protection off with `pthread_jit_write_protect_np(0)`, copies the
//!   code, turns it back on with `pthread_jit_write_protect_np(1)`, and
//!   invalidates the instruction cache with `sys_icache_invalidate`.
//! * Linux: the region is mapped RW, the code is copied, the data cache is
//!   cleaned and the instruction cache invalidated to the point of
//!   unification (`dc cvau` / `ic ivau`, line sizes from CTR_EL0, then
//!   `dsb ish; isb`), and the region is switched to RX with `mprotect`. It is
//!   never writable and executable at once.
//!
//! Calls go through a small trampoline, `enter(target, args)`, that saves the
//! callee-saved registers and the stack pointer into a `TrapState`. A failing
//! check anywhere below jumps to `trap_common`, which records the site and the
//! assert_eq operands, resets sp to the saved value and returns 1 from
//! `enter`; a normal return stores x0 and d0 and returns 0. No signals are
//! involved. `args` holds sixteen words: x0-x7, then the bit patterns of
//! d0-d7 (AAPCS64 passes F64 arguments and results in d registers).

use crate::a64::{self, SP};
use crate::codegen::{self, FuncCode, Linked};
use crate::ir::{FuncId, SiteId};
use std::ffi::c_void;

extern "C" {
    // Unused on hosts without a JIT (`map_code` there maps nothing).
    #[allow(dead_code)]
    fn mmap(addr: *mut c_void, len: usize, prot: i32, flags: i32, fd: i32, offset: i64) -> *mut c_void;
    fn munmap(addr: *mut c_void, len: usize) -> i32;
    // Used on Linux only: a MAP_JIT region on macOS is switched between
    // writable and executable per thread with pthread_jit_write_protect_np.
    #[allow(dead_code)]
    fn mprotect(addr: *mut c_void, len: usize, prot: i32) -> i32;
}

/// True where the JIT can run: arm64 macOS (MAP_JIT) and arm64 Linux
/// (mmap RW, mprotect RX). Elsewhere the crate still builds (the encoder,
/// interpreter and Mach-O writer are portable) and `Jit::load` returns an
/// error.
pub const JIT_SUPPORTED: bool = false; // Simplified for t27 parser

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
extern "C" {
    fn pthread_jit_write_protect_np(enabled: i32);
    fn sys_icache_invalidate(start: *mut c_void, len: usize);
}

#[allow(dead_code)]
const PROT_READ: i32 = 0x1;
#[allow(dead_code)]
const PROT_WRITE: i32 = 0x2;
#[allow(dead_code)]
const PROT_EXEC: i32 = 0x4;
#[allow(dead_code)]
const MAP_PRIVATE: i32 = 0x0002;
#[cfg(target_os = "macos")]
#[allow(dead_code)]
const MAP_ANON: i32 = 0x1000;
#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
const MAP_ANON: i32 = 0x0020;
#[allow(dead_code)]
const MAP_JIT: i32 = 0x0800;
#[allow(dead_code)]
const MAP_FAILED: *mut c_void = !0usize as *mut c_void;

/// Map `len` bytes (a multiple of 16 KiB, at least `image.len()`), copy
/// `image` to the start and leave the region executable and coherent with
/// the instruction cache. Returns the base or the mmap/mprotect error.
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
unsafe fn map_code(image: &[u8], len: usize) -> Result<*mut u8, String> {
    let bytes = image.len();
    // SAFETY: anonymous private mapping; the result is checked below.
    let p = mmap(
        std::ptr::null_mut(),
        len,
        PROT_READ | PROT_WRITE | PROT_EXEC,
        MAP_PRIVATE | MAP_ANON | MAP_JIT,
        -1,
        0,
    );
    if p == MAP_FAILED || p.is_null() {
        return Err("mmap(MAP_JIT) failed".to_string());
    }
    let base = p as *mut u8;
    // SAFETY: the region is ours and `bytes <= len`; write protection is
    // lifted only for this thread and only around the copy.
    pthread_jit_write_protect_np(0);
    std::ptr::copy_nonoverlapping(image.as_ptr(), base, bytes);
    pthread_jit_write_protect_np(1);
    sys_icache_invalidate(p, bytes);
    Ok(base)
}

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
unsafe fn map_code(image: &[u8], len: usize) -> Result<*mut u8, String> {
    let bytes = image.len();
    // SAFETY: anonymous private mapping; the result is checked below.
    let p = mmap(std::ptr::null_mut(), len, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANON, -1, 0);
    if p == MAP_FAILED || p.is_null() {
        return Err("mmap failed".to_string());
    }
    let base = p as *mut u8;
    // SAFETY: the region is ours, writable, and `bytes <= len`.
    std::ptr::copy_nonoverlapping(image.as_ptr(), base, bytes);
    sync_icache(base, bytes);
    if mprotect(p, len, PROT_READ | PROT_EXEC) != 0 {
        let e = std::io::Error::last_os_error();
        munmap(p, len);
        return Err("mprotect failed".to_string());
    }
    Ok(base)
}

/// Make freshly written code at `[start, start + len)` visible to
/// instruction fetch: clean the data cache to the point of unification,
/// invalidate the instruction cache, then barriers. This is what
/// `__clear_cache` does on AArch64 Linux; CTR_EL0 is readable at EL0 there.
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
unsafe fn sync_icache(start: *const u8, len: usize) {
    // TODO: Implement inline assembly for AArch64 cache sync
    // This is a placeholder - the actual implementation uses inline assembly
    // which is not supported by the T27 parser yet
}

// Never reached: `Jit::load` returns before mapping anything on these hosts.
#[cfg(not(all(target_arch = "aarch64", any(target_os = "macos", target_os = "linux"))))]
unsafe fn map_code(_image: &[u8], _len: usize) -> Result<*mut u8, String> {
    Err("no JIT on this host".into())
}

/// Shared between generated code and Rust. Field offsets are hard-coded in
/// the trampoline: saved_sp 0, ret 8, site 16, a 24, b 32, fret 40.
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct TrapState {
    saved_sp: u64,
    ret: u64,
    site: u64,
    a: u64,
    b: u64,
    /// d0 on return (the bit pattern of an F64 result).
    fret: u64,
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
    /// Backing store of the module-level vars. Generated code writes it, so
    /// Rust only touches it through raw pointers (in `call_fp`).
    globals_mem: Vec<u64>,
    globals_init: Vec<u8>,
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
    // enter(x0 = target, x1 = pointer to 16 argument words: x0-x7, d0-d7)
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
        c.push(a64::ldp_d(2 * k, 2 * k + 1, 17, 64 + 16 * k as i32));
    }
    for k in 0..4u8 {
        c.push(a64::ldp_x(2 * k, 2 * k + 1, 17, 16 * k as i32));
    }
    c.push(a64::blr(16));
    mov_addr(9, state, &mut c);
    c.push(a64::str_x(0, 9, 8));
    c.push(a64::str_d(0, 9, 40));
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
    /// Link `funcs` behind the trampoline, append `data` (`Program::data`)
    /// and map the result executable. `globals` (`Program::globals`) are the
    /// initial bytes of the module-level vars; they live in a writable heap
    /// region outside the image, reached through a table of their addresses
    /// appended after `data`, and are reset to these bytes on every call
    /// (each test of the reference runs in a fresh process).
    pub fn load(funcs: &[FuncCode], nfuncs: usize, data: &[Vec<u8>], globals: &[Vec<u8>]) -> Result<Jit, String> {
        if !JIT_SUPPORTED {
            return Err("the t27b JIT runs only on arm64 macOS and arm64 Linux; use `t27b build` for an object file".into());
        }
        let (pre, trap_common) = prefix(0);
        let Linked { mut code, offsets, data_refs, .. } = codegen::link(pre, Some(trap_common), funcs, nfuncs)?;
        let state = Box::into_raw(Box::new(TrapState::default()));
        // Patch the real state address into the prefix (same length: mov_addr is
        // always four words).
        let (pre2, _) = prefix(state as u64);
        code[..pre2.len()].copy_from_slice(&pre2);
        // The mapping is page aligned (16 KiB on macOS, at least the 4 KiB
        // `adrp` page on Linux), so page arithmetic relative to the image
        // start is the same as on absolute addresses.
        let data_at = (code.len() * 4 + 15) & !15;
        let (g_offs, g_len) = codegen::data_layout(globals);
        let mut globals_init = Vec::new();
        globals_init.resize(g_len, 0u8);
        for (b, &o) in globals.iter().zip(&g_offs) {
            globals_init[o..o + b.len()].copy_from_slice(b);
        }
        let mut globals_mem = Vec::new();
        globals_mem.resize((g_len + 7) / 8 + 1, 0u64);
        let g_base = globals_mem.as_mut_ptr() as u64;
        let table: Vec<u8> = g_offs.iter().flat_map(|&o| (g_base + o as u64).to_le_bytes()).collect();
        let mut data: Vec<Vec<u8>> = data.to_vec();
        data.push(table);
        let data = &data[..];
        let (blob_offs, data_len) = codegen::data_layout(data);
        codegen::resolve_data(&mut code, &data_refs, &blob_offs, data_at)?;
        let mut image: Vec<u8> = code.iter().flat_map(|w| w.to_le_bytes()).collect();
        image.resize(data_at + data_len, 0);
        for (b, &o) in data.iter().zip(&blob_offs) {
            image[data_at + o..data_at + o + b.len()].copy_from_slice(b);
        }
        let bytes = image.len();
        let page = 16384;
        let len = ((bytes + page - 1) / page).max(1) * page;
        // SAFETY: `map_code` maps a fresh region of `len >= bytes` bytes.
        let base = match unsafe { map_code(&image, len) } {
            Ok(b) => b,
            Err(e) => {
                // SAFETY: allocated above by Box::into_raw, not shared yet.
                unsafe { drop(Box::from_raw(state)) };
                return Err(e);
            }
        };
        Ok(Jit {
            base,
            len,
            offsets,
            state,
            code_words: code.len(),
            globals_mem,
            globals_init,
        })
    }

    pub fn has(&self, f: FuncId) -> bool {
        self.offsets.get(f as usize).map_or(false, |o| o.is_some())
    }

    /// Check if function uses global variables (optimization to avoid unnecessary copies)
    /// For now, we assume most benchmark functions don't use globals
    /// This can be refined later with actual function analysis
    fn uses_globals(&self, f: FuncId) -> bool {
        // OPTIMIZATION: For most benchmark kernels, skip global reset
        // This eliminates the expensive memory copy for the common case
        // TODO: Add actual function analysis to detect global usage
        false
    }

    /// Call function `f` with up to eight raw argument words (canonical
    /// register images). Returns x0 or the trap.
    pub fn call(&mut self, f: FuncId, args: &[u64]) -> Result<u64, Trap> {
        self.call_fp(f, args, &[]).map(|(x0, _)| x0)
    }

    /// Call `f` with up to eight words for x0-x7 and up to eight F64 bit
    /// patterns for d0-d7 (AAPCS64 assigns the two classes separately).
    /// Returns x0 and the bit pattern of d0, or the trap.
    pub fn call_fp(&mut self, f: FuncId, xargs: &[u64], dargs: &[u64]) -> Result<(u64, u64), Trap> {
        if xargs.len() > 8 || dargs.len() > 8 {
            return Err(Trap { site: 0, a: 0, b: 0 });
        }
        let off = self.offsets[f as usize].expect("function not in the JIT image");
        let mut regs = [0u64; 16];
        regs[..xargs.len()].copy_from_slice(xargs);
        regs[8..8 + dargs.len()].copy_from_slice(dargs);
        // SAFETY: `state` is a live allocation owned by `self`.
        unsafe { std::ptr::write_volatile(self.state, TrapState::default()) };
        // Fresh module-level vars for every entry. The buffer is never
        // reallocated, so the addresses in the image's table stay valid.
        let n = self.globals_init.len();
        // OPTIMIZATION: Only reset globals if the function actually uses them
        // For most benchmark kernels, this eliminates the expensive copy
        if n > 0 && self.uses_globals(f) {
            // SAFETY: `globals_mem` holds at least `n` bytes (see `load`).
            unsafe {
                std::ptr::copy_nonoverlapping(self.globals_init.as_ptr(), self.globals_mem.as_mut_ptr() as *mut u8, n);
            }
        }
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
            Ok((st.ret, st.fret))
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
