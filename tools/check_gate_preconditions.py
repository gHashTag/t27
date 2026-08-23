#!/usr/bin/env python3
"""Every gate must fail loudly when it cannot do its work.

A gate's own `--self-check` plants a fault INSIDE a well-formed world: a row
that points nowhere, a file that lost its data, a count that drifted. It never
breaks the world's EXISTENCE -- the baseline that is not there, the compiler
that was not built, the scan that matched nothing. Those branches are one
`return` away from turning a gate into a silent pass that announces itself:

    no baseline; run --update-baseline once      <- printed
    exit 0                                       <- and green

`tri gates mutate` found that class across six gates. This is the one control
that covers it for all of them, rather than six bespoke cases.

THE PLANTED WORLD IS EMPTY, and that is the whole trick. Every precondition
these gates have -- a baseline file, a built t27c, a tree with specs in it,
seals on disk -- is absent from an empty directory at once. One planting, six
gates, and any gate added to the table below comes along for free.

HOW THE GATE IS AIMED AT THE EMPTY TREE: the script is COPIED into it, so its
module-level ROOT resolves there by the ordinary parent.parent rule. No --root
flag and no environment override, so nothing here adds a way to aim a live
gate at somewhere harmless. (Three of these six do accept a T27_*_ROOT
override; using it would have worked for those three and not for the other
three, and would have made the control depend on a lever that exists for
testing rather than on the gate's ordinary behaviour.)

  tools/check_gate_preconditions.py                gate
  tools/check_gate_preconditions.py --self-check   negative control
"""
import os
import pathlib
import shutil
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _prereq import plant  # noqa: E402
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
T27C = ROOT / "target/release/t27c"

# (script, stage, a string its precondition branch prints). The message is
# asserted, not just the exit code: several of these gates reach a non-zero
# exit from many branches, and "it went red" does not say it went red for this
# reason. A gate that failed on an empty tree by CRASHING would satisfy an
# exit-code-only check and satisfy nothing anyone wanted.
#
# Two stages, because preconditions are checked in order and the first one to
# fire hides the rest. `bare` is the empty tree. `t27c` additionally copies the
# built compiler in, which lets the gates get far enough to reach their second
# precondition -- the "the instrument is broken, not the tree" branches, which
# are the ones worth having. Written from a measurement: the first version of
# this table expected check_specs_generate.py to say "found no .t27 at all" on
# a bare tree. It says "t27c not built", and this file's own control is what
# said so.
BARE, WITH_T27C, WITH_IVERILOG = "bare", "t27c", "t27c+iverilog"
# T106: the fourth stage, and the one the note below said could not exist.
# The tree is EMPTY of the artefact under test and the CORPUS is real: the
# child runs with cwd=ROOT, so `t27c fpga-build --smoke` resolves the real
# specs and succeeds, while T27_ELAB_ROOT points at a planted tree holding a
# built t27c, an empty generated/ and no baseline. Nothing in the real tree
# is written or moved -- verified by git status before and after.
REAL_CORPUS = "real-corpus"
GATES = [
    ("check_duplicate_agreement.py", BARE, "t27c not built"),
    ("check_duplicate_agreement.py", WITH_T27C, "no duplicated function was found at all"),
    ("check_elab_ratchet.py", BARE, "iverilog or target/release/t27c missing"),
    ("check_seal_coverage.py", BARE, "no seals found at all"),
    ("check_specs_generate.py", BARE, "t27c not built"),
    ("check_specs_generate.py", WITH_T27C, "found no .t27 at all"),
    ("check_specs_parse.py", BARE, "t27c not built"),
    ("check_vector_data.py", BARE, "no baseline"),
    # T95: the third stage. This one needs a tool that cannot be planted --
    # iverilog lives on PATH, not in the tree -- so it reports UNRUN when the
    # tool is absent rather than passing or guessing. Same choice the WITH_T27C
    # rows make: a row that cannot run proved nothing, and reporting it as
    # absent would be the vacuous pass this file exists to catch, one level up.
    ("check_elab_ratchet.py", WITH_IVERILOG, "t27c fpga-build --smoke failed"),
    ("check_elab_ratchet.py", REAL_CORPUS, "no baseline"),
]

