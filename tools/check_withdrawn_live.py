#!/usr/bin/env python3
"""A withdrawn number must not survive in a live document.

Written because one did, for ten days, in the two files that carry claims to
readers. `323 MHz` was established on 2026-08-05 to be the toggle rate of a
20-stage ring oscillator clocking a 23-bit counter, on a netlist whose design
module holds 55 cells and none of the GF16 arithmetic being claimed. Three
separate documents recorded the withdrawal. The arXiv draft and its .tex kept
the number in the title, the abstract, the results table and a sentence
asserting it came "from actual FPGA hardware runs" -- through an intervening
honesty pass over the same file, which was looking at a different sentence.

The lesson generalises past this one number: a correction recorded in notes does
not propagate to documents, and nothing was checking. So this checks.

  live document      any tracked .md / .tex / .rst / .t27 / .csv / .json, minus the
                     exclusions below. The machine-readable types are in scope
                     because the first version was not, and was green while
                     specs/numeric/formats_catalog.t27 -- the canonical CATALOG row
                     that feeds the published dataset -- still carried the number.
                     A gate that is green because it is under-scoped is the same
                     failure it was built to kill.
  exclusions         the claims registry itself (it must state what it withdrew),
                     dated history under docs/reports/ (a record of what was
                     believed then is not a live claim), and this gate's own data
  baseline           tools/withdrawn_live_baseline.txt -- occurrences that are
                     text ABOUT the withdrawal, keyed by path + pattern + a hash
                     of the LINE. Keying on the line and not just the file matters
                     for append-only documents such as docs/NOW.md: baselining the
                     file wholesale would let a future genuine claim through.

Usage:
  tools/check_withdrawn_live.py                 gate; exits non-zero on any new hit
  tools/check_withdrawn_live.py --self-check    negative control: plant a hit in a
                                                temp tree and prove the gate fires
  tools/check_withdrawn_live.py --update-baseline

Exits non-zero on any failure.
"""
import hashlib
import os
import pathlib
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _prereq import broken, plant  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parent.parent
REGISTRY = ROOT / "tools/withdrawn.txt"
BASELINE = ROOT / "tools/withdrawn_live_baseline.txt"
SUFFIXES = {".md", ".tex", ".rst", ".t27", ".csv", ".json"}
EXCLUDE_PREFIXES = (
    "docs/nona-03-manifest/RESEARCH_CLAIMS.md",   # must state what it withdrew
    "docs/reports/",                              # dated history, not live claims
    "conformance/vectors/",                       # numeric test data, not prose
    "tools/withdrawn.txt",
    "tools/withdrawn_live_baseline.txt",
    "tools/check_withdrawn_live.py",
)


def rules(registry=None):
    """(compiled regex, reason, pointer) for every row of the registry."""
    registry = registry or REGISTRY
    out = []
    for line in registry.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = [p.strip() for p in line.split("|")]
        if len(parts) != 3:
            sys.exit(f"malformed row in {registry.name}: {line!r}")
        out.append((re.compile(parts[0], re.I), parts[1], parts[2]))
    if not out:
        sys.exit(f"{registry.name} lists no withdrawn numbers -- the gate would pass vacuously")
    return out


def live_documents(root=ROOT):
    """Tracked documents that carry claims to a reader."""
    try:
        listed = subprocess.run(["git", "ls-files"], cwd=root, capture_output=True,
                                text=True, check=True).stdout.split("\n")
    except Exception:
        listed = [str(p.relative_to(root)) for p in root.rglob("*") if p.is_file()]
    for rel in listed:
        if not rel or pathlib.PurePosixPath(rel).suffix not in SUFFIXES:
            continue
        if any(rel.startswith(x) for x in EXCLUDE_PREFIXES):
            continue
        yield rel


def scan(root=ROOT, registry=None):
    hits = []
    for pat, reason, where in rules(registry):
        for rel in live_documents(root):
            try:
                text = (root / rel).read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            for i, line in enumerate(text.splitlines(), 1):
                if pat.search(line):
                    key = hashlib.sha1(" ".join(line.split()).encode()).hexdigest()[:12]
                    hits.append((rel, i, pat.pattern, reason, where,
                                 line.strip()[:110], key))
    return hits


