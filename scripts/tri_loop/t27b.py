#!/usr/bin/env python3
"""tri t27b -- the t27b steward's tick card (status), anomaly scan (doctor), per-spec ratchet between lab runs (delta), gen-c proof on the t27c lab (gen-check), the merge gate (ready), the stack merger (watch), the reference-backed lane picker (next), our own specs first (dogfood), the repro reducer (reduce) and the differential fuzzer (fuzz).

WHY THIS EXISTS
---------------
The t27b coverage loop (epic #6063: an hourly steward cron plus lane
subagents) reads its state from seven places at the top of every tick. Read by
hand on 2026-10-04, that state produced four slips in one afternoon (#6112):

  * A tick saw /tmp/t27b-unconflict mid-merge and decided the subagent that
    owned it had died. That subagent was alive. A second subagent was sent into
    the same path, and the first one's uncommitted merge was reverted under it.
    A mid-merge worktree is not a dead one; the processes whose cwd is inside
    it say which.
  * `railway whoami` said Unauthorized after the owner had logged in: the
    first `railway` on PATH was 4.5.4, and the login is read only by 5.x.
  * The lab's numbers were quoted as the project's while its `ref` was a
    branch, not master.
  * A local three-hour reference run kept a Mac at load 700 while the lab
    produced the same denominator in 321 s.

And one slip this tool made itself (#6184): `status` printed `t27b 70 /
reference 648` and the steward posted it. 14 of those 70 passes were specs the
reference fails; the honest figure was 56 of 648. Of those 56, 29 ran no test
and no invariant -- they compiled and checked nothing. The card now prints the
in-reference ratio, splits it into checked and compile-only, and keeps the
out-of-reference passes on their own line.

WHAT IS READ (nothing is written, fetched or pushed)
----------------------------------------------------
  lab        GET <lab>/latest.json                       (fixture: lab.json)
  labstatus  GET <lab>/status.json: the running image    (status.json)
  labfiles   gh api contents/contrib/railway/t27b-lab at
             master: each file's git blob sha            (master_lab.json)
  master     git ls-remote origin refs/heads/master      (master.txt)
  prs        gh pr list --search t27b, open, with checks  (prs.json)
  claim      ~/.local/state/t27b-queen/claim.json         (claim.json)
  pids       kill(pid, 0) for the claim's pid             (alive_pids.txt)
  ledger     the last `| 20..` row of ledger.md           (ledger.md)
  worktrees  every /tmp/t27b-* that is a git worktree     (worktrees.txt)
  cwd        lsof -d cwd -Fpn: which process sits where   (cwd.txt)
  ps         ps -axo pid=,etime=,command=                 (ps.txt)
  railway    `railway --version` of the first on PATH     (railway.txt)
  runs       GET <lab>/runs/ and <lab>/runs/<sha>.json    (runs/<sha>.json)
             -- delta only
  now        the clock                                    (now.txt)

--fixture DIR reads every source from DIR instead (worktrees.txt lists real
paths, which are still inspected with git). A missing fixture file is a source
that could not be read, the same as a failed command -- never an empty answer.

ANOMALY CODES (doctor exits 1 when any is printed, 0 when none, 2 on usage)
---------------------------------------------------------------------------
  LAB-UNREADABLE      latest.json could not be read or parsed
  LAB-NOT-MASTER      the lab built a ref other than master
  LAB-BEHIND          the lab's commit is not the master tip
  LAB-STALE           the lab's run finished (or, failed, was last updated)
                      more than --stale-hours ago
  LAB-CHECKOUT        the lab could not check out its commit: the run measured
                      nothing; the lab re-clones once (#6220), so a repeat
                      means the deploy predates #6220 or the remote is broken
  LAB-RECLONED        the lab healed a broken clone with a fresh one: the run
                      counts, but say so in the report (skill rule Q18)
  LAB-IMAGE-STALE     the running lab's lab.py or Dockerfile (git blob sha,
                      `image` in status.json, else latest.json) is not
                      master's contrib/railway/t27b-lab, or the lab does not
                      say (an image from before #6443): redeploy from source
  LAB-MISMATCH        jit_interp_mismatch (alias mismatch) > 0: t27b's JIT
                      against t27b's own interpreter, same IR; a stop and a
                      report, never a skip
  LAB-REF-DISAGREE    reference_disagree > 0: t27b against the reference
                      (t27c + zig), test by test (#6441); stops the lanes the
                      same way, one issue per root cause
  LAB-OUTSIDE-REF     t27b passes where the reference does not: a reference
                      defect or a vacuous pass; either way not in the number
  LAB-ERROR           lab_error, crash, timeout or t27b fail > 0
  LAB-TESTS-RED       a lab step other than checkout reported ok=false, or
                      cargo test failed
  LAB-RATCHET         the lab's ratchet step is red: a spec moved against
                      docs/reports/t27b_expectations.json (see RATCHET below)
  PR-CONFLICTING      an open t27b PR cannot merge into its base
  PR-REQUIRED-RED     validate / check-linked-issue / parse-ratchet failed
  PR-BASE-NOT-MASTER  stacked PR: retarget after its parent merges
  CLAIM-DEAD          a claim is held by a pid that is not running
  CLAIM-OLD           a claim is held by a live pid past --claim-minutes
  WORKTREE-MIDOP      a /tmp/t27b-* worktree is mid-merge or mid-rebase
  WORKTREE-DIRTY      a /tmp/t27b-* worktree has uncommitted changes
  RAILWAY-OLD-CLI     the first railway on PATH is older than 5.x
  LOCAL-REFERENCE     a local `t27b corpus ... --reference` run is going
                      while the lab answers
  LEDGER-QUIET        the last ledger row is older than --quiet-hours
  LAB-FRONTEND-DISAGREES  t27b's frontend rejects a spec the reference
                      passes: a disagreement `mismatch` does not count

DELTA (tri t27b delta [--from SHA] [--to SHA]; exit 1 on the first three)
---------------------------------------------------------------------------
Per spec, over the specs the reference passes in BOTH runs:
  REGRESSED     t27b passed in --from and does not in --to
  NEW-MISMATCH  t27b fails (or crashes) where it did not, reference passing
  CHECK-LOST    a pass that ran tests/invariants now runs none
  GAINED        t27b passes now and did not before          (reported)
  REF-MOVED     the reference's own verdict changed          (reported: the
                denominator moved, not t27b)
--to defaults to latest.json; --from to the run before it by `finished`.

WHAT THIS DOES NOT ESTABLISH
----------------------------
  * That a worktree with a process inside is healthy. Only that something is
    there; a WORKTREE-MIDOP with owners is someone's work, not debris.
  * That a worktree with no process inside is abandoned. A subagent between
    two commands has no shell open. Read the age before touching it.
  * That the lab's numbers are right. Only that they are master's, fresh, and
    free of the failure classes above.
  * Anything about PRs that do not match the search `t27b`.
  * That a "checked" pass checks much: one test block counts. Compile-only
    means 0 tests and 0 invariants as the lab reports them.

    tri t27b status                # the tick card
    tri t27b doctor                # anomalies, one per line, with the fix
    tri t27b doctor --json         # the same, machine-readable
    tri t27b delta                 # what moved per spec since the previous lab run
    tri t27b gen-check             # is gen/c/tri/t27b/steward.c what master's t27c emits?
    tri t27b diff SPEC [--run R]   # t27b's per-test verdicts beside the reference's (#6441)
    tri t27b ready [N ...]         # may each open t27b PR merge now? (never merges)
    tri t27b reduce SPEC [--family X | --disagree]  # shrink SPEC to a minimal repro (#6445)
    tri t27b fuzz --cases N --seed S  # generated programs, t27b against the reference (#6442)

REDUCE (#6445): a minimal repro, reference-guarded, decided in reduce.t27
-------------------------------------------------------------------------
ddmin over declarations, then statements, then expressions, until a cycle
removes nothing. Every candidate must parse (`t27c parse`); one is kept only
when the reference passes it and t27b's first blocker is still the family
(--family, default the spec's own) or t27b still disagrees with it
(--disagree). A spec the reference does not pass is refused with
REFERENCE-NOT-PASS: it is a reference bug, not a lane (Q38). The fixture lands
in cli/t27b/tests/reduced/ with a header naming its origin, and `next` links it
under its lane. The reference runs through `t27b corpus --reference
--reference-cache` (default ~/.cache/t27/t27b-reduce/refcache.tsv), so a rerun
hits the cache. scripts/tri_loop/t27b_reduce.py; --help lists the flags.

READY (#6244): the steward's merge gate, decided in steward.t27
---------------------------------------------------------------
One line per PR: READY, WAIT, RED, CONFLICT, RETARGET, BLOCKED or CLOSED, then the
checks that decided it. Required: validate, check-linked-issue,
parse-ratchet, and loop-tools-tracked, t27b-native-linux and t27b-native-macos
(#6444) when they report on the PR; each must be SUCCESS. t27b-native-ratchet
(the native corpus against the ledger) is not required and falls under Q16. A non-required check blocks only when it is red on the PR and its
latest completed result on master is success (Q16); red on master too means
master broke it. Master's results are the newest completed run per check
name over the last 12 master commits, looked up past a tip whose run is still
going (#6334): red below a running tip stays red, reported as
`coverage=FAILURE (master FAILURE@f329e27c1, newer run going)`; green below
it, or no completed run in the window, makes the PR WAIT, never READY
(steward.t27 `master_state`). Exit 0 when every PR listed is READY,
1 otherwise, 2 when GitHub could not be read. Fixtures: prs.json,
master_checks.json ({name: state} or {name: {last, sha, running}}).

    tri t27b ready [N ...] [--json] [--fixture DIR]

GEN-CHECK (#6231): the committed gen-c output against the t27c lab
-------------------------------------------------------------------
gen/c/tri/t27b/steward.c is generated (L2) by `t27c gen-c
specs/tri/t27b/steward.t27`, and t27c runs on the Railway lab t27c-lab, never
on this machine. gen-check sends the LOCAL spec to the lab (base64 inside
`railway ssh`, chunked under the 128 KiB per-argument cap), runs the lab's
master t27c there in a scratch dir under /tmp that it removes, and compares
the sha256 and length it gets back with the local gen file:

  SAME <sha12>                        exit 0
  DIFFERS lab <sha12> local <sha12>   exit 1
  UNREACHABLE <reason>                exit 2 -- ssh failed, the lab answered
                                      only in part, or gen-c itself failed

and a second line with the lab binary's path, mtime and sha, and the commit
of the lab's source checkout (STALE? when the binary is older than it).
Overrides: T27C_LAB_RAILWAY (the railway 5.x CLI), T27C_LAB_DIR (a dir linked
to the lab's Railway project), T27C_LAB_SERVICE, T27C_LAB_ENV, T27C_LAB_BIN,
T27C_LAB_SRC; T27B_GEN_CHECK_FAKE (a JSON fixture standing in for the lab,
tests only).

    tri t27b gen-check [--spec specs/tri/t27b/steward.t27] [--gen gen/c/tri/t27b/steward.c] [--json]

RATCHET (#6115): the per-spec ledger, so the t27b number cannot move silently
------------------------------------------------------------------------------
A total ("t27b 56 / reference 648") cannot show a regression: one spec that
breaks while another starts passing leaves it unchanged. The ledger
docs/reports/t27b_expectations.json names, for EVERY spec the reference path
passes, what t27b does with it: `pass`, `pass_vacuous` (passes, but its tests
executed 0 runtime asserts), or its first blocker with a reason
(`unimplemented` | `reference-bug` | `n/a`). `ratchet` diffs one lab run
(latest.json shape) against it:

  UNEXPECTED FAILURE  a ledger pass that is not a pass now (a pass that turned
                      vacuous included); a ledger pass_vacuous that no longer
                      passes at all
  UNEXPECTED PASS     a ledger non-pass that passes now; a pass_vacuous that
                      is a real pass now. Red too: an improvement nobody
                      blesses leaves slack for the next regression
  UNLISTED            the reference passes a spec the ledger does not name
  STALE               a ledger spec the reference now fails, or that is gone
  OVER CAP            more non-pass entries than `max_not_pass`, which only
                      moves down (raising it is a hand edit in the PR)
  BAD REASON          a non-pass entry whose reason is not one of the three
  -- not red --
  MOVED               still not a pass, but the first blocker changed
  UNJUDGED            the reference could not judge the spec in this run
                      (lab_error, timeout, skip): no verdict either way
  VACUITY MEASURED    a ledger pass blessed from a run that did not count
                      asserts (source.asserts_counted false) is pass_vacuous
                      in a run that does: a first measurement, not a
                      regression. Bless to record it

    tri t27b ratchet                          # latest lab run vs the ledger
    tri t27b ratchet --run runs/<sha>.json    # a saved run (path or URL)
    tri t27b ratchet --bless                  # rewrite the ledger from the run
    tri t27b ratchet --json                   # the verdict, machine-readable

Exit 0 green, 1 red (or a refused bless), 2 usage or an unreadable input.
"""
import argparse
import datetime as dt
import functools
import glob
import json
import os
import re
import subprocess
import sys
import time
import urllib.request


