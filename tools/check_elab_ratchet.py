#!/usr/bin/env python3
"""Elaboration errors may fall, never rise (#2325).

The generated Verilog of the 32-module fpga set went from 573 iverilog
elaboration errors to 186 in one evening, across three emitter classes:
string fields poisoning a whole struct (#2424), fields of array elements of a
struct flattening to an empty base (#2325), and unannotated locals losing the
type they were copied from. Nothing prevents that from sliding back: the
conformance job compiles two modules, and the other thirty are only linted by
yosys, which accepts many references iverilog rejects.

So this gate holds the line per module. It does not demand zero -- the
remainder is two named design decisions (string comparison in hardware; unsized
array params, #2410). It fails when any module gains errors, and asks you to
record the win when a module loses them.

    tools/check_elab_ratchet.py                    # verify
    tools/check_elab_ratchet.py --update-baseline  # after a deliberate change

Requires iverilog and a built t27c; skips cleanly (exit 0) when either is
missing, so a Rust-only checkout is never blocked by it -- the same contract
emit-bitexact-gate.yml uses.
"""
import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
GEN = ROOT / "build/fpga/generated"
BASELINE = ROOT / "tools/elab_baseline.txt"
T27C = ROOT / "target/release/t27c"


def counts():
    """{module: elaboration error count} over the generated set."""
    out = {}
    for v in sorted(GEN.glob("*.v")):
        proc = subprocess.run(
            ["iverilog", "-g2012", "-DSIMULATION", "-o", "/dev/null", str(v)],
            capture_output=True,
            text=True,
        )
        n = sum(1 for ln in proc.stderr.splitlines() if " error" in ln)
        out[v.stem] = n
    return out


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
            "# the numbers may fall, never rise (#2325). The remainder is two named\n"
            "# design decisions -- string comparison in hardware, and unsized array\n"
            "# params (#2410) -- not an oversight.\n"
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
