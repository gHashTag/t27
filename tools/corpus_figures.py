#!/usr/bin/env python3
"""The figures measured in the GENERATED C, pinned with their instrument.

`tools/published_figures.py` pins the spec-side populations. This pins the
other side, and it is the side both of that audit's unit conflations came from:
`len(x)` was published as 142 -- a DIAGNOSTIC count in the generated C -- inside
a sentence about the specs, and `pub const OP_*` as 20, a count of list SITES in
the C rather than of declarations. A number from this side, quoted without
saying which side it came from, is how that happens.

TWO THINGS IT REFUSES TO GUESS.

  * THE INSTRUMENT. The same command gives 20 diagnostics on Apple clang and 50
    on ubuntu's gcc for the same input (#3450), and 141 corpus files hit clang's
    default cap, which made every total measured without `-ferror-limit=0` a
    floor (#3448). The pin records the compiler that produced it, and `--check`
    REFUSES to compare across instruments: exit 3, cannot tell, rather than a
    drift report that is really a machine report.

  * WHICH UNIT. Every row carries one: `units`, `errors` (diagnostics),
    `lines` (distinct source lines), `files`. A class is pinned by BOTH its
    diagnostic count and its distinct-line count, because the ratio between
    them is what says whether a population is real or partly a cascade -- 1.00
    means every diagnostic is its own site.

It is a READER with a pin, not a fast gate: it rebuilds the corpus, which is
minutes. Run it when a pass claims a corpus number.

Usage:
  tools/corpus_figures.py             derive and compare with the pin
  tools/corpus_figures.py --bless     rewrite the pin from this run
  tools/corpus_figures.py --self-check  negative control

Exit codes:
  0  every figure matches its pin
  1  a figure drifted under the SAME instrument
  2  COULD NOT RUN (no t27c, no cc, no specs)
  3  CANNOT TELL -- the instrument differs from the pin's
"""

import collections
import os
import re
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PIN = os.path.join(ROOT, "tools", "corpus_figures_pin.txt")

# The LINE NUMBER is captured, not inferred. The first version keyed distinct
# lines on the diagnostic's byte offset, so every diagnostic got its own key and
# the `lines` column came out exactly equal to `errors` on every row -- a column
# that always equals its neighbour measures nothing, and it was one command from
# being pinned as a fact.
DIAG = re.compile(r"^\S+:(\d+):\d+: error: (.*)$", re.M)

# Classes pinned by name. Hand-chosen, like the parity tables' rows: the ones
# this campaign has quoted. A class not listed still counts toward `errors`.
CLASSES = [
    "expected expression",
    "cannot use '__auto_type' with initializer list in C",
    "use of undeclared identifier 'POS'",
    "use of undeclared identifier 'NEG'",
    "call to undeclared function 'len'",
    "redefinition of",
    "incompatible integer to pointer conversion initializing",
]


def t27c() -> str:
    for rel in ("target/release/t27c", "bootstrap/target/release/t27c"):
        p = os.path.join(ROOT, rel)
        if os.path.isfile(p) and os.access(p, os.X_OK):
            return p
    env = os.environ.get("TRI_T27C", "")
    if env and os.access(env, os.X_OK):
        return env
    print("corpus_figures: t27c not built. Exit 2 = COULD NOT RUN.", file=sys.stderr)
    sys.exit(2)


def instrument() -> str:
    v = subprocess.run(["cc", "--version"], capture_output=True, text=True)
    if v.returncode != 0:
        print("corpus_figures: no cc. Exit 2 = COULD NOT RUN.", file=sys.stderr)
        sys.exit(2)
    first = v.stdout.strip().splitlines()[0] if v.stdout.strip() else "unknown cc"
    return first[:72]


def uncap_flag() -> str:
    """gcc rejects `-ferror-limit`, and clang ACCEPTS `-fmax-errors=0` while
    ignoring it. Asking the vendor is the only way to remove the cap."""
    v = subprocess.run(["cc", "--version"], capture_output=True, text=True).stdout
    return "-ferror-limit=0" if "clang" in v.lower() else "-fmax-errors=0"


def derive():
    binary, flag = t27c(), uncap_flag()
    specs = sorted(
        os.path.join(r, f)
        for r, _, fs in os.walk(os.path.join(ROOT, "specs"))
        for f in fs
        if f.endswith(".t27")
    )
    if not specs:
        print("corpus_figures: no specs. Exit 2.", file=sys.stderr)
        sys.exit(2)
    units = generated = clean = 0
    errors = 0
    cls_d = collections.Counter()
    cls_l = collections.defaultdict(set)
    with tempfile.TemporaryDirectory() as d:
        for s in specs:
            units += 1
            h = os.path.join(d, os.path.relpath(s, ROOT).replace("/", "_") + ".h")
            with open(h, "w") as fh:
                subprocess.run([binary, "gen-c", s], stdout=fh, stderr=subprocess.DEVNULL)
            if os.path.getsize(h) == 0:
                continue
            generated += 1
            r = subprocess.run(
                ["cc", "-std=c11", flag, "-fsyntax-only", "-x", "c", h],
                capture_output=True, text=True,
            )
            text = r.stdout + r.stderr
            found = 0
            for m in DIAG.finditer(text):
                found += 1
                msg = m.group(2)
                for c in CLASSES:
                    if msg.startswith(c):
                        cls_d[c] += 1
                        cls_l[c].add((h, m.group(1)))
                        break
            errors += found
            if found == 0:
                clean += 1
    rows = [
        ("specs seen", "units", units),
        ("specs that generate", "units", generated),
        ("translation units compiling clean", "units", clean),
        ("total errors", "errors", errors),
    ]
    for c in CLASSES:
        rows.append((f"class: {c}", "errors", cls_d[c]))
        rows.append((f"class: {c}", "lines", len(cls_l[c])))
    return rows


