#!/usr/bin/env python3
"""Every spec population this campaign has published, with the matcher that
produced it, re-derivable on demand.

WHY. A population column in a reader counted COMMENTS for one pass and inflated
seven rows; `sign` was published as "63 uses in 48 specs" and is one use in one
spec, and that 63 had already become a recommendation to treat it as a
language-level question (#3501). The obvious response is to re-count every
figure. Doing so found something else: almost all of the movement was NOT
miscounting. It was that the published sentence and the matcher behind it asked
different questions.

    `len(` "142"        was a DIAGNOSTIC count in the generated C.
                        The spec population is 296 in 29 specs.
    `pub const OP_*` 20 was a count of LIST SITES in the generated C.
                        The declarations are 11, in one spec.
    `[T]` 220           depends on which names count as a type: 220 with one
                        set of primitives, 228 with `float` and `int` added.

A number without its matcher is not reproducible, and a number whose unit is
implicit is not comparable. So this file pins BOTH, and `--check` re-derives
each and prints the drift. Comments are excluded everywhere, which is what the
one genuine miscount was about.

WHEN IT IS RED (#7174). A pin is derived data: a function of the spec corpus
and of its matcher. For three days every merge that added or removed spec
text, from any lane, turned this check red on master and on every open spec
PR; the pins were re-set by hand in #6899, #6894 and #6969 and each re-pin
lagged master before its own CI ran. That lag is not a defect of anybody's
change. So --check now asks specs/ci/derived_data.t27 (KIND_PUBLISHED_FIGURE)
for a verdict per figure, and this file only measures:

  RED      the matcher changed meaning -- the tool of this change and the tool
           of its base count different numbers on the SAME corpus -- and the
           pin was not restated; or a pin this change wrote is not the count
           (at the merge ref or at the PR head). These are what the gate is for.
  REPORT   this change's own spec diff moves the figure. Annotated, not red.
  PENDING  the pin lags merges that are not this change. Not red.
  REFRESH  on master: the pin lags. A warning and a job summary; refresh it
           with --bless, which writes the per-merge trail.

The base is the commit before the change: on a pull request the merge ref's
first parent (the PR head is the second), on master the previous commit. The
corpus at the base is not checked out: every matcher is per line, so the count
at another commit is the count here minus the `git diff` lines it matches.

Usage:
  tools/published_figures.py            the table, re-derived now
  tools/published_figures.py --check [--event pr|master] [--base REV] [--pr-head REV]
                                        the verdicts; event defaults from
                                        GITHUB_EVENT_NAME, base to HEAD^1,
                                        pr-head to HEAD^2 on a pr event
  tools/published_figures.py --bless [--ref '#N'] [--since REV]
                                        rewrite drifted pins to the count now and
                                        append the per-merge trail since REV
                                        (default: the last commit that touched
                                        this file)
  tools/published_figures.py --self-check  negative control

Exit codes:
  0  no red verdict (or the table printed, or the pins blessed)
  1  a red verdict -- a matcher changed meaning, or a written pin is not the count
  2  COULD NOT RUN (no specs, no base commit, no t27c to compile the rule)
"""

import ast
import ctypes
import datetime
import os
import re
import shutil
import subprocess
import sys
import tempfile
import types

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SPECS = os.path.join(ROOT, "specs")