# NOTHING IS UNCOVERED HERE ANY MORE, and the reason the last one stood for a
# week is worth keeping.
#
# The note that lived here read: "Reaching it needs `t27c fpga-build --smoke` to
# SUCCEED and then find no baseline, and a smoke build that succeeds needs the
# real spec tree -- not something an empty directory can be given. Planting a
# fake success would test the plant, not the gate."
#
# Every clause of that is TRUE. The conclusion does not follow. A control does
# not have to use an empty directory -- the note reasoned entirely inside the
# frame of run_on_empty_tree(), the helper this file happens to be built around,
# and never asked whether a stage could keep the real corpus and empty only what
# it is testing. Splitting cwd from T27_ELAB_ROOT does exactly that, and the
# branch turned out to be reachable in under a second.
#
# The tell was that the justification sounded mechanical and no measurement had
# produced it (number-audit 8.6). One command falsified it. So: for every
# limitation written down here, name the experiment that would show it is not a
# limitation, and run it -- a reason you cannot falsify in one command is a
# hypothesis wearing a fact's clothes.
#
# Kept at 0 rather than deleted: the count is asserted in main()'s OK line, and
# a reader who sees the number go 1 -> 0 learns something a removed line cannot
# tell them.
UNCOVERED = 0

TIMEOUT = 300


def run_on_empty_tree(script, stage=BARE, source=None):
    """Copy `script` into an empty tree and run it there. Returns the result."""
    with tempfile.TemporaryDirectory() as td:
        tree = pathlib.Path(td)
        (tree / "tools").mkdir()
        # Plant unconditionally: the mutated-source branch needs the same
        # siblings, and writing only the mutant left the copy dying on
        # ImportError -- scored WRONG ("red, but not through the branch
        # that explains why") when the case wanted VACUOUS.
            # plant(), not copy(): a gate that imports a sibling dies on
            # ImportError here, and a traceback is scored WRONG -- "it went
            # red, but not through the branch that explains why". The
            # verdict would be about this planting, and would name the
            # gate.
        plant(ROOT / "tools" / script, tree / "tools")
        if source is not None:
            (tree / "tools" / script).write_text(source, encoding="utf-8")
        if stage in (WITH_T27C, WITH_IVERILOG):
            (tree / "target/release").mkdir(parents=True)
            shutil.copy(T27C, tree / "target/release/t27c")
        return subprocess.run(
            [sys.executable, str(tree / "tools" / script)],
            capture_output=True, text=True, cwd=td, timeout=TIMEOUT,
        )


def smoke_builds():
    """Does `t27c fpga-build --smoke` succeed at ROOT? Cached: the REAL_CORPUS
    rows all ask, and the answer cannot change inside one run."""
    if not hasattr(smoke_builds, "_v"):
        try:
            smoke_builds._v = subprocess.run(
                [str(T27C), "fpga-build", "--smoke"], capture_output=True,
                text=True, cwd=str(ROOT), timeout=TIMEOUT).returncode == 0
        except Exception:
            smoke_builds._v = False
    return smoke_builds._v


