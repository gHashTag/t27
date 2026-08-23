#!/usr/bin/env python3
"""Elaboration errors may fall, never rise (#2325).

This gate holds the line per module over the generated fpga set. It does not
demand zero: most of what is left waits on two named design decisions -- what a
string field means in generated hardware (#2433) and unsized array params
(#2410).

It does NOT print a classification, and the reason is a lesson. It used to. The
table said 57 condition expressions, 64 unbound, 21 whole-array, 4 malformed,
5 unknown-module, 2 missing-function -- which sums to 153 under a headline of
161, so it was arithmetically wrong the day it was written, and within an hour
three of its six rows were also stale: unbound is 72, condition is 59, malformed
is 0 (that defect was closed by the next commit), and a seventh class it never
listed (Enable of unknown task, 3) exists. A hand-copied distribution rots
faster than anyone re-reads it.

    tri elab classify     # the distribution, measured, by message shape
    tri elab secondary    # which of them are derived from another above them

Two things this gate counts carefully, because it got both wrong once:

  * iverilog's closing "N error(s) during elaboration." is a TOTAL, not an
    error. Counting it added one phantom per failing module -- 25 of a
    published 186.
  * a "diagnostic line" is not a "bad construct". One bad construct can emit
    `syntax error` AND `error: Malformed statement` on the same line, so the
    number here is diagnostic LINES. Do not quote it as a defect count.

    tools/check_elab_ratchet.py                    # verify
    tools/check_elab_ratchet.py --update-baseline  # after a deliberate change
    tools/check_elab_ratchet.py --self-check       # negative control

Requires iverilog and a built t27c; skips cleanly (exit 0) when either is
missing, so a Rust-only checkout is never blocked by it.
"""
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

# T88: overridable so the negative control can aim the WHOLE program at a
# planted tree -- the real regeneration, the real iverilog, the real
# comparison. Nothing in the repository sets it.
#
# It is read here, at import, and that is exactly what makes it usable: the
# control re-runs THIS FILE as a subprocess rather than copying it somewhere.
# A copy is how this goes wrong. ROOT is `parent.parent` of the script, so a
# control that copies the tool to a temp directory and runs it from there gets
# ROOT=/tmp -- or, one level shallower, ROOT=/ -- where the glob matches
# nothing, every baseline module reads as absent, and the run "passes" for a
# reason that has nothing to do with the tree under test.
_ENV_ROOT = os.environ.get("T27_ELAB_ROOT")
if _ENV_ROOT and not pathlib.Path(_ENV_ROOT).is_dir():
    sys.exit(f"T27_ELAB_ROOT={_ENV_ROOT} is not a directory")
ROOT = (
    pathlib.Path(_ENV_ROOT).resolve()
    if _ENV_ROOT
    else pathlib.Path(__file__).resolve().parent.parent
)
GEN = ROOT / "build/fpga/generated"
BASELINE = ROOT / "tools/elab_baseline.txt"
T27C = ROOT / "target/release/t27c"

# iverilog ends a failing file with "N error(s) during elaboration." -- a TOTAL,
# not an error. Counting it added exactly one phantom per failing module, so the
# published figure was 25 too high in both directions (see the module docstring).
# Hand-written notes live below this line in the baseline and survive
# --update-baseline. Everything above it is regenerated.
NOTES_MARK = "# --- notes (hand-written, preserved) ---\n"

SUMMARY = re.compile(r"\d+ error\(s\) during elaboration")


def counts(gen=None):
    """{module: elaboration error count} over the generated set.

    `gen` is resolved here rather than as a default argument: `def counts(gen=GEN)`
    would bind at import, so the negative control could not aim it anywhere.
    """
    out = {}
    for v in sorted((gen or GEN).glob("*.v")):
        proc = subprocess.run(
            ["iverilog", "-g2012", "-DSIMULATION", "-o", "/dev/null", str(v)],
            capture_output=True,
            text=True,
        )
        n = sum(
            1
            for ln in proc.stderr.splitlines()
            if " error" in ln and not SUMMARY.search(ln)
        )
        out[v.stem] = n
    return out


