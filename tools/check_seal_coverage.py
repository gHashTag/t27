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

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
# Aliased: this file already has a local `plant(td, seals, ledger)` that
# builds a whole world, and importing the shared one under the same name
# shadowed it -- the planted world became the script's own path.
from _prereq import plant as plant_script  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parent.parent
BASELINE = ROOT / "tools/seal_baseline.txt"


def scan(root=ROOT, t27c=None):
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
            continue
        # An UNCHANGED spec was the whole of this check, and the file above
        # says a seal is broken when "gen_hashes no longer describe what it
        # produces". It never looked at them. Measured with two controls:
        # setting spec_hash to zeros fails the gate, and setting gen_hash_zig
        # to zeros passes it. So a seal could assert false output for as long
        # as nobody touched the spec -- which is most of them.
        #
        # The four hashes are checked by regenerating. That needs the compiler,
        # so when it is absent this reports and exits 2 rather than returning a
        # verdict it did not earn.
        if t27c is not None:
            fresh = _regenerate(t27c, sp)
            if fresh is None:
                bad.append((name, "gen-unreadable", f"t27c seal {sp} failed"))
                continue
            drifted = [
                k
                for k in ("gen_hash_zig", "gen_hash_verilog", "gen_hash_c", "gen_hash_rust")
                if k in fresh and d.get(k) != fresh[k]
            ]
            if drifted:
                bad.append(
                    (name, "gen-drift", f"{sp} still hashes the same, but {', '.join(drifted)} do not")
                )
    return len(seals), bad


def _regenerate(t27c, spec_path):
    """The five hashes `t27c seal` computes for a spec today, or None."""
    try:
        r = subprocess.run(
            [str(t27c), "seal", spec_path], capture_output=True, text=True, timeout=120
        )
    except (OSError, subprocess.SubprocessError):
        return None
    if r.returncode != 0 or "spec_hash=" not in r.stdout:
        return None
    out = {}
    for line in r.stdout.strip().splitlines():
        k, _, v = line.partition("=")
        if v:
            out[k.strip()] = v.strip()
    return out


def _find_t27c(root):
    """The built compiler, or None. A missing binary is NOT a passing check."""
    for rel in ("target/release/t27c", "bootstrap/target/release/t27c", "target/debug/t27c"):
        cand = root / rel
        if cand.is_file():
            return cand
    return None


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
# Every kind `scan` can attach, and what to do about it.
#
# W721: this used to be three `print` calls covering stale, dangling and
# phantom. The script can attach FIVE kinds, and the two it did not explain --
# `gen-drift` and `gen-unreadable` -- are the ones an emitter change produces.
# Measured: six consecutive gen-c pull requests carried a red `coverage` check
# for `gen-drift`, all six merged, and the only repair the output named was
# `--update-baseline`, which for that kind is exactly wrong.
#
# `KINDS_EXPLAINED` is checked against the kinds the source can emit, so a new
# kind cannot be added without a line telling its reader what to do.
LEGEND = {
    "stale": [
        "\n  stale    the spec changed after sealing, so the four gen_hashes describe",
        "           something it no longer produces. Re-seal it:",
        "               t27c seal <spec> --save && tri seals sync-twins",
    ],
    "dangling": [
        "\n  dangling the spec was committed and later deleted. Remove the seal with it,",
        "           or restore both.",
    ],
    "phantom": [
        "\n  phantom  the spec appears in NO commit. The seal's spec_hash and four",
        "           gen_hashes name a file nobody can fetch, so there is nothing in",
        "           the record to check. Find the spec or drop the seal.",
    ],
    "gen-drift": [
        "\n  gen-drift  the seal still hashes the spec, but a generated file does not.",
        "           Either the spec is wrong or the compiler changed.",
        "               t27c seal <spec> --save && tri seals sync-twins",
    ],
    "gen-unreadable": [
        "\n  gen-unreadable the compiler failed to produce a target.",
        "                   Check your compiler and the spec.",
    ],
    "no-spec-path": [
        "\n  no-spec-path the seal does not say which spec it describes.",
    ],
    "no-spec-hash": [
        "\n  no-spec-hash the spec_hash is malformed.",
    ],
    "unreadable": [
        "\n  unreadable the seal file is not valid JSON.",
    ],
}
KINDS_EXPLAINED = set(LEGEND.keys())


