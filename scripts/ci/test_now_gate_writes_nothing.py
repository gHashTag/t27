#!/usr/bin/env python3
"""#5482: the NOW sync gate asks a question and writes nothing.

`scripts/ci/now-sync-gate-diff.sh` used to end, after a pass, by appending
`.claude/skills/ci-gates/SKILL.md merge=union` to .gitattributes and setting
merge.union.name/driver in the repository config. CI threw both away with the
runner. A contributor's clone kept them: `tri hooks pre-push` runs the same
script there (and `tri gates preview` did, until #5935), and the driver it set REPLACED
git's built-in `union` -- which `.trinity/experience/*.jsonl merge=union`
relies on -- with a command whose variables git never sets.

Each caller gets its own scratch repository whose range PASSES the gate,
because the writer ran only after `NOW sync gate passed`: a refused range
proves nothing. Its own, because the writer is idempotent -- in a shared
repository the first caller writes and every later one rewrites the same bytes
and reads as clean, which is what the first draft of this file reported for
four of its five callers against the unfixed script. In each one,
`.git/config`, `.gitattributes`, `.git/info/attributes` and `git status` must
be identical before and after, and a union merge must still work afterwards:

  * the script, on each arm that can reach a pass (pull_request; push; push
    from the all-zero sha);
  * `tri hooks pre-push`, given `--tri PATH`. Without a binary it is NOT RUN
    and the summary says so: this file runs in two workflows, and only cli-tri
    builds `tri`. (`tri gates preview` was a caller until its
    check-now-freshness row was removed with the NOW gate, #5935.)

Controls, so that this file can fail:

  * planted writer: the script under test with the removed lines appended must
    be caught changing .git/config and .gitattributes -- or the comparison is
    blind and every "unchanged" above is worthless;
  * the consequence, both ways: after each real caller, two branches appending
    to one `*.jsonl` file merge cleanly; after the planted writer, the same
    merge fails.

Exit 0 everything held, 1 a write or a failed control, 2 could not run.
"""

import argparse
import datetime
import os
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SCRIPT = REPO / "scripts/ci/now-sync-gate-diff.sh"
FAILURES = []

# The routing master's .gitattributes gives the experience logs. Written here
# rather than copied, so that the planted writer always finds its SKILL.md line
# absent and writes it: a control that leans on today's .gitattributes stops
# firing the day somebody commits that line.
ATTRIBUTES = ".trinity/experience/*.jsonl merge=union\n"

# The writer #4728 added: the same commands, taken out of their function.
# Appended to the script under test, they run where the call used to: after
# the pass.
PLANTED = r'''
# planted by scripts/ci/test_now_gate_writes_nothing.py
if ! grep -q "^.claude/skills/ci-gates/SKILL.md" .gitattributes 2>/dev/null; then
  echo ".claude/skills/ci-gates/SKILL.md merge=union" >> .gitattributes
fi
git config merge.union.name "union merge for appends"
git config merge.union.driver "bash -c 'cat \$BASE \$LOCAL \$RIGHT > \$REMOTE'"
'''

WATCHED = (".git/config", ".gitattributes", ".git/info/attributes")
LOG = ".trinity/experience/control.jsonl"


class CouldNotRun(Exception):
    pass


def check(name, ok, detail=""):
    print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
    if not ok:
        FAILURES.append(f"{name}: {detail}")


def isolated_env(tmp):
    """git with no global or system config, and nothing inherited that points
    git at another repository: a hook exports GIT_DIR, and a global
    merge.union.driver would decide the merge below instead of the gate."""
    env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    empty = tmp / "global.gitconfig"
    empty.write_text("")
    env.update({
        "GIT_CONFIG_GLOBAL": str(empty),
        "GIT_CONFIG_NOSYSTEM": "1",
        "GIT_AUTHOR_NAME": "control",
        "GIT_AUTHOR_EMAIL": "control@example.invalid",
        "GIT_COMMITTER_NAME": "control",
        "GIT_COMMITTER_EMAIL": "control@example.invalid",
        "GIT_TERMINAL_PROMPT": "0",
    })
    return env