def iverilog_version():
    """The calibration of this instrument, recorded beside its numbers.

    A ratchet compares counts produced by a specific iverilog; an apt upgrade
    on the runner can move them without a single line of the compiler changing.
    The first CI run matched the local baseline exactly (186 vs 186), which is
    luck, not a property -- so the version is stored and a mismatch is named in
    the failure text instead of being mistaken for a regression.
    """
    try:
        out = subprocess.run(["iverilog", "-V"], capture_output=True, text=True)
    except Exception:
        return "unknown"
    m = re.search(r"version\s+(\S+)", out.stdout)
    return m.group(1) if m else "unknown"


def baseline_version():
    if not BASELINE.exists():
        return None
    for ln in BASELINE.read_text().splitlines():
        if ln.startswith("# iverilog-version "):
            return ln.split(" ", 2)[2].strip()
    return None


def baseline():
    if not BASELINE.exists():
        return {}
    d = {}
    for ln in BASELINE.read_text().splitlines():
        ln = ln.strip()
        if not ln or ln.startswith("#"):
            continue
        name, _, num = ln.rpartition(" ")
        try:
            d[name.strip()] = int(num)
        except ValueError:
            continue
    return d


# --------------------------------------------------------------- control
#
# The fault cannot go in the corpus. main() regenerates build/fpga/generated
# with `t27c fpga-build --smoke` BEFORE it counts, so a damaged .v planted
# there is overwritten between the planting and the reading. The baseline is
# the half of the comparison that survives regeneration, so that is where the
# fault goes: record what the planted tree really contains, then change one
# line of the record and require the gate to name the direction.
#
# Every marker below is the branch's own text. WORSE and NEW share a closing
# paragraph, so those two are separated by their row prefixes instead. Each
# case also asserts its NEIGHBOURS are silent: all four directions exit 1, so
# an exit code cannot say which one fired, and a plant that broke some other
# way -- a corpus that failed to generate, a baseline that failed to parse --
# would exit 1 too, through a branch that proves nothing about the ratchet.
WORSE_P = "A module gained elaboration errors."
GONE_P = "A module in the baseline was not generated."
BETTER_P = "Modules improved -- classify before recording."
OK_LINE = "OK: no module gained elaboration errors"

# T90: the calibration note is the one branch that reads the baseline's HEADER
# rather than its rows, and neither half of it was proven. The four direction
# cases are structurally blind to it for the same reason they are blind to a
# miscount: they only ever read the module rows, and the note prints BESIDE a
# WORSE row without changing the verdict, so iverilog_version() could return
# anything and baseline_version() could return nothing and every case above
# stayed green. Both were measured survivors of `tri gates mutate`.
VERSION_MARK = "# iverilog-version "
STALE_VER = "0.0-planted"
NOTE_HEAD = "NOTE: the baseline was taken with iverilog"


def _plant(td):
    """A tree the gate can run in: real specs, real t27c, its own build/ and
    tools/. The specs and the compiler are symlinked because the point is to
    exercise the real corpus, not a toy one."""
    t = pathlib.Path(td)
    (t / "tools").mkdir()
    (t / "target/release").mkdir(parents=True)
    (t / "specs").symlink_to(ROOT / "specs")
    (t / "target/release/t27c").symlink_to(T27C)
    return t


def _run(t, *args):
    """This very file, against the planted tree.

    cwd is the plant, and that is load-bearing rather than tidiness. main()
    invokes t27c with no cwd of its own, and `fpga-build --smoke` resolves
    specs/fpga and build/fpga/generated relative to the working directory,
    while GEN is resolved relative to ROOT. Run the child from anywhere else
    and the corpus is written where the gate will not look for it.
    """
    return subprocess.run(
        [sys.executable, os.path.abspath(__file__), *args],
        capture_output=True,
        text=True,
        cwd=str(t),
        env={**os.environ, "T27_ELAB_ROOT": str(t)},
    )


