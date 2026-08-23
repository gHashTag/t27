#!/usr/bin/env python3
"""Lock INDEX_all_formats.json against the packs it summarises.

WHY
---
Honesty rule #10 -- a pack is not labelled bit-precise without an independent
second witness -- is the corpus's central claim. The pack file is the artefact of
record for its own status; the index is a summary consumers and papers read.
Nothing checked that the two agreed.

That gap was not hypothetical. gen_all_formats.py hardcoded
`kind: bitexact_selfconsistent` for the six wide GoldenFloat rungs, which was
correct when written. The rungs later acquired independent second witnesses and
were promoted in the pack files; the generator was never updated. Re-running it
rewrote the index from 75/0/8 back to 69/6/8 -- silently reverting an honesty-rule
promotion the packs themselves record, with no gate to catch it.

This gate closes that class rather than that instance. It is stdlib-only, needs no
network, and is falsifiable: --selftest plants a mutant for every check and fails
if any mutant survives, and then runs the whole SCRIPT against a planted tree --
clean, failing, and index-less -- because a mutant inside check() cannot tell you
whether check()'s answer still reaches the process exit code.

CHECKS
------
  A  every index entry names a pack file that exists
  B  the recorded sha256 matches the file on disk
  C  kind agrees with the pack: bitexact + witnesses[] => "bitexact",
     and a pack that is not bitexact is not labelled anything else.
     An empty witnesses[] on a bitexact pack is NOT a failure -- see the note at
     the check itself for why demanding one everywhere misstates rule #10.
  D  the index witness count equals len(pack["witnesses"])
  E  the header totals equal the entries actually present
  F  no pack file in the directory is missing from the index

Usage:
    python3 tools/pack_index_consistency_gate.py [--vectors DIR] [--json OUT]
    python3 tools/pack_index_consistency_gate.py --selftest
"""
from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DEFAULT_VECTORS = os.path.join(REPO, "conformance", "vectors")
INDEX_NAME = "INDEX_all_formats.json"


# ---------------------------------------------------------------------------
# The checks. Each returns a list of failure strings.
# ---------------------------------------------------------------------------
def check(index: dict, packs: dict, on_disk: set, notes: list | None = None) -> list[str]:
    """packs maps filename -> parsed pack (or None if unreadable/absent).

    Anything appended to `notes` is reported but does not fail the gate.
    """
    if notes is None:
        notes = []
    fails = []
    entries = index.get("packs", [])

    for e in entries:
        cid, fname = e.get("id", "<no id>"), e.get("file", "")
        pack = packs.get(fname)

        # A -- the file exists
        if pack is None:
            fails.append(f"A {cid}: index names {fname!r}, which is not readable")
            continue

        # B -- digest matches
        recorded = e.get("sha256")
        actual = pack["__sha256__"]
        # T72: `is not None` caught an empty string but not an ABSENT key, so
        # `del entry["sha256"]` turned a caught tamper into CLEAN here and in
        # wp18 both. The condition alone is not enough -- `recorded[:12]` on
        # None raises -- so the message names which of the two states it is.
        if recorded != actual:
            shown = ("<no sha256 key>" if recorded is None
                     else "<empty sha256>" if recorded == ""
                     else f"{recorded[:12]}...")
            fails.append(f"B {cid}: sha256 {shown} recorded, "
                         f"{actual[:12]}... on disk")

        # C -- tier agrees with the pack's own state.
        #
        # An ABSENT bitexact key is unknown, not false. Five hand-curated packs
        # predate the flag and carry no such key while the index labels them
        # bitexact; reading silence as a denial would turn those into failures and
        # the gate would be asserting something the pack never said. They are
        # counted as notes instead.
        declared = pack.get("bitexact", None)
        bitexact = bool(declared)
        witnessed = bool(pack.get("witnesses"))
        kind = e.get("kind")
        if declared is None:
            notes.append(f"{cid}: pack declares no bitexact flag; index says "
                         f"kind={kind!r} (hand-curated pack predating the flag)")
            continue
        #
        # Deliberately NOT checked: a bitexact pack with an empty witnesses[].
        # That is the normal case, not a violation. The 60-odd uncontested packs
        # were bit-precise from the start, their independent reference codec being
        # part of how the vectors were generated; witnesses[] records the packs
        # whose promotion was CONTESTED and had to be argued. A check demanding a
        # witness everywhere would fail the whole corpus and would misstate rule
        # #10. The asymmetry is the point: a recorded witness must not be ignored,
        # but its absence is not evidence of anything.
        if bitexact and witnessed:
            if kind != "bitexact":
                fails.append(f"C {cid}: pack is bitexact with "
                             f"{len(pack['witnesses'])} witness(es) but index "
                             f"says kind={kind!r}, not 'bitexact'")
        elif not bitexact:
            if kind not in ("structural", None):
                fails.append(f"C {cid}: pack is not bitexact but index says "
                             f"kind={kind!r}")

        # D -- witness count agrees
        if "witnesses" in e:
            want = len(pack.get("witnesses", []) or [])
            if e["witnesses"] != want:
                fails.append(f"D {cid}: index says {e['witnesses']} witness(es), "
                             f"pack has {want}")

    # E -- header totals
    kinds = [e.get("kind") for e in entries]
    for key, want in (("total_packs", len(entries)),
                      ("bitexact_packs", kinds.count("bitexact")),
                      ("structural_packs", kinds.count("structural"))):
        if key in index and index[key] != want:
            fails.append(f"E header {key}={index[key]}, entries give {want}")
    if "witnessed_packs" in index:
        want = sum(1 for e in entries if e.get("witnesses"))
        if index["witnessed_packs"] != want:
            fails.append(f"E header witnessed_packs={index['witnessed_packs']}, "
                         f"entries give {want}")

    # F -- nothing on disk is left out of the index
    listed = {e.get("file") for e in entries}
    for fname in sorted(on_disk - listed):
        fails.append(f"F {fname} is a pack file but appears in no index entry")

    return fails