def rules():
    """The steward's decisions, compiled from specs/tri/t27b/steward.t27 (#6198).
    Loaded on first use: `status --json` and `doctor` do not need it."""
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    import t27b_rules
    return t27b_rules


def dogfood_rules():
    """Which specs are our own and what a lab row of one means, compiled from
    specs/tri/t27b/dogfood.t27 (#6457). Loaded on first use."""
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    import t27b_dogfood
    return t27b_dogfood


def reduce_tool():
    """`tri t27b reduce` and the fixtures it wrote, decided in specs/tri/t27b/reduce.t27 (#6445)."""
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    import t27b_reduce
    return t27b_reduce

LAB = "https://t27b-lab-production.up.railway.app"
REPO = "gHashTag/t27"
STATE = os.path.expanduser("~/.local/state/t27b-queen")
REQUIRED = ("validate", "check-linked-issue", "parse-ratchet")
# t27b-native-linux / -macos (#6444): cargo test --release -p t27b on arm64 Linux
# and Apple Silicon (workflow t27b-native.yml). Its third check,
# t27b-native-ratchet, is not required: master's ledger may be red, so Q16
# judges it like any other non-required check.
REQUIRED_WHEN_PRESENT = ("loop-tools-tracked", "t27b-native-linux", "t27b-native-macos")
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LEDGER = os.path.join(ROOT, "docs", "reports", "t27b_expectations.json")


class Unreadable(Exception):
    pass


def run(argv, cwd=None, timeout=60):
    try:
        p = subprocess.run(argv, cwd=cwd, capture_output=True, text=True, timeout=timeout,
                           env=dict(os.environ, GIT_OPTIONAL_LOCKS="0"))
    except (OSError, subprocess.TimeoutExpired) as e:
        raise Unreadable(f"{argv[0]}: {e}")
    if p.returncode != 0:
        raise Unreadable(f"{' '.join(argv[:3])}: exit {p.returncode}: {p.stderr.strip()[:200]}")
    return p.stdout


class Sources:
    """Every external read goes through here, so a fixture can stand in for it."""

    def __init__(self, fixture=None, lab=LAB, state=STATE):
        self.fixture, self.lab, self.state = fixture, lab, state

    def _fx(self, name):
        path = os.path.join(self.fixture, name)
        if not os.path.exists(path):
            raise Unreadable(f"fixture {name} absent")
        with open(path, encoding="utf-8") as f:
            return f.read()

    def now(self):
        if self.fixture:
            return parse_time(self._fx("now.txt").strip())
        return dt.datetime.now(dt.timezone.utc)

    def lab_json(self):
        if self.fixture:
            text = self._fx("lab.json")
        else:
            try:
                with urllib.request.urlopen(self.lab + "/latest.json", timeout=30) as r:
                    text = r.read().decode("utf-8")
            except Exception as e:  # noqa: BLE001 -- any failure is "could not read"
                raise Unreadable(f"{self.lab}/latest.json: {e}")
        try:
            return json.loads(text)
        except ValueError as e:
            raise Unreadable(f"latest.json: {e}")

    def lab_status(self):
        """The running lab process's /status.json (it carries `image`, #6443)."""
        if self.fixture:
            text = self._fx("status.json")
        else:
            try:
                with urllib.request.urlopen(self.lab + "/status.json", timeout=30) as r:
                    text = r.read().decode("utf-8")
            except Exception as e:  # noqa: BLE001 -- any failure is "could not read"
                raise Unreadable(f"{self.lab}/status.json: {e}")
        try:
            return json.loads(text)
        except ValueError as e:
            raise Unreadable(f"status.json: {e}")

    def lab_files(self, ref):
        """{file name: git blob sha} of contrib/railway/t27b-lab at `ref`."""
        if self.fixture:
            return json.loads(self._fx("master_lab.json"))
        out = json.loads(run(["gh", "api", f"repos/{REPO}/contents/contrib/railway/t27b-lab?ref={ref}"],
                             timeout=60))
        if not isinstance(out, list):
            raise Unreadable("gh api contents: not a directory listing")
        return {e["name"]: e["sha"] for e in out if e.get("type") == "file"}

    def master(self):
        if self.fixture:
            return self._fx("master.txt").strip()
        out = run(["git", "ls-remote", "origin", "refs/heads/master"], cwd=ROOT)
        return out.split()[0] if out.strip() else ""

    def origin_master(self):
        """This clone's origin/master, as last fetched -- no network (#6325)."""
        if self.fixture:
            return self._fx("master.txt").strip()
        return run(["git", "rev-parse", "origin/master"], cwd=ROOT).strip()

    def behind_count(self, sha, master):
        """`git rev-list --count sha..master`, or None when sha is not in this
        clone (fixture: behind.txt, absent = not present locally)."""
        if self.fixture:
            try:
                return int(self._fx("behind.txt").strip())
            except Unreadable:
                return None
        try:
            run(["git", "cat-file", "-e", f"{sha}^{{commit}}"], cwd=ROOT)
            return int(run(["git", "rev-list", "--count", f"{sha}..{master}"], cwd=ROOT).strip())
        except (Unreadable, ValueError):
            return None

    def prs(self):
        if self.fixture:
            prs = json.loads(self._fx("prs.json"))
        else:
            prs = json.loads(run(["gh", "pr", "list", "--repo", REPO, "--state", "open", "--search",
                                  "t27b", "--json",
                                  "number,title,body,headRefName,baseRefName,mergeable,statusCheckRollup"],
                                 timeout=90))
        return [p for p in prs if is_t27b(p)]

    PR_FIELDS = "number,title,body,state,headRefName,baseRefName,mergeable,statusCheckRollup"

    def pr(self, n):
        if self.fixture:
            for p in json.loads(self._fx("prs.json")):
                if p.get("number") == n:
                    return p
            raise Unreadable(f"fixture prs.json has no #{n}")
        return json.loads(run(["gh", "pr", "view", str(n), "--repo", REPO, "--json", self.PR_FIELDS], timeout=90))

    def heads(self, branch):
        """[{number, state}] of every PR whose head is `branch` (watch: the stacked parent)."""
        if self.fixture:
            return json.loads(self._fx("heads.json")).get(branch, [])
        return json.loads(run(["gh", "pr", "list", "--repo", REPO, "--head", branch, "--state", "all",
                               "--json", "number,state"], timeout=60))

    def master_checks(self, depth=12):
        """{check name: state} -- the newest VERDICT per name over the last
        `depth` master commits. The tip's runs may still be going, and master's
        runs are often cancelled by the next push: a cancelled, skipped or
        neutral run says nothing about the check, so the scan looks further
        back (live 2026-10-04: with cancelled runs counted, five checks red on
        a PR and green on master read as "red on master too")."""
        if self.fixture:
            return json.loads(self._fx("master_checks.json"))
        shas = run(["gh", "api", f"repos/{REPO}/commits?sha=master&per_page={depth}", "--jq", ".[].sha"],
                   timeout=60).split()
        if not shas:
            raise Unreadable("no master commits from the GitHub API")
        return fold_master([run(["gh", "api", "--paginate", f"repos/{REPO}/commits/{sha}/check-runs?per_page=100",
                                 "--jq", r'.check_runs[] | "\(.name)\t\(.conclusion // .status)"'],
                                timeout=90) for sha in shas], shas)

    def claim(self):
        text = self._fx("claim.json") if self.fixture else _read(os.path.join(self.state, "claim.json"))
        return json.loads(text) if text and text.strip() else None

    def alive(self, pid):
        if self.fixture:
            return str(pid) in self._fx("alive_pids.txt").split()
        try:
            os.kill(int(pid), 0)
            return True
        except ProcessLookupError:
            return False
        except PermissionError:
            return True

    def ledger(self):
        return self._fx("ledger.md") if self.fixture else _read(os.path.join(self.state, "ledger.md"))

    def worktrees(self):
        if self.fixture:
            return [p for p in self._fx("worktrees.txt").split("\n") if p.strip()]
        return sorted(p for p in glob.glob("/tmp/t27b-*")
                      if os.path.isdir(p) and os.path.exists(os.path.join(p, ".git")))

    def cwds(self):
        text = self._fx("cwd.txt") if self.fixture else run(["lsof", "-w", "-d", "cwd", "-Fpn"], timeout=60)
        return parse_lsof(text)

    def ps(self):
        return self._fx("ps.txt") if self.fixture else run(["ps", "-axo", "pid=,etime=,command="])

    def railway(self):
        if self.fixture:
            return self._fx("railway.txt").strip()
        return run(["railway", "--version"], timeout=20).strip()

    def _get(self, path):
        try:
            with urllib.request.urlopen(f"{self.lab}/{path}", timeout=60) as r:
                return r.read().decode("utf-8")
        except Exception as e:  # noqa: BLE001 -- any failure is "could not read"
            raise Unreadable(f"{self.lab}/{path}: {e}")

    def run_shas(self):
        if self.fixture:
            d = os.path.join(self.fixture, "runs")
            if not os.path.isdir(d):
                raise Unreadable("fixture runs/ absent")
            names = os.listdir(d)
        else:
            names = re.findall(r'href="([0-9a-f]{7,40}\.json)"', self._get("runs/"))
        return sorted(n[:-5] for n in names if n.endswith(".json"))

    def run_json(self, sha):
        text = self._fx(os.path.join("runs", sha + ".json")) if self.fixture else self._get(f"runs/{sha}.json")
        try:
            return json.loads(text)
        except ValueError as e:
            raise Unreadable(f"runs/{sha}.json: {e}")


def _read(path):
    try:
        with open(path, encoding="utf-8") as f:
            return f.read()
    except FileNotFoundError:
        return None
    except OSError as e:
        raise Unreadable(f"{path}: {e}")


