#!/usr/bin/env python3
"""Elaboration errors may fall, never rise (#2325).

This gate holds the line per module over the generated fpga set. It does not
demand zero: most of what is left waits on two named design decisions -- what a
string field means in generated hardware (#2433) and unsized array params
(#2410).

It does NOT print a classification, and the reason is a lesson. It used to. The
table said 57 condition expressions, 64 unbound, 21 whole-array, 4 malformed,
5 unknown-module, 2 missing-function -- which sums to 153 under a headline of
161, so it was arithmetically wrong the day it was written, and within an hour
three of its six rows were also stale: unbound is 72, condition is 59, malformed
is 0 (that defect was closed by the next commit), and a seventh class it never
listed (Enable of unknown task, 3) exists. A hand-copied distribution rots
faster than anyone re-reads it.

    tri elab classify     # the distribution, measured, by message shape
    tri elab secondary    # which of them are derived from another above them

Two things this gate counts carefully, because it got both wrong once:

  * iverilog's closing "N error(s) during elaboration." is a TOTAL, not an
    error. Counting it added one phantom per failing module -- 25 of a
    published 186.
  * a "diagnostic line" is not a "bad construct". One bad construct can emit
    `syntax error` AND `error: Malformed statement` on the same line, so the
    number here is diagnostic LINES. Do not quote it as a defect count.

    tools/check_elab_ratchet.py                    # verify
    tools/check_elab_ratchet.py --update-baseline  # after a deliberate change

Requires iverilog and a built t27c; skips cleanly (exit 0) when either is
missing, so a Rust-only checkout is never blocked by it.
"""
import pathlib
import re
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
GEN = ROOT / "build/fpga/generated"
BASELINE = ROOT / "tools/elab_baseline.txt"
T27C = ROOT / "target/release/t27c"

# iverilog ends a failing file with "N error(s) during elaboration." -- a TOTAL,
# not an error. Counting it added exactly one phantom per failing module, so the
# published figure was 25 too high in both directions (see the module docstring).
# Hand-written notes live below this line in the baseline and survive
# --update-baseline. Everything above it is regenerated.
NOTES_MARK = "# --- notes (hand-written, preserved) ---\n"

SUMMARY = re.compile(r"\d+ error\(s\) during elaboration")


def counts():
    """{module: elaboration error count} over the generated set."""
    out = {}
    for v in sorted(GEN.glob("*.v")):
        proc = subprocess.run(
            ["iverilog", "-g2012", "-DSIMULATION", "-o", "/dev/null", str(v)],
            capture_output=True,
            text=True,
        )
        n = sum(
            1
            for ln in proc.stderr.splitlines()
            if " error" in ln and not SUMMARY.search(ln)
        )
        out[v.stem] = n
    return out


def iverilog_version():
    """The calibration of this instrument, recorded beside its numbers.

    A ratchet compares counts produced by a specific iverilog; an apt upgrade
    on the runner can move them without a single line of the compiler changing.
    The first CI run matched the local baseline exactly (186 vs 186), which is
    luck, not a property -- so the version is stored and a mismatch is named in
    the failure text instead of being mistaken for a regression.
    """
    try:
        out = subprocess.run(["iverilog", "-V"], capture_output=True, text=True)
    except Exception:
        return "unknown"
    m = re.search(r"version\s+(\S+)", out.stdout)
    return m.group(1) if m else "unknown"


def baseline_version():
    if not BASELINE.exists():
        return None
    for ln in BASELINE.read_text().splitlines():
        if ln.startswith("# iverilog-version "):
            return ln.split(" ", 2)[2].strip()
    return None


def baseline():
    if not BASELINE.exists():
        return {}
    d = {}
    for ln in BASELINE.read_text().splitlines():
        ln = ln.strip()
        if not ln or ln.startswith("#"):
            continue
        name, _, num = ln.rpartition(" ")
        try:
            d[name.strip()] = int(num)
        except ValueError:
            continue
    return d


