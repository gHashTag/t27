#!/usr/bin/env python3
"""A function with no test is a claim. There are 1427 of them, and the swarm was idle.

WHY THIS EXISTS

Measured 2026-09-20 on the live deployment: ten workers, and a tick skipping
697 candidate issues - 554 with no `## Boundary`, 43 already claimed, 43 already
completed. Of 673 open issues only 119 carry a boundary at all. The swarm was
not slow; it had run out of work it could take.

The feeders that existed had drained their pools. `feed_empty_bodies.py`
reported `0 uncovered` on the same morning: 28 files hold 146 empty bodies and
an open issue already covers 100 files. `feed_defects.py` files one issue per
oracle error CLUSTER, which is a handful. Nothing else generated fuel.

This is the next pool, and it is the compiler's own answer rather than a grep:

    $ t27c coverage specs/git/diff.t27
    Functions: 15
    Tested:    12 (80%)
    Untested:  3

Measured over the corpus with that command, one spec at a time: 887 specs
answer, 278 of them have at least one untested function, and 1427 functions
have none. 236 of those specs have between one and eight, which is one issue
each and 692 functions of work.

WHY EIGHT IS THE CEILING HERE

A criterion has to be exact. "`Untested:` has gone down" is not a string a
grep can match; `Untested:  0` is. So this files only for specs a bee can
finish - 1..8 untested functions - and the 42 specs above that line are left
for a change that can chunk them without writing a criterion nobody can check.
They are reported, not silently dropped.

WHAT STOPS A BEE SATISFYING THIS BY DELETING THE FUNCTIONS

`Untested: 0` is also what an empty file prints. So the function count is a
criterion too, and it is measured here rather than typed. Eleven issues once
claimed "today: 0 tests" about files carrying nine, because the number was
written by one piece of code and the command by another - every number below is
produced by running the exact string the issue prints.

Run it from the repository root with a built t27c on PATH (or T27C_BIN):

    python3 tools/queen/feed_untested.py --dry-run --limit 3
    python3 tools/queen/feed_untested.py --when-idle --limit 8
"""
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys

# ONE implementation of the machinery, not two. The empty-bodies feeder already
# knows how to run a command as the bee will run it, how to refuse an issue
# whose numbers do not reproduce, which files an open issue already claims, and
# how much room the swarm has. A second copy of that is the duplication this
# repository gates against in its specs.
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from feed_empty_bodies import (  # noqa: E402
    BEE_T27C, OUTD, PREAMBLE, REPO, T27C, TEST_GREP, WORK,
    bee_shell, claims_hold, log, open_boundaries, queue_idle, run,
    scenario_and_requirements, sync_master,
)

MAX_UNTESTED = 8
FN_GREP = "grep -cE '^[[:space:]]*(pub[[:space:]]+)?fn[[:space:]]'"


def coverage(rel):
    """(functions, untested, [names]) as `t27c coverage` reports them, or None.

    The compiler's own answer. A regex over the source was tried for this in
    an earlier pass and failed the same way every time: `test "name"` and
    `test name {` are both valid, the corpus holds 13201 of the second spelling
    against 2072 of the first, and a matcher that knows one of them reports a
    file as untested while its tests run.
    """
    out = run([T27C, "coverage", rel])
    fns = re.search(r"(?m)^Functions:\s+(\d+)", out)
    unt = re.search(r"(?m)^Untested:\s+(\d+)", out)
    if not fns or not unt:
        return None
    names = []
    seen_header = False
    for line in out.split("\n"):
        if line.startswith("--- untested functions"):
            seen_header = True
            continue
        if seen_header:
            name = line.strip()
            if name and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", name):
                names.append(name)
    return int(fns.group(1)), int(unt.group(1)), names


def signature_of(text, name):
    """The signature exactly as the file writes it, or None if it is not there."""
    m = re.search(
        r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:extern\s+)?fn\s+" + re.escape(name) + r"\s*[(<]",
        text,
    )
    if not m:
        return None
    depth, end = 0, len(text)
    for k in range(m.end(), len(text)):
        ch = text[k]
        if ch in "([<":
            depth += 1
        elif ch in ")]>":
            depth = max(0, depth - 1)
        elif ch in "{;" and depth == 0:
            end = k
            break
    return " ".join(text[m.start():end].split())