def parse_time(s):
    s = s.strip().replace("Z", "+00:00")
    if re.match(r"^\d{4}-\d\d-\d\dT\d\d:\d\d\+", s):  # minute precision, as the ledger writes it
        s = s.replace("+", ":00+", 1)
    t = dt.datetime.fromisoformat(s)
    return t if t.tzinfo else t.replace(tzinfo=dt.timezone.utc)


def parse_lsof(text):
    """lsof -Fpn: a `p<pid>` line, then `n<path>` for its cwd. Returns [(pid, path)]."""
    out, pid = [], None
    for line in text.splitlines():
        if line.startswith("p"):
            pid = line[1:]
        elif line.startswith("n") and pid:
            out.append((pid, line[1:]))
    return out


def is_t27b(pr):
    """GitHub's search `t27b` also matches any PR whose body mentions it in passing
    (live, 2026-10-04: 16 hits, 2 of them t27b work). A t27b PR names t27b in its
    title or head branch, or refers to the epic #6063 or the umbrella #5977."""
    text = f"{pr.get('title', '')} {pr.get('headRefName', '')}".lower()
    return "t27b" in text or bool(re.search(r"#(6063|5977)\b", pr.get("body") or ""))


def required_states(pr):
    """{check name: conclusion-or-state} for the required checks that reported."""
    got = {}
    for c in pr.get("statusCheckRollup") or []:
        name = c.get("name") or c.get("context")
        if name in REQUIRED:
            got[name] = (c.get("conclusion") or c.get("state") or c.get("status") or "").upper()
    return got


def honest(lab):
    """The numbers a report may quote. `t27b_pass` alone is not one of them: it
    includes passes on specs the reference fails (#6184)."""
    s = lab.get("summary") or {}
    outside = s.get("t27b_pass_where_reference_does_not") or 0
    in_ref = s.get("t27b_pass_where_reference_passes")
    if in_ref is None and s.get("t27b_pass") is not None:
        in_ref = s["t27b_pass"] - outside
    h = {"in_ref": in_ref, "reference": s.get("reference_pass"), "outside": outside,
         "mismatch": s.get("mismatch"),
         "jit_interp_mismatch": s.get("jit_interp_mismatch", s.get("mismatch")),
         # None: the run did not compare per test (a lab or t27b before #6441).
         "reference_disagree": s.get("reference_disagree"),
         "reference_disagree_tests": s.get("reference_disagree_tests"),
         "reference_compared": s.get("reference_compared"),
         "checked": None, "compile_only": None, "frontend": []}
    results = lab.get("results")
    if isinstance(results, list) and results:
        ok = [x for x in results if x.get("t27b") == "pass" and x.get("reference") == "pass"]
        h["checked"] = sum(1 for x in ok if (x.get("tests") or 0) + (x.get("invariants") or 0) > 0)
        h["compile_only"] = len(ok) - h["checked"]
        h["frontend"] = sorted(x.get("file", "?") for x in results
                               if x.get("t27b") == "frontend" and x.get("reference") == "pass")
    return h


def worktree_state(path):
    git_path = lambda p: run(["git", "rev-parse", "--git-path", p], cwd=path).strip()  # noqa: E731
    midop = []
    for marker, label in (("MERGE_HEAD", "merge"), ("rebase-merge", "rebase"),
                          ("rebase-apply", "rebase"), ("CHERRY_PICK_HEAD", "cherry-pick")):
        p = git_path(marker)
        if not os.path.isabs(p):
            p = os.path.join(path, p)
        if os.path.exists(p):
            midop.append(label)
    dirty = [l for l in run(["git", "status", "--porcelain", "--untracked-files=normal"],
                            cwd=path).splitlines() if l.strip()]
    branch = run(["git", "rev-parse", "--abbrev-ref", "HEAD"], cwd=path).strip()
    index = os.path.join(path, git_path("index")) if not os.path.isabs(git_path("index")) else git_path("index")
    mtime = os.path.getmtime(index) if os.path.exists(index) else None
    return {"path": path, "branch": branch, "midop": sorted(set(midop)), "dirty": len(dirty),
            "index_mtime": mtime}


def collect(src, args):
    """Read every source once. Each value is the data or an Unreadable."""
    data = {}
    for key, fn in (("now", src.now), ("lab", src.lab_json), ("master", src.master), ("prs", src.prs),
                    ("claim", src.claim), ("ledger", src.ledger), ("worktrees", src.worktrees),
                    ("cwds", src.cwds), ("ps", src.ps), ("railway", src.railway)):
        try:
            data[key] = fn()
        except Unreadable as e:
            data[key] = e
        except (ValueError, KeyError) as e:
            data[key] = Unreadable(f"{key}: {e}")
    if isinstance(data["now"], Unreadable):
        data["now"] = dt.datetime.now(dt.timezone.utc)
    wts = []
    if not isinstance(data["worktrees"], Unreadable):
        for p in data["worktrees"]:
            try:
                wts.append(worktree_state(p))
            except Unreadable as e:
                wts.append({"path": p, "error": str(e)})
    data["wt"] = wts
    for key, fn in (("lab_status", src.lab_status),
                    ("lab_files", lambda: src.lab_files(data["master"] if isinstance(data["master"], str)
                                                         and data["master"] else "master"))):
        try:
            data[key] = fn()
        except Unreadable as e:
            data[key] = e
        except (ValueError, KeyError) as e:
            data[key] = Unreadable(f"{key}: {e}")
    claim = data["claim"]
    data["claim_alive"] = None
    if isinstance(claim, dict) and claim.get("pid") is not None:
        try:
            data["claim_alive"] = src.alive(claim["pid"])
        except Unreadable as e:
            data["claim_alive"] = e
    return data


def lab_image(d):
    """The running lab's image identity: status.json (the live process), else
    the last run's latest.json; {} when neither says."""
    for src in (d.get("lab_status"), d.get("lab")):
        if isinstance(src, dict) and isinstance(src.get("image"), dict):
            return src["image"]
    return {}


def anomalies(d, args):
    out = []
    add = lambda code, what, fix: out.append({"code": code, "what": what, "fix": fix})  # noqa: E731
    now = d["now"]

    lab = d["lab"]
    if isinstance(lab, Unreadable):
        add("LAB-UNREADABLE", str(lab), "open the lab URL; check the Railway deploy (railway 5.x: logs -s t27b-lab)")
    else:
        ref, commit = lab.get("ref"), lab.get("commit", "")
        if ref != "master":
            add("LAB-NOT-MASTER", f"lab ref is {ref!r}", "its numbers are that branch's; after the lab PR merges set T27_REF=master")
        master = d["master"]
        if isinstance(master, str) and master and commit and commit != master:
            add("LAB-BEHIND", f"lab commit {commit[:9]}, master {master[:9]}", "wait for the lab's next run before quoting master numbers")
        fin = lab.get("finished") or lab.get("updated")
        if fin:
            age = (now - parse_time(fin)).total_seconds() / 60
            if rules().lab_stale(age, args.stale_hours * 60):
                word = "finished" if lab.get("finished") else "updated (no finished run)"
                add("LAB-STALE", f"last run {word} {age / 60:.1f} h ago", "check /status.json and the Railway deploy logs")
        image, files = lab_image(d), d.get("lab_files")
        known = isinstance(files, dict) and "lab.py" in files
        pairs = (("lab_py_sha", "lab.py"), ("dockerfile_sha", "Dockerfile"))
        reported = bool(image.get("lab_py_sha"))
        same = known and all(image.get(k) == files.get(f) for k, f in pairs if image.get(k) or f == "lab.py")
        if rules().image(known, reported, same):
            what = (", ".join(f"{f} lab {str(image.get(k))[:9]} master {str(files.get(f))[:9]}"
                              for k, f in pairs if image.get(k) != files.get(f))
                    if reported else "the lab does not report its image (deployed before #6443)")
            built = image.get("image_built")
            add("LAB-IMAGE-STALE", what + (f"; image built {built}" if built else ""),
                "redeploy from master's contrib/railway/t27b-lab (railway 5.x up, linked dir); "
                "then check railway deployment list and the next run's image")
        s = lab.get("summary") or {}
        hh = honest(lab)
        jim, rd = hh["jit_interp_mismatch"] or 0, hh["reference_disagree"] or 0
        if rules().lanes_stop(jim, rd):
            if jim:
                add("LAB-MISMATCH", f"jit/interp mismatch {jim}", "stop the lanes; reduce and report each mismatch (runs/<sha>.json)")
            if rd:
                add("LAB-REF-DISAGREE",
                    f"reference disagree {rd} file(s), {hh['reference_disagree_tests']} test(s), of {hh['reference_compared']} compared",
                    "stop the lanes; one issue per root cause, from each record's reference_disagree (runs/<sha>.json)")
        if s.get("t27b_pass_where_reference_does_not", 0):
            add("LAB-OUTSIDE-REF", f"t27b passes {s['t27b_pass_where_reference_does_not']} spec(s) the reference fails",
                "list them; a reference defect or a vacuous t27b pass, not coverage")
        errs = {k: s.get(k, 0) for k in ("reference_lab_error", "crash", "timeout", "t27b_fail") if s.get(k, 0)}
        if isinstance(lab.get("results"), list):
            # Per-spec records: a fail or timeout the reference shares is not an alarm (#6237).
            alarms = [x for x in lab["results"]
                      if rules().is_alarm_tests(x.get("t27b") or "missing", x.get("reference") or "missing",
                                                "reference_disagree" in x, len(x.get("reference_disagree") or []))]
            errs = {k: v for k, v in errs.items() if k == "reference_lab_error"}
            for k, want in (("crash", "crash"), ("timeout", "timeout"), ("t27b_fail", "fail")):
                n = sum(1 for x in alarms if x.get("t27b") == want)
                if n:
                    errs[k] = n
        if errs:
            add("LAB-ERROR", ", ".join(f"{k} {v}" for k, v in errs.items()), "read runs/<sha>.log for each")
        for f in honest(lab)["frontend"]:
            add("LAB-FRONTEND-DISAGREES", f"{f}: t27b's frontend rejects it, the reference passes it",
                "a t27b parser/typecheck defect (or a reference that accepts too much): file it, not a blocker")
        steps = lab.get("steps") or {}
        co = steps.get("checkout")
        if isinstance(co, dict):
            kind = rules().checkout(co.get("ok") is not False, co.get("clone") == "recloned")
            if kind == "LAB-CHECKOUT":
                add(kind, f"commit {commit[:9]}: {str(co.get('error') or co.get('first_error') or '')[:160]}",
                    "the run measured nothing; a lab deploy older than #6220 never re-clones: "
                    "ask the owner for a redeploy (railway 5.x: logs -s t27b-lab first)")
            elif kind == "LAB-RECLONED":
                add(kind, f"commit {commit[:9]}: first error {str(co.get('first_error') or '')[:140]}",
                    "the run counts; name the heal in the report and watch the next run for a repeat")
        red = [k for k, v in steps.items()
               if isinstance(v, dict) and v.get("ok") is False and k not in ("ratchet", "checkout")]
        rat = steps.get("ratchet") or {}
        if rat.get("ok") is False:
            add("LAB-RATCHET", f"ratchet {rat.get('verdict')}: " + ", ".join(
                f"{k} {v}" for k, v in sorted((rat.get("counts") or {}).items()) if v),
                "read steps.ratchet.findings in latest.json; fix the spec, or bless: tri t27b ratchet --bless")
        ct = steps.get("cargo_test_t27b") or {}
        if ct.get("failed", 0):
            red.append(f"cargo_test_t27b failed {ct['failed']}")
        if red:
            add("LAB-TESTS-RED", "; ".join(red), "the lab's run is not a measurement until these are green")

    prs = d["prs"]
    if isinstance(prs, Unreadable):
        add("PR-UNREADABLE", str(prs), "gh auth status; retry")
    else:
        for pr in prs:
            n = pr.get("number")
            if pr.get("mergeable") == "CONFLICTING":
                add("PR-CONFLICTING", f"#{n} {pr.get('headRefName')}",
                    "one subagent merges origin/master into it (merge commit, plain push), in its own fresh worktree")
            bad = {k: v for k, v in required_states(pr).items() if v in ("FAILURE", "ERROR", "CANCELLED", "TIMED_OUT")}
            if bad:
                add("PR-REQUIRED-RED", f"#{n} " + ", ".join(f"{k}={v}" for k, v in sorted(bad.items())),
                    "read the failing job log; fix on the branch")
            if pr.get("baseRefName") not in (None, "master"):
                add("PR-BASE-NOT-MASTER", f"#{n} base {pr.get('baseRefName')}",
                    "when the parent merges: gh pr edit --base master, then wait for checks")

    claim, alive = d["claim"], d["claim_alive"]
    if isinstance(claim, dict) and not claim.get("released"):
        since = claim.get("since")
        age = (now - parse_time(since)).total_seconds() / 60 if since else None
        kind = rules().claim(alive if alive in (True, False) else None, age, args.claim_minutes)
        if kind == "CLAIM-DEAD":
            add(kind, f"pid {claim.get('pid')} not running, claim since {since}",
                "the claim is free; check the worktrees below before reusing any of them")
        elif kind == "CLAIM-OLD":
            add(kind, f"pid {claim.get('pid')} alive, claim {age:.0f} min old",
                "a tick is running long; read uptime and its processes before calling it stuck")

    cwds = d["cwds"] if not isinstance(d["cwds"], Unreadable) else []
    for w in d["wt"]:
        if "error" in w:
            continue
        owners = sorted({pid for pid, p in cwds if p == w["path"] or p.startswith(w["path"] + "/")}, key=int)
        who = f"processes inside: {', '.join(owners)}" if owners else "no process inside"
        age = ""
        if w.get("index_mtime"):
            mins = (now.timestamp() - w["index_mtime"]) / 60
            age = f", index touched {mins:.0f} min ago"
        if w["midop"]:
            add("WORKTREE-MIDOP", f"{w['path']} ({w['branch']}) mid-{'/'.join(w['midop'])}; {who}{age}",
                "someone's work if a process is inside or the index is recent: never reuse, reset or abort it; "
                "start new work in a fresh /tmp/t27b-<lane>-<UTC> path")
        elif w["dirty"]:
            add("WORKTREE-DIRTY", f"{w['path']} ({w['branch']}) {w['dirty']} uncommitted path(s); {who}{age}",
                "same as above: read it, do not clean it")

    rw = d["railway"]
    if not isinstance(rw, Unreadable):
        m = re.search(r"(\d+)\.(\d+)\.(\d+)", rw)
        if m and rules().railway_old(int(m.group(1))):
            add("RAILWAY-OLD-CLI", f"first railway on PATH is {m.group(0)}",
                "call ~/.nvm/versions/node/v22.22.0/bin/railway or ~/.bun/bin/railway (5.x)")

    ps = d["ps"]
    if not isinstance(ps, Unreadable) and not isinstance(lab, Unreadable):
        for line in ps.splitlines():
            if re.search(r"\bt27b\b.*\bcorpus\b.*--reference\b", line) and "tri t27b" not in line:
                parts = line.split(None, 2)
                add("LOCAL-REFERENCE", f"pid {parts[0]} running {parts[1]}: {parts[2][:100]}",
                    "the lab computes the reference denominator; ask the owner of that run before stopping it")

    led = d["ledger"]
    if isinstance(led, str):
        rows = [l for l in led.splitlines() if l.startswith("| 20")]
        if rows:
            last = parse_time(rows[-1].split("|")[1])
            mins = (now - last).total_seconds() / 60
            if rules().ledger_quiet(mins, args.quiet_hours * 60):
                add("LEDGER-QUIET", f"last ledger row {mins / 60:.1f} h old", "is the scheduled task t27b-queen-steward still enabled?")
    return out