# (claim, unit, regex over CODE lines, pinned value, where it was published)
FIGURES = [
    ("cast_i8 uses", "uses",
     r"(?<![\w.@])cast_i8\s*\(", 1081, "#3497; 2026-10-02 (#5497) 1079 -> 1081 at 769f3252: 1079 at d3224e69, core 1079->1079, specs/port/ +2"),
    ("cast_i16 uses", "uses",
     r"(?<![\w.@])cast_i16\s*\(", 38, "#3497"),
    ("[]T{} empty slice literals", "literals",
     r"\[\]\s*[A-Za-z_][\w:]*\s*\{\s*\}", 522, "#3495; 2026-10-02 (#5497) 478 -> 525 at 769f3252: 481 at d3224e69, core 481->505, specs/port/ +20; 2026-10-06 (#6899) 525 -> 522 at b82c01ba9: #5617 (fifteen ports redone) -5, #6580 (t27b array-literal conformance) +2"),
    ("x.len() with an identifier base", "call sites",
     r"\b[A-Za-z_]\w*\s*\.\s*len\s*\(", 1364, "#3489, corrected from 1322; 2026-10-02 (#5497) 1319 -> 1414 at 769f3252: 1319 at d3224e69, core 1319->1301, specs/port/ +113; 2026-10-06 (#6899) 1414 -> 1371 at b82c01ba9: #5617 -46, #5795 (xilinx7 packets) +6, #5871 -6, #5796 #6528 #6423 +1 each; 2026-10-07 (#7174) 1371 -> 1364 at 670480bdd plus 2 commit(s) of this branch: #6815 -7, merged at 17976bd25 before the 2026-10-06 re-pins and missed by them"),
    ("x.len with an identifier base", "field reads",
     r"\b[A-Za-z_]\w*\s*\.\s*len\b(?!\s*\()", 2160, "#3489, corrected from 687; 2026-10-02 (#5497) 680 -> 2197 at 769f3252: 1075 at d3224e69, core 1075->1567, specs/port/ +630; the core discard repair added two asserts on .len: 2197 + 2 = 2199; the ml discard repair folded eight `then x.len == n` clauses into the braced tests that replace them: 2199 - 8 = 2191; 2026-10-06 (#6899) 2191 -> 2147 at b82c01ba9: #5617 -198, #6566 -14, #6307 -7, and 32 merges +175 (largest #5926 +31, #6194 +18, #6729 +13, #6555 +12, #6556 +11, #6442 +10); master then merged #6938 (+2) at 33e61b373: 2147 + 2 = 2149; #6940 adds none; 2026-10-07 (#7174) 2149 -> 2160 at 670480bdd plus 2 commit(s) of this branch: #6956 +5, #6919 +3, #7111 +2, #6970 +1"),
    ("len(x) free-function spelling", "call sites",
     r"(?<![\w.@])len\s*\(", 332, "#3489 said 142 -- that was a DIAGNOSTIC count; 2026-10-02 (#5497) 296 -> 339 at 769f3252: 296 at d3224e69, core 296->332, specs/port/ +7; 2026-10-06 (#6899) 339 -> 332 at b82c01ba9: #5617 -7"),
    ("three-segment paths a::b::c", "occurrences",
     r"\b[A-Za-z_]\w*::[A-Za-z_]\w*::[A-Za-z_]\w*", 635, "#3473, corrected from 477; 2026-10-02 (#5497) 473 -> 619 at 769f3252: 474 at d3224e69, core 474->503, specs/port/ +116; 2026-10-06 (#6899) 619 -> 631 at b82c01ba9: #5617 -10, #6307 -2, and 21 merges +24 (#5926 +3, #6276 +2, nineteen +1); 2026-10-07 (#7174) 631 -> 635 at 670480bdd plus 2 commit(s) of this branch: #6979 +3, #6956 +1"),
    ("pub const OP_* declarations", "declarations",
     r"^\s*pub\s+const\s+OP_\w+", 79, "#3497 said 20 -- that was a SITE count in the C; 2026-10-02 (#5497) 11 -> 61 at 769f3252: 61 at d3224e69, core 61->61, specs/port/ +0; 2026-10-06 (#6899) 61 -> 79 at b82c01ba9: #6442 (specs/tri/t27b/fuzz.t27) +15, #5795 (specs/xilinx7/packets.t27) +4, #5871 -1"),
    ("abs( uses", "uses",
     r"(?<![\w.@])abs\s*\(", 473, "#3501; 2026-10-02 (#5497) 389 -> 418 at 769f3252: 389 at d3224e69, core 389->418, specs/port/ +0; the ml discard repair removed contrastive_loss's duplicated loop body: 418 - 1 = 417; 2026-10-06 (#6899) 417 -> 473 at b82c01ba9: invariants that now check concrete points -- #5702 (GF8/20/24/32) +33, #5727 (sacred_physics) +12, #5724 (GF12) +8 -- and #6755 +2, #5739 +1"),
    # The pin FOLLOWED the corpus, and the movement is explained rather than
    # blessed away: #3482 deleted 188 duplicate test blocks whose bodies were
    # byte-identical to their twin. 12644 - 188 = 12456, which is what a
    # re-derivation gives -- the first thing this file caught, on its first run.
    # #3557 added specs/ui/viewport.t27 with six test blocks: 12456 + 6 = 12462.
    # #3556 (Closes #3559) added specs/automation/inngest-probe-suite.t27 with
    # three test blocks: 12462 + 3 = 12465.
    # The 2026-09-10 queen harvest (#3560) landed 9 bee patches (#3508 #3515
    # #3524 #3525 #3530 #3531 #3534 #3536 #3538), each adding test blocks
    # beside the functions it implemented: 12465 + 77 = 12542.
    # #3561 added specs/memory/tmem/ (six Trinity Memory contract specs) with
    # 51 test blocks: 12542 + 51 = 12593.
    # #3563 (S01 of gHashTag/trinity#988) added specs/trinity/ (project.t27 with
    # five test blocks, fifty capability cards with one each) and the canonical
    # copy specs/catalog/discovery.t27 with five: 12593 + 60 = 12653.
    # #3564 (S02) added specs/trinity/compiler_matrix.t27 with six: 12653 + 6 = 12659.
    # Its fixtures live under bootstrap/tests/fixtures/trinity_matrix/, outside
    # specs/, and are not counted here.
    # #3565 (S03) added specs/trinity/build_graph.t27 with four: 12659 + 4 = 12663.
    # #3566 (S04) added specs/vsa/trinity_compat.t27 with ten: 12663 + 10 = 12673.
    # #3567 (S05) rewrote specs/isa/ternary_encoding.t27 (eleven) and added specs/isa/tri27_machine.t27
    # (ten), specs/isa/tri27_bytecode.t27 (six), specs/vm/trinity_vm.t27 (five), specs/api/c_abi.t27
    # (four): 12673 + 36 = 12709.
    # #3568 (S06) added specs/tools/catalog.t27 (five) and specs/tools/mcp_protocol.t27 (five); the 29
    # cards under specs/tools/trinity/tri/ carry no test block: 12709 + 10 = 12719.
    # #3596 added specs/automation/crm-lead-magnet.t27 with five: 12719 + 5 = 12724.
    # #3598 added specs/automation/crm-sellers.t27 with five: 12724 + 5 = 12729.
    # #3600 added specs/automation/leela-agent-link.t27 with five: 12729 + 5 = 12734.
    # #3602 added specs/automation/crm-duet.t27 with five: 12734 + 5 = 12739.
    # #3604 added specs/automation/crm-client-workspace.t27 with five: 12739 + 5 = 12744.
    # #3608 added specs/automation/crm-client-ownership.t27 with five: 12744 + 5 = 12749.
    # #3613 added three to specs/automation/crm-duet.t27: 12749 + 3 = 12752.
    # #3615 added one to crm-duet.t27 and three in the new
    # specs/automation/agent-provider-chain.t27: 12752 + 4 = 12756.
    # #3617 added two to crm-duet.t27 (discovery gate, negation): 12756 + 2 = 12758.
    # reserve max_tokens added one to agent-provider-chain.t27: 12758 + 1 = 12759.
    # #3576 added specs/memory/tmem/session.t27 (durable session record layout
    # and recovery rules) with 24 test blocks, re-derived directly rather than
    # trusted from the branch's own stale comment (which said 22): 12759 + 24 = 12783.
    # 2026-10-02 (#5497): no pin had moved since d3224e69 (2026-09-16), and five
    # had already drifted AT that commit -- the merge that resolved this file's
    # conflict kept the older numbers. Since then specs/ grew from 946 to 1146
    # files, 200 of them under specs/port/, which did not exist at d3224e69: the
    # port waves added specs without moving these pins. Each note below gives the
    # value at d3224e69 and the split of the change into the rest of specs/
    # ("core") and specs/port/. No matcher changed in this file's history.
    # 2026-10-06 (#6899): the pins had not moved since 5b2f8e478 (#5613), where
    # every figure still equals its pin; spec-guards went red as merges landed.
    # Each note below names the merges that moved the figure, counted per merge
    # with these regexes. Again no matcher changed.
    ("test blocks", "blocks",
     r"^\s*test\s+(?:\"[^\"]*\"|[A-Za-z_][\w\-]*)\s*\{?\s*$", 15841,
     "#3479 pinned 12644; #3482 removed 188; #3557 added 6; #3556 added 3; #3560 added 77; #3561 added 51; #3596 added 5; #3598 added 5; #3600 added 5; #3613 added 3; #3576 added 24; 2026-10-02 (#5497) 12783 -> 14350 at 769f3252: 13056 at d3224e69, core 13056->13419, specs/port/ +931; the core discard repair removed 20 never-parsed test blocks and restored one malformed header: 14350 - 20 + 1 = 14331; the ml discard repair removed sigmoid's reflection test: 14331 - 1 = 14330; master then measured 14394 at 4c597ec7 (+64 from port PRs that did not re-pin), and removing the 80 `{ /* verify baseline */ }` placeholder tests of igla/coder/pipeline: 14394 - 80 = 14314; 2026-10-06 (#6899) master measured 15603 at b82c01ba9 (+1289 net from the merges since 5b2f8e478, which did not re-pin; #5617 -52, #6307 -6, #6566 -4 among them), and #6894 added eight to specs/numeric/formats.t27: 15603 + 8 = 15611; master then merged #6892 (+1) and #6880 (+2) at 2838800389: 15611 + 3 = 15614; master then merged #6966 (+16), #6943 (+6), #6938 (+6), #6828 (+38), #6749 (+6), #6900 (+1) and #6747 (+27) at 33e61b373: 15614 + 100 = 15714; master then merged #6965 (+6) at 38a6e30ca: 15714 + 6 = 15720; #6940 added five decision pins to specs/numeric/gf16.t27 (ties, overflow tie, no subnormals, canonical NaN, exact decode): 15720 + 5 = 15725; 2026-10-07 (#7174) 15725 -> 15841 at 670480bdd plus 2 commit(s) of this branch: #6695 +12, #7061 +10, #6919 +8, #6952 +8, #7111 +7, #6956 +7, #6998 +6, #7045 +6, and 6 more merges +22, +30 not attributable to a merge since 2116d206a (the pin did not hold there)"),
]