def build(rel):
    """One issue for one spec, or None when the instruments disagree."""
    with open(os.path.join(WORK, rel), errors="replace") as handle:
        text = handle.read()
    measured = coverage(rel)
    if not measured:
        log(f"skip {rel}: t27c coverage printed no Functions/Untested line")
        return None
    total, untested, names = measured
    if not (1 <= untested <= MAX_UNTESTED) or len(names) != untested:
        log(f"skip {rel}: coverage says {untested} untested and names {len(names)}")
        return None

    sigs = [(n, signature_of(text, n)) for n in names]
    if any(s is None for _, s in sigs):
        missing = [n for n, s in sigs if s is None]
        log(f"skip {rel}: coverage names {missing} which the source does not declare")
        return None

    cov_cmd = f"{BEE_T27C} coverage {rel} 2>&1 | grep -cE '^Untested: +0$'"
    fn_cmd = f"{FN_GREP} {rel}"
    test_cmd = f"{TEST_GREP} {rel}"
    status_cmd = f"{BEE_T27C} spec-status {rel}"

    fn_now = bee_shell(fn_cmd)
    test_now = bee_shell(test_cmd)
    status_now = bee_shell(status_cmd).split("\n")[0].strip()
    if not fn_now.isdigit() or not test_now.isdigit():
        log(f"skip {rel}: the counting commands printed {fn_now!r} and {test_now!r}")
        return None
    # The one-word verdict, or no issue. An empty answer here produced eight
    # issues reading ``prints `` (today: )`` - a criterion nobody can satisfy,
    # on work the oracle had already passed.
    if not status_now:
        log(f"skip {rel}: `{status_cmd}` printed nothing, and an empty answer is "
            "not a measurement")
        return None
    # Every number the issue prints, produced by the command the issue prints.
    if not claims_hold(rel, [(cov_cmd, 0), (fn_cmd, fn_now), (test_cmd, test_now),
                             (status_cmd, status_now)]):
        return None
    blocked = bee_shell(f"{BEE_T27C} test-report {rel} 2>&1 | grep -c BLOCKED")

    title = (f"Test the {untested} untested function"
             f"{'' if untested == 1 else 's'} in {rel}")
    L = [f"# {title}\n", "## Context\n",
         f"`{BEE_T27C} coverage {rel}` says this file declares {total} function"
         f"{'' if total == 1 else 's'} and {untested} of them "
         f"{'has' if untested == 1 else 'have'} no test. A function with no test is a "
         "claim: it compiles, it generates, and nothing in this repository says what it "
         "is supposed to do.\n",
         PREAMBLE,
         "## Current state - re-run these yourself, from the repository root\n", "```",
         f"$ {BEE_T27C} coverage {rel}",
         f"Functions: {total}", f"Untested:  {untested}", "```\n",
         *scenario_and_requirements(
             rel,
             f"{untested} of its {total} function"
             f"{'' if total == 1 else 's'} {'has' if untested == 1 else 'have'} no test",
             names,
             "each test MUST assert on a result. `assert true` and a body of only "
             "comments are counted as vacuous by `t27c validate-vacuity`, and the "
             "corpus ratchet fails a file that gains one."),
         "## What has no test\n",
         "Quoted verbatim from the file - keep every signature exactly as it is:\n"]
    for i, (name, sig) in enumerate(sigs, 1):
        L.append(f"{i}. `{sig}`")
    L += ["",
          f"The file already has {test_now} `test` declaration(s). Write one for each "
          "function above. Either spelling is accepted: `test name {` or `test \"name\" {`.\n",
          "A test that asserts nothing is not coverage. `assert true` and a body of only "
          "comments are both counted as vacuous by `t27c validate-vacuity`, and the corpus "
          "ratchet fails a file that gains one.\n",
          "## Acceptance criteria\n",
          f"- 1. `{cov_cmd}` prints `1` (today: 0 - it prints `Untested:  {untested}`)",
          f"- 2. `{fn_cmd}` prints `{fn_now}` - the same functions are still there "
          "(deleting a function also makes it untested-free)",
          f"- 3. `{test_cmd}` prints at least `{int(test_now) + untested}` (today: {test_now})",
          f"- 4. `{status_cmd}` prints `{status_now}` (today: {status_now})"]
    if blocked == "0":
        L.append(f"- 5. `{BEE_T27C} test-report {rel} 2>&1 | grep -c BLOCKED` prints `0` "
                 "- this spec compiles today and must still compile (today: 0)")
    # `t27c lint` is NOT a criterion here, and the reason is measured. On 60 specs
    # it printed 677 `has no test or invariant` warnings where `t27c coverage`
    # found 190 untested functions, disagreeing on 59 of the 60. Checked by hand
    # on specs/base/ternary_encoding.t27: lint warns about `bit_to_trit_pair`,
    # which `test bit_to_trit_pair_zero` calls on line 249. Coverage is right and
    # lint over-reports, so a criterion on lint would be one no bee can satisfy.
    # Filed as its own defect; the warning is quoted in the brief as a hint, not
    # as a gate.
    L += ["", "## Boundary\n", rel]
    return title, "\n".join(L) + "\n"


