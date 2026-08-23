#!/usr/bin/env python3
"""Does every spec the cross-target gate depends on still parse?

Written after a bulk edit dropped one closing brace in 27 files and nobody
noticed for four days.

`test X_w339_batch_depth_invariant_2 {` was appended to 27 specs with its `}`
omitted, so the parser hit the next `test` keyword while still inside the block:

    parse error in fn 'ternary_mac_w339_batch_depth_invariant_2' near line 1815:
    unexpected token after expression statement: KwTest

The message was exact. Nothing surfaced it, because three layers reported
something else:

  the CI gate said   IGLA RACE CROSS-TARGET MISMATCH
  the script said    FAIL: C backend failed to build/run
  the truth was      the spec does not parse, so no backend ever ran

The script's own build helpers pass capture_output=True and inspect only
returncode, so the compiler's message -- which named the file, the function and
the line -- was collected and thrown away. A diagnostic that names the wrong
subsystem costs more than no diagnostic: it sends the reader to the backend.

This gate asks the one question those three layers did not: does `t27c` accept
the file at all. It reports the compiler's own error verbatim.

Usage:
  tools/check_specs_parse.py                  gate over the required set
  tools/check_specs_parse.py --all            every spec under specs/, reporting
  tools/check_specs_parse.py --self-check     negative control

Exits non-zero if a required spec does not parse.
"""
import glob
import os
import pathlib
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _prereq import plant  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parent.parent

# Specs the cross-target bit-exactness proof consumes. These must parse; a PR
# that breaks one is breaking the proof, not merely a file.
# T76: top-level tokens the parser DROPS while still exiting 0, per required
# spec, as measured on 2026-08-23 with `t27c parse-complete`. Debt, not a
# target: these two carry 2,548 tokens between them -- 41 of ternary_mac's 137
# `invariant` declarations among them -- and every backend reports success on
# what is left. The number may fall; a rise is a new failure.
DISCARD_DEBT = {
    "specs/igla/race/ternary_mac.t27": 1139,
    "specs/igla/race/systolic_ternary.t27": 1409,
}


def parse_completeness(t27c):
    """{spec: discarded token count} from the compiler's own completeness pass."""
    r = subprocess.run([t27c, "parse-complete"], capture_output=True, text=True, cwd=ROOT)
    out = {}
    for ln in (r.stdout + r.stderr).splitlines():
        m = re.match(r"\s*(\S+\.t27):\s*DISCARDED\s+(\d+)", ln)
        if m:
            out[m.group(1)] = int(m.group(2))
    return out


REQUIRED = [
    "specs/igla/race/ternary_mac.t27",
    "specs/igla/race/systolic_ternary.t27",
    "specs/ternary/gft_smul.t27",
    "specs/ternary/gft_sadd.t27",
]


def find_t27c():
    for p in ("target/release/t27c", "target/debug/t27c"):
        c = ROOT / p
        if c.exists():
            return str(c)
    return None


def parses(t27c, spec):
    """(ok, first line of the compiler's own error)."""
    r = subprocess.run([t27c, "gen-c", spec], capture_output=True, text=True, cwd=ROOT)
    if r.returncode == 0:
        return True, ""
    err = (r.stderr or r.stdout or "").strip().split("\n")
    return False, err[0] if err else f"exit {r.returncode}, no message"


# --- negative control -------------------------------------------------------
#
# T91: `tri gates mutate` rewrote each of main()'s two verdicts to `return 0`
# in turn and this control noticed neither. It proved parses() -- the FUNCTION
# -- and nothing proved the wiring from either verdict to the process exit
# code. Measured: with the drift verdict's `return 1` changed to `return 0` the
# gate printed "OK: all 4 required specs parse and none discards more than
# recorded" over a spec whose invariants the parser had thrown away, and the
# line below still reported "planted spec rejected = True".
#
# So the three cases at the end run the WHOLE program against a planted tree.
# The script is COPIED into that tree, which makes its module-level ROOT
# resolve there by the ordinary parent.parent rule -- no --root flag and no
# environment override, so nothing here adds a way to aim a live gate at
# somewhere harmless.

