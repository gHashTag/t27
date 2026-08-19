#!/usr/bin/env python3
"""tri loop-rules -- verify the loop-rule file against its recorded checksum.

Why this exists. The durable rules of the hourly tick used to live inside one
long scheduled-task field. That field is silently truncated: 13277 characters
were submitted and 10069 stored, with no diagnostic. A rule that disappears
without a report is worse than a rule never written, because the tick keeps
behaving as if it were in force.

So the rules live in docs/loop/LOOP-RULES.md under version control, and their
digest lives in docs/loop/LOOP-RULES.sha256. The scheduled task keeps only a
pointer plus the expected digest, so a truncated field still names where the
rules are and which version was expected.

What this tool does and does not claim. It certifies that the rule text is
byte-identical to the text that was sealed. It says nothing about whether the
rules are good, nor whether the tick followed them. Identity, not correctness --
the same distinction as the compiler seal (R6).

Exit codes:
  0  digest matches
  1  digest differs, or the rule file / seal file is missing
"""
from __future__ import annotations

import argparse
import hashlib
import subprocess
import sys
from pathlib import Path

RULES = Path("docs/loop/LOOP-RULES.md")
SEAL = Path("docs/loop/LOOP-RULES.sha256")


def repo_root() -> Path:
    try:
        out = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            capture_output=True, text=True, check=True,
        ).stdout.strip()
        return Path(out)
    except Exception:
        return Path.cwd()


def digest(p: Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def read_seal(p: Path) -> str | None:
    if not p.exists():
        return None
    # Seal format is "<64-hex>  <repo-relative-path>", the format FROZEN.md
    # declares but the compiler seal file does not actually honour. Here the
    # declared format and the written format agree, deliberately.
    first = p.read_text().strip().split("\n")[0]
    tok = first.split()
    return tok[0] if tok else None


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(prog="tri loop-rules", description=__doc__)
    ap.add_argument("--reseal", action="store_true",
                    help="record the current digest as the sealed one; use only "
                         "after a deliberate edit to the rule file")
    ap.add_argument("--print-digest", action="store_true",
                    help="print only the current digest and exit 0")
    args = ap.parse_args(argv)

    root = repo_root()
    rules, seal = root / RULES, root / SEAL

    if not rules.exists():
        print(f"MISSING: {RULES} does not exist.")
        print("The rules are not recoverable from this tool. Recover them from "
              "git history, not from memory: memory reproduces the defect with "
              "the rule (R5).")
        return 1

    cur = digest(rules)
    if args.print_digest:
        print(cur)
        return 0

    n_lines = len(rules.read_text().splitlines())
    n_bytes = rules.stat().st_size

    if args.reseal:
        seal.parent.mkdir(parents=True, exist_ok=True)
        prev = read_seal(seal)
        seal.write_text(f"{cur}  {RULES.as_posix()}\n")
        print(f"resealed {RULES}")
        print(f"  previous  {prev or '(none)'}")
        print(f"  current   {cur}")
        print(f"  size      {n_bytes} bytes, {n_lines} lines")
        print("\nA reseal records that the rules changed. It does not record WHY."
              "\nThe reason belongs in the commit message and in the ledger.")
        return 0

    want = read_seal(seal)
    print(f"rule file : {RULES}  ({n_bytes} bytes, {n_lines} lines)")
    print(f"current   : {cur}")
    print(f"sealed    : {want or '(no seal recorded)'}")

    if want is None:
        print("\nNOT SEALED. Nothing certifies which version of the rules is in "
              "force. Run `tri loop-rules --reseal` once, in a tick that also "
              "commits the rule file.")
        return 1

    if want == cur:
        print("\nOK -- the rule text is byte-identical to the sealed text.")
        print("This certifies identity only. It does not certify that the rules "
              "are correct, nor that the tick obeyed them.")
        return 0

    print("\nMISMATCH -- the rule text is not the sealed text.")
    print("Do not stop the tick over this, and do not reseal to make it quiet. "
          "Record in the ledger WHICH version was in force, because a tick run "
          "against unknown rules cannot be audited afterwards. Then either "
          "restore the sealed text or reseal deliberately with a stated reason.")
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
