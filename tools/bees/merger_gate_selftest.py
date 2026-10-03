#!/usr/bin/env python3
"""Run the merger's REAL "Find ready PRs" script against a fake `gh`.

`.github/workflows/auto-merge-ready-prs.yml` decides what merges, and nothing
ran it except GitHub on a schedule -- so a gate that let the wrong approval
through would be found by a merge. This extracts the step's `run:` block from
the workflow file as committed, puts a stub `gh` first on PATH, and drives it
through one scenario per way a pull request must be refused (#5547).

The stub answers `--jq` by running the real `jq -r` over fixture JSON, so the
workflow's own filters are exercised, not a re-typed copy of them.

The red-check scenarios use a review body built by `reviewer.compose_body`,
the function the reviewer bee posts with, so a format change on either side
fails here before it fails on a real pull request.

    python3 tools/bees/merger_gate_selftest.py      # needs bash and jq
"""

import json
import os
import pathlib
import re
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import reviewer  # noqa: E402
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
    if "/rules/branches/" in path:
        if fx.get("rules_fail"):
            sys.exit(1)
        out(fx["rules"])
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


WORKFLOW_NAME = re.search(r"^name:\s*(.+)$", WORKFLOW.read_text(), re.M).group(1).strip()


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


def review(login, state, sha, at, body=""):
    return {"user": {"login": login}, "state": state, "commit_id": sha, "submitted_at": at,
            "body": body}


def check(name, conclusion="SUCCESS", **kw):
    return {"__typename": "CheckRun", "name": name, "status": "COMPLETED" if conclusion else "IN_PROGRESS",
            "conclusion": conclusion, "workflowName": "CI", **kw}


def label(by, at, name="bee-reviewed"):
    return {"event": "labeled", "label": {"name": name}, "created_at": at, "actor": {"login": by}}


def pr(reviews, events, **kw):
    p = {
        "view": {"title": "x (Closes #1)", "body": "", "labels": [{"name": "bee-reviewed"}],
                 "headRefOid": HEAD, "baseRefName": "master",
                 "statusCheckRollup": [check("validate"), check("build")]},
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
RULES = [{"type": "pull_request", "parameters": {}},
         {"type": "required_status_checks",
          "parameters": {"required_status_checks": [{"context": "validate"}]}}]


def bee_body(*discounted, kind="approve"):
    """The body the reviewer bee posts, built by the bee's own function."""
    text = "BEE-VERDICT: APPROVE\nsummary: ok\ncriterion: the issue's goal -- met -- diff\n"
    text += "".join(f"discounted-check: {d} -- red on master too\n" for d in discounted)
    v = reviewer.parse_verdict(text)
    return reviewer.compose_body(kind, HEAD, v, list(discounted), "evidence", "self-test")


def with_checks(extra, body="", **kw):
    p = pr([review(BOT, "APPROVED", HEAD, "2026-10-02T10:05:00Z", body)], [GOOD_LABEL], **kw)
    p["view"]["statusCheckRollup"] = [check("validate"), check("build")] + extra
    return p

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
    # -- the check gate: what a red check needs (#5547 follow-up, 2026-10-03)
    ("red advisory check, discounted in the bee's approval", BOT,
     with_checks([check("spec-guards", "FAILURE")], bee_body("spec-guards")), True),
    ("red advisory check, not discounted", BOT,
     with_checks([check("spec-guards", "FAILURE")], bee_body()), False),
    ("two red checks, only one discounted", BOT,
     with_checks([check("spec-guards", "FAILURE"), check("check", "FAILURE")], bee_body("spec-guards")),
     False),
    ("red REQUIRED check, discounted anyway", BOT,
     dict(with_checks([], bee_body("validate")),
          view=dict(with_checks([], "")["view"], statusCheckRollup=[check("validate", "FAILURE")])),
     False),
    ("required check never posted", BOT,
     dict(with_checks([], ""), view=dict(with_checks([], "")["view"], statusCheckRollup=[check("build")])),
     False),
    ("a check still running", BOT,
     with_checks([check("spec-guards", "")], bee_body("spec-guards")), False),
    ("ruleset unreadable: a discounted red check still blocks", BOT,
     with_checks([check("spec-guards", "FAILURE")], bee_body("spec-guards"), rules_fail=True), False),
    ("ruleset unreadable, every check green", BOT,
     with_checks([], "", rules_fail=True), True),
    ("discount written in a later COMMENT, not in the approval", BOT,
     dict(with_checks([check("spec-guards", "FAILURE")], bee_body()),
          reviews=[review(BOT, "APPROVED", HEAD, "2026-10-02T10:05:00Z", bee_body()),
                   review(BOT, "COMMENTED", HEAD, "2026-10-02T10:05:30Z", bee_body("spec-guards"))]),
     False),
    ("discount by a human's approval does not count", BOT,
     dict(with_checks([check("spec-guards", "FAILURE")], bee_body()),
          reviews=[review(BOT, "APPROVED", HEAD, "2026-10-02T10:05:00Z", bee_body()),
                   review("gHashTag", "APPROVED", HEAD, "2026-10-02T10:05:30Z", bee_body("spec-guards"))]),
     False),
    ("red commit status (StatusContext), discounted by its context", BOT,
     with_checks([{"__typename": "StatusContext", "context": "ext/scan", "state": "FAILURE"}],
                 bee_body("ext/scan")), True),
    ("pending commit status blocks", BOT,
     with_checks([{"__typename": "StatusContext", "context": "ext/scan", "state": "PENDING"}],
                 bee_body("ext/scan")), False),
    ("this workflow's own in-progress run is not a pending check", BOT,
     with_checks([check("auto-merge", "", workflowName="Auto Merge Ready PRs")], ""), True),
]


