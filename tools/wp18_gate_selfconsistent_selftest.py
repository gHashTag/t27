#!/usr/bin/env python3
"""
FAIL-reachable self-test for the WP-33 tri-state extension of wp18_conformance_gate.py.

Verifies that Check B now recognises THREE pack kinds (bitexact / bitexact_selfconsistent
/ structural) and counts each separately, WITHOUT silently promoting a self-consistent
pack to the stronger "bitexact" label. Every positive assertion is paired with a negative
control (a planted defect that MUST flip the gate verdict), per the FAIL-reachable rule.

stdlib only, ZERO local imports beyond the gate under test. ASCII-only. Apache-2.0.

Run:  python3 wp18_gate_selfconsistent_selftest.py
Exit: 0 = all checks pass (incl. all negative controls flip), 1 = a check failed.
"""
import json
import math
import os
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import wp18_conformance_gate as gate  # noqa: E402

PASS = 0
FAIL = 0


def check(label, cond):
    global PASS, FAIL
    if cond:
        PASS += 1
        print("PASS %s" % label)
    else:
        FAIL += 1
        print("FAIL %s" % label)


def _write(path, obj):
    with open(path, "w", encoding="ascii") as fh:
        json.dump(obj, fh)


def _ssot(path, ids):
    with open(path, "w", encoding="ascii") as fh:
        for i in ids:
            fh.write("// CATALOG: id=%s\n" % i)


def _bitexact_pack(name):
    # one finite row that re-derives exactly (abs_error 0), plus an inf special row
    return {
        "format": name,
        "vectors": [
            {"name": "three", "input_f64": 3.0, "decoded_f64": 3.0,
             "abs_error": 0.0, "category": "normal"},
            {"name": "pos_inf", "input_f64": "Infinity", "decoded_f64": "Infinity",
             "abs_error": 0.0, "category": "inf"},
        ],
    }


def _selfconsistent_pack(name):
    # dyadic-exact finite rows, abs_error 0 by construction, honest single-witness
    return {
        "format": name,
        "decode_provenance": "single decode law, no independent second witness",
        "vectors": [
            {"name": "one", "input_f64": 1.0, "decoded_f64": 1.0,
             "abs_error": 0.0, "category": "normal"},
            {"name": "two", "input_f64": 2.0, "decoded_f64": 2.0,
             "abs_error": 0.0, "category": "normal"},
        ],
    }


def build_tree(tmp, kinds):
    """kinds: list of (id, kind). Builds SSOT + vectors dir + INDEX with correct counts.
    Returns (ssot_path, vectors_dir, index_dict)."""
    vec = os.path.join(tmp, "vectors")
    os.makedirs(vec, exist_ok=True)
    ids = [k[0] for k in kinds]
    ssot = os.path.join(tmp, "formats_catalog.t27")
    _ssot(ssot, ids)
    packs = []
    be = sc = st = 0
    for pid, kind in kinds:
        fn = "%s_conformance_v0.json" % pid
        fpath = os.path.join(vec, fn)
        if kind == "structural":
            _write(fpath, {"format": pid, "vectors": []})
            st += 1
        elif kind == "bitexact_selfconsistent":
            _write(fpath, _selfconsistent_pack(pid))
            sc += 1
        else:
            _write(fpath, _bitexact_pack(pid))
            be += 1
        packs.append({"id": pid, "file": fn, "kind": kind,
                      "n_vectors": 0 if kind == "structural" else 2,
                      "sha256": gate._sha256_file(fpath)})
    index = {
        "schema": "t27-conformance-index/v0.1",
        "total_formats": len(ids),
        "total_packs": len(packs),
        "bitexact_packs": be,
        "selfconsistent_packs": sc,
        "structural_packs": st,
        "packs": packs,
    }
    _write(os.path.join(vec, "INDEX_all_formats.json"), index)
    return ssot, vec, index


def run(ssot, vec):
    code, report = gate.run_gate(ssot, vec, None)
    return code, report


def b_check(report):
    return report["checks"]["B_index_counts"]