# Parses, consumes every token, and gives the two faults below a function to
# refer to.
_GOOD_SPEC = "module Ok;\n\nfn f(a: i32) -> i32 {\n    return a + 1;\n}\n"

# Parses -- gen-c exits 0 -- while top-level drop-recovery throws the body away.
# This is the shape the recorded debt is actually made of: `parse-complete
# --show specs/igla/race/ternary_mac.t27` shows 41 `invariant` declarations
# discarding exactly like this. Measured at 10 discarded tokens; the case below
# asserts the BRANCH and not the number, so a parser that drops more still
# reads as the same fault. A parser that learned to READ this would not make
# the case pass vacuously -- it demands the FAIL text, not merely a non-zero
# exit, so it would go loudly red instead.
_DISCARDS = "\ninvariant f_grows\n    forall a : i32\n    f(a) == a + 1\n"

# Does not parse at all: the paren never closes, so recovery cannot resync and
# gen-c exits 1 naming the file and the line.
#
# T91: this is the exact string the T76 note in main() offers as an example of
# a spec that DISCARDS while every backend still exits 0. It does not do that
# today. Measured on 2026-08-23 against target/release/t27c, on a planted spec
# and on a copy of the real specs/igla/race/ternary_mac.t27, bare and with the
# brace closed: gen-c exits 1 with "Expected RParen, got Eof". Whatever it did
# when that note was written, it is now a REJECT -- which is why the control
# uses it for the parse verdict and an `invariant` block for the discard one.
_REJECTED = "\nfn broken(( -> {\n"

# The spec the control plants into: a REQUIRED one carrying NO recorded debt,
# so any discard at all is a rise. Chosen by that rule rather than by index --
# reordering REQUIRED must not silently pick a spec whose 1,139-token debt
# swallows a 10-token plant and turns the drift case into a vacuous pass.
_CONTROL_SPEC = next((r for r in REQUIRED if DISCARD_DEBT.get(r, 0) == 0), None)


def _self_check_plant(td, faulty=None, extra=""):
    """The four REQUIRED specs under `td`; `faulty` gets `extra` appended.

    The other three stay clean, so the COUNT in the gate's message ("1 required
    spec(s)") is asserted too -- a fault that spread to every spec would reach
    the same branch and the same exit code through a different world.
    """
    root = pathlib.Path(td)
    for rel in REQUIRED:
        p = root / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(_GOOD_SPEC + (extra if rel == faulty else ""), encoding="utf-8")
    return root


