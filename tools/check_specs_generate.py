#!/usr/bin/env python3
"""Does every .t27 spec still compile to at least one target?

The README says `.t27` specs in → Zig, Verilog, C out, and the constitution makes specs
the single source of truth. Measured on 2026-08-18:

    1114  specs tracked
     766  generate            68.8%
     348  do NOT generate     31.2%

Not one of the 348 is a backend mismatch. On a 25-spec random sample, **zero** generated
with any of gen-c / gen-rust / gen-verilog / gen-zig -- they fail in the parser, before a
backend is reached. That alternative was checked because "31% of the source of truth does
not compile" is an alarming claim, and an alarming claim is usually a fault in the
instrument (see .claude/skills/ci-gates/SKILL.md §7).

Where they are, and how they fail:

    specs/tri/     70     parse error at module level  120
    specs/scratch/ 58     parse error in fn            114
    specs/fpga/    35     Expected RBrace               38
    specs/igla/    15     Expected LBrace               36
    specs/numeric/ 15     unknown cast target           34
    ...                   Unexpected top-level token     4

How this was found, which matters for what to do next. `t27c seal <spec> --save`
re-seals a spec **that does not generate**, writing `gen_hash_rust=none`. So a stale seal
can be "fixed" into a seal that records reproducibility for output which does not exist.
Batch re-sealing the 113 stale seals would have blessed 46 non-generating specs that way,
and written them under new filenames besides, leaving the originals stale. Testing one
instead of the batch is what surfaced this.

The 348 are recorded in tools/specs_generate_baseline.txt as debt, one per line with its
first compiler message, so this gate holds the line without demanding they all be fixed.
The number can only go down.

Usage:
  tools/check_specs_generate.py                  gate
  tools/check_specs_generate.py --self-check     negative control
  tools/check_specs_generate.py --update-baseline
  tools/check_specs_generate.py --summary        counts by directory and error class

Exits non-zero if a spec that used to generate stops generating.
"""
import collections
import os
import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BASELINE = ROOT / "tools/specs_generate_baseline.txt"
# The Zig backend is `gen`, not `gen-zig` -- there is no gen-zig subcommand, so the
# first version of this list had a dead arm that always returned non-zero. It did not
# change the count (a spec passing any other backend still passed) but it meant the
# Zig backend never actually contributed a verdict.
BACKENDS = ("c", "rust", "verilog")
ZIG = "gen"


def t27c():
    for p in ("target/release/t27c", "target/debug/t27c"):
        c = ROOT / p
        if c.exists():
            return str(c)
    sys.exit("FAIL: t27c not built. Run: cargo build --release -p t27c")


# Inputs whose entire purpose is to be rejected. `git ls-files "*.t27"` cannot
# tell "must compile" from "must NOT compile", so it swept the parser's own
# negative fixtures into the compile-debt ledger: twelve deliberately damaged
# files, seven malformed generic-const declarations, two truncated-at-EOF
# hazards. Recording those as debt inverts their meaning -- the day one of them
# starts generating is the day a parser bug shipped, and the ledger would have
# read that as progress.
# T84: these three prefixes were one list under one rationale, and the
# rationale did not describe them. The comment enumerated 21 files -- "twelve
# deliberately damaged, seven malformed generic-const declarations, two
# truncated-at-EOF hazards" -- while the prefixes exclude 29. Measured: 21 fail
# to generate, 8 do not, and the 8 are two different kinds of thing.
#
# `terminator/` holds parser CONTROLS with their own assertions in
# bootstrap/tests/struct_body_terminator.rs -- several are meant to parse, and
# excluding them from a generate census is right.
CONTROL_FIXTURES = (
    "bootstrap/tests/fixtures/terminator/",
)

# These exist to be REJECTED. The comment that used to sit over all three said
# it out loud -- "the day one of them starts generating is the day a parser bug
# shipped" -- and nothing enforced it: a repo-wide grep for `damage_class` in
# tests and tools returns nothing at all. Three of them generate today, and the
# C they emit does not compile (`type name requires a specifier or qualifier`).
# So the alarm has been ringing, unread, into an empty room.
MUST_NOT_GENERATE = (
    "bootstrap/tests/fixtures/damage/",
    "bootstrap/tests/fixtures/generic_const/neg_",
)

# The three already generating, frozen as named debt so master stays green and
# the class cannot grow. Removing a line when the parser rejects it again is
# the ratchet; adding one has to be a hand edit with a reason.
GENERATING_DAMAGE_DEBT = {
    "bootstrap/tests/fixtures/damage/damage_class_03.t27",
    "bootstrap/tests/fixtures/damage/damage_class_04.t27",
    "bootstrap/tests/fixtures/damage/damage_class_12.t27",
}

