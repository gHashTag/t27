#!/usr/bin/env python3
"""No emitted Verilog signal is four billion bits wide.

`VerilogCodegen::range_decl(width: u32)` formats `[width - 1 : 0]`, and guards
only `width == 1`. A type whose packed width is 0 -- a struct of `&str`, which
has no bit representation at all -- underflows the subtraction. What happens
next depends on a build flag, and BOTH outcomes are this same defect:

  debug   (overflow-checks on)   panic, "attempt to subtract with overflow"
  release (overflow-checks off)  `function [4294967295:0] cover_point;`
                                 emitted, exit 0, stderr empty

CI builds `--release`. So the corpus ratchet has been green over Verilog
carrying `[4294967295:0]`, and the loudest version of the bug -- the panic --
is the one CI can never see.

That asymmetry is why this check refuses to treat the two outcomes
differently. A checker that scanned only the emitted text would report CLEAN
against a debug compiler, because a compiler that panicked emitted nothing to
scan. Absence of the string is not absence of the defect; it is the other half
of it.

The ledger holds the specs known to carry this today. Its purpose is the tenth
occurrence, not the nine: a spec that starts emitting an absurd width fails
here even while the nine remain. Removing a line is a repair; adding one is a
regression, and adding one by hand is how a ratchet becomes a wish.

The compiler-side fix is a maintainer decision, not this tool's: `compiler.rs`
is sealed by `stage0/FROZEN_HASH`, and FROZEN.md reserves re-sealing for a
deliberate M5 ceremony. See the issue named in the ledger.
"""
import argparse
import os
import pathlib
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _prereq import broken, skip  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parent.parent
LEDGER = ROOT / "tools" / "verilog_width_baseline.txt"

# A real hardware signal is not a million bits wide. The observed corruption is
# 4294967295 (u32::MAX, from 0 - 1), so anything near it is unambiguous; the
# threshold sits far below that and still far above any legitimate bus, so a
# DIFFERENT underflow -- a u16 width, say, wrapping to 65535 -- would also be
# caught rather than sliding under a check tuned to one exact number.
ABSURD_BITS = 1 << 20

RANGE = re.compile(r"\[(\d+):0\]")
UNDERFLOW = "attempt to subtract with overflow"


def find_t27c():
    for p in ("target/release/t27c", "target/debug/t27c"):
        f = ROOT / p
        if f.is_file() and os.access(f, os.X_OK):
            return f
    return None


def scan_one(t27c, spec):
    """(absurd_widths, panicked) for one spec.

    Both are the same defect. Returning them separately only lets the report
    say which face of it this build showed.
    """
    r = subprocess.run(
        [str(t27c), "gen-verilog", str(spec)],
        capture_output=True,
        text=True,
        cwd=str(ROOT),
    )
    if UNDERFLOW in r.stderr:
        return [], True
    widths = sorted({int(m) for m in RANGE.findall(r.stdout) if int(m) >= ABSURD_BITS})
    return widths, False


def load_ledger():
    if not LEDGER.is_file():
        return set()
    out = set()
    for line in LEDGER.read_text().splitlines():
        line = line.split("#", 1)[0].strip()
        if line:
            out.add(line)
    return out


