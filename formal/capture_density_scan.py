#!/usr/bin/env python3
"""Gate 27: if the parser's output does not grow with the file, it read nothing.

Gate 19 asks whether `t27c parse` recovered from errors, and gate 24 whether a
file's delimiters can close. Both are control-flow questions. **A file can pass
every one of them and still be read as an empty shell**, and 8 do:

    coa_planning.t27      496 code lines -> 2 AST nodes
    composition.t27       420 code lines -> 2 AST nodes
    asp_solver.t27        408 code lines -> 2 AST nodes
                          ...  8 files, 2865 lines, ALL exactly 2 nodes

The two nodes are `kind: Module, name: ""` -- an anonymous wrapper and nothing
inside it. Zero recovery events, zero declarations swallowed, delimiters balanced.

Two independent reasons gate 19 could not see this, both instances of Prop. 194's
numerator fallacy inside my own instrument:

  1. its blind check tests `captured == 0`, and these capture 2;
  2. that check is guarded by "does the file declare something `pub`", and these
     8 declare nothing public -- so the guard, added to reduce false positives,
     removed exactly the cases where capture is worst.

THE QUESTION THIS ASKS. Not "does the file contain shape X" (Prop. 193 -- a shape
search cannot bound its residue) but: **does the parser's output scale with its
input?** That is total over the corpus, needs no vocabulary of known defects, and
a file cannot satisfy it accidentally.

THRESHOLD, HONESTLY. The first draft of this docstring asserted the corpus was
sharply bimodal -- median ~1.95, outliers at 0.004-0.009, "a factor of 200 with
nothing in between" -- and the gate's own gap line refuted it on the first run by
printing **6x**. The real distribution is:

    0.0040 .. 0.0087   8 files, ALL capturing exactly 2 nodes (empty shells)
    ----------------   a 5.9x jump, the largest gap in the low tail
    0.0513 .. 0.07+    a continuum of weakly-read files, no gap at all

So the threshold isolates the total shells and nothing more. The 0.05-0.07 band
is **acknowledged residue**: those files are also read poorly, and this gate says
nothing about them. That is recorded here rather than smoothed over because the
instrument falsifying its author's stated assumption, in its own log, on its first
run, is the only reason the claim did not ship (Prop. 195).

COVERAGE. Examines every spec under `specs/` with at least 40 non-comment lines:
332 of 497. Files below that are excluded because node-per-line is noise at small
n, and that exclusion is a real residue -- a 20-line spec read as an empty shell
is invisible here. A second, larger residue: the 6 files between 0.05 and 0.07
nodes/line are weakly read and are NOT flagged, because no gap separates them from
the corpus above. It measures node COUNT, not node correctness: a file whose
declarations are all captured with wrong contents passes, and Prop. 192's 18
corrupted field types are exactly that case. This gate bounds how much was read,
never whether what was read is right.

ARTIFACTS. Reads `specs/**/*.t27`, runs `./target/release/t27c`. WRITES
`formal/capture_density_baseline.txt`, and only when `--init` is passed. Nothing else.

Prop. 195.
"""
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
T27C = ROOT / "target" / "release" / "t27c"
BASELINE = ROOT / "formal" / "capture_density_baseline.txt"
MIN_LINES = 40
THRESHOLD = 0.05          # nodes per code line; see the bimodality note above


def code_lines(text):
    return len([l for l in text.splitlines()
                if l.strip() and not l.strip().startswith(("//", "#"))])


def main():
    if not T27C.exists():
        print("::error::capture density scan: no such file "
              "'target/release/t27c' -- build it first; nothing was measured")
        return 1
    specs = sorted((ROOT / "specs").rglob("*.t27"))
    if not specs:
        print("::error::capture density scan: found no .t27 files under specs/ "
              "-- nothing was measured")
        return 1

    rows = []
    for f in specs:
        text = f.read_text(errors="ignore")
        n = code_lines(text)
        if n < MIN_LINES:
            continue
        r = subprocess.run([str(T27C), "parse", str(f)],
                           capture_output=True, text=True)
        if r.returncode not in (0, 1):
            print(f"::error::capture density scan: t27c returned "
                  f"{r.returncode} on {f.relative_to(ROOT)} -- neither parse "
                  f"nor error")
            return 1
        rows.append((len(re.findall(r"kind: ", r.stdout)) / n,
                     str(f.relative_to(ROOT)), n))
    if not rows:
        print(f"::error::capture density scan: no spec has >= {MIN_LINES} code "
              f"lines -- nothing was measured")
        return 1

    rows.sort()
    dens = [d for d, _, _ in rows]
    median = dens[len(dens) // 2]
    low = [r for r in rows if r[0] < THRESHOLD]
    # Publish the gap the single threshold rests on, so the assumption is
    # falsifiable from the log rather than only from the source.
    above = next((d for d in dens if d >= THRESHOLD), None)
    gap = f"{above / max(dens[len(low) - 1], 1e-9):.0f}x" if low and above else "n/a"

    print(f"capture density scan: {len(rows)} specs >= {MIN_LINES} code lines, "
          f"median {median:.2f} nodes/line, {len(low)} below {THRESHOLD} "
          f"(gap to the next spec above: {gap})")

    now = sorted(f"{p}\t{n}" for _, p, n in low)
    if not BASELINE.exists():
        # Prop. 211c: writing a baseline is an explicit act, never a fallback.
        # `if not exists(): write(now); return 0` resets the ratchet on one
        # `rm`, and on a clone that never had the file it rubber-stamps the tree
        # it was handed and exits 0. Measured before f66561f33: 8 of the 13
        # baselines in this suite were on disk and in no commit, and 8 of the 13
        # gates owning them re-baseline a possibly-broken tree and pass.
        if "--init" not in sys.argv[1:]:
            print(f"::error::capture density scan: {BASELINE.name} does not exist and "
                  f"--init was not given. Writing one here would record "
                  f"whatever this tree contains as the accepted state -- on a "
                  f"fresh clone that is a green run which checked nothing. "
                  f"Genuine first run: `python3 formal/capture_density_scan.py --init`. "
                  f"Otherwise the baseline was lost and belongs in the commit "
                  f"that lost it (Prop. 211)")
            return 1
        BASELINE.write_text("\n".join(now) + ("\n" if now else ""))
        print(f"capture density scan: baseline written to {BASELINE.name} "
              f"({len(now)} shells)")
        return 0
    was = [l for l in BASELINE.read_text().splitlines() if l.strip()]
    new = [x for x in now if x not in was]
    if new:
        print(f"::error::capture density scan: {len(new)} spec(s) whose parsed "
              f"output does not scale with the file. Zero recovery events and "
              f"balanced delimiters do not mean the parser read the spec -- "
              f"these are anonymous empty shells (Prop. 195)")
        for x in new[:10]:
            path, n = x.split("\t")
            print(f"  {path}: {n} code lines")
        return 1
    fixed = [w for w in was if w not in now]
    if fixed:
        print(f"capture density scan: {len(fixed)} spec(s) now read; update "
              f"{BASELINE.name} to lock it in")
    print(f"capture density scan: ratchet holds ({len(now)} <= {len(was)} shells)")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::capture density scan: could not measure specs/ "
              f"({type(exc).__name__}: {exc}) -- nothing was measured")
        sys.exit(1)