NEGATIVE_FIXTURES = CONTROL_FIXTURES + MUST_NOT_GENERATE


def specs():
    r = subprocess.run(["git", "ls-files", "*.t27"], cwd=ROOT, capture_output=True, text=True)
    return sorted(
        x for x in r.stdout.split()
        if x and not x.startswith(NEGATIVE_FIXTURES)
    )


def generates(t, sp):
    """(ok, first message). ok if ANY backend accepts it -- a spec written for one
    target should not be reported as broken because another target rejects it."""
    first = ""
    for cmd in [["gen-" + m] for m in BACKENDS] + [[ZIG]]:
        r = subprocess.run([t] + cmd + [sp], capture_output=True, text=True, cwd=ROOT)
        if r.returncode == 0:
            return True, ""
        if not first:
            first = (r.stderr or r.stdout or "").strip().split("\n")[0][:150]
    return False, first


def baseline():
    if not BASELINE.exists():
        return set()
    return {l.split("|")[0].strip() for l in BASELINE.read_text().splitlines()
            if l.strip() and not l.startswith("#")}


# One spec the compiler accepts and one it does not, named once. Both the
# in-process probe and every planted tree below use these, so "a spec that
# generates" cannot come to mean two different files in one control.
SPEC_OK = "module G;\nfn f(a: u8) -> u8 { return a; }\n"
SPEC_BROKEN = "module B;\nfn f(a: u8) -> u8 { return a\n"   # missing ; and }

PLANT_TIMEOUT = 300


def _self_check_plant(td, files, baseline_lines=None, args=()):
    """Build a tree this gate can be RUN in, run it there, return the process.

    HOW THE GATE IS AIMED AT THE PLANTED TREE: the script is COPIED into it, so
    its module-level ROOT resolves there by the ordinary parent.parent rule --
    the same trick tools/check_gate_preconditions.py uses, and for the same
    reason: no --root flag and no environment override, so nothing here adds a
    way to aim the LIVE gate at somewhere harmless.

    `files` maps repo-relative path -> body; every .t27 among them is `git
    add`ed, because specs() reads `git ls-files "*.t27"` and a file merely on
    disk is invisible to it. That difference is not a detail of the planting --
    it IS the DEPARTED branch, which exists because a spec leaving the index
    used to read as a spec that started compiling.

    `baseline_lines` writes tools/specs_generate_baseline.txt; None leaves the
    file absent, which is a different world (baseline() returns an empty set
    either way, but --update-baseline's `prior` is what distinguishes them).
    """
    root = pathlib.Path(td)
    me = pathlib.Path(__file__).resolve()
    (root / "tools").mkdir(parents=True)
    shutil.copy(me, root / "tools" / me.name)
    (root / "target/release").mkdir(parents=True)
    shutil.copy(t27c(), root / "target/release/t27c")
    for rel, body in files.items():
        p = root / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(body, encoding="utf-8")
    if baseline_lines is not None:
        (root / "tools/specs_generate_baseline.txt").write_text(
            "# planted by --self-check\n" + "".join(l + "\n" for l in baseline_lines),
            encoding="utf-8")
    # A private git identity: a global core.excludesFile or init.templateDir
    # would otherwise decide what this tree contains.
    env = dict(os.environ, GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_SYSTEM=os.devnull)
    subprocess.run(["git", "init", "-q"], cwd=root, env=env, check=True,
                   capture_output=True, text=True)
    subprocess.run(["git", "add", "-f", "--"] + [f for f in files if f.endswith(".t27")],
                   cwd=root, env=env, check=True, capture_output=True, text=True)
    return subprocess.run([sys.executable, str(root / "tools" / me.name)] + list(args),
                          capture_output=True, text=True, cwd=root, timeout=PLANT_TIMEOUT)


# NOT COVERED BY THIS FILE, said out loud rather than left to be inferred from
# a green:
#
#   main()  "git ls-files found no .t27 at all -- the scan is broken, not the
#           tree". Reaching it needs a tree with a built t27c and no specs,
#           which is exactly the world tools/check_gate_preconditions.py plants;
#           that file names this gate twice in its GATES table and `tri gates
#           mutate` runs it as this gate's external control. A second copy here
#           would certify the copy.
#   t27c()  `sys.exit("FAIL: t27c not built")` is not a `return`, so the mutant
#           scanner never offers it as a site. check_gate_preconditions.py
#           covers it anyway, on its BARE stage.


