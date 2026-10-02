#!/usr/bin/env python3
"""Run the merger's REAL "Find ready PRs" script against a fake `gh`.

`.github/workflows/auto-merge-ready-prs.yml` decides what merges, and nothing
ran it except GitHub on a schedule -- so a gate that let the wrong approval
through would be found by a merge. This extracts the step's `run:` block from
the workflow file as committed, puts a stub `gh` first on PATH, and drives it
through one scenario per way a pull request must be refused (#5547).

The stub answers `--jq` by running the real `jq -r` over fixture JSON, so the
workflow's own filters are exercised, not a re-typed copy of them.

    python3 tools/bees/merger_gate_selftest.py      # needs bash and jq
"""

import json
import os
import pathlib
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
# MERGER_WORKFLOW points it at another copy, e.g. the pre-#5547 file, as a
# negative control: that one must FAIL here.
WORKFLOW = pathlib.Path(os.environ.get("MERGER_WORKFLOW")
                        or ROOT / ".github" / "workflows" / "auto-merge-ready-prs.yml")
REPO = "gHashTag/t27"
BOT = "t27-bees[bot]"
HEAD = "a" * 40
OLD = "b" * 40

STUB_GH = r'''#!/usr/bin/env python3
import json, os, subprocess, sys
fx = json.load(open(os.environ["FIXTURE"]))
a = sys.argv[1:]
jq = a[a.index("--jq") + 1] if "--jq" in a else None
def out(data):
    if jq is None:
        print(json.dumps(data)); return
    r = subprocess.run(["jq", "-r", jq], input=json.dumps(data), capture_output=True, text=True)
    sys.stdout.write(r.stdout); sys.exit(r.returncode)
if a[:2] == ["pr", "list"]:
    out([{"number": n} for n in fx["prs"]])
elif a[:2] == ["pr", "view"]:
    out(fx["prs"][a[2]]["view"])
elif a[0] == "api":
    path = a[1]
    for n, p in fx["prs"].items():
        if path.endswith(f"/issues/{n}/events"):
            out(p["events"])
        if path.endswith(f"/pulls/{n}/reviews"):
            if p.get("reviews_fail"):
                sys.exit(1)
            out(p["reviews"])
        sha = p["view"]["headRefOid"]
        if path.endswith(f"/commits/{sha}"):
            out({"commit": {"committer": {"date": p["committed"]}}})
        if f"/commits/{sha}/check-runs" in path:
            out({"check_runs": [{"started_at": p["checks_started"]}]})
    sys.exit(1)
else:
    sys.exit(1)
'''


def extract_find_ready():
    lines = WORKFLOW.read_text().splitlines()
    i = next(n for n, l in enumerate(lines) if l.strip() == "id: find-ready")
    while lines[i].strip() != "run: |":
        i += 1
    indent = len(lines[i + 1]) - len(lines[i + 1].lstrip())
    body = []
    for l in lines[i + 1:]:
        if l.strip() and len(l) - len(l.lstrip()) < indent:
            break
        body.append(l[indent:])
    return "\n".join(body).replace("${{ github.repository }}", REPO)


def review(login, state, sha, at):
    return {"user": {"login": login}, "state": state, "commit_id": sha, "submitted_at": at}


def label(by, at, name="bee-reviewed"):
    return {"event": "labeled", "label": {"name": name}, "created_at": at, "actor": {"login": by}}


def pr(reviews, events, **kw):
    p = {
        "view": {"title": "x (Closes #1)", "body": "", "labels": [{"name": "bee-reviewed"}],
                 "headRefOid": HEAD,
                 "statusCheckRollup": [{"name": "build", "conclusion": "SUCCESS"}]},
        "committed": "2026-10-02T10:00:00Z",
        "checks_started": "2026-10-02T10:01:00Z",
        "reviews": reviews,
        "events": events,
    }
    # The same reviews in `gh pr view --json reviews` shape, so a workflow that
    # reads them from there (the pre-#5547 one) is judged on the same facts.
    p["view"]["reviews"] = [{"author": {"login": r["user"]["login"]}, "state": r["state"]}
                            for r in reviews]
    p.update(kw)
    return p


