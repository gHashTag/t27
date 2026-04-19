use std::path::PathBuf;
use std::process::Command;

fn main() {
    let zig_path = "vendor/zig-crypto-mining";

    if !PathBuf::from(zig_path).join("build.zig").exists() {
        println!("cargo:warning=zig-crypto-mining vendor not found, skipping Zig build");
        return;
    }

    let status = Command::new("zig")
        .args(["build", "-Doptimize=ReleaseFast"])
        .current_dir(zig_path)
        .status()
        .expect("Failed to execute zig build");

    assert!(status.success(), "zig build failed for zig-crypto-mining");

    println!("cargo:rustc-link-search=native={}/zig-out/lib", zig_path);
    println!("cargo:rustc-link-lib=static=crypto_mining");
    println!("cargo:rerun-if-changed={}/src", zig_path);
}