def main():
    # ----------------------------------------------------------------- T1 / T1b
    # T1: a tree with all three kinds + a correct INDEX -> CLEAN, recount matches.
    with tempfile.TemporaryDirectory() as tmp:
        ssot, vec, idx = build_tree(tmp, [
            ("gf16", "bitexact"),
            ("gf48", "bitexact_selfconsistent"),
            ("gf96", "bitexact_selfconsistent"),
            ("decimal32", "structural"),
        ])
        code, rep = run(ssot, vec)
        b = b_check(rep)
        check("T1 tri-state tree is CLEAN (exit 0)", code == 0)
        check("T1 recount [4,1,2,1] matches claimed",
              b["recount_total_bitexact_selfconsistent_structural"] == [4, 1, 2, 1]
              and b["ok"] is True)

        # T1b NEGATIVE CONTROL: claim the 2 selfconsistent packs as bitexact
        # (selfconsistent_packs=0, bitexact_packs=3) -> Check B MUST fail.
        idxp = json.load(open(os.path.join(vec, "INDEX_all_formats.json"), encoding="ascii"))
        idxp["bitexact_packs"] = 3
        idxp["selfconsistent_packs"] = 0
        _write(os.path.join(vec, "INDEX_all_formats.json"), idxp)
        code_b, rep_b = run(ssot, vec)
        check("T1b CONTROL mislabel sc-as-bitexact -> DRIFT (exit 2)", code_b == 2)
        check("T1b CONTROL Check B not ok", b_check(rep_b)["ok"] is False)

    # ----------------------------------------------------------------- T2 / T2b
    # T2: legacy 2-kind tree with NO selfconsistent key -> defaults to 0, CLEAN.
    with tempfile.TemporaryDirectory() as tmp:
        ssot, vec, idx = build_tree(tmp, [
            ("gf16", "bitexact"),
            ("decimal32", "structural"),
        ])
        # strip the optional key to simulate a legacy INDEX
        idxp = json.load(open(os.path.join(vec, "INDEX_all_formats.json"), encoding="ascii"))
        del idxp["selfconsistent_packs"]
        _write(os.path.join(vec, "INDEX_all_formats.json"), idxp)
        code, rep = run(ssot, vec)
        check("T2 legacy INDEX (no sc key) stays CLEAN (backward compat)", code == 0)
        check("T2 recount selfconsistent defaults to 0",
              b_check(rep)["recount_total_bitexact_selfconsistent_structural"] == [2, 1, 0, 1])

        # T2b NEGATIVE CONTROL: same legacy tree but add a real selfconsistent pack on
        # disk + INDEX entry, leaving total/bitexact stale -> recount must catch it.
        vecdir = vec
        scfn = "gf48_conformance_v0.json"
        _write(os.path.join(vecdir, scfn), _selfconsistent_pack("gf48"))
        idxp = json.load(open(os.path.join(vecdir, "INDEX_all_formats.json"), encoding="ascii"))
        idxp["packs"].append({"id": "gf48", "file": scfn, "kind": "bitexact_selfconsistent",
                              "n_vectors": 2, "sha256": gate._sha256_file(os.path.join(vecdir, scfn))})
        # deliberately do NOT bump total_packs / selfconsistent_packs
        _write(os.path.join(vecdir, "INDEX_all_formats.json"), idxp)
        _ssot(ssot, ["gf16", "decimal32", "gf48"])  # keep Check A happy
        code_b, rep_b = run(ssot, vecdir)
        check("T2b CONTROL added sc pack w/ stale counts -> DRIFT (exit 2)", code_b == 2)

    # ----------------------------------------------------------------- T3 / T3b
    # T3: an UNKNOWN kind is drift, NOT silently folded into bitexact.
    with tempfile.TemporaryDirectory() as tmp:
        ssot, vec, idx = build_tree(tmp, [
            ("gf16", "bitexact"),
            ("decimal32", "structural"),
        ])
        idxp = json.load(open(os.path.join(vec, "INDEX_all_formats.json"), encoding="ascii"))
        idxp["packs"][0]["kind"] = "totally_made_up_kind"
        _write(os.path.join(vec, "INDEX_all_formats.json"), idxp)
        code, rep = run(ssot, vec)
        check("T3 unknown kind -> DRIFT (exit 2)", code == 2)
        check("T3 unknown_kind list is non-empty",
              len(b_check(rep)["unknown_kind"]) == 1)

        # T3b CONTROL: rename it back to the valid kind -> CLEAN again.
        idxp["packs"][0]["kind"] = "bitexact"
        _write(os.path.join(vec, "INDEX_all_formats.json"), idxp)
        code_b, rep_b = run(ssot, vec)
        check("T3b CONTROL valid kind restored -> CLEAN (exit 0)", code_b == 0)

    # ----------------------------------------------------------------- T4 / T4b
    # T4: Check D STILL runs on selfconsistent packs (they are not structural), so a
    # corrupted abs_error in a selfconsistent pack MUST be caught (no free pass).
    with tempfile.TemporaryDirectory() as tmp:
        ssot, vec, idx = build_tree(tmp, [
            ("gf48", "bitexact_selfconsistent"),
        ])
        code, rep = run(ssot, vec)
        check("T4 clean selfconsistent pack -> CLEAN (exit 0)", code == 0)

        # T4b CONTROL: lie about abs_error in a selfconsistent row -> Check D fires.
        scfile = os.path.join(vec, "gf48_conformance_v0.json")
        pk = json.load(open(scfile, encoding="ascii"))
        pk["vectors"][0]["abs_error"] = 0.5  # decoded==input==1.0 so true error is 0
        _write(scfile, pk)
        # refresh sha so Check C does not pre-empt Check D
        idxp = json.load(open(os.path.join(vec, "INDEX_all_formats.json"), encoding="ascii"))
        idxp["packs"][0]["sha256"] = gate._sha256_file(scfile)
        _write(os.path.join(vec, "INDEX_all_formats.json"), idxp)
        code_b, rep_b = run(ssot, vec)
        check("T4b CONTROL corrupted abs_error in sc pack -> caught by D (exit 2)", code_b == 2)
        check("T4b CONTROL Check D not ok",
              rep_b["checks"]["D_rederive_abs_error"]["ok"] is False)

    print("")
    print("SELFTEST RESULT: %d PASS, %d FAIL" % (PASS, FAIL))
    return 0 if FAIL == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
