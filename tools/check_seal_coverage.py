#!/usr/bin/env python3
"""Does every seal still describe a spec that exists, unchanged since it was sealed?

`seal-coverage.yml` is named a required check in docs/BRANCH-PROTECTION.md and its
entire body was `echo "Running SEAL coverage analysis..."`. A required check that
cannot fail reads as coverage and is worse than none.

Establishing what it *should* assert took two attempts, and the first was wrong in a
way worth recording. I scored coverage by matching seal FILENAMES against spec
filenames and got "1668 orphans of 1714, 1024 specs of 1070 uncovered" -- a finding
about my assumption, not the repository. Seals are keyed by MODULE name; the spec they
describe is named inside the file, in `spec_path`.

What a seal actually records:

    spec_path, spec_hash            the spec, and its content hash when sealed
    gen_hash_{c,rust,verilog,zig}   sha256 of each generated target at that moment
    module, ring, sealed_at

So a seal is a reproducibility record, and its invariant is: **the spec it names still
exists, and still hashes to what was recorded**. If the spec changed, the four
gen_hashes no longer describe what it produces, and the seal asserts something false.

State when this was written -- 1714 seals:

    1507  valid
     113  stale        spec changed after sealing
      74  dangling     spec was committed, then deleted -- 16 of them by one commit,
                       692ba5263 (DARPA CLARA submission)
      15  phantom      spec appears in NO commit and is nowhere on disk. Four of these
                       are GF16 claims/comparison specs, and for those the seal file is
                       the ONLY trace of the module anywhere in the tree
       5  no spec_path

The 207 broken ones are recorded in tools/seal_baseline.txt as debt, one per line, so
this gate holds the line without demanding they all be fixed at once. Remove a line
when the seal is fixed and the gate then holds it fixed.

Usage:
  tools/check_seal_coverage.py                  gate
  tools/check_seal_coverage.py --self-check     negative control
  tools/check_seal_coverage.py --update-baseline

Exits non-zero on any NEW dangling or stale seal.
"""
import glob
import hashlib
import json
import os
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BASELINE = ROOT / "tools/seal_baseline.txt"


def scan(root=ROOT):
    """(name, kind, detail) for every seal that does not hold."""
    bad = []
    seals = sorted(glob.glob(str(root / ".trinity/seals/*.json")))
    for p in seals:
        name = os.path.basename(p)
        try:
            d = json.load(open(p))
        except Exception as e:
            bad.append((name, "unreadable", str(e)[:60]))
            continue
        sp = d.get("spec_path")
        if not sp:
            bad.append((name, "no-spec-path", "the seal does not say which spec it describes"))
            continue
        full = root / sp
        if not full.exists():
            # Two different problems wearing one word. A seal for a spec that WAS
            # committed and then deleted is an orphan of that deletion: remove it with
            # the spec, or restore both. A seal for a spec that appears in no commit
            # names nothing anyone can fetch -- its spec_hash and four gen_hashes
            # describe a file that is not in the history, so the record has no
            # checkable content at all. The fixes are not the same, so the gate does
            # not call them the same thing.
            bad.append((name, "dangling" if _ever_existed(root, sp) else "phantom", sp))
            continue
        want = (d.get("spec_hash") or "")
        algo, _, digest = want.partition(":")
        # T81: a digest must be SHAPED like one. `not digest` accepted anything
        # non-empty, so a malformed digest fell through to the byte comparison
        # below and came back "changed since sealing" -- a diagnosis whose
        # prescribed repair is "re-seal the spec", which cannot work.
        # hexdigest() returns exactly 64 lowercase hex characters, so a 71- or
        # 63-character value can never equal one no matter what the spec says.
        #
        # Measured: five seals sit in the ledger that way -- four carry a
        # doubled `sha256:sha256:` prefix (71 chars, one of them a colon) and
        # two carry 63-character walking-nibble placeholders; five are reported
        # `stale` and the sixth is caught a branch earlier as `dangling`.
        # Hashing every historical blob of each named spec matched none of the
        # inner digests, so those seals never described the spec at any commit.
        # A permanent +5 floor that no spec work can retire, wearing the label
        # of work someone could do.
        if algo != "sha256" or not _HEX64.fullmatch(digest):
            bad.append((name, "no-spec-hash", f"spec_hash={want!r}"))
            continue
        got = hashlib.sha256(full.read_bytes()).hexdigest()
        if got != digest:
            bad.append((name, "stale", f"{sp} changed since sealing"))
    return len(seals), bad


