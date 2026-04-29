use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=../gen/c/numeric");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let repo_root = manifest_dir.parent().expect("ffi should be at repo root");

    // Only compile C code if CC_FORCE_DISABLE is not set
    if env::var("CC_FORCE_DISABLE").is_err() {
        let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
        let c_src_dir = repo_root.join("gen/c/numeric");

        let mut build = cc::Build::new();

        for c_file in [
            "gf4.c", "gf8.c", "gf12.c", "gf16.c", "gf20.c", "gf24.c", "gf32.c",
            "goldenfloat_family.c", "phi_ratio.c", "tf3.c"
        ] {
            let path = c_src_dir.join(c_file);
            if path.exists() {
                build.file(path);
            }
        }

        build
            .include(&c_src_dir)
            .warnings_into_errors(false)
            .compile("goldenfloat_c");

        println!("cargo:rustc-link-lib=static=goldenfloat_c");
        println!("cargo:rustc-link-search={}", out_dir.display());
    }

    // Generate unified C header using cbindgen
    let header_path = repo_root.join("include/golden_float.h");

    cbindgen::Builder::new()
        .with_crate(&manifest_dir)
        .with_language(cbindgen::Language::C)
        .with_pragma_once(true)
        .with_include_guard("GOLDEN_FLOAT_H")
        .with_sys_include("stdint.h")
        .with_sys_include("stdbool.h")
        .with_sys_include("math.h")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(header_path);
}