RULE = os.path.join("specs", "ci", "derived_data.t27")


class CouldNotRun(Exception):
    pass


def is_code(line):
    t = line.lstrip()
    # The one genuine miscount this file exists for.
    return not (t.startswith("//") or t.startswith("#"))


def code_lines():
    if not os.path.isdir(SPECS):
        print("published_figures: no specs directory. Exit 2.", file=sys.stderr)
        sys.exit(2)
    for r, _, fs in os.walk(SPECS):
        for f in sorted(fs):
            if not f.endswith(".t27"):
                continue
            p = os.path.join(r, f)
            for line in open(p, encoding="utf-8", errors="replace"):
                if is_code(line):
                    yield p, line


def derive():
    rows = [(re.compile(pat), 0, set()) for _, _, pat, _, _ in FIGURES]
    for p, line in code_lines():
        for i, (rx, _, _) in enumerate(rows):
            k = len(rx.findall(line))
            if k:
                rows[i] = (rx, rows[i][1] + k, rows[i][2] | {p})
    return [(n, len(f)) for _, n, f in rows]


# ---- git: the corpus at another commit, without checking it out ----------------------------

def git(*args):
    r = subprocess.run(["git", "-c", "core.quotePath=false", *args], cwd=ROOT, capture_output=True)
    return r.returncode, r.stdout.decode("utf-8", "replace")


