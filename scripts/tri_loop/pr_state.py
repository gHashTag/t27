#!/usr/bin/env python3
"""tri pr-state -- every pull request a cron loop's state names, with the verdict `tri pr ready` gives it now.

WHY THIS EXISTS
---------------
On 2026-10-04 a loop wrote gHashTag/t27#5787 as "open" in six status lines in
a row while its `cli-tri` check had been red for hours: a census pin had moved.
"open" was true and said nothing -- every pull request in that state file was
open. The question a report needs answered is whether anything is red HERE that
is not red everywhere else, and `tri pr ready` answers exactly that for one
pull request. Nothing ran it over the nine the state named, in four
repositories, at the moment the report was written. This does.

NOT A NEW MEASUREMENT
---------------------
The verdict is `tri pr ready`'s, quoted with its exit code; the failing-check
names under it are its lines too. This adds no classification of checks. What
it adds is what only the state knows:

  head-moved   the pull request's head is not the `head` the state recorded:
               a push the state never saw (another session, or a tick that
               died before writing it down)
  not-open     merged or closed while the state still lists it; `tri pr ready`
               is not asked about it
  unresolved   the key names no repository. `owner/repo#N` names one;
               `alias#N` needs state.repos[alias]. An alias is never expanded
               by guessing the owner.
  unreadable   `gh pr view` gave no answer, so nothing else was asked
  no-verdict   `tri pr ready` did not run, or printed no `VERDICT:` line

The verdicts DO NOT MERGE, CANNOT TELL and WAIT are printed as verdicts, not as
anomalies of the state, but each of them makes the exit code 1: a loop report
may call a pull request done only when this exits 0.

READ-ONLY
---------
Nothing is written: not the state, not the ledger, not a pull request. Merging
is not offered -- `tri pr ready --merge` exists and is deliberately not passed.

WHAT THIS DOES NOT ESTABLISH
----------------------------
  * That a "safe to merge" pull request was reviewed, or should be merged. It
    says every failing check fails elsewhere too (`tri pr ready`'s baseline:
    the default branch and the last 5 merged pull requests).
  * That the verdict outlives the next push or the next CI run.
  * That anything was measured. When a repository's CI cannot run at all
    (gHashTag/999-multibots-telegraf, 2026-10-04: Actions billing-blocked,
    every check red on every pull request), "failing elsewhere too" is
    true and `tri pr ready` says safe -- about checks that never started.
  * Anything about a pull request the state does not list. `tri stranded`
    finds pushed work that has none.

    tri pr-state                     # the one directory under cron_tracking/
    tri pr-state --id 8782e5f8 --jobs 6
    tri pr-state --json

Exit codes: 0 every listed pull request is open, at its recorded head, and safe
by `tri pr ready`; 1 anything else; 2 usage.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from tick import default_dir, git, load_state, pick_id  # noqa: E402  (same directory, deliberate)

KEY = re.compile(r"^(?P<repo>[A-Za-z0-9_.-]+(?:/[A-Za-z0-9_.-]+)?)#(?P<n>\d+)$")
VERDICT = re.compile(r"^VERDICT:\s*(.*)$")
READY_TIMEOUT_S = 600


def resolve(key: str, aliases: dict) -> tuple[str | None, int | None, str | None]:
    """(owner/repo, number, None) or (None, None, why it does not resolve)."""
    m = KEY.match(key)
    if not m:
        return None, None, "not of the form owner/repo#N or alias#N"
    repo, n = m.group("repo"), int(m.group("n"))
    if "/" in repo:
        return repo, n, None
    full = aliases.get(repo)
    if isinstance(full, str) and full.count("/") == 1:
        return full, n, None
    return None, None, (f"alias {repo!r} is not in state.repos; add "
                        f'"repos": {{"{repo}": "<owner>/<repo>"}}')


def ask_gh_view(repo: str, n: int) -> tuple[dict | None, str | None]:
    try:
        r = subprocess.run(["gh", "pr", "view", str(n), "-R", repo, "--json",
                            "state,headRefOid,isDraft,url"],
                           capture_output=True, text=True, timeout=120)
    except FileNotFoundError:
        return None, "gh is not installed"
    except subprocess.TimeoutExpired:
        return None, "gh timed out after 120 s"
    if r.returncode != 0:
        return None, f"gh exited {r.returncode}: {(r.stderr.strip().splitlines() or [''])[0][:100]}"
    try:
        view = json.loads(r.stdout)
    except ValueError:
        return None, "gh did not return JSON"
    return (view, None) if isinstance(view, dict) else (None, "gh returned no object")


def ask_ready(tri: str, repo: str, n: int) -> tuple[int | None, str]:
    try:
        r = subprocess.run([tri, "pr", "ready", str(n), "--repo", repo],
                           capture_output=True, text=True, timeout=READY_TIMEOUT_S)
    except FileNotFoundError:
        return None, f"tri binary not found at {tri} (cargo build --release -p tri)"
    except subprocess.TimeoutExpired:
        return None, f"tri pr ready timed out after {READY_TIMEOUT_S} s"
    return r.returncode, r.stdout + r.stderr


def read_verdict(out: str) -> tuple[str | None, list[str]]:
    """The VERDICT line's text and the `  - name` lines that follow it."""
    lines = out.splitlines()
    for i, line in enumerate(lines):
        m = VERDICT.match(line.strip())
        if m:
            names = [l.strip()[2:] for l in lines[i + 1:] if l.startswith("  - ")]
            return m.group(1).strip(), names
    return None, []


