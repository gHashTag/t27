use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=../gen/c/numeric");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let repo_root = manifest_dir.parent().expect("ffi should be at repo root");
    let c_src_dir = repo_root.join("gen/c/numeric");

    // Compile generated C code into a static library (for future C consumers)
    if c_src_dir.exists() {
        let mut build = cc::Build::new();

        // Add all C source files
        for c_file in [
            "gf4.c", "gf8.c", "gf12.c", "gf16.c", "gf20.c", "gf24.c", "gf32.c",
            "goldenfloat_family.c", "phi_ratio.c", "tf3.c"
        ] {
            build.file(c_src_dir.join(c_file));
        }

        build
            .include(&c_src_dir)
            .warnings_into_errors(true)
            .compile("goldenfloat_c");

        // Link to compiled C library
        println!("cargo:rustc-link-lib=static=goldenfloat_c");
        println!("cargo:rustc-link-search={}", out_dir.display());
    }
}
