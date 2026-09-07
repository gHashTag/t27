#!/usr/bin/env python3
"""Does a `rings/*` crate still agree with the spec its own doc comment names?

Ten of the seventeen crates under `rings/` name a spec that exists in this
repository -- `rings/ring-090-rust` says it is "faithful to the spec"
`specs/fpga/simulator.t27`. Nothing checked that claim, and one of them was not.

What this reports, and the taxonomy matters more than the counts:

  CONVERGED   every shared function has an identical signature. The pair is a
              candidate for the hand-written crate to be REPLACED by generated
              code -- and nothing here proves it can be, see the warning below.
  DRIFTED     some signatures match and some do not. A real divergence in a pair
              that was once the same thing.
  UNRELATED   no shared function, or none whose signature matches. The doc
              comment names a spec the crate was never generated from; these are
              two designs sharing a vocabulary, and reconciling them is a design
              decision rather than a repair.

**A matching signature is not matching behaviour, and this tool cannot tell you
that it is.** `ring-090` measured CONVERGED at 16 of 16 identical signatures and
still disagreed with its spec on **126 of 1190** differential cases: the spec
wrapped where the hand-written model saturated, so `sim_time_ns` returned
2,820,130,816 instead of 4,294,967,295 at 2e9 cycles (#3420). Only compiling and
running both found it. Treat CONVERGED as "worth building a differential harness
for", never as "these agree".

Usage:
  tools/check_ring_spec_drift.py                report; exit 1 if any pair DRIFTED
  tools/check_ring_spec_drift.py --self-check   negative control
"""

import os
import re
import subprocess
import sys

RINGS = "rings"
SIG = re.compile(
    r"^pub (?:const )?fn ([a-z_][a-z_0-9]*)\s*\(([^)]*)\)\s*->\s*([^{;]+)", re.M
)


def t27c() -> str:
    for p in ("target/release/t27c", "bootstrap/target/release/t27c"):
        if os.path.isfile(p) and os.access(p, os.X_OK):
            return p
    env = os.environ.get("TRI_T27C", "")
    if env and os.access(env, os.X_OK):
        return env
    print("check_ring_spec_drift: t27c not built. Exit 2 = COULD NOT RUN.", file=sys.stderr)
    sys.exit(2)


def signatures(text: str) -> dict:
    """Name -> (parameter list, return type), whitespace and a trailing comma normalised.

    The trailing comma is not cosmetic here: a multi-line signature keeps one and
    a single-line one does not, and without stripping it `sim_config_with_trace`
    read as a difference when the two were character-for-character the same.
    """
    out = {}
    for m in SIG.finditer(text):
        params = re.sub(r"[\s,]+$", "", re.sub(r"\s+", " ", m.group(2))).strip()
        out[m.group(1)] = (params, m.group(3).strip())
    return out


def classify(hand: dict, spec: dict) -> str:
    shared = set(hand) & set(spec)
    if not shared:
        return "UNRELATED"
    same = sum(1 for k in shared if hand[k] == spec[k])
    if same == 0:
        return "UNRELATED"
    if same == len(shared):
        return "CONVERGED"
    return "DRIFTED"


def pairs():
    """(ring, spec_path) for every crate whose doc comment names an existing spec."""
    found = []
    if not os.path.isdir(RINGS):
        return found
    for d in sorted(os.listdir(RINGS)):
        lib = os.path.join(RINGS, d, "src", "lib.rs")
        if not os.path.isfile(lib):
            continue
        m = re.search(r"specs/[a-z0-9_/]+\.t27", open(lib, errors="replace").read())
        if m:
            found.append((d, m.group(0)))
    return found


def report(binary: str) -> int:
    rows = []
    for ring, spec in pairs():
        if not os.path.exists(spec):
            rows.append((ring, spec, "SPEC-MISSING", 0, 0, 0, []))
            continue
        hand = signatures(open(f"{RINGS}/{ring}/src/lib.rs", errors="replace").read())
        gen = subprocess.run([binary, "gen-rust", spec], capture_output=True, text=True).stdout
        s = signatures(gen)
        shared = sorted(set(hand) & set(s))
        differing = [k for k in shared if hand[k] != s[k]]
        rows.append((ring, spec, classify(hand, s), len(hand), len(s), len(shared), differing))

    if not rows:
        # A tool reporting "0 problems" over 0 inputs has measured nothing.
        print("check_ring_spec_drift: no ring names a spec. Exit 2.", file=sys.stderr)
        return 2

    print(f"ring/spec pairs: {len(rows)}")
    for ring, spec, verdict, nh, ns, nsh, diff in rows:
        print(f"  {verdict:<12} {ring:<18} {spec}")
        print(f"               hand fns {nh}, spec fns {ns}, shared {nsh}, differing {len(diff)}")
        for k in diff[:6]:
            print(f"                 differs: {k}")
    conv = sum(1 for r in rows if r[2] == "CONVERGED")
    print(
        f"\nCONVERGED {conv} -- a matching signature is NOT matching behaviour. "
        f"ring-090 read 16 of 16 identical and still disagreed on 126 of 1190 "
        f"differential cases (#3420)."
    )
    return 1 if any(r[2] == "DRIFTED" for r in rows) else 0


def self_check() -> int:
    """Each verdict must be reachable, or the taxonomy is decoration."""
    a = {"f": ("x: u32", "u32"), "g": ("", "bool")}
    same = dict(a)
    partial = {"f": ("x: u32", "u32"), "g": ("y: i8", "bool")}
    none = {"f": ("x: u64", "u64")}
    disjoint = {"zzz": ("", "bool")}
    checks = [
        ("CONVERGED", classify(a, same)),
        ("DRIFTED", classify(a, partial)),
        ("UNRELATED", classify(a, none)),
        ("UNRELATED", classify(a, disjoint)),
    ]
    # The trailing-comma normalisation, which is what made ring-090 read as
    # identical rather than as one spurious difference.
    tc = signatures("pub fn f(a: u32, b: u32,) -> u32 {\n")
    tc2 = signatures("pub fn f(a: u32, b: u32) -> u32 {\n")
    ok = all(want == got for want, got in checks) and tc == tc2
    for want, got in checks:
        print(f"  self-check: want {want:<10} got {got:<10} {'ok' if want == got else 'FAIL'}")
    print(f"  self-check: trailing comma normalised -- {'ok' if tc == tc2 else 'FAIL'}")
    return 0 if ok else 1


def converged_pairs(binary: str) -> int:
    """Print `<ring-dir> <spec>` for every CONVERGED pair, one per line.

    So a caller can run the differential harness over exactly the pairs where a
    signature match makes one meaningful, without hard-coding the list -- which
    would go stale the first time a pair converged or drifted.
    """
    n = 0
    for ring, spec in pairs():
        if not os.path.exists(spec):
            continue
        hand = signatures(open(f"{RINGS}/{ring}/src/lib.rs", errors="replace").read())
        gen = subprocess.run([binary, "gen-rust", spec], capture_output=True, text=True).stdout
        if classify(hand, signatures(gen)) == "CONVERGED":
            print(f"{RINGS}/{ring} {spec}")
            n += 1
    if n == 0:
        # Loudly. A caller looping over an empty list would run zero harnesses
        # and report success.
        print("check_ring_spec_drift: no CONVERGED pair.", file=sys.stderr)
    return 0


def main() -> int:
    if "--self-check" in sys.argv:
        return self_check()
    if "--converged" in sys.argv:
        return converged_pairs(t27c())
    return report(t27c())


if __name__ == "__main__":
    sys.exit(main())