def population():
    """Every spec with 1..MAX_UNTESTED untested functions, smallest first.

    Smallest first on purpose: a bee that finishes is a bee that is available
    again, and the 42 specs above the ceiling are reported rather than dropped
    in silence.
    """
    specs = sorted(
        os.path.relpath(os.path.join(d, f), WORK)
        for d, _, fs in os.walk(os.path.join(WORK, "specs"))
        for f in fs if f.endswith(".t27")
    )
    small, big, none = [], 0, 0
    for rel in specs:
        measured = coverage(rel)
        if not measured:
            continue
        _, untested, _ = measured
        if untested == 0:
            none += 1
        elif untested <= MAX_UNTESTED:
            small.append((rel, untested))
        else:
            big += 1
    small.sort(key=lambda x: (x[1], x[0]))
    return small, big, none


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=8)
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--when-idle", action="store_true")
    ap.add_argument("--runway", default="0",
                    help="top the queue up to this many dispatchable issues "
                         "whatever the lanes are doing; `auto` means twice the "
                         "lanes the swarm reports")
    args = ap.parse_args()

    if args.when_idle:
        want = queue_idle(args.runway)
        if want is None:
            log("could not read the swarm, so nothing was fed - and this run is RED "
                "rather than a green run that fed nothing")
            raise SystemExit(2)
        if want <= 0:
            log("swarm busy - nothing added")
            return
        args.limit = min(args.limit, want)

    sha = sync_master()
    small, big, clean = population()
    covered = open_boundaries()
    todo = [(rel, n) for rel, n in small if rel not in covered]
    log(f"master {sha}: {len(small)} spec(s) have 1..{MAX_UNTESTED} untested functions "
        f"({sum(n for _, n in small)} functions); {big} spec(s) have more and are left for "
        f"a change that can chunk them; {clean} are fully tested; "
        f"{len(small) - len(todo)} already covered by an open issue; {len(todo)} uncovered")

    os.makedirs(OUTD, exist_ok=True)
    made = 0
    for rel, _ in todo:
        if made >= args.limit:
            break
        built = build(rel)
        if not built:
            continue
        title, body = built
        slug = re.sub(r"[^a-z0-9]+", "-", title.lower()).strip("-")[:120]
        path = os.path.join(OUTD, slug + ".md")
        with open(path, "w") as handle:
            handle.write(body)
        if args.dry_run:
            log(f"dry-run: {title} -> {path}")
            made += 1
            continue
        done = subprocess.run(
            ["gh", "issue", "create", "--repo", REPO, "--title", title, "--body-file", path],
            capture_output=True, text=True, timeout=120,
        )
        if done.returncode != 0:
            log(f"create FAILED for {title}: {done.stderr.strip()[:200]}")
            continue
        log(f"created {done.stdout.strip()} {title}")
        made += 1
    log(f"done: {made} issue(s) {'would be ' if args.dry_run else ''}created")


if __name__ == "__main__":
    main()