def rev(name):
    rc, out = git("rev-parse", "--verify", "--quiet", name + "^{commit}")
    return out.strip() if rc == 0 and out.strip() else None


def diff_delta(a, b, figures=FIGURES):
    """count(b) - count(a) per figure, read off `git diff a b -- specs`. Exact, because
    every matcher here is applied line by line, to code lines only, in .t27 files."""
    rxs = [re.compile(f[2]) for f in figures]
    d = [0] * len(rxs)
    rc, out = git("diff", "--no-color", "--no-renames", "--no-ext-diff", "-U0", a, b, "--", "specs")
    if rc != 0:
        raise CouldNotRun(f"git diff {a} {b} failed")
    pa = pb = None
    hunk = False
    for line in out.split("\n"):
        if line.startswith("diff --git "):
            pa = pb = None
            hunk = False
            continue
        if not hunk:
            if line.startswith("--- "):
                pa = line[4:].strip('"')
            elif line.startswith("+++ "):
                pb = line[4:].strip('"')
            elif line.startswith("@@"):
                hunk = True
            continue
        if line.startswith("+"):
            sign, path = 1, pb
        elif line.startswith("-"):
            sign, path = -1, pa
        else:
            continue
        text = line[1:]
        if not path or not path.endswith(".t27") or not is_code(text):
            continue
        for i, rx in enumerate(rxs):
            d[i] += sign * len(rx.findall(text))
    return d


