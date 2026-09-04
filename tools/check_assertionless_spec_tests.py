#!/usr/bin/env python3
"""A `test` whose body is only a comment cannot fail, and there are 1792 of them.

WHAT THIS COUNTS
----------------
A spec test declaration whose entire body is comments:

    test igla_race_gemm_w347_batch_depth_invariant_1 { /* verify baseline */ }

Measured 2026-09-04 on master: **1813 of 9752** `.t27` test declarations (18.6%),
in 32 files. `.tri` specs carry 90 declarations and none of this shape.

The bulk is one pattern: **1792** carrying the identical body
`{ /* verify baseline */ }`, exactly 64 in each of 28 files under
`specs/igla/race/` and `specs/igla/coder/` -- two per wave, thirty-two waves of
the batch-append protocol.

The other **21, in 4 files, are why the matcher reads the body** rather than the
line. They are invisible to a single-line grep and just as unable to fail:

    test single_node_depth {              // specs/isa/ternary_tree.t27:115
        // Validated via invariant        //   says so itself
    }

    test test_utilization_creation {      // specs/fpga/testbench/power_analysis_tb.t27:39
//         given u = utilization(...)     //   the body is commented OUT
//         then u.luts == 5000
    }

The first census reported 1792 and was an undercount by exactly these 21.

WHY A BASELINE AND NOT A FIX
----------------------------
Three things could be done and two of them rewrite 28 spec files: give all 1792
real assertions, or delete them. Both are the owner's call (#3141). The third is
this: pin the existing debt so a NEW one fails and the old ones stay visible.
That is the shape `tools/devhome_baseline.txt` already uses here, and it is the
industry's: Betterer, ESLint bulk suppressions, SonarQube's clean-as-you-code,
`baseline`. The common core is a ceiling that only moves down.

This repository's version is stricter than the common one in a way worth naming:
a file that SHRINKS also fails, and must be re-blessed in the same commit. Most
ratchets let the number fall silently, which banks slack against the next
regression -- fix three and add two and the ceiling never notices. Here it does.

WHY THE MATCHER READS THE WHOLE BODY
------------------------------------
The census that found these matched a single line. There are no multi-line
comment-only bodies today -- a walk that opens each `test ... {` and reads to the
closing brace finds zero -- but a ratchet a newline can evade is not a ratchet.
This reads the body to its closing brace and asks whether every line in it is a
comment or blank.

    python3 tools/check_assertionless_spec_tests.py            # check
    python3 tools/check_assertionless_spec_tests.py --bless    # regenerate
    python3 tools/check_assertionless_spec_tests.py --self-check
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
BASELINE = REPO_ROOT / "tools" / "assertionless_spec_tests_baseline.txt"
SPEC_SUFFIXES = (".t27", ".tri")

OPEN = re.compile(r"^\s*test\s+[A-Za-z0-9_]+\s*\{(?P<rest>.*)$")
COMMENT_ONLY = re.compile(r"^\s*(?://.*|/\*[^*]*\*/)?\s*$")


def refuse(msg: str) -> None:
    print(f"check_assertionless_spec_tests: {msg}", file=sys.stderr)
    print("  Exit 2 = could not run, not a clean tree.", file=sys.stderr)
    raise SystemExit(2)


def assertionless_in(text: str) -> int:
    """Count test declarations whose body holds no statement.

    Handles the one-liner and the multi-line form with the same rule: strip the
    body of comments and blanks and see whether anything is left.
    """
    lines = text.split("\n")
    found = 0
    i = 0
    while i < len(lines):
        m = OPEN.match(lines[i])
        if not m:
            i += 1
            continue
        rest = m.group("rest")
        if "}" in rest:                      # one-liner: body is before the brace
            body = [rest[: rest.index("}")]]
            i += 1
        else:
            body, j = [], i + 1
            while j < len(lines) and lines[j].strip() != "}":
                body.append(lines[j])
                j += 1
            if j >= len(lines):              # unterminated: not ours to judge
                i += 1
                continue
            i = j + 1
        if all(COMMENT_ONLY.match(b) for b in body):
            found += 1
    return found


def census() -> dict[str, int]:
    specs = REPO_ROOT / "specs"
    if not specs.is_dir():
        refuse("specs/ is not a directory.")
    out: dict[str, int] = {}
    seen_any = False
    for p in sorted(specs.rglob("*")):
        if not p.is_file() or p.suffix not in SPEC_SUFFIXES:
            continue
        seen_any = True
        try:
            n = assertionless_in(p.read_text(encoding="utf-8", errors="replace"))
        except OSError as exc:
            refuse(f"could not read {p}: {exc}")
        if n:
            out[p.relative_to(REPO_ROOT).as_posix()] = n
    if not seen_any:
        refuse("no .t27 or .tri file was read, so this measured nothing.")
    return out


def load_baseline() -> dict[str, int]:
    if not BASELINE.is_file():
        refuse(f"{BASELINE.relative_to(REPO_ROOT)} is missing. Regenerate with --bless.")
    rows: dict[str, int] = {}
    for ln in BASELINE.read_text(encoding="utf-8").split("\n"):
        if not ln.strip() or ln.lstrip().startswith("#"):
            continue
        try:
            path, count = ln.rsplit("\t", 1)
            rows[path.strip()] = int(count)
        except ValueError:
            refuse(f"baseline row is not `path<TAB>count`: {ln[:80]!r}")
    return rows


def write_baseline(rows: dict[str, int]) -> None:
    total = sum(rows.values())
    head = (
        "# Spec `test` declarations whose body is only comments, per file.\n"
        "# They parse, they are counted as tests, and they cannot fail.\n"
        "#\n"
        "# Pinned so a NEW one fails and the existing debt stays visible. A file\n"
        "# that GROWS fails. A file that SHRINKS also fails -- re-bless in the same\n"
        "# commit, so the slack cannot be banked against the next one.\n"
        "#\n"
        "# Regenerate: python3 tools/check_assertionless_spec_tests.py --bless\n"
        f"# {len(rows)} file(s), {total} declaration(s).\n"
    )
    body = "".join(f"{p}\t{n}\n" for p, n in sorted(rows.items()))
    BASELINE.write_text(head + body, encoding="utf-8")


def self_check() -> int:
    cases = [
        ("one-liner is assertionless",
         "test a_b { /* verify baseline */ }\n", 1),
        ("multi-line comment body is assertionless",
         "test a_b {\n  // nothing here\n  /* nor here */\n}\n", 1),
        ("a real assertion is NOT",
         "test a_b { assert x == 1 }\n", 0),
        ("a multi-line real body is NOT",
         "test a_b {\n  // explain\n  assert x == 1\n}\n", 0),
        ("an empty body counts",
         "test a_b {\n}\n", 1),
        ("a non-test block is ignored",
         "invariant a_b { /* c */ }\n", 0),
        ("an unterminated block is not judged",
         "test a_b {\n  // and then the file ends\n", 0),
        ("two on one file are two",
         "test a { /* x */ }\ntest b { /* y */ }\n", 2),
    ]
    ok = True
    for label, src, want in cases:
        got = assertionless_in(src)
        good = got == want
        ok = ok and good
        print(f"  self-check  {label:44} {'ok' if good else f'BROKEN got {got} want {want}'}")
    return 0 if ok else 2


def main() -> int:
    if "--self-check" in sys.argv:
        return self_check()

    now = census()
    if "--bless" in sys.argv:
        write_baseline(now)
        print(f"blessed: {len(now)} file(s), {sum(now.values())} declaration(s)")
        return 0

    base = load_baseline()
    total_now, total_base = sum(now.values()), sum(base.values())
    print(f"assertionless spec tests: {total_now} in {len(now)} file(s)  "
          f"(baseline {total_base} in {len(base)})")

    grew = {p: (base.get(p, 0), n) for p, n in now.items() if n > base.get(p, 0)}
    shrank = {p: (base[p], now.get(p, 0)) for p in base if now.get(p, 0) < base[p]}

    if grew:
        print(f"\nFAIL: {len(grew)} file(s) gained an assertionless test.")
        for p, (was, is_) in sorted(grew.items()):
            print(f"  {p}  {was} -> {is_}")
        print("\n  A `test` whose body is only a comment cannot fail. Give it an\n"
              "  assertion, or do not add it. See #3141.")
        return 1

    if shrank:
        print(f"\nFAIL: {len(shrank)} file(s) lost one -- good, but the baseline must move too.")
        for p, (was, is_) in sorted(shrank.items()):
            print(f"  {p}  {was} -> {is_}")
        print("\n  Re-bless in the SAME commit:\n"
              "    python3 tools/check_assertionless_spec_tests.py --bless\n"
              "  A ceiling left above the real number banks slack against the next one.")
        return 1

    print("\nok: no file gained an assertionless test, and none silently lost one.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