def self_check(t27c):
    """Prove the gate red: the parse fault in-process, then both verdicts end to end."""
    import tempfile, shutil
    ok = True

    src = ROOT / REQUIRED[0]
    with tempfile.TemporaryDirectory() as td:
        bad = os.path.join(td, "bad.t27")
        text = src.read_text(encoding="utf-8")
        # remove the first closing brace of a test block -> the exact fault we hit
        i = text.find("\n}\n", text.find("test "))
        shutil.copy(src, bad)
        pathlib.Path(bad).write_text(text[:i] + "\n" + text[i + 3:], encoding="utf-8")
        ok_bad, msg = parses(t27c, bad)
        ok_good, _ = parses(t27c, str(src))
    print(f"  self-check: planted spec rejected = {not ok_bad}, real spec accepted = {ok_good}")
    if not ok_bad:
        print(f"              reported: {msg[:100]}")
    if ok_bad or not ok_good:
        ok = False

    name = pathlib.Path(__file__).name

    def spawned(label, want, says, absent, args=(), **kw):
        """Run the whole program in a planted tree and demand the exact verdict.

        `says` is every marker the branch must print; `absent` is every marker
        belonging to its NEIGHBOURS. Both verdicts open with "FAIL:" and both
        exit 1, so neither the exit code nor the first word can tell them
        apart, and a crash reaches the same exit code through no branch at all.
        Only the tails separate the three.
        """
        nonlocal ok
        with tempfile.TemporaryDirectory() as td:
            root = _self_check_plant(td, **kw)
            (root / "tools").mkdir(parents=True, exist_ok=True)
            plant(__file__, root / "tools")
            (root / "target/release").mkdir(parents=True, exist_ok=True)
            # Linked, not copied: find_t27c() asks only whether the path is
            # there, and the compiler that parses the planted specs has to be
            # the one CI runs. A stub would agree with the gate by construction.
            os.symlink(t27c, root / "target/release/t27c")
            proc = subprocess.run([sys.executable, str(root / "tools" / name), *args],
                                  timeout=300,
                                  capture_output=True, text=True, cwd=str(root))
        out = proc.stdout + proc.stderr
        missing = [s for s in says if s not in out]
        leaked = [a for a in absent if a in out]
        good = proc.returncode == want and not missing and not leaked
        print("  %-26s %s" % (label, "exit %d, says it" % want if good else "CONTROL FAILED"))
        if not good:
            ok = False
            print("       exit %r (want %r)" % (proc.returncode, want))
            if missing:
                print("       marker never printed: %r" % (missing,))
            if leaked:
                print("       neighbouring marker leaked: %r" % (leaked,))
            print("       output %r" % (out[:400],))

    # The clean planted tree must be green, or the two faults below go red for
    # whatever is wrong with the planting and prove nothing about the gate.
    # The count comes from len(REQUIRED) rather than a literal 4: adding a
    # REQUIRED spec must not turn this case red for a reason that has nothing
    # to do with the gate. The sentence itself is still unique to that branch.
    spawned("end-to-end clean tree", 0,
            says=("OK: all %d required specs parse" % len(REQUIRED),),
            absent=("FAIL", "t27c not built"))

    if _CONTROL_SPEC is None:
        # NOT COVERED, said out loud rather than skipped quietly: with every
        # REQUIRED spec carrying debt there is no spec a small plant can push
        # over its own ratchet, and a silent skip here would read as coverage.
        print("  %-26s %s" % ("drift: discards more",
                              "NOT RUN -- every REQUIRED spec carries recorded debt"))
        ok = False
    else:
        spawned("drift: discards more", 1,
                says=("FAIL: 1 required spec(s) discard MORE than recorded",
                      "%s: 0 -> " % _CONTROL_SPEC,
                      "top-level token(s) dropped",
                      "raise the number in DISCARD_DEBT"),
                absent=("do not parse", "compiler's own message", "OK: all",
                        "t27c not built"),
                faulty=_CONTROL_SPEC, extra=_DISCARDS)

    # T97: the case that tells `was` from the constant 0.
    #
    # Every case above plants into a spec whose recorded debt is ZERO, so
    # `now > was` and `now > 0` are the same expression and a mutant that
    # forgets the ledger entirely is invisible. Measured: `if now > 0` passes
    # this control and turns the LIVE gate red -- every spec carrying debt
    # would fail, and nothing here would have said which change did it.
    #
    # The economical distinguisher is not more debt, it is debt UNDER the
    # recorded figure: a spec that owes 1,139 tokens, planted with about ten.
    # Correct code is silent; `now > 0` raises a false alarm. So this case
    # asserts SILENCE, and it is the only case here that does.
    _DEBTOR = next((r for r in REQUIRED if DISCARD_DEBT.get(r, 0) > 0), None)
    if _DEBTOR is None:
        print("  %-26s %s" % ("ledger: under recorded debt",
                              "NOT RUN -- no REQUIRED spec carries recorded debt"))
        ok = False
    else:
        spawned("ledger: under recorded debt", 0,
                says=("OK: all %d required specs parse" % len(REQUIRED),),
                absent=("discard MORE than recorded", "FAIL", "t27c not built"),
                faulty=_DEBTOR, extra=_DISCARDS)

    spawned("parse: spec rejected", 1,
            says=("FAIL: 1 required spec(s) do not parse",
                  _CONTROL_SPEC or REQUIRED[0],
                  "compiler's own message"),
            absent=("discard MORE than recorded", "DISCARD_DEBT", "OK: all",
                    "t27c not built"),
            faulty=_CONTROL_SPEC or REQUIRED[0], extra=_REJECTED)

    # T101: --all is a REPORT mode, and this file says so in its own output:
    # "a report, not a gate". That distinction is worth keeping and it is not a
    # reason to leave the report's exit code unmeasured -- a report that prints
    # its table and then reports failure breaks any script reading it, and
    # nothing here would have said so. Found by `tri gates mutate --loud`.
    spawned("report mode --all", 0,
            says=("--all is a report, not a gate",),
            absent=("FAIL:", "discard MORE than recorded", "t27c not built"),
            args=("--all",))

    print("  self-check: %s" % ("both verdicts proven red" if ok else "FAILED"))
    return 0 if ok else 1


