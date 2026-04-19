use std::path::PathBuf;
use std::process::Command;

fn main() {
    let zig_path = "vendor/zig-hdc";
    
    // Sacred geometry symbols
    if PathBuf::from(zig_path).join("build.zig.zon")).exists() {
        println!("cargo:rustc-link-search=native={}", zig_path);
    }
    
    // HDC symbols
    if PathBuf::from(zig_path).join("src/sequence_hdc.zig")).exists() {
        println!("cargo:rustc-link-search=native={}", zig_path);
    }
    
    if PathBuf::from(zig_path).join("src/vsa.zig")).exists() {
        println!("cargo:rustc-link-search=native={}", zig_path);
    }
}