def run_with_real_corpus(script, source=None):
    """Real corpus, planted ROOT. The inverse of run_on_empty_tree.

    cwd is the real repository so `t27c fpga-build --smoke` finds real specs and
    succeeds; T27_ELAB_ROOT is a planted tree carrying a built t27c, an empty
    generated/ and NO baseline, so the gate gets past the smoke build and then
    finds nothing to compare against. The real tree is only read.
    """
    with tempfile.TemporaryDirectory() as td:
        tree = pathlib.Path(td)
        (tree / "target/release").mkdir(parents=True)
        shutil.copy(T27C, tree / "target/release/t27c")
        # Present and empty. Absent would make counts() raise, and a crash is
        # not the branch -- it is the WRONG verdict this file exists to name.
        (tree / "build/fpga/generated").mkdir(parents=True)
        (tree / "tools").mkdir()
        if source is None:
            target = ROOT / "tools" / script
        else:
            # Plant the siblings first, then overwrite the script with the
            # mutant. Writing only the mutant left the copy importing a module
            # that was not there, so the case that wanted VACUOUS got a
            # traceback and was scored WRONG -- a verdict about this planting,
            # wearing the gate's name.
            plant(ROOT / "tools" / script, tree / "tools")
            target = tree / "tools" / script
            target.write_text(source, encoding="utf-8")
        return subprocess.run(
            [sys.executable, str(target)],
            capture_output=True, text=True, cwd=str(ROOT), timeout=TIMEOUT,
            env={**os.environ, "T27_ELAB_ROOT": str(tree)},
        )


EXPLAIN = {
    "VACUOUS": "exits 0 with nothing to check",
    "WRONG": "went red, but not through the branch that explains why",
}


def first_line(r):
    out = (r.stdout + r.stderr).strip()
    return out.splitlines()[0][:88] if out else "(nothing)"


def classify(r, want):
    """The single verdict rule. None means the precondition was loud and right.

    The control calls THIS, not a second copy of it: a control that
    re-implements the rule certifies the re-implementation, and mutating the
    real rule leaves the copy agreeing with itself.
    """
    if r.returncode == 0:
        return "VACUOUS"
    if want not in (r.stdout + r.stderr):
        return "WRONG"
    return None


def bare_plants():
    """Files that copy a script into a planted tree without its imports.

    Measured before this existed, by injecting one unused sibling import into
    each self-planting gate and re-running its own self-check in the real tree:

        check_specs_parse         5 controls broken
        check_catalog_integrity   4 controls broken
        check_gate_preconditions  2 controls broken

    Eleven controls, every one of them reporting `stdout ''` and "expected text
    absent" -- which reads as a broken gate on a day when only the plant was
    broken. `plant()` copies the script together with the siblings it imports;
    a bare `shutil.copy` of the script alone is the defect, and it is invisible
    until the day someone adds an import.
    """
    out = []
    for f in sorted((ROOT / "tools").glob("*.py")):
        text = f.read_text(errors="replace")
        for i, line in enumerate(text.splitlines(), 1):
            t = line.strip()
            if not t.startswith("shutil.copy("):
                continue
            # Only copies whose DESTINATION is a planted tools/ directory. A
            # copy of a binary into target/release, or of a fixture, is not a
            # plant and needs no imports.
            # Both spellings of the destination. The first version of this
            # matched only `/ "tools"` as a separate component and missed
            # `t / "tools/check_withdrawn_live.py"`, where the directory is
            # inside one string -- a detector keyed on a shape finding the
            # instances that share the shape, for the third time in this
            # campaign. Caught when adding an import to that very file broke
            # four of its own controls.
            # Keyed on BOTH ends. Destination alone was wrong twice, in
            # opposite directions: `/ "tools"` as a separate component missed
            # `t / "tools/withdrawn.txt"`, and matching `tools/` anywhere then
            # flagged that same line -- which copies a DATA file and needs no
            # imports. A plant is a copy of a SCRIPT into a planted tools/.
            dest_planted = 'tools/' in t or '/ "tools"' in t or "/ 'tools'" in t
            copies_script = "__file__" in t or '.py"' in t or ".py'" in t
            if dest_planted and copies_script:
                out.append(f"{f.name}:{i}  {t[:64]}")
    return out