def inspect(key: str, spec, aliases: dict, tri: str, answers: dict | None) -> dict:
    spec = spec if isinstance(spec, dict) else {}
    row = {"key": key, "repo": None, "number": None, "recorded_head": spec.get("head"),
           "state": None, "head": None, "draft": None, "url": None,
           "ready_exit": None, "verdict": None, "only_here": [], "note": None}
    repo, n, why = resolve(key, aliases)
    if repo is None:
        row["note"] = why
        return row
    row["repo"], row["number"] = repo, n
    full = f"{repo}#{n}"
    a = {}
    if answers is not None:
        a = answers.get(full) if isinstance(answers.get(full), dict) else {}
        view, err = (a["view"], None) if isinstance(a.get("view"), dict) else \
            (None, f"no view for {full} in --answers")
    else:
        view, err = ask_gh_view(repo, n)
    if view is None:
        row["note"] = err
        return row
    row["state"] = view.get("state")
    row["head"] = view.get("headRefOid")
    row["draft"] = view.get("isDraft")
    row["url"] = view.get("url")
    if row["state"] != "OPEN":
        return row
    if answers is not None:
        ready = a.get("ready") if isinstance(a.get("ready"), dict) else None
        code, out = ((ready.get("code"), str(ready.get("out", ""))) if ready else
                     (None, f"no ready answer for {full} in --answers"))
    else:
        code, out = ask_ready(tri, repo, n)
    row["ready_exit"] = code
    row["verdict"], row["only_here"] = read_verdict(out)
    if row["verdict"] is None:
        tail = [l for l in out.strip().splitlines() if l.strip()]
        row["note"] = tail[-1][:160] if tail else "no output"
    return row


def head_moved(row: dict) -> bool:
    rec, live = row["recorded_head"], row["head"]
    if not isinstance(rec, str) or not rec or not isinstance(live, str):
        return False
    return not live.lower().startswith(rec.lower())


def anomalies_for(rows: list[dict]) -> list[dict]:
    out = []

    def add(code, row, detail, fix):
        out.append({"code": code, "subject": f"prs.{row['key']}", "detail": detail, "fix": fix})

    for r in rows:
        if r["repo"] is None:
            add("unresolved", r, r["note"], "name the repository in the key or in state.repos")
            continue
        full = f"{r['repo']}#{r['number']}"
        if r["state"] is None:
            add("unreadable", r, f"gh pr view {full}: {r['note']}",
                "check `gh auth status` and the number; nothing else was asked")
            continue
        if r["state"] != "OPEN":
            add("not-open", r, f"{full} is {r['state']} and the state still lists it",
                f"`tri pr landed {r['number']} --repo {r['repo']} --probe ...` to see "
                f"whether its content reached the default branch, then move it to done")
            continue
        if head_moved(r):
            add("head-moved", r,
                f"head is {r['head'][:9]}, the state recorded {r['recorded_head']}",
                f"read `git log {r['recorded_head']}..{r['head'][:9]}` on the branch: "
                f"if the push is yours, record the new head; if not, another session "
                f"is on this branch (LOOP-RULES R17)")
        if r["verdict"] is None:
            add("no-verdict", r, f"tri pr ready {full}: {r['note']}",
                "run `tri pr ready` by hand and read its last lines")
    return out