def lab_card(lab):
    if isinstance(lab, Unreadable):
        return [f"lab        UNREADABLE: {lab}"]
    s = lab.get("summary") or {}
    h = honest(lab)
    pct = f" ({rules().pct(h['in_ref'], h['reference'])}%)" if h["in_ref"] is not None and h["reference"] else ""
    lines = [f"lab        ref {lab.get('ref')} @ {str(lab.get('commit', ''))[:9]}, finished {lab.get('finished')}",
             f"           t27b passes {h['in_ref']} of the {h['reference']} specs the reference passes{pct}, "
             f"mismatch {h['mismatch']}  ({s.get('files')} files)"]
    if h["reference_disagree"] is None:
        lines.append(f"           jit/interp mismatch {h['jit_interp_mismatch']}, reference disagree not measured "
                     "(the run has no per-test reference verdicts)")
    else:
        lines.append(f"           jit/interp mismatch {h['jit_interp_mismatch']}, reference disagree {h['reference_disagree']} "
                     f"file(s) / {h['reference_disagree_tests']} test(s) (of {h['reference_compared']} compared per test)")
    if h["checked"] is not None:
        lines.append(f"           of those {h['checked']} ran a test or invariant, {h['compile_only']} compile-only")
    lines.append(f"           not counted: {h['outside']} t27b pass(es) where the reference fails, "
                 f"{len(h['frontend'])} frontend reject(s) where it passes")
    for b in (lab.get("top_blockers") or [])[:5]:
        lines.append(f"           blocker {b.get('construct'):<34} first {b.get('first'):>4}  all {b.get('all')}")
    return lines


def status(d):
    lines = lab_card(d["lab"])
    m = d["master"]
    lines.append(f"master     {m[:9] if isinstance(m, str) else 'UNREADABLE: ' + str(m)}")
    prs = d["prs"]
    if isinstance(prs, Unreadable):
        lines.append(f"prs        UNREADABLE: {prs}")
    else:
        for pr in prs:
            req = required_states(pr)
            reqs = " ".join(f"{k}={req.get(k, 'none')}" for k in REQUIRED)
            lines.append(f"pr         #{pr['number']} {pr.get('mergeable')} base={pr.get('baseRefName')} {reqs}  {pr.get('title', '')[:60]}")
        if not prs:
            lines.append("prs        none open matching t27b")
    c = d["claim"]
    if isinstance(c, dict):
        lines.append(f"claim      since {c.get('since')} pid {c.get('pid')} alive={d['claim_alive']}"
                     + (" released" if c.get("released") else ""))
    else:
        lines.append(f"claim      {'none' if c is None else c}")
    led = d["ledger"]
    if isinstance(led, str):
        rows = [l for l in led.splitlines() if l.startswith("| 20")]
        lines.append(f"ledger     {rows[-1][:160] if rows else 'no rows'}")
    for w in d["wt"]:
        if "error" in w:
            lines.append(f"worktree   {w['path']} UNREADABLE: {w['error']}")
        else:
            lines.append(f"worktree   {w['path']} {w['branch']} dirty={w['dirty']} midop={','.join(w['midop']) or '-'}")
    return lines


# ------------------------------------------------------------------ ratchet

REASONS = ("unimplemented", "reference-bug", "n/a")
# Report order only; which kind is red is the spec's (`ratchet_is_red`).
KINDS = ("UNEXPECTED FAILURE", "UNEXPECTED PASS", "UNLISTED", "STALE", "OVER CAP", "BAD REASON",
         "MOVED", "UNJUDGED", "VACUITY MEASURED")


def read_run(where):
    """A lab run document from a path or an http(s) URL."""
    try:
        if re.match(r"^https?://", where):
            with urllib.request.urlopen(where, timeout=60) as r:
                text = r.read().decode("utf-8")
        else:
            with open(where, encoding="utf-8") as f:
                text = f.read()
        doc = json.loads(text)
    except Exception as e:  # noqa: BLE001 -- any failure is "could not read"
        raise Unreadable(f"run {where}: {e}")
    results = doc.get("results") if isinstance(doc, dict) else None
    if not isinstance(results, list) or not results:
        raise Unreadable(f"run {where}: no results[]")
    if not any(r.get("reference") == "pass" for r in results):
        raise Unreadable(f"run {where}: no reference verdicts (reference never ran?)")
    return doc


def observed(rec):
    """(verdict, first blocker) of one run record, t27b's side."""
    v = rec.get("t27b") or "missing"
    if rules().is_pass(v):
        return v, None
    blockers = rec.get("blockers") or []
    first = blockers[0] if blockers else (rec.get("detail") or "")
    return v, first[:200]


def ratchet(run, ledger):
    """Every finding of `run` against `ledger`, as [{kind, path, what}].

    Which finding an entry gets is decided by specs/tri/t27b/steward.t27
    (`entry_code`, `unlisted`, `over_cap`); this function only words it.
    """
    r = rules()
    found = []
    add = lambda kind, path, what: found.append({"kind": kind, "path": path, "what": what})  # noqa: E731
    by_path = {x.get("file"): x for x in run["results"]}
    entries = {e["path"]: e for e in ledger.get("entries", [])}
    # A ledger blessed from a run without assert counts cannot tell pass from
    # pass_vacuous; the first counted run measures it instead of regressing it.
    counted = (ledger.get("source") or {}).get("asserts_counted") is not False
    for path, e in sorted(entries.items()):
        want = e.get("t27b") or "missing"
        if not r.is_pass(want) and e.get("reason") not in REASONS:
            add("BAD REASON", path, f"reason {e.get('reason')!r} is not one of {', '.join(REASONS)}")
        rec = by_path.get(path)
        if rec is None:
            add("STALE", path, "not in this run: the spec is gone or was renamed")
            continue
        ref = rec.get("reference") or "missing"
        got, blocker = observed(rec)
        kind = r.entry(want, ref, got, counted, blocker == e.get("blocker"))
        if kind is None:
            continue
        if kind == "STALE":
            what = f"the reference does not pass it any more ({ref}: {(rec.get('reference_detail') or '')[:120]})"
        elif kind == "UNJUDGED":
            what = f"reference {ref}: no verdict either way"
        elif kind == "VACUITY MEASURED":
            what = ("ledger pass was blessed before asserts were counted; "
                    "this run counts 0 runtime asserts: bless to record pass_vacuous")
        elif kind == "UNEXPECTED FAILURE":
            why = "passes with 0 runtime asserts" if got == "pass_vacuous" else f"{got}: {blocker}"
            what = f"ledger {want}, now {why}"
        elif kind == "UNEXPECTED PASS" and want == "pass_vacuous":
            what = f"ledger pass_vacuous, now a pass with {rec.get('asserts')} runtime asserts"
        elif kind == "UNEXPECTED PASS":
            what = f"ledger {want} ({e.get('blocker')}), now {got}"
        else:
            what = f"{want} ({e.get('blocker')}) -> {got} ({blocker})"
        add(kind, path, what)
    for path, rec in sorted(by_path.items()):
        if r.unlisted(rec.get("reference") or "missing", path in entries):
            got, blocker = observed(rec)
            add("UNLISTED", path, f"the reference passes it and the ledger does not name it (t27b {got}"
                + (f": {blocker})" if blocker else ")"))
    not_pass = sum(1 for e in entries.values() if not r.is_pass(e.get("t27b") or "missing"))
    cap = ledger.get("max_not_pass")
    if r.over_cap(not_pass, cap):
        add("OVER CAP", "-", f"{not_pass} non-pass entries, max_not_pass {cap}")
    return found


