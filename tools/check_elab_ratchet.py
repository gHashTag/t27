#!/usr/bin/env python3
"""Elaboration errors may fall, never rise (#2325).

The generated Verilog of the 32-module fpga set went from 573 iverilog
elaboration errors to 186 in one evening, across three emitter classes:
string fields poisoning a whole struct (#2424), fields of array elements of a
struct flattening to an empty base (#2325), and unannotated locals losing the
type they were copied from. Nothing prevents that from sliding back: the
conformance job compiles two modules, and the other thirty are only linted by
yosys, which accepts many references iverilog rejects.

So this gate holds the line per module. It does not demand zero, and the
reason it does not is worth stating accurately, because an earlier version of
this docstring got it wrong. It said "the remainder is two named design
decisions". That was measured over the UNBOUND-IDENTIFIER errors only -- 68 of
them, and there the claim holds exactly (56 string reads, 12 unsized-array
reads, nothing else). It was written as though it described the whole
remainder. The whole remainder, classified:

    57  condition expressions -- SECONDARY, same source line as an unbound
        identifier above them; they disappear with their cause
    64  unbound identifiers    -- #2433 (strings) and #2410 (unsized arrays)
    21  whole-array reads      -- an array used where a value is expected
     4  malformed statements   -- a keyword-named identifier escaped at its
        declaration and not at its use; a real emitter defect, not a decision
     5  unknown module types   -- self-test modules that never elaborated
     2  missing functions
    ---
    161 real errors across 25 of 32 modules

It fails when any module gains errors, and asks you to record the win when a
module loses them.

    tools/check_elab_ratchet.py                    # verify
    tools/check_elab_ratchet.py --update-baseline  # after a deliberate change

Requires iverilog and a built t27c; skips cleanly (exit 0) when either is
missing, so a Rust-only checkout is never blocked by it -- the same contract
emit-bitexact-gate.yml uses.
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
        header = (
            "# iverilog elaboration errors per generated module. This is a RATCHET:\n"
            "# the numbers may fall, never rise (#2325). iverilog's own \"N error(s)\n"
            "# during elaboration\" summary line is NOT counted -- it is a total, and\n"
            "# counting it added one phantom error per failing module.\n"
            "# The remainder is not one thing: see the module docstring for the full\n"
            "# classification. Two design decisions (#2433, #2410) cover the unbound\n"
            "# identifiers; the malformed statements are an emitter defect.\n"
            f"# iverilog-version {iverilog_version()}\n"
        )
        body = "\n".join(f"{k} {v}" for k, v in sorted(now.items()))
        BASELINE.write_text(header + body + "\n")
        print(f"baseline updated: {len(now)} modules, {total} errors")
        return 0

    base = baseline()
    if not base:
        print("no baseline; run --update-baseline once")
        return 1

    worse, better, new = [], [], []
    for m, n in sorted(now.items()):
        if m not in base:
            new.append((m, n))
        elif n > base[m]:
            worse.append((m, base[m], n))
        elif n < base[m]:
            better.append((m, base[m], n))

    print(f"elaboration errors: {total} (baseline {sum(base.values())})")
    for m, b, n in better:
        print(f"  BETTER  {m}: {b} -> {n}")
    for m, n in new:
        print(f"  NEW     {m}: {n} errors, not in baseline")
    for m, b, n in worse:
        print(f"  WORSE   {m}: {b} -> {n}")

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
        print("Modules improved. Record it: tools/check_elab_ratchet.py --update-baseline")
        return 1
    print("OK: no module gained elaboration errors")
    return 0


if __name__ == "__main__":
    sys.exit(main())
