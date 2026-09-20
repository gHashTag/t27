#!/usr/bin/env python3
"""Keep the swarm's queue from running dry, from CI rather than from a laptop.

The Queen refuses a tick with "nothing to choose" when no open issue is both a
spec (a `## Boundary` naming one file, criteria under a heading that contains
"acceptance criteria") and unclaimed.

This walks the specs on the checkout it is run against, asks the COMPILER which
function bodies are still empty, subtracts every file an open issue already
claims, and opens one issue per file - or several parts sharing one boundary
when the file is large.

TWO THINGS IT DOES THAT THE LAPTOP VERSION LEARNED THE HARD WAY.

1. It measures with the compiler a bee actually runs. The same script, pointed
   at a binary built eight days earlier from uncommitted source, counted 483
   empty bodies in 139 files where the compiler from master counts 245 in 70 -
   and every issue it wrote named work that did not exist.

2. It EXECUTES every command it is about to quote, as the exact string a bee
   will read, and refuses to open the issue unless each prints the value the
   issue claims. A number that was typed rather than measured is how eleven
   issues came to say "today: 0 tests" about files carrying nine.

Run it with a built t27c on PATH (or T27C_BIN) from the repository root:

    python3 tools/queen/feed_empty_bodies.py --dry-run --limit 5
    python3 tools/queen/feed_empty_bodies.py --when-idle --limit 8
"""

import argparse, json, os, re, subprocess, sys, tempfile, urllib.request, datetime
from concurrent.futures import ThreadPoolExecutor

# The checkout this runs against: the repository root, which in CI is what
# actions/checkout put there and locally is wherever the script is invoked from.
WORK = os.environ.get("T27_REPO_ROOT") or os.getcwd()
OUTD = os.path.join(tempfile.gettempdir(), "queen-feed-issues")
LOG = os.environ.get("QUEEN_FEED_LOG", "")
# The compiler this measures with, and the one the issue tells a bee to run.
# They must be the same build: see the docstring.
T27C = os.environ.get("T27C_BIN") or "t27c"
BEE_T27C = "t27c"
REPO = "gHashTag/t27"
STATUS = "https://trios-agent-server-production.up.railway.app/queen/status"
CHUNK = 8

def log(msg):
    line = f"{datetime.datetime.now(datetime.timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')} {msg}"
    print(line)
    if LOG:
        with open(LOG, "a") as f: f.write(line + "\n")

# A directory where `t27c` IS the compiler this measures with, so the command
# an issue quotes can be run as the exact string a bee will read.
BEE_BIN = os.path.join(tempfile.gettempdir(), "queen-feed-bin")

def bee_shell(cmd):
    """Run a command EXACTLY as an issue prints it, with `t27c` resolving to the judge.

    Issue text used to be written by one piece of code and its numbers measured by
    another: the body said `grep ... prints 3` because Python's regex found 3, while
    the grep a bee would run was never executed. Running the very string the bee
    will read is the only way the "today" value in the body is a measurement."""
    os.makedirs(BEE_BIN, exist_ok=True)
    link = os.path.join(BEE_BIN, BEE_T27C)
    if os.path.realpath(link) != os.path.realpath(T27C):
        if os.path.lexists(link): os.remove(link)
        os.symlink(T27C, link)
    env = dict(os.environ, PATH=BEE_BIN + os.pathsep + os.environ.get("PATH", ""))
    r = subprocess.run(["bash", "-c", cmd], cwd=WORK, env=env, capture_output=True, text=True, timeout=300)
    return r.stdout.strip()

def claims_hold(rel, claims):
    """Every (command, value today) pair an issue states must reproduce, or no issue."""
    for cmd, want in claims:
        got = bee_shell(cmd)
        if got != str(want):
            log(f"skip {rel}: the issue would claim `{cmd}` prints {want!r}, it prints {got[:80]!r}")
            return False
    return True

def gen_stubs_cmd(rel):
    # `gen | grep -c` prints 0 when gen FAILS - a spec a bee broke would pass a
    # "prints 0" criterion. `&&` makes a failed generation print nothing at all.
    return f"{BEE_T27C} gen {rel} > /tmp/t27-gen.zig && grep -c 'not yet implemented' /tmp/t27-gen.zig"

def names_cmd(rel, names):
    return ("grep -cE '^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?(extern[[:space:]]+)?fn[[:space:]]+("
            + "|".join(names) + f")[[:space:]]*[(<]' {rel}")