def check():
    """Returns a list of problems. Empty means every precondition is loud."""
    problems = []

    for hit in bare_plants():
        problems.append(
            f"BARE      {hit}\n"
            f"            plants a script without the siblings it imports; use\n"
            f"            plant() from _prereq, or the controls below go silent\n"
            f"            the day that script gains an import"
        )

    for script, stage, want in GATES:
        if not (ROOT / "tools" / script).exists():
            problems.append(f"GONE      tools/{script} is in the table and not on disk")
            continue
        if stage in (WITH_IVERILOG, REAL_CORPUS) and not shutil.which("iverilog"):
            problems.append(
                f"UNRUN     {script} [{stage}] needs iverilog on PATH\n"
                f"            it is not in the tree and cannot be planted"
            )
            continue
        if stage in (WITH_T27C, WITH_IVERILOG, REAL_CORPUS) and not T27C.exists():
            # Not a pass. A row that cannot run is a row that proved nothing,
            # and reporting it as absent would be the vacuous pass this file
            # exists to catch, one level up.
            problems.append(
                f"UNRUN     {script} [{stage}] needs target/release/t27c\n"
                f"            build it: cargo build --release -p t27c"
            )
            continue
        if stage == REAL_CORPUS and not smoke_builds():
            # Same doctrine as the rows above: a stage that cannot run proved
            # nothing, and calling that a pass would be the vacuous green this
            # file exists to catch. It is also what a copied tree looks like --
            # the end-to-end cases plant a corpus precisely so this does not
            # fire there and hide the wiring they measure.
            problems.append(
                f"UNRUN     {script} [{stage}] needs a spec corpus at ROOT\n"
                f"            `t27c fpga-build --smoke` does not succeed there"
            )
            continue
        try:
            if stage == REAL_CORPUS:
                r = run_with_real_corpus(script)
            else:
                r = run_on_empty_tree(script, stage)
        except subprocess.TimeoutExpired:
            problems.append(f"HUNG      {script} [{stage}] did not finish")
            continue
        verdict = classify(r, want)
        if verdict:
            problems.append(f"{verdict:<9} {script} [{stage}] {EXPLAIN[verdict]}\n"
                            f"            wanted {want!r}\n"
                            f"            said:  {first_line(r)}")
    return problems


