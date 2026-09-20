#!/usr/bin/env python3
"""The doctor: turn what the oracle measured into work a bee can take.

WHY. The oracle has been telling this repository the truth for days and nobody
was listening in a form the swarm could act on. Measured 2026-09-17 on master:
of 906 specs, 654 generate Zig that does not compile, 89 do not generate at all,
13 compile and fail their own tests. Those are not 756 separate mysteries -
they cluster. `use of undeclared identifier` is one defect appearing in many
files; `invalid builtin function '@noop'` is another.

A cluster is a task. This reads the oracle's `results.tsv`, groups the failures
by the error the compiler actually printed, and opens one issue per cluster with
the command that reproduces it, the files it affects, and criteria measured
against the corpus rather than typed.

WHAT IT REFUSES TO DO. It does not invent a fix, it does not guess a cause, and
it does not file an issue for a cluster that already has one open: a fuel line
that duplicates its own work is a fuel line that buries the swarm.

Usage (from a repository root, after the oracle has run):

    python3 tools/queen/feed_defects.py --results /tmp/t27-oracle/results.tsv --dry-run
    python3 tools/queen/feed_defects.py --results /tmp/t27-oracle/results.tsv --limit 5
    python3 tools/queen/feed_defects.py --self-test
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from collections import defaultdict

REPO = os.environ.get("DOCTOR_REPO", "gHashTag/t27")
# How many files a cluster must touch before it is worth an issue of its own.
# One file with a unique error is a spec-level bug and the repairer fuel already
# covers it; the value here is in the shapes that repeat.
MIN_CLUSTER = 3
# What a cluster's issue is called, so a later run can find its own work.
TITLE = "Oracle cluster: {error} ({count} specs)"


def normalise(message: str) -> str:
    """The error, with everything file-specific removed.

    Two specs failing with `expected ',' after initializer` are the same defect
    whatever their paths, line numbers and identifiers say. Without this, 654
    failures look like 654 problems.
    """
    text = message.strip()
    text = re.sub(r"[\w./-]+\.zig:\d+:\d+:?", "", text)
    text = re.sub(r"\[in [^\]]+\]", "", text)
    text = re.sub(r"'[^']{1,60}'", "'X'", text)
    text = re.sub(r"\b\d+\b", "N", text)
    return re.sub(r"\s+", " ", text).strip()[:120]


def read_results(path: str) -> list[tuple[str, str, str]]:
    """(zig file, verdict, message) as tools/oracle/run.sh writes them."""
    rows: list[tuple[str, str, str]] = []
    with open(path, errors="replace") as handle:
        for line in handle:
            parts = line.rstrip("\n").split("\t")
            if len(parts) < 2:
                continue
            zig, verdict = parts[0], parts[1]
            message = parts[2] if len(parts) > 2 else ""
            rows.append((zig, verdict, message))
    return rows


def spec_of(zig_path: str) -> str:
    return "specs/" + re.sub(r"\.zig$", ".t27", zig_path)


def clusters(rows: list[tuple[str, str, str]]) -> dict[str, list[str]]:
    """The failing specs, grouped by the error the compiler printed."""
    found: dict[str, list[str]] = defaultdict(list)
    for zig, verdict, message in rows:
        if verdict in ("PASS", "NOGEN"):
            continue
        if not message.strip():
            continue
        found[normalise(message)].append(spec_of(zig))
    return {k: sorted(set(v)) for k, v in found.items() if len(set(v)) >= MIN_CLUSTER}


def open_titles() -> set[str]:
    done = subprocess.run(
        ["gh", "issue", "list", "--repo", REPO, "--state", "all", "--limit", "400",
         "--json", "title"],
        capture_output=True, text=True, timeout=180,
    )
    if done.returncode != 0:
        raise SystemExit("gh issue list failed: " + done.stderr[:200])
    return {row["title"] for row in json.loads(done.stdout or "[]")}


def measured(command: str, cwd: str = ".") -> str:
    done = subprocess.run(
        ["bash", "-c", command], capture_output=True, text=True, cwd=cwd,
        timeout=300, stdin=subprocess.DEVNULL,
    )
    return (done.stdout or "").strip()


def issue_for(error: str, specs: list[str], t27c: str) -> tuple[str, str]:
    first = specs[0]
    # MEASURED with the binary this job built, QUOTED as `t27c`, which is what a
    # bee has on PATH in its container. An issue that quotes a path from the
    # machine that wrote it is an issue nobody else can run - that mistake put
    # `/Users/playom/t27/target/release/t27c` into 137 open issues.
    run_cmd = (
        f"{t27c} gen {first} > /tmp/t27-doctor.zig && "
        "zig test /tmp/t27-doctor.zig --test-no-exec 2>&1 | head -3"
    )
    count_cmd = (
        f"t27c gen {first} > /tmp/t27-doctor.zig && "
        "zig test /tmp/t27-doctor.zig --test-no-exec 2>&1 | head -3"
    )
    today = measured(run_cmd) or "(the command printed nothing here)"
    body = [
        "## What the oracle measured",
        "",
        f"`{error}`",
        "",
        f"It is not one file's problem: **{len(specs)} specs fail the same way**. "
        "The oracle generates every spec to Zig and runs `zig test` over the result; "
        "these are the ones whose generated code the compiler refuses, with the same "
        "message.",
        "",
        "## Reproduce it",
        "",
        "```",
        f"$ {count_cmd}",
        today,
        "```",
        "",
        "## The specs this affects",
        "",
    ]
    body += [f"- `{spec}`" for spec in specs[:25]]
    if len(specs) > 25:
        body.append(f"- ...and {len(specs) - 25} more")
    body += [
        "",
        "## What to look at first",
        "",
        "The error is in GENERATED Zig, so the fix is in the generator or in the "
        "spec that feeds it - never in the `.zig`, which is overwritten on every "
        "run. If every spec in the list shares a construct, the generator is the "
        "suspect; if only some do, the specs are.",
        "",
        "## Acceptance criteria",
        "",
        f"- 1. `{count_cmd}` prints nothing (today it prints the error above)",
        f"- 2. `t27c spec-status {first}` does not print `NOPARSE`",
        f"- 3. `t27c gen {first} > /tmp/t27-doctor.zig && grep -c 'not yet implemented' "
        "/tmp/t27-doctor.zig` prints `0`",
        "- 4. `cargo test -p t27c` passes, if the fix is in the generator",
        "",
        "## Boundary",
        "",
        first,
    ]
    return TITLE.format(error=error[:70], count=len(specs)), "\n".join(body)


def self_test() -> int:
    """The negative control: what must and must not become a cluster."""
    rows = [
        ("a/b.zig", "NOCOMPILE", "a/b.zig:12:3: error: expected ',' after initializer"),
        ("c/d.zig", "NOCOMPILE", "c/d.zig:44:9: error: expected ',' after initializer"),
        ("e/f.zig", "NOCOMPILE", "e/f.zig:7:1: error: expected ',' after initializer"),
        ("g/h.zig", "NOCOMPILE", "g/h.zig:3:3: error: use of undeclared identifier 'foo'"),
        ("i/j.zig", "NOCOMPILE", "i/j.zig:9:3: error: use of undeclared identifier 'bar'"),
        ("k/l.zig", "PASS", ""),
        ("m/n.zig", "NOGEN", "generation failed"),
    ]
    found = clusters(rows)
    problems = 0
    if len(found) != 1:
        print(f"  self-test FAILED: expected 1 cluster at the floor of {MIN_CLUSTER}, got {len(found)}")
        problems += 1
    only = next(iter(found.values()), [])
    if sorted(only) != ["specs/a/b.t27", "specs/c/d.t27", "specs/e/f.t27"]:
        print(f"  self-test FAILED: the cluster holds {only}")
        problems += 1
    if normalise("x.zig:1:2: error: use of undeclared identifier 'foo'") != normalise(
        "y.zig:9:9: error: use of undeclared identifier 'bar'"
    ):
        print("  self-test FAILED: two identifiers must normalise to one error")
        problems += 1
    if normalise("error: expected ',' after initializer") == normalise(
        "error: use of undeclared identifier 'x'"
    ):
        print("  self-test FAILED: different errors must not collapse")
        problems += 1
    if problems:
        return 1
    print(
        "ok: 4 shapes - two identifiers collapse to one defect, two different errors "
        "do not, PASS and NOGEN are not defects, and a pair below the floor is not a cluster"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--results", default=os.environ.get("ORACLE_RESULTS", ""))
    parser.add_argument("--t27c", default=os.environ.get("T27C_BIN", "./target/release/t27c"))
    parser.add_argument("--limit", type=int, default=3)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        return self_test()

    if not args.results or not os.path.exists(args.results):
        print("could not run: no oracle results (pass --results)", file=sys.stderr)
        return 2
    rows = read_results(args.results)
    if not rows:
        print(f"could not run: {args.results} holds no rows", file=sys.stderr)
        return 2

    found = clusters(rows)
    print(f"{len(rows)} oracle row(s); {len(found)} cluster(s) of {MIN_CLUSTER}+ specs")
    known = open_titles()
    made = 0
    for error, specs in sorted(found.items(), key=lambda kv: -len(kv[1])):
        if made >= args.limit:
            break
        title, body = issue_for(error, specs, args.t27c)
        if title in known:
            print(f"  already filed: {title}")
            continue
        print(f"  x{len(specs)} {error[:80]}")
        if args.dry_run:
            out = os.path.join(tempfile.gettempdir(), "doctor-preview.md")
            with open(out, "w") as handle:
                handle.write(body)
            print(f"      dry-run: body written to {out}")
            made += 1
            continue
        with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False) as handle:
            handle.write(body)
            path = handle.name
        done = subprocess.run(
            ["gh", "issue", "create", "--repo", REPO, "--title", title, "--body-file", path],
            capture_output=True, text=True, timeout=180,
        )
        os.unlink(path)
        if done.returncode != 0:
            print(f"      create FAILED: {done.stderr.strip()[:200]}")
            continue
        print(f"      {done.stdout.strip()}")
        made += 1
    print(f"done: {made} issue(s) {'would be ' if args.dry_run else ''}created")
    return 0


if __name__ == "__main__":
    sys.exit(main())