def run(args, cwd=WORK, timeout=300):
    r = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=timeout)
    return (r.stdout or "") + (r.stderr or "")

def sync_master():
    """What the checkout is, not what it should be.

    In CI `actions/checkout` has already put the revision here, and a script
    that re-fetches would be measuring something other than the commit its own
    run was triggered for.
    """
    return run(["git", "rev-parse", "--short", "HEAD"]).strip()

def gen_count(rel):
    return run([T27C, "gen", rel]).count("not yet implemented")

def measure_all():
    files = sorted(os.path.relpath(os.path.join(d, f), WORK)
                   for d, _, fs in os.walk(os.path.join(WORK, "specs")) for f in fs if f.endswith(".t27"))
    with ThreadPoolExecutor(8) as ex:
        counts = list(ex.map(gen_count, files))
    return {f: c for f, c in zip(files, counts) if c > 0}

def open_boundaries():
    """Every file an OPEN issue already claims - by its `## Boundary` section,
    falling back to a specs path in the title."""
    out = subprocess.run(["gh", "issue", "list", "--repo", REPO, "--state", "open", "--limit", "1000",
                          "--json", "number,title,body"], capture_output=True, text=True, timeout=120)
    if out.returncode != 0:
        raise SystemExit("gh issue list failed: " + out.stderr[:300])
    covered = set()
    for it in json.loads(out.stdout):
        body = it.get("body") or ""
        m = re.search(r"(?ims)^##\s*boundary\s*$(.*?)(?=^#|\Z)", body)
        section = m.group(1) if m else ""
        found = re.findall(r"specs/[\w./-]+\.t27", section) or re.findall(r"specs/[\w./-]+\.t27", it["title"])
        covered.update(found)
    return covered

def empty_bodies(rel, t):
    """Functions the CODEGEN leaves unimplemented, named by the codegen itself.

    A source-side detector (brace matching for a body with no statement) agreed
    with `t27c gen` on 31 of 88 files and disagreed on 57: declarations that end
    in `;` with no body at all, and bodies the codegen cannot translate and so
    emits as `@panic("not yet implemented")`. The acceptance criterion counts
    the codegen's panics, so the list of functions must come from the same
    place - each panic is attributed to the nearest preceding `fn name(` in the
    generated output, and the signature is then quoted from the .t27 source."""
    out_lines = run([T27C, "gen", rel]).split("\n")
    names, last = [], None
    for l in out_lines:
        m = re.search(r'\bfn\s+(\w+)\s*\(', l)
        if m: last = m.group(1)
        if 'not yet implemented' in l and last and last not in names:
            names.append(last)
    out = []
    for name in names:
        m = re.search(r'(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:extern\s+)?fn\s+' + re.escape(name) + r'\s*[(<]', t)
        if not m: return None                       # codegen names a function the source does not: do not guess
        # The signature ends at the first `{` or `;` OUTSIDE brackets - `&[u8; 32]`
        # carries a `;` of its own and once truncated a quoted signature at it.
        depth, end = 0, len(t)
        for k in range(m.end(), len(t)):
            ch = t[k]
            if ch in '([<': depth += 1
            elif ch in ')]>': depth = max(0, depth - 1)
            elif ch in '{;' and depth == 0: end = k; break
        sig = " ".join(t[m.start():end].split())
        out.append((name, sig, t[:m.start()].count('\n') + 1))
    return out

def spec_status(rel):
    """t27c's own one-word verdict: IMPLEMENTED, PARTIAL, UNWRITTEN, NOPARSE or NOFN.

    Replaces a gate on parse metrics (recovery-events and friends). The master
    t27c prints no such metrics, so that gate returned None for every spec and
    would have silently stopped the feeder creating anything once the stale
    binary was retired. The exit code is 0 for every status, so the WORD is what
    is compared."""
    return run([T27C, "spec-status", rel]).strip().split("\n")[0].strip() or None

# Both spellings. The corpus holds 13201 unquoted `test name {` against 2072
# quoted `test "name"`; counting only the quoted form missed 86% of tests, and the
# criterion built on it failed a bee for writing the dominant, valid form.
TEST_RE = r'(?m)^\s*test\s+("|[A-Za-z_])'
TEST_GREP = "grep -cE '^[[:space:]]*test[[:space:]]+(\"|[A-Za-z_])'"
def tests_in(t): return len(re.findall(TEST_RE, t))