def tool_at(commit):
    """This file as it was at `commit`, loaded as a module that counts THIS tree.
    Returns ({claim: count here}, {claim: pin}), or None if the file did not exist."""
    rc, src = git("show", f"{commit}:tools/published_figures.py")
    if rc != 0:
        return None
    mod = types.ModuleType("published_figures_at_" + commit[:9])
    mod.__file__ = os.path.abspath(__file__)
    try:
        exec(compile(src, f"{commit[:9]}:tools/published_figures.py", "exec"), mod.__dict__)
        mod.SPECS = SPECS
        counts = {row[0]: n for row, (n, _) in zip(mod.FIGURES, mod.derive())}
        pins = {row[0]: row[3] for row in mod.FIGURES}
    except Exception as e:  # the base tool is master's own; if it cannot count, nothing is judged
        raise CouldNotRun(f"the tool at {commit[:9]} could not count this tree: {e}")
    return counts, pins


# ---- the rule: specs/ci/derived_data.t27, compiled from its gen-c output -------------------

CONSTS = ("KIND_PUBLISHED_FIGURE", "IN_SPEC", "IN_MATCHER", "EV_PULL_REQUEST", "EV_MASTER", "BY_HUMAN",
          "V_OK", "V_PENDING", "V_REPORT", "V_REFRESH", "V_OWNER", "V_RED")


def find_t27c():
    for c in (os.environ.get("T27C"), os.path.join(ROOT, "target", "release", "t27c"),
              os.path.join(ROOT, "target", "debug", "t27c"), shutil.which("t27c")):
        if c and os.path.isfile(c) and os.access(c, os.X_OK):
            return c
    return None


