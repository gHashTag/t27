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
import pathlib
import shutil
import subprocess
import sys
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
BARE, WITH_T27C = "bare", "t27c"
GATES = [
    ("check_duplicate_agreement.py", BARE, "t27c not built"),
    ("check_duplicate_agreement.py", WITH_T27C, "no duplicated function was found at all"),
    ("check_elab_ratchet.py", BARE, "iverilog or target/release/t27c missing"),
    ("check_seal_coverage.py", BARE, "no seals found at all"),
    ("check_specs_generate.py", BARE, "t27c not built"),
    ("check_specs_generate.py", WITH_T27C, "found no .t27 at all"),
    ("check_specs_parse.py", BARE, "t27c not built"),
    ("check_vector_data.py", BARE, "no baseline"),
]

# NOT COVERED, said out loud rather than left to be inferred from a count:
#
#   check_elab_ratchet.py  "t27c fpga-build --smoke failed"
#   check_elab_ratchet.py  "no baseline; run --update-baseline once"
#
# Named by message, not by line. The first version of this note said :346 and
# :390, and the edit that fixed the SKIP branch eight lines above them moved
# both to :359 and :403 in the same commit -- a comment that was measurably
# false about its own repository before it was ever pushed.
#
# Both sit behind an iverilog check, so reaching them needs iverilog on PATH --
# true in fpga-conformance, not true everywhere this file runs. An assertion
# whose expected message changes with the machine is not an assertion. Covering
# them needs a stage that requires iverilog and skips loudly without it; that
# is a bigger change than this file is, and it is filed rather than faked.
UNCOVERED = 2

TIMEOUT = 300


def run_on_empty_tree(script, stage=BARE, source=None):
    """Copy `script` into an empty tree and run it there. Returns the result."""
    with tempfile.TemporaryDirectory() as td:
        tree = pathlib.Path(td)
        (tree / "tools").mkdir()
        if source is None:
            shutil.copy(ROOT / "tools" / script, tree / "tools" / script)
        else:
            (tree / "tools" / script).write_text(source, encoding="utf-8")
        if stage == WITH_T27C:
            (tree / "target/release").mkdir(parents=True)
            shutil.copy(T27C, tree / "target/release/t27c")
        return subprocess.run(
            [sys.executable, str(tree / "tools" / script)],
            capture_output=True, text=True, cwd=td, timeout=TIMEOUT,
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


def check():
    """Returns a list of problems. Empty means every precondition is loud."""
    problems = []
    for script, stage, want in GATES:
        if not (ROOT / "tools" / script).exists():
            problems.append(f"GONE      tools/{script} is in the table and not on disk")
            continue
        if stage == WITH_T27C and not T27C.exists():
            # Not a pass. A row that cannot run is a row that proved nothing,
            # and reporting it as absent would be the vacuous pass this file
            # exists to catch, one level up.
            problems.append(
                f"UNRUN     {script} [{stage}] needs target/release/t27c\n"
                f"            build it: cargo build --release -p t27c"
            )
            continue
        try:
            r = run_on_empty_tree(script, stage)
        except subprocess.TimeoutExpired:
            problems.append(f"HUNG      {script} [{stage}] did not finish on an empty tree")
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
        shutil.copy(__file__, tree / "tools" / me)
        for script, _, _ in GATES:
            shutil.copy(ROOT / "tools" / script, tree / "tools" / script)
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


def main():
    if "--self-check" in sys.argv:
        return self_check()
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
    names = len({g for g, _, _ in GATES})
    print(f"OK: {len(GATES)} precondition(s) across {names} gates fail loudly; "
          f"{UNCOVERED} known-uncovered (see UNCOVERED in this file)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
