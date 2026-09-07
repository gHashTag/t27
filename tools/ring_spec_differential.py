#!/usr/bin/env python3
"""Run a ring and its spec's generated Rust on the same inputs and compare.

`check_ring_spec_drift.py` reports CONVERGED when every shared function has an
identical signature, and carries a warning it earned: **a matching signature is
not matching behaviour.** `ring-090` measured CONVERGED at 16 of 16 identical
and still disagreed with its spec on 126 of 1190 cases -- the spec wrapped where
the hand-written model saturated (#3420). Only compiling and running both found
it, and that harness was written by hand.

This writes the harness. For a CONVERGED pair it emits a Rust program that
includes both modules, calls every shared function with the same synthesised
inputs, and compares what comes back.

WHAT IT REFUSES. A function whose parameter or return type it cannot synthesise
or compare is NOT skipped -- it is named, and the run exits 2. A harness that
quietly covered eight of sixteen functions and reported "agree" would be worse
than no harness: the eight it dropped are exactly where a difference could hide.

Synthesis rules, deliberately small:
  * integers    -- a fixed grid including 0, 1, the type maximum and its
                   neighbours, because the ring-090 defect lived at 2e9
  * bool        -- both
  * &'static str-- a short list including the empty string
  * a struct    -- built by calling a PRODUCER: a same-module function whose
                   return type is that struct, driven by the same grid. This is
                   how the hand-written harness built SimConfig and SimResult,
                   and it needs no knowledge of the struct's fields
  * &mut [T]    -- a local array, compared element-wise after the call
  * &mut T      -- a local, compared after the call

Usage:
  tools/ring_spec_differential.py <ring-dir> <spec>   emit, build and run
  tools/ring_spec_differential.py --self-check        negative control
"""

import os
import re
import subprocess
import sys
import tempfile

SIG = re.compile(r"^pub (?:const )?fn ([a-z_][a-z_0-9]*)\s*\(([^)]*)\)\s*->\s*([^{;]+)", re.M)
INT_TYPES = {"u8", "u16", "u32", "u64", "usize", "i8", "i16", "i32", "i64", "isize"}

GRID = {
    "u8": ["0", "1", "7", "255"],
    "u16": ["0", "1", "1000", "65535"],
    "u32": ["0", "1", "1000", "1000000", "2000000000", "u32::MAX"],
    "u64": ["0", "1", "1000000", "u64::MAX"],
    "usize": ["0", "1", "7", "1000"],
    "i8": ["-128", "-1", "0", "127"],
    "i16": ["-1", "0", "1000"],
    "i32": ["-2147483648", "-1", "0", "1", "2147483647"],
    "i64": ["-1", "0", "1000000"],
    "isize": ["-1", "0", "1000"],
    "bool": ["false", "true"],
    "&'static str": ['""', '"sim"', '"a_longer_name"'],
}


def signatures(text: str) -> dict:
    out = {}
    for m in SIG.finditer(text):
        params = []
        for p in m.group(2).split(","):
            p = p.strip()
            if not p:
                continue
            name, ty = p.split(":", 1)
            params.append((name.strip(), ty.strip()))
        out[m.group(1)] = (params, m.group(3).strip())
    return out


def producers(sigs: dict) -> dict:
    """Return type -> a function that returns it, taking only synthesisable params."""
    out = {}
    for fn, (params, ret) in sigs.items():
        if ret in GRID or ret in INT_TYPES or ret == "bool":
            continue
        if all(t in GRID for _, t in params):
            out.setdefault(ret, (fn, params))
    return out


def comparable(ty: str, structs: set) -> bool:
    return ty in GRID or ty in INT_TYPES or ty in ("bool", "f32", "f64") or ty in structs