def bless(run, old, accept_new=False):
    """(new ledger, refusals). Reasons a human wrote are kept; the cap never rises.

    What is recorded, with which reason, and whether the cap may move is
    decided by specs/tri/t27b/steward.t27 (`unlisted`, `is_pass`,
    `bless_reason`, `cap_rises`)."""
    r = rules()
    old_entries = {e["path"]: e for e in (old or {}).get("entries", [])}
    entries, refused = [], []
    for rec in sorted(run["results"], key=lambda x: x.get("file") or ""):
        if not r.unlisted(rec.get("reference") or "missing", False):
            continue
        path = rec["file"]
        got, blocker = observed(rec)
        if r.is_pass(got):
            entries.append({"path": path, "t27b": got})
            continue
        reason = r.bless_reason(got, (old_entries.get(path) or {}).get("reason"))
        if reason is None:
            refused.append(f"{path}: t27b {got} where the reference passes ({blocker}); read it, then add the "
                           f"entry by hand with reason reference-bug or n/a")
            continue
        entries.append({"path": path, "t27b": got, "blocker": blocker, "reason": reason})
    counts = {"pass": sum(1 for e in entries if e["t27b"] == "pass"),
              "pass_vacuous": sum(1 for e in entries if e["t27b"] == "pass_vacuous")}
    counts["not_pass"] = len(entries) - counts["pass"] - counts["pass_vacuous"]
    asserts_counted = any("asserts" in r for r in run["results"])
    if not asserts_counted:
        # The run's t27b did not count runtime asserts: 0 would be a claim.
        counts["pass_vacuous"] = None
    old_cap = (old or {}).get("max_not_pass")
    if r.cap_rises(counts["not_pass"], old_cap):
        # Accounting (#6237): specs the old ledger never named may raise the cap by exactly their
        # count; a rise from specs it already named is a regression and still refused.
        new_np = sum(1 for e in entries if e["t27b"] not in ("pass", "pass_vacuous") and e["path"] not in old_entries)
        gone = sum(1 for p, e in old_entries.items()
                   if p not in {x["path"] for x in entries} and not r.is_pass(e.get("t27b") or "missing"))
        covered = r.cap_rise_is_new(counts["not_pass"], old_cap, new_np)
        acct = (f"accounting: {counts['not_pass']} non-pass now = {counts['not_pass'] - new_np} already named "
                f"+ {new_np} new to the ledger ({gone} named non-pass entries left the run); cap {old_cap}")
        if not (accept_new and covered):
            hint = ("the rise is covered by new specs: re-run with --accept-new" if covered
                    else "the rise is larger than the new specs: an entry the ledger named regressed")
            refused.append(f"{counts['not_pass']} non-pass entries would exceed max_not_pass {old_cap}; {acct}; {hint}")
    new = {
        "schema_version": 1,
        "generated_by": "tri t27b ratchet --bless (scripts/tri_loop/t27b.py), #6115",
        "source": {"commit": run.get("commit"), "ref": run.get("ref"), "finished": run.get("finished"),
                   "asserts_counted": asserts_counted},
        "reasons": {"unimplemented": "t27b cannot compile the spec yet; the blocker is the first construct it rejects",
                    "reference-bug": "the reference passes it for the wrong reason; t27b is right not to",
                    "n/a": "outside what t27b is for (say why in the PR that sets it)"},
        "max_not_pass": counts["not_pass"],
        "counts": counts,
        "entries": entries,
    }
    return new, refused


def dump_ledger(doc):
    """The header indented, then one entry per line: a spec that moves is a
    one-line diff in review."""
    head = {k: v for k, v in doc.items() if k != "entries"}
    text = json.dumps(head, indent=1)[:-2]
    rows = ",\n".join("  " + json.dumps(e) for e in doc["entries"])
    return text + ',\n "entries": [\n' + rows + "\n ]\n}\n"


def ratchet_main(argv):
    ap = argparse.ArgumentParser(prog="tri t27b ratchet", description="t27b per-spec ledger vs a lab run (#6115)")
    ap.add_argument("--run", default=LAB + "/latest.json", help="lab run JSON: a path or a URL")
    ap.add_argument("--ledger", default=LEDGER)
    ap.add_argument("--bless", action="store_true", help="rewrite the ledger from the run")
    ap.add_argument("--accept-new", action="store_true",
                    help="bless: let the cap rise by exactly the non-pass specs new to the ledger")
    ap.add_argument("--json", action="store_true")
    try:
        args = ap.parse_args(argv)
    except SystemExit as e:
        return 2 if e.code else 0
    try:
        run = read_run(args.run)
    except Unreadable as e:
        print(f"tri t27b ratchet: UNREADABLE {e}", file=sys.stderr)
        return 2
    old = None
    if os.path.exists(args.ledger):
        try:
            with open(args.ledger, encoding="utf-8") as f:
                old = json.load(f)
        except ValueError as e:
            print(f"tri t27b ratchet: UNREADABLE ledger {args.ledger}: {e}", file=sys.stderr)
            return 2
    if args.bless:
        new, refused = bless(run, old, args.accept_new)
        for r in refused:
            print(f"REFUSED  {r}", file=sys.stderr)
        if refused:
            print(f"tri t27b ratchet --bless: {len(refused)} refusal(s); ledger not written", file=sys.stderr)
            return 1
        tmp = args.ledger + ".tmp"
        with open(tmp, "w", encoding="utf-8") as f:
            f.write(dump_ledger(new))
        os.replace(tmp, args.ledger)
        c = new["counts"]
        print(f"blessed {args.ledger} from {str(run.get('commit'))[:9]}: pass {c['pass']}, "
              f"pass_vacuous {'not counted' if c['pass_vacuous'] is None else c['pass_vacuous']}, not_pass {c['not_pass']} (max_not_pass {new['max_not_pass']})")
        return 0
    if old is None:
        print(f"tri t27b ratchet: UNREADABLE ledger {args.ledger}: absent", file=sys.stderr)
        return 2
    try:
        found = ratchet(run, old)
        counts = {k: sum(1 for f in found if f["kind"] == k) for k in KINDS}
        red = sum(v for k, v in counts.items() if rules().ratchet_is_red(k))
    except Exception as e:  # noqa: BLE001 -- RulesUnavailable lives in a lazily imported module
        if type(e).__name__ != "RulesUnavailable":
            raise
        print(f"tri t27b ratchet: UNREADABLE rules: {e}", file=sys.stderr)
        return 2
    verdict = "red" if red else "green"
    if args.json:
        print(json.dumps({"verdict": verdict, "commit": run.get("commit"),
                          "ledger_commit": (old.get("source") or {}).get("commit"),
                          "counts": counts, "findings": found}, indent=1))
    else:
        for f in found:
            print(f"{f['kind']:<19} {f['path']}: {f['what']}")
        print(f"tri t27b ratchet: {verdict} -- run {str(run.get('commit'))[:9]} vs ledger "
              f"{str((old.get('source') or {}).get('commit'))[:9]}: "
              + ", ".join(f"{k} {v}" for k, v in counts.items()))
    return 1 if red else 0


def delta(src, frm, to):
    """Per-spec transitions between two lab runs. Returns (findings, header)."""
    a, b = src.run_json(frm), (src.lab_json() if to == "latest" else src.run_json(to))
    by = lambda run: {x["file"]: x for x in run.get("results") or [] if "file" in x}  # noqa: E731
    ra, rb = by(a), by(b)
    if not ra or not rb:
        raise Unreadable("a run has no per-spec results")
    out = []
    add = lambda code, f, what: out.append({"code": code, "file": f, "what": what})  # noqa: E731
    checks = lambda x: (x.get("tests") or 0) + (x.get("invariants") or 0)  # noqa: E731
    for f in sorted(set(ra) | set(rb)):
        x, y = ra.get(f), rb.get(f)
        if x is None or y is None:
            continue
        tx, ty = x.get("t27b"), y.get("t27b")
        # The decision is the spec's (specs/tri/t27b/steward.t27), not this file's.
        code = rules().delta(x.get("reference"), y.get("reference"), tx, ty, checks(x), checks(y))
        if code == "REF-MOVED":
            add(code, f, f"reference {x.get('reference')} -> {y.get('reference')}")
        elif code in ("REGRESSED", "NEW-MISMATCH"):
            add(code, f, f"t27b {tx} -> {ty}: {(y.get('detail') or '')[:100]}")
        elif code == "GAINED":
            add(code, f, f"t27b {tx} -> pass" + ("" if checks(y) else " (compile-only)"))
        elif code == "CHECK-LOST":
            add(code, f, f"tests+invariants {checks(x)} -> 0")
    ha, hb = honest(a), honest(b)
    head = (f"from {str(a.get('commit', ''))[:9]} ({a.get('ref')}, {a.get('finished')})  "
            f"to {str(b.get('commit', ''))[:9]} ({b.get('ref')}, {b.get('finished')})\n"
            f"in-reference passes {ha['in_ref']}/{ha['reference']} -> {hb['in_ref']}/{hb['reference']}; "
            f"checked {ha['checked']} -> {hb['checked']}")
    return out, head


def previous_run(src, to):
    """The latest run that finished before --to, by `finished` (the listing is by sha)."""
    target = src.lab_json() if to == "latest" else src.run_json(to)
    tfin, tsha = target.get("finished") or "", target.get("commit")
    best = None
    for sha in src.run_shas():
        if tsha and (sha == tsha or tsha.startswith(sha) or sha.startswith(tsha)):
            continue
        fin = src.run_json(sha).get("finished") or ""
        if fin < tfin and (best is None or fin > best[0]):
            best = (fin, sha)
    if best is None:
        raise Unreadable("no earlier lab run to compare with")
    return best[1]




def main_delta(args):
    src = Sources(fixture=args.fixture, lab=args.lab)
    try:
        frm = args.from_ or previous_run(src, args.to)
        found, head = delta(src, frm, args.to)
    except Unreadable as e:
        print(f"tri t27b delta: could not read: {e}", file=sys.stderr)
        return 2
    if args.json:
        print(json.dumps({"header": head, "findings": found}, indent=1))
    else:
        print(head)
        for f in found:
            print(f"{f['code']:<13} {f['file']}  {f['what']}")
        red = sum(1 for f in found if rules().is_red(f["code"]))
        print(f"tri t27b delta: {red} regression(s), {sum(1 for f in found if f['code'] == 'GAINED')} gained, "
              f"{sum(1 for f in found if f['code'] == 'REF-MOVED')} reference move(s)")
    return 1 if any(rules().is_red(f["code"]) for f in found) else 0


# --- gen-check (#6231): is the committed C what master's t27c emits? ---------
# t27c runs on the Railway lab t27c-lab, never on this machine (owner's rule,
# 2026-10-04). The spec text goes there as base64 inside the ssh command; one
# argument is capped at 128 KiB by the kernel (MAX_ARG_STRLEN: a 133 KB command
# came back "exec: Argument list too long", 80 KB went through), so a larger
# spec is sent in GEN_CHECK_CHUNK pieces appended to one scratch file.