def settled(rows: list[dict], found: list[dict]) -> bool:
    if found:
        return False
    return all(r["ready_exit"] == 0 and (r["verdict"] or "").startswith("safe to merge")
               for r in rows)


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(prog="tri pr-state", description=__doc__.split("\n")[0])
    ap.add_argument("--id", help="cron id (a directory under cron_tracking/)")
    ap.add_argument("--dir", help="the cron_tracking directory "
                                  "(default: <repository top>/cron_tracking)")
    ap.add_argument("--jobs", type=int, default=4,
                    help="pull requests asked at once (default 4; each `tri pr ready` "
                         "takes about a minute)")
    ap.add_argument("--tri", help="the Rust tri binary (default: $TRI_BIN, else "
                                  "<repository top>/target/release/tri)")
    ap.add_argument("--answers", metavar="FILE",
                    help='JSON {"owner/repo#N": {"view": {...}, "ready": {"code": N, '
                         '"out": "..."}}} used instead of asking gh and tri (offline use, tests)')
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args(argv)

    root = os.path.abspath(args.dir or default_dir())
    cid, code = pick_id(root, args.id, "tri pr-state")
    if cid is None:
        return code
    d = os.path.join(root, cid)
    state, err = load_state(d)
    if state is None:
        print(f"tri pr-state: {err}", file=sys.stderr)
        return 2
    prs = state.get("prs") if isinstance(state.get("prs"), dict) else {}
    aliases = state.get("repos") if isinstance(state.get("repos"), dict) else {}

    answers = None
    if args.answers:
        try:
            with open(args.answers) as fh:
                answers = json.load(fh)
        except (OSError, ValueError) as exc:
            print(f"tri pr-state: --answers {args.answers}: {exc}", file=sys.stderr)
            return 2
        if not isinstance(answers, dict):
            print(f"tri pr-state: --answers {args.answers}: not a JSON object", file=sys.stderr)
            return 2
    rc, top = git(".", "rev-parse", "--show-toplevel")
    tri = args.tri or os.environ.get("TRI_BIN") or os.path.join(
        top.strip() if rc == 0 else os.getcwd(), "target", "release", "tri")

    with ThreadPoolExecutor(max_workers=max(1, args.jobs)) as pool:
        rows = list(pool.map(lambda kv: inspect(kv[0], kv[1], aliases, tri, answers),
                             prs.items()))
    found = anomalies_for(rows)
    ok = settled(rows, found)

    if args.json:
        print(json.dumps({"cron_id": cid, "dir": d, "prs": rows, "anomalies": found,
                          "settled": ok}, indent=1))
        return 0 if ok else 1

    print(f"tri pr-state -- cron {cid}   ({len(rows)} pull request(s) named by the state)")
    for r in rows:
        where = f"{r['repo']}#{r['number']}" if r["repo"] else "(unresolved)"
        print(f"  {r['key']:<16} {where}")
        if r["state"] is None:
            continue
        draft = " draft" if r["draft"] else ""
        if r["head"]:
            mark = ("no head recorded" if not r["recorded_head"] else
                    f"state says {r['recorded_head']}" if head_moved(r) else
                    "as recorded")
            print(f"      {r['state']}{draft}   head {r['head'][:9]} ({mark})")
        else:
            print(f"      {r['state']}{draft}")
        if r["verdict"] is not None:
            print(f"      {r['verdict']}   [tri pr ready exit {r['ready_exit']}]")
            for name in r["only_here"]:
                print(f"        - {name}")
    print()
    print(f"ANOMALIES: {len(found)}")
    for a in found:
        print(f"  [{a['code']}] {a['subject']}: {a['detail']}")
        print(f"      fix: {a['fix']}")
    print()
    if ok:
        print("SETTLED: yes -- every listed pull request is open, at its recorded head,")
        print("and safe by `tri pr ready`.")
    else:
        print("SETTLED: no -- a report does not call these pull requests done until")
        print("this exits 0; it quotes the lines above instead.")
    print()
    print("NOT ESTABLISHED: that any check ran -- where CI cannot start at all (a")
    print("billing-blocked repository: every check red on every PR), \"failing")
    print("elsewhere too\" holds and `tri pr ready` says safe about nothing measured;")
    print("that a safe pull request was reviewed or should merge; that a verdict")
    print("outlives the next push; anything about work the state does not list")
    print("(`tri stranded`). Nothing was written, nothing was merged.")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