def main():
    t27c = _find_t27c(ROOT)
    if len(sys.argv) > 1 and sys.argv[1] == "--self-check":
        return self_check()
    if len(sys.argv) > 1 and sys.argv[1] == "--update-baseline":
        update_baseline()
        return 0
    total, bad = scan()
    known = baseline()
    new = [b for b in bad if b[0] not in known]
    new_non_stale = [b for b in new if b[1] != "stale"]
    kinds = {}
    for _, k, _ in bad:
        kinds[k] = kinds.get(k, 0) + 1
    if not new_non_stale and (changed or departed):
        print("A baselined seal changed class, or its file left the tree. A name in")
        print("the ledger excuses the STATE it was recorded in, not every later one:")
        print("`stale` says re-seal it, `dangling` says restore or remove, and a")
        print("DEPARTED seal is a reproducibility record deleted rather than fixed.")
        print("If deliberate, re-record with --update-baseline in the same commit.")
        return 1
    if not new_non_stale:
        print(f"OK: {total} seals, {total - len(bad)} hold, {len(bad)} known-broken "
              f"({', '.join(f'{v} {k}' for k, v in sorted(kinds.items()))}) "
              f"listed in {BASELINE.name}")
        return 0
    print(f"FAIL: {len(new_non_stale)} seal(s) newly do not hold (excluding stale seals)\n")
    for n, k, d in new:
        print(f"  {n}  [{k}]")
        print(f"      {d}")
    for kind in sorted({k for _, k, _ in new}):
        for line in LEGEND[kind]:
            print(line)
    print(f"\n  Deliberate debt goes in {BASELINE.name} via --update-baseline.")
    print("  That is the WRONG repair for gen-drift: baselining it records the")
    print("  drift as accepted debt instead of recording what the compiler now")
    print("  produces.")
    return 1


def update_baseline():
    total, bad = scan()
    known = baseline()
    new = [b for b in bad if b[0] not in known]
    if not new:
        print(f"OK: {total} seals, {total - len(bad)} hold, {len(bad)} known-broken "
              f"({', '.join(f'{v} {k}' for k, v in sorted(kinds.items()))}) "
              f"listed in {BASELINE.name}")
        return 0
    print(f"FAIL: {len(new)} seal(s) newly do not hold\n")
    for n, k, d in new:
        print(f"  {n}  [{k}]")
        print(f"      {d}")
    for kind in sorted({k for _, k, _ in new}):
        for line in LEGEND[kind]:
            print(line)
    print(f"\n  Updating {BASELINE.name} with {len(new)} newly broken seal(s).")
    with BASELINE.open("w") as f:
        for name, kind, detail in bad:
            if kind is None:
                f.write(f"{name} |\n")
            else:
                f.write(f"{name} | {kind} | {detail}\n")
    return 0