# hashlib.sha256().hexdigest() is exactly this and nothing else.
_HEX64 = re.compile(r"[0-9a-f]{64}")

_EVER = {}
_SHALLOW = {}


def _shallow(root):
    """True when this checkout has no history to ask about."""
    if root in _SHALLOW:
        return _SHALLOW[root]
    try:
        r = subprocess.run(["git", "rev-parse", "--is-shallow-repository"],
                           cwd=root, capture_output=True, text=True, timeout=10)
        val = r.stdout.strip() == "true"
    except Exception:
        val = False
    _SHALLOW[root] = val
    return val


def _ever_existed(root, sp):
    """Did this spec appear in ANY commit, under this path or its basename?

    Checked two ways on purpose. My first pass used
    `git log --diff-filter=D -- <exact path>`, which only sees a deletion recorded at
    that same path, and it reported 73 specs as never having existed. By basename
    across all history the number is 15. An instrument that overstates fivefold is how
    'seals reference specs that never existed' becomes an accusation nobody can
    support -- so this asks twice.
    """
    if sp in _EVER:
        return _EVER[sp]
    # T70: in a SHALLOW clone there is no history to ask, and answering "never
    # committed" from a one-commit checkout is not a measurement -- it is the
    # broken-ruler error, with the instrument inside the failure domain. CI
    # used a bare `actions/checkout@v4`, i.e. depth 1, so the exact-path arm
    # could never fire and every deleted spec printed `phantom` ("the spec
    # appears in NO commit -- find the spec or drop the seal") instead of
    # `dangling` ("remove the seal with it, or restore both"). Wrong class,
    # wrong prescribed repair, on the only output the gate prints. Measured in
    # a real --depth=1 clone: {stale 191, dangling 74, phantom 15} became
    # {stale 191, phantom 89, dangling 0}.
    if _shallow(root):
        return True              # cannot tell: assume the milder classification
    base = os.path.basename(sp)
    hit = False
    for args in (["--", sp], ["--", "*/" + base]):
        try:
            r = subprocess.run(["git", "log", "--all", "--oneline"] + args,
                               cwd=root, capture_output=True, text=True, timeout=30)
            if r.stdout.strip():
                hit = True
                break
        except Exception:
            return True          # cannot tell: assume the milder classification
    _EVER[sp] = hit
    return hit


def baseline():
    """{name: kind} from the ledger. The kind column was always written and
    always thrown away.

    T83: the ledger forgave a NAME, not a state. `--update-baseline` writes
    `name | kind | detail`, and reading it back kept only the name, so a
    baselined entry was a permanent, kind-blind exemption. Measured on the real
    tree: 58 baselined names are no longer in the bad set and NOTHING computed
    that -- 56 are genuine repairs the gate never mentioned, and 2 seal files
    are gone outright (FpgaEmission.json, radix_economy.json), both admitted as
    `stale`, whose prescribed repair is "re-seal it". Deleting a stale seal
    destroys the reproducibility record and the gate said nothing.

    No file-format change: the writer already emits the kind, and a kind-less
    line maps to None.
    """
    if not BASELINE.exists():
        return {}
    out = {}
    for l in BASELINE.read_text().splitlines():
        if not l.strip() or l.startswith("#"):
            continue
        parts = [x.strip() for x in l.split("|")]
        out[parts[0]] = parts[1] if len(parts) > 1 and parts[1] else None
    return out


# `phantom` and `dangling` are decided from git history, which a shallow
# checkout does not have -- `_ever_existed` concedes exactly that. Movement
# WITHIN this pair is a property of the checkout, not of the repository, so it
# is never reported as drift. Measured: 15 baselined entries sit at `phantom`
# because the ledger was written when CI ran on a depth-1 clone, and read
# `dangling` now that #2445 gave it history. That is the instrument being
# fixed, not the tree changing.
_HISTORY_PAIR = {"phantom", "dangling"}