def baseline():
    if not BASELINE.exists():
        return set()
    return {l.strip() for l in BASELINE.read_text(encoding="utf-8").splitlines()
            if l.strip() and not l.startswith("#")}


# Helpers below belong to the CONTROL, not to the gate. They are named for
# "self_check" on purpose: `tri gates mutate` skips top-level defs matching that
# name, so a return rewritten in here breaks the ruler instead of the thing
# being measured. None of them returns a bare verdict either, so the exemption
# is belt and braces rather than the only thing holding.


def _self_check_tree(td, docs, registry_text, baseline_text=None):
    """Plant a whole repository under `td` and hand back its root.

    The script is COPIED in, so its module-level ROOT resolves to the planted
    tree by the ordinary parent.parent rule. Deliberately no --root flag and no
    T27_*_ROOT variable: a lever that aims this gate at a harmless directory
    would exist in the LIVE gate too, and this gate is the last thing standing
    between a retracted number and a reader.
    """
    import shutil
    t = pathlib.Path(td)
    (t / "tools").mkdir(parents=True, exist_ok=True)
    (t / "docs").mkdir(parents=True, exist_ok=True)
    plant(pathlib.Path(__file__).resolve(), t / "tools")
    (t / "tools/withdrawn.txt").write_text(registry_text, encoding="utf-8")
    if baseline_text is not None:
        (t / "tools/withdrawn_live_baseline.txt").write_text(baseline_text,
                                                             encoding="utf-8")
    for name, body in docs.items():
        (t / "docs" / name).write_text(body, encoding="utf-8")
    return t


def _self_check_run(t, args=()):
    """Run the WHOLE program in a planted tree: (exit code, stdout+stderr).

    Calling scan() in-process proves the checking FUNCTION and not the wiring
    from it to the process exit code -- and the wiring is exactly where both of
    this gate's verdicts were unguarded. Measured on this file: rewriting
    either `return 1` in main() to `return 0` left the gate printing its full
    FAIL text and exiting 0, and the control saw nothing.
    """
    p = subprocess.run([sys.executable, str(t / "tools/check_withdrawn_live.py"), *args],
                       cwd=str(t), capture_output=True, text=True)
    return p.returncode, p.stdout + p.stderr


def _self_check_verdict(label, got, out, want, must, must_not):
    """Assert the exit code AND the message AND the silence of the siblings.

    Both failure branches open with "FAIL:", so an assertion on the first word
    cannot tell them apart, and a crashed interpreter reaches the same non-zero
    exit as a considered verdict. So each case names the tail that identifies
    its own branch and the tails of every branch that must have stayed quiet.
    """
    missing = [m for m in must if m not in out]
    leaked = [m for m in must_not if m in out]
    ok = got == want and not missing and not leaked
    print(f"  self-check: {label}: exit {got} (want {want})"
          + (f", MISSING {missing!r}" if missing else "")
          + (f", LEAKED {leaked!r}" if leaked else "")
          + ("" if ok else "   <-- FAILED"))
    return ok


