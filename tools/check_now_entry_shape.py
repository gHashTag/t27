#!/usr/bin/env python3
"""The docs/now/ entry a PR adds must SAY something.

WHY THIS EXISTS
---------------
`t27-master-protection` requires four status checks, and one of them, `check`,
was this:

    - name: Check freshness
      run: |
        # Add freshness check logic here in future
        echo "Checking repository freshness..."

A required gate that echoes a string. It passed every pull request it ever ran
on, and its green meant nothing. The ruleset is not this file's to edit -- so
the job keeps its required name and is given something real to do instead.

WHAT IT CHECKS, AND WHAT IT DELIBERATELY DOES NOT
-------------------------------------------------
NOW Sync Gate already requires that an entry was ADDED. It does not read it.
So the two are complementary and neither is redundant:

    NOW Sync Gate   ->  "you wrote one"
    this            ->  "what you wrote says something"

Checked, per entry added by the pull request:

  * the filename is `YYYY-MM-DD-<slug>.md`
  * the first line is `# NOW -- <title> (YYYY-MM-DD)`
  * the date in that heading matches the date in the filename -- an entry
    dated one day and filed under another sorts wrong in a log whose whole
    job is chronology
  * there is a `## ` section heading
  * there is at least one `- ` bullet, and the bullets are not placeholders

NOT checked: whether the content is TRUE. No gate can do that, and pretending
otherwise would be the same defect one level up.

ZERO ENTRIES IS A FAILURE, NOT A PASS
-------------------------------------
If the pull request adds no entry, this exits 1 rather than passing quietly.
NOW Sync Gate should have caught that first; if it did not, two gates are
wrong at once and that is worth hearing about. A check that reports success
over an empty input set is the shape this file was written to replace.
"""

import os
import pathlib
import re
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent

FILENAME = re.compile(r"^(\d{4}-\d{2}-\d{2})-[a-z0-9][a-z0-9-]*\.md$")
HEADING = re.compile(r"^# NOW -- (.+) \((\d{4}-\d{2}-\d{2})\)\s*$")
PLACEHOLDER = re.compile(r"^(tbd|todo|wip|n/?a|\.\.\.|-+)$", re.I)


def added_now_entries(base, head):
    """Files under docs/now/ that this PR adds. README.md is not an entry."""
    # `-z`, because `--name-only` C-quotes any path with a non-ASCII character:
    # `"docs/now/2026-09-05-\320\277\321\200..."`. The quoted form does not end in
    # `.md`, so the entry left the population SILENTLY and the gate reported that the
    # change adds none. Measured on a real Cyrillic filename.
    r = subprocess.run(
        # `--no-renames`, because rename detection decides this gate's population.
        # git reports a rename as a single R entry, so `--diff-filter=A` misses it
        # and a docs/now/ path whose content newly lands by rename never reaches
        # the shape reader. With the flag git reports D(old) + A(new) and the new
        # path is judged like any other.
        ["git", "diff", "-z", "--name-only", "--no-renames", "--diff-filter=A", f"{base}...{head}",
         "--", "docs/now/"],
        capture_output=True,
        text=True,
        cwd=ROOT,
    )
    if r.returncode != 0:
        return None
    return [
        p for p in r.stdout.split("\0") if p.endswith(".md") and not p.endswith("README.md")
    ]


def check_entry(path, text):
    """Every complaint about one entry. Empty list means it is well formed."""
    bad = []
    name = pathlib.PurePosixPath(path).name

    m_name = FILENAME.match(name)
    if not m_name:
        bad.append(f"filename is not YYYY-MM-DD-<slug>.md: {name}")

    lines = text.splitlines()
    if not lines:
        bad.append("the file is empty")
        return bad

    m_head = HEADING.match(lines[0])
    if not m_head:
        bad.append(f"first line is not `# NOW -- <title> (YYYY-MM-DD)`: {lines[0][:70]!r}")
    elif m_name and m_head.group(2) != m_name.group(1):
        bad.append(
            f"heading is dated {m_head.group(2)} but the filename says {m_name.group(1)} "
            "-- a log whose job is chronology cannot have the two disagree"
        )

    if not any(l.startswith("## ") for l in lines):
        bad.append("no `## ` section heading")

    bullets = [l[2:].strip() for l in lines if l.startswith("- ")]
    if not bullets:
        bad.append("no `- ` bullets: the entry states nothing")
    else:
        empty = [b for b in bullets if len(b) < 12 or PLACEHOLDER.match(b)]
        if len(empty) == len(bullets):
            bad.append(f"every bullet is a placeholder or too short to say anything: {bullets}")

    return bad


