//! TRIOS Hybrid — HybridBigInt
//!
//! FFI bindings to Zig-based HybridBigInt implementation.

use std::ffi::c_void;

pub type Hybrid = *mut c_void;

#[no_mangle]
pub extern "C" fn hybrid_create() -> Hybrid {
    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn hybrid_destroy(h: Hybrid) {
    // TODO
}

#[no_mangle]
pub unsafe extern "C" fn hybrid_add(a: Hybrid, b: Hybrid, out: Hybrid) -> i32 {
    0
}
