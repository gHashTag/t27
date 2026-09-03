#!/usr/bin/env python3
"""#3071: the issue-number parser answered on 1048 branches that carry no issue.

    ISSUE_NUM=$(echo "$BRANCH" | grep -oE '(issue-|#)?[0-9]+' | head -1 | tr -d 'issue-#')

The prefix was OPTIONAL, so any digits anywhere became an issue number. Measured
over every branch in this repository, local and remote: 1294 examined, 1048 given
a number the branch does not carry, and ZERO branches using the `issue-N` or `#N`
form the comment documents. `w42-status-ruler`, `w42-tri-vsim`,
`w42-verilog-break` and `w42-vsim-unknown` all answered #42 -- a WAVE number.

The documented population is empty, and the optional prefix turned that emptiness
into 1048 confident wrong answers.

The parser is EXTRACTED from each hook rather than restated here, so a hook that
drifts back is caught, and the extraction is itself asserted -- a test that
silently exercises nothing is worse than no test.
"""

import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HOOKS = [REPO / ".githooks/post-merge", REPO / ".githooks/pre-commit"]
FAILURES = []

# (branch, the number a correct parser yields)
CASES = [
    ("issue-357-xyz", "357"),
    ("feature/issue-42", "42"),
    ("#1031-topic", "1031"),
    ("w42-status-ruler", ""),
    ("w58-rings-matrix-refuses-empty", ""),
    ("master", ""),
    ("release-2026-09", ""),
]


def check(name, ok, detail=""):
    print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
    if not ok:
        FAILURES.append(f"{name}: {detail}")


def parser_line(hook):
    for line in hook.read_text().splitlines():
        if "ISSUE_NUM=$(echo" in line:
            return line.strip()
    return None


def run(line, branch, var):
    body = f'{var}="{branch}"\n{line}\nprintf "%s" "$ISSUE_NUM"'
    r = subprocess.run(["bash", "-c", body], capture_output=True, text=True)
    return r.stdout


def main():
    for hook in HOOKS:
        line = parser_line(hook)
        check(f"{hook.name}: the parser line was found", line is not None,
              "no ISSUE_NUM assignment -- this test would assert nothing")
        if line is None:
            continue
        var = "MERGED_BRANCH" if "MERGED_BRANCH" in line else "BRANCH_NAME"
        for branch, want in CASES:
            got = run(line, branch, var)
            label = f"{hook.name}: {branch!r} -> {want or '<none>'}"
            check(label, got == want, f"got {got!r}")

    # The measurement that made this a defect rather than a nit, re-taken here so
    # it cannot rot: over the repository's real branch names, the documented
    # forms are absent and the wNN- form is everywhere.
    out = subprocess.run(["git", "branch", "-a", "--format=%(refname:short)"],
                         capture_output=True, text=True, cwd=REPO).stdout
    names = {n.replace("origin/", "", 1) for n in out.split() if n}
    documented = [n for n in names if re.search(r"(issue-|#)\d+", n)]
    wave = [n for n in names if re.match(r"^w\d+", n)]
    check("the documented issue-N form is still rare in this repository",
          len(documented) <= len(wave),
          f"{len(documented)} documented vs {len(wave)} wave-style -- if this flips, revisit the parser")
    print(f"      (branches: {len(names)}; issue-shaped {len(documented)}; wNN- {len(wave)})")

    print()
    if FAILURES:
        print("FAILED:")
        for f in FAILURES:
            print(f"  - {f}")
        return 1
    print("ok: no hook invents an issue number.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