def read_pin():
    if not os.path.exists(PIN):
        return None, []
    inst, rows = "", []
    for line in open(PIN, encoding="utf-8"):
        if line.startswith("# instrument:"):
            inst = line.split(":", 1)[1].strip()
        line = line.split("#", 1)[0].strip()
        if not line:
            continue
        name, unit, val = line.rsplit("|", 2)
        rows.append((name.strip(), unit.strip(), int(val)))
    return inst, rows


def write_pin(inst, rows):
    with open(PIN, "w", encoding="utf-8") as fh:
        fh.write("# Figures measured in the GENERATED C, with the instrument that\n"
                 "# produced them. A number from this side quoted without saying so is\n"
                 "# how `len(x)` came to be published as 142 -- a diagnostic count --\n"
                 "# inside a sentence about the specs. See tools/corpus_figures.py.\n"
                 "#\n"
                 f"# instrument: {inst}\n"
                 "# name | unit | value\n")
        for name, unit, val in rows:
            fh.write(f"{name} | {unit} | {val}\n")


def self_check() -> int:
    """A planted error must be counted and a clean file must not, or a report of
    0 and a report that could not run look alike."""
    ok = True
    flag = uncap_flag()
    with tempfile.TemporaryDirectory() as d:
        bad = os.path.join(d, "bad.h")
        open(bad, "w").write("int f(void){ return undefined_zzz; }\n")
        r = subprocess.run(["cc", "-std=c11", flag, "-fsyntax-only", "-x", "c", bad],
                           capture_output=True, text=True)
        n = len(DIAG.findall(r.stdout + r.stderr))
        print(f"  planted 1 error -> counted {n} {'PASS' if n >= 1 else 'FAIL'}")
        ok &= n >= 1
        # Two diagnostics on ONE line must count as one line. Without this the
        # `lines` column silently equals `errors`, which is what the first
        # version did on every row.
        two = os.path.join(d, "two.h")
        open(two, "w").write("int g(void){ return aaa_undef + bbb_undef; }\n")
        r2 = subprocess.run(["cc", "-std=c11", flag, "-fsyntax-only", "-x", "c", two],
                            capture_output=True, text=True)
        ms = list(DIAG.finditer(r2.stdout + r2.stderr))
        lines_seen = len({m.group(1) for m in ms})
        print(f"  2 errors on 1 line -> {len(ms)} errors, {lines_seen} line "
              f"({'PASS' if len(ms) >= 2 and lines_seen == 1 else 'FAIL'})")
        ok &= len(ms) >= 2 and lines_seen == 1
        good = os.path.join(d, "good.h")
        open(good, "w").write("int f(void){ return 0; }\n")
        r = subprocess.run(["cc", "-std=c11", flag, "-fsyntax-only", "-x", "c", good],
                           capture_output=True, text=True)
        n = len(DIAG.findall(r.stdout + r.stderr))
        print(f"  clean file      -> counted {n} {'PASS' if n == 0 else 'FAIL'}")
        ok &= n == 0
        print(f"  uncap flag for this vendor -> {flag}")
    return 0 if ok else 2


def main() -> int:
    if "--self-check" in sys.argv:
        return self_check()
    inst = instrument()
    rows = derive()
    if "--bless" in sys.argv:
        write_pin(inst, rows)
        print(f"pinned {len(rows)} figures, instrument: {inst}")
        return 0
    pin_inst, pin_rows = read_pin()
    print(f"instrument now: {inst}")
    if pin_inst and pin_inst != inst:
        print(f"instrument pinned: {pin_inst}")
        print("\nCANNOT TELL. The pin was taken with a different compiler, and the same")
        print("command gives 20 diagnostics on one and 50 on another for the same input.")
        print("A comparison across instruments is a machine report wearing a drift's clothes.")
        return 3
    pinned = {(n, u): v for n, u, v in pin_rows}
    drift = 0
    print(f"\n{'figure':52} {'unit':7} {'pinned':>8} {'now':>8}")
    for name, unit, val in rows:
        p = pinned.get((name, unit))
        mark = "" if p == val else "  DRIFT"
        if p != val:
            drift += 1
        print(f"{name[:52]:52} {unit:7} {('-' if p is None else p):>8} {val:8}{mark}")
    if drift:
        print(f"\n{drift} figure(s) drifted under the SAME instrument. Say which and why,")
        print("then `--bless`. A corpus figure that moves silently is how a class")
        print("gets re-counted three passes running without anyone noticing.")
    return 1 if drift else 0


if __name__ == "__main__":
    sys.exit(main())