# --- negative control -------------------------------------------------------
#
# T89: the recursion has to stop somewhere, but not before this file has been
# shown to go red. Both directions are planted, because a control that only
# proves it can fail may be failing for the wrong reason:
#
#   VACUOUS -- a gate rewritten to return 0 when its precondition is missing.
#              This is the exact defect the file exists to catch, and it was
#              a LIVE one: check_elab_ratchet.py printed
#              "SKIP: iverilog or target/release/t27c missing" and exited 0.
#   WRONG   -- a gate that reds on an empty tree with a different message. An
#              exit-code-only control passes this and proves nothing.
def self_check():
    ok = True

    def case(label, mutate, want_marker, absent):
        nonlocal ok
        script = "check_vector_data.py"
        src = (ROOT / "tools" / script).read_text(encoding="utf-8")
        # Through the SAME planting the gate uses, so the control certifies the
        # real runner rather than a second copy of it.
        r = run_on_empty_tree(script, BARE, source=mutate(src))
        got = classify(r, "no baseline") or "(silent)"
        good = got == want_marker and got != absent
        print("  %-28s %s" % (label, "classified %s" % got if good else "CONTROL FAILED"))
        if not good:
            ok = False
            print("       wanted %s, got %s; exit %r" % (want_marker, got, r.returncode))
            print("       said   %s" % (first_line(r),))

    # A gate that swallows its own missing precondition.
    case(
        "gate returns 0 when blind",
        lambda s: s.replace(
            'print("no baseline; run --update-baseline once")',
            'print("no baseline; run --update-baseline once")\n        return 0  # planted',
            1,
        ),
        "VACUOUS",
        "WRONG",
    )
    # A gate that reds for some other reason. Asserted separately because the
    # exit code is identical to a correct red: only the message separates them,
    # which is the reason this file asserts messages at all.
    case(
        "gate reds without saying why",
        lambda s: s.replace(
            'print("no baseline; run --update-baseline once")',
            'print("something else entirely")',
            1,
        ),
        "WRONG",
        "VACUOUS",
    )

    # T106: the same two directions for the REAL_CORPUS stage. The two cases
    # above certify run_on_empty_tree; this stage uses a different runner, and a
    # runner that cannot make its row go red is the vacuous pass one level up --
    # which is the whole subject of this file. The victim is the real gate whose
    # branch the stage exists to reach.
    def real_corpus_case(label, mutate, want_marker, absent):
        nonlocal ok
        script = "check_elab_ratchet.py"
        src = (ROOT / "tools" / script).read_text(encoding="utf-8")
        r = run_with_real_corpus(script, source=mutate(src))
        got = classify(r, "no baseline") or "(silent)"
        good = got == want_marker and got != absent
        print("  %-28s %s" % (label, "classified %s" % got if good else "CONTROL FAILED"))
        if not good:
            ok = False
            print("       wanted %s, got %s; exit %r" % (want_marker, got, r.returncode))
            print("       said   %s" % (first_line(r),))

    real_corpus_case(
        "real-corpus: swallows it",
        lambda s: s.replace(
            'print("no baseline; run --update-baseline once")\n        return 1',
            'print("no baseline; run --update-baseline once")\n        return 0',
            1,
        ),
        "VACUOUS",
        "WRONG",
    )
    real_corpus_case(
        "real-corpus: reds mutely",
        lambda s: s.replace(
            'print("no baseline; run --update-baseline once")',
            'print("something else entirely")',
            1,
        ),
        "WRONG",
        "VACUOUS",
    )

    # Both cases above prove classify(). Neither proves that a non-empty
    # problem list becomes a non-zero exit code AND says so -- and this file's
    # own WRONG class is precisely "went red without explaining". Measured:
    # deleting main()'s FAIL line left every case above passing.
    #
    # So run the WHOLE program in a tree of its own: this file plus the gates
    # it names, with one of them neutered. ROOT resolves there by parent.parent
    # because the script is copied in, same as everywhere else here.
    with tempfile.TemporaryDirectory() as td:
        tree = pathlib.Path(td)
        (tree / "tools").mkdir()
        me = pathlib.Path(__file__).name
        plant(__file__, tree / "tools")
        for script, _, _ in GATES:
            # plant(), not copy(): a gate that imports a sibling dies on
            # ImportError here, and a traceback is scored WRONG -- "it went
            # red, but not through the branch that explains why". The
            # verdict would be about this planting, and would name the
            # gate.
            plant(ROOT / "tools" / script, tree / "tools")
        victim = tree / "tools" / "check_vector_data.py"
        victim.write_text(
            victim.read_text(encoding="utf-8").replace(
                'print("no baseline; run --update-baseline once")',
                'print("no baseline; run --update-baseline once")\n        return 0  # planted',
                1,
            ),
            encoding="utf-8",
        )
        if T27C.exists():
            (tree / "target/release").mkdir(parents=True)
            shutil.copy(T27C, tree / "target/release/t27c")
        # T106: and a real corpus, 1.2 MB of it. The REAL_CORPUS row needs
        # `t27c fpga-build --smoke` to SUCCEED at the tree root; without specs
        # the row reports UNRUN and both end-to-end cases below would be
        # measuring the absence of a corpus rather than the wiring they exist
        # to measure.
        if (ROOT / "specs/fpga").is_dir():
            shutil.copytree(ROOT / "specs/fpga", tree / "specs/fpga")
        r = subprocess.run([sys.executable, str(tree / "tools" / me)],
                           capture_output=True, text=True, cwd=td, timeout=TIMEOUT)
    said = "FAIL: 1 gate(s) do not fail loudly" in r.stdout
    e2e = r.returncode == 1 and said
    print("  %-28s %s" % ("end-to-end, one gate blind",
                          "exit 1, says which" if e2e else "CONTROL FAILED"))
    if not e2e:
        ok = False
        print("       exit %r (want 1); named the count: %s" % (r.returncode, said))
        print("       said   %s" % (first_line(r),))

    # T100: the SUCCESS wiring, and it is the mirror of the case above.
    # `tri gates mutate --loud` rewrote this file's own success return to a
    # failure and NOTHING noticed: the gate printed "OK: 9 precondition(s)..."
    # and exited 1 on a clean tree. Every mutation this campaign made until
    # then turned a failure into a pass, so the instrument and the blind spot
    # shared a direction -- and the file that enforces this discipline for six
    # other gates was the worst offender.
    #
    # Same tree, no victim: every gate healthy, so the program must say OK and
    # exit 0. The count is asserted with it, because an exit 0 from a table
    # that silently emptied would satisfy the code alone.
    with tempfile.TemporaryDirectory() as td:
        tree = pathlib.Path(td)
        (tree / "tools").mkdir()
        me = pathlib.Path(__file__).name
        plant(__file__, tree / "tools")
        for script, _, _ in GATES:
            # plant(), not copy(): a gate that imports a sibling dies on
            # ImportError here, and a traceback is scored WRONG -- "it went
            # red, but not through the branch that explains why". The
            # verdict would be about this planting, and would name the
            # gate.
            plant(ROOT / "tools" / script, tree / "tools")
        if T27C.exists():
            (tree / "target/release").mkdir(parents=True)
            shutil.copy(T27C, tree / "target/release/t27c")
        # T106: and a real corpus, 1.2 MB of it. The REAL_CORPUS row needs
        # `t27c fpga-build --smoke` to SUCCEED at the tree root; without specs
        # the row reports UNRUN and both end-to-end cases below would be
        # measuring the absence of a corpus rather than the wiring they exist
        # to measure.
        if (ROOT / "specs/fpga").is_dir():
            shutil.copytree(ROOT / "specs/fpga", tree / "specs/fpga")
        g = subprocess.run([sys.executable, str(tree / "tools" / me)],
                           capture_output=True, text=True, cwd=td, timeout=TIMEOUT)
    counted = ("OK: %d precondition(s)" % len(GATES)) in g.stdout
    quiet = g.returncode == 0 and counted
    print("  %-28s %s" % ("end-to-end, all healthy",
                          "exit 0, says OK" if quiet else "CONTROL FAILED"))
    if not quiet:
        ok = False
        print("       exit %r (want 0); counted the table: %s" % (g.returncode, counted))
        print("       said   %s" % (first_line(g),))

    # And the clean direction: the real table must be silent, or both cases
    # above pass for free on a file that always reports something.
    live = check()
    print("  %-28s %s" % ("live tree", "silent" if not live else "CONTROL FAILED"))
    if live:
        ok = False
        for p in live:
            print("       " + p)

    print("  self-check: %s" % ("both misclassifications caught, and the whole program end to end" if ok else "FAILED"))
    return 0 if ok else 1