def main():
    t27c = find_t27c()
    if not t27c:
        print("FAIL: t27c not built. Run: cargo build --release -p t27c")
        return 1
    if "--self-check" in sys.argv:
        return self_check(t27c)

    targets = (sorted(glob.glob(str(ROOT / "specs/**/*.t27"), recursive=True))
               if "--all" in sys.argv else [str(ROOT / p) for p in REQUIRED])
    bad = []
    for spec in targets:
        rel = os.path.relpath(spec, ROOT)
        ok, msg = parses(t27c, spec)
        if not ok:
            bad.append((rel, msg))
        elif "--all" not in sys.argv:
            print(f"  ok   {rel}")

    if "--all" in sys.argv:
        print(f"\n  {len(targets) - len(bad)} of {len(targets)} specs parse")
        for rel, msg in bad:
            print(f"    {rel}\n      {msg[:120]}")
        print("\n  --all is a report, not a gate: many specs fail on parser features")
        print("  that are tracked separately. Only the REQUIRED set gates CI.")
        return 0

    # T76: "gen-c exited 0" is not "the parser read the file". Measured on the
    # REQUIRED set: appending `fn broken(( -> {` to a spec leaves gen-c,
    # gen-rust, gen-verilog, gen, typecheck AND parse all exiting 0 -- the
    # top-level drop-recovery discards what it cannot parse and says nothing.
    # `t27c parse-complete` is the stronger answer the compiler already ships
    # and no gate was calling: 650 specs, 430 consume all, 66 DISCARD 26,546
    # tokens between them.
    #
    # Two of the four REQUIRED specs discard today, so this is a RATCHET rather
    # than a demand for zero: the counts are frozen as named debt and may fall,
    # never rise. A REQUIRED spec that starts discarding is a new failure.
    discarded = parse_completeness(t27c)
    drift = []
    for rel in REQUIRED:
        now = discarded.get(rel, 0)
        was = DISCARD_DEBT.get(rel, 0)
        if now > was:
            drift.append((rel, was, now))
    if discarded:
        shown = ", ".join(f"{os.path.basename(r)} {n}" for r, n in sorted(discarded.items())
                          if r in REQUIRED) or "none"
        print(f"  discarded top-level tokens in the required set: {shown}")
    if drift:
        print(f"\nFAIL: {len(drift)} required spec(s) discard MORE than recorded\n")
        for rel, was, now in drift:
            print(f"  {rel}: {was} -> {now} top-level token(s) dropped")
        print()
        print("  The parser accepted the file without reading all of it, and more of")
        print("  it than before. Every backend still exits 0 on the part it skipped,")
        print("  so nothing else in CI can see this. Inspect with:")
        print(f"    t27c parse-complete --show {drift[0][0]}")
        print("  If the increase is deliberate, raise the number in DISCARD_DEBT")
        print("  in this file, in the same commit, and say why.")
        return 1

    if not bad:
        print(f"\nOK: all {len(targets)} required specs parse and none discards more "
              f"than recorded")
        return 0
    print(f"\nFAIL: {len(bad)} required spec(s) do not parse\n")
    for rel, msg in bad:
        print(f"  {rel}")
        print(f"      {msg}")
    print("\n  This is the compiler's own message. It names the file, the function")
    print("  and the line -- read it before looking at any backend.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
