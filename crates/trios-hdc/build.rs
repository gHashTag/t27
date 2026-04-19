use std::path::PathBuf;
use std::process::Command;

fn main() {
    let zig_path = "vendor/zig-hdc";

    if !PathBuf::from(zig_path).join("build.zig").exists() {
        println!("cargo:warning=zig-hdc vendor not found, skipping Zig build");
        return;
    }

    let status = Command::new("zig")
        .args(["build", "-Doptimize=ReleaseFast"])
        .current_dir(zig_path)
        .status();

    match status {
        Ok(s) if s.success() => {
            let lib_dir = std::env::current_dir().unwrap().join(zig_path).join("zig-out/lib");
            let static_lib = lib_dir.join("libhdc.a");
            if static_lib.exists() {
                println!("cargo:rustc-link-search=native={}", lib_dir.display());
                println!("cargo:rustc-link-lib=static=hdc");
            } else {
                println!("cargo:warning=zig build succeeded but static library not found at {:?}", static_lib);
            }
        }
        Ok(s) => {
            println!("cargo:warning=zig build failed for zig-hdc (exit {:?})", s.code());
        }
        Err(e) => {
            println!("cargo:warning=failed to run zig build for zig-hdc: {}", e);
        }
    }

    println!("cargo:rerun-if-changed={}/src", zig_path);
}