def _rows(t):
    """(header lines, [(module, count)]) of the planted baseline.

    Splitting the file to edit one line is fixture construction. It is not the
    comparison -- that stays in main(), in the child process, where the control
    cannot reach in and rewrite it.
    """
    head, rows = [], []
    for ln in (t / "tools/elab_baseline.txt").read_text().splitlines():
        if not ln.strip() or ln.startswith("#"):
            head.append(ln)
        else:
            name, _, num = ln.rpartition(" ")
            rows.append((name.strip(), int(num)))
    return head, rows


def _write(t, head, rows):
    (t / "tools/elab_baseline.txt").write_text(
        "\n".join(head + [f"{k} {v}" for k, v in rows]) + "\n"
    )


def _worse(head, rows):
    """The busiest module's record loses one, so the corpus is now above it.
    Taking the largest count means this still plants a fault on the day every
    module reaches zero -- the record goes to -1 and 0 > -1 is still WORSE."""
    i = max(range(len(rows)), key=lambda j: rows[j][1])
    rows[i] = (rows[i][0], rows[i][1] - 1)
    return head, rows


def _new(head, rows):
    """A generated module the baseline has never heard of."""
    return head, rows[1:]


def _gone(head, rows):
    """A baseline module that is not generated -- the disappearance that reads
    as progress in the total, which is why the gate iterates the union."""
    return head, rows + [("zz_never_generated", 1)]


def _better(head, rows):
    """An improvement, which must be recorded rather than silently banked."""
    rows[0] = (rows[0][0], rows[0][1] + 1)
    return head, rows


def _stale_version(head, rows):
    """A record taken with a DIFFERENT iverilog, beside something to report.

    Two faults on purpose, and they are not independent: the note prints only
    inside the worse-or-new branch, so a stale header ALONE leaves the run
    green and silent and proves nothing. Planting it next to a WORSE row is
    the only way to reach the comparison at all.
    """
    head = [
        VERSION_MARK + STALE_VER if ln.startswith(VERSION_MARK) else ln for ln in head
    ]
    return _worse(head, rows)


def _case(t, label, mutate, want, absent):
    _write(t, *mutate(*_rows(t)))
    r = _run(t)
    named = all(w in r.stdout for w in want)
    quiet = not any(a in r.stdout for a in absent)
    ok = named and quiet and r.returncode == 1
    print(
        f"  self-check {label:<6}: named = {named}, neighbours silent = {quiet}, "
        f"exit = {r.returncode} (want 1)"
    )
    if not ok:
        print(f"      wanted {want}\n      absent {absent}")
        print("      " + "\n      ".join((r.stdout + r.stderr).splitlines()[-14:]))
    return ok


def _check_summary_filter():
    """counts() must read iverilog's closing total as a TOTAL, not an error.

    This is the one defect this file records having shipped, and the four
    direction cases below are structurally blind to it: they take a baseline
    written by counts() and move it by a known delta, so a miscount shifts the
    record and the measurement by the same amount and cancels. Only an
    ABSOLUTE expectation catches it, which needs a module whose errors are
    hand-made rather than generated.

    `missing_mod` is instantiated once and never defined. iverilog emits one
    diagnostic line for it and then "2 error(s) during elaboration." -- so the
    filter is the whole difference between 1 and 2 here, which is exactly the
    one-phantom-per-failing-module that put a published figure 25 too high.
    """
    with tempfile.TemporaryDirectory() as td:
        g = pathlib.Path(td)
        (g / "clean.v").write_text("module clean; endmodule\n")
        (g / "one.v").write_text("module one; missing_mod u(); endmodule\n")
        got = counts(g)
    ok = got == {"clean": 0, "one": 1}
    print(f"  self-check counts: clean 0, one real error 1, total line not counted = {ok}")
    if not ok:
        print(f"      got {got}, want {{'clean': 0, 'one': 1}}")
    return ok