def sweep():
    """Run EVERY gate in a fully planted empty tree and say what each does.

    Written because I re-implemented this probe by hand three iterations
    running, and the third time it was wrong: it planted each gate ALONE, so
    two gates died on an import and were recorded as crashing on the
    repository. That table then justified work. A probe kept in the tree is
    measured once and reused; a probe retyped each time is a new instrument
    with new defects.

    Three outcomes, and only one of them is a problem by itself:

      PASS     exit 0 over nothing. Legitimate for a self-test that builds its
               own corpus, and for a skip() that turns fatal under --require.
               Everything else here is a gate that cannot fail.
      VERDICT  non-zero, with a sentence saying which input is missing.
      CRASH    non-zero through a traceback. Loud, and still not a verdict --
               it reports the harness, not the subject.

    Reports; does not gate. The judgement of which PASS is legitimate belongs
    to a reader, and encoding it here would be a second list to drift out of
    date against GATES.
    """
    # Every checker in tools/, not only the ones named check_ or *gate*. The
    # first version of this swept those two patterns and left thirteen
    # verifiers outside it -- including one wired into a CI step called
    # "Prove the trainer LEARNS" that passes in an empty tree because it never
    # touches the compiler. A sweep that selects by NAME measures the naming
    # convention; this one selects by "is a python file in tools/ that is not
    # a private module", and lets the reader judge what belongs.
    gates = sorted(q.name for q in (ROOT / "tools").glob("*.py")
                   if not q.name.startswith("_"))
    rows = []
    for g in gates:
        with tempfile.TemporaryDirectory() as td:
            tree = pathlib.Path(td)
            (tree / "tools").mkdir()
            # The WHOLE tools directory, not just this gate. Planting one file
            # is what made the third probe lie.
            for f in (ROOT / "tools").glob("*.py"):
                shutil.copy(f, tree / "tools" / f.name)
            try:
                r = subprocess.run(
                    [sys.executable, f"tools/{g}"], cwd=str(tree),
                    capture_output=True, text=True, timeout=TIMEOUT)
                out = (r.stdout + r.stderr).strip().splitlines()
                first = out[0][:56] if out else "(silent)"
                kind = ("PASS" if r.returncode == 0
                        else "CRASH" if "Traceback" in (r.stderr or "")
                        else "VERDICT")
                rows.append((g, r.returncode, kind, first))
            except subprocess.TimeoutExpired:
                rows.append((g, -1, "HUNG", f"no verdict in {TIMEOUT}s"))

    print(f"every gate in a planted empty tree ({len(rows)} gate(s))\n")
    for g, rc, kind, first in rows:
        print(f"  {g:<40} rc={rc:<3} {kind:<8} {first}")
    tally = {}
    for _, _, kind, _ in rows:
        tally[kind] = tally.get(kind, 0) + 1
    print("\n  " + "   ".join(f"{k}: {v}" for k, v in sorted(tally.items())))
    print("\n  A PASS is not automatically wrong: a self-test that plants its own")
    print("  corpus, and a skip() that --require turns fatal, both belong here.")
    print("  A CRASH always is -- it reports the harness and not the subject.")
    return 0