def self_check():
    """Negative control. A gate nobody has seen fail is not a gate.

    Case 1 proves the scanner in-process. Cases 2-4 run the whole program
    against a planted tree, one case per verdict main() can reach, because a
    control that stops at scan() leaves every exit code unmeasured.

    NOT COVERED, said out loud so nobody reads a green here as completeness:

      * the `git ls-files` branch of live_documents(). A TemporaryDirectory is
        not a git repository, so every case below exercises the rglob fallback
        while the live gate takes the git path. What documents get scanned is
        therefore unproven by this control.
      * the two sys.exit() calls in rules() -- malformed row, empty registry.
      * the --update-baseline path, which writes rather than judges.

    The last two are not `return`s carrying a verdict, so they are not mutable
    sites; both also fail in a maintainer's face the first time the tool runs,
    rather than rotting quietly the way a verdict that stopped firing does.
    """
    import tempfile, shutil

    # Markers of every branch that prints. The two FAIL branches share their
    # first word, so cases below match on the distinguishing tail.
    M_HIT = "withdrawn number(s) stated in a live document"
    M_GONE = "rule(s) removed from"
    M_OK = "OK: no withdrawn number is stated in a live document"
    M_BASELINE = "baseline written"
    M_CONTROL = "self-check: planted hit found"
    CLEAN_DOC = "The design synthesises and its testbench passes.\n"

    # 1. The scanner sees a planted number and stays silent on a clean file.
    with tempfile.TemporaryDirectory() as td:
        t = pathlib.Path(td)
        (t / "tools").mkdir()
        (t / "docs").mkdir()
        shutil.copy(REGISTRY, t / "tools/withdrawn.txt")
        (t / "docs/planted.md").write_text("The design reaches 323 MHz on Artix-7.\n")
        (t / "docs/clean.md").write_text(CLEAN_DOC)
        hits = scan(t, t / "tools/withdrawn.txt")
        planted = [h for h in hits if h[0] == "docs/planted.md"]
        clean = [h for h in hits if h[0] == "docs/clean.md"]
        ok = len(planted) == 1 and not clean
        print(f"  self-check: planted hit found = {len(planted) == 1}, "
              f"clean file silent = {not clean}")

    registry_text = REGISTRY.read_text(encoding="utf-8")

    # 2. A withdrawn number stated in a live document. No baseline file is
    #    planted, so the deleted-rule branch above it is never reached and
    #    cannot be what fired.
    with tempfile.TemporaryDirectory() as td:
        t = _self_check_tree(
            td,
            {"planted.md": "The design reaches 323 MHz on Artix-7.\n",
             "clean.md": CLEAN_DOC},
            registry_text)
        got, out = _self_check_run(t)
        ok = _self_check_verdict(
            "live claim -> FAIL", got, out, 1,
            must=[f"FAIL: 1 {M_HIT}", "docs/planted.md:1", "323 MHz"],
            must_not=[M_GONE, M_OK, M_BASELINE, M_CONTROL]) and ok

    # 3. A registry row deleted while its baseline exemption stayed -- the T71
    #    direction. The planted documents are CLEAN on purpose: neuter this
    #    branch and the run falls through to "OK ... exit 0", which is what the
    #    exit code below reads. Plant a live hit here instead and the OTHER
    #    failure path fires, the process still exits 1, and the mutant walks.
    rows = [ln for ln in registry_text.splitlines()
            if ln.strip() and not ln.strip().startswith("#")]
    if len(rows) < 2:
        print(f"  self-check: deleted-rule case NOT RUN -- {REGISTRY.name} holds "
              f"{len(rows)} row(s); the case needs one to delete and one to keep")
        ok = False
    else:
        dropped = rows[0].split("|")[0].strip()
        exemption = f"# planted baseline\ndocs/clean.md | {dropped} | 0123456789ab\n"
        with tempfile.TemporaryDirectory() as td:
            t = _self_check_tree(td, {"clean.md": CLEAN_DOC},
                                 "".join(ln + "\n" for ln in rows[1:]),
                                 baseline_text=exemption)
            got, out = _self_check_run(t)
            ok = _self_check_verdict(
                "deleted rule -> FAIL", got, out, 1,
                must=[M_GONE, f"/{dropped}/", "un-withdrawn"],
                must_not=[M_HIT, M_OK, M_BASELINE, M_CONTROL]) and ok

        # 4. The same tree with the row RESTORED: neither verdict may fire. A
        #    control that goes red whatever it is shown proves nothing, and
        #    this is the case that would catch it.
        with tempfile.TemporaryDirectory() as td:
            t = _self_check_tree(td, {"clean.md": CLEAN_DOC},
                                 "".join(ln + "\n" for ln in rows),
                                 baseline_text=exemption)
            got, out = _self_check_run(t)
            ok = _self_check_verdict(
                "intact registry -> OK", got, out, 0,
                must=[M_OK],
                must_not=["FAIL:", M_GONE, M_BASELINE, M_CONTROL]) and ok

    # T100: the LEDGER-WRITING path, which nothing exercised. Every case above
    # runs the verify path; `--update-baseline` writes the exemption ledger and
    # returns success, and `tri gates mutate --loud` rewrote that success return
    # to a failure with no assertion here noticing. The same site survived in
    # four gates.
    #
    # Exit AND effect: exit alone would pass a run that returned 0 without
    # writing, and the marker alone would pass one that wrote and then reported
    # failure -- which is the mutation that found this.
    with tempfile.TemporaryDirectory() as td:
        if True:
            t = _self_check_tree(
                td,
                {"planted.md": "The design reaches 323 MHz on Artix-7.\n",
                 "clean.md": CLEAN_DOC},
                registry_text,
                baseline_text="")
            got, out = _self_check_run(t, args=("--update-baseline",))
            ok = _self_check_verdict(
                "--update-baseline -> writes", got, out, 0,
                must=[M_BASELINE],
                must_not=["FAIL:", M_GONE, M_CONTROL]) and ok

    return 0 if ok else 1


