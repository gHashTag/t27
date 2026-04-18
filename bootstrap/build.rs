//! Hard language guard: fail `cargo build` if Cyrillic appears in specs or Rust sources.
//! Scope: ONLY .t27/.tri specs and .rs sources. .md documentation is OUT OF SCOPE.
//! See docs/nona-03-manifest/SOUL.md Law #1, docs/T27-CONSTITUTION.md Article LANG-EN.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const ERR_HEAD: &str = "t27c LANGUAGE POLICY VIOLATION";

fn is_cyrillic(c: char) -> bool {
    matches!(c, '\u{0400}'..='\u{04ff}')
}

fn collect_files(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            collect_files(&p, ext, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some(ext) {
            out.push(p);
        }
    }
}

fn scan_cyrillic(
    path: &Path,
    rel_posix: &str,
) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|e| {
        format!("{ERR_HEAD}: cannot read {rel_posix}: {e}\nSee docs/nona-03-manifest/SOUL.md Law #1.")
    })?;
    for (line_no, line) in content.lines().enumerate() {
        for (col, c) in line.chars().enumerate() {
            if is_cyrillic(c) {
                let snippet: String = line.chars().take(120).collect();
                return Err(format!(
                    "{ERR_HEAD}: Cyrillic character U+{:04X} ('{}') in file {}\n\
                     Location: line {}, column {}\n\
                     Snippet: {}\n\
                     Fix: use English only in .t27/.tri specs and .rs sources.\n\
                     Docs: docs/nona-03-manifest/SOUL.md Law #1, docs/T27-CONSTITUTION.md (LANG-EN).",
                    c as u32,
                    c,
                    rel_posix,
                    line_no + 1,
                    col + 1,
                    snippet
                ));
            }
        }
    }
    Ok(())
}

fn rel_from_root(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

fn rerun_line(_manifest_dir: &Path, root: &Path, file: &Path) {
    let rel = file.strip_prefix(root).unwrap_or(file);
    let flag = Path::new("..").join(rel);
    let s = flag.to_string_lossy().to_string();
    println!("cargo:rerun-if-changed={}", s);
}

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .expect("bootstrap crate must live one level below repo root")
        .to_path_buf();

    // --- Bootstrap compiler sources: no Cyrillic in repo-owned Rust ---
    let boot_src = manifest_dir.join("src");
    if boot_src.is_dir() {
        let mut rs_files = Vec::new();
        collect_files(&boot_src, "rs", &mut rs_files);
        for path in &rs_files {
            let rel = rel_from_root(&root, path);
            if let Err(msg) = scan_cyrillic(path, &rel) {
                panic!("{msg}");
            }
            rerun_line(&manifest_dir, &root, path);
        }
    }
    let boot_tests = manifest_dir.join("tests");
    if boot_tests.is_dir() {
        let mut rs_files = Vec::new();
        collect_files(&boot_tests, "rs", &mut rs_files);
        for path in &rs_files {
            let rel = rel_from_root(&root, path);
            if let Err(msg) = scan_cyrillic(path, &rel) {
                panic!("{msg}");
            }
            rerun_line(&manifest_dir, &root, path);
        }
    }

    // --- .t27 / .tri under specs/: no Cyrillic ever (strict SSOT requirement) ---
    let specs = root.join("specs");
    if specs.is_dir() {
        let mut spec_files = Vec::new();
        collect_files(&specs, "t27", &mut spec_files);
        collect_files(&specs, "tri", &mut spec_files);
        for path in &spec_files {
            let rel = rel_from_root(&root, path);
            if let Err(msg) = scan_cyrillic(path, &rel) {
                panic!("{msg}");
            }
            rerun_line(&manifest_dir, &root, path);
        }
    }

    // Note: .md documentation files are OUT OF SCOPE for LANG-EN enforcement.
    // Documentation may be in any language. Only .t27/.tri specs and .rs sources are checked.

    println!("cargo:rerun-if-changed=build.rs");
}
