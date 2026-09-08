//! A struct whose fields are all `Copy` must itself be `Copy` -- including
//! when a field's type is another declared struct or an enum.
//!
//! `gen-rust` passes a struct parameter BY VALUE. A non-`Copy` struct is
//! therefore MOVED into the call, and a caller that reads the same value twice
//! does not compile. The derive used to ask a static helper that knows the
//! scalars and `[T; N]` but nothing about declared types, so:
//!
//!     struct Inner { a : i32 }     // all scalars    -> Copy
//!     struct Outer { i : Inner }   // a struct field -> NOT Copy
//!     (first(o), second(o))        // error[E0382]: use of moved value: `o`
//!
//! The assertions below are not on the derive text alone. A derive that reads
//! right can still leave the caller un-compilable -- the text says `Copy` is
//! present, and then `rustc` is asked whether the two-read caller is actually
//! accepted, and whether it is actually rejected when the struct genuinely
//! cannot be `Copy`.

use std::process::Command;

fn rustc_present() -> bool {
    Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn dir_for(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("t27c-tcopy-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("create temp dir");
    d
}

fn gen_rust(spec: &str, dir: &std::path::Path) -> String {
    let spec_path = dir.join("in.t27");
    std::fs::write(&spec_path, spec).expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-rust")
        .arg(&spec_path)
        .output()
        .expect("run t27c");
    assert!(
        out.status.success(),
        "gen-rust failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// Compile `generated + extra` as a library and return rustc's stderr.
///
/// `--emit=metadata` with a real output path, NOT `-o /dev/null`: rustc cannot
/// create its temporary directory beside `/dev/null` and fails with an error
/// that has nothing to do with the code under test. That false signal cost a
/// measurement here once already.
fn compile(generated: &str, extra: &str, dir: &std::path::Path, tag: &str) -> String {
    let src = format!("{generated}\n{extra}\n");
    let rs = dir.join(format!("{tag}.rs"));
    std::fs::write(&rs, &src).expect("write rs");
    let out = Command::new("rustc")
        .args(["--edition", "2021", "--crate-type", "lib", "--crate-name", "m", "-A", "warnings"])
        .arg("--emit=metadata")
        .arg("-o")
        .arg(dir.join(format!("{tag}.rmeta")))
        .arg(&rs)
        .output()
        .expect("run rustc");
    String::from_utf8_lossy(&out.stderr).to_string()
}

const NESTED: &str = r#"
module nest {
    struct Inner {
        a : i32,
    }
    struct Outer {
        i : Inner,
        b : i32,
    }
    fn first(o: Outer) -> i32 { return o.b; }
    fn second(o: Outer) -> i32 { return o.i.a; }
}
"#;

#[test]
fn a_struct_of_copy_structs_is_copy() {
    let dir = dir_for("nested");
    let src = gen_rust(NESTED, &dir);
    let outer = src
        .split("pub struct Outer")
        .next()
        .expect("Outer must be emitted")
        .rsplit("#[derive(")
        .next()
        .expect("Outer must carry a derive");
    assert!(
        outer.starts_with("Debug, Clone, Copy)"),
        "Outer's fields are all Copy, so Outer must be Copy; got derive({outer:.40})"
    );
}

#[test]
fn the_caller_may_read_the_same_value_twice() {
    if !rustc_present() {
        eprintln!("SKIP the_caller_may_read_the_same_value_twice: no rustc on PATH");
        return;
    }
    let dir = dir_for("twice");
    let src = gen_rust(NESTED, &dir);

    // Control: the generated module must compile on its own, or the probe
    // below would be measuring someone else's error.
    let alone = compile(&src, "", &dir, "alone");
    assert!(
        !alone.contains("error"),
        "the generated module must compile by itself:\n{alone}"
    );

    let twice = compile(
        &src,
        "pub fn probe(o: Outer) -> (i32, i32) { (first(o), second(o)) }",
        &dir,
        "twice",
    );
    assert!(
        !twice.contains("E0382"),
        "reading a Copy struct twice must compile:\n{twice}"
    );
    assert!(
        !twice.contains("error"),
        "the two-read caller must compile:\n{twice}"
    );
}

#[test]
fn an_enum_typed_field_also_qualifies_its_owner() {
    // Every enum is emitted `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`
    // unconditionally, so a field of enum type cannot be what stops its owner.
    let dir = dir_for("enumfield");
    let src = gen_rust(
        r#"
module ef {
    enum Colour { Red, Green }
    struct Holder {
        c : Colour,
        n : u8,
    }
    fn n_of(h: Holder) -> u8 { return h.n; }
}
"#,
        &dir,
    );
    let holder = src
        .split("pub struct Holder")
        .next()
        .expect("Holder must be emitted")
        .rsplit("#[derive(")
        .next()
        .expect("Holder must carry a derive");
    assert!(
        holder.starts_with("Debug, Clone, Copy)"),
        "an enum-typed field must not disqualify its owner; got derive({holder:.40})"
    );
}

#[test]
fn an_array_of_copy_structs_qualifies_its_owner() {
    let dir = dir_for("arrfield");
    let src = gen_rust(
        r#"
module af {
    struct Cell {
        v : u32,
    }
    struct Grid {
        cells : [4]Cell,
        used : u32,
    }
    fn used_of(g: Grid) -> u32 { return g.used; }
}
"#,
        &dir,
    );
    let grid = src
        .split("pub struct Grid")
        .next()
        .expect("Grid must be emitted")
        .rsplit("#[derive(")
        .next()
        .expect("Grid must carry a derive");
    assert!(
        grid.starts_with("Debug, Clone, Copy)"),
        "[Cell; 4] where Cell is Copy must not disqualify Grid; got derive({grid:.40})"
    );
}

#[test]
fn a_struct_that_cannot_be_copy_does_not_qualify_its_owner() {
    // The negative half, and it has to reach one level DEEPER than it looks.
    //
    // A first version asserted only that `Bag` -- which holds a growable field
    // -- is not `Copy`. A mutant admitting EVERY struct to the fixpoint passed
    // it, because the derive asks about a FIELD's type (`Vec<u8>`) and never
    // about `Bag`'s own name. The mutant was invisible until a struct was
    // given `Bag` as a field. So `Holder` is the control, and rustc is asked
    // whether the result compiles -- `#[derive(Copy)]` on a struct holding a
    // `Vec` is an error, which is precisely what a too-permissive fixpoint
    // would emit.
    if !rustc_present() {
        eprintln!("SKIP a_struct_that_cannot_be_copy_does_not_qualify_its_owner: no rustc");
        return;
    }
    let dir = dir_for("negative");
    let src = gen_rust(
        r#"
module neg {
    struct Bag {
        items : []u8,
        n : u32,
    }
    struct Holder {
        b : Bag,
        k : u32,
    }
    fn k_of(h: Holder) -> u32 { return h.k; }
}
"#,
        &dir,
    );
    for name in ["Bag", "Holder"] {
        let d = src
            .split(&format!("pub struct {name}"))
            .next()
            .unwrap_or_else(|| panic!("{name} must be emitted"))
            .rsplit("#[derive(")
            .next()
            .unwrap_or_else(|| panic!("{name} must carry a derive"));
        assert!(
            !d.starts_with("Debug, Clone, Copy)"),
            "{name} cannot be Copy -- it reaches a growable field; got derive({d:.40})"
        );
    }
    let out = compile(&src, "", &dir, "negative");
    assert!(
        !out.contains("error"),
        "a wrongly-derived Copy would not compile; generated module said:\n{out}"
    );
}