# The commands a bee may use, read from the one document that lists them.
#
# `t27c --help` carries 155 subcommands. This brief named four, and so did every
# criterion filed against the repository, so every other question an agent had -
# which functions already have a test, what a reviewer will warn about, whether
# the thing even builds - was answered with a grep or not at all. `coverage`,
# `lint` and `test-report` have been in the compiler the whole time.
#
# The list lives in docs/BEE_TOOLBELT.md and NOT here: `tri toolbelt` exercises
# every command in that document against a real spec, and
# check_documented_commands_exist.py holds every `t27c <sub>` named under docs/
# to the binary's own --help. A second copy in this file would be covered by
# neither, which is exactly how this brief came to tell bees for one day to
# reuse functions with `use module::name;` - an instruction that does not
# compile (#4298).
sys.path.insert(0, os.path.join(WORK, "tools"))
from toolbelt import brief as toolbelt_brief  # noqa: E402

BELT = toolbelt_brief(WORK)

PREAMBLE = ("**`.t27` is the hand-authored source language.** `t27c` compiles it out to "
            "C, Rust, Verilog and Zig. You are writing source, not compiler output, and not "
            "prose about an implementation.\n\n"
            # DRY, as a question a bee can actually answer. Measured 2026-09-20:
            # 576 of 4021 function bodies in specs/ are byte-identical copies -
            # `magadd` written 30 times, `sadd` 29 - because nothing could tell
            # an agent that the function it was about to write already existed.
            # The command that answers it is the last line of the toolbelt below,
            # which is why it is not repeated here.
            "**Before you write a function, ask whether it already exists.** 576 of the "
            "4021 function bodies in `specs/` are byte-identical copies of another one - "
            "`magadd` written 30 times, `sadd` 29 - because nothing told the agent writing "
            "them that the function was already there.\n\n"
            # WHAT TO DO WITH THE ANSWER, corrected 2026-09-20. This said "reuse it
            # (`use module::name;`)" for one day, and that instruction produces code
            # that does not compile: `use m::f;` and `use m;` both generate the
            # comment `// use f: no references in this module`, no `@import`, and an
            # unqualified call, so zig answers `use of undeclared identifier`.
            # Proven on a two-spec minimal case. Telling a bee to reuse across
            # modules is telling it to fail the oracle.
            "**If it already exists, do not copy it and do not try to import it.** "
            "Cross-module reuse does not generate yet: `use other::fn;` compiles to a "
            "comment and an unqualified call, and the Zig fails with `use of undeclared "
            "identifier`. Say so in your report instead - name the file and line where "
            "the function lives and state that this spec needs it - and implement only "
            "what this spec's own criteria ask for. `Duplicate Body Ratchet` fails a "
            "pull request that adds a new copy, and 576 of 4021 bodies here are already "
            "copies, each of which has to be fixed everywhere it was written.\n"
            + ("\n**The compiler answers more questions than `grep` does.** Every one of "
               "these reads the file and prints; none of them writes. Run them from the "
               "repository root, with `<spec>` replaced by the file named in "
               "`## Boundary`:\n\n" + BELT + "\n\n"
               "**Check yourself with `t27c test-report` before you report.** It builds "
               "this spec and runs its own tests, which is what the review does. `BLOCKED` "
               "means the generated Zig does not compile, and the error is printed beside "
               "it. If the block comes from something outside this file, say so in your "
               "report and name the error rather than working around it. The rest of the "
               "toolbelt is in `docs/BEE_TOOLBELT.md`.\n" if BELT else ""))

