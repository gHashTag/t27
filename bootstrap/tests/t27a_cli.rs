//! `t27c asm` / `t27c disasm` (#6507), and the two gen-rust lowerings they
//! needed.
//!
//! The assembler is specs/isa/t27a.t27, lowered by `t27c gen-rust` into the
//! checked-in bootstrap/gen/rust/isa/t27a.rs; main.rs only splits lines and
//! prints. So the tests check three things:
//!
//! - the checked-in Rust is byte-for-byte what gen-rust writes from the spec
//!   today, so the spec stays the only home of the T736 field table;
//! - the CLI gives the words and listings the spec's own tests pin, and the
//!   two commands invert each other through a pipe, including `.word N` for
//!   undefined bytes;
//! - gen-rust indexes a `string` through `.as_bytes()` and bridges `x.len`
//!   (usize) against a u32: without both, t27a.rs did not compile (five E0277
//!   and six E0308).
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn t27c(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .args(args)
        .current_dir(repo_root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run t27c");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(stdin.as_bytes())
        .expect("write stdin");
    child.wait_with_output().expect("wait for t27c")
}

fn ok(out: &Output) -> String {
    assert!(
        out.status.success(),
        "t27c failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout.clone()).expect("utf-8 stdout")
}

/// The words specs/isa/t27a.t27 pins in `assembles_the_words_the_encoding_spec_pins`
/// and lists back in `lists_the_words_the_encoding_spec_pins`.
const PINNED: [(&str, u32); 6] = [
    ("ADD t3, t1, t2", 533264),
    ("LDI t3, t0, -5", 4294312708),
    ("MOV t3, t1", 8975),
    ("JGT t2, t1, 7", 926276),
    ("BUNDLE3 t3, t1, t2, t4", 34087779),
    ("NOP t0, t0", 0),
];

#[test]
fn the_checked_in_rust_is_what_gen_rust_writes_from_the_spec() {
    let fresh = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .args(["gen-rust", "specs/isa/t27a.t27"])
        .current_dir(repo_root())
        .output()
        .expect("run t27c gen-rust");
    assert!(
        fresh.status.success(),
        "gen-rust failed: {}",
        String::from_utf8_lossy(&fresh.stderr)
    );
    let checked_in = std::fs::read(repo_root().join("bootstrap/gen/rust/isa/t27a.rs"))
        .expect("read bootstrap/gen/rust/isa/t27a.rs");
    assert!(
        fresh.stdout == checked_in,
        "bootstrap/gen/rust/isa/t27a.rs drifted from specs/isa/t27a.t27: regenerate it with \
         `t27c gen-rust specs/isa/t27a.t27 > bootstrap/gen/rust/isa/t27a.rs`, never hand-edit it"
    );
}

#[test]
fn asm_writes_the_words_the_spec_pins() {
    let src: String = PINNED.iter().map(|(t, _)| format!("{t}\n\n")).collect();
    let want: String = PINNED.iter().map(|(_, w)| format!("0x{w:08x}\n")).collect();
    assert_eq!(ok(&t27c(&["asm"], &src)), want);
    assert_eq!(ok(&t27c(&["asm", "-"], &src)), want);
}

#[test]
fn disasm_lists_the_words_the_spec_pins() {
    let words: Vec<String> = PINNED.iter().map(|(_, w)| format!("0x{w:08x}")).collect();
    let args: Vec<&str> = std::iter::once("disasm")
        .chain(words.iter().map(|s| s.as_str()))
        .collect();
    let want: String = PINNED.iter().map(|(t, _)| format!("{t}\n")).collect();
    assert_eq!(ok(&t27c(&args, "")), want);
    let decimal: String = PINNED.iter().map(|(_, w)| format!("{w} ")).collect();
    assert_eq!(ok(&t27c(&["disasm"], &decimal)), want);
}

#[test]
fn disasm_then_asm_gives_every_word_back() {
    // Every low byte (47 opcodes and 209 undefined bytes), then pseudo-random
    // words with high bits set, most of them non-canonical.
    let mut words: Vec<u32> = (0..256).collect();
    let mut x: u32 = 12345;
    for _ in 0..2048 {
        x = x.wrapping_mul(1103515245).wrapping_add(12345);
        words.push(x);
    }
    let input: String = words.iter().map(|w| format!("0x{w:08x}\n")).collect();
    let listing = ok(&t27c(&["disasm"], &input));
    assert_eq!(listing.lines().count(), words.len());
    assert!(
        listing.lines().nth(255).expect("byte 255").starts_with(".word "),
        "an undefined byte must list as `.word N`, not as an instruction"
    );
    assert_eq!(ok(&t27c(&["asm"], &listing)), input);
}

#[test]
fn asm_refuses_ill_formed_text_and_names_the_line() {
    let out = t27c(&["asm"], "ADD t3, t1, t2\nADD t27, t1, t2\nadd t3, t1, t2\n");
    assert!(!out.status.success(), "ill-formed text must fail");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("<stdin>:2: register"), "{err}");
    assert!(err.contains("<stdin>:3: unknown mnemonic"), "{err}");
    assert!(err.contains("2 line(s) did not assemble"), "{err}");
}

#[test]
fn disasm_refuses_a_token_that_is_not_a_word() {
    for bad in ["0x1_0000_0000", "4294967296", "t3", "0xZZ"] {
        let out = t27c(&["disasm", bad], "");
        assert!(!out.status.success(), "`{bad}` must be refused");
    }
}

/// gen-rust on a spec that indexes a `string` and compares `.len` with a u32,
/// compiled by rustc with warnings denied and executed.
#[test]
fn gen_rust_indexes_a_string_as_bytes_and_bridges_its_len() {
    let dir = std::env::temp_dir().join(format!(
        "t27c-t27a-strlen-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).expect("scratch dir");
    let spec = dir.join("fixture.t27");
    std::fs::write(
        &spec,
        "module StrBytes {\n\
         \x20   pub const NAMES : [2]str = [\"AB\", \"XYZ\"];\n\
         \x20   pub fn sum(text: string) -> u32 {\n\
         \x20       var k : u32 = 0;\n\
         \x20       var s : u32 = 0;\n\
         \x20       while (k < text.len) { s = s + (text[k] as u32); k = k + 1; }\n\
         \x20       return s;\n\
         \x20   }\n\
         \x20   pub fn name_char(i: u32, k: u32) -> u32 {\n\
         \x20       var name : string = NAMES[i];\n\
         \x20       if (k < name.len) { return name[k] as u32; }\n\
         \x20       return 0;\n\
         \x20   }\n\
         \x20   test sums { assert sum(\"AB\") == 131; }\n\
         }\n",
    )
    .expect("write fixture");
    let generated = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-rust")
        .arg(&spec)
        .output()
        .expect("run gen-rust");
    assert!(
        generated.status.success(),
        "gen-rust failed: {}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let mut rust = generated.stdout;
    rust.extend_from_slice(
        b"\nfn main() {\n    assert_eq!(sum(\"AB\"), 131);\n    assert_eq!(sum(\"\"), 0);\n    \
          assert_eq!(name_char(1, 2), 90);\n    assert_eq!(name_char(0, 2), 0);\n}\n",
    );
    let file = dir.join("fixture.rs");
    std::fs::write(&file, &rust).expect("write generated Rust");
    let binary = dir.join("fixture");
    let compiled = Command::new("rustc")
        // gen-rust always writes `let mut` and wraps operands in parentheses;
        // main.rs allows the same lints on t27a.rs. Everything else is denied.
        .args([
            "--edition=2021",
            "-D",
            "warnings",
            "-A",
            "unused-parens",
            "-A",
            "unused-mut",
            "-A",
            "dead-code",
        ])
        .arg(&file)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("run rustc");
    assert!(
        compiled.status.success(),
        "generated Rust must compile:\n{}\n{}",
        String::from_utf8_lossy(&compiled.stderr),
        String::from_utf8_lossy(&rust)
    );
    let run = Command::new(&binary).output().expect("run fixture");
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        run.status.success(),
        "generated Rust gave wrong answers: {}",
        String::from_utf8_lossy(&run.stderr)
    );
}