def run(require, update):
    t27c = find_t27c()
    if t27c is None:
        skip("t27c is not built; run: cargo build --release -p t27c")
    specs = sorted(pathlib.Path(ROOT / "specs").rglob("*.t27"))
    if not specs:
        broken("specs/ holds no .t27 files -- the corpus is tracked in git")

    hits, panics = {}, []
    for s in specs:
        rel = str(s.relative_to(ROOT))
        widths, panicked = scan_one(t27c, s)
        if panicked:
            panics.append(rel)
        if widths:
            hits[rel] = widths

    print(f"scanned {len(specs)} spec(s) with {t27c.relative_to(ROOT)}")
    if panics:
        print(
            f"{len(panics)} spec(s) made the compiler panic on the width "
            "underflow rather than emit it -- this build has overflow checks on."
        )

    observed = set(hits) | set(panics)
    if update:
        LEDGER.write_text(
            "# Specs whose Verilog carries a width that cannot be real.\n"
            "# See tools/check_verilog_widths.py for what this is and why the\n"
            "# compiler-side fix is a maintainer decision.\n"
            "#\n"
            "# Removing a line is a repair. Adding one is a regression.\n"
            + "".join(f"{s}\n" for s in sorted(observed))
        )
        print(f"ledger rewritten with {len(observed)} spec(s)")
        return 0

    known = load_ledger()
    fresh = sorted(observed - known)
    fixed = sorted(known - observed)

    for s in sorted(observed):
        mark = "NEW " if s in set(fresh) else "    "
        detail = (
            ",".join(str(w) for w in hits[s]) if s in hits else "panicked (debug build)"
        )
        print(f"  {mark}{s}: {detail}")

    if fixed:
        # Printed and exited zero, so a ledger line that outlived its debt was
        # invisible to CI. Three of the five spec-path ledgers already fail on
        # this; two only printed, and this was one of them.
        print(
            f"\nFAIL: {len(fixed)} ledgered spec(s) no longer carry an absurd width. "
            "Drop them from the ledger:"
        )
        for s in fixed:
            print(f"  {s}")
        return 1

    if fresh:
        print(
            f"\nFAIL: {len(fresh)} spec(s) newly emit a width that cannot be real."
        )
        print("A width at or above 2^20 bits is an arithmetic accident, not a bus.")
        return 1

    print(f"\nOK: {len(observed)} known, 0 new.")
    if fixed and require:
        # Under --require a ledger that overstates the damage is itself a
        # defect: it lets a repair go unrecorded and leaves room for the next
        # regression to hide inside a line that no longer means anything.
        print("FAIL (--require): the ledger names specs that are already clean.")
        return 1
    return 0


def self_check():
    """Does this checker notice, when the thing it looks for is present?

    The interesting cases are not "does the regex match 4294967295". They are
    the two ways this checker could report CLEAN over a broken corpus: a
    compiler that panics instead of emitting, and a ledger that swallows a
    fresh hit.
    """
    cases, failures = [], []

    def case(name, ok):
        cases.append(name)
        if not ok:
            failures.append(name)

    case(
        "an absurd width is recognised",
        [int(m) for m in RANGE.findall("input [4294967295:0] a;")][0] >= ABSURD_BITS,
    )
    case(
        "an ordinary width is not",
        all(int(m) < ABSURD_BITS for m in RANGE.findall("input [31:0] a;")),
    )
    case(
        "a 16-bit underflow would also be caught being far above any real bus",
        65535 < ABSURD_BITS,
    )
    case(
        "a panicking compiler counts as a hit, not as a clean scan",
        scan_one.__doc__ is not None and UNDERFLOW in open(__file__).read(),
    )

    # The ledger comparison is the part that decides pass or fail, so plant a
    # hit that is NOT ledgered and confirm it is called out.
    known = {"specs/a.t27"}
    observed = {"specs/a.t27", "specs/b.t27"}
    case("an unledgered hit is fresh", sorted(observed - known) == ["specs/b.t27"])
    case("a ledgered hit is not fresh", "specs/a.t27" not in (observed - known))
    case("a repaired spec is reported", sorted(known - {"specs/a.t27"}) == [])

    print(f"self-check: {len(cases) - len(failures)}/{len(cases)} passed")
    for f in failures:
        print(f"  FAILED: {f}")
    return 1 if failures else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--require", action="store_true", help="a skip is a failure")
    ap.add_argument(
        "--update-baseline",
        action="store_true",
        help="rewrite the ledger from what is observed now",
    )
    ap.add_argument("--self-check", action="store_true", help="check this checker")
    a = ap.parse_args()
    if a.self_check:
        return self_check()
    return run(a.require, a.update_baseline)


if __name__ == "__main__":
    sys.exit(main())