def git(repo, *args, env, must=True):
    try:
        r = subprocess.run(["git", *args], cwd=repo, env=env, capture_output=True, text=True)
    except OSError as e:
        raise CouldNotRun(f"git could not be started: {e}")
    if must and r.returncode != 0:
        raise CouldNotRun(f"git {' '.join(args)} exited {r.returncode} in {repo}: {r.stderr.strip()}")
    return r


def commit_all(repo, msg, env):
    git(repo, "add", "-A", env=env)
    git(repo, "commit", "-q", "-m", msg, env=env)
    return git(repo, "rev-parse", "HEAD", env=env).stdout.strip()


def make_repo(root, script_text, env):
    """A repository whose base..HEAD range passes the gate: HEAD adds one entry,
    dated today in UTC, with a heading and a bullet."""
    root.mkdir(parents=True)
    git(root, "init", "-q", env=env)
    git(root, "symbolic-ref", "HEAD", "refs/heads/master", env=env)
    gate = root / "scripts/ci/now-sync-gate-diff.sh"
    gate.parent.mkdir(parents=True)
    gate.write_text(script_text)
    (root / ".gitattributes").write_text(ATTRIBUTES)
    log = root / LOG
    log.parent.mkdir(parents=True)
    log.write_text('{"n": 0}\n')
    base = commit_all(root, "base", env)
    today = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d")
    entry = root / f"docs/now/{today}-control.md"
    entry.parent.mkdir(parents=True)
    entry.write_text(f"# NOW -- control ({today})\n\n## control\n\n- an entry the gate accepts\n")
    head = commit_all(root, "entry", env)
    return base, head


def snapshot(repo, env):
    s = {}
    for rel in WATCHED:
        p = repo / rel
        s[rel] = p.read_bytes() if p.exists() else None
    st = git(repo, "status", "--porcelain", "--untracked-files=all", env=env).stdout
    s["git status"] = st.encode()
    return s


def describe(before, after):
    """What moved, readable: the names, and the lines each one gained."""
    parts = []
    for k in before:
        if before[k] == after[k]:
            continue
        old = (before[k] or b"").decode(errors="replace").splitlines()
        new = (after[k] or b"").decode(errors="replace").splitlines()
        gained = [l.strip() for l in new if l not in old]
        parts.append(f"{k} (+{gained})")
    return "; ".join(parts)


def run(argv, repo, env, extra=None):
    e = dict(env)
    e.update(extra or {})
    try:
        return subprocess.run(argv, cwd=repo, env=e, capture_output=True, text=True,
                               stdin=subprocess.DEVNULL, timeout=300)
    except (OSError, subprocess.TimeoutExpired) as ex:
        raise CouldNotRun(f"{argv[0]} could not be run: {ex}")


def gate_passed(r):
    return r.returncode == 0 and "NOW sync gate passed" in r.stdout


GATE = ["bash", "scripts/ci/now-sync-gate-diff.sh"]


def pull_request(base, head):
    return GATE, {"GITHUB_EVENT_NAME": "pull_request", "PR_BASE_SHA": base, "PR_HEAD_SHA": head}


def callers(tri):
    """(label, (base, head) -> (argv, extra env), did the run reach the gate's pass?)"""
    out = [
        ("script, pull_request arm", pull_request, gate_passed),
        ("script, push arm",
         lambda b, h: (GATE, {"GITHUB_EVENT_NAME": "push", "PUSH_BEFORE": b, "PUSH_AFTER": h}),
         gate_passed),
        ("script, push arm from the all-zero sha",
         lambda b, h: (GATE, {"GITHUB_EVENT_NAME": "push", "PUSH_BEFORE": "0" * 40, "PUSH_AFTER": h}),
         gate_passed),
    ]
    if tri:
        out.append(("tri hooks pre-push",
                    lambda b, h: ([tri, "hooks", "pre-push", "--base", b], {}),
                    lambda r: gate_passed(r) and "tri hooks pre-push: PASSED" in r.stdout))
    return out