def single_issue(rel, t, emp, status):
    names = [e[0] for e in emp]; n = len(emp)
    stubs, have, tests = gen_stubs_cmd(rel), names_cmd(rel, names), f"{TEST_GREP} {rel}"
    ntests = bee_shell(tests)
    if not claims_hold(rel, [(f"{BEE_T27C} spec-status {rel}", status), (stubs, n), (have, n),
                             (tests, tests_in(t))]):
        return None
    ntests = int(ntests)
    title = f"Implement the {n} empty function bod{'y' if n == 1 else 'ies'} in {rel}"
    L = [f"# {title}\n", "## Context\n",
         f"`{rel}` is {len(t.splitlines())} lines long. {n} of its functions "
         f"{'is' if n == 1 else 'are'} declared with a body that holds no statement - a signature "
         f"and a comment where the implementation belongs.\n", PREAMBLE,
         "## Current state - re-run these yourself, from the repository root\n", "```",
         f"$ {BEE_T27C} spec-status {rel}", f"{status}",
         f"$ {stubs}", f"{n}", "```\n",
         f"`{BEE_T27C} spec-status` is the compiler's own verdict on the file, one word: "
         "IMPLEMENTED, PARTIAL, UNWRITTEN, NOPARSE or NOFN. It exits 0 whatever it says, so "
         "compare the word it prints, not the exit code. The `&&` in the second command is "
         "deliberate: if generation fails it prints nothing, not `0`.\n",
         "## What to write\n",
         "Keep every signature exactly as it is - the signature is the contract. Quoted verbatim from the file:\n"]
    for i, (nm, sig, line) in enumerate(emp, 1): L.append(f"{i}. line {line} - `{sig}`")
    L += ["", f"The file already has {ntests} `test` declaration(s). Add one for each function you "
              "implement; a body with nothing asserting on it is a claim, not a result. Either "
              "spelling is fine: `test name {` or `test \"name\" {`.\n",
          "## Acceptance criteria\n",
          f"- 1. `{BEE_T27C} spec-status {rel}` prints `IMPLEMENTED` (today: {status})",
          f"- 2. `{stubs}` prints `0` (today: {n})",
          (f"- 3. the name above still exists: `{have}` prints `1`" if n == 1 else
           f"- 3. all {n} names above still exist: `{have}` prints `{n}`"),
          f"- 4. `{tests}` prints at least `{ntests + n}` (today: {ntests})"]
    # A REGRESSION GUARD, NOT A NEW DEMAND. 388 of the 837 specs the oracle can
    # reach generate Zig that does not compile, for reasons that have nothing to
    # do with the bodies this issue asks for, and a criterion demanding a clean
    # build there is a criterion nobody can satisfy. Where the spec DOES build
    # today, it must still build afterwards - which is the one thing a
    # grep-shaped criterion cannot say, and the reason 388 specs satisfy every
    # criterion ever written against them and still fail the oracle.
    blocked = bee_shell(f"{BEE_T27C} test-report {rel} 2>&1 | grep -c BLOCKED")
    if blocked == "0":
        L.append(f"- 5. `{BEE_T27C} test-report {rel} 2>&1 | grep -c BLOCKED` prints `0` "
                 "- this spec compiles today and must still compile (today: 0)")
    L += ["", "## Boundary\n", rel]
    return [(title, "\n".join(L) + "\n")]

def part_issues(rel, t, emp, status):
    chunks = [emp[i:i + CHUNK] for i in range(0, len(emp), CHUNK)]
    stubs = gen_stubs_cmd(rel)
    if not claims_hold(rel, [(f"{BEE_T27C} spec-status {rel}", status), (stubs, len(emp))]
                       + [(names_cmd(rel, [e[0] for e in ch]), len(ch)) for ch in chunks]):
        return None
    res = []
    for idx, ch in enumerate(chunks):
        before = len(emp) - idx * CHUNK; after = before - len(ch)
        names = [e[0] for e in ch]; have = names_cmd(rel, names)
        last = idx == len(chunks) - 1
        title = f"Implement part {idx + 1} of {len(chunks)} of the empty bodies in {rel} ({len(ch)} functions)"
        L = [f"# {title}\n", "## Context\n",
             f"`{rel}` holds {len(emp)} functions declared with a body that contains no statement. "
             f"It is split into {len(chunks)} issues because one turn cannot carry them all.\n",
             f"**All {len(chunks)} parts name the same file, so they share one boundary and cannot run "
             f"at the same time** - the swarm holds the file for whoever has it. They must be done in "
             f"order, and the check below stops you if you arrived out of turn.\n", PREAMBLE,
             "## Before you start, from the repository root\n", "```",
             f"$ {stubs}",
             f"{before}      # if it is not {before}, this part is not yours yet - stop", "```\n",
             "The `&&` is deliberate: if generation fails the command prints nothing, not a number.\n",
             "## What to write\n", "Keep every signature exactly as it is. Quoted verbatim:\n"]
        for k, (nm, sig, line) in enumerate(ch, 1): L.append(f"{k}. line {line} - `{sig}`")
        L += ["", "Add a `test` declaration for each function you implement "
                  "(`test name {` or `test \"name\" {`).\n", "## Acceptance criteria\n",
              f"- 1. `{stubs}` prints `{after}` (on arrival: {before})",
              (f"- 2. `{BEE_T27C} spec-status {rel}` prints `IMPLEMENTED` - this is the last part"
               if last else
               f"- 2. `{BEE_T27C} spec-status {rel}` does not print `NOPARSE` - the file still parses"),
              f"- 3. all {len(ch)} names above still exist: `{have}` prints `{len(ch)}`",
              "", "## Boundary\n", rel]
        res.append((title, "\n".join(L) + "\n"))
    return res