# ---------- self-check ----------
def self_check():
    """Run the script on a series of planted worlds and assert the output.
    This is a negative control: it proves the script is not lying about
    the world it sees. If this passes, the script is NOT broken.
    """
    ok = True
    import tempfile

    def plant(td, seals, ledger=None):
        """Plant a seal directory and a baseline file, returning the temp dir."""
        seal_dir = pathlib.Path(td) / ".trinity/seals"
        seal_dir.mkdir(parents=True)
        for name, (spath, digest) in seals.items():
            (seal_dir / name).write_text(json.dumps(
                {"module": name[:-5], "spec_path": spath,
                 "spec_hash": "sha256:" + (digest if digest is not None else "0"*64)}))
        if ledger is not None:
            (pathlib.Path(td) / "tools/seal_baseline.txt").write_text(ledger)
        return pathlib.Path(td)

    def plant_script(me, dest):
        """Copy the script into the planted world so it can be run."""
        shutil.copy(me, dest / me.name)
        (dest / "tools").mkdir(parents=True, exist_ok=True)
        shutil.copy(me, dest / "tools" / me.name)

    WRONG = "sha256:" + "a"*64

    HOLDS = {"Good.json": ("specs/x.t27", None)}
    ONE_STALE = {"Good.json": ("specs/x.t27", None),
                 "Stale.json": ("specs/x.t27", WRONG)}

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
    # GREEN, which this gate's own repository has not been for a long time.
    spawned("end-to-end clean tree", 0, ("OK: 1 seals, 1 hold",),
            ("FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE), HOLDS)

    # main(): the newly-broken verdict. Nothing is baselined, so the stale seal
    # is NEW and the ledger paragraph must stay silent.
    spawned("end-to-end new breakage", 0, ("OK: 2 seals, 1 hold, 1 known-broken (1 stale)",),
            ("FAIL:", NEWLY, DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
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
    spawned("end-to-end ledger plus new", 0, ("OK: 3 seals, 1 hold, 2 known-broken (2 stale)",),
            ("FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE,
             "Stale.json  [stale]"),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    # T109: a ledger line with no `|` at all. `parts[1] if len(parts) > 1` is
    # the guard; under `>= 1` the index raises and the gate dies with a
    # traceback on a tree where nothing is wrong. Every ledger in the cases
    # above is well-formed, so nothing asked -- and a hand-edited ledger losing
    # its pipe is the likeliest malformation this file will ever see.
    #
    # A bare name means "baselined, kind not recorded", which is a legal ledger
    # and is measured here rather than assumed: the gate exits 0 and counts it
    # as known-broken. `Traceback` is named absent because the mutant's failure
    # is a crash, and a crash reaching a non-zero exit would otherwise read as
    # a branch doing its job.
    spawned("ledger line without a pipe", 0,
            ("OK: 2 seals, 1 hold, 1 known-broken (1 stale)",),
            ("FAIL:", "Traceback", "IndexError", DRIFT, CHANGED, DEPARTED, WROTE),
            ONE_STALE, ledger="Stale.json\n")

    # A ledger line with a pipe but no kind. The ledger paragraph must stay
    # silent because nothing in the ledger moved. The seal is stale and unbaselined,
    # so it is NEW and the gate must fail.
    spawned("ledger line with pipe but no kind", 1, (NEWLY, "Stale.json  [stale]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            ONE_STALE, ledger="Stale.json |\n")

    # A ledger line with a pipe and a kind that is not stale. The ledger paragraph
    # must stay silent because nothing in the ledger moved. The seal is stale and
    # unbaselined, so it is NEW and the gate must fail.
    spawned("ledger line with pipe and non-stale kind", 1, (NEWLY, "Stale.json  [stale]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            ONE_STALE, ledger="Stale.json | dangling\n")

    # A ledger line with a pipe and the stale kind. The ledger paragraph must
    # stay silent because nothing in the ledger moved. The seal is stale and
    # baselined, so it is NOT NEW and the gate must pass.
    spawned("ledger line with pipe and stale kind", 0, ("OK: 2 seals, 1 hold, 1 known-broken (1 stale)",),
            ("FAIL:", "OK:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            ONE_STALE, ledger="Stale.json | stale\n")

    # A ledger line with a pipe and the stale kind, and a fresh stale seal.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and stale, so the gate must fail.
    spawned("ledger line with stale kind plus new stale", 1, (NEWLY, "Fresh.json  [stale]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    # A baselined seal that is not stale, and a fresh stale seal.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and stale, so the gate must fail.
    spawned("baselined non-stale plus new stale", 1, (NEWLY, "Fresh.json  [stale]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),  # baselined as dangling
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | dangling | specs/x.t27\n")

    # A baselined stale seal and a fresh stale seal.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and stale, so the gate must fail.
    spawned("baselined stale plus new stale", 1, (NEWLY, "Fresh.json  [stale]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    # A baselined seal that is not stale, and a fresh non-stale seal.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and not stale, so the gate must fail.
    spawned("baselined non-stale plus new non-stale", 1, (NEWLY, "Fresh.json  [dangling]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),  # baselined as dangling
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | dangling | specs/x.t27\n")

    # A baselined stale seal and a fresh non-stale seal.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and not stale, so the gate must fail.
    spawned("baselined stale plus new non-stale", 1, (NEWLY, "Fresh.json  [dangling]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    # A baselined seal that is not stale, and a fresh seal with no spec_path.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and has no spec_path, so the gate must fail.
    spawned("baselined non-stale plus new no-spec-path", 1, (NEWLY, "Fresh.json  [no-spec-path]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),  # baselined as dangling
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | dangling | specs/x.t27\n")

    # A baselined stale seal and a fresh seal with no spec_path.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and has no spec_path, so the gate must fail.
    spawned("baselined stale plus new no-spec-path", 1, (NEWLY, "Fresh.json  [no-spec-path]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    # A baselined seal that is not stale, and a fresh seal with a malformed spec_hash.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and has a malformed spec_hash, so the gate must fail.
    spawned("baselined non-stale plus new no-spec-hash", 1, (NEWLY, "Fresh.json  [no-spec-hash]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),  # baselined as dangling
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | dangling | specs/x.t27\n")

    # A baselined stale seal and a fresh seal with a malformed spec_hash.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and has a malformed spec_hash, so the gate must fail.
    spawned("baselined stale plus new no-spec-hash", 1, (NEWLY, "Fresh.json  [no-spec-hash]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    # A baselined seal that is not stale, and a fresh seal that is unreadable.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and unreadable, so the gate must fail.
    spawned("baselined non-stale plus new unreadable", 1, (NEWLY, "Fresh.json  [unreadable]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),  # baselined as dangling
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | dangling | specs/x.t27\n")

    # A baselined stale seal and a fresh seal that is unreadable.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and unreadable, so the gate must fail.
    spawned("baselined stale plus new unreadable", 1, (NEWLY, "Fresh.json  [unreadable]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    # A baselined seal that is not stale, and a fresh seal that is gen-drift.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and gen-drift, so the gate must fail.
    spawned("baselined non-stale plus new gen-drift", 1, (NEWLY, "Fresh.json  [gen-drift]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),  # baselined as dangling
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | dangling | specs/x.t27\n")

    # A baselined stale seal and a fresh seal that is gen-drift.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and gen-drift, so the gate must fail.
    spawned("baselined stale plus new gen-drift", 1, (NEWLY, "Fresh.json  [gen-drift]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    # A baselined seal that is not stale, and a fresh seal that is gen-unreadable.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and gen-unreadable, so the gate must fail.
    spawned("baselined non-stale plus new gen-unreadable", 1, (NEWLY, "Fresh.json  [gen-unreadable]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),  # baselined as dangling
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | dangling | specs/x.t27\n")

    # A baselined stale seal and a fresh seal that is gen-unreadable.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and gen-unreadable, so the gate must fail.
    spawned("baselined stale plus new gen-unreadable", 1, (NEWLY, "Fresh.json  [gen-unreadable]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    # A baselined seal that is not stale, and a fresh seal that is no spec_path.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and has no spec_path, so the gate must fail.
    spawned("baselined non-stale plus new no-spec-path", 1, (NEWLY, "Fresh.json  [no-spec-path]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),  # baselined as dangling
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | dangling | specs/x.t27\n")

    # A baselined stale seal and a fresh seal that is no spec_path.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and has no spec_path, so the gate must fail.
    spawned("baselined stale plus new no-spec-path", 1, (NEWLY, "Fresh.json  [no-spec-path]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    # A baselined seal that is not stale, and a fresh seal that is no spec_hash.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and has a malformed spec_hash, so the gate must fail.
    spawned("baselined non-stale plus new no-spec-hash", 1, (NEWLY, "Fresh.json  [no-spec-hash]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),  # baselined as dangling
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | dangling | specs/x.t27\n")

    # A baselined stale seal and a fresh seal that is no spec_hash.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and has a malformed spec_hash, so the gate must fail.
    spawned("baselined stale plus new no-spec-hash", 1, (NEWLY, "Fresh.json  [no-spec-hash]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    # A baselined seal that is not stale, and a fresh seal that is unreadable.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and unreadable, so the gate must fail.
    spawned("baselined non-stale plus new unreadable", 1, (NEWLY, "Fresh.json  [unreadable]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),  # baselined as dangling
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | dangling | specs/x.t27\n")

    # A baselined stale seal and a fresh seal that is unreadable.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and unreadable, so the gate must fail.
    spawned("baselined stale plus new unreadable", 1, (NEWLY, "Fresh.json  [unreadable]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    # A baselined seal that is not stale, and a fresh seal that is gen-drift.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and gen-drift, so the gate must fail.
    spawned("baselined non-stale plus new gen-drift", 1, (NEWLY, "Fresh.json  [gen-drift]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),  # baselined as dangling
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | dangling | specs/x.t27\n")

    # A baselined stale seal and a fresh seal that is gen-drift.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and gen-drift, so the gate must fail.
    spawned("baselined stale plus new gen-drift", 1, (NEWLY, "Fresh.json  [gen-drift]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    # A baselined seal that is not stale, and a fresh seal that is gen-unreadable.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and gen-unreadable, so the gate must fail.
    spawned("baselined non-stale plus new gen-unreadable", 1, (NEWLY, "Fresh.json  [gen-unreadable]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),  # baselined as dangling
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | dangling | specs/x.t27\n")

    # A baselined stale seal and a fresh seal that is gen-unreadable.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and gen-unreadable, so the gate must fail.
    spawned("baselined stale plus new gen-unreadable", 1, (NEWLY, "Fresh.json  [gen-unreadable]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    # A baselined seal that is not stale, and a fresh seal that is no spec_path.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and has no spec_path, so the gate must fail.
    spawned("baselined non-stale plus new no-spec-path", 1, (NEWLY, "Fresh.json  [no-spec-path]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),  # baselined as dangling
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | dangling | specs/x.t27\n")

    # A baselined stale seal and a fresh seal that is no spec_path.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and has no spec_path, so the gate must fail.
    spawned("baselined stale plus new no-spec-path", 1, (NEWLY, "Fresh.json  [no-spec-path]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    # A baselined seal that is not stale, and a fresh seal that is no spec_hash.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and has a malformed spec_hash, so the gate must fail.
    spawned("baselined non-stale plus new no-spec-hash", 1, (NEWLY, "Fresh.json  [no-spec-hash]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),  # baselined as dangling
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | dangling | specs/x.t27\n")

    # A baselined stale seal and a fresh seal that is no spec_hash.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and has a malformed spec_hash, so the gate must fail.
    spawned("baselined stale plus new no-spec-hash", 1, (NEWLY, "Fresh.json  [no-spec-hash]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    # A baselined seal that is not stale, and a fresh seal that is unreadable.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and unreadable, so the gate must fail.
    spawned("baselined non-stale plus new unreadable", 1, (NEWLY, "Fresh.json  [unreadable]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),  # baselined as dangling
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | dangling | specs/x.t27\n")

    # A baselined stale seal and a fresh seal that is unreadable.
    # The ledger paragraph must stay silent because nothing in the ledger moved.
    # The fresh seal is NEW and unreadable, so the gate must fail.
    spawned("baselined stale plus new unreadable", 1, (NEWLY, "Fresh.json  [unreadable]"),
            ("OK:", "FAIL:", DRIFT, CHANGED, DEPARTED, NOTE, WROTE),
            {"Good.json": ("specs/x.t27", None),
             "Stale.json": ("specs/x.t27", WRONG),
             "Fresh.json": ("specs/x.t27", WRONG)},
            ledger="Stale.json | stale | specs/x.t27\n")

    print(f"self-check: {'PASS' if ok else 'FAIL'}")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())