GOOD_REVIEW = review(BOT, "APPROVED", HEAD, "2026-10-02T10:05:00Z")
GOOD_LABEL = label(BOT, "2026-10-02T10:06:00Z")

# (name, login variable, pr fixture, expected to be ready)
SCENARIOS = [
    ("bee approved this head after it arrived, bee labeled", BOT,
     pr([GOOD_REVIEW], [GOOD_LABEL]), True),
    ("only the owner approved", BOT,
     pr([review("gHashTag", "APPROVED", HEAD, "2026-10-02T10:05:00Z")], [GOOD_LABEL]), False),
    ("bee approved an older SHA", BOT,
     pr([review(BOT, "APPROVED", OLD, "2026-10-02T10:05:00Z")], [GOOD_LABEL]), False),
    ("bee approved, then requested changes", BOT,
     pr([GOOD_REVIEW, review(BOT, "CHANGES_REQUESTED", HEAD, "2026-10-02T10:07:00Z")],
        [GOOD_LABEL]), False),
    ("bee approved, then only commented (still approved)", BOT,
     pr([GOOD_REVIEW, review(BOT, "COMMENTED", HEAD, "2026-10-02T10:07:00Z")], [GOOD_LABEL]), True),
    ("bee approval dismissed", BOT,
     pr([GOOD_REVIEW, review(BOT, "DISMISSED", HEAD, "2026-10-02T10:07:00Z")], [GOOD_LABEL]), False),
    ("bee approved before a force-push re-delivered the head", BOT,
     pr([GOOD_REVIEW], [{"event": "head_ref_force_pushed", "created_at": "2026-10-02T10:05:30Z",
                         "actor": {"login": "gHashTag"}}, label(BOT, "2026-10-02T10:08:00Z")]),
     False),
    ("label applied by the owner, not the bee", BOT,
     pr([GOOD_REVIEW], [label("gHashTag", "2026-10-02T10:06:00Z")]), False),
    ("reviews cannot be read", BOT,
     pr([GOOD_REVIEW], [GOOD_LABEL], reviews_fail=True), False),
    ("variable set to a human login: nothing merges", "gHashTag",
     pr([review("gHashTag", "APPROVED", HEAD, "2026-10-02T10:05:00Z")],
        [label("gHashTag", "2026-10-02T10:06:00Z")]), False),
    ("variable empty: nothing merges", "",
     pr([GOOD_REVIEW], [GOOD_LABEL]), False),
]


def run(script, login, fixture, tmp):
    fx_path = tmp / "fixture.json"
    fx_path.write_text(json.dumps({"prs": {"7": fixture}}))
    out_path = tmp / "out"
    out_path.write_text("")
    env = dict(os.environ, PATH=f"{tmp}:{os.environ['PATH']}", FIXTURE=str(fx_path),
               GITHUB_OUTPUT=str(out_path), BEE_REVIEWER_LOGIN=login)
    r = subprocess.run(["bash", "-e", "-c", script], env=env, capture_output=True, text=True,
                       stdin=subprocess.DEVNULL, timeout=600)
    outputs = dict(l.split("=", 1) for l in out_path.read_text().splitlines() if "=" in l)
    return r, outputs


def main():
    script = extract_find_ready()
    tmp = pathlib.Path(tempfile.mkdtemp(prefix="merger-gate-"))
    gh = tmp / "gh"
    gh.write_text(STUB_GH)
    gh.chmod(0o755)
    failures = 0
    for name, login, fixture, want in SCENARIOS:
        r, outputs = run(script, login, fixture, tmp)
        ready = outputs.get("ready_prs", "").split()
        got = "7" in ready
        ok = r.returncode == 0 and got == want and outputs.get("count") is not None
        failures += not ok
        print(f"  {'ok  ' if ok else 'FAIL'} {name}: {'ready' if got else 'skipped'}")
        if not ok:
            print(r.stdout[-1500:], r.stderr[-800:])
    print(f"merger gate self-test: {failures} failure(s) of {len(SCENARIOS)}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