def union_merge(repo, env):
    """Two branches each append one line to an experience log, then merge.

    Only the log is staged, never `-a`: the planted writer leaves
    .gitattributes modified, and committing that would change what is merged."""
    log = repo / LOG
    git(repo, "checkout", "-q", "-b", "side", env=env)
    with log.open("a") as f:
        f.write('{"n": "side"}\n')
    git(repo, "add", LOG, env=env)
    git(repo, "commit", "-q", "-m", "side", env=env)
    git(repo, "checkout", "-q", "master", env=env)
    with log.open("a") as f:
        f.write('{"n": "master"}\n')
    git(repo, "add", LOG, env=env)
    git(repo, "commit", "-q", "-m", "master", env=env)
    r = git(repo, "merge", "--no-edit", "side", env=env, must=False)
    text = log.read_text()
    kept = '{"n": "side"}' in text and '{"n": "master"}' in text and "<<<<<<<" not in text
    return r, kept


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--tri", help="a tri binary: also run `tri hooks pre-push`")
    args = ap.parse_args()

    if not SCRIPT.is_file():
        raise CouldNotRun(f"{SCRIPT} is missing")
    tri = None
    if args.tri:
        p = Path(args.tri).resolve()
        if not (p.is_file() and os.access(p, os.X_OK)):
            raise CouldNotRun(f"--tri {args.tri}: not an executable file")
        tri = str(p)
    script_text = SCRIPT.read_text()

    with tempfile.TemporaryDirectory(prefix="now-gate-writes-nothing-") as t:
        tmp = Path(t).resolve()
        env = isolated_env(tmp)

        print("The NOW gate and its callers, each in its own repository, on a range it passes")
        labels = []
        for i, (label, make, reached) in enumerate(callers(tri)):
            labels.append(label)
            repo = tmp / f"caller-{i}"
            base, head = make_repo(repo, script_text, env)
            if i == 0:
                # Without this routing the log takes git's text merge, where a
                # two-sided append conflicts whatever the config says -- and the
                # planted control's failed merge below would prove nothing.
                attr = git(repo, "check-attr", "merge", "--", LOG, env=env).stdout.strip()
                check("the scratch experience log is routed to `union`",
                      attr.endswith(": merge: union"), attr)
            argv, extra = make(base, head)
            before = snapshot(repo, env)
            r = run(argv, repo, env, extra)
            after = snapshot(repo, env)
            out = (r.stdout + r.stderr)[-600:]
            check(f"{label}: reached the gate's pass", reached(r),
                  f"exit {r.returncode}; a run that never passes never reached the writer; output={out!r}")
            check(f"{label}: .git/config, .gitattributes, .git/info/attributes and git status unchanged",
                  before == after, describe(before, after))
            merged, kept = union_merge(repo, env)
            check(f"{label}: a union merge of the experience log still succeeds afterwards",
                  merged.returncode == 0 and kept,
                  f"exit {merged.returncode}: {merged.stderr.strip()[:300]}")

        print("Control: a planted writer is seen, and breaks the same merge")
        planted = tmp / "planted"
        pbase, phead = make_repo(planted, script_text + PLANTED, env)
        argv, extra = pull_request(pbase, phead)
        before = snapshot(planted, env)
        r = run(argv, planted, env, extra)
        after = snapshot(planted, env)
        moved = [k for k in before if before[k] != after[k]]
        check("the planted writer's run reached the gate's pass", gate_passed(r),
              f"exit {r.returncode}; output={(r.stdout + r.stderr)[-600:]!r}")
        check("the comparison sees its write to .git/config", ".git/config" in moved,
              f"moved: {moved} -- a blind comparison makes every 'unchanged' above worthless")
        check("the comparison sees its write to .gitattributes", ".gitattributes" in moved,
              f"moved: {moved}")
        broken, _ = union_merge(planted, env)
        check("after the planted writer, the union merge fails", broken.returncode != 0,
              "it merged cleanly, so the merge checks above cannot tell the two drivers apart")
        if broken.returncode != 0:
            first = (broken.stderr.strip().splitlines() or [""])[0]
            print(f"            (git said: {first})")

    print()
    print(f"  scope: {len(labels)} callers, one repository each -- {'; '.join(labels)}")
    if not tri:
        print("  NOT RUN here: `tri hooks pre-push` (no --tri).")
        print("  cli-tri.yml runs it against the binary it builds.")
    print()
    if FAILURES:
        print("FAILED:")
        for f in FAILURES:
            print(f"  - {f}")
        return 1
    print("ok: no caller of the NOW gate wrote to the clone, and the planted writer was caught.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except CouldNotRun as e:
        print(f"could not run: {e}\nThe run stopped before its verdict. Exit 2 = could not run, not failed.")
        sys.exit(2)