def emit(ring_lib: str, spec_rs: str, sigs: dict, prod: dict) -> tuple:
    """Return (rust source, covered fns, refused [(fn, reason)])."""
    structs = set(prod)
    covered, refused, body = [], [], []
    for fn in sorted(sigs):
        params, ret = sigs[fn]
        why = None
        for _, t in params:
            if t in GRID:
                continue
            if t in prod:
                continue
            if t.startswith("&mut "):
                continue
            why = f"parameter type {t}"
            break
        if why is None and not comparable(ret, structs):
            why = f"return type {ret}"
        if why:
            refused.append((fn, why))
            continue
        covered.append(fn)

        loops, args_h, args_s, post, pre = [], [], [], [], []
        for i, (pname, t) in enumerate(params):
            v = f"v{i}"
            if t in GRID:
                loops.append((v, GRID[t]))
                args_h.append(v)
                args_s.append(v)
            elif t in prod:
                pfn, pparams = prod[t]
                sub = []
                for j, (_, pt) in enumerate(pparams):
                    w = f"{v}_{j}"
                    loops.append((w, GRID[pt]))
                    sub.append(w)
                a = ", ".join(sub)
                pre.append(f"let {v}h = hand::{pfn}({a}); let {v}s = spec::{pfn}({a});")
                args_h.append(f"{v}h")
                args_s.append(f"{v}s")
            elif t.startswith("&mut ["):
                el = t[6:-1]
                zero = "false" if el == "bool" else "0"
                pre.append(f"let mut {v}h: [{el}; 8] = [{zero}; 8]; let mut {v}s: [{el}; 8] = [{zero}; 8];")
                args_h.append(f"&mut {v}h")
                args_s.append(f"&mut {v}s")
                post.append(f"&& {v}h == {v}s")
            else:  # &mut T
                el = t[5:]
                pre.append(f"let mut {v}h: {el} = 0; let mut {v}s: {el} = 0;")
                args_h.append(f"&mut {v}h")
                args_s.append(f"&mut {v}s")
                post.append(f"&& {v}h == {v}s")

        cmp_ = "rh == rs" if ret not in structs else "fields_eq(&rh, &rs)"
        if ret in structs:
            cmp_ = f"format!(\"{{:?}}\", rh) == format!(\"{{:?}}\", rs)"
        # Both calls run under catch_unwind. A function that PANICS in one
        # module and returns in the other is the sharpest disagreement there
        # is, and without this the process dies with no output at all -- which
        # reads as "no cases run" rather than as a difference. Overflow checks
        # are on for the same reason: a release build would wrap silently in
        # both and hide it.
        inner = (
            "\n".join("        " + p for p in pre)
            + f"\n        let rh = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| hand::{fn}({', '.join(args_h)})));"
            + f"\n        let rs = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| spec::{fn}({', '.join(args_s)})));"
            + f"\n        n += 1;"
            + "\n        let agree = match (&rh, &rs) {"
            + f"\n            (Ok(a), Ok(b)) => {{ let (rh, rs) = (a, b); {cmp_} }}"
            + "\n            (Err(_), Err(_)) => true,"
            + "\n            _ => false,"
            + "\n        };"
            + f"\n        if !(agree {' '.join(post)}) {{ bad += 1;"
            + f' if first.is_empty() {{ first = format!("{fn}"); }} }}'
        )
        for v, vals in reversed(loops):
            arr = ", ".join(vals)
            inner = f"    for {v} in [{arr}] {{\n{inner}\n    }}"
        body.append("    {\n" + inner + "\n    }")

    src = f"""#[allow(dead_code, unused_mut, non_snake_case, unused_variables)] mod hand {{ include!("{ring_lib}"); }}
#[allow(dead_code, unused_mut, non_snake_case, unused_variables)] mod spec {{ include!("{spec_rs}"); }}
fn main() {{
    // Silence the per-panic backtrace: a panic here is DATA, not a crash.
    std::panic::set_hook(Box::new(|_| {{}}));
    let mut n = 0usize; let mut bad = 0usize; let mut first = String::new();
{chr(10).join(body)}
    println!("cases {{n}}  agree {{}}  disagree {{bad}}", n - bad);
    if bad > 0 {{ println!("first disagreement: {{first}}"); }}
    println!("harness control (1 != 2): {{}}", 1u32 != 2u32);
    std::process::exit(if bad == 0 {{ 0 }} else {{ 1 }});
}}
"""
    return src, covered, refused


def t27c() -> str:
    for p in ("target/release/t27c", "bootstrap/target/release/t27c"):
        if os.path.isfile(p) and os.access(p, os.X_OK):
            return p
    print("ring_spec_differential: t27c not built. Exit 2 = COULD NOT RUN.", file=sys.stderr)
    sys.exit(2)