GEN_CHECK_SPEC = "specs/tri/t27b/steward.t27"
GEN_CHECK_GEN = "gen/c/tri/t27b/steward.c"
GEN_CHECK_CHUNK = 64000
GEN_CHECK_MARK = "T27B-GEN-CHECK"


def gen_check_env():
    e = os.environ
    return {
        "railway": e.get("T27C_LAB_RAILWAY") or os.path.expanduser("~/.nvm/versions/node/v22.22.0/bin/railway"),
        "dir": e.get("T27C_LAB_DIR") or "/private/tmp/t27c-lab/infra/t27c-lab",
        "service": e.get("T27C_LAB_SERVICE") or "t27c-lab",
        "environment": e.get("T27C_LAB_ENV") or "production",
        "bin": e.get("T27C_LAB_BIN") or "/data/target/release/t27c",
        "src": e.get("T27C_LAB_SRC") or "/data/src",
    }


def railway_runner(env):
    """A runner: one shell command on the lab -> (exit code, stdout, stderr).
    Raises Unreadable when the lab cannot be asked at all."""
    def runner(cmd):
        argv = [env["railway"], "ssh", "-s", env["service"], "-e", env["environment"], cmd]
        try:
            p = subprocess.run(argv, cwd=env["dir"], capture_output=True, text=True, timeout=180)
        except (OSError, subprocess.SubprocessError) as e:
            raise Unreadable(f"railway ssh: {e}")
        return p.returncode, p.stdout, p.stderr
    return runner


def fixture_runner(path):
    """T27B_GEN_CHECK_FAKE: a JSON file {"rc", "stdout", "stderr"} answering the
    final (gen-c) command, or {"unreachable": reason}. Chunk uploads answer 0."""
    def runner(cmd):
        try:
            with open(path, encoding="utf-8") as f:
                fx = json.load(f)
        except (OSError, ValueError) as e:
            raise Unreadable(f"fixture {path}: {e}")
        if fx.get("unreachable"):
            raise Unreadable(fx["unreachable"])
        if GEN_CHECK_MARK not in cmd:
            return 0, "", ""
        return fx.get("rc", 0), fx.get("stdout", ""), fx.get("stderr", "")
    return runner


def gen_check_commands(spec_rel, spec_bytes, env, token, chunk=GEN_CHECK_CHUNK):
    """The lab commands, in order: zero or more chunk uploads, then gen-c.
    The last command removes the scratch dir whatever happens."""
    import base64
    import shlex
    b64 = base64.b64encode(spec_bytes).decode("ascii")
    parts = [b64[i:i + chunk] for i in range(0, len(b64), chunk)] or [""]
    d = f"/tmp/t27b-gen-check-{token}"
    q = shlex.quote
    up = lambda p: f"mkdir -p {d} && printf %s {q(p)} >> {d}/spec.b64"  # noqa: E731
    cmds = [up(p) for p in parts[:-1]]
    m, binp, rel = GEN_CHECK_MARK, q(env["bin"]), q(spec_rel)
    cmds.append(
        f"trap 'rm -rf {d}' EXIT; {up(parts[-1])} && mkdir -p {d}/w/$(dirname {rel}) && "
        f"base64 -d {d}/spec.b64 > {d}/w/{rel} && cd {d}/w && "
        f"echo {m} bin {binp}; echo {m} mtime $(date -u -r {binp} +%Y-%m-%dT%H:%M:%SZ); "
        f"echo {m} binsha $(sha256sum {binp} | cut -c1-64); "
        f"echo {m} src $(git -C {q(env['src'])} log -1 --format='%H %cI' 2>/dev/null); "
        f"{binp} gen-c {rel} > {d}/out.c 2> {d}/err.txt; echo {m} rc $?; "
        f"echo {m} sha $(sha256sum < {d}/out.c | cut -c1-64); echo {m} bytes $(wc -c < {d}/out.c); "
        f"echo {m} err $(head -c 300 {d}/err.txt | tr '\\n' ' ')")
    return cmds, d


def parse_gen_check(stdout):
    got = {}
    for line in stdout.splitlines():
        if line.startswith(GEN_CHECK_MARK + " "):
            parts = line.split(" ", 2)
            if len(parts) > 1:
                got.setdefault(parts[1], parts[2].strip() if len(parts) > 2 else "")
    return got


def gen_check(spec, gen, runner=None, env=None, token=None, chunk=GEN_CHECK_CHUNK):
    """Returns a dict with verdict SAME | DIFFERS | UNREACHABLE. Never guesses:
    anything the lab did not answer in full is UNREACHABLE."""
    import hashlib
    import uuid
    env = env or gen_check_env()
    out = {"verdict": "UNREACHABLE", "spec": spec, "gen": gen}
    try:
        with open(spec if os.path.isabs(spec) else os.path.join(ROOT, spec), "rb") as f:
            spec_bytes = f.read()
        with open(gen if os.path.isabs(gen) else os.path.join(ROOT, gen), "rb") as f:
            local = f.read()
    except OSError as e:
        out["reason"] = f"local file: {e}"
        return out
    out["local_sha256"], out["local_bytes"] = hashlib.sha256(local).hexdigest(), len(local)
    rel = os.path.relpath(os.path.abspath(spec if os.path.isabs(spec) else os.path.join(ROOT, spec)), ROOT)
    if rel.startswith(".."):
        rel = os.path.basename(spec)
    if runner is None:
        fake = os.environ.get("T27B_GEN_CHECK_FAKE")
        runner = fixture_runner(fake) if fake else railway_runner(env)
        out["runner"] = f"fixture {fake}" if fake else f"railway ssh -s {env['service']} -e {env['environment']}"
    cmds, scratch = gen_check_commands(rel, spec_bytes, env, token or uuid.uuid4().hex[:12], chunk)
    out["lab_scratch"], out["lab_commands"] = scratch, len(cmds)
    try:
        for c in cmds[:-1]:
            rc, so, se = runner(c)
            if rc != 0:
                runner(f"rm -rf {scratch}")
                raise Unreadable(f"chunk upload exit {rc}: {(se or so).strip()[:200]}")
        rc, so, se = runner(cmds[-1])
    except Unreadable as e:
        out["reason"] = str(e)
        return out
    got = parse_gen_check(so)
    src = got.get("src", "").split()
    out["lab"] = {"bin": got.get("bin"), "mtime": got.get("mtime"), "bin_sha256": got.get("binsha"),
                  "src_commit": src[0] if src else None, "src_committed": src[1] if len(src) > 1 else None}
    # A binary older than the checkout it is meant to be built from may be stale.
    def when(t):
        try:
            return parse_time(t) if t else None
        except ValueError:
            return None
    m, c = when(got.get("mtime")), when(src[1] if len(src) > 1 else "")
    out["lab"]["stale"] = bool(m and c and m < c)
    if "rc" not in got or "sha" not in got or "bytes" not in got:
        out["reason"] = f"lab answered without a result (ssh exit {rc}): {(se or so).strip()[-200:]}"
        return out
    if got["rc"] != "0":
        out["reason"] = f"t27c gen-c exit {got['rc']} on the lab: {got.get('err', '')[:200]}"
        return out
    if not re.fullmatch(r"[0-9a-f]{64}", got["sha"]) or not got["bytes"].isdigit():
        out["reason"] = f"lab answered an unreadable digest: {got['sha']!r} {got['bytes']!r}"
        return out
    out["lab_sha256"], out["lab_bytes"] = got["sha"], int(got["bytes"])
    same = out["lab_sha256"] == out["local_sha256"] and out["lab_bytes"] == out["local_bytes"]
    out["verdict"] = "SAME" if same else "DIFFERS"
    return out


def gen_check_main(argv, runner=None):
    ap = argparse.ArgumentParser(prog="tri t27b gen-check",
                                 description="is the committed gen-c output what master's t27c emits, on the t27c lab (#6231)")
    ap.add_argument("--spec", default=GEN_CHECK_SPEC)
    ap.add_argument("--gen", default=GEN_CHECK_GEN)
    ap.add_argument("--json", action="store_true")
    try:
        args = ap.parse_args(argv)
    except SystemExit as e:
        return 2 if e.code else 0
    r = gen_check(args.spec, args.gen, runner=runner)
    if args.json:
        print(json.dumps(r, indent=1))
    else:
        if r["verdict"] == "SAME":
            print(f"SAME {r['lab_sha256'][:12]}")
        elif r["verdict"] == "DIFFERS":
            print(f"DIFFERS lab {r['lab_sha256'][:12]} local {r['local_sha256'][:12]}"
                  f" ({r['lab_bytes']} vs {r['local_bytes']} bytes)")
        else:
            print(f"UNREACHABLE {r.get('reason')}")
        lab = r.get("lab")
        if lab:
            print(f"lab t27c {lab['bin']} mtime {lab['mtime']} sha {str(lab['bin_sha256'])[:12]}; "
                  f"lab source {str(lab['src_commit'])[:9]} ({lab['src_committed']})"
                  + ("  STALE? the binary is older than the source checkout" if lab["stale"] else ""))
    return {"SAME": 0, "DIFFERS": 1}.get(r["verdict"], 2)


MASTER_VERDICTS = ("SUCCESS", "FAILURE", "TIMED_OUT", "ACTION_REQUIRED", "STARTUP_FAILURE")
# A run still going on a newer master commit than the last verdict: master's
# answer is not known yet, so a red check on the PR waits (Q29) instead of
# being judged against a stale verdict.
MASTER_RUNNING = ("QUEUED", "IN_PROGRESS", "WAITING", "PENDING", "REQUESTED")


def fold_master(outputs, shas=None):
    """`name<TAB>conclusion` lines per master commit, newest commit first ->
    {name: {"last": the newest conclusion that is a verdict or None, "sha":
    the commit it ran on (9 chars) or None, "running": a newer run has no
    conclusion yet (Q29)}}. The scan goes past a running tip to the newest
    verdict below it (#6334: #6333 merged on `master PENDING` while coverage
    had failed on the three commits under the queued tip). Cancelled, skipped
    and neutral runs are passed over (MASTER_VERDICTS); a check with none of
    either in the window is left out (ABSENT)."""
    got = {}
    shas = list(shas or [])
    for i, out in enumerate(outputs):
        sha = shas[i][:9] if i < len(shas) else None
        for line in out.splitlines():
            name, _, concl = line.partition("\t")
            concl = concl.strip().upper()
            if not name:
                continue
            e = got.get(name)
            if e is not None and e["last"] is not None:
                continue
            if concl in MASTER_VERDICTS:
                got[name] = {"last": concl, "sha": sha, "running": bool(e and e["running"])}
            elif concl in MASTER_RUNNING:
                got[name] = {"last": None, "sha": None, "running": True}
    return got


def master_entry(v):
    """One master_checks value as {last, sha, running}. A fixture may give a
    plain state: PENDING is a running tip with no verdict below it, anything
    else a settled verdict."""
    if isinstance(v, dict):
        return {"last": v.get("last"), "sha": v.get("sha"), "running": bool(v.get("running"))}
    if v is None:
        return {"last": None, "sha": None, "running": False}
    if str(v).upper() in MASTER_RUNNING:
        return {"last": None, "sha": None, "running": True}
    return {"last": v, "sha": None, "running": False}