def rules():
    """The rule's functions and constants, compiled from `t27c gen-c` of the spec. This file
    decides nothing itself: without them it does not run."""
    t27c = find_t27c()
    if not t27c:
        raise CouldNotRun("no t27c (set T27C, or build -p t27c) to compile " + RULE)
    r = subprocess.run([t27c, "gen-c", os.path.join(ROOT, RULE)], capture_output=True, text=True)
    if r.returncode != 0 or "figure_regression" not in r.stdout:
        raise CouldNotRun(f"t27c gen-c {RULE} failed: {r.stderr.strip()[:300]}")
    d = tempfile.mkdtemp(prefix="published-figures-")
    gen = os.path.join(d, "derived_data.c")
    open(gen, "w").write(r.stdout)
    shim = os.path.join(d, "shim.c")
    # A one-line accessor per constant: the spec's #defines have no symbol of their own.
    open(shim, "w").write(f'#include "{gen}"\n' + "".join(
        f"int t27_k_{n}(void) {{ return {n}; }}\n" for n in CONSTS))
    so = os.path.join(d, "rule.so")
    cc = os.environ.get("CC", "cc")
    r = subprocess.run([cc, "-shared", "-fPIC", "-O1", "-w", "-o", so, shim], capture_output=True, text=True)
    if r.returncode != 0:
        raise CouldNotRun(f"{cc} could not compile {RULE}: {r.stderr.strip()[:300]}")
    lib = ctypes.CDLL(so)
    lib.verdict.argtypes = [ctypes.c_uint8] * 3 + [ctypes.c_bool] * 2 + [ctypes.c_uint8, ctypes.c_bool]
    lib.verdict.restype = ctypes.c_uint8
    lib.figure_regression.argtypes = [ctypes.c_uint8, ctypes.c_bool]
    lib.figure_regression.restype = ctypes.c_bool
    lib.is_red.argtypes = [ctypes.c_uint8]
    lib.is_red.restype = ctypes.c_bool
    k = {}
    for n in CONSTS:
        f = getattr(lib, "t27_k_" + n)
        f.restype = ctypes.c_int
        k[n] = f()
    return lib, k


def arg(name, default=None):
    if name in sys.argv:
        i = sys.argv.index(name)
        if i + 1 < len(sys.argv):
            return sys.argv[i + 1]
    return default


def check() -> int:
    got = derive()
    event = arg("--event") or ("pr" if os.environ.get("GITHUB_EVENT_NAME", "").startswith("pull_request") else "master")
    base = rev(arg("--base", "HEAD^1"))
    if not base:
        raise CouldNotRun("no base commit (CI needs actions/checkout fetch-depth: 2)")
    pr_head = rev(arg("--pr-head", "HEAD^2")) if event == "pr" else None
    lib, k = rules()
    vname = {k["V_OK"]: "ok", k["V_PENDING"]: "pending", k["V_REPORT"]: "report",
             k["V_REFRESH"]: "refresh", k["V_OWNER"]: "owner", k["V_RED"]: "RED"}
    ev = k["EV_PULL_REQUEST"] if event == "pr" else k["EV_MASTER"]
    old = tool_at(base)
    old_counts, old_pins = old if old else ({}, {})
    own = diff_delta(base, "HEAD")
    at_head = diff_delta(pr_head, "HEAD") if pr_head else None

    print(f"published figures: event {event}, base {base[:9]}"
          + (f", PR head {pr_head[:9]}" if pr_head else "") + f", rule {RULE}")
    print(f"{'figure':38} {'unit':13} {'pinned':>7} {'now':>7} {'this':>6} {'specs':>6}  verdict")
    rows, red = [], 0
    for i, ((name, unit, _, pinned, _), (now, nspecs)) in enumerate(zip(FIGURES, got)):
        touched = old_pins.get(name) != pinned
        matches = pinned == now or (at_head is not None and pinned == now - at_head[i])
        moved = k["IN_SPEC"] if own[i] else 0
        why = []
        if name in old_counts and old_counts[name] != now:
            moved |= k["IN_MATCHER"]
            why.append(f"the matcher changed meaning: the base tool counts {old_counts[name]} on this tree, this one {now}")
        regression = lib.figure_regression(moved, touched)
        v = lib.verdict(k["KIND_PUBLISHED_FIGURE"], ev, k["BY_HUMAN"], touched, matches, moved, regression)
        if lib.is_red(v):
            red += 1
            if regression:
                why.append("restate the pin in the same change, and say why in its note")
            else:
                why.append(f"this change writes {pinned}; the count is {now}"
                           + (f" here and {now - at_head[i]} at the PR head" if at_head is not None else ""))
        elif own[i]:
            why.append(f"this change moves it {own[i]:+d}")
        if not lib.is_red(v) and now != pinned and now - pinned != own[i]:
            why.append(f"{now - pinned - own[i]:+d} against the pin from merges that are not this change")
        print(f"{name:38} {unit:13} {pinned:7} {now:7} {own[i]:+6d} {nspecs:6}  {vname.get(v, v)}"
              + (" -- " + "; ".join(why) if why else ""))
        rows.append((name, pinned, now, own[i], vname.get(v, v), "; ".join(why)))
    report(rows, red, event)
    if red:
        print(f"\n{red} figure(s) RED. A matcher that changes meaning changes every published sentence that "
              f"quotes it;\nrestate the pin in the same change and say why in its note.")
    elif any(r[1] != r[2] for r in rows):
        print("\nNo red verdict. Drifted pins lag the corpus; refresh them with\n"
              "  python3 tools/published_figures.py --bless --ref '#<issue>'")
    return 1 if red else 0