def build(rel):
    t = open(os.path.join(WORK, rel), encoding="utf-8", errors="replace").read()
    status = spec_status(rel)
    # Only files with work in them. NOPARSE cannot be implemented into; NOFN and
    # IMPLEMENTED have nothing to do.
    if status not in ("UNWRITTEN", "PARTIAL"): return None
    emp = empty_bodies(rel, t)
    if not emp: return None
    if gen_count(rel) != len(emp): return None      # instruments disagree: skip rather than guess
    return (part_issues if len(emp) > CHUNK else single_issue)(rel, t, emp, status)

def boundaried_open():
    """Open issues that carry a `## Boundary`, which is what can be dispatched at all.

    Measured 2026-09-20: 673 open issues, 119 with a boundary. The count of open
    issues says nothing about how much work the swarm can take.
    """
    out = subprocess.run(["gh", "issue", "list", "--repo", REPO, "--state", "open",
                          "--limit", "1000", "--json", "number,body"],
                         capture_output=True, text=True, timeout=180)
    if out.returncode != 0:
        return -1
    return sum(1 for i in json.loads(out.stdout or "[]")
               if re.search(r"(?ims)^##\s*boundary\s*$", i.get("body") or ""))

PULSE_TITLE = "System pulse: what the swarm is doing, and what has stopped"

def last_scanned_skips():
    """`skip_claimed` and `skip_completed` from the pusher's last SCANNED reading.

    Empty when the issue is missing, unparseable, or its reading was itself
    taken from a tick that had not scanned. A number this cannot vouch for is
    worse than no number: it would feed a swarm that has plenty, or starve one
    that has none.
    """
    out = subprocess.run(
        ["gh", "issue", "list", "--repo", REPO, "--state", "open", "--limit", "50",
         "--search", PULSE_TITLE, "--json", "title,body"],
        capture_output=True, text=True, timeout=120,
    )
    if out.returncode != 0:
        return {}
    for issue in json.loads(out.stdout or "[]"):
        if issue.get("title") != PULSE_TITLE:
            continue
        match = re.search(r"```json\n(\{.*?\})\n```", issue.get("body") or "", re.S)
        if not match:
            return {}
        try:
            reading = json.loads(match.group(1))
        except json.JSONDecodeError:
            return {}
        if not reading.get("skips_are_fresh"):
            return {}
        return {"claimed": int(reading.get("skip_claimed", 0) or 0),
                "completed": int(reading.get("skip_completed", 0) or 0)}
    return {}

def resolve_runway(raw, capacity):
    """`auto` means twice the lanes; a number means that number; 0 means off.

    A fixed floor is the same mistake the lane count was. 24 was chosen when the
    swarm ran ten bees; the day it ran twenty, one reading showed about 12
    dispatchable issues for 20 lanes and the floor said 24, which is barely one
    round. The lanes are reported by the swarm itself on every reading, so the
    floor can come from them.
    """
    if isinstance(raw, str) and raw.strip().lower() == "auto":
        return max(4, 2 * int(capacity or 0))
    try:
        return max(0, int(raw))
    except (TypeError, ValueError):
        return 0

