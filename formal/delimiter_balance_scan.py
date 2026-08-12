#!/usr/bin/env python3
"""Gate 25: a spec whose delimiters cannot close was written by a broken generator.

Two members of one corruption family have now been found by accident, six waves
apart, each after a long chase of downstream symptoms:

  * Prop. 186: 107 field types `bits : [[]Usize",`  -- `"` written where `]` belongs
  * Prop. 192:  18 field types `x : [[]Const [,`    -- `[` written where `]` belongs

Both were emitted by a generator that substituted a WRONG CHARACTER, and in both
cases the defect was invisible to every component built to tolerate it
(Prop. 192): the parser does not track nesting, and the recovery counters measure
control flow while these are value defects.

The two were found one at a time because each search was written for the shape
already known. This gate inverts that: it does not look for `[[]Const [` or for
`[[]Usize"`. It asks the question those are both instances of -- **can this file's
delimiters close at all?** -- which is decidable, cheap, and does not need to know
what the next substitution will be.

WHAT IT MEASURES. Global balance of `()`, `[]`, `{}` per file, plus an
unterminated string at EOF. Comment- and string-aware, using the state machine
from Prop. 188 (splitting on `//` before counting cuts `"https://..."` in half and
produced 74 false positives).

HONEST SCOPE -- read this before quoting the number. A non-zero global balance is
**necessary** evidence of a defect, not sufficient proof of one: this language has
no construct that legitimately leaves a file unbalanced, but the scanner's own
string/comment handling is a model of the lexer, not the lexer. The findings are
therefore RATCHETED, not repaired, and the baseline is a set of file+class+delta
triples rather than a count -- so a file that swaps one defect for another still
fails.

ARTIFACTS. Reads `specs/**/*.t27`. WRITES `formal/delimiter_balance_baseline.txt`
when no baseline exists. Nothing else.

Prop. 193.
"""
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SPECS = ROOT / "specs"
CLOSE = {")": "(", "]": "[", "}": "{"}


def balance(text):
    """Net depth per opener class, and whether a string is still open at EOF."""
    depth = {"(": 0, "[": 0, "{": 0}
    in_str = False
    for line in text.splitlines():
        s = line.strip()
        if s.startswith("//") or s.startswith("#"):
            continue
        i = 0
        while i < len(line):
            c = line[i]
            if in_str:
                if c == "\\":
                    i += 2
                    continue
                if c == '"':
                    in_str = False
            elif c == '"':
                in_str = True
            elif c == "'" and i + 2 < len(line) and line[i + 2] == "'":
                i += 3            # a character literal, e.g. '(' or '"'
                continue
            elif c == "/" and i + 1 < len(line) and line[i + 1] == "/":
                break             # a real comment: nothing after it is lexed
            elif c in depth:
                depth[c] += 1
            elif c in CLOSE:
                depth[CLOSE[c]] -= 1
            i += 1
    return depth, in_str


def main():
    if not SPECS.exists():
        print("::error::delimiter balance scan: no such directory 'specs' under "
              "the repository root -- nothing was scanned")
        return 1
    files = sorted(SPECS.rglob("*.t27"))
    if not files:
        print("::error::delimiter balance scan: found no .t27 files under "
              "specs/ -- nothing was scanned")
        return 1

    now = []
    for f in files:
        try:
            depth, open_str = balance(f.read_text(errors="ignore"))
        except OSError:
            continue
        rel = str(f.relative_to(ROOT))
        for cls, d in sorted(depth.items()):
            if d:
                now.append(f"{rel}\t{cls}\t{d}")
        if open_str:
            now.append(f"{rel}\tSTR\t1")
    now.sort()
    bad_files = len({e.split('\t')[0] for e in now})

    print(f"delimiter balance scan: {len(files)} specs, {bad_files} whose "
          f"delimiters cannot close ({len(now)} class-level imbalances)")

    baseline = ROOT / "formal" / "delimiter_balance_baseline.txt"
    if not baseline.exists():
        baseline.write_text("\n".join(now) + ("\n" if now else ""))
        print(f"delimiter balance scan: baseline written to {baseline.name} "
              f"({len(now)} entries)")
        return 0

    was = [l for l in baseline.read_text().splitlines() if l.strip()]
    new = [e for e in now if e not in was]
    if new:
        print(f"::error::delimiter balance scan: {len(new)} new delimiter "
              f"imbalance(s). No construct in this language leaves a file "
              f"unbalanced, so this is a generator writing a wrong character -- "
              f"the defect class that hid 107 sites for months (Prop. 186) and "
              f"18 more behind a tolerant parser (Prop. 192)")
        for e in new[:12]:
            path, cls, d = e.split("\t")
            print(f"  {path}: {cls} off by {d}")
        return 1
    fixed = [e for e in was if e not in now]
    if fixed:
        print(f"delimiter balance scan: {len(fixed)} imbalance(s) resolved; "
              f"update {baseline.name} to lock it in")
    print(f"delimiter balance scan: ratchet holds "
          f"({len(now)} <= {len(was)} imbalances)")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::delimiter balance scan: could not scan specs/ "
              f"({type(exc).__name__}: {exc}) -- nothing was scanned")
        sys.exit(1)