def report(rows, red, event):
    """GitHub annotations and a job summary, when running under Actions."""
    if os.environ.get("GITHUB_ACTIONS") != "true":
        return
    for name, pinned, now, own, v, why in rows:
        msg = f"{name}: pinned {pinned}, now {now}" + (f" -- {why}" if why else "")
        if v == "RED":
            print(f"::error title=published figure::{msg}")
        elif v in ("report", "refresh"):
            print(f"::warning title=published figure ({v})::{msg}")
        elif v == "pending":
            print(f"::notice title=published figure (pending)::{msg}")
    out = os.environ.get("GITHUB_STEP_SUMMARY")
    if not out:
        return
    with open(out, "a") as f:
        f.write(f"### Published figures ({event})\n\n| figure | pinned | now | this change | verdict | why |\n"
                "|---|---:|---:|---:|---|---|\n")
        for name, pinned, now, own, v, why in rows:
            f.write(f"| {name} | {pinned} | {now} | {own:+d} | {v} | {why} |\n")
        f.write(f"\n{red} red. Lag is not red (specs/ci/derived_data.t27, #7174); refresh drifted pins with "
                "`python3 tools/published_figures.py --bless --ref '#<issue>'`.\n")


# ---- --bless: the recount, with the trail of the merges that moved it ------------------------

def label(commit):
    _, subj = git("log", "-1", "--format=%s", commit)
    # "Merge pull request #N", or the last parenthesis: "... (Closes #A) (#N)" is pull request N.
    m = re.search(r"Merge pull request #(\d+)", subj) or re.search(r"#(\d+)\)\s*$", subj.strip())
    return f"#{m.group(1)}" if m else commit[:9]


