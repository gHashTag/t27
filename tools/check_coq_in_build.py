#!/usr/bin/env python3
r"""A Coq file no `_CoqProject` names is compiled by nothing, and cannot fail.

Measured 2026-09-04: **41** Coq files are named in a `_CoqProject` and carry 234
`Qed` and **zero** `Admitted`. **18** are named in none, and five of those hold
all **32** `Admitted` in the repository -- `Bounds_LeptonMasses.v` has 8 theorems,
8 `Admitted` and 0 `Qed`, so nothing in it is proven. The compiled set is spotless
because the unfinished proofs are outside it (#3153).

This does not fix that. It pins the 18 so a NINETEENTH cannot arrive unnoticed,
the same shape `tools/devhome_baseline.txt` and the assertionless-test ratchet
already use here: a list that only moves down, and a shrink that must be
re-blessed in the same commit so slack cannot be banked.

TWO WAYS TO COUNT THIS WRONG, BOTH WALKED INTO
----------------------------------------------
**`.v` is Coq AND Verilog.** `git ls-files '*.v'` returns 225 here and **166 are
Verilog**. A first pass was about to report 184 uncompiled Coq files. Files are
classified by content -- `Require`/`Theorem`/`Qed` against
`module`/`endmodule`/`always` -- not by extension.

**`-R .` maps a logical path and adds no files.** `coq_makefile` compiles what is
listed, so the file list IS the population. Reading `-R .` as "everything below
here" makes the question vanish.

    python3 tools/check_coq_in_build.py
    python3 tools/check_coq_in_build.py --bless
    python3 tools/check_coq_in_build.py --self-check
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
BASELINE = REPO_ROOT / "tools" / "coq_outside_build_baseline.txt"

COQ = re.compile(r"^\s*(?:Require|Theorem|Lemma|Definition|Inductive|Proof\b|Qed\b)", re.M)
VERILOG = re.compile(r"^\s*(?:module\b|endmodule\b|always\b|`timescale)", re.M)


def refuse(msg: str) -> None:
    print(f"check_coq_in_build: {msg}", file=sys.stderr)
    print("  Exit 2 = could not run, not a clean tree.", file=sys.stderr)
    raise SystemExit(2)


def is_coq(text: str) -> bool:
    """Coq or Verilog? Both use `.v`, and this repository is full of both."""
    return len(COQ.findall(text)) > len(VERILOG.findall(text))


def projects() -> list[Path]:
    found = sorted(REPO_ROOT.rglob("_CoqProject"))
    if not found:
        refuse("no _CoqProject anywhere, so nothing declares a Coq build.")
    return found


def listed() -> set[str]:
    out: set[str] = set()
    for proj in projects():
        base = proj.parent
        for ln in proj.read_text(encoding="utf-8", errors="replace").split("\n"):
            ln = ln.strip()
            # `-R .` maps a logical path; it does not add a file.
            if not ln or ln.startswith(("#", "-")):
                continue
            out.add((base / ln).resolve().relative_to(REPO_ROOT).as_posix())
    return out


def coq_files() -> list[str]:
    out = []
    for p in sorted(REPO_ROOT.rglob("*.v")):
        rel = p.relative_to(REPO_ROOT).as_posix()
        if rel.startswith((".git/", "target/", "node_modules/")):
            continue
        try:
            if is_coq(p.read_text(encoding="utf-8", errors="replace")):
                out.append(rel)
        except OSError:
            continue
    if not out:
        refuse("not one .v file classified as Coq, which is the matcher failing.")
    return out


def outside() -> list[str]:
    inside = listed()
    return [f for f in coq_files() if f not in inside]


def load() -> set[str]:
    if not BASELINE.is_file():
        refuse(f"{BASELINE.name} is missing. Regenerate with --bless.")
    return {l.strip() for l in BASELINE.read_text(encoding="utf-8").split("\n")
            if l.strip() and not l.lstrip().startswith("#")}


def write(rows: list[str]) -> None:
    head = (
        "# Coq files named by no _CoqProject: nothing compiles them, so nothing\n"
        "# they contain can fail. Pinned so a NEW one is noticed.\n"
        "#\n"
        "# The list only moves DOWN. Adding one fails. Removing one also fails --\n"
        "# re-bless in the same commit, so the slack cannot be banked.\n"
        "#\n"
        "# Regenerate: python3 tools/check_coq_in_build.py --bless\n"
        f"# {len(rows)} file(s).\n"
    )
    BASELINE.write_text(head + "".join(f"{r}\n" for r in rows), encoding="utf-8")


def self_check() -> int:
    cases = [
        ("a Coq file is Coq", "Require Import Foo.\nTheorem t : True.\nProof. exact I. Qed.\n", True),
        ("a Verilog file is not", "`timescale 1ns/1ps\nmodule m;\nalways @(*) x = 1;\nendmodule\n", False),
        ("a Coq file with no Require still counts", "Lemma l : True.\nProof. exact I. Qed.\n", True),
        ("an empty file is not Coq", "", False),
        ("a Verilog file naming a module only", "module m;\nendmodule\n", False),
        # The `module` and `` `timescale`` alternatives change no file's
        # classification on this corpus -- mutation deleting them broke nothing.
        # They are kept as a defence against a Verilog file carrying an
        # incidental Coq keyword, and this fixture is what makes that defence
        # testable rather than decorative.
        ("Verilog markers outweigh an incidental Coq keyword",
         "`timescale 1ns/1ps\nmodule m;\nDefinition x := 1.\n", False),
    ]
    ok = True
    for label, src, want in cases:
        got = is_coq(src)
        good = got == want
        ok = ok and good
        print(f"  self-check  {label:44} {'ok' if good else f'BROKEN got {got}'}")
    return 0 if ok else 2


def main() -> int:
    if "--self-check" in sys.argv:
        return self_check()

    now = sorted(outside())
    if "--bless" in sys.argv:
        write(now)
        print(f"blessed: {len(now)} Coq file(s) outside every _CoqProject")
        return 0

    base = load()
    print(f"Coq files outside every _CoqProject: {len(now)}  (baseline {len(base)})")

    added = sorted(set(now) - base)
    gone = sorted(base - set(now))

    if added:
        print(f"\nFAIL: {len(added)} Coq file(s) newly compiled by nothing.")
        for f in added:
            print(f"  {f}")
        print("\n  A file no _CoqProject names is compiled by nothing, so nothing in it\n"
              "  can fail. Add it to a _CoqProject, or bless it as debt. See #3153.")
        return 1

    if gone:
        print(f"\nFAIL: {len(gone)} file(s) joined a build -- good, but the list must move too.")
        for f in gone:
            print(f"  {f}")
        print("\n  Re-bless in the SAME commit:\n"
              "    python3 tools/check_coq_in_build.py --bless")
        return 1

    print("\nok: no Coq file newly fell out of every build, and none silently joined one.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
