use std::path::PathBuf;
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let subcmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match subcmd {
        "build-ext" => build_ext(),
        "help" | _ => {
            eprintln!("Usage: cargo xtask <command>");
            eprintln!("  build-ext  Build trios-ext WASM + wasm-bindgen → dist/");
        }
    }
}

fn build_ext() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let ext_dir = PathBuf::from(&manifest_dir).join("..");
    let dist_dir = ext_dir.join("extension").join("dist");

    eprintln!("[xtask] Building trios-ext for wasm32-unknown-unknown...");
    let status = Command::new("cargo")
        .args([
            "build",
            "-p",
            "trios-ext",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
        ])
        .current_dir(&ext_dir)
        .status()
        .expect("failed to run cargo build");
    assert!(status.success(), "cargo build failed");

    let wasm_input = ext_dir.join("../../target/wasm32-unknown-unknown/release/trios_ext.wasm");

    if !wasm_input.exists() {
        eprintln!("[xtask] ERROR: {} not found", wasm_input.display());
        std::process::exit(1);
    }

    std::fs::create_dir_all(&dist_dir).expect("create dist dir");

    eprintln!("[xtask] Running wasm-bindgen...");
    let status = Command::new("wasm-bindgen")
        .arg("--target")
        .arg("web")
        .arg("--out-dir")
        .arg(&dist_dir)
        .arg("--no-modules-global")
        .arg("trios_ext_init")
        .arg(&wasm_input)
        .status()
        .expect("failed to run wasm-bindgen");

    if !status.success() {
        eprintln!("[xtask] wasm-bindgen failed. Trying with --target web...");
        let status2 = Command::new("wasm-bindgen")
            .arg("--target")
            .arg("no-modules")
            .arg("--out-dir")
            .arg(&dist_dir)
            .arg(&wasm_input)
            .status()
            .expect("failed to run wasm-bindgen (attempt 2)");
        assert!(status2.success(), "wasm-bindgen failed on both attempts");
    }

    let js_out = dist_dir.join("trios_ext.js");
    let wasm_out = dist_dir.join("trios_ext_bg.wasm");
    if js_out.exists() && wasm_out.exists() {
        let js_size = std::fs::metadata(&js_out).map(|m| m.len()).unwrap_or(0);
        let wasm_size = std::fs::metadata(&wasm_out).map(|m| m.len()).unwrap_or(0);
        eprintln!(
            "[xtask] SUCCESS: dist/trios_ext.js ({:.1} KB) + dist/trios_ext_bg.wasm ({:.1} KB)",
            js_size as f64 / 1024.0,
            wasm_size as f64 / 1024.0
        );
    } else {
        eprintln!("[xtask] WARNING: expected output files not found");
    }
}