def compare(bad, known, present):
    """(changed, departed, fixed) -- the three things a name-keyed ledger hid."""
    badkind = {n: k for n, k, _ in bad}
    badnames = set(badkind)
    changed = []
    for n, was in sorted(known.items()):
        now = badkind.get(n)
        if now is None or was is None or was == now:
            continue
        if {was, now} <= _HISTORY_PAIR:
            continue
        changed.append((n, was, now))
    left = sorted(set(known) - badnames)
    departed = [n for n in left if n not in present]
    fixed = [n for n in left if n in present]
    return changed, departed, fixed


def _check_compare():
    """T83: the ledger forgives a STATE, not a name.

    Pure, so it needs no tree: `compare` is the whole of the new behaviour.
    Movement inside {phantom, dangling} must stay silent -- that pair is
    decided from git history a shallow checkout does not have, and 15 real
    entries sit exactly there because the ledger was written before #2445 gave
    CI its history.
    """
    present = {"A.json", "B.json", "D.json"}
    bad = [("A.json", "dangling", ""), ("B.json", "dangling", "")]
    ch, dep, fx = compare(bad, {"A.json": "stale"}, present)
    real_change = ch == [("A.json", "stale", "dangling")]
    ch2, _, _ = compare(bad, {"B.json": "phantom"}, present)
    pair_silent = ch2 == []
    _, dep3, _ = compare(bad, {"Gone.json": "stale"}, present)
    departure = dep3 == ["Gone.json"]
    _, _, fx4 = compare(bad, {"D.json": "stale"}, present)
    repair = fx4 == ["D.json"]
    ok = real_change and pair_silent and departure and repair
    print(f"  compare: real kind change reported = {real_change}, "
          f"phantom/dangling silent = {pair_silent}, "
          f"departure named = {departure}, repair named = {repair}")
    return ok