# ---------------------------------------------------------------------------
def load(vectors_dir: str):
    index_path = os.path.join(vectors_dir, INDEX_NAME)
    if not os.path.exists(index_path):
        print(f"no {INDEX_NAME} in {vectors_dir}", file=sys.stderr)
        raise SystemExit(3)
    index = json.load(open(index_path))

    packs, on_disk = {}, set()
    for fname in os.listdir(vectors_dir):
        if not fname.endswith(".json") or fname == INDEX_NAME:
            continue
        blob = open(os.path.join(vectors_dir, fname), "rb").read()
        try:
            pack = json.loads(blob)
        except json.JSONDecodeError:
            continue
        # a conformance pack, not some other json living in the directory
        if "vectors" not in pack and "bitexact" not in pack:
            continue
        # instance_*.json carry kind: "instance" -- an instance-level pack for one
        # fixed parameterization beneath a catalog-STRUCTURAL family. The index has
        # one entry per catalog format, so these belong outside it by design.
        if pack.get("kind") == "instance":
            continue
        pack["__sha256__"] = hashlib.sha256(blob).hexdigest()
        packs[fname] = pack
        on_disk.add(fname)
    return index, packs, on_disk


# ---------------------------------------------------------------------------
# Self-test: every check must be reachable, or the gate is decoration.
# ---------------------------------------------------------------------------
def selftest() -> int:
    base_index = {
        "total_packs": 2, "bitexact_packs": 1, "structural_packs": 1,
        "witnessed_packs": 1,
        "packs": [
            {"id": "alpha", "file": "alpha.json", "kind": "bitexact",
             "sha256": "", "witnesses": 1},
            {"id": "beta", "file": "beta.json", "kind": "structural",
             "sha256": "", "witnesses": 0},
        ],
    }
    base_packs = {
        "alpha.json": {"bitexact": True, "witnesses": [{"kind": "sw"}],
                       "vectors": [], "__sha256__": "aa"},
        "beta.json": {"bitexact": False, "vectors": [], "__sha256__": "bb"},
    }
    base_index["packs"][0]["sha256"] = "aa"
    base_index["packs"][1]["sha256"] = "bb"
    on_disk = {"alpha.json", "beta.json"}

    clean = check(copy.deepcopy(base_index), copy.deepcopy(base_packs), set(on_disk))
    if clean:
        print("SELFTEST BROKEN: the clean fixture already fails:")
        for f in clean:
            print("   ", f)
        return 1

    def mutate(name, letter, fn):
        idx, pks, disk = (copy.deepcopy(base_index), copy.deepcopy(base_packs),
                          set(on_disk))
        fn(idx, pks, disk)
        got = check(idx, pks, disk)
        killed = any(f.startswith(letter + " ") for f in got)
        print(f"  [{'PASS' if killed else 'FAIL'}] {name}")
        if not killed:
            print(f"         mutant survived; findings were: {got}")
        return 0 if killed else 1

    def m_missing(i, p, d):
        p.pop("alpha.json"); d.discard("alpha.json")

    def m_digest(i, p, d):
        p["alpha.json"]["__sha256__"] = "zz"

    def m_demote(i, p, d):
        # exactly the pass-48 regression: a witnessed, bitexact pack demoted
        i["packs"][0]["kind"] = "bitexact_selfconsistent"

    def m_count(i, p, d):
        i["packs"][0]["witnesses"] = 7

    def m_header(i, p, d):
        i["bitexact_packs"] = 99

    def m_orphan(i, p, d):
        p["gamma.json"] = {"bitexact": False, "vectors": [], "__sha256__": "cc"}
        d.add("gamma.json")

    print("selftest -- every check must be FAIL-reachable:")
    bad = 0
    bad += mutate("A missing pack file is caught", "A", m_missing)
    bad += mutate("B digest drift is caught", "B", m_digest)
    bad += mutate("C witnessed pack demoted in index is caught", "C", m_demote)
    bad += mutate("D witness count mismatch is caught", "D", m_count)
    bad += mutate("E header total drift is caught", "E", m_header)
    bad += mutate("F pack absent from the index is caught", "F", m_orphan)

    # -----------------------------------------------------------------------
    # The six mutants above prove check(). They do NOT prove that check()'s
    # answer reaches the process exit code, and they never call load() at all,
    # so both of this gate's own verdicts were unexercised:
    #
    #   load()'s `raise SystemExit(3)`      -- nothing above builds a world
    #                                          without an index in it
    #   main()'s `return 0 if not fails else 1`
    #                                       -- forced to `return 0` the gate
    #                                          still prints "verdict: FAIL",
    #                                          exits 0, and every mutant above
    #                                          stays red
    #
    # The measured version of that second class: check_catalog_integrity.py with
    # main()'s `return 1` rewritten to `return 0` printed OK on a broken catalog
    # while its control reported every branch red. So the cases below run the
    # SCRIPT and read its exit status, not just its findings list.
    #
    # No new lever is introduced to aim it: the script is COPIED into the planted
    # tree, so the copy's module-level REPO/DEFAULT_VECTORS resolve inside that
    # tree by the ordinary dirname(dirname(__file__)) rule and it runs with no
    # arguments at all.
    #
    # NOT covered here, said plainly rather than left to be inferred from a
    # green: the --json output file, the JSONDecodeError skip and the
    # kind=="instance" skip in load(), and the "pack declares no bitexact flag"
    # notes branch in check(). None of those is a verdict site, so no mutant is
    # generated for them, but neither is any of them exercised end to end.
    def plant(root, index):
        """Build tools/ + conformance/vectors/ under root; return the script."""
        tools = os.path.join(root, "tools")
        vectors = os.path.join(root, "conformance", "vectors")
        os.makedirs(tools)
        os.makedirs(vectors)
        me = os.path.basename(os.path.abspath(__file__))
        shutil.copyfile(os.path.abspath(__file__), os.path.join(tools, me))
        digests = {}
        for fname, pack in (("alpha.json", {"bitexact": True,
                                            "witnesses": [{"kind": "sw"}],
                                            "vectors": []}),
                            ("beta.json", {"bitexact": False, "vectors": []})):
            blob = json.dumps(pack, indent=2).encode()
            with open(os.path.join(vectors, fname), "wb") as fh:
                fh.write(blob)
            digests[fname] = hashlib.sha256(blob).hexdigest()
        # index is None => a well-formed world that is simply missing its index,
        # which is the only way to reach load()'s SystemExit(3).
        if index is not None:
            for e in index["packs"]:
                e["sha256"] = digests[e["file"]]
            with open(os.path.join(vectors, INDEX_NAME), "w") as fh:
                json.dump(index, fh, indent=2)
        return os.path.join(tools, me)

    def run_planted(index):
        root = tempfile.mkdtemp(prefix="pack-index-selftest-")
        try:
            proc = subprocess.run([sys.executable, plant(root, index)],
                                  capture_output=True, text=True)
            return proc.returncode, proc.stdout, proc.stderr
        finally:
            shutil.rmtree(root, ignore_errors=True)

    def planted_index():
        return {"total_packs": 2, "bitexact_packs": 1, "structural_packs": 1,
                "witnessed_packs": 1,
                "packs": [
                    {"id": "alpha", "file": "alpha.json", "kind": "bitexact",
                     "sha256": "", "witnesses": 1},
                    {"id": "beta", "file": "beta.json", "kind": "structural",
                     "sha256": "", "witnesses": 0},
                ]}

    def report(name, checks):
        """checks: (holds, what-was-required). Every one of them must hold.

        An exit code alone is not enough: these paths reach one code from
        several branches, and a crash reaches most of them too. So each case
        below names the message it demands AND the sibling messages that must
        stay silent.
        """
        missing = [why for holds, why in checks if not holds]
        print(f"  [{'PASS' if not missing else 'FAIL'}] {name}")
        for why in missing:
            print(f"         required {why}")
        return 1 if missing else 0

    print("\nwhole-program -- check() reaching the process exit code:")

    rc, out, err = run_planted(planted_index())
    bad += report("a clean planted tree exits 0 and reports CLEAN", [
        (rc == 0, f"exit 0, got {rc}"),
        ("verdict: CLEAN" in out, "'verdict: CLEAN' on stdout"),
        ("failures      : 0" in out, "a failure count of zero"),
        ("verdict: FAIL" not in out, "no FAIL verdict alongside it"),
        ("Traceback" not in err, f"no traceback; stderr was {err.strip()!r}"),
    ])

    broken = planted_index()
    broken["packs"][0]["witnesses"] = 7      # D alone: witnessed_packs stays 1
    rc, out, err = run_planted(broken)
    siblings = [ltr for ltr in "ABCEF" if f"\n    {ltr} " in out]
    bad += report("a failing planted tree exits 1 and reports FAIL", [
        (rc == 1, f"exit 1, got {rc} -- main()'s verdict must reach the caller"),
        ("verdict: FAIL" in out, "'verdict: FAIL' on stdout"),
        ("\n    D alpha: index says 7 witness(es), pack has 1" in out,
         "the D line naming the planted mismatch"),
        ("failures      : 1" in out, "a failure count of exactly one"),
        (not siblings, f"every sibling check silent; {siblings} fired too"),
        ("verdict: CLEAN" not in out, "no CLEAN verdict alongside it"),
        ("Traceback" not in err, f"no traceback; stderr was {err.strip()!r}"),
    ])

    rc, out, err = run_planted(None)
    bad += report("a vectors dir with no index exits 3 and reports nothing", [
        (rc == 3, f"exit 3, got {rc} -- load() must abort, not fall through"),
        (f"no {INDEX_NAME} in " in err, "the missing-index line on stderr"),
        ("verdict:" not in out, "no verdict; load() never returned"),
        ("index entries" not in out, "no report; the run never got that far"),
        ("Traceback" not in err, f"no traceback; stderr was {err.strip()!r}"),
    ])

    print("\n" + ("selftest OK: the gate is falsifiable."
                  if not bad else f"selftest FAILED: {bad} case(s) survived."))
    return 1 if bad else 0


# ---------------------------------------------------------------------------
def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--vectors", default=DEFAULT_VECTORS)
    ap.add_argument("--json", dest="json_out")
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()

    if args.selftest:
        return selftest()

    index, packs, on_disk = load(args.vectors)
    notes: list[str] = []
    fails = check(index, packs, on_disk, notes)

    print(f"index entries : {len(index.get('packs', []))}")
    print(f"pack files    : {len(on_disk)}")
    print(f"notes         : {len(notes)}")
    for n in notes:
        print("    note:", n)
    print(f"failures      : {len(fails)}")
    for f in fails:
        print("   ", f)

    verdict = "CLEAN" if not fails else "FAIL"
    print(f"\nverdict: {verdict}")

    if args.json_out:
        with open(args.json_out, "w") as fh:
            json.dump({"schema": "pack-index-consistency/v1",
                       "vectors_dir": os.path.abspath(args.vectors),
                       "entries": len(index.get("packs", [])),
                       "pack_files": len(on_disk),
                       "failures": fails,
                       "verdict": verdict}, fh, indent=2)
    return 0 if not fails else 1


if __name__ == "__main__":
    raise SystemExit(main())
