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

T91: every case below called `G.run_gate()` in-process, which proves the checking
FUNCTION and nothing about the wiring from it to the process exit code, and never
broke the WORLD's existence -- only the contents of a well-formed one. Measured
with `tri gates mutate --only wp18_conformance_gate.py`: five of the gate's six
`return 3` sites survived (224, 235, 245, 597, 600) while all 23 assertions stayed
green. The TP_* cases at the end run the gate as a whole PROCESS against a broken
world, one broken thing at a time.

NOT covered, so nobody infers it from a green: the per-pack `parse error` at
wp18_conformance_gate.py:373 records a D failure and CONTINUES rather than
returning, so it is not a `return` site, `tri gates mutate` never perturbs it, and
no case here plants an unparseable pack file.
"""
import copy
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile

# T96: drop any cached bytecode for the gate BEFORE importing it.
#
# Python keys a .pyc on (source mtime in whole seconds, source size). Editing
# the gate's `return 1` to `return 0` preserves the size, and an edit-run-edit
# loop finishes well inside one second -- so this control can be handed the
# PREVIOUS state's bytecode and go red on a tree that matches git exactly.
#
# Measured here, on this file, while testing this gate: five hand mutations in
# quick succession left a .pyc that made the control report five failures with
# `git status` clean and the source sha matching HEAD. `tri gates mutate` clears
# this between mutants; a person at a terminal does not, and neither did this
# control. Three lines remove the whole class.
def _drop_stale_bytecode():
    import pathlib
    here = pathlib.Path(__file__).resolve().parent / "__pycache__"
    for p in here.glob("wp18_conformance_gate.*.pyc"):
        try:
            p.unlink()
        except OSError:
            pass


_drop_stale_bytecode()

import wp18_conformance_gate as G  # noqa: E402

# The gate file as a SCRIPT. `G.__file__` is the module this control already
# imported, so a mutant of the gate is the thing that gets run -- no --root
# flag and no T27_*_ROOT to aim a live gate somewhere harmless, and no second
# copy that could drift from the one under test.
GATE_SCRIPT = os.path.abspath(G.__file__)


SSOT_TEXT = "\n".join([
    "// CATALOG: id=fp8_e4m3",
    "// CATALOG: id=fp8_e5m2",
    "// CATALOG: id=gf16",
    "// CATALOG: id=gf256",
]) + "\n"


def _sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        h.update(fh.read())
    return h.hexdigest()


def _bitexact_pack(pid, rows):
    return {"schema": "t27-conformance/v0", "format": pid, "n_vectors": len(rows), "vectors": rows}


def _wide_pack(pid, rows):
    """T78: the fixture was a SCHEMA MONOCULTURE -- one row shape, so the branch
    that drops every other shape was taken zero times by any planted mutant,
    across both selftests. Live, that branch swallowed 2110 of 5905 rows."""
    return {"schema": "t27-conformance/v0", "format": pid, "kind": "bitexact",
            "n_vectors": len(rows), "vectors": rows}


def _structural_pack(pid):
    return {"schema": "t27-conformance/v0", "format": pid, "kind": "structural", "n_vectors": 0, "vectors": []}


def build_clean_corpus(root):
    """A small, fully-consistent corpus: 2 f64 packs, 1 WIDE pack, 1 structural.

    The wide pack is not decoration. Until T78 every fixture row was the f64
    shape, so the branch that drops other shapes was taken ZERO times by any
    planted mutant in either selftest -- while live it swallowed 2110 of 5905
    rows across seven packs the INDEX calls bitexact.
    """
    vec = os.path.join(root, "vectors")
    os.makedirs(vec, exist_ok=True)

    ssot_path = os.path.join(root, "formats_catalog.t27")
    with open(ssot_path, "w", encoding="ascii") as fh:
        fh.write(SSOT_TEXT)

    # fp8_e4m3: one exact finite row + one nan-code row + one inf-code row
    e4m3 = _bitexact_pack("fp8_e4m3", [
        {"name": "pos_1p0", "input_f64": 1.0, "input_f64_hex": "0x3FF0000000000000",
         "decoded_f64": 1.0, "decoded_f64_hex": "0x3FF0000000000000",
         "abs_error": 0.0, "category": "normal"},
        {"name": "nan_code", "input_f64": "nan", "decoded_f64": "nan", "abs_error": "nan", "category": "nan"},
    ])
    # fp8_e5m2: one exact row + one inf-code row (inf -> inf, abs_error placeholder 0.0)
    e5m2 = _bitexact_pack("fp8_e5m2", [
        {"name": "pos_2p0", "input_f64": 2.0, "decoded_f64": 2.0, "abs_error": 0.0, "category": "normal"},
        {"name": "pos_inf", "input_f64": "inf", "decoded_f64": "inf", "abs_error": 0.0, "category": "inf"},
        # T80: a FINITE input that overflowed to inf. Neither special nor
        # finite -- it used to fall through both and be counted twice, which is
        # why the fixture needs one for the partition assertion to be testable.
        {"name": "overflow_to_inf", "input_f64": 1e+40,
         "input_f64_hex": "0x483D6329F1C35CA5",
         "decoded_f64": "inf", "decoded_f64_hex": "0x7FF0000000000000",
         "abs_error": "inf", "category": "normal"},
    ])
    # gf16 declared structural here (no bit-exact rows) to exercise the structural path
    gf16 = _structural_pack("gf16")
    # gf256: the WIDE shape -- an integer code and its exact value, no f64
    # round-trip, because the format is wider than f64.
    gf256 = _wide_pack("gf256", [
        {"name": "pos_zero", "bits": 0, "value": "0", "abs_error": "0"},
        {"name": "pos_one", "bits": 1, "value": "1", "abs_error": "0"},
    ])

    files = {
        "fp8_e4m3_conformance_v0.json": e4m3,
        "fp8_e5m2_conformance_v0.json": e5m2,
        "gf16_conformance_v0.json": gf16,
        "gf256_conformance_v0.json": gf256,
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
        {"id": "gf256", "file": "gf256_conformance_v0.json", "kind": "bitexact",
         "sha256": _sha(os.path.join(vec, "gf256_conformance_v0.json"))},
    ]
    index = {"total_packs": 4, "bitexact_packs": 3, "structural_packs": 1, "packs": packs}
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


def _run_gate_process(ssot, vec, allow=None):
    """Run the gate the way CI does: a whole process, exit code and streams.

    Returns (returncode, stdout, stderr). The two precondition branches in
    `main()` live entirely outside `run_gate()`, so an in-process call cannot
    reach them at all; and every exit-3 branch INSIDE `run_gate()` reaches the
    process exit code through a `code, report = run_gate(...)` unpack that an
    in-process caller performs itself. Forcing any of those five `return 3`s
    to 0 left all 23 in-process assertions green.
    """
    argv = [sys.executable or "python3", GATE_SCRIPT, "--ssot", ssot, "--vectors", vec]
    if allow is not None:
        argv += ["--allowlist", allow]
    p = subprocess.run(argv, capture_output=True, text=True)
    return p.returncode, p.stdout, p.stderr


def _input_failure_details(stdout):
    """The `detail` strings the gate printed for its bad-input failures.

    Empty when the gate printed no report at all, which is itself the signal
    that `main()` refused before `run_gate()` ever ran.
    """
    if not stdout.strip():
        return []
    try:
        rep = json.loads(stdout)
    except ValueError:
        return []
    return [str(f.get("detail", "")) for f in rep.get("failures", [])]


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

    # ---------- TH: the row partition must actually hold ----------
    # T80: without a mutant here the assertion is decoration. Reverting the
    # partition fix left the whole selftest green until this existed -- a gap
    # in MY controls, found by mutating my own patch and noticing nothing died.
    # The fixture now carries an overflow row, so the three buckets are
    # non-trivial and a double count is observable.
    root = tempfile.mkdtemp(prefix="wp18st_H_")
    try:
        ssot, vec, allow = build_clean_corpus(root)
        code, rep = G.run_gate(ssot, vec, allow)
        h = rep["checks"].get("H_row_partition", {})
        check("TH_partition_holds_on_clean_fixture",
              h.get("ok") is True
              and h.get("overflow_rows") == 1
              and h.get("sum") == h.get("rows_checked"))
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TG: a decimal must match its own hex twin ----------
    # T80: Check D cannot constrain an overflow row -- abs(inf - x) is inf for
    # every finite x -- so swapping input_f64 on such a row was invisible. The
    # row carries its own oracle; this plants a decimal that contradicts it.
    root = tempfile.mkdtemp(prefix="wp18st_G_")
    try:
        ssot, vec, allow = build_clean_corpus(root)

        def lie(pack):
            pack["vectors"][0]["input_f64"] = 2.0   # hex still says 1.0
        _edit_pack(vec, "fp8_e4m3_conformance_v0.json", lie)
        _reindex_sha(vec)
        code, rep = G.run_gate(ssot, vec, allow)
        g = rep["checks"].get("G_decimal_matches_hex", {})
        check("TG_decimal_contradicts_hex_fails_G",
              g.get("ok") is False
              and g.get("disagree_count") == 1
              and rep["checks"]["C_sha_freshness"]["ok"] is True)
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TD_tol: the tolerance's own BOUNDARY ----------
    # T107: `_diff > REDERIVE_TOL` rewritten to `>=` passed this entire control.
    # Every case here sits far from the tolerance -- a clean row has _diff 0.0
    # and TD_nan's row has none at all -- so nothing ever asked what happens AT
    # it. Found by the boundary operator (`tri gates mutate --all`), which is
    # the whole reason a fourth operator was written.
    #
    # The arithmetic is exact on purpose: row 0 decodes to its own input, so
    # `rederived = abs(dec - inp)` is 0.0, and an abs_error of exactly
    # REDERIVE_TOL makes `_diff = abs(0.0 - 1e-12)` the tolerance itself with no
    # rounding anywhere. Equal to the tolerance is WITHIN it: the check asks
    # whether the re-derivation disagrees by MORE than tol.
    #
    # Asserted on D's own dict rather than the exit code. A nonzero abs_error
    # also trips E, the honesty allow-list, and an exit-code assertion could
    # not tell the two branches apart -- which is the confusion this whole file
    # exists to prevent.
    root = tempfile.mkdtemp(prefix="wp18st_Dtol_")
    try:
        ssot, vec, allow = build_clean_corpus(root)

        def at_tolerance(pack):
            pack["vectors"][0]["abs_error"] = G.REDERIVE_TOL
        _edit_pack(vec, "fp8_e4m3_conformance_v0.json", at_tolerance)
        _reindex_sha(vec)
        code, rep = G.run_gate(ssot, vec, allow)
        d = rep["checks"]["D_rederive_abs_error"]
        check("TDtol_exactly_the_tolerance_is_within_it",
              d["ok"] is True and not d.get("mismatch"))
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TD_nan: a comparison that is not a number is a failure ----------
    root = tempfile.mkdtemp(prefix="wp18st_Dnan_")
    try:
        ssot, vec, allow = build_clean_corpus(root)

        def nanify(pack):
            pack["vectors"][0]["abs_error"] = "nan"
        _edit_pack(vec, "fp8_e4m3_conformance_v0.json", nanify)
        _reindex_sha(vec)
        code, rep = G.run_gate(ssot, vec, allow)
        d = rep["checks"]["D_rederive_abs_error"]
        check("TDnan_nan_comparison_fails_D",
              d["ok"] is False
              and any("not a number" in str(m.get("detail", "")) for m in d.get("mismatch", []))
              and rep["checks"].get("G_decimal_matches_hex", {}).get("ok") is True)
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TF: a wide row's stored error must be exactly zero ----------
    # T78: before this, NO fixture row had the wide shape, so the branch that
    # drops non-f64 rows was taken zero times by every planted mutant while it
    # swallowed 2110 live rows. This plants the fault that was invisible:
    # a wide row whose abs_error is not zero.
    root = tempfile.mkdtemp(prefix="wp18st_F_")
    try:
        ssot, vec, allow = build_clean_corpus(root)

        def spoil(pack):
            pack["vectors"][0]["abs_error"] = "1e300"
        _edit_pack(vec, "gf256_conformance_v0.json", spoil)
        _reindex_sha(vec)
        code, rep = G.run_gate(ssot, vec, allow)
        f = rep["checks"].get("F_row_schema_recognised", {})
        check("TF_wide_row_nonzero_error_fails_F",
              f.get("ok") is False
              and f.get("wide_nonzero_count") == 1
              and rep["checks"]["D_rederive_abs_error"]["ok"] is True
              and rep["checks"]["C_sha_freshness"]["ok"] is True)
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TF2: a row shape nobody planned for is reported ----------
    root = tempfile.mkdtemp(prefix="wp18st_F2_")
    try:
        ssot, vec, allow = build_clean_corpus(root)

        def add_alien(pack):
            pack["vectors"].append({"surprise": 1, "shape": "unplanned"})
        _edit_pack(vec, "gf256_conformance_v0.json", add_alien)
        _reindex_sha(vec)
        code, rep = G.run_gate(ssot, vec, allow)
        f = rep["checks"].get("F_row_schema_recognised", {})
        check("TF2_unrecognised_row_shape_fails_F",
              f.get("ok") is False and f.get("unrecognised_count") == 1)
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TB2b: the demotion guard must see the WIDE shape too ----------
    # The guard added in #2448 counted only f64 rows, so demoting a wide pack
    # to `structural` passed with 2021 rows in the file on the live corpus.
    root = tempfile.mkdtemp(prefix="wp18st_B2b_")
    try:
        ssot, vec, allow = build_clean_corpus(root)

        def demote(idx):
            for p in idx["packs"]:
                if p["id"] == "gf256":
                    p["kind"] = "structural"
            idx["bitexact_packs"] -= 1
            idx["structural_packs"] += 1
        _edit_index(vec, demote)
        code, rep = G.run_gate(ssot, vec, allow)
        b2 = rep["checks"].get("B2_structural_carries_no_rows", {})
        check("TB2b_wide_pack_demotion_fails_B2",
              b2.get("ok") is False
              and any(m["file"].startswith("gf256") for m in b2.get("mislabelled", [])))
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
            # T80: the hex twins move with the decimals. Once G exists, a
            # fixture that rewrites one and not the other is internally
            # inconsistent and trips G instead of the check it is testing --
            # which is G doing its job, not a reason to weaken it.
            p["vectors"][0].update({"input_f64": 0.1,
                                    "input_f64_hex": "0x3FB999999999999A",
                                    "decoded_f64": 0.10009765625,
                                    "decoded_f64_hex": "0x3FB9A00000000000",
                                    "abs_error": abs(0.10009765625 - 0.1),
                                    "category": "normal"})
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
            # T80: the hex twins move with the decimals. Once G exists, a
            # fixture that rewrites one and not the other is internally
            # inconsistent and trips G instead of the check it is testing --
            # which is G doing its job, not a reason to weaken it.
            p["vectors"][0].update({"input_f64": 0.1,
                                    "input_f64_hex": "0x3FB999999999999A",
                                    "decoded_f64": 0.10009765625,
                                    "decoded_f64_hex": "0x3FB9A00000000000",
                                    "abs_error": abs(0.10009765625 - 0.1),
                                    "category": "normal"})
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

    # ---------- TP0: the WHOLE PROGRAM on a clean world -> exit 0 ----------
    # Every TP_* case below asserts exit 3. Without one run that reaches exit 0
    # through the same path, "exit 3" would be satisfied by a gate that can only
    # ever say 3, and the five cases would be measuring nothing.
    root = tempfile.mkdtemp(prefix="wp18st_P0_")
    try:
        ssot, vec, allow = build_clean_corpus(root)
        rc, out, err = _run_gate_process(ssot, vec, allow)
        # A bare json.loads here aborts the WHOLE control with a traceback when
        # the gate emits nothing or emits garbage -- which is exactly the
        # failure this case exists to catch. A control that dies instead of
        # reporting FAIL leaves every case after it unrun, and the exit code
        # looks the same as a caught failure only by luck.
        try:
            verdict = json.loads(out).get("verdict")
        except (ValueError, AttributeError):
            verdict = None
        check("TP0_whole_program_clean_exit0",
              rc == 0
              and verdict == "CLEAN"
              and "WP-18 verdict: CLEAN (exit 0)" in err
              and "bad-input" not in err)
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TP_ssot_missing: main() refuses a --ssot that is not a file ----------
    # Gate line 597. Nothing here had ever run the gate as a process, so this
    # branch and its sibling were unreachable by this file: forcing `return 3`
    # to `return 0` printed the same bad-input line on stderr and exited 0.
    root = tempfile.mkdtemp(prefix="wp18st_Pssot_")
    try:
        ssot, vec, allow = build_clean_corpus(root)
        absent = os.path.join(root, "no_such_formats_catalog.t27")
        rc, out, err = _run_gate_process(absent, vec, allow)
        check("TP_ssot_not_a_file_exit3",
              rc == 3
              and "bad-input: SSOT not found" in err
              # the sibling precondition (line 600) opens with the same word
              and "vectors dir not found" not in err
              # every exit-3 branch INSIDE run_gate prints the report and the
              # verdict line; this one refuses before either exists
              and "WP-18 verdict" not in err
              and out.strip() == "")
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TP_vectors_missing: main() refuses a --vectors that is not a dir ----------
    # Gate line 600, the sibling of the case above.
    root = tempfile.mkdtemp(prefix="wp18st_Pvec_")
    try:
        ssot, vec, allow = build_clean_corpus(root)
        absent = os.path.join(root, "no_such_vectors_dir")
        rc, out, err = _run_gate_process(ssot, absent, allow)
        check("TP_vectors_not_a_dir_exit3",
              rc == 3
              and "bad-input: vectors dir not found" in err
              and "SSOT not found" not in err
              and "WP-18 verdict" not in err
              and out.strip() == "")
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TP_index_unparseable: an INDEX that is not JSON -> exit 3 ----------
    # Gate line 224. T_input_missing_index_exit3 removes the INDEX (line 219);
    # a file that EXISTS and does not parse is the other half of that pair, and
    # it was covered by nothing.
    root = tempfile.mkdtemp(prefix="wp18st_Pidx_")
    try:
        ssot, vec, allow = build_clean_corpus(root)
        with open(os.path.join(vec, "INDEX_all_formats.json"), "w", encoding="ascii") as fh:
            fh.write("{ this INDEX is not JSON\n")
        rc, out, err = _run_gate_process(ssot, vec, allow)
        details = _input_failure_details(out)
        check("TP_index_unparseable_exit3",
              rc == 3
              and any(d.startswith("INDEX parse error:") for d in details)
              # the three sibling bad-input branches, named so a shared prefix
              # cannot let one pass for another
              and not any("INDEX_all_formats.json not found" in d for d in details)
              and not any("allowlist parse error" in d for d in details)
              and not any("no SSOT CATALOG ids" in d for d in details)
              and "WP-18 verdict: BAD_INPUT (exit 3)" in err)
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TP_allowlist_unparseable: an allow-list that is not JSON -> exit 3 ----------
    # Gate line 235. A gate that swallowed this would read "cannot parse the
    # disclosures" as "there are no disclosures to make", which is check E's
    # whole subject matter.
    root = tempfile.mkdtemp(prefix="wp18st_Pal_")
    try:
        ssot, vec, allow = build_clean_corpus(root)
        with open(allow, "w", encoding="ascii") as fh:
            fh.write("[ this allow-list is not JSON\n")
        rc, out, err = _run_gate_process(ssot, vec, allow)
        details = _input_failure_details(out)
        check("TP_allowlist_unparseable_exit3",
              rc == 3
              and any(d.startswith("allowlist parse error:") for d in details)
              and not any("INDEX" in d for d in details)
              and not any("no SSOT CATALOG ids" in d for d in details)
              and "WP-18 verdict: BAD_INPUT (exit 3)" in err)
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # ---------- TP_ssot_declares_nothing: an SSOT with no CATALOG ids -> exit 3 ----------
    # Gate line 245. An empty id-set would make check A's "missing / extra"
    # arithmetic vacuously satisfiable, so an unparsed SSOT must be bad input
    # rather than an SSOT that happens to declare nothing.
    root = tempfile.mkdtemp(prefix="wp18st_Pids_")
    try:
        ssot, vec, allow = build_clean_corpus(root)
        with open(ssot, "w", encoding="ascii") as fh:
            fh.write("// this file declares no catalog id at all\n")
        rc, out, err = _run_gate_process(ssot, vec, allow)
        details = _input_failure_details(out)
        check("TP_ssot_declares_no_ids_exit3",
              rc == 3
              and any(d.startswith("no SSOT CATALOG ids parsed") for d in details)
              and not any("INDEX" in d for d in details)
              and not any("allowlist parse error" in d for d in details)
              and "WP-18 verdict: BAD_INPUT (exit 3)" in err)
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