def _self_check_iverilog_version():
    """`iverilog -V`, read by the CONTROL rather than through the gate.

    iverilog_version() is one of the two things the version note is being
    tested for, so a control that asked it for the expected answer would move
    both sides of the comparison together and cancel -- the same blindness
    _check_summary_filter() exists to break for counts(). This is the ABSOLUTE
    expectation the direction cases cannot supply.

    The name carries `self_check` deliberately. That substring is how
    `tri gates mutate` tells the instrument from the thing being measured; a
    return in here must never be scored as one of the gate's failure paths.
    """
    out = subprocess.run(["iverilog", "-V"], capture_output=True, text=True)
    m = re.search(r"version\s+(\S+)", out.stdout)
    return m.groups()[0] if m else "unknown"


def self_check():
    """Plant a fault in the ratchet's record and require the gate to name it."""
    if not shutil.which("iverilog") or not T27C.exists():
        # The gate skips without these, so the control cannot run either. Say
        # that it proved nothing rather than letting a green be read as one.
        print("SKIP: iverilog or target/release/t27c missing -- control PROVED NOTHING")
        return 0

    ok = _check_summary_filter()

    with tempfile.TemporaryDirectory() as td:
        t = _plant(td)

        # Ground truth, written by the gate's OWN --update-baseline, against
        # the PLANTED tree and never the repository. Deriving it beats
        # hardcoding numbers: the fault below is a known delta from a record
        # that was true one run earlier, whatever the real counts are today,
        # so this control does not rot as the corpus improves.
        u = _run(t, "--update-baseline")
        if u.returncode != 0 or not (t / "tools/elab_baseline.txt").exists():
            print("  self-check: could not record a baseline in the plant")
            print("      " + (u.stdout + u.stderr).strip()[-400:])
            return 1
        truth = (t / "tools/elab_baseline.txt").read_text()

        # What --update-baseline recorded as this instrument's calibration,
        # against `iverilog -V` read by the control itself. Nothing else in
        # this file looks at that header line: the cases below take it as
        # given and edit the rows underneath it, so a version that stopped
        # being a version -- or stopped being written at all -- left every one
        # of them green.
        real = _self_check_iverilog_version()
        recorded = next(
            (
                ln[len(VERSION_MARK) :].strip()
                for ln in truth.splitlines()
                if ln.startswith(VERSION_MARK)
            ),
            None,
        )
        vok = recorded == real
        print(
            f"  self-check version: baseline records {recorded!r}, "
            f"`iverilog -V` says {real!r} = {vok}"
        )
        ok = vok and ok

        # The plant must be GREEN before it is faulted. Without this the four
        # cases below could all be firing on a broken plant rather than on the
        # fault, and a red for the wrong reason is what this whole exercise is
        # about.
        c = _run(t)
        if OK_LINE not in c.stdout or c.returncode != 0:
            print(f"  self-check: plant not clean before faulting (exit {c.returncode})")
            print("      " + (c.stdout + c.stderr).strip()[-400:])
            return 1
        print(f"  self-check clean : {OK_LINE!r}, exit = {c.returncode} (want 0)")

        # NOTE_HEAD is in the absent list of exactly the two cases that reach
        # the branch which can print it. The plant's record was written by
        # --update-baseline moments earlier, on this machine, by this iverilog,
        # so a calibration warning there is a false alarm -- and a false alarm
        # beside a real WORSE row is how a reader learns to skip the paragraph.
        # GONE returns before that branch and BETTER after it, so naming it
        # there would assert nothing.
        for label, mut, want, absent in (
            (
                "WORSE",
                _worse,
                ["  WORSE   ", WORSE_P],
                ["  NEW     ", GONE_P, BETTER_P, NOTE_HEAD],
            ),
            (
                "NEW",
                _new,
                ["  NEW     ", WORSE_P],
                ["  WORSE   ", GONE_P, BETTER_P, NOTE_HEAD],
            ),
            ("GONE", _gone, ["  GONE    ", GONE_P], [WORSE_P, BETTER_P]),
            ("BETTER", _better, ["  BETTER  ", BETTER_P], [WORSE_P, GONE_P]),
            (
                "STALE",
                _stale_version,
                [
                    "  WORSE   ",
                    WORSE_P,
                    f"{NOTE_HEAD} {STALE_VER}; this run used {real}.",
                ],
                ["  NEW     ", GONE_P, BETTER_P],
            ),
        ):
            (t / "tools/elab_baseline.txt").write_text(truth)
            ok = _case(t, label, mut, want, absent + [OK_LINE, "SKIP:"]) and ok

        # T101: the OPT-OUT direction. check_gate_preconditions.py proves this
        # gate reds when the tools are missing; nothing proved that
        # --allow-missing-tools then returns SUCCESS. `tri gates mutate --loud`
        # rewrote that return to a failure and no assertion noticed. An opt-out
        # that fails anyway is worse than none: it teaches that the flag does
        # not work, and the next person removes the guard instead.
        #
        # A tree with no t27c, so the precondition genuinely fails.
        with tempfile.TemporaryDirectory() as bare:
            b = pathlib.Path(bare)
            (b / "tools").mkdir()
            shutil.copy(__file__, b / "tools" / pathlib.Path(__file__).name)
            o = _run(b, "--allow-missing-tools")
            said = "reporting nothing, deliberately" in o.stdout
            good = o.returncode == 0 and said
            print(f"  self-check OPTOUT: said = {said}, exit = {o.returncode} (want 0)")
            if not good:
                ok = False
                print("      " + "\n      ".join((o.stdout + o.stderr).splitlines()[-8:]))

        return 0 if ok else 1