def master_why(m):
    """`FAILURE@f329e27c1`, `PENDING (no verdict in window)`, ... for the why list."""
    last = (m["last"] + (f"@{m['sha']}" if m["sha"] else "")) if m["last"] else None
    if m["running"]:
        return f"{last}, newer run going" if last else "PENDING, no completed run in window"
    return last or "ABSENT"


def rollup(pr):
    """{check name: state} of a PR; a re-run's later entry wins."""
    got = {}
    for c in pr.get("statusCheckRollup") or []:
        name = c.get("name") or c.get("context")
        if name:
            got[name] = (c.get("conclusion") or c.get("state") or c.get("status") or "").upper() or "PENDING"
    return got


def ready(pr, master):
    """One PR against master's check results -> {number, verdict, why}. The
    verdict is steward.t27's (`check_effect`, `effect_fold`, `pr_ready`)."""
    r = rules()
    checks = rollup(pr)
    required = REQUIRED + tuple(n for n in REQUIRED_WHEN_PRESENT if n in checks)
    why, eff_req, eff_other = [], [], []
    for name in required:
        m = master_entry(master.get(name))
        e = r.check_effect(checks.get(name, "ABSENT"), r.master_state(m["running"], m["last"] or "ABSENT"), True)
        eff_req.append(e)
        if e:
            why.append(f"{name}={checks.get(name, 'ABSENT')}")
    for name, state in sorted(checks.items()):
        if name in required:
            continue
        m = master_entry(master.get(name))
        e = r.check_effect(state, r.master_state(m["running"], m["last"] or "ABSENT"), False)
        eff_other.append(e)
        if e or (m["running"] and r.is_red_state(state)):  # report the look-back past a running tip
            why.append(f"{name}={state} (master {master_why(m)})")
    base_master = pr.get("baseRefName") == "master"
    is_open = (pr.get("state") or "OPEN").upper() == "OPEN"
    verdict = r.pr_ready(is_open, pr.get("mergeable"), base_master, eff_req, eff_other)
    if verdict == "CLOSED":
        why = [f"state={pr.get('state')}"]
    elif verdict == "CONFLICT":
        why.insert(0, "mergeable=CONFLICTING")
    elif verdict == "RETARGET":
        why.insert(0, f"base={pr.get('baseRefName')}")
    elif verdict == "WAIT" and not why:
        why.append(f"mergeable={pr.get('mergeable')}")
    return {"number": pr.get("number"), "verdict": verdict, "title": pr.get("title", ""), "why": why}


def ready_main(argv):
    ap = argparse.ArgumentParser(prog="tri t27b ready", description="may each t27b PR merge now? (#6244; never merges)")
    ap.add_argument("numbers", nargs="*", type=int, help="PR numbers (default: every open t27b PR)")
    ap.add_argument("--fixture", help="read prs.json and master_checks.json from this directory (tests)")
    ap.add_argument("--json", action="store_true")
    try:
        args = ap.parse_args(argv)
    except SystemExit as e:
        return 2 if e.code else 0
    src = Sources(fixture=args.fixture)
    try:
        prs = [src.pr(n) for n in args.numbers] if args.numbers else src.prs()
        master = src.master_checks()
    except (Unreadable, ValueError) as e:
        print(f"tri t27b ready: UNREADABLE {e}", file=sys.stderr)
        return 2
    try:
        out = [ready(p, master) for p in prs]
    except ValueError as e:  # an unknown check state is an error, never a guess
        print(f"tri t27b ready: UNREADABLE {e}", file=sys.stderr)
        return 2
    if args.json:
        print(json.dumps(out, indent=1))
    else:
        for o in out:
            print(f"#{o['number']:<6} {o['verdict']:<9} {', '.join(o['why']) or '-'}  {o['title'][:60]}")
        if not out:
            print("tri t27b ready: no open t27b PR")
    return 0 if out and all(o["verdict"] == "READY" for o in out) else 1


def parent_of(src, pr):
    """NONE | MERGED | OPEN | GONE: the state of the PR whose head is this PR's
    base. Read from GitHub every time, never inferred from a read that failed
    (Q33: a watcher that guessed skipped the retarget of #6254)."""
    base = pr.get("baseRefName")
    if base == "master":
        return "NONE"
    states = {(p.get("state") or "").upper() for p in src.heads(base)}
    if "MERGED" in states:
        return "MERGED"
    if "OPEN" in states:
        return "OPEN"
    return "GONE"


def watch_step(src, n):
    """One look at PR n -> (action, line). The action is steward.t27's
    `watch_action`; nothing is changed here."""
    pr = src.pr(n)
    v = ready(pr, src.master_checks())
    parent = parent_of(src, pr)
    act = rules().watch_action(v["verdict"], parent)
    why = ", ".join(v["why"]) or "-"
    return act, f"#{n:<6} {v['verdict']:<9} parent={parent:<7} -> {act:<8} {why}"


def watch_act(n, act):
    """Run the action. A merge is `gh pr merge --merge` only; master comes in
    through `gh pr update-branch`, a merge commit on the PR branch (no force-push)."""
    if act == "MERGE":
        run(["gh", "pr", "merge", str(n), "--repo", REPO, "--merge"], timeout=120)
    elif act == "RETARGET":
        run(["gh", "pr", "edit", str(n), "--repo", REPO, "--base", "master"], timeout=60)
        run(["gh", "pr", "update-branch", str(n), "--repo", REPO], timeout=120)


def watch_main(argv, act=watch_act, sleep=time.sleep):
    ap = argparse.ArgumentParser(prog="tri t27b watch",
                                 description="merge a t27b PR stack in order, as steward.t27 decides (#6285)")
    ap.add_argument("numbers", nargs="+", type=int, help="PR numbers, parent first")
    ap.add_argument("--fixture", help="read prs.json, master_checks.json, heads.json from here; never acts")
    ap.add_argument("--once", action="store_true", help="look at the first open PR once and exit")
    ap.add_argument("--dry-run", action="store_true", help="print the action, do not run it")
    ap.add_argument("--interval", type=float, default=150.0, help="seconds between looks (default 150)")
    ap.add_argument("--hours", type=float, default=8.0, help="give up after this long (default 8)")
    try:
        args = ap.parse_args(argv)
    except SystemExit as e:
        return 2 if e.code else 0
    src = Sources(fixture=args.fixture)
    # a fixture never reaches GitHub: only an injected `act` (tests) runs there
    dry = args.dry_run or (bool(args.fixture) and act is watch_act)
    queue = list(args.numbers)
    deadline = time.time() + args.hours * 3600
    while queue and time.time() < deadline:
        n = queue[0]
        try:
            a, line = watch_step(src, n)
        except (Unreadable, ValueError) as e:
            print(f"#{n:<6} UNREADABLE {e} -- looking again, no action", flush=True)
            if args.once:
                return 2
            sleep(args.interval)
            continue
        print(line + (" (dry run)" if dry and a in ("MERGE", "RETARGET") else ""), flush=True)
        if a == "STOP":
            return 1
        if not dry and a in ("MERGE", "RETARGET"):
            try:
                act(n, a)
            except Unreadable as e:
                print(f"#{n:<6} {a} failed: {e}", flush=True)
                return 1
        if a == "DONE" or (a == "MERGE" and not dry):
            queue.pop(0)
            continue
        if args.once:
            return 0 if a == "MERGE" else 1
        sleep(args.interval)
    if queue:
        print(f"tri t27b watch: still open after {args.hours} h: {queue}", flush=True)
        return 1
    print("tri t27b watch: all merged or closed", flush=True)
    return 0


def next_lanes(lab):
    """Count one lab run for `tri t27b next` (#6317). Which side of the picker
    a file counts on is steward.t27's `lane_kind`, the rank its `lane_score`,
    the with-tests denominator its `with_tests`; this only counts and sorts.
    Slip Q38: a lane picked from blockers without the reference moved the
    counted pass number by 0, because the reference could not compile its files."""
    R = rules()
    D = dogfood_rules()
    results = lab.get("results")
    if not isinstance(results, list) or not results:
        raise Unreadable("the lab run has no per-file results[]")
    split = {"pass": 0, "pass_vacuous": 0, "blocked": 0, "other": 0}
    fams = {"LANE": {}, "REFERENCE-BUG": {}}
    files = {"LANE": 0, "REFERENCE-BUG": 0}
    for x in results:
        ref, t = x.get("reference"), x.get("t27b")
        if ref == "pass":
            split[t if t in split else "other"] += 1
        kind = R.lane_kind(ref, t)
        blockers = [b for b in (x.get("blockers") or []) if b]
        if kind is None or not blockers:
            continue
        files[kind] += 1
        for i, b in enumerate(dict.fromkeys(blockers)):
            f = fams[kind].setdefault(b, {"family": b, "sole": 0, "first": 0, "any": 0, "own": 0})
            f["any"] += 1
            f["own"] += 1 if D.own(x.get("file") or "") else 0
            f["first"] += 1 if i == 0 else 0
            f["sole"] += 1 if len(set(blockers)) == 1 else 0
    for f in fams["LANE"].values():
        f["score"] = R.lane_score(f["sole"], f["first"])
    # Order: dogfood.t27's lane_before (score, then our own specs as the tie-break); then any, then name.
    def cmp(a, b):
        if D.lane_before(a["score"], a["own"], b["score"], b["own"]):
            return -1
        if D.lane_before(b["score"], b["own"], a["score"], a["own"]):
            return 1
        return (b["any"] - a["any"]) or ((a["family"] > b["family"]) - (a["family"] < b["family"]))
    lanes = sorted(fams["LANE"].values(), key=functools.cmp_to_key(cmp))
    bugs = sorted(fams["REFERENCE-BUG"].values(), key=lambda f: (-f["any"], f["family"]))
    ref_pass = sum(split.values())
    tested = R.with_tests(ref_pass, split["pass_vacuous"])
    return {"commit": lab.get("commit"), "ref": lab.get("ref"), "finished": lab.get("finished"),
            "reference_pass": ref_pass, "t27b": split, "with_tests": tested,
            "pct_with_tests": R.pct(split["pass"], tested),
            "lane_files": files["LANE"], "reference_bug_files": files["REFERENCE-BUG"],
            "lanes": lanes, "reference_bugs": bugs}


def lab_behind(src, commit):
    """Is the lab run older than origin/master? (#6325) False when it is the
    tip, None when either side is unknown, else {lab, master, commits} with
    commits None when the lab sha is not in this clone. Informational only:
    the ranking itself is unchanged, but families fixed on master since the
    run may still rank first."""
    try:
        master = src.origin_master()
    except Unreadable:
        return None
    if not commit or not master:
        return None
    if commit == master:
        return False
    return {"lab": commit, "master": master, "commits": src.behind_count(commit, master)}


def behind_line(b):
    if not b:
        return None
    n = b["commits"]
    how = "behind (unknown count)" if n is None else f"{n} commit{'' if n == 1 else 's'} behind"
    return (f"lab run {b['lab'][:9]} is {how} origin/master {b['master'][:9]}; families fixed since "
            "may still rank -- prefer the last lane's fresh --reference list")


