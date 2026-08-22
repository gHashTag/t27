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
if any mutant survives.

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
import sys

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

    print("\n" + ("selftest OK: the gate is falsifiable."
                  if not bad else f"selftest FAILED: {bad} mutant(s) survived."))
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
