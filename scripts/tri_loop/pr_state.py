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

The verdicts DO NOT MERGE, CANNOT TELL, NEW REASON and WAIT are printed as
verdicts, not as anomalies of the state, but each of them makes the exit code
1: a loop report may call a pull request done only when this exits 0.

THE LISTS UNDER A VERDICT
-------------------------
`tri pr ready` can print up to three lists after its VERDICT line, and a
fourth before it. Each name is kept in the list it was printed under:

  only_here     "DO NOT MERGE -- N appear only here", or "and N appear only
                here" under CANNOT TELL
  no_baseline   "CANNOT TELL -- N have no baseline to compare against"
  new_reason    "NEW REASON -- N are red elsewhere too, but not for the same
                reason", or "and N ... not for the same reason" under another
                verdict (--why only)
  not_compared  "NOT compared, so not established either way" (--why only,
                printed above the VERDICT line)

A list under a heading this does not know is kept as `other`, with the
heading, and printed -- never dropped. Until 2026-10-04 every `  - ` line after
the VERDICT went into one list called only_here, so CANNOT TELL's no-baseline
names, and a DO NOT MERGE's new-reason names, read as "only here".

--why
-----
Without it a failure is called pre-existing when a check of the same NAME is
red elsewhere. `--why` passes `--why` to `tri pr ready` (t27#5853), which
compares the failing step's own output on both sides; a different reason is
NEW REASON, exit 7. It reads two job logs per failure, so it is slower: on
2026-10-04, the 18 pull requests of cron 8782e5f8 at --jobs 11 took 115 s
without it, and 168 s and 220 s in two runs with it.
The card ends with a REASONS line naming each pull request with such a
failure, counted over the pull requests tri gave a verdict -- the rest are
named as not compared, so "0" never stands for "nothing was compared". A tri
built without the flag rejects it with exit 2 -- WAIT's code -- so that answer
is reported as no-verdict and named, never read as a verdict.

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
  * Without --why: that a failure called pre-existing fails for the same
    reason -- only its check's name was compared. With it: that the same
    failing-step text is the same cause, or that a NEW REASON was caused by
    the change (`tri pr ready --why` says both of itself).

    tri pr-state                     # the one directory under cron_tracking/
    tri pr-state --id 8782e5f8 --jobs 6
    tri pr-state --json
    tri pr-state --why --tri <a tri built with t27#5853>

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
# Each list `tri pr ready` prints, by the heading it prints over it.
VERDICT_LIST = (("DO NOT MERGE", "only_here"), ("CANNOT TELL", "no_baseline"),
                ("NEW REASON", "new_reason"))
AND_LIST = ((re.compile(r"^and \d+ failure\(s\) appear only here:$"), "only_here"),
            (re.compile(r"^and \d+ failure\(s\) red elsewhere too, but not for the same "
                        r"reason:$"), "new_reason"))
NOT_COMPARED = re.compile(r"^NOT compared, so not established either way: \d+ failure\(s\):$")
LISTS = ("only_here", "no_baseline", "new_reason", "not_compared")
NO_WHY_FLAG = "unexpected argument '--why'"


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


def ask_ready(tri: str, repo: str, n: int, why: bool = False) -> tuple[int | None, str]:
    try:
        r = subprocess.run([tri, "pr", "ready", str(n), "--repo", repo,
                            *(["--why"] if why else [])],
                           capture_output=True, text=True, timeout=READY_TIMEOUT_S)
    except FileNotFoundError:
        return None, f"tri binary not found at {tri} (cargo build --release -p tri)"
    except subprocess.TimeoutExpired:
        return None, f"tri pr ready timed out after {READY_TIMEOUT_S} s"
    return r.returncode, r.stdout + r.stderr


def read_verdict(out: str) -> tuple[str | None, dict]:
    """The VERDICT line's text, and each `  - name` under the heading it was
    printed under: LISTS, plus `other` -- [{"head", "names"}] -- for names
    under a heading this does not know, or under none. Above the VERDICT
    only NOT_COMPARED is a list; the rest there is tri's per-check report."""
    lists = {k: [] for k in LISTS}
    lists["other"] = []
    verdict, cur = None, None

    def other(head):
        lists["other"].append({"head": head, "names": []})
        return "other"

    for line in out.splitlines():
        s = line.strip()
        m = VERDICT.match(s)
        if m and verdict is None:
            verdict = m.group(1).strip()
            cur = next((k for word, k in VERDICT_LIST if verdict.startswith(word)), None)
            if cur is None:
                cur = other("under the VERDICT line:")
            continue
        if NOT_COMPARED.match(s):
            cur = "not_compared"
            continue
        if not s:
            cur = None
            continue
        if line.startswith("  - "):
            if cur is None and verdict is not None:
                cur = other("(under no heading):")
            if cur == "other":
                lists["other"][-1]["names"].append(s[2:])
            elif cur:
                lists[cur].append(s[2:])
            continue
        if verdict is None:
            cur = None
            continue
        cur = next((k for rx, k in AND_LIST if rx.match(s)), None)
        if cur is None and not line[0].isspace():
            cur = other(s)
    lists["other"] = [o for o in lists["other"] if o["names"]]
    return verdict, lists


def inspect(key: str, spec, aliases: dict, tri: str, answers: dict | None,
            why: bool = False) -> dict:
    spec = spec if isinstance(spec, dict) else {}
    row = {"key": key, "repo": None, "number": None, "recorded_head": spec.get("head"),
           "state": None, "head": None, "draft": None, "url": None,
           "ready_exit": None, "verdict": None, "only_here": [], "no_baseline": [],
           "new_reason": [], "not_compared": [], "other": [], "note": None}
    repo, n, bad = resolve(key, aliases)
    if repo is None:
        row["note"] = bad
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
        code, out = ask_ready(tri, repo, n, why)
    row["ready_exit"] = code
    row["verdict"], lists = read_verdict(out)
    row.update(lists)
    if row["verdict"] is None:
        tail = [l for l in out.strip().splitlines() if l.strip()]
        row["note"] = tail[-1][:160] if tail else "no output"
        if NO_WHY_FLAG in out:
            row["note"] = (f"this tri has no --why ({tri}); exit {code} is clap's usage "
                           f"error, not WAIT -- build t27#5853's branch and pass --tri")
    return row


HEADS = {"only_here": "and {} failure(s) appear only here:",
         "no_baseline": "and {} failure(s) have no baseline to compare against:",
         "new_reason": "and {} failure(s) red elsewhere too, but not for the same reason:",
         "not_compared": "--why could not compare {} failure(s) (not established either way):"}


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
    ap.add_argument("--why", action="store_true",
                    help="pass --why to tri pr ready: a failure red elsewhere is compared "
                         "by its failing step's output, not its name (t27#5853; slower: 18 PRs "
                         "took 168-220 s, 115 s without, on 2026-10-04)")
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
        rows = list(pool.map(lambda kv: inspect(kv[0], kv[1], aliases, tri, answers,
                                                args.why), prs.items()))
    found = anomalies_for(rows)
    ok = settled(rows, found)

    if args.json:
        print(json.dumps({"cron_id": cid, "dir": d, "why": args.why, "prs": rows,
                          "anomalies": found, "settled": ok}, indent=1))
        return 0 if ok else 1

    print(f"tri pr-state -- cron {cid}   ({len(rows)} pull request(s) named by the state)")
    if args.why:
        print("  --why: a failure red elsewhere is compared by what its failing step printed")
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
            own = next((k for word, k in VERDICT_LIST if r["verdict"].startswith(word)), None)
            for k in LISTS:
                if not r[k]:
                    continue
                if k != own:
                    print(f"      {HEADS[k].format(len(r[k]))}")
                for name in r[k]:
                    print(f"        - {name}")
            for o in r["other"]:
                print(f"      {o['head']}")
                for name in o["names"]:
                    print(f"        - {name}")
    if args.why:
        # Counted over the rows tri answered: a row with no verdict was not
        # compared, and "0 differ" over nothing compared would read as clean.
        judged = [r for r in rows if r["verdict"] is not None]
        differ = [r["key"] for r in judged if r["new_reason"]]
        print()
        print(f"REASONS: {len(differ)} of {len(judged)} pull request(s) with a verdict have a "
              f"failure that is red elsewhere for another reason"
              f"{': ' + ', '.join(differ) if differ else ''}")
        if len(judged) < len(rows):
            print(f"  {len(rows) - len(judged)} have no verdict (not open, or tri gave none): "
                  f"their reasons were not compared")
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
    if args.why:
        print("With --why: that the same failing-step text is the same cause.")
    else:
        print("Without --why: that a failure called pre-existing fails for the same")
        print("reason -- only its check's NAME was compared (pass --why).")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