def main():
    if not shutil.which("iverilog") or not T27C.exists():
        print("SKIP: iverilog or target/release/t27c missing")
        return 0

    gen = subprocess.run(
        [str(T27C), "fpga-build", "--smoke"], capture_output=True, text=True
    )
    if gen.returncode != 0:
        print(gen.stderr.strip()[-600:])
        print("t27c fpga-build --smoke failed")
        return 1

    now = counts()
    total = sum(now.values())

    if "--update-baseline" in sys.argv:
        # T66: the header is regenerated from a literal, so every hand-written
        # note under it was deleted by the very command this gate recommends --
        # including the one explaining that removing a syntax error can RAISE a
        # module's count. Notes below the sentinel are carried across.
        notes = ""
        if BASELINE.exists():
            txt = BASELINE.read_text()
            if NOTES_MARK in txt:
                # Bound the section by the comment lines themselves, not by a
                # blank line: this writer emits none after them, so a
                # blank-line bound swallowed the whole module list on the
                # SECOND consecutive run and wrote it back twice.
                tail = txt.split(NOTES_MARK, 1)[1].splitlines()
                kept = []
                for ln in tail:
                    if not ln.startswith("#"):
                        break
                    kept.append(ln)
                if kept:
                    notes = NOTES_MARK + "\n".join(kept) + "\n"
        header = (
            "# iverilog elaboration errors per generated module. This is a RATCHET:\n"
            "# the numbers may fall, never rise (#2325). iverilog's own \"N error(s)\n"
            "# during elaboration\" summary line is NOT counted -- it is a total, and\n"
            "# counting it added one phantom error per failing module.\n"
            "# The remainder is not one thing. Do not copy a classification here:\n"
            "# the last one was wrong when written and stale within the hour. Run\n"
            "# `tri elab classify` -- it measures the distribution by message shape.\n"
            f"# iverilog-version {iverilog_version()}\n"
        )
        body = "\n".join(f"{k} {v}" for k, v in sorted(now.items()))
        BASELINE.write_text(header + notes + body + "\n")
        print(f"baseline updated: {len(now)} modules, {total} errors")
        return 0

    base = baseline()
    if not base:
        print("no baseline; run --update-baseline once")
        return 1

    # T66: iterate the UNION. Iterating `now` alone means a module that stops
    # being generated -- dropped from the hardcoded list in main.rs, or a whole
    # empty output directory -- contributes no row at all, the total falls, and
    # the gate prints OK. A ratchet that only looks at what is present scores a
    # disappearance as an improvement.
    worse, better, new, gone = [], [], [], []
    for m in sorted(set(now) | set(base)):
        if m not in base:
            new.append((m, now[m]))
        elif m not in now:
            gone.append((m, base[m]))
        elif now[m] > base[m]:
            worse.append((m, base[m], now[m]))
        elif now[m] < base[m]:
            better.append((m, base[m], now[m]))

    print(f"elaboration errors: {total} (baseline {sum(base.values())})")
    for m, b, n in better:
        print(f"  BETTER  {m}: {b} -> {n}")
    for m, n in new:
        print(f"  NEW     {m}: {n} errors, not in baseline")
    for m, b, n in worse:
        print(f"  WORSE   {m}: {b} -> {n}")
    for m, b in gone:
        print(f"  GONE    {m}: was {b}, is no longer generated at all")

    if gone:
        print()
        print("A module in the baseline was not generated. Its errors did not")
        print("get fixed -- they left the measured set, which reads as progress")
        print("in the total above. Restore the module, or remove its baseline")
        print("line deliberately with --update-baseline.")
        return 1

    if worse or new:
        print()
        bv, nv = baseline_version(), iverilog_version()
        if bv and bv != nv:
            print(f"NOTE: the baseline was taken with iverilog {bv}; this run used {nv}.")
            print("A version change can move these counts without any compiler change,")
            print("so check that before reading the rows above as a regression.")
        print("A module gained elaboration errors. Generated Verilog that iverilog")
        print("cannot elaborate cannot be simulated, so its vectors can never run.")
        print("If the increase is deliberate: tools/check_elab_ratchet.py --update-baseline")
        return 1
    if better:
        print()
        # T66: a drop is not automatically a win. A syntax error TRUNCATES the
        # file, so introducing one makes a module's count collapse and this
        # branch used to answer "Modules improved. Record it." with no hedge --
        # and obeying it froze the truncated number as the new baseline.
        print("Modules improved -- classify before recording. A drop caused by a")
        print("syntax error is not a fix: a syntax error truncates the file, so")
        print("everything after it stops being counted. Check with:")
        print("    tri elab classify")
        print("Then, if the drop is real: tools/check_elab_ratchet.py --update-baseline")
        return 1
    print("OK: no module gained elaboration errors")
    return 0


if __name__ == "__main__":
    sys.exit(main())