def self_check():
    """A spec with a deliberate syntax error must be reported; a good one must not."""
    import tempfile
    t = t27c()
    with tempfile.TemporaryDirectory() as td:
        good = os.path.join(td, "good.t27")
        bad = os.path.join(td, "bad.t27")
        open(good, "w").write(SPEC_OK)
        open(bad, "w").write(SPEC_BROKEN)
        g_ok, _ = generates(t, good)
        b_ok, msg = generates(t, bad)
    ok = g_ok and not b_ok
    print(f"  self-check: valid spec generates = {g_ok}, broken spec reported = {not b_ok}")
    if not b_ok and msg:
        print(f"              reported: {msg[:90]}")

    # The probe above proves generates(). It proves nothing about the four
    # branches that carry this gate's verdicts, all of which live in main() --
    # and a verdict is only a verdict once it reaches the process exit code.
    # Measured elsewhere in this tree: check_catalog_integrity.py with main()'s
    # `return 1` rewritten to `return 0` printed OK on a broken catalog while
    # its own control still reported every branch red.
    #
    # So run the WHOLE program, once per branch, in a tree planted for it.
    #
    # Why the exit code is asserted and not only the text: all four verdicts
    # print their whole explanation BEFORE the `return 1` that carries them, so
    # a gate whose verdict has been rewritten to `return 0` prints byte for byte
    # what the healthy gate prints and then exits green. Measured on all four
    # sites: the expected message was present every time and the exit code was
    # the only thing that moved. Nothing in the OUTPUT separates a neutered
    # verdict from a live one -- which is exactly how these four survived a
    # control that never left the process.
    #
    # `absent` does the other job, and it is not the same job: it names the
    # markers of the SIBLING branches, so a planted world that reds through the
    # wrong branch cannot pass for a world that reds through the planted one.
    # DEPARTED and LEAKED are the pair that needs it. They are two `return 1`s
    # three lines apart, LEAKED is tested first, and a tree that triggered both
    # would score the DEPARTED mutant as killed when what answered was LEAKED.
    def spawned(label, want, expect_text, absent, **kw):
        nonlocal ok
        with tempfile.TemporaryDirectory() as td:
            r = _self_check_plant(td, **kw)
        out = r.stdout + r.stderr
        said = expect_text in out
        leaked = [a for a in absent if a in out]
        good = r.returncode == want and said and not leaked
        print("  %-30s %s" % (label, "exit %d, right branch" % want if good
                              else "CONTROL FAILED"))
        if not good:
            ok = False
            print("       exit %r (want %r); %r present: %s"
                  % (r.returncode, want, expect_text, said))
            if leaked:
                print("       neighbouring marker leaked: %r" % (leaked,))
            print("       said   %s"
                  % ((out.strip().splitlines() or ["(nothing)"])[0][:110],))

    GREEN, NEW, DEP, LEAK, BLESS = "OK:", "FAIL:", "DEPARTED", "LEAKED", "REFUSING"

    # The clean tree first. Without it every case below could be passing on a
    # tree that reds for a reason nobody planted.
    spawned("clean tree", 0, "OK: 1 specs, 1 generate",
            (NEW, DEP, LEAK, BLESS),
            files={"specs/ok.t27": SPEC_OK}, baseline_lines=[])

    # A tracked spec that does not generate and is not in the ledger. The
    # gate's headline verdict, and the only one of the four whose alarm text
    # would also vanish if the branch were deleted outright.
    spawned("newly broken spec", 1, "FAIL: 1 spec(s) newly do not generate",
            (GREEN, DEP, LEAK, BLESS),
            files={"specs/ok.t27": SPEC_OK, "specs/new.t27": SPEC_BROKEN},
            baseline_lines=[])

    # T69's class: a ledger line whose spec left `git ls-files`. Nothing on
    # disk was repaired and nothing generates that did not before -- the
    # measured set simply got smaller, which used to print a congratulation.
    spawned("baseline spec untracked", 1, "DEPARTED 1 spec(s) in the baseline",
            (GREEN, NEW, LEAK, BLESS),
            files={"specs/ok.t27": SPEC_OK},
            baseline_lines=["specs/gone.t27 | parse error at module level"])

    # T84's class: a fixture whose whole purpose is to be REJECTED, generating.
    # Checked BEFORE the departed branch, so this tree carries no departed
    # ledger line: with one, a neutered LEAKED return would fall into the
    # DEPARTED return and the mutant would read as killed by the wrong branch.
    spawned("damaged fixture generates", 1, "LEAKED 1 damaged fixture(s) now GENERATE",
            (GREEN, NEW, DEP, BLESS),
            files={"specs/ok.t27": SPEC_OK,
                   "bootstrap/tests/fixtures/damage/damage_class_99.t27": SPEC_OK},
            baseline_lines=[])

    # T77's class: --update-baseline is the documented blessing command, so it
    # is the one command that must refuse to bless growth. Two specs fail
    # against a one-line ledger. "baseline written" is named as absent because
    # the ledger being rewritten is the exact damage, and it is the only marker
    # that separates a refusal from a blessing.
    spawned("blessing command grows ledger", 1,
            "REFUSING to grow the ledger: 1 -> 2 entries",
            (GREEN, NEW, DEP, LEAK, "baseline written"),
            files={"specs/ok.t27": SPEC_OK,
                   "specs/known.t27": SPEC_BROKEN,
                   "specs/new.t27": SPEC_BROKEN},
            baseline_lines=["specs/known.t27 | parse error in fn"],
            args=("--update-baseline",))

    # T100: the same command's SUCCESS path, which nothing asserted. The case
    # above proves --update-baseline refuses to GROW the ledger; nothing proved
    # that a legitimate blessing writes it and returns 0. `tri gates mutate
    # --loud` rewrote that success return to a failure and no assertion here
    # noticed -- the same site survived in four gates.
    #
    # The ledger SHRINKS here (two recorded, one still broken), which is the
    # direction the gate permits. Exit and effect are asserted together: exit
    # alone would pass a run that returned 0 without writing, and the marker
    # alone would pass one that wrote and then reported failure.
    spawned("blessing command shrinks ledger", 0, "baseline written: 1 entries",
            (GREEN, NEW, DEP, LEAK, BLESS),
            files={"specs/ok.t27": SPEC_OK, "specs/known.t27": SPEC_BROKEN},
            baseline_lines=["specs/known.t27 | parse error in fn",
                            "specs/fixed.t27 | parse error in fn"],
            args=("--update-baseline",))

    # T101: --summary is a REPORT mode, not a gate, and its success return was
    # unasserted like the four ledger writers before it. A report that prints
    # its table and then reports failure would break any script reading it, and
    # nothing here would have said so. The distinction between a report and a
    # gate is worth keeping; leaving the report's exit code unmeasured is not
    # part of it.
    spawned("report mode --summary", 0, "specs, ",
            (NEW, DEP, LEAK, BLESS),
            files={"specs/ok.t27": SPEC_OK, "specs/known.t27": SPEC_BROKEN},
            baseline_lines=["specs/known.t27 | parse error in fn"],
            args=("--summary",))

    print("  self-check: %s" % ("every verdict reaches the exit code"
                                if ok else "FAILED"))
    return 0 if ok else 1


