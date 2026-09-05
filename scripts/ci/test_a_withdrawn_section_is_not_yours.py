#!/usr/bin/env python3
"""A section the base WITHDREW must not be carried forward as if it were yours.

Measured 2026-09-05: master rewrote SKILL section 554 in 6a49402c because its
claim was wrong. A branch that had merged the version BEFORE the correction saw
the withdrawn title as present-here-absent-there -- indistinguishable from a
section it had written -- and a by-title rebuild put the retraction back under a
fresh number.

Two cheaper discriminators do not work. The MERGE BASE is useless once the
branch has merged the base: it becomes the base's head. ANCESTRY is useless
because this repository squash-merges, so the commit that introduced the
withdrawn section is not an ancestor of the base -- and neither are the
branch's own.

What works is the base's own history OF THE TEXT, and that is the half no unit
test can reach: it is a `git log -S` against a real repository. This builds one.
"""
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
TRI = os.path.join(ROOT, "target", "debug", "tri")
SKILL = os.path.join(".claude", "skills", "ci-gates", "SKILL.md")


def git(d, *a):
    return subprocess.run(["git", "-C", d, *a], capture_output=True, text=True)


def write(d, text):
    p = os.path.join(d, SKILL)
    os.makedirs(os.path.dirname(p), exist_ok=True)
    with open(p, "w") as fh:
        fh.write(text)


def main():
    if not os.path.exists(TRI):
        print("SKIP: no tri binary at target/debug/tri -- build it first")
        return 0

    with tempfile.TemporaryDirectory() as d:
        os.makedirs(os.path.join(d, ".trinity"))
        git(d, "init", "-q", "-b", "main")
        git(d, "config", "user.email", "t@example.com")
        git(d, "config", "user.name", "t")

        # The shape that actually occurred. The branch diverges when the file
        # holds Alpha alone, so everything after it is "appended here" -- and
        # what the branch appended includes a section it got FROM main by
        # merging, which main later withdrew.
        A = "## 1. Alpha\n\nalpha body\n"
        write(d, A)
        git(d, "add", "-A")
        git(d, "commit", "-qm", "add Alpha")

        git(d, "checkout", "-qb", "mine")

        # main adds Bravo; the branch merges it and appends its own Charlie.
        git(d, "checkout", "-q", "main")
        write(d, A + "\n## 2. Bravo\n\nbravo body\n")
        git(d, "add", "-A")
        git(d, "commit", "-qm", "add Bravo")

        git(d, "checkout", "-q", "mine")
        write(d, A + "\n## 2. Bravo\n\nbravo body\n\n## 3. Charlie\n\ncharlie body\n")
        git(d, "add", "-A")
        git(d, "commit", "-qm", "take Bravo from main, append Charlie")

        # main WITHDRAWS Bravo -- the claim was wrong -- and replaces it.
        git(d, "checkout", "-q", "main")
        write(d, A + "\n## 2. Delta\n\ndelta body\n")
        git(d, "add", "-A")
        git(d, "commit", "-qm", "withdraw Bravo, add Delta")

        git(d, "checkout", "-q", "mine")
        r = subprocess.run(
            [TRI, "skill", "renumber", "--check", "--base", "main", "--file", SKILL],
            cwd=d, capture_output=True, text=True,
        )
        out = r.stdout + r.stderr

        if "Bravo" not in out:
            print("FAIL: the withdrawn section was not named. Carrying it forward")
            print("      resurrects a retraction, which is the whole defect.")
            print(out)
            return 1
        if r.returncode == 0:
            print("FAIL: the command succeeded. It must REFUSE and write nothing.")
            print(out)
            return 1
        if "Charlie" in out.split("Bravo")[0]:
            print("FAIL: Charlie is the branch's OWN section and must not be refused.")
            print(out)
            return 1

        # Negative control: with nothing withdrawn, the command must not refuse.
        git(d, "checkout", "-q", "main")
        write(d, A + "\n## 2. Bravo\n\nbravo body\n\n## 3. Delta\n\ndelta body\n")
        git(d, "add", "-A")
        git(d, "commit", "-qm", "put Bravo back beside Delta")
        git(d, "checkout", "-q", "mine")
        r2 = subprocess.run(
            [TRI, "skill", "renumber", "--check", "--base", "main", "--file", SKILL],
            cwd=d, capture_output=True, text=True,
        )
        if "REMOVED from main on purpose" in (r2.stdout + r2.stderr):
            print("FAIL: nothing is withdrawn now, so nothing may be refused.")
            print(r2.stdout + r2.stderr)
            return 1

        # --drop-withdrawn removes exactly those and proceeds. It is opt-in:
        # deleting text nobody asked to delete is how a tool earns distrust.
        git(d, "checkout", "-q", "main")
        write(d, A + "\n## 2. Delta\n\ndelta body\n")
        git(d, "add", "-A")
        git(d, "commit", "-qm", "withdraw Bravo again")
        git(d, "checkout", "-q", "mine")
        r3 = subprocess.run(
            [TRI, "skill", "renumber", "--base", "main", "--file", SKILL, "--drop-withdrawn"],
            cwd=d, capture_output=True, text=True,
        )
        out3 = r3.stdout + r3.stderr
        if r3.returncode != 0:
            print("FAIL: --drop-withdrawn must proceed, not refuse.")
            print(out3)
            return 1
        if "Bravo" not in out3:
            print("FAIL: it must NAME what it dropped. A silent deletion is the")
            print("      thing this whole guard exists to prevent.")
            print(out3)
            return 1
        after = open(os.path.join(d, SKILL)).read()
        if "## 2. Bravo" in after or "bravo body" in after:
            print("FAIL: Bravo is still in the file after --drop-withdrawn.")
            return 1
        if "Charlie" not in after:
            print("FAIL: Charlie is the branch's OWN section and must survive.")
            return 1
        if "Delta" not in after:
            print("FAIL: Delta is the base's and must survive.")
            return 1

    print("ok       a withdrawn section is refused by name; --drop-withdrawn removes")
    print("         exactly it, names it, and leaves both the branch's and the base's")
    return 0


if __name__ == "__main__":
    sys.exit(main())
