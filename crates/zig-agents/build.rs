//! Build script for zig-agents.
//!
//! Only attempts Zig compilation when the `ffi` feature is enabled.
//! In stub mode, does nothing (no Zig toolchain required).

fn main() {
    // Only compile Zig when FFI feature is requested
    if std::env::var("CARGO_FEATURE_FFI").is_ok() {
        let status = std::process::Command::new("zig")
            .args(["build", "-Doptimize=ReleaseFast"])
            .current_dir(".")
            .status();

        match status {
            Ok(s) if s.success() => {
                let lib_dir = std::env::current_dir().unwrap();
                let static_lib = lib_dir.join("zig-out/lib/libzig_agents.a");
                if static_lib.exists() {
                    println!("cargo:rustc-link-search=native={}", lib_dir.display());
                    println!("cargo:rustc-link-lib=static=zig_agents");
                } else {
                    println!("cargo:warning=zig build succeeded but static library not found at {:?}", static_lib);
                }
            }
            Ok(s) => {
                println!("cargo:warning=zig build failed for zig-agents (exit {:?})", s.code());
            }
            Err(e) => {
                println!("cargo:warning=failed to run zig build for zig-agents: {}", e);
            }
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.zig");
    println!("cargo:rerun-if-changed=build.zig.zon");
}
