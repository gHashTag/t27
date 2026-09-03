#!/usr/bin/env python3
"""#3067: a matcher that matched nothing is not a reading.

`last_lesson_no` tested that the file EXISTS and then returned whatever the
matcher produced -- empty, with exit 0, when the file was present and the format
had moved past the pattern. So `$(last_lesson_no || echo '-')` never substituted
the '-', and `no="$(last_lesson_no)" || die` never died.

Measured on the real document: `.claude/skills/t27-wave-loop.md` is 242 KB and
present, its worked examples are headed `## Worked example -- Wave Loop 898`,
and `grep -cE '^\\*\\*[0-9]+\\.'` on it is 0. `tri wave` printed

    lesson    last  -> next 1

where the document's own last wave loop is 898. `tri lesson` reached `next=1`
the same way -- traced with `bash -x` -- and wrote nothing only because an
unrelated downstream step failed on the same empty anchor.

Each assertion has a control: a document that DOES carry lessons must still be
read and written correctly, or this file cannot fail.
"""

import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FAILURES = []

NO_LESSONS = """# Wave loop

## Worked example -- Wave Loop 898

Body text, with no `**N.` lesson line anywhere.
"""

HAS_LESSONS = """# Wave loop

**41. An existing lesson.** body

## Next heading

more
"""


def check(name, ok, detail=""):
    print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
    if not ok:
        FAILURES.append(f"{name}: {detail}")


def tree(doc):
    d = tempfile.mkdtemp(prefix=f"tri-wave-{os.getpid()}-")
    shutil.copytree(REPO / "scripts", os.path.join(d, "scripts"))
    os.makedirs(os.path.join(d, ".claude/skills"))
    with open(os.path.join(d, ".claude/skills/t27-wave-loop.md"), "w") as f:
        f.write(doc)
    subprocess.run(["git", "init", "-q", "."], cwd=d, capture_output=True)
    return d


def tri(d, *args):
    return subprocess.run(["bash", os.path.join(d, "scripts", "tri"), *args],
                          capture_output=True, text=True, cwd=d)


def main():
    d = tree(NO_LESSONS)
    try:
        w = tri(d, "wave")
        check("a matcher that matched nothing does not become a number",
              "next 1" not in w.stdout, f"stdout={w.stdout!r}")
        check("and says the line is unreadable, naming the file",
              "UNREADABLE" in w.stdout and "t27-wave-loop.md" in w.stdout, f"stdout={w.stdout!r}")

        l = tri(d, "lesson", "A brand new lesson", "with a body")
        check("tri lesson refuses instead of numbering from an empty match",
              l.returncode != 0, f"rc={l.returncode} out={(l.stdout+l.stderr)!r}")
        check("and the refusal is the one written for it",
              "no numbered lessons found" in (l.stdout + l.stderr), f"out={(l.stdout+l.stderr)!r}")
        doc = Path(d, ".claude/skills/t27-wave-loop.md").read_text()
        check("and nothing was written", "**1." not in doc, "a lesson numbered 1 was inserted")
    finally:
        shutil.rmtree(d, ignore_errors=True)

    # CONTROLS. Without these, a `last_lesson_no` that always fails passes above.
    d = tree(HAS_LESSONS)
    try:
        w = tri(d, "wave")
        check("control: a document with lessons is still read",
              "last 41 -> next 42" in w.stdout, f"stdout={w.stdout!r}")
        l = tri(d, "lesson", "A brand new lesson", "with a body")
        check("control: and a lesson is still written, with the next number",
              l.returncode == 0, f"rc={l.returncode} out={(l.stdout+l.stderr)!r}")
        doc = Path(d, ".claude/skills/t27-wave-loop.md").read_text()
        check("control: numbered 42, not 1", "**42. A brand new lesson.**" in doc, f"doc={doc!r}")
    finally:
        shutil.rmtree(d, ignore_errors=True)

    print()
    if FAILURES:
        print("FAILED:")
        for f in FAILURES:
            print(f"  - {f}")
        return 1
    print("ok: an empty match refuses; a real one still reads and writes.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