def main():
    if "--self-check" in sys.argv:
        return self_check()
    # A crash is not a verdict. Run where the tracked input is absent, this
    # raised FileNotFoundError and a traceback -- which check_gate_preconditions
    # scores WRONG: "it went red, but not through the branch that explains
    # why". broken(), not skip(): a missing TOOL is the environment, a missing
    # file this repository tracks is the repository.
    if not REGISTRY.is_file():
        broken("tools/withdrawn.txt is missing. It is the register of withdrawn "
               "claims this gate searches for, and it is tracked in git.")
    hits = scan()
    if "--update-baseline" in sys.argv:
        BASELINE.write_text(
            "# Occurrences that are text ABOUT a withdrawal, not a live claim.\n"
            "# Keyed path | pattern | sha1(line)[:12] -- editing the line re-opens the gate,\n"
            "# which is what we want for append-only documents like docs/NOW.md.\n"
            + "".join(f"{rel} | {pat} | {key}\n"
                      for rel, pat, key in sorted({(h[0], h[2], h[6]) for h in hits})),
            encoding="utf-8")
        print(f"  baseline written: {len({(h[0], h[2], h[6]) for h in hits})} entries")
        return 0
    known = baseline()

    # T71: the REVERSE direction. This gate asked one question -- is a withdrawn
    # number stated in a live document -- and guarded only against the registry
    # being emptied entirely. Deleting a single row is invisible: measured, six
    # of the seven rows can be removed with the gate printing
    # "OK: no withdrawn number is stated in a live document" and exiting 0,
    # while coverage genuinely lapses (a planted "41.2 GOPS" scores 1 hit with
    # the rule and 0 without). The seventh, 323 MHz, is pinned only because
    # self_check() happens to hardcode that string.
    #
    # No new data file is needed: the baseline already records WHICH pattern
    # excused each accepted occurrence, so a pattern present there and absent
    # from the registry is a rule that was deleted while its exemptions stayed.
    # `rsplit` on the last separator keeps this correct if a regex contains "|".
    live_patterns = {p.pattern for p, _, _ in rules()}
    excused = {ln.split(" | ", 1)[1].rsplit(" | ", 1)[0]
               for ln in known if ln.count(" | ") >= 2}
    gone = sorted(excused - live_patterns)
    if gone:
        print(f"FAIL: {len(gone)} rule(s) removed from {REGISTRY.name} while their")
        print("baseline exemptions remain:\n")
        for pat in gone:
            print(f"  /{pat}/ -- no longer scanned for; the number is un-withdrawn")
        print()
        print("  A registry row is the only thing standing between a retracted")
        print("  number and a live document. Deleting one is a withdrawal being")
        print("  reversed, which is an owner decision, not a cleanup. If it IS")
        print("  deliberate, drop the matching baseline lines in the same commit.")
        return 1

    new = [h for h in hits if f"{h[0]} | {h[2]} | {h[6]}" not in known]
    if not new:
        print(f"OK: no withdrawn number is stated in a live document "
              f"({len(list(live_documents()))} documents scanned)")
        return 0
    print(f"FAIL: {len(new)} withdrawn number(s) stated in a live document\n")
    for rel, line, pat, reason, where, text, _key in new:
        print(f"  {rel}:{line}")
        print(f"      matches /{pat}/ -- {reason}")
        print(f"      see {where}")
        print(f"      > {text}")
    print("\n  Fix the document. If the line is text ABOUT the withdrawal, add it to")
    print("  tools/withdrawn_live_baseline.txt with --update-baseline.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