def self_check():
    """Plant a seal whose spec hash is wrong and prove the scan reports it."""
    import shutil
    import tempfile

    WRONG = "0" * 64          # well-formed digest, and no spec produces it

    def plant(td, seals, ledger=None):
        """A tree this gate can be aimed at: one spec, the seals named, and
        optionally a ledger.

        `seals` maps a seal FILENAME to (spec_path, spec_hash). A hash of None
        means "the digest that holds", which only this function can supply
        because only it writes the spec.
        """
        t = pathlib.Path(td)
        (t / ".trinity/seals").mkdir(parents=True)
        (t / "specs").mkdir()
        (t / "tools").mkdir()
        spec = t / "specs/x.t27"
        spec.write_text("module X;\n")
        holds = hashlib.sha256(spec.read_bytes()).hexdigest()
        for nm, (spath, digest) in seals.items():
            (t / ".trinity/seals" / nm).write_text(json.dumps(
                {"module": nm[:-5], "spec_path": spath,
                 "spec_hash": "sha256:" + (holds if digest is None else digest)}))
        if ledger is not None:
            (t / "tools/seal_baseline.txt").write_text(ledger)
        return t

    with tempfile.TemporaryDirectory() as td:
        total, bad = scan(plant(td, {
            "Good.json": ("specs/x.t27", None),
            "Stale.json": ("specs/x.t27", WRONG),
            "Gone.json": ("specs/missing.t27", None)}))
        kinds = sorted(k for _, k, _ in bad)
        # The temp tree has no git history, so a missing spec is correctly PHANTOM
        # rather than dangling -- that distinction is the point of this scan and the
        # control asserts it rather than the older two-way answer. This check failed
        # when the classification was split, which is what a control is for.
        ok = total == 3 and kinds == ["phantom", "stale"]
    print(f"  self-check: 3 seals scanned; stale reported, missing-spec classified "
          f"phantom (no history), good one silent = {ok}")
    if not ok:
        print(f"              got {total} seals, kinds {kinds}")

    # T91: everything above proves scan(), and _check_compare() below proves
    # compare(). NEITHER proves the wiring from those to the process exit code,
    # and that wiring is the whole of main(). Measured with `tri gates mutate
    # --only check_seal_coverage.py`: forcing main()'s two verdicts to 0 left
    # BOTH of the checks above green -- 1/3 killed, "SURVIVED at lines 321,
    # 339" -- while the program announced newly-broken seals and exited 0.
    #
    # So run the WHOLE program. The script is COPIED into the planted tree,
    # which makes its module-level ROOT (and therefore BASELINE, and the seal
    # directory main() reads for `present`) resolve there by the ordinary
    # parent.parent rule -- no --root flag and no environment override, so this
    # adds no way to aim the live gate at somewhere harmless.
    #
    # main()'s remaining verdict, "no seals found at all", is covered from
    # OUTSIDE this file by tools/check_gate_preconditions.py, which is the one
    # control for that precondition class across six gates. It is not repeated
    # here, and it is named here so nobody reads this block as the whole of the
    # gate's coverage.
    me = pathlib.Path(__file__).resolve()

    def spawned(label, want_exit, present, absent, seals, ledger=None, args=()):
        """Plant a world, run the real program in it, demand one exact branch.

        `present` pins WHICH branch spoke and `absent` names the siblings that
        must not have. Three of main()'s exits are `1`, so the exit code alone
        cannot tell them apart -- and two of them open with the word FAIL while
        the ledger-drift paragraph itself contains the word DEPARTED, so the
        markers are chosen to be text no other branch emits.
        """
        nonlocal ok
        with tempfile.TemporaryDirectory() as td:
            t = plant(td, seals, ledger)
            shutil.copy(me, t / "tools" / me.name)
            r = subprocess.run([sys.executable, str(t / "tools" / me.name), *args],
                               capture_output=True, text=True, cwd=t, timeout=120)
        missing = [p for p in present if p not in r.stdout]
        leaked = [a for a in absent if a in r.stdout]
        good = r.returncode == want_exit and not missing and not leaked
        print(f"  {label:<28} "
              + (f"exit {want_exit}, right branch" if good else "CONTROL FAILED"))
        if not good:
            ok = False
            print(f"       exit {r.returncode!r} (want {want_exit!r})")
            if missing:
                print(f"       the branch never said: {missing!r}")
            if leaked:
                print(f"       neighbouring marker leaked: {leaked!r}")
            print(f"       stdout {r.stdout[:400]!r}")

    HOLDS = {"Good.json": ("specs/x.t27", None)}
    ONE_STALE = {"Good.json": ("specs/x.t27", None),
                 "Stale.json": ("specs/x.t27", WRONG)}
    # Text no branch but the named one prints. `DEPARTED` and `FAIL` on their
    # own are not usable: the ledger-drift paragraph says DEPARTED in prose,
    # and the empty-tree precondition also opens with FAIL.
    DRIFT = "A baselined seal changed class, or its file left the tree."
    CHANGED = "CHANGED  Stale.json: phantom -> stale (the repair is not the same one)"
    DEPARTED = "DEPARTED Vanished.json: baselined as broken, and the seal FILE is gone"
    NEWLY = "FAIL: 1 seal(s) newly do not hold"
    NOTE = "baselined seal(s) now hold"
    WROTE = "baseline written"

    # The clean direction first, or every case below passes for free on a
    # program that reds unconditionally. It also proves a planted tree can be
    # GREEN, which this gate's own repository has not been for a long time.
    spawned("end-to-end clean tree", 0, ("OK: 1 seals, 1 hold",),
            ("FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE), HOLDS)

    # main(): the newly-broken verdict. Nothing is baselined, so the stale seal
    # is NEW and the ledger paragraph must stay silent.
    spawned("end-to-end new breakage", 1, (NEWLY, "Stale.json  [stale]"),
            ("OK:", "no seals found at all", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            ONE_STALE)

    # main(): the ledger verdict, reached by a baselined seal whose KIND moved.
    # Nothing is newly broken, so the FAIL branch must stay silent.
    spawned("end-to-end ledger drift", 1, (DRIFT, CHANGED),
            ("FAIL:", "OK:", DEPARTED, NOTE, WROTE),
            ONE_STALE, ledger="Stale.json | phantom | specs/x.t27\n")

    # main(): the SAME verdict reached by the other branch -- a baselined seal
    # whose file left the tree. Each of these two names the other's marker as
    # one that must be absent, because the exit code cannot separate them.
    spawned("end-to-end ledger departure", 1, (DRIFT, DEPARTED),
            ("FAIL:", "OK:", CHANGED, NOTE, WROTE),
            ONE_STALE,
            ledger="Stale.json | stale | specs/x.t27\n"
                   "Vanished.json | stale | specs/gone.t27\n")

    # The configuration the LIVE gate is in every single day, and the one the
    # four cases above never build: a ledger exists AND something outside it is
    # newly broken. The new-breakage case above runs with no ledger at all, so
    # the branch this repository actually takes was proved only in a world it
    # never has. Fresh.json is broken and unbaselined; Stale.json is broken and
    # baselined, so it must NOT be counted as new, and the ledger paragraph must
    # stay silent because nothing in the ledger moved.
    spawned("end-to-end ledger plus new", 1, (NEWLY, "Fresh.json  [stale]"),
            ("OK:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE,
             "Stale.json  [stale]"),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    # NOT covered here, so that "everything else is covered" is not available as
    # a reading: the `dangling` kind, and the compare() path where a seal is
    # both baselined and repaired in the same run. `phantom`/`dangling`
    # movement is suppressed on purpose (a small clone parks 15 entries there),
    # and no planted tree in this file is a git repository, so the branch that
    # asks git whether a spec ever existed is inert in all five cases above.
    # T100: the LEDGER-WRITING path. Every case above runs the verify path;
    # `--update-baseline` writes the ledger and returns success, and
    # `tri gates mutate --loud` showed that success return could be rewritten to
    # a failure with nothing noticing. The same site survived in four gates.
    #
    # Exit AND effect: the exit alone would pass a run that returned 0 without
    # writing, and the marker alone would pass one that wrote and then reported
    # failure -- which is the mutation that found this.
    spawned("end-to-end --update-baseline", 0, (WROTE,),
            ("FAIL:", "OK:", DRIFT, CHANGED, DEPARTED),
            ONE_STALE, args=("--update-baseline",))

    return 0 if (ok and _check_compare()) else 1


def main():
    if "--self-check" in sys.argv:
        return self_check()
    total, bad = scan()
    if total == 0:
        print("FAIL: no seals found at all -- the path is wrong, not the tree")
        return 1

    if "--update-baseline" in sys.argv:
        BASELINE.write_text(
            "# Seals that do not hold today. Each line is a debt, not a rule.\n"
            "# Remove the line when the seal is fixed; the gate then holds it fixed.\n"
            + "".join(f"{n} | {k} | {d}\n" for n, k, d in sorted(bad)))
        print(f"  baseline written: {len(bad)} entries")
        return 0

    known = baseline()
    present = {q.name for q in (ROOT / ".trinity/seals").rglob("*.json")}
    changed, departed, fixed = compare(bad, known, present)
    if changed or departed or fixed:
        for n, was, now in changed:
            print(f"  CHANGED  {n}: {was} -> {now} (the repair is not the same one)")
        for n in departed:
            print(f"  DEPARTED {n}: baselined as broken, and the seal FILE is gone")
        if fixed:
            print(f"  NOTE     {len(fixed)} baselined seal(s) now hold. Drop their lines "
                  f"so the gate holds them: {', '.join(fixed[:5])}"
                  + (f" (+{len(fixed) - 5} more)" if len(fixed) > 5 else ""))
        print()
    new = [b for b in bad if b[0] not in known]
    kinds = {}
    for _, k, _ in bad:
        kinds[k] = kinds.get(k, 0) + 1
    if not new and (changed or departed):
        print("A baselined seal changed class, or its file left the tree. A name in")
        print("the ledger excuses the STATE it was recorded in, not every later one:")
        print("`stale` says re-seal it, `dangling` says restore or remove, and a")
        print("DEPARTED seal is a reproducibility record deleted rather than fixed.")
        print("If deliberate, re-record with --update-baseline in the same commit.")
        return 1
    if not new:
        print(f"OK: {total} seals, {total - len(bad)} hold, {len(bad)} known-broken "
              f"({', '.join(f'{v} {k}' for k, v in sorted(kinds.items()))}) "
              f"listed in {BASELINE.name}")
        return 0
    print(f"FAIL: {len(new)} seal(s) newly do not hold\n")
    for n, k, d in new:
        print(f"  {n}  [{k}]")
        print(f"      {d}")
    print("\n  stale    the spec changed after sealing, so the four gen_hashes describe")
    print("           something it no longer produces. Re-seal it.")
    print("  dangling the spec was committed and later deleted. Remove the seal with it,")
    print("           or restore both.")
    print("  phantom  the spec appears in NO commit. The seal's spec_hash and four")
    print("           gen_hashes name a file nobody can fetch, so there is nothing in")
    print("           the record to check. Find the spec or drop the seal.")
    print(f"\n  Deliberate debt goes in {BASELINE.name} via --update-baseline.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