def run(script, login, fixture, tmp):
    fx_path = tmp / "fixture.json"
    fixture = dict(fixture)
    top = {k: fixture.pop(k) for k in ("rules_fail",) if k in fixture}
    fx_path.write_text(json.dumps({"prs": {"7": fixture}, "rules": RULES, **top}))
    out_path = tmp / "out"
    out_path.write_text("")
    env = dict(os.environ, PATH=f"{tmp}:{os.environ['PATH']}", FIXTURE=str(fx_path),
               GITHUB_OUTPUT=str(out_path), BEE_REVIEWER_LOGIN=login, SELF_WORKFLOW=WORKFLOW_NAME)
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
        # A ready PR also hands the merge step the exact head it judged.
        pinned = (outputs.get("ready_heads", "").split() == [f"7:{HEAD}"]) if want else True
        ok = r.returncode == 0 and got == want and pinned and outputs.get("count") is not None
        failures += not ok
        print(f"  {'ok  ' if ok else 'FAIL'} {name}: {'ready' if got else 'skipped'}")
        if not ok:
            print(r.stdout[-1500:], r.stderr[-800:])
    # The merge step must merge THAT head and nothing newer: a push between the
    # gate and the merge is code no bee reviewed.
    merge_step = WORKFLOW.read_text().split("- name: Merge Ready PRs", 1)[-1]
    pin_ok = ('--match-head-commit "$sha"' in merge_step
              and "steps.find-ready.outputs.ready_heads" in merge_step)
    failures += not pin_ok
    print(f"  {'ok  ' if pin_ok else 'FAIL'} merge step pins the judged head (--match-head-commit)")
    # The reviewer must leave out exactly the checks this gate leaves out, or it
    # asks a bee to answer for a check the merger never reads (or skips one it does).
    skipped_here = re.findall(r'\.name != "([^"]+)"', script)
    drift_ok = (set(skipped_here) == set(reviewer.IGNORED_CHECKS)
                and reviewer.IGNORED_WORKFLOWS == (WORKFLOW_NAME,))
    failures += not drift_ok
    print(f"  {'ok  ' if drift_ok else 'FAIL'} reviewer ignores the same checks as this gate"
          f"{'' if drift_ok else f' (gate {skipped_here}, reviewer {reviewer.IGNORED_CHECKS}, {reviewer.IGNORED_WORKFLOWS})'}")
    print(f"merger gate self-test: {failures} failure(s) of {len(SCENARIOS) + 2}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
