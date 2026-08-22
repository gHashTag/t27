#!/usr/bin/env python3
"""
WP-18 self-test for wp18_conformance_gate.py (stdlib only).

A gate that can only ever say CLEAN is worthless. This self-test demonstrates the gate
is FALSIFIABLE: for every check (A, B, C, D, D2, E) we plant exactly one defect in a
synthetic mini-corpus and assert the gate FAILS that specific check; we also assert the
clean corpus PASSES, that the verdict is DETERMINISTIC across repeated runs, and that
the gate NEVER emits a capability/benchmark metric.

This is the metamorphic "mutation-kill" discipline: a test suite is only trustworthy if
each planted mutant is killed by at least one assertion.

Exit 0 = all self-tests pass. Exit 1 = a self-test failed (gate is not falsifiable).
ASCII-only. Apache-2.0.
"""
import copy
import hashlib
import json
import os
import shutil
import tempfile

import wp18_conformance_gate as G


SSOT_TEXT = "\n".join([
    "// CATALOG: id=fp8_e4m3",
    "// CATALOG: id=fp8_e5m2",
    "// CATALOG: id=gf16",
]) + "\n"


def _sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        h.update(fh.read())
    return h.hexdigest()


def _bitexact_pack(pid, rows):
    return {"schema": "t27-conformance/v0", "format": pid, "n_vectors": len(rows), "vectors": rows}


def _structural_pack(pid):
    return {"schema": "t27-conformance/v0", "format": pid, "kind": "structural", "n_vectors": 0, "vectors": []}


def build_clean_corpus(root):
    """A small, fully-consistent corpus: 2 bit-exact packs + 1 structural pack."""
    vec = os.path.join(root, "vectors")
    os.makedirs(vec, exist_ok=True)

    ssot_path = os.path.join(root, "formats_catalog.t27")
    with open(ssot_path, "w", encoding="ascii") as fh:
        fh.write(SSOT_TEXT)

    # fp8_e4m3: one exact finite row + one nan-code row + one inf-code row
    e4m3 = _bitexact_pack("fp8_e4m3", [
        {"name": "pos_1p0", "input_f64": 1.0, "decoded_f64": 1.0, "abs_error": 0.0, "category": "normal"},
        {"name": "nan_code", "input_f64": "nan", "decoded_f64": "nan", "abs_error": "nan", "category": "nan"},
    ])
    # fp8_e5m2: one exact row + one inf-code row (inf -> inf, abs_error placeholder 0.0)
    e5m2 = _bitexact_pack("fp8_e5m2", [
        {"name": "pos_2p0", "input_f64": 2.0, "decoded_f64": 2.0, "abs_error": 0.0, "category": "normal"},
        {"name": "pos_inf", "input_f64": "inf", "decoded_f64": "inf", "abs_error": 0.0, "category": "inf"},
    ])
    # gf16 declared structural here (no bit-exact rows) to exercise the structural path
    gf16 = _structural_pack("gf16")

    files = {
        "fp8_e4m3_conformance_v0.json": e4m3,
        "fp8_e5m2_conformance_v0.json": e5m2,
        "gf16_conformance_v0.json": gf16,
    }
    for fn, obj in files.items():
        with open(os.path.join(vec, fn), "w", encoding="ascii") as fh:
            json.dump(obj, fh)

    packs = [
        {"id": "fp8_e4m3", "file": "fp8_e4m3_conformance_v0.json", "kind": "bitexact",
         "sha256": _sha(os.path.join(vec, "fp8_e4m3_conformance_v0.json"))},
        {"id": "fp8_e5m2", "file": "fp8_e5m2_conformance_v0.json", "kind": "bitexact",
         "sha256": _sha(os.path.join(vec, "fp8_e5m2_conformance_v0.json"))},
        {"id": "gf16", "file": "gf16_conformance_v0.json", "kind": "structural",
         "sha256": _sha(os.path.join(vec, "gf16_conformance_v0.json"))},
    ]
    index = {"total_packs": 3, "bitexact_packs": 2, "structural_packs": 1, "packs": packs}
    with open(os.path.join(vec, "INDEX_all_formats.json"), "w", encoding="ascii") as fh:
        json.dump(index, fh)

    allow_path = os.path.join(root, "allowlist.json")
    with open(allow_path, "w", encoding="ascii") as fh:
        json.dump({"allow": []}, fh)

    return ssot_path, vec, allow_path