def main():
    if "--self-check" in sys.argv:
        return self_check()

    # T89: this used to `return 0`. The ratchet then greenlights an unchecked
    # tree and says so out loud -- exactly the "SKIP: tool missing -> 0" shape
    # the gate audit asks about. It is not exploitable in fpga-conformance
    # today, because the iverilog install step above it exits 1 after three
    # bounded attempts. But that guarantee lives in a NEIGHBOURING step and is
    # invisible from here: add this gate to another job -- which is what
    # happened to its own control one day earlier -- and the guarantee does not
    # travel with it. The precondition now belongs to the gate.
    if not shutil.which("iverilog") or not T27C.exists():
        print("SKIP: iverilog or target/release/t27c missing")
        if "--allow-missing-tools" in sys.argv:
            print("  --allow-missing-tools given: reporting nothing, deliberately")
            return 0
        print("  FAIL: nothing was checked. Pass --allow-missing-tools to")
        print("        accept that locally; CI must never pass it.")
        return 1

    gen = subprocess.run(
        [str(T27C), "fpga-build", "--smoke"], capture_output=True, text=True
    )
    if gen.returncode != 0:
        print(gen.stderr.strip()[-600:])
        print("t27c fpga-build --smoke failed")
        return 1

    now = counts()
    total = sum(now.values())

    if "--update-baseline" in sys.argv:
        # T66: the header is regenerated from a literal, so every hand-written
        # note under it was deleted by the very command this gate recommends --
        # including the one explaining that removing a syntax error can RAISE a
        # module's count. Notes below the sentinel are carried across.
        notes = ""
        if BASELINE.exists():
            txt = BASELINE.read_text()
            if NOTES_MARK in txt:
                # Bound the section by the comment lines themselves, not by a
                # blank line: this writer emits none after them, so a
                # blank-line bound swallowed the whole module list on the
                # SECOND consecutive run and wrote it back twice.
                tail = txt.split(NOTES_MARK, 1)[1].splitlines()
                kept = []
                for ln in tail:
                    if not ln.startswith("#"):
                        break
                    kept.append(ln)
                if kept:
                    notes = NOTES_MARK + "\n".join(kept) + "\n"
        header = (
            "# iverilog elaboration errors per generated module. This is a RATCHET:\n"
            "# the numbers may fall, never rise (#2325). iverilog's own \"N error(s)\n"
            "# during elaboration\" summary line is NOT counted -- it is a total, and\n"
            "# counting it added one phantom error per failing module.\n"
            "# The remainder is not one thing. Do not copy a classification here:\n"
            "# the last one was wrong when written and stale within the hour. Run\n"
            "# `tri elab classify` -- it measures the distribution by message shape.\n"
            f"# iverilog-version {iverilog_version()}\n"
        )
        body = "\n".join(f"{k} {v}" for k, v in sorted(now.items()))
        BASELINE.write_text(header + notes + body + "\n")
        print(f"baseline updated: {len(now)} modules, {total} errors")
        return 0

    base = baseline()
    if not base:
        print("no baseline; run --update-baseline once")
        return 1

    # T66: iterate the UNION. Iterating `now` alone means a module that stops
    # being generated -- dropped from the hardcoded list in main.rs, or a whole
    # empty output directory -- contributes no row at all, the total falls, and
    # the gate prints OK. A ratchet that only looks at what is present scores a
    # disappearance as an improvement.
    worse, better, new, gone = [], [], [], []
    for m in sorted(set(now) | set(base)):
        if m not in base:
            new.append((m, now[m]))
        elif m not in now:
            gone.append((m, base[m]))
        elif now[m] > base[m]:
            worse.append((m, base[m], now[m]))
        elif now[m] < base[m]:
            better.append((m, base[m], now[m]))

    print(f"elaboration errors: {total} (baseline {sum(base.values())})")
    for m, b, n in better:
        print(f"  BETTER  {m}: {b} -> {n}")
    for m, n in new:
        print(f"  NEW     {m}: {n} errors, not in baseline")
    for m, b, n in worse:
        print(f"  WORSE   {m}: {b} -> {n}")
    for m, b in gone:
        print(f"  GONE    {m}: was {b}, is no longer generated at all")

    if gone:
        print()
        print("A module in the baseline was not generated. Its errors did not")
        print("get fixed -- they left the measured set, which reads as progress")
        print("in the total above. Restore the module, or remove its baseline")
        print("line deliberately with --update-baseline.")
        return 1

    if worse or new:
        print()
        bv, nv = baseline_version(), iverilog_version()
        if bv and bv != nv:
            print(f"NOTE: the baseline was taken with iverilog {bv}; this run used {nv}.")
            print("A version change can move these counts without any compiler change,")
            print("so check that before reading the rows above as a regression.")
        print("A module gained elaboration errors. Generated Verilog that iverilog")
        print("cannot elaborate cannot be simulated, so its vectors can never run.")
        print("If the increase is deliberate: tools/check_elab_ratchet.py --update-baseline")
        return 1
    if better:
        print()
        # T66: a drop is not automatically a win. A syntax error TRUNCATES the
        # file, so introducing one makes a module's count collapse and this
        # branch used to answer "Modules improved. Record it." with no hedge --
        # and obeying it froze the truncated number as the new baseline.
        print("Modules improved -- classify before recording. A drop caused by a")
        print("syntax error is not a fix: a syntax error truncates the file, so")
        print("everything after it stops being counted. Check with:")
        print("    tri elab classify")
        print("Then, if the drop is real: tools/check_elab_ratchet.py --update-baseline")
        return 1
    print("OK: no module gained elaboration errors")
    return 0


if __name__ == "__main__":
    sys.exit(main())