def main():
    t = t27c()
    if "--self-check" in sys.argv:
        return self_check()

    all_specs = specs()
    if not all_specs:
        print("FAIL: git ls-files found no .t27 at all -- the scan is broken, not the tree")
        return 1
    bad = []
    for sp in all_specs:
        ok, msg = generates(t, sp)
        if not ok:
            bad.append((sp, msg))

    if "--summary" in sys.argv:
        print(f"  {len(all_specs)} specs, {len(all_specs)-len(bad)} generate "
              f"({100*(len(all_specs)-len(bad))/len(all_specs):.1f}%), {len(bad)} do not\n")
        print("  by directory:")
        # T84: this took path component [1] and glued "specs/" back on, so
        # rows merged trees that share a second component and invented labels
        # for the 17 of 171 entries that are not under specs/ at all.
        # Measured: "specs/runtime/ 5" was 3 under specs/runtime plus
        # compiler/runtime/{commands,validation}.t27 -- a count true of neither
        # directory it names. Print the directory that exists.
        for d, c in collections.Counter(
                sp.rsplit("/", 1)[0] for sp, _ in bad).most_common(12):
            print(f"    {c:>4}  {d}/")
        print("\n  by error class:")
        def cls(m):
            for k in ("unknown cast target", "parse error at module level",
                      "Unexpected top-level token", "Expected LBrace", "Expected RBrace",
                      "parse error in fn"):
                if k in m:
                    return k
            return m[:44]
        for k, c in collections.Counter(cls(m) for _, m in bad).most_common(10):
            print(f"    {c:>4}  {k}")
        return 0

    if "--update-baseline" in sys.argv:
        # T77: the cap only ever moves DOWN. This command is the documented way
        # to bless a change, and it rewrote the ledger unconditionally -- so the
        # debt list could grow as a SIDE EFFECT of the very command the gate
        # recommends, while the docstring asserts the number can only fall.
        # bootstrap/src/suite.rs:2634 already does it right for the corpus
        # ratchet, with the same words: "raising the cap must be a hand edit in
        # the pull request, never a side effect of running the blessing
        # command." Same rule, same file format, now the same behaviour.
        prior = len(baseline())
        if prior and len(bad) > prior:
            print(f"REFUSING to grow the ledger: {prior} -> {len(bad)} entries.")
            print()
            print("  These specs newly fail to generate and are not in the ledger:")
            for sp, msg in [(sp, m) for sp, m in bad if sp not in baseline()][:10]:
                print(f"    {sp}\n      {msg[:110]}")
            print()
            print("  Fix them, or add their lines BY HAND in the same pull request")
            print("  with a reason. A ledger that grows when you run the blessing")
            print("  command is a ledger that records nothing.")
            return 1
        BASELINE.write_text(
            "# Specs that do not generate with ANY backend. Each line is a debt.\n"
            "# Remove the line when the spec compiles; the gate then holds it compiling.\n"
            + "".join(f"{sp} | {msg}\n" for sp, msg in bad))
        print(f"  baseline written: {len(bad)} entries")
        return 0

    known = baseline()
    new = [(sp, m) for sp, m in bad if sp not in known]

    # T69: a spec that LEAVES the index has not been repaired. `bad` is built
    # only from `git ls-files "*.t27"`, so `known - bad` folded two very
    # different events into one congratulation: a spec that started compiling,
    # and a spec that stopped being tracked while still failing on disk.
    # Untracking three debt specs printed "NOTE 3 spec(s) in the baseline now
    # generate" and exited 0. Commit 2255e4c32 removed 58 ledger lines with 455
    # deletions and 0 modifications to the specs themselves -- its own message
    # concedes "those 58 would read as fixed". Departure is now its own class,
    # and it fails.
    tracked = set(all_specs)
    departed = sorted(known - tracked)
    fixed = sorted((known & tracked) - {sp for sp, _ in bad})
    if departed:
        print(f"DEPARTED {len(departed)} spec(s) in the baseline are no longer tracked.")
        print("They did not start generating -- they left the measured set, which")
        print("reads as progress in the count below and is not. Drop their ledger")
        print("lines in the same commit that removes them, deliberately:")
        for sp in departed[:10]:
            print(f"  {sp}")
        if len(departed) > 10:
            print(f"  ... and {len(departed) - 10} more")
        print()
    if fixed:
        print(f"NOTE {len(fixed)} spec(s) in the baseline now generate. Remove them from "
              f"{BASELINE.name} so the gate holds them:")
        for sp in fixed[:10]:
            print(f"  {sp}")
        print()
    # T84: an input whose purpose is to be rejected, and is not.
    leaked = []
    _tracked = subprocess.run(["git", "ls-files", "*.t27"], cwd=ROOT,
                              capture_output=True, text=True).stdout.split()
    for sp in sorted(f for f in _tracked
                     if any(f.startswith(x) for x in MUST_NOT_GENERATE)):
        ok, _msg = generates(t, sp)
        if ok and sp not in GENERATING_DAMAGE_DEBT:
            leaked.append(sp)
    if leaked:
        print(f"LEAKED {len(leaked)} damaged fixture(s) now GENERATE:")
        for sp in leaked:
            print(f"  {sp}")
        print()
        print("  These files exist to be rejected. One of them generating is a")
        print("  parser accepting input it was built to refuse -- the comment over")
        print("  the list has said so since it was written, and nothing enforced it.")
        print("  If deliberate, add the path to GENERATING_DAMAGE_DEBT with a reason.")
        return 1

    if departed:
        return 1
    if not new:
        print(f"OK: {len(all_specs)} specs, {len(all_specs)-len(bad)} generate, "
              f"{len(bad)} known-broken in {BASELINE.name}")
        return 0
    print(f"FAIL: {len(new)} spec(s) newly do not generate with any backend\n")
    for sp, m in new:
        print(f"  {sp}\n      {m}")
    print("\n  The message is the compiler's own. A spec that does not generate is not a")
    print("  source of truth for anything, and t27c seal --save will still seal it with")
    print("  gen_hash=none -- so this must fail rather than be sealed over.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