def _reindex_sha(vec):
    """Recompute INDEX sha256 entries to match files on disk (use after editing files)."""
    ipath = os.path.join(vec, "INDEX_all_formats.json")
    idx = json.load(open(ipath))
    for p in idx["packs"]:
        p["sha256"] = _sha(os.path.join(vec, p["file"]))
    with open(ipath, "w", encoding="ascii") as fh:
        json.dump(idx, fh)


def _edit_index(vec, fn):
    ipath = os.path.join(vec, "INDEX_all_formats.json")
    idx = json.load(open(ipath))
    fn(idx)
    with open(ipath, "w", encoding="ascii") as fh:
        json.dump(idx, fh)


def _edit_pack(vec, fname, fn):
    fpath = os.path.join(vec, fname)
    pack = json.load(open(fpath))
    fn(pack)
    with open(fpath, "w", encoding="ascii") as fh:
        json.dump(pack, fh)


def main():
    results = []

    def check(name, ok):
        results.append((name, bool(ok)))

    # ---------- T0: clean corpus PASSES ----------
    root = tempfile.mkdtemp(prefix="wp18st_clean_")
    try:
        ssot, vec, allow = build_clean_corpus(root)
        code, rep = G.run_gate(ssot, vec, allow)
        check("T0_clean_passes (exit 0 + verdict CLEAN)", code == 0 and rep["verdict"] == "CLEAN")
        check("T0_no_capability_metric", rep.get("capability_measured") is False)
        # no key anywhere that looks like a benchmark/score metric
        blob = json.dumps(rep).lower()
        forbidden = ["pass@", "compile@", "accuracy", "score", "benchmark", "throughput"]
        check("T0_no_fabricated_metric_strings", not any(t in blob for t in forbidden))
        # determinism: run twice, byte-identical report (minus volatile abspaths already fixed)
        _, rep2 = G.run_gate(ssot, vec, allow)
        check("T0_deterministic", json.dumps(rep, sort_keys=True) == json.dumps(rep2, sort_keys=True))
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TA: Check A fails on extra pack (SSOT/pack mismatch) ----------
    root = tempfile.mkdtemp(prefix="wp18st_A_")
    try:
        ssot, vec, allow = build_clean_corpus(root)
        # add an extra pack id to INDEX not present in SSOT
        def add_extra(idx):
            idx["packs"].append({"id": "ghost_fmt", "file": "fp8_e4m3_conformance_v0.json",
                                 "kind": "bitexact", "sha256": "x"})
            idx["total_packs"] = 4
            idx["bitexact_packs"] = 3
        _edit_index(vec, add_extra)
        code, rep = G.run_gate(ssot, vec, allow)
        check("TA_extra_pack_fails_A", code == 2 and rep["checks"]["A_packset_equals_ssot"]["ok"] is False
              and "ghost_fmt" in rep["checks"]["A_packset_equals_ssot"]["extra_packs"])
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TA2: Check A fails on missing pack ----------
    root = tempfile.mkdtemp(prefix="wp18st_A2_")
    try:
        ssot, vec, allow = build_clean_corpus(root)
        # drop fp8_e5m2 from INDEX (still in SSOT) -> missing
        def drop_one(idx):
            idx["packs"] = [p for p in idx["packs"] if p["id"] != "fp8_e5m2"]
            idx["total_packs"] = 2
            idx["bitexact_packs"] = 1
        _edit_index(vec, drop_one)
        code, rep = G.run_gate(ssot, vec, allow)
        check("TA2_missing_pack_fails_A", rep["checks"]["A_packset_equals_ssot"]["ok"] is False
              and "fp8_e5m2" in rep["checks"]["A_packset_equals_ssot"]["missing_packs"])
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TB2: a `kind` must be verified, not believed ----------
    # T72: relabelling a bit-exact pack `structural` in the INDEX made the
    # D/D2/E loop skip it, so a planted drift went from exit 2 to CLEAN with
    # the pack file byte for byte unchanged. This mutant plants exactly that
    # and requires B2 to be the thing that catches it -- C and B must both
    # stay green, or the assertion is passing for the wrong reason.
    root = tempfile.mkdtemp(prefix="wp18st_B2_")
    try:
        ssot, vec, allow = build_clean_corpus(root)

        def relabel(idx):
            for p in idx["packs"]:
                if p["id"] == "fp8_e4m3":
                    p["kind"] = "structural"
            idx["bitexact_packs"] -= 1
            idx["structural_packs"] += 1
        _edit_index(vec, relabel)
        code, rep = G.run_gate(ssot, vec, allow)
        b2 = rep["checks"].get("B2_structural_carries_no_rows", {})
        check("TB2_relabel_to_structural_fails_B2",
              b2.get("ok") is False
              and any(m["file"].startswith("fp8_e4m3") for m in b2.get("mislabelled", []))
              and rep["checks"]["C_sha_freshness"]["ok"] is True
              and rep["checks"]["B_index_counts"]["ok"] is True)
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TC2: an ABSENT digest is a failure, not an exemption ----------
    # T72: `if want and want != got` read a missing or empty sha256 as "no
    # freshness requirement". Deleting the key turned a caught tamper into
    # CLEAN in this gate and in pack_index_consistency_gate both.
    root = tempfile.mkdtemp(prefix="wp18st_C2_")
    try:
        ssot, vec, allow = build_clean_corpus(root)

        def drop_sha(idx):
            for p in idx["packs"]:
                if p["id"] == "fp8_e4m3":
                    p.pop("sha256", None)
        _edit_index(vec, drop_sha)
        code, rep = G.run_gate(ssot, vec, allow)
        c = rep["checks"]["C_sha_freshness"]
        check("TC2_absent_sha_fails_C",
              c["ok"] is False
              and any(d.get("index_sha") is None for d in c.get("drift", [])))
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TB: Check B fails on wrong INDEX count ----------
    root = tempfile.mkdtemp(prefix="wp18st_B_")
    try:
        ssot, vec, allow = build_clean_corpus(root)
        def bad_count(idx):
            idx["bitexact_packs"] = 99  # lie about the count, keep packs list intact
        _edit_index(vec, bad_count)
        code, rep = G.run_gate(ssot, vec, allow)
        check("TB_wrong_count_fails_B", code == 2 and rep["checks"]["B_index_counts"]["ok"] is False)
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TC: Check C fails on stale SHA ----------
    root = tempfile.mkdtemp(prefix="wp18st_C_")
    try:
        ssot, vec, allow = build_clean_corpus(root)
        # mutate a pack file body WITHOUT updating its INDEX sha256 -> stale manifest
        _edit_pack(vec, "fp8_e4m3_conformance_v0.json",
                   lambda p: p["vectors"][0].update({"input_f64": 1.0000001, "decoded_f64": 1.0000001}))
        code, rep = G.run_gate(ssot, vec, allow)
        check("TC_stale_sha_fails_C", code == 2 and rep["checks"]["C_sha_freshness"]["ok"] is False)
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TD: Check D fails on corrupted decoded_f64 (mislabeled abs_error) ----------
    root = tempfile.mkdtemp(prefix="wp18st_D_")
    try:
        ssot, vec, allow = build_clean_corpus(root)
        # make a finite row whose decoded differs from input but abs_error still 0.0
        _edit_pack(vec, "fp8_e4m3_conformance_v0.json",
                   lambda p: p["vectors"][0].update({"decoded_f64": 1.5, "abs_error": 0.0}))
        _reindex_sha(vec)  # keep SHA fresh so ONLY D fails
        code, rep = G.run_gate(ssot, vec, allow)
        d = rep["checks"]["D_rederive_abs_error"]
        check("TD_mislabeled_abs_error_fails_D", code == 2 and d["ok"] is False and d["mismatch_count"] >= 1)
        check("TD_isolates_only_D", rep["checks"]["A_packset_equals_ssot"]["ok"]
              and rep["checks"]["B_index_counts"]["ok"] and rep["checks"]["C_sha_freshness"]["ok"])
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TD2: Check D2 fails on broken special-value round-trip ----------
    root = tempfile.mkdtemp(prefix="wp18st_D2_")
    try:
        ssot, vec, allow = build_clean_corpus(root)
        # nan-code row whose decoded is NOT nan (round-trip broken)
        _edit_pack(vec, "fp8_e4m3_conformance_v0.json",
                   lambda p: p["vectors"][1].update({"input_f64": "nan", "decoded_f64": 0.0, "category": "nan"}))
        _reindex_sha(vec)
        code, rep = G.run_gate(ssot, vec, allow)
        d2 = rep["checks"]["D2_special_value_roundtrip"]
        check("TD2_broken_special_roundtrip_fails_D2", code == 2 and d2["ok"] is False and d2["broken_count"] >= 1)
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TE: Check E fails on undisclosed finite nonzero abs_error ----------
    root = tempfile.mkdtemp(prefix="wp18st_E_")
    try:
        ssot, vec, allow = build_clean_corpus(root)
        # finite row with a HONEST nonzero abs_error (consistent w/ D) but NOT on allowlist
        def leak(p):
            p["vectors"][0].update({"input_f64": 0.1, "decoded_f64": 0.10009765625,
                                    "abs_error": abs(0.10009765625 - 0.1), "category": "normal"})
        _edit_pack(vec, "fp8_e4m3_conformance_v0.json", leak)
        _reindex_sha(vec)
        code, rep = G.run_gate(ssot, vec, allow)
        e = rep["checks"]["E_honesty_allowlist"]
        check("TE_undisclosed_nonzero_fails_E", code == 2 and e["ok"] is False and len(e["undisclosed_nonzero"]) >= 1)
        check("TE_D_stays_clean (nonzero is honest)", rep["checks"]["D_rederive_abs_error"]["ok"])
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TE2: same row, now allow-listed -> E passes (allowlist actually works) ----------
    root = tempfile.mkdtemp(prefix="wp18st_E2_")
    try:
        ssot, vec, allow = build_clean_corpus(root)
        def leak2(p):
            p["vectors"][0].update({"input_f64": 0.1, "decoded_f64": 0.10009765625,
                                    "abs_error": abs(0.10009765625 - 0.1), "category": "normal"})
        _edit_pack(vec, "fp8_e4m3_conformance_v0.json", leak2)
        _reindex_sha(vec)
        with open(allow, "w", encoding="ascii") as fh:
            json.dump({"allow": [{"pack_id": "fp8_e4m3", "row_name": "pos_1p0",
                                  "reason": "0.1 not on fp8_e4m3 grid; honest rounding"}]}, fh)
        code, rep = G.run_gate(ssot, vec, allow)
        check("TE2_allowlisted_passes_E", code == 0 and rep["checks"]["E_honesty_allowlist"]["ok"]
              and len(rep["checks"]["E_honesty_allowlist"]["disclosed_nonzero"]) >= 1)
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- T_input: missing INDEX -> exit 3 ----------
    root = tempfile.mkdtemp(prefix="wp18st_in_")
    try:
        ssot, vec, allow = build_clean_corpus(root)
        os.remove(os.path.join(vec, "INDEX_all_formats.json"))
        code, rep = G.run_gate(ssot, vec, allow)
        check("T_input_missing_index_exit3", code == 3)
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- report ----------
    passed = sum(1 for _, ok in results if ok)
    total = len(results)
    print("WP-18 gate self-test: %d/%d assertions passed" % (passed, total))
    for name, ok in results:
        print("  [%s] %s" % ("PASS" if ok else "FAIL", name))
    return 0 if passed == total else 1


if __name__ == "__main__":
    import sys
    sys.exit(main())
