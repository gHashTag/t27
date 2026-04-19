use std::path::PathBuf;
use std::process::Command;

fn main() {
    let zig_path = "vendor/zig-sacred-geometry";

    if !PathBuf::from(zig_path).join("build.zig").exists() {
        println!("cargo:warning=zig-sacred-geometry vendor not found, skipping Zig build");
        return;
    }

    let status = Command::new("zig")
        .args(["build", "-Doptimize=ReleaseFast"])
        .current_dir(zig_path)
        .status()
        .expect("Failed to execute zig build");

    assert!(status.success(), "zig build failed for zig-sacred-geometry");

    println!("cargo:rustc-link-search=native={}/zig-out/lib", zig_path);
    println!("cargo:rustc-link-lib=static=sacred_geometry");
    println!("cargo:rerun-if-changed={}/src", zig_path);
}