def self_check():
    """Break it on purpose. A gate nobody has seen fail is not a gate."""
    good = (
        "# NOW -- A real entry about a real thing (2026-08-28)\n\n"
        "## A real entry about a real thing (Refs #1)\n\n"
        "- something specific happened and here is what it was\n"
    )
    cases = [
        ("well formed", "2026-08-28-a-real-entry.md", good, 0),
        (
            "heading date disagrees with filename",
            "2026-08-27-a-real-entry.md",
            good,
            1,
        ),
        (
            "no bullets",
            "2026-08-28-a-real-entry.md",
            "# NOW -- Title (2026-08-28)\n\n## Title\n",
            1,
        ),
        (
            "placeholder bullets",
            "2026-08-28-a-real-entry.md",
            "# NOW -- Title (2026-08-28)\n\n## Title\n\n- TBD\n- ...\n",
            1,
        ),
        (
            "wrong first line",
            "2026-08-28-a-real-entry.md",
            "# Some other heading\n\n## Title\n\n- a bullet that is long enough\n",
            1,
        ),
        ("empty file", "2026-08-28-a-real-entry.md", "", 1),
    ]
    failures = 0
    for label, name, text, want in cases:
        got = 1 if check_entry(f"docs/now/{name}", text) else 0
        ok = got == want
        print(f"  {label:<38} {'ok' if ok else 'CONTROL FAILED'}"
              f"{'' if ok else f'  (wanted {want}, got {got})'}")
        if not ok:
            failures += 1
    if failures:
        print(f"\nFAIL: {failures} control(s) did not behave as stated.")
        return 1
    print("\nOK: every control behaves as stated.")
    return 0


def main():
    if "--self-check" in sys.argv:
        return self_check()

    # `--check-files <path>...`: judge exactly these paths, no git range.
    # An empty list is refused rather than passed -- a caller that computed no
    # files and got a green back would read it as "the entries are fine".
    if "--check-files" in sys.argv:
        paths = [a for a in sys.argv[sys.argv.index("--check-files") + 1:]
                 if not a.startswith("--")]
        if not paths:
            print("FAIL: --check-files was given no paths. Nothing was checked,")
            print("  and a pass over an empty set is the shape this file replaces.")
            return 1
        return report(paths, from_index="--from-index" in sys.argv)

    base = os.environ.get("PR_BASE_SHA", "")
    head = os.environ.get("PR_HEAD_SHA", "HEAD")

    # A dispatched run has no pull request, so there is no set of added entries
    # to read. That is NOT the same as a run that should have had one and did
    # not: this check's subject is a pull request, and a dispatch is not one.
    #
    # Found by dispatching all 27 unmeasured workflows at master, which is
    # exactly what a dispatch is for -- it took a reading, and the reading said
    # this gate goes red when fired outside its subject.
    event = os.environ.get("GITHUB_EVENT_NAME", "")
    if not base and event and event != "pull_request":
        print(f"NOT APPLICABLE: this check reads the docs/now/ entry a PULL REQUEST adds,")
        print(f"  and the event here is `{event}`. There is no pull request to read.")
        print("  Nothing was checked and nothing is claimed.")
        return 0

    if not base:
        print("check_now_entry_shape: PR_BASE_SHA is unset on a pull_request event, so")
        print("  the set of entries this change adds cannot be computed. Reporting")
        print("  nothing rather than a pass this run did not earn.")
        return 2

    entries = added_now_entries(base, head)
    if entries is None:
        print(f"FAIL: `git diff {base}...{head}` failed -- the range is wrong, not the tree")
        return 1

    if not entries:
        print("FAIL: this change adds no docs/now/ entry.")
        print("  NOW Sync Gate should have caught that first. Two gates disagreeing")
        print("  about the same requirement is worth hearing about, so this one")
        print("  refuses rather than passing over an empty set.")
        return 1

    return report(entries)


def report(entries, from_index=False):
    """Read and judge each entry. The ONE body both callers share.

    `--check-files` exists so a contributor can ask this gate its own question
    before pushing. It must not be a second implementation of the answer: five
    local instruments already check `docs/now/` and every one of them reads
    FRESHNESS -- that an entry exists, dated inside the window -- while the
    required `check` context reads SHAPE. Same directory, same label, different
    question, and the local ones went green on an entry this gate rejects.
    """
    bad = 0
    for path in entries:
        if from_index:
            # THE DEFECT THIS EXISTS FOR. The caller LISTS the index (`git diff --cached`)
            # and this JUDGED the working tree. Staging a malformed entry and then fixing
            # the working copy produced `ok <path>`, exit 0, and the malformed one is what
            # `git commit` would take.
            r = subprocess.run(["git", "show", f":{path}"],
                               capture_output=True, cwd=ROOT)
            if r.returncode != 0:
                print(f"FAIL {path}: not in the index ({r.stderr.decode('utf-8','replace').strip()})")
                bad += 1
                continue
            try:
                text = r.stdout.decode("utf-8")
            except UnicodeDecodeError:
                print(f"FAIL {path}: the staged copy is not UTF-8")
                bad += 1
                continue
        else:
            full = ROOT / path
            try:
                text = full.read_text()
            except OSError as e:
                print(f"FAIL {path}: cannot read it ({e})")
                bad += 1
                continue
        problems = check_entry(path, text)
        if problems:
            bad += 1
            print(f"FAIL {path}")
            for p in problems:
                print(f"    {p}")
        else:
            print(f"ok   {path}")

    if bad:
        print(f"\nFAIL: {bad} of {len(entries)} entr(y/ies) do not say anything checkable.")
        return 1
    print(f"\nOK: {len(entries)} entr(y/ies) added, each well formed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