def run(ring: str, spec: str) -> int:
    binary = t27c()
    lib = os.path.join(ring, "src", "lib.rs")
    if not os.path.isfile(lib) or not os.path.isfile(spec):
        print(f"ring_spec_differential: {lib} or {spec} missing. Exit 2.", file=sys.stderr)
        return 2
    gen = subprocess.run([binary, "gen-rust", spec], capture_output=True, text=True).stdout
    hand_src = open(lib, errors="replace").read()
    hand_src = hand_src.split("#[cfg(test)]")[0]
    # `include!` inside a `mod` rejects inner doc comments and inner attributes:
    # "inner doc comments can only appear before items", "an inner attribute is
    # not permitted in this context". Both crates start with them. Stripping is
    # reported, not silent -- it edits someone else's file to run the test, and
    # a reader has to be able to see what was removed.
    kept, dropped = [], 0
    for line in hand_src.split("\n"):
        t = line.lstrip()
        if t.startswith("//!") or t.startswith("#!["):
            dropped += 1
            continue
        kept.append(line)
    hand_src = "\n".join(kept)
    if dropped:
        print(f"  stripped {dropped} inner doc-comment/attribute line(s) from {lib} "
              f"so it can be included as a module")
    h, s = signatures(hand_src), signatures(gen)
    shared = {k: s[k] for k in set(h) & set(s) if h[k] == s[k]}
    if not shared:
        print(f"ring_spec_differential: no shared function with an identical signature. Exit 2.")
        return 2
    src, covered, refused = emit("hand.rs", "spec.rs", shared, producers(s))
    with tempfile.TemporaryDirectory() as d:
        open(os.path.join(d, "hand.rs"), "w").write(hand_src)
        open(os.path.join(d, "spec.rs"), "w").write("\n".join(
            l for l in gen.split("\n") if not l.startswith("//")))
        open(os.path.join(d, "main.rs"), "w").write(src)
        b = subprocess.run(
            ["rustc", "--edition", "2021", "-A", "warnings", "-C", "overflow-checks=on", "-o",
             os.path.join(d, "diff"), os.path.join(d, "main.rs")],
            capture_output=True, text=True, cwd=d)
        if b.returncode != 0:
            print("ring_spec_differential: the harness did not compile. Exit 2 = COULD NOT RUN.")
            print(b.stderr[:1500])
            return 2
        r = subprocess.run([os.path.join(d, "diff")], capture_output=True, text=True)
        print(r.stdout.strip())
    print(f"  covered {len(covered)} of {len(shared)} identical-signature functions")
    if refused:
        # Named, never silently dropped.
        print(f"  REFUSED {len(refused)} -- these were NOT tested:")
        for fn, why in refused:
            print(f"    {fn}: {why}")
        return 2
    return 0 if r.returncode == 0 else 1


def self_check() -> int:
    """Two modules that differ by construction must be reported as disagreeing."""
    good = "pub fn f(a: u32) -> u32 { a + 1 }\n"
    bad = "pub fn f(a: u32) -> u32 { a + 2 }\n"
    sigs = signatures(good)
    src, covered, refused = emit("hand.rs", "spec.rs", sigs, {})
    with tempfile.TemporaryDirectory() as d:
        open(os.path.join(d, "hand.rs"), "w").write(good)
        open(os.path.join(d, "spec.rs"), "w").write(bad)
        open(os.path.join(d, "main.rs"), "w").write(src)
        b = subprocess.run(["rustc", "--edition", "2021", "-A", "warnings", "-C", "overflow-checks=on", "-o",
                            os.path.join(d, "diff"), os.path.join(d, "main.rs")],
                           capture_output=True, text=True, cwd=d)
        if b.returncode != 0:
            print("  self-check: harness did not compile -- FAIL"); print(b.stderr[:600]); return 1
        r = subprocess.run([os.path.join(d, "diff")], capture_output=True, text=True)
    saw = r.returncode == 1 and "disagree 0" not in r.stdout
    print(f"  self-check: two modules that differ -> {r.stdout.strip().splitlines()[0]} "
          f"-- {'PASS' if saw else 'FAIL'}")
    # And the refusal path must fire on a type it cannot synthesise.
    _, _, ref = emit("h", "s", signatures("pub fn g(x: SomeUnknown) -> u32 { 0 }\n"), {})
    ok2 = len(ref) == 1
    print(f"  self-check: an unsynthesisable parameter is REFUSED -- {'PASS' if ok2 else 'FAIL'}")
    return 0 if (saw and ok2) else 1


def main() -> int:
    if "--self-check" in sys.argv:
        return self_check()
    if len(sys.argv) != 3:
        print(__doc__.strip().split("Usage:")[-1])
        return 2
    return run(sys.argv[1], sys.argv[2])


if __name__ == "__main__":
    sys.exit(main())