def main():
    if "--self-check" in sys.argv:
        return self_check()
    if "--sweep" in sys.argv:
        return sweep()
    problems = check()
    if problems:
        for p in problems:
            print(p)
        print(f"FAIL: {len(problems)} gate(s) do not fail loudly with nothing to check")
        print()
        print("  VACUOUS  the gate exits 0 having checked nothing. Everything")
        print("           downstream reads green.")
        print("  WRONG    it goes red, but not through the branch that explains")
        print("           why -- usually a crash, which is not a verdict.")
        print("  HUNG     no verdict at all.")
        return 1
    names = {g for g, _, _ in GATES}
    print(f"OK: {len(GATES)} precondition(s) across {len(names)} gates fail loudly; "
          f"{UNCOVERED} known-uncovered row(s) (see UNCOVERED in this file)")

    # The denominator, because `UNCOVERED = 0` counts ROWS this file chose to
    # write and reads like it counts GATES. It does not: this file exercises
    # six of the gate scripts in tools/, and said "0 known-uncovered" while
    # eleven others had never been run in an empty tree at all.
    #
    # Measured when they were: one of them PASSED there --
    # `check_json_parses` printed `OK: 0 tracked JSON files` and returned 0.
    # The rest skipped or crashed, which is survivable; a pass is not.
    #
    # Reported, not enforced. Making an uncovered gate a failure would turn
    # this green file red today over gates whose empty-tree behaviour is
    # already loud, and a gate that is red on the day it lands teaches people
    # to ignore red. The number is here so the gap is a fact rather than an
    # implication.
    on_disk = sorted(
        q.name for q in (ROOT / "tools").glob("*.py")
        if (q.name.startswith("check_") or "gate" in q.name)
        and q.name != pathlib.Path(__file__).name
    )
    uncovered = [g for g in on_disk if g not in names]
    print(f"    coverage: {len(on_disk) - len(uncovered)} of {len(on_disk)} "
          f"gate script(s) in tools/ are exercised here")
    if uncovered:
        print(f"    not exercised in an empty tree by this file ({len(uncovered)}):")
        for g in uncovered:
            print(f"      {g}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
