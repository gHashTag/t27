#!/usr/bin/env python3
"""tri tick -- the resume card for an autonomous cron loop: claim and its age, last ledger section, worktrees, anomalies.

WHY THIS EXISTS
---------------
A cron tick starts cold. What it knows about the previous tick is in
cron_tracking/<id>/tick-state.json and ledger.md (LOOP-RULES R0) and in the
worktrees the state names. Reading that by hand is five commands per worktree,
and the fact that matters most -- did the previous tick die mid-work? -- is a
join across two of them: a claim older than 45 minutes AND a dirty tree. Nobody
does that join by eye at the top of a tick. This prints it.

A claim the previous tick RELEASED is not held, however old: the tick ended.
And a file the loop keeps untracked on purpose -- the cron_tracking directory
itself, or a worktree's `keep_untracked` paths -- is not work in progress. Both
used to count (2026-10-04, tick 18 of cron 8782e5f8): a finished tick's claim
turned "dead" after 45 minutes on a worktree whose only untracked files were
cron_tracking/ and .claude/launch.json, and the printed fix was `git add -A &&
git commit`, which would have committed the loop's state into a pull request's
branch. The fix now names the files it would add.

WHAT IS READ
------------
  tick-state.json   cron_id, topic, tick, claim {item, since, released},
                    worktrees {name: {path, branch, base, keep_untracked}},
                    prs, queue, done, rules
  ledger.md         the last `## ` section, and the tick number in its heading
  each worktree     exists, current branch, dirty files (every untracked file
                    listed, minus the cron_tracking directory and the
                    worktree's keep_untracked paths), ahead/behind against
                    its base (local refs only), last commit age

ANOMALIES, EACH WITH A ONE-LINE FIX
-----------------------------------
  stale-claim-dirty     a claim not released, older than --stale-minutes
                        (45), and a dirty tree: the previous tick died
                        mid-work. The fix names each file to add, never -A
  orphaned-dirty        NO claim held (released, or none), yet a worktree is
                        dirty, its newest dirty file is older than
                        --stale-minutes, and no process has its cwd inside
                        it: whoever was writing there -- usually a delegated
                        background agent, named by the worktree's `held_by`
                        -- is gone and left the work uncommitted. Fresh
                        writes or a live process are a DELEGATE line on the
                        card, not an anomaly. Before this, a released claim
                        made every dirty tree silent (2026-10-04, tick 22 of
                        cron 8782e5f8: two dead agents' work, ANOMALIES 0)
  worktree-missing      a listed path is absent or is not a git worktree
  branch-mismatch       the worktree is on another branch than the state says
  base-unknown          the base ref does not resolve locally, or is not given
  state-unparseable     tick-state.json is missing or is not a JSON object
  claim-unparseable     claim.since (or a non-null claim.released) is not
                        ISO 8601, so staleness is undecided
  claim-in-future       claim.since is more than 2 minutes ahead of this
                        clock: a hand-typed time, and until it passes the
                        stale-claim check above cannot fire (first seen on the
                        real state this was written against, 2026-10-04)
  ledger-tick-mismatch  the ledger's last `## ` tick number is not the state's
                        tick (also: ledger missing, or no number in the heading)

The fixes are SUGGESTIONS for the tick to run. This never writes -- not the
state, not the ledger, not a worktree, not a ref -- and git runs with
GIT_OPTIONAL_LOCKS=0, so `git status` does not even refresh an index.

WHAT THIS DOES NOT ESTABLISH
----------------------------
  * That ahead/behind is current: it is against local refs as of the last
    fetch, and nothing is fetched.
  * Whose edits the dirty files are. A second session standing in the same
    worktree gives the same count (LOOP-RULES R17).
  * That a process standing in a worktree is its holder, or that none is
    writing there: a process whose cwd is elsewhere can edit the files by an
    absolute path, and lsof sees only this machine. Without lsof the process
    check is "not run" and the card says so.
  * That a tick whose ledger section exists reached its outcome (R0). The
    heading says a section was written, not what it closed.
  * Whether the listed pull requests are still open; they are not queried
    here. `tri pr-state` asks, with `tri pr ready`'s verdict for each.
  * Claim age beyond this machine's clock against the state's own text.

    tri tick                        # the one directory under cron_tracking/
    tri tick --id 8782e5f8
    tri tick --dir /path/to/cron_tracking --json

Exit codes: 0 no anomalies; 1 at least one anomaly; 2 usage (no cron_tracking
directory, several ids and no --id, an unknown id).
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import subprocess
import sys
import time
from datetime import datetime, timezone

ENV = dict(os.environ, GIT_OPTIONAL_LOCKS="0", GIT_TERMINAL_PROMPT="0")
TICK_IN_HEADING = re.compile(r"\btick\s+#?(\d+)", re.I)
FUTURE_SLACK_S = 120  # clock drift between machines is not an anomaly
FIX_PATHS_SHOWN = 8   # a longer list goes to --json's dirty_paths


def git(path: str, *args: str) -> tuple[int, str]:
    r = subprocess.run(["git", "-C", path, *args], capture_output=True,
                       text=True, errors="replace", env=ENV)
    return r.returncode, r.stdout


def usage(msg: str, prog: str = "tri tick") -> int:
    print(f"{prog}: {msg}", file=sys.stderr)
    return 2


def default_dir() -> str:
    rc, top = git(".", "rev-parse", "--show-toplevel")
    return os.path.join(top.strip() if rc == 0 else os.getcwd(), "cron_tracking")


def load_state(d: str) -> tuple[dict | None, str | None]:
    p = os.path.join(d, "tick-state.json")
    try:
        with open(p) as fh:
            st = json.load(fh)
    except FileNotFoundError:
        return None, f"{p} does not exist"
    except (OSError, ValueError) as exc:
        return None, f"{p}: {exc}"
    if not isinstance(st, dict):
        return None, f"{p}: the top level is not a JSON object"
    return st, None


def last_section(d: str) -> tuple[list[str] | None, int | None]:
    """(lines of the last `## ` section, tick number in its heading)."""
    try:
        with open(os.path.join(d, "ledger.md"), errors="replace") as fh:
            lines = fh.read().split("\n")
    except OSError:
        return None, None
    heads = [i for i, l in enumerate(lines) if l.startswith("## ")]
    if not heads:
        return [], None
    sec = lines[heads[-1]:]
    while sec and not sec[-1].strip():
        sec.pop()
    m = TICK_IN_HEADING.search(sec[0])
    return sec, int(m.group(1)) if m else None


def parse_since(text) -> tuple[datetime | None, bool]:
    """(aware datetime or None, whether a timezone was given)."""
    if not isinstance(text, str) or not text.strip():
        return None, False
    t = text.strip()
    if t.endswith("Z"):
        t = t[:-1] + "+00:00"
    try:
        dt = datetime.fromisoformat(t)
    except ValueError:
        return None, False
    if dt.tzinfo is None:
        return dt.astimezone(), False
    return dt, True


def span(seconds: float) -> str:
    m = int(seconds // 60)
    if m < 60:
        return f"{m}m"
    if m < 48 * 60:
        return f"{m // 60}h {m % 60:02d}m"
    return f"{m / 1440:.1f}d"


def status_paths(porcelain_z: str) -> list[tuple[str, str]]:
    """(XY, path) per entry of `git status --porcelain=v1 -z`. A rename or
    copy carries its source as the next NUL field; it is skipped, the new path
    is the one to add."""
    out, fields, i = [], porcelain_z.split("\0"), 0
    while i < len(fields):
        f = fields[i]
        i += 1
        if len(f) < 4:
            continue
        xy, path = f[:2], f[3:]
        out.append((xy, path))
        if xy[0] in "RC":
            i += 1
    return out


def is_kept(path: str, kept: list[str]) -> bool:
    """Whether an untracked path is one the loop keeps untracked on purpose."""
    p = path.rstrip("/")
    return any(p == k or p.startswith(k + "/") for k in kept)


def inspect_worktree(name: str, spec, now: float, state_dir: str,
                     procs: list[tuple[int, str]] | None = None) -> dict:
    spec = spec if isinstance(spec, dict) else {}
    keep = spec.get("keep_untracked")
    keep = [k.strip("/") for k in keep if isinstance(k, str) and k.strip("/")] \
        if isinstance(keep, list) else []
    row = {"name": name, "path": spec.get("path"), "branch": spec.get("branch"),
           "base": spec.get("base"), "exists": False, "current": None,
           "dirty": None, "dirty_paths": None, "kept_untracked": None,
           "ahead": None, "behind": None, "base_resolves": None,
           "last_commit_age_s": None, "last_subject": None, "note": None,
           "held_by": spec.get("held_by"), "newest_write_age_s": None,
           "processes": None}
    path = row["path"]
    if not isinstance(path, str) or not os.path.isdir(path):
        row["note"] = "path does not exist" if path else "no path in the state"
        return row
    rc, top = git(path, "rev-parse", "--show-toplevel")
    if rc != 0:
        row["note"] = "directory exists but is not a git worktree"
        return row
    row["exists"] = True
    rc, cur = git(path, "symbolic-ref", "--short", "-q", "HEAD")
    row["current"] = cur.strip() if rc == 0 and cur.strip() else "(detached HEAD)"
    # The state directory is untracked by rule wherever it sits; a worktree
    # that holds it would otherwise read as dirty on every tick.
    rel = os.path.relpath(os.path.realpath(state_dir), os.path.realpath(top.strip()))
    if rel != "." and not rel.startswith(".."):
        keep = keep + [rel.replace(os.sep, "/")]
    # Every untracked file, not the directory that holds it: `?? .claude/` would
    # hide whether anything in it is beside the kept path.
    rc, st = git(path, "status", "--porcelain=v1", "-z", "--untracked-files=all")
    if rc == 0:
        entries = status_paths(st)
        kept = [p for xy, p in entries if xy == "??" and is_kept(p, keep)]
        row["dirty_paths"] = [p for xy, p in entries if not (xy == "??" and is_kept(p, keep))]
        row["dirty"] = len(row["dirty_paths"])
        row["kept_untracked"] = len(kept)
        row["newest_write_age_s"] = newest_write_age(path, row["dirty_paths"], now)
    row["processes"] = processes_in(path, procs)
    rc, log = git(path, "log", "-1", "--format=%ct%x00%s", "HEAD")
    if rc == 0 and "\x00" in log:
        ts, subject = log.strip("\n").split("\x00", 1)
        row["last_commit_age_s"] = now - int(ts)
        row["last_subject"] = subject
    base = row["base"]
    if isinstance(base, str) and base:
        rc, _ = git(path, "rev-parse", "--verify", "-q", f"{base}^{{commit}}")
        row["base_resolves"] = rc == 0
        if rc == 0:
            rc, c = git(path, "rev-list", "--left-right", "--count", f"{base}...HEAD")
            if rc == 0 and len(c.split()) == 2:
                row["behind"], row["ahead"] = (int(x) for x in c.split())
    return row


def cwd_processes() -> list[tuple[int, str]] | None:
    """(pid, real cwd) of every process lsof can see, or None when it cannot
    run. This process is left out: it stands wherever it was started."""
    try:
        r = subprocess.run(["lsof", "-w", "-d", "cwd", "-Fpn"], capture_output=True,
                           text=True, errors="replace", timeout=20)
    except (OSError, subprocess.TimeoutExpired):
        return None
    if r.returncode not in (0, 1) or not r.stdout:
        return None
    out, pid = [], None
    for line in r.stdout.splitlines():
        if line.startswith("p") and line[1:].isdigit():
            pid = int(line[1:])
        elif line.startswith("n") and pid is not None and pid != os.getpid():
            out.append((pid, os.path.realpath(line[1:])))
    return out


def processes_in(path: str, procs: list[tuple[int, str]] | None) -> list[int] | None:
    if procs is None:
        return None
    top = os.path.realpath(path)
    return sorted({pid for pid, cwd in procs if cwd == top or cwd.startswith(top + os.sep)})


def newest_write_age(path: str, paths: list[str], now: float) -> float | None:
    """Seconds since the newest dirty file was written; None when none of them
    is on disk (deletions only)."""
    ages = []
    for rel in paths:
        try:
            ages.append(now - os.lstat(os.path.join(path, rel)).st_mtime)
        except OSError:
            continue
    return min(ages) if ages else None


def recover_fix(path: str, paths: list[str], tick) -> str:
    """The commit that keeps a dead tick's work, naming each file: `add -A`
    would also take what the loop keeps untracked on purpose."""
    shown = " ".join(shlex.quote(p) for p in paths[:FIX_PATHS_SHOWN])
    more = len(paths) - FIX_PATHS_SHOWN
    add = f"git -C {shlex.quote(path)} add -- {shown}"
    if more > 0:
        add += f" <and the {more} more in --json worktrees[].dirty_paths>"
    return (f"{add} && git -C {shlex.quote(path)} commit -m "
            f"'wip(tick {tick}): recovered'  (read the list first: a second "
            f"session's edits look the same)")


def anomalies_for(state, state_err, since_text, since, tick, ledger_tick, ledger,
                  rows, claim_age_s, stale_minutes, released_text=None,
                  released=None, is_released=False, held=None) -> list[dict]:
    out = []

    def add(code, subject, detail, fix):
        out.append({"code": code, "subject": subject, "detail": detail, "fix": fix})

    if state_err:
        add("state-unparseable", "tick-state.json", state_err,
            "rebuild tick-state.json by hand from the ledger's last section; "
            "the ledger is append-only and is the record")
        return out
    if since_text is not None and since is None:
        add("claim-unparseable", "claim.since", f"cannot read {since_text!r} as ISO 8601",
            "write claim.since with an offset, e.g. 2026-10-04T00:20+07:00")
    if released_text is not None and released is None:
        add("claim-unparseable", "claim.released",
            f"cannot read {released_text!r} as ISO 8601",
            "write claim.released from the clock when the tick ends (date -Iseconds), "
            "or null while it is held")
    if claim_age_s is not None and claim_age_s < -FUTURE_SLACK_S:
        add("claim-in-future", "claim.since",
            f"{since_text} is {span(-claim_age_s)} ahead of this machine's clock",
            "write claim.since from the clock, not by hand: date -Iseconds")
    if ledger is None:
        add("ledger-tick-mismatch", "ledger.md", "ledger.md does not exist",
            "create ledger.md and append the section for the tick in the state "
            "(`## tick N -- <date>`)")
    elif ledger_tick is None:
        add("ledger-tick-mismatch", "ledger.md",
            "the last `## ` heading carries no tick number" if ledger else
            "the ledger has no `## ` section",
            "append `## tick N -- <date>` with the outcome of the tick in the state")
    elif ledger_tick != tick:
        add("ledger-tick-mismatch", "ledger.md",
            f"ledger's last section is tick {ledger_tick}, state says tick {tick}",
            "the previous tick did not write its section, or `tick` was bumped "
            "twice: append the missing section, or correct `tick` in the state")
    stale = (not is_released and claim_age_s is not None
             and claim_age_s > stale_minutes * 60)
    if held is None:
        held = not is_released
    for r in rows:
        who = f"worktrees.{r['name']}"
        if not r["exists"]:
            add("worktree-missing", who, f"{r['path']}: {r['note']}",
                f"git worktree add {r['path']} {r['branch']} from the clone that owns "
                f"the branch, or drop {who} from the state if its work landed")
            continue
        if r["branch"] and r["current"] != r["branch"]:
            add("branch-mismatch", who,
                f"on {r['current']}, the state says {r['branch']}",
                f"commit any WIP first, then git -C {r['path']} switch {r['branch']} "
                f"(never reset); if the switch was deliberate, update {who}.branch")
        if not r["base"]:
            add("base-unknown", who, "no base in the state",
                f"add {who}.base (e.g. origin/main)")
        elif r["base_resolves"] is False:
            add("base-unknown", who, f"{r['base']} does not resolve in {r['path']}",
                f"git -C {r['path']} fetch origin (a fetch is yours to run), "
                f"or correct {who}.base")
        if stale and r["dirty"]:
            add("stale-claim-dirty", who,
                f"claim is {span(claim_age_s)} old (> {stale_minutes}m), not released, "
                f"and {r['dirty']} file(s) are dirty: the previous tick died mid-work",
                recover_fix(r["path"], r["dirty_paths"] or [], tick))
        if not held and r["dirty"] and orphaned(r, stale_minutes):
            age = r["newest_write_age_s"]
            wrote = (f"newest dirty write {span(age)} ago" if age is not None
                     else "no dirty file is on disk (deletions only)")
            procs = ("no process stands in it" if r["processes"] is not None
                     else "process check NOT RUN (no lsof)")
            holder = (f"held_by says {str(r['held_by'])[:120]!r}, but "
                      if r["held_by"] else "")
            add("orphaned-dirty", who,
                f"no claim is held, {r['dirty']} file(s) are dirty, {wrote}, "
                f"{procs}: {holder}whoever was writing here is gone",
                f"read the diff (git -C {shlex.quote(r['path'])} diff), then keep the "
                f"work: {recover_fix(r['path'], r['dirty_paths'] or [], tick)}; "
                f"clear {who}.held_by")
    return out


def orphaned(r: dict, stale_minutes: float) -> bool:
    """A dirty tree nobody is writing in: no fresh write, no process inside."""
    age = r.get("newest_write_age_s")
    if age is not None and age <= stale_minutes * 60:
        return False
    return not r.get("processes")


def pick_id(root: str, want: str | None, prog: str = "tri tick") -> tuple[str | None, int]:
    if not os.path.isdir(root):
        return None, usage(f"no cron_tracking directory at {root} (give --dir)", prog)
    ids = sorted(d for d in os.listdir(root) if os.path.isdir(os.path.join(root, d)))
    if want:
        if want not in ids:
            return None, usage(f"no cron id {want!r} under {root}; present: "
                               f"{', '.join(ids) or 'none'}", prog)
        return want, 0
    if len(ids) == 1:
        return ids[0], 0
    if not ids:
        return None, usage(f"{root} holds no cron directories", prog)
    print(f"{prog}: {len(ids)} cron ids under {root}; pick one with --id:", file=sys.stderr)
    for i in ids:
        st, _ = load_state(os.path.join(root, i))
        topic = (st or {}).get("topic", "(state unreadable)")
        tick = (st or {}).get("tick", "?")
        print(f"  {i:<12} tick {tick:<4} {str(topic)[:70]}", file=sys.stderr)
    return None, 2


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(prog="tri tick", description=__doc__.split("\n")[0])
    ap.add_argument("--id", help="cron id (a directory under cron_tracking/)")
    ap.add_argument("--dir", help="the cron_tracking directory "
                                  "(default: <repository top>/cron_tracking)")
    ap.add_argument("--stale-minutes", type=float, default=45.0,
                    help="claim age past which a dirty tree means a dead tick (default 45)")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args(argv)

    root = os.path.abspath(args.dir or default_dir())
    cid, code = pick_id(root, args.id)
    if cid is None:
        return code
    d = os.path.join(root, cid)
    now = time.time()

    state, state_err = load_state(d)
    st = state or {}
    tick = st.get("tick")
    claim = st.get("claim") if isinstance(st.get("claim"), dict) else {}
    since_text = claim.get("since")
    since, has_tz = parse_since(since_text)
    claim_age_s = (now - since.timestamp()) if since else None
    released_text = claim.get("released")
    released, _ = parse_since(released_text)
    # A release older than the claim is a leftover from an earlier tick: the
    # claim was re-taken without clearing it, and is held.
    is_released = released is not None and (
        since is None or released.timestamp() >= since.timestamp() - FUTURE_SLACK_S)
    ledger, ledger_tick = last_section(d)
    wts = st.get("worktrees") if isinstance(st.get("worktrees"), dict) else {}
    held = bool(claim.get("item")) and not is_released
    procs = cwd_processes() if wts else []
    rows = [inspect_worktree(n, s, now, root, procs) for n, s in wts.items()]
    found = anomalies_for(state, state_err, since_text, since, tick, ledger_tick,
                          ledger, rows, claim_age_s, args.stale_minutes,
                          released_text, released, is_released, held)

    if args.json:
        print(json.dumps({
            "cron_id": cid, "dir": d, "state_error": state_err,
            "topic": st.get("topic"), "tick": tick, "ledger_tick": ledger_tick,
            "claim": {"item": claim.get("item"), "since": since_text,
                      "age_seconds": round(claim_age_s) if claim_age_s is not None else None,
                      "timezone_given": has_tz, "released": released_text,
                      "held": held},
            "ledger_last_section": ledger, "worktrees": rows,
            "prs": st.get("prs"), "queue": st.get("queue"), "done": st.get("done"),
            "rules": st.get("rules"), "anomalies": found}, indent=1))
        return 1 if found else 0

    print(f"tri tick -- cron {cid}   ({d})")
    if state_err:
        print(f"  STATE     UNREADABLE: {state_err}")
    print(f"  topic     {st.get('topic', '(none)')}")
    print(f"  tick      {tick}   (ledger's last section: "
          f"{'tick ' + str(ledger_tick) if ledger_tick is not None else 'no tick number'})")
    if claim.get("item"):
        if claim_age_s is None:
            age = "age unknown"
        elif claim_age_s < 0:
            age = f"{span(-claim_age_s)} IN THE FUTURE"
        else:
            age = span(claim_age_s) + " ago"
        tz = "" if has_tz or since is None else "  (no offset: read as local time)"
        if is_released:
            print(f"  claim     released {released_text}  (none held)\n"
                  f"            last: {claim.get('item')}  since {since_text}")
        else:
            print(f"  claim     {claim.get('item')}  since {since_text}  ({age}){tz}")
    else:
        print("  claim     none held")
    prs = st.get("prs") if isinstance(st.get("prs"), dict) else {}
    print(f"  prs       {len(prs)}" + "".join(f"\n            {k}  {v}" for k, v in prs.items()))
    queue = st.get("queue") if isinstance(st.get("queue"), list) else []
    done = st.get("done") if isinstance(st.get("done"), list) else []
    print(f"  queue     {len(queue)} item(s)" + "".join(f"\n            - {str(q)[:90]}" for q in queue[:5])
          + (f"\n            ... {len(queue) - 5} more" if len(queue) > 5 else ""))
    print(f"  done      {len(done)} item(s)")
    print()
    print("LAST LEDGER SECTION (ledger.md)")
    if ledger is None:
        print("  (ledger.md does not exist)")
    elif not ledger:
        print("  (no `## ` section)")
    for line in ledger or []:
        print(f"  {line}")
    print()
    print("WORKTREES")
    if not rows:
        print("  (none listed in the state)")
    for r in rows:
        print(f"  {r['name']:<10} {r['path']}")
        if not r["exists"]:
            print(f"             MISSING: {r['note']}")
            continue
        match = "matches" if r["current"] == r["branch"] else f"state says {r['branch']}"
        kept = f", {r['kept_untracked']} kept untracked" if r["kept_untracked"] else ""
        print(f"             branch {r['current']} ({match}), dirty {r['dirty']}{kept}")
        if r["base_resolves"]:
            print(f"             vs {r['base']}: ahead {r['ahead']}, behind {r['behind']} (local refs)")
        else:
            print(f"             vs {r['base'] or '(no base)'}: does not resolve")
        if r["dirty"]:
            age = r["newest_write_age_s"]
            ps = r["processes"]
            print(f"             newest dirty write "
                  f"{span(age) + ' ago' if age is not None else 'none on disk'}; "
                  f"processes here: {'NOT RUN' if ps is None else len(ps)}"
                  + (f"; held_by: {str(r['held_by'])[:70]}" if r["held_by"] else ""))
            if not held and not orphaned(r, args.stale_minutes):
                print("             DELEGATE: no claim held, but this is being written "
                      "now -- leave it to its holder")
        if r["last_commit_age_s"] is not None:
            print(f"             last commit {span(r['last_commit_age_s'])} ago: "
                  f"{(r['last_subject'] or '')[:70]}")
    rules = st.get("rules") if isinstance(st.get("rules"), list) else []
    if rules:
        print()
        print("RULES (from the state)")
        for rule in rules:
            print(f"  - {rule}")
    print()
    print(f"ANOMALIES: {len(found)}")
    for a in found:
        print(f"  [{a['code']}] {a['subject']}: {a['detail']}")
        print(f"      fix: {a['fix']}")
    print()
    print("NOT ESTABLISHED: ahead/behind is against local refs (nothing fetched);")
    print("whose edits the dirty files are (a second session in the same worktree")
    print("reads the same); that a ledger section means its tick reached an outcome;")
    print("whether the listed PRs are still open (not queried here: `tri pr-state`).")
    print("Nothing was written.")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
