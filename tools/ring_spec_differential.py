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
  * f32, f64    -- a fixed grid with both zeros, ties, denormal-sized values,
                   the GF16 overflow edge, +/-Inf and NaN; compared by `{:?}`,
                   which round-trips a float exactly, tells -0.0 from 0.0 and
                   prints every NaN as `NaN` (#6893)
  * &'static str-- a short list including the empty string
  * an enum     -- every variant. The two modules' variants are paired by name,
                   case-insensitively (`Pos` and `pos`); an enum whose variants
                   do not pair one to one is REFUSED. Returns compare by the
                   paired variant, so neither side needs a derive (#6893)
  * a struct    -- built by calling a PRODUCER: a same-module function whose
                   return type is that struct, driven by the same grid. This is
                   how the hand-written harness built SimConfig and SimResult,
                   and it needs no knowledge of the struct's fields. A producer
                   that panics on one side only is a disagreement (#6893)
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
    # 0.6 / -0.75 sit either side of a ternary threshold; 1.0009765625 is a
    # GF16 rounding tie; 4.656612873077392578125e-10 is 2^-31, a GF16 denormal;
    # 5e-13 is below half the smallest one; 4290772992 is the largest finite
    # GF16 value and 4292870144 the tie above it.
    "f32": ["0.0", "-0.0", "0.5", "-0.5", "0.6", "-0.75", "1.0", "-1.5", "1.0009765625",
            "4.656612873077392578125e-10", "1e-12", "5e-13", "4290772992.0", "4292870144.0",
            "1e30", "f32::INFINITY", "f32::NEG_INFINITY", "f32::NAN"],
    "f64": ["0.0", "-0.0", "0.5", "-0.5", "1.0", "-1.5", "1e-12", "4290772992.0", "1e300",
            "f64::INFINITY", "f64::NEG_INFINITY", "f64::NAN"],
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


ENUM = re.compile(r"^pub enum ([A-Za-z_]\w*)\s*\{([^}]*)\}", re.M)


def enums(text: str) -> dict:
    """Enum name -> variant names in declaration order."""
    out = {}
    for m in ENUM.finditer(text):
        body = re.sub(r"//[^\n]*", "", m.group(2))
        body = re.sub(r"#\[[^\]]*\]", "", body)
        names = []
        for part in body.split(","):
            v = re.match(r"\s*([A-Za-z_]\w*)", part)
            if v:
                names.append(v.group(1))
        out[m.group(1)] = names
    return out


def pair_enums(hand: dict, spec: dict) -> tuple:
    """Enum name -> [(hand variant, spec variant)], and name -> why unpaired."""
    paired, unpaired = {}, {}
    for name in sorted(set(hand) | set(spec)):
        if name not in hand or name not in spec:
            unpaired[name] = "declared on one side only"
            continue
        h = {v.lower(): v for v in hand[name]}
        sp = {v.lower(): v for v in spec[name]}
        if len(h) != len(hand[name]) or len(sp) != len(spec[name]) or set(h) != set(sp):
            unpaired[name] = f"variants do not pair one to one: {hand[name]} vs {spec[name]}"
            continue
        paired[name] = [(h[k], sp[k]) for k in (v.lower() for v in spec[name])]
    return paired, unpaired


def producers(sigs: dict, enum_names=()) -> dict:
    """Return type -> a function that returns it, taking only synthesisable params."""
    out = {}
    for fn, (params, ret) in sigs.items():
        if ret in GRID or ret in INT_TYPES or ret == "bool" or ret in enum_names:
            continue
        if all(t in GRID for _, t in params):
            out.setdefault(ret, (fn, params))
    return out


def comparable(ty: str, structs: set) -> bool:
    return ty in GRID or ty in INT_TYPES or ty in ("bool", "f32", "f64") or ty in structs


def emit(ring_lib: str, spec_rs: str, sigs: dict, prod: dict, paired=None, unpaired=None) -> tuple:
    """Return (rust source, covered fns, refused [(fn, reason)])."""
    paired, unpaired = paired or {}, unpaired or {}
    structs = set(prod)
    covered, refused, body = [], [], []
    for fn in sorted(sigs):
        params, ret = sigs[fn]
        why = None
        for _, t in params:
            if t in GRID or t in paired:
                continue
            if t in unpaired:
                why = f"parameter type {t}: {unpaired[t]}"
                break
            if t in prod:
                continue
            if t.startswith("&mut "):
                continue
            why = f"parameter type {t}"
            break
        if why is None and ret in unpaired:
            why = f"return type {ret}: {unpaired[ret]}"
        if why is None and not (comparable(ret, structs) or ret in paired):
            why = f"return type {ret}"
        if why:
            refused.append((fn, why))
            continue
        covered.append(fn)

        loops, args_h, args_s, post, pre = [], [], [], [], []
        need_buf_loop = False
        for i, (pname, t) in enumerate(params):
            v = f"v{i}"
            if t in GRID:
                loops.append((v, GRID[t]))
                args_h.append(v)
                args_s.append(v)
            elif t in paired:
                loops.append((v, [str(k) + "usize" for k in range(len(paired[t]))]))
                args_h.append(f"enum_h_{t}({v})")
                args_s.append(f"enum_s_{t}({v})")
            elif t in prod:
                pfn, pparams = prod[t]
                sub = []
                for j, (_, pt) in enumerate(pparams):
                    w = f"{v}_{j}"
                    loops.append((w, GRID[pt]))
                    sub.append(w)
                a = ", ".join(sub)
                # The producer runs under catch_unwind too. Called bare, one
                # panicking side killed the whole harness with no output (#6893).
                pre.append(
                    f"let {v}h = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| hand::{pfn}({a}))); "
                    f"let {v}s = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| spec::{pfn}({a})));")
                pre.append(
                    f"let ({v}h, {v}s) = match ({v}h, {v}s) {{ (Ok(a), Ok(b)) => (a, b), "
                    f"(Err(_), Err(_)) => {{ n += 1; break 'case; }} "
                    f"_ => {{ n += 1; bad += 1; if first.is_empty() {{ first = format!(\"{fn} (producer {pfn} panicked on one side)\"); }} break 'case; }} }};")
                args_h.append(f"{v}h")
                args_s.append(f"{v}s")
            elif t.startswith("&mut ["):
                # A buffer parameter used to be one zero-filled `[T; 8]`, so the
                # 25 cases for ring-099 compared what came BACK while never
                # varying what went IN. Length and fill are now driven, and the
                # zero length is in the grid because an empty buffer is where an
                # off-by-one lives.
                #
                # All slice parameters of one call share the (len, fill) pair:
                # in the corpus they are parallel arrays, and giving each its
                # own loop would multiply the case count without adding a shape
                # the callee can distinguish.
                el = t[6:-1]
                need_buf_loop = True
                fill = "bfill != 0" if el == "bool" else f"bfill as {el}"
                pre.append(
                    f"let mut {v}h: Vec<{el}> = vec![{fill}; blen]; "
                    f"let mut {v}s: Vec<{el}> = vec![{fill}; blen];"
                )
                args_h.append(f"&mut {v}h[..]")
                args_s.append(f"&mut {v}s[..]")
                post.append(f"&& {v}h == {v}s")
            else:  # &mut T
                el = t[5:]
                pre.append(f"let mut {v}h: {el} = 0; let mut {v}s: {el} = 0;")
                args_h.append(f"&mut {v}h")
                args_s.append(f"&mut {v}s")
                post.append(f"&& {v}h == {v}s")

        cmp_ = "rh == rs"
        if ret in structs or ret in ("f32", "f64"):
            cmp_ = f"format!(\"{{:?}}\", rh) == format!(\"{{:?}}\", rs)"
        if ret in paired:
            cmp_ = f"idx_h_{ret}(rh) == idx_s_{ret}(rs)"
        # Both calls run under catch_unwind. A function that PANICS in one
        # module and returns in the other is the sharpest disagreement there
        # is, and without this the process dies with no output at all -- which
        # reads as "no cases run" rather than as a difference. Overflow checks
        # are on for the same reason: a release build would wrap silently in
        # both and hide it.
        inner = (
            "        'case: {\n"
            + "\n".join("        " + p for p in pre)
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
            + "\n        }"
        )
        for v, vals in reversed(loops):
            arr = ", ".join(vals)
            inner = f"    for {v} in [{arr}] {{\n{inner}\n    }}"
        if need_buf_loop:
            inner = (
                "    for blen in [0usize, 1, 4, 8] {\n"
                "    for bfill in [0u8, 1, 255] {\n" + inner + "\n    }\n    }"
            )
        body.append("    {\n" + inner + "\n    }")

    # One builder and one indexer per side for each paired enum, written as
    # matches so that neither module needs Copy, Debug or PartialEq.
    helpers = []
    for t, pairs in sorted(paired.items()):
        for side, mod, col in (("h", "hand", 0), ("s", "spec", 1)):
            arms = " ".join(f"{k} => {mod}::{t}::{pr[col]}," for k, pr in enumerate(pairs))
            helpers.append(f"fn enum_{side}_{t}(k: usize) -> {mod}::{t} {{ match k {{ {arms} _ => unreachable!() }} }}")
            arms = " ".join(f"{mod}::{t}::{pr[col]} => {k}," for k, pr in enumerate(pairs))
            helpers.append(f"fn idx_{side}_{t}(x: &{mod}::{t}) -> usize {{ match x {{ {arms} }} }}")

    src = f"""#[allow(dead_code, unused_mut, non_snake_case, unused_variables)] mod hand {{ include!("{ring_lib}"); }}
#[allow(dead_code, unused_mut, non_snake_case, unused_variables)] mod spec {{ include!("{spec_rs}"); }}
{chr(10).join(helpers)}
#[allow(unused_labels)]
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
    paired, unpaired = pair_enums(enums(hand_src), enums(gen))
    src, covered, refused = emit("hand.rs", "spec.rs", shared, producers(s, set(paired) | set(unpaired)),
                                 paired, unpaired)
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


def _pair_run(hand: str, spec: str):
    """Build and run the harness for two module texts. None if it did not compile."""
    sigs = signatures(hand)
    paired, unpaired = pair_enums(enums(hand), enums(spec))
    src, _, refused = emit("hand.rs", "spec.rs", sigs,
                           producers(signatures(spec), set(paired) | set(unpaired)), paired, unpaired)
    with tempfile.TemporaryDirectory() as d:
        open(os.path.join(d, "hand.rs"), "w").write(hand)
        open(os.path.join(d, "spec.rs"), "w").write(spec)
        open(os.path.join(d, "main.rs"), "w").write(src)
        b = subprocess.run(["rustc", "--edition", "2021", "-A", "warnings", "-C", "overflow-checks=on", "-o",
                            os.path.join(d, "diff"), os.path.join(d, "main.rs")],
                           capture_output=True, text=True, cwd=d)
        if b.returncode != 0:
            print(b.stderr[:600])
            return None
        return subprocess.run([os.path.join(d, "diff")], capture_output=True, text=True)


def self_check() -> int:
    """Two modules that differ by construction must be reported as disagreeing."""
    good = "pub fn f(a: u32) -> u32 { a + 1 }\n"
    bad = "pub fn f(a: u32) -> u32 { a + 2 }\n"
    r = _pair_run(good, bad)
    if r is None:
        print("  self-check: harness did not compile -- FAIL"); return 1
    saw = r.returncode == 1 and "disagree 0" not in r.stdout
    print(f"  self-check: two modules that differ -> {r.stdout.strip().splitlines()[0]} "
          f"-- {'PASS' if saw else 'FAIL'}")
    # Enums (#6893): variants pair by name across case; a swapped mapping is a
    # disagreement, and an enum return compares by the paired variant.
    eh = "pub enum E { Lo, Hi }\npub fn g(e: E) -> u32 { match e { E::Lo => 0, E::Hi => 1 } }\n" \
         "pub fn h(a: bool) -> E { if a { E::Hi } else { E::Lo } }\n"
    es_same = "pub enum E { lo, hi }\npub fn g(e: E) -> u32 { match e { E::lo => 0, E::hi => 1 } }\n" \
              "pub fn h(a: bool) -> E { if a { E::hi } else { E::lo } }\n"
    es_diff = "pub enum E { lo, hi }\npub fn g(e: E) -> u32 { match e { E::lo => 1, E::hi => 0 } }\n" \
              "pub fn h(a: bool) -> E { if a { E::lo } else { E::hi } }\n"
    r1, r2 = _pair_run(eh, es_same), _pair_run(eh, es_diff)
    ok_e = (r1 is not None and r1.returncode == 0 and "cases 4 " in r1.stdout
            and r2 is not None and r2.returncode == 1 and "disagree 4" in r2.stdout)
    print(f"  self-check: enum params/returns -- same agree, swapped disagree on all 4 -- {'PASS' if ok_e else 'FAIL'}")
    # Floats (#6893): NaN against NaN agrees; -0.0 against 0.0 does not.
    fh = "pub fn q(x: f32) -> f32 { x * 1.0 }\n"
    fs_same = "pub fn q(x: f32) -> f32 { x }\n"
    fs_diff = "pub fn q(x: f32) -> f32 { if x == 0.0 { 0.0 } else { x } }\n"
    r3, r4 = _pair_run(fh, fs_same), _pair_run(fh, fs_diff)
    ok_f = (r3 is not None and r3.returncode == 0 and r4 is not None and r4.returncode == 1
            and "disagree 1" in r4.stdout)
    print(f"  self-check: floats -- NaN agrees with NaN, -0.0 differs from 0.0 -- {'PASS' if ok_f else 'FAIL'}")
    # A producer that panics on one side only is a disagreement, not a dead harness.
    ph = "#[derive(Debug)] pub struct S(u8);\npub fn mk(a: u8) -> S { S(a) }\npub fn a_use(s: S) -> u8 { s.0 }\n"
    ps = "#[derive(Debug)] pub struct S(u8);\npub fn mk(a: u8) -> S { assert!(a != 7); S(a) }\npub fn a_use(s: S) -> u8 { s.0 }\n"
    r5 = _pair_run(ph, ps)
    ok_p = (r5 is not None and r5.returncode == 1 and "disagree 2" in r5.stdout
            and "producer mk panicked on one side" in r5.stdout)
    print(f"  self-check: a one-sided producer panic is a disagreement -- {'PASS' if ok_p else 'FAIL'}")
    saw = saw and ok_e and ok_f and ok_p
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