def next_card(n, top):
    s = n["t27b"]
    out = [f"lab run {str(n['commit'])[:9]} ({n['ref']}), finished {n['finished']}",
           f"reference passes {n['reference_pass']}: t27b pass {s['pass']}, pass_vacuous {s['pass_vacuous']} "
           f"(0 tests), blocked {s['blocked']}, other {s['other']}",
           f"t27b pass {s['pass']} / {n['with_tests']} with tests ({n['pct_with_tests']}%)",
           "",
           f"next lane -- {n['lane_files']} files where the reference passes and t27b is blocked",
           f"  {'rank':>4}  {'sole':>4}  {'first':>5}  {'any':>4}  {'own':>4}  family   (sole: unlocks on its own;"
           " order: steward.t27 lane_score, ties: dogfood.t27 own specs)"]
    for i, f in enumerate(n["lanes"][:top], 1):
        out.append(f"  {i:>4}  {f['sole']:>4}  {f['first']:>5}  {f['any']:>4}  {f['own']:>4}  {f['family']}")
        if f.get("repro"):
            out.append(f"  {'':>4}  repro: {f['repro']}")
    if n["lanes"] and "repro" in n["lanes"][0] and not n["lanes"][0]["repro"]:
        out.append(f"  {'':>4}  no reduced repro for rank 1 yet: tri t27b reduce <a spec it blocks> "
                   f"--family \"{n['lanes'][0]['family']}\"")
    if len(n["lanes"]) > top:
        out.append(f"  ... {len(n['lanes']) - top} more families (--top N)")
    out += ["",
            f"reference bugs to file, not lanes -- {n['reference_bug_files']} files t27b blocks where the "
            "reference does not pass (Q38)",
            f"  {'':>4}  {'sole':>4}  {'first':>5}  {'any':>4}  family"]
    for f in n["reference_bugs"][:top]:
        out.append(f"  {'':>4}  {f['sole']:>4}  {f['first']:>5}  {f['any']:>4}  {f['family']}")
    if len(n["reference_bugs"]) > top:
        out.append(f"  ... {len(n['reference_bugs']) - top} more families (--top N)")
    return out


def next_main(argv):
    ap = argparse.ArgumentParser(prog="tri t27b next",
                                 description="rank blocker families for the next t27b lane, reference-backed (#6317)")
    ap.add_argument("--fixture", help="read lab.json from this directory instead of the lab")
    ap.add_argument("--lab", default=LAB)
    ap.add_argument("--top", type=int, default=15, help="families to print per list (default 15)")
    ap.add_argument("--json", action="store_true")
    try:
        args = ap.parse_args(argv)
    except SystemExit as e:
        return 2 if e.code else 0
    src = Sources(fixture=args.fixture, lab=args.lab)
    try:
        n = next_lanes(src.lab_json())
    except Unreadable as e:
        print(f"tri t27b next: UNREADABLE {e}")
        return 2
    n["lab_behind"] = lab_behind(src, n["commit"])
    # A reduced repro per family, from the headers `tri t27b reduce` wrote (#6445).
    try:
        found = reduce_tool().repros()
    except Exception as e:  # the repro link is an extra: never let it hide the ranking
        found = {}
        print(f"tri t27b next: repro links unavailable: {e}", file=sys.stderr)
    for f in n["lanes"] + n["reference_bugs"]:
        f["repro"] = found.get(f["family"])
    if args.json:
        print(json.dumps(n, indent=1))
    else:
        line = behind_line(n["lab_behind"])
        print("\n".join(([line, ""] if line else []) + next_card(n, args.top)))
    return 0


# --- dogfood (#6457): our own specs first --------------------------------------

def dogfood(lab):
    """Every own spec of one lab run, by dogfood.t27's row. This only reads and counts."""
    D = dogfood_rules()
    results = lab.get("results")
    if not isinstance(results, list) or not results:
        raise Unreadable("the lab run has no per-file results[]")
    names = D.text("ROW_NAMES").split(",")
    rows = {k: [] for k in names[1:]}
    for x in results:
        try:
            r = D.row(x)
        except ValueError as e:
            raise Unreadable(f"{x.get('file')}: {e}")
        if r in rows:
            rows[r].append(x)
    return {"commit": lab.get("commit"), "ref": lab.get("ref"), "finished": lab.get("finished"),
            "own_prefixes": D.text("OWN_PREFIXES"), "loader_dir": D.text("LOADER_DIR"), "fed": list(D.fed_specs()),
            "counts": {k: len(v) for k, v in rows.items()},
            "groups": [{"row": k, "title": D.text("GROUP_T27B_LANE" if i == 0 else "GROUP_T27C_ISSUE"),
                        "specs": [{"file": x.get("file"), "reference": x.get("reference"), "t27b": x.get("t27b"),
                                   "first": (x.get("blockers") or [])[:3] if k == "T27B-LANE"
                                   else re.sub(r"^does not compile: \S*spec\.zig:", "zig:",
                                               x.get("reference_detail") or "")[:110]}
                                  for x in sorted(rows[k], key=lambda x: x.get("file") or "")]}
                       for i, k in enumerate(n for n in names[1:] if D.is_work(n))]}


def dogfood_card(d):
    out = [f"tri t27b dogfood -- lab run {str(d['commit'])[:9]} ({d['ref']}), finished {d['finished']}",
           f"our own specs: prefixes {d['own_prefixes']}; fed to {d['loader_dir']}: {', '.join(d['fed']) or '-'}",
           "  " + "  ".join(f"{k} {v}" for k, v in d["counts"].items()) + f"  (total {sum(d['counts'].values())})"]
    for g in d["groups"]:
        out += ["", f"{g['title']} -- {len(g['specs'])}"]
        for x in g["specs"]:
            first = ", ".join(x["first"]) if isinstance(x["first"], list) else x["first"]
            out.append(f"  {x['file']:<52} ref {x['reference']:<8} t27b {x['t27b']:<9} {first}")
    return out


def dogfood_main(argv):
    ap = argparse.ArgumentParser(prog="tri t27b dogfood",
                                 description="our own specs in one lab run: t27b lanes and t27c issues (#6457)")
    ap.add_argument("--fixture", help="read lab.json from this directory instead of the lab")
    ap.add_argument("--lab", default=LAB)
    ap.add_argument("--json", action="store_true")
    try:
        args = ap.parse_args(argv)
    except SystemExit as e:
        return 2 if e.code else 0
    try:
        d = dogfood(Sources(fixture=args.fixture, lab=args.lab).lab_json())
    except Unreadable as e:
        print(f"tri t27b dogfood: UNREADABLE {e}")
        return 2
    print(json.dumps(d, indent=1) if args.json else "\n".join(dogfood_card(d)))
    return 0


# --- diff (#6441): one spec's verdicts, test by test, both sides ------------

def diff_rows(rec):
    """(name, t27b, reference, differs) per test name, sorted; a side without
    the test reads "-". Verdicts are "pass"/"fail"."""
    t = rec.get("test_verdicts") or {}
    r = rec.get("reference_tests") or {}
    word = lambda v: "-" if v is None else ("pass" if v else "fail")  # noqa: E731
    return [(n, word(t.get(n)), word(r.get(n)), t.get(n) != r.get(n)) for n in sorted(set(t) | set(r))]


def diff_main(argv):
    ap = argparse.ArgumentParser(prog="tri t27b diff",
                                 description="t27b's per-test verdicts beside the reference's for one spec (#6441)")
    ap.add_argument("spec")
    ap.add_argument("--run", default=LAB + "/latest.json",
                    help="lab run JSON or `t27b corpus --reference --json` output: a path or a URL")
    ap.add_argument("--json", action="store_true")
    try:
        args = ap.parse_args(argv)
    except SystemExit as e:
        return 2 if e.code else 0
    try:
        doc = read_run(args.run)
    except Unreadable as e:
        print(f"tri t27b diff: UNREADABLE {e}")
        return 2
    want = args.spec.rstrip("/")
    recs = [x for x in doc["results"] if (x.get("file") or "") == want or (x.get("file") or "").endswith("/" + want)]
    if len(recs) != 1:
        print(f"tri t27b diff: {len(recs)} record(s) match {want!r} in {args.run}")
        return 2
    rec = recs[0]
    rows = diff_rows(rec)
    if args.json:
        print(json.dumps({"file": rec.get("file"), "t27b": rec.get("t27b"), "reference": rec.get("reference"),
                          "tests": [{"name": n, "t27b": a, "reference": b, "differs": d} for n, a, b, d in rows]},
                         indent=1))
        return 1 if any(d for *_, d in rows) else 0
    print(f"{rec.get('file')}: t27b {rec.get('t27b')}, reference {rec.get('reference')}")
    if "test_verdicts" not in rec or "reference_tests" not in rec:
        print("  not compared per test: the run has no "
              + " and no ".join(k for k in ("test_verdicts", "reference_tests") if k not in rec))
        return 2
    w = max([len(n) for n, *_ in rows] + [4])
    print(f"  {'test':<{w}}  t27b  reference")
    for n, a, b, d in rows:
        print(f"  {n:<{w}}  {a:<4}  {b:<9}{'  DIFFERS' if d else ''}".rstrip())
    k = sum(1 for *_, d in rows if d)
    print(f"  {k} of {len(rows)} test(s) differ")
    return 1 if k else 0


def main(argv):
    if argv[:1] == ["diff"]:
        return diff_main(argv[1:])
    if argv[:1] == ["dogfood"]:
        return dogfood_main(argv[1:])
    if argv[:1] == ["next"]:
        return next_main(argv[1:])
    if argv[:1] == ["reduce"]:
        return reduce_tool().main(argv[1:])
    if argv[:1] == ["watch"]:
        return watch_main(argv[1:])
    if argv[:1] == ["ready"]:
        return ready_main(argv[1:])
    if argv[:1] == ["ratchet"]:
        return ratchet_main(argv[1:])
    if argv[:1] == ["gen-check"]:
        return gen_check_main(argv[1:])
    if argv[:1] == ["fuzz"]:
        # #6442: the generator and the judge are specs/tri/t27b/fuzz*.t27; t27b_fuzz.py is their I/O.
        import t27b_fuzz
        return t27b_fuzz.main(argv[1:])
    ap = argparse.ArgumentParser(prog="tri t27b", description=__doc__.split("\n")[0])
    ap.add_argument("action", choices=("status", "doctor", "delta", "ratchet", "gen-check", "ready", "watch", "next", "diff",
                                       "dogfood", "fuzz"))
    ap.add_argument("--from", dest="from_", help="delta: the earlier lab run's sha (default: the run before --to)")
    ap.add_argument("--to", default="latest", help="delta: the later run's sha (default: latest.json)")
    ap.add_argument("--fixture", help="read every source from this directory (tests)")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--lab", default=LAB)
    ap.add_argument("--stale-hours", type=float, default=6.0)
    ap.add_argument("--claim-minutes", type=float, default=100.0)
    ap.add_argument("--quiet-hours", type=float, default=2.5)
    try:
        args = ap.parse_args(argv)
    except SystemExit as e:
        return 2 if e.code else 0
    if args.action == "delta":
        return main_delta(args)
    src = Sources(fixture=args.fixture, lab=args.lab)
    d = collect(src, args)
    found = anomalies(d, args)
    if args.action == "status":
        if args.json:
            lab = d["lab"]
            print(json.dumps({"anomalies": found,
                              "lab": None if isinstance(lab, Unreadable) else lab.get("summary"),
                              "honest": None if isinstance(lab, Unreadable) else honest(lab)}, indent=1))
        else:
            print("\n".join(status(d)))
            print(f"anomalies  {len(found)} (tri t27b doctor)")
        return 0
    if args.json:
        print(json.dumps(found, indent=1))
    else:
        for a in found:
            print(f"{a['code']:<20} {a['what']}\n{'':<20} fix: {a['fix']}")
        print(f"tri t27b doctor: {len(found)} anomaly(ies)")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
