#!/usr/bin/env python3
"""Gate 20: a `Qed.` in a file nobody compiles is not a machine-checked proof.

The README reports "546 Qed. across 41 files", and that count is exactly right.
It is also not what a reader infers from it: **69 of those 546, across 7 files,
are in no `_CoqProject`**, so no `coq_makefile` build and no CI job ever
type-checks them. Two of the seven carry headers stating in capitals that they
do not compile and should be treated as research notes.

This is the campaign's most-repeated shape (Props. 116b, 142, 149): an accurate
count of a different denominator. `grep -c 'Qed\\.'` measures proof
*terminators in text*. Only membership in a build measures proofs.

WHAT THIS GATE REQUIRES. Every `.v` file under a Coq tree must either

  (a) appear in that tree's `_CoqProject`, so a build type-checks it, or
  (b) carry an explicit NOT-BUILT marker in its first 40 lines, naming itself
      as unverified.

Neither is a judgement about the mathematics. The gate exists so that a file
cannot sit in a proof directory, contribute to a published count, and be
type-checked by nothing, without that being written down where a reader looks.

ARTIFACTS. Reads `coq/**/*.v`, `trios-coq/**/*.v`, `proofs/**/*.v` and each
tree's `_CoqProject`. WRITES `formal/coq_build_baseline.txt` -- the ratcheted
set of files that are neither built nor self-declared. Nothing else.

Prop. 154.
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
TREES = ["coq", "trios-coq", "proofs"]

# A file excluded from the build must say so. Any of these, in the first 40
# lines, counts as the file declaring its own status.
MARKERS = ("NOT COMPILABLE", "NOT BUILT", "ASPIRATIONAL",
           "not machine-checked", "NOT MACHINE-CHECKED")


def qed_count(text):
    # comment-scan: Coq block comments are `(* ... *)` and a `Qed.` inside one
    # is not a proof terminator. Strip them before counting.
    stripped = re.sub(r"\(\*.*?\*\)", "", text, flags=re.S)
    return len(re.findall(r"\bQed\.", stripped))


def main():
    trees = [ROOT / t for t in TREES if (ROOT / t).exists()]
    if not trees:
        print(f"::error::coq build scan: no such directory "
              f"'{TREES[0]}' under the repository root -- nothing was scanned")
        return 1

    built_q = unbuilt_q = declared_q = 0
    built_f = declared_f = 0
    undeclared = []
    scanned = 0

    for tree in trees:
        proj = tree / "_CoqProject"
        listed = set()
        if proj.exists():
            listed = {l.lstrip("./")
                      for l in re.findall(r"(\S+\.v)", proj.read_text())}
        for p in sorted(tree.rglob("*.v")):
            scanned += 1
            rel = str(p.relative_to(tree))
            text = p.read_text(errors="ignore")
            n = qed_count(text)
            if rel in listed:
                built_q += n
                built_f += 1
                continue
            head = "\n".join(text.splitlines()[:40])
            if any(m in head for m in MARKERS):
                declared_q += n
                declared_f += 1
            else:
                undeclared.append((n, f"{tree.name}/{rel}"))
            unbuilt_q += n

    if scanned == 0:
        print("::error::coq build scan: found no .v files under "
              f"{', '.join(t.name for t in trees)} -- nothing was scanned")
        return 1

    print(f"coq build scan: {scanned} .v files, "
          f"{built_q} Qed in a _CoqProject build ({built_f} files), "
          f"{unbuilt_q} Qed unbuilt ({declared_f} files declare it)")

    # RATCHET, not a wall. Whether an unbuilt proof file SHOULD be added to a
    # _CoqProject is a mathematical judgement about that file, not something a
    # scanner can decide -- and a gate that lands red on 17 pre-existing files
    # gets disabled rather than obeyed (Prop. 26). It fails when the set grows.
    baseline = ROOT / "formal" / "coq_build_baseline.txt"
    now = sorted(f for _, f in undeclared)
    if not baseline.exists():
        baseline.write_text("\n".join(now) + ("\n" if now else ""))
        print(f"coq build scan: baseline written to {baseline.name} "
              f"({len(now)} undeclared)")
        return 0
    was = [l for l in baseline.read_text().splitlines() if l.strip()]
    new_files = [f for f in now if f not in was]
    if new_files:
        print(f"::error::coq build scan: {len(new_files)} .v file(s) under the "
              f"Coq trees are in no _CoqProject and do not declare themselves "
              f"unverified. A `Qed.` in a file nobody compiles is not a "
              f"machine-checked proof -- add the file to its _CoqProject, or "
              f"add a header naming it unverified (one of: "
              f"{', '.join(MARKERS[:3])})")
        for f in new_files:
            print(f"  {f}")
        return 1
    fixed = [f for f in was if f not in now]
    if fixed:
        print(f"coq build scan: {len(fixed)} file(s) now built or declared; "
              f"update {baseline.name} to lock it in")
    print(f"coq build scan: ratchet holds ({len(now)} <= {len(was)} undeclared)")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::coq build scan: could not scan the Coq trees "
              f"({type(exc).__name__}: {exc}) -- nothing was scanned")
        sys.exit(1)