def queue_idle(runway=0):
    """How many issues to add now: enough to keep every lane fed.

    TWO QUESTIONS, and the first one alone was not enough. "Are lanes free?" is
    answered no by a swarm at ten of ten - which is exactly when its queue is
    being emptied fastest. The pusher's `fuel-runway` rule fires in that state,
    dispatches this feeder, and the feeder used to answer "swarm busy - nothing
    added" and add nothing. A tank is refilled while the engine runs.

    So with --runway N, this also asks "is there N issues of work left?", and
    tops up to N whatever the lanes are doing. The estimate is a subtraction of
    counts: the tick reports how many candidates it skipped as claimed or
    completed but caps the lists it prints, so the exact set cannot be removed.
    """
    try:
        d = json.load(urllib.request.urlopen(STATUS, timeout=30))
    except Exception as e:
        # None, not False. `False <= 0` is True in Python, so returning False
        # here made the caller print "swarm busy - nothing added" and exit 0 -
        # a GREEN run that fed nothing, saying the opposite of what happened,
        # on the one day the fuel line mattered most. A feeder that cannot see
        # the swarm has not measured it busy; it has not measured it at all.
        log(f"status unreachable: {e}")
        return None
    q = (d.get("queue") or {}).get("state"); w = d.get("workers") or {}
    log(f"status: queue={q} workers active={w.get('active')}/{w.get('capacity')} refusal={(d.get('lastTick') or {}).get('refusal')}")
    # How many issues to add: the idle lanes plus a small buffer, so a tick
    # never finds an empty queue but the backlog never balloons either.
    free = (w.get("capacity") or 0) - (w.get("active") or 0)
    want = free + 2 if (q == "no-eligible-work" or free > 0) else 0
    runway = resolve_runway(runway, w.get("capacity") or 0)
    if runway > 0:
        skips = {k: (v or {}).get("count", 0)
                 for k, v in ((d.get("lastTick") or {}).get("skipSummary") or {}).items()}
        # A tick that refused on capacity never scanned the board, so its
        # skipSummary is empty - and an empty summary reads exactly like a board
        # with nothing claimed. Measured 2026-09-20T12:10Z: the estimate said
        # "about 123 dispatchable" and added nothing, while the last scanning
        # tick had counted 68 claimed and 42 completed, leaving about 12. Zero
        # because nobody looked is not zero.
        #
        # The pusher takes a reading every fifteen minutes and keeps it in its
        # own issue, including whether the tick had scanned. Reading it here is
        # a coupling, and it is named as one: the alternative is a feeder that
        # cannot judge the runway in the state it exists for, because a swarm at
        # full capacity is exactly when its queue empties fastest.
        if not skips:
            skips = last_scanned_skips()
            if skips:
                log("runway: this tick refused before scanning, so the claimed and "
                    "completed counts come from the pusher's last scanned reading")
            else:
                log("runway: this tick never scanned the board and no scanned reading "
                    "is available, so only the free lanes are counted")
                return want
        have = boundaried_open()
        if have < 0:
            log("runway: gh issue list failed, so only the free lanes are counted")
        else:
            left = max(0, have - skips.get("claimed", 0) - skips.get("completed", 0))
            log(f"runway: about {left} dispatchable issue(s) against a floor of {runway} "
                f"({have} carry a boundary, {skips.get('claimed', 0)} claimed, "
                f"{skips.get('completed', 0)} completed)")
            want = max(want, runway - left)
    return want

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=8)
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--when-idle", action="store_true")
    ap.add_argument("--runway", default="0",
                    help="top the queue up to this many dispatchable issues "
                         "whatever the lanes are doing; `auto` means twice the "
                         "lanes the swarm reports")
    ap.add_argument("--order", choices=["big", "small"], default="big",
                    help="big: most empty bodies first (default); small: fewest first")
    a = ap.parse_args()
    if a.when_idle:
        want = queue_idle(a.runway)
        if want is None:
            log("could not read the swarm, so nothing was fed - and this run is RED "
                "rather than a green run that fed nothing")
            raise SystemExit(2)
        if want <= 0:
            log("swarm busy - nothing added"); return
        a.limit = min(a.limit, want)
    sha = sync_master()
    empt = measure_all()
    covered = open_boundaries()
    todo = sorted(((p, c) for p, c in empt.items() if p not in covered),
                  key=lambda x: (x[1] if a.order == "small" else -x[1], x[0]))
    log(f"master {sha}: {len(empt)} files hold {sum(empt.values())} empty bodies; "
        f"{len(covered)} files covered by open issues; {len(todo)} uncovered")
    os.makedirs(OUTD, exist_ok=True)
    made = 0
    for rel, _ in todo:
        if made >= a.limit: break
        issues = build(rel)
        if not issues:
            log(f"skip {rel}: instruments disagree"); continue
        for title, body in issues:
            if made >= a.limit: break
            slug = re.sub(r'[^a-z0-9]+', '-', title.lower()).strip('-')[:120]
            path = os.path.join(OUTD, slug + ".md")
            open(path, "w").write(body)
            if a.dry_run:
                log(f"dry-run: {title}"); made += 1; continue
            r = subprocess.run(["gh", "issue", "create", "--repo", REPO, "--title", title, "--body-file", path],
                               capture_output=True, text=True, timeout=120)
            if r.returncode != 0:
                log(f"create FAILED for {title}: {r.stderr.strip()[:200]}"); continue
            log(f"created {r.stdout.strip()} {title}"); made += 1
    log(f"done: {made} issue(s) {'would be ' if a.dry_run else ''}created")

if __name__ == "__main__":
    main()