def bless() -> int:
    got = derive()
    head = rev("HEAD")
    since = arg("--since")
    if not since:
        _, since = git("log", "-1", "--format=%H", "HEAD", "--", "tools/published_figures.py")
        since = since.strip()
    since = rev(since) if since else None
    if not since:
        raise CouldNotRun("no --since commit to attribute the drift from")
    ref = arg("--ref", "bless")
    # Name a commit a reader can find: master's, plus this branch's own commits when blessed on a branch.
    mb = rev("origin/master")
    mb = rev(git("merge-base", "HEAD", mb)[1].strip()) if mb else None
    at = head[:9]
    if mb and mb != head:
        _, n = git("rev-list", "--count", f"{mb}..HEAD")
        at = f"{mb[:9]} plus {n.strip()} commit(s) of this branch"
    old = tool_at(since)
    old_counts = old[0] if old else {}
    _, out = git("rev-list", "--first-parent", "--reverse", f"{since}..HEAD")
    per = []
    for c in out.split():
        d = diff_delta(c + "^1", c)
        if any(d):
            per.append((label(c), d))
    day = datetime.date.today().isoformat()
    path = os.path.abspath(__file__)
    src = open(path).read()
    table = next(n.value for n in ast.parse(src).body
                 if isinstance(n, ast.Assign) and any(getattr(t, "id", None) == "FIGURES" for t in n.targets))
    edits = []
    for i, ((name, _, _, pinned, _), (now, _), node) in enumerate(zip(FIGURES, got, table.elts)):
        if now == pinned:
            continue
        moves = sorted(((lab, d[i]) for lab, d in per if d[i]), key=lambda x: -abs(x[1]))
        shown, rest = moves[:8], moves[8:]
        parts = [f"{lab} {dv:+d}" for lab, dv in shown]
        if rest:
            parts.append(f"and {len(rest)} more merges {sum(dv for _, dv in rest):+d}")
        left = now - pinned - sum(dv for _, dv in moves)
        if name in old_counts and old_counts[name] != now:
            parts.append(f"the matcher changed meaning since {since[:9]} (its tool counts {old_counts[name]} here)")
        elif left:
            parts.append(f"{left:+d} not attributable to a merge since {since[:9]} (the pin did not hold there)")
        trail = f"; {day} ({ref}) {pinned} -> {now} at {at}: " + (", ".join(parts) or "no merge moved it")
        pin_node, note_node = node.elts[3], node.elts[4]
        edits.append((pin_node.lineno, pin_node.col_offset, pin_node.end_col_offset, str(now)))
        edits.append((note_node.end_lineno, note_node.end_col_offset - 1, note_node.end_col_offset - 1, trail))
        print(f"{name}: {trail[2:]}")
    if not edits:
        print("published figures: every pin holds; nothing to bless.")
        return 0
    lines = src.split("\n")
    for ln, a, b, text in sorted(edits, reverse=True):
        s = lines[ln - 1]
        lines[ln - 1] = s[:a] + text + s[b:]
    open(path, "w").write("\n".join(lines))
    print(f"\n{len(edits) // 2} pin(s) blessed in tools/published_figures.py at {at}.")
    return 0


def self_check() -> int:
    """A comment line must not count, and a code line must. Without both, a
    file that reads nothing and a file that reads everything look alike."""
    import tempfile
    ok = True
    with tempfile.TemporaryDirectory() as d:
        p = os.path.join(d, "x.t27")
        open(p, "w").write("// cast_i8(1) in a comment\nvar a = cast_i8(2);\n")
        rx = re.compile(r"(?<![\w.@])cast_i8\s*\(")
        seen = sum(
            len(rx.findall(l))
            for l in open(p)
            if not l.lstrip().startswith("//")
        )
        print(f"  one in code, one in a comment -> counted {seen} "
              f"({'PASS' if seen == 1 else 'FAIL'})")
        ok &= seen == 1
    return 0 if ok else 2


def main() -> int:
    if "--self-check" in sys.argv:
        return self_check()
    try:
        if "--check" in sys.argv:
            return check()
        if "--bless" in sys.argv:
            return bless()
    except CouldNotRun as e:
        print(f"published_figures: COULD NOT RUN -- {e}. Exit 2.", file=sys.stderr)
        return 2
    got = derive()
    drift = 0
    print(f"{'figure':38} {'unit':13} {'pinned':>7} {'now':>7} {'specs':>6}  published as")
    for (name, unit, _, pinned, where), (now, nspecs) in zip(FIGURES, got):
        mark = "" if now == pinned else "  DRIFT"
        if now != pinned:
            drift += 1
        print(f"{name:38} {unit:13} {pinned:7} {now:7} {nspecs:6}  {where}{mark}")
    if drift:
        print(f"\n{drift} figure(s) drifted. --check says whether that is lag or a defect of the change.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
