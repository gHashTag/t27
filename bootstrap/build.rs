//! Hard language guard: fail `cargo build` if Cyrillic appears in specs or unlisted docs.
//! See docs/nona-03-manifest/SOUL.md Law #1, architecture/ADR-004-language-policy.md, docs/T27-CONSTITUTION.md Article LANG-EN.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const ERR_HEAD: &str = "t27c LANGUAGE POLICY VIOLATION";

fn is_cyrillic(c: char) -> bool {
    matches!(c, '\u{0400}'..='\u{04ff}')
}

fn load_allowlist(root: &Path) -> HashSet<String> {
    let p = root.join("docs/.legacy-non-english-docs");
    let mut set = HashSet::new();
    let Ok(txt) = fs::read_to_string(&p) else {
        return set;
    };
    for line in txt.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if !line.is_empty() {
            set.insert(line.replace('\\', "/"));
        }
    }
    set
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
    allow: &HashSet<String>,
) -> Result<(), String> {
    if allow.contains(rel_posix) {
        return Ok(());
    }
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
                     Fix: use English only in first-party sources and docs.\n\
                     Docs: docs/nona-03-manifest/SOUL.md Law #1, architecture/ADR-004-language-policy.md, docs/T27-CONSTITUTION.md (LANG-EN).\n\
                     If this file is grandfathered non-English, add its repo-relative path to docs/.legacy-non-english-docs (Architect approval only).",
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
    let allow = load_allowlist(&root);

    // --- Bootstrap compiler sources: no Cyrillic in repo-owned Rust ---
    let boot_src = manifest_dir.join("src");
    if boot_src.is_dir() {
        let mut rs_files = Vec::new();
        collect_files(&boot_src, "rs", &mut rs_files);
        for path in &rs_files {
            let rel = rel_from_root(&root, path);
            if let Err(msg) = scan_cyrillic(path, &rel, &HashSet::new()) {
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
            if let Err(msg) = scan_cyrillic(path, &rel, &HashSet::new()) {
                panic!("{msg}");
            }
            rerun_line(&manifest_dir, &root, path);
        }
    }

    // --- .t27 / .tri under specs/: no Cyrillic ever (no allowlist) ---
    let specs = root.join("specs");
    if specs.is_dir() {
        let mut spec_files = Vec::new();
        collect_files(&specs, "t27", &mut spec_files);
        collect_files(&specs, "tri", &mut spec_files);
        for path in &spec_files {
            let rel = rel_from_root(&root, path);
            if let Err(msg) = scan_cyrillic(path, &rel, &HashSet::new()) {
                panic!("{msg}");
            }
            rerun_line(&manifest_dir, &root, path);
        }
    }

    // --- First-party Markdown (same rules as CI script) ---
    for dir in ["docs", "architecture", "clara-bridge", "conformance"] {
        let base = root.join(dir);
        if !base.is_dir() {
            continue;
        }
        let mut md_files = Vec::new();
        collect_files(&base, "md", &mut md_files);
        for path in md_files {
            let rel = rel_from_root(&root, &path);
            if let Err(msg) = scan_cyrillic(&path, &rel, &allow) {
                panic!("{msg}");
            }
            rerun_line(&manifest_dir, &root, &path);
        }
    }
    // specs/**/*.md
    let specs_md = root.join("specs");
    if specs_md.is_dir() {
        let mut md_files = Vec::new();
        collect_files(&specs_md, "md", &mut md_files);
        for path in md_files {
            let rel = rel_from_root(&root, &path);
            if let Err(msg) = scan_cyrillic(&path, &rel, &allow) {
                panic!("{msg}");
            }
            rerun_line(&manifest_dir, &root, &path);
        }
    }
    for name in [
        "README.md",
        "AGENTS.md",
        "CLAUDE.md",
        "NOW.md",
        "SOUL.md",
        "OWNERS.md",
        "CONTRIBUTING.md",
        "SECURITY.md",
        "CODE_OF_CONDUCT.md",
    ] {
        let path = root.join(name);
        if path.is_file() {
            if let Err(msg) = scan_cyrillic(&path, name, &allow) {
                panic!("{msg}");
            }
            rerun_line(&manifest_dir, &root, &path);
        }
    }

    println!("cargo:rerun-if-changed=../docs/.legacy-non-english-docs");
    println!("cargo:rerun-if-changed=../NOW.md");
    println!("cargo:rerun-if-changed=build.rs");
}
