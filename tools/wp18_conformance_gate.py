#!/usr/bin/env python3
"""
WP-18 conformance-corpus integrity gate (stdlib only, ZERO local imports).

Locks the t27 numeric conformance corpus against silent drift, the conformance-layer
analog of the catalog-count gate. Checks, as DISTINCT failures (CertifiedData v2
"which surface drifted" discipline):

  A. pack-set name-set == SSOT format id-set (no missing / no extra)
  B. INDEX total/bitexact/selfconsistent/structural counts == recount from the pack
     files. THREE pack kinds are recognised: "bitexact" (an independent oracle, e.g.
     a silicon ROM or a second reference impl, witnesses every vector), "structural"
     (no value vectors, n_vectors=0), and "bitexact_selfconsistent" (dyadic-exact
     vectors that re-derive under ONE decode law and carry honest abs_error, but have
     NO independent second witness). The selfconsistent kind is counted separately so
     a pack is never silently promoted to the stronger "bitexact" label. The INDEX key
     "selfconsistent_packs" is OPTIONAL and defaults to 0 (backward compatible: an
     INDEX with no selfconsistent packs and no key still recounts 0 and stays CLEAN).
  C. per-pack SHA-256 in INDEX == file hash (manifest freshness)
  D. FINITE-VALUE row arithmetic consistency: for every row carrying input_f64 +
     decoded_f64 + abs_error whose values are finite, the stored abs_error must equal
     abs(decoded_f64 - input_f64) within a tiny tolerance (re-derivation is the oracle;
     metamorphic round-trip relation).
  D2. SPECIAL-VALUE round-trip: rows encoding a special value (category in {inf, nan},
     i.e. an inf-code or nan-code) are validated by same-class round-trip instead of an
     arithmetic magnitude: nan -> nan, +inf -> +inf, -inf -> -inf. Their abs_error is a
     placeholder (nan for nan-codes; 0.0/inf for inf-codes) and is NOT an arithmetic
     error, so it is exempt from D and E.
  E. honesty allow-list: any FINITE NONZERO abs_error must appear on the explicit
     allow-list with a machine-readable reason; an undisclosed finite nonzero error is
     a LEAK.

Exit 0 = CLEAN. Exit 2 = DRIFT or LEAK. Exit 3 = bad input.
NO capability/quality/benchmark metric is ever emitted. ASCII-only. Apache-2.0.

Science backbone: Berkeley TestFloat/SoftFloat (independent re-derivation as oracle),
IeeeCC754, IBM FPgen vectors, Matula round-trip theorem (narrow-width decode-to-f64 is
exact; wide mantissas flagged), metamorphic testing (necessary-relation check),
ACM artifact review + CertifiedData v2 separate-surface hashing. See
RESEARCH_BIBLIOGRAPHY_WP18.md.

Usage:
  wp18_conformance_gate.py --ssot <formats_catalog.t27> --vectors <conformance/vectors dir>
                           [--allowlist <abs_error_allowlist.json>] [--json <out.json>]
"""
import argparse
import glob
import hashlib
import json
import math
import os
import re
import sys

SCHEMA = "wp18-conformance-gate/v2"
# abs_error re-derivation tolerance: stored vs recomputed must agree this closely.
# Both are f64 subtractions of f64 inputs, so they should match to rounding noise.
REDERIVE_TOL = 1e-12
# Row categories that denote a SPECIAL-VALUE code (inf / nan encoding), where
# abs(decoded - input) is not a meaningful arithmetic error and the correctness
# criterion is a same-class round-trip (inf->inf, nan->nan) instead.
SPECIAL_CATEGORIES = frozenset({"inf", "nan"})


def _read_ssot_ids(ssot_path):
    ids = []
    with open(ssot_path, "r", encoding="ascii") as fh:
        for line in fh:
            m = re.search(r"//\s*CATALOG:\s*id=(\S+)", line)
            if m:
                ids.append(m.group(1))
    return ids


def _count_value_rows(o):
    """Rows the D/D2/E checks would read: the three fields they re-derive from."""
    n = 0
    if isinstance(o, dict):
        if "abs_error" in o and "input_f64" in o and "decoded_f64" in o:
            n += 1
        for v in o.values():
            n += _count_value_rows(v)
    elif isinstance(o, list):
        for x in o:
            n += _count_value_rows(x)
    return n


def _sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def _to_float(v):
    """Parse a stored numeric that may be a JSON number or 'Infinity'/'NaN' string."""
    if isinstance(v, bool):
        return None
    if isinstance(v, (int, float)):
        return float(v)
    if isinstance(v, str):
        s = v.strip().lower()
        if s in ("inf", "infinity", "+inf", "+infinity"):
            return math.inf
        if s in ("-inf", "-infinity"):
            return -math.inf
        if s in ("nan",):
            return math.nan
    return None


def _rows_of(pack):
    return pack.get("vectors") or pack.get("rows") or []


def _is_special_row(row, inp, dec):
    """A special-value row encodes a non-finite value (inf/nan code).

    Identified by an explicit category tag in SPECIAL_CATEGORIES, or, defensively,
    by a non-finite input or decoded value. Such a row's abs_error is a placeholder,
    not an arithmetic magnitude.
    """
    cat = (row.get("category") or "").strip().lower()
    if cat in SPECIAL_CATEGORIES:
        return True
    if inp is not None and (math.isinf(inp) or math.isnan(inp)):
        return True
    if dec is not None and (math.isinf(dec) or math.isnan(dec)):
        return True
    return False


def run_gate(ssot_path, vectors_dir, allowlist_path=None):
    report = {
        "schema": SCHEMA,
        "ssot": os.path.abspath(ssot_path),
        "vectors_dir": os.path.abspath(vectors_dir),
        "checks": {},
        "failures": [],
        "capability_measured": False,  # this gate measures integrity, never capability
    }

    # ---- load INDEX ----
    index_path = os.path.join(vectors_dir, "INDEX_all_formats.json")
    if not os.path.isfile(index_path):
        report["failures"].append({"check": "input", "detail": "INDEX_all_formats.json not found"})
        return 3, report
    try:
        index = json.load(open(index_path, "r", encoding="ascii"))
    except Exception as exc:  # noqa: BLE001
        report["failures"].append({"check": "input", "detail": "INDEX parse error: %s" % exc})
        return 3, report

    # ---- load allow-list ----
    allow = {}
    if allowlist_path and os.path.isfile(allowlist_path):
        try:
            al = json.load(open(allowlist_path, "r", encoding="ascii"))
            for entry in al.get("allow", []):
                allow[(entry["pack_id"], entry["row_name"])] = entry.get("reason", "")
        except Exception as exc:  # noqa: BLE001
            report["failures"].append({"check": "input", "detail": "allowlist parse error: %s" % exc})
            return 3, report
    report["allowlist_entries"] = sorted("%s::%s" % k for k in allow)

    packs = index.get("packs", [])
    pack_ids = set(p["id"] for p in packs)

    # ---- Check A: pack-set == SSOT id-set ----
    ssot_ids = set(_read_ssot_ids(ssot_path))
    if not ssot_ids:
        report["failures"].append({"check": "input", "detail": "no SSOT CATALOG ids parsed"})
        return 3, report
    missing = sorted(ssot_ids - pack_ids)
    extra = sorted(pack_ids - ssot_ids)
    report["checks"]["A_packset_equals_ssot"] = {
        "ssot_count": len(ssot_ids),
        "pack_count": len(pack_ids),
        "missing_packs": missing,
        "extra_packs": extra,
        "ok": not missing and not extra,
    }
    if missing or extra:
        report["failures"].append({"check": "A", "missing": missing, "extra": extra})

    # ---- Check B: INDEX counts == recount (tri-state: bitexact / selfconsistent / structural) ----
    be = sc = st = 0
    unknown_kind = []
    file_missing = []
    for p in packs:
        fn = os.path.join(vectors_dir, p["file"])
        if not os.path.isfile(fn):
            file_missing.append(p["file"])
            continue
        kind = p.get("kind")
        if kind == "structural":
            st += 1
        elif kind == "bitexact_selfconsistent":
            sc += 1
        elif kind == "bitexact":
            be += 1
        else:
            # An unrecognised kind is drift, NOT silently folded into bitexact.
            unknown_kind.append({"file": p["file"], "kind": str(kind)})
    # "selfconsistent_packs" is OPTIONAL; default 0 keeps a legacy 2-kind INDEX green.
    claimed = (
        index.get("total_packs"),
        index.get("bitexact_packs"),
        index.get("selfconsistent_packs", 0),
        index.get("structural_packs"),
    )
    recount = (len(packs), be, sc, st)
    report["checks"]["B_index_counts"] = {
        "claimed_total_bitexact_selfconsistent_structural": list(claimed),
        "recount_total_bitexact_selfconsistent_structural": list(recount),
        "unknown_kind": unknown_kind,
        "file_missing": file_missing,
        "ok": claimed == recount and not file_missing and not unknown_kind,
    }
    if claimed != recount or file_missing or unknown_kind:
        report["failures"].append({"check": "B", "claimed": claimed, "recount": recount,
                                   "file_missing": file_missing, "unknown_kind": unknown_kind})

    # ---- Check C: SHA freshness ----
    sha_drift = []
    for p in packs:
        fn = os.path.join(vectors_dir, p["file"])
        if not os.path.isfile(fn):
            continue
        want = p.get("sha256")
        got = _sha256_file(fn)
        # T72: `if want and ...` read an ABSENT or EMPTY digest as "no freshness
        # requirement". Measured: tamper a pack in a way only this check sees,
        # and with the digest present both this gate and pack_index fail; delete
        # the key and BOTH return CLEAN, exit 0. An anomaly must not be read as
        # an absence. All 109 live entries carry a digest, so this cannot fire
        # today -- it fires the moment one stops carrying it.
        if want != got:
            sha_drift.append({"file": p["file"], "index_sha": want, "file_sha": got})
    report["checks"]["C_sha_freshness"] = {"drift_count": len(sha_drift), "drift": sha_drift[:10], "ok": not sha_drift}
    if sha_drift:
        report["failures"].append({"check": "C", "sha_drift": sha_drift[:10]})

    # ---- Check B2: a `kind` is a claim, not evidence ----
    # T72: the D/D2/E loop below skips `kind == "structural"` on the grounds
    # that such packs carry no round-trip rows -- an assumption never checked
    # against the pack's actual contents. Measured: relabel one pack
    # bitexact -> structural in the INDEX and adjust the two header counts, and
    # a planted drift goes from exit 2 to CLEAN with the pack file byte for
    # byte unchanged; rows_checked falls 3795 -> 3787 in silence. 89 packs are
    # demotable that way. So the label is verified against the rows before it
    # is allowed to excuse them. All 20 structural packs measured at 0 value
    # rows today, so this cannot fire on the clean tree.
    mislabelled = []
    for p in packs:
        if p.get("kind") != "structural":
            continue
        fn = os.path.join(vectors_dir, p["file"])
        if not os.path.isfile(fn):
            continue
        try:
            doc = json.load(open(fn, encoding="utf-8"))
        except Exception:
            continue
        n = _count_value_rows(doc)
        if n:
            mislabelled.append({"file": p["file"], "value_rows": n})
    report["checks"]["B2_structural_carries_no_rows"] = {
        "structural_packs": sum(1 for p in packs if p.get("kind") == "structural"),
        "mislabelled_count": len(mislabelled),
        "mislabelled": mislabelled[:10],
        "ok": not mislabelled,
    }
    if mislabelled:
        report["failures"].append({"check": "B2", "mislabelled": mislabelled[:10]})

    # ---- Checks D + D2 + E: row re-derivation + special-value + honesty allow-list ----
    rederive_mismatch = []
    undisclosed_nonzero = []
    rows_checked = 0
    finite_rows = 0
    special_rows = 0
    special_mismatch = []
    nonzero_disclosed = []
    for p in packs:
        if p.get("kind") == "structural":
            continue  # structural packs carry no bit-exact round-trip rows
        fn = os.path.join(vectors_dir, p["file"])
        if not os.path.isfile(fn):
            continue
        try:
            pack = json.load(open(fn, "r", encoding="ascii"))
        except Exception as exc:  # noqa: BLE001
            report["failures"].append({"check": "D", "file": p["file"], "detail": "parse error: %s" % exc})
            continue
        for r in _rows_of(pack):
            if not isinstance(r, dict):
                continue
            if "abs_error" not in r or "input_f64" not in r or "decoded_f64" not in r:
                continue
            ae_stored = _to_float(r.get("abs_error"))
            inp = _to_float(r.get("input_f64"))
            dec = _to_float(r.get("decoded_f64"))
            if ae_stored is None or inp is None or dec is None:
                continue
            rows_checked += 1

            # ---- D2: special-value rows (inf-code / nan-code) ----
            if _is_special_row(r, inp, dec):
                special_rows += 1
                # Correctness = same-class round-trip.
                if math.isnan(inp):
                    roundtrip_ok = (math.isnan(dec))
                elif math.isinf(inp):
                    roundtrip_ok = (math.isinf(dec) and (inp > 0) == (dec > 0))
                elif math.isinf(dec):
                    # finite input overflowed to inf (declared overflow_to_inf etc.):
                    # fall through to the finite-error honesty check below.
                    roundtrip_ok = None
                else:
                    # tagged special but both finite -> treat as ordinary finite row
                    roundtrip_ok = None
                if roundtrip_ok is False:
                    special_mismatch.append({"pack": p["id"], "row": r.get("name"),
                                             "category": r.get("category"),
                                             "input": str(inp), "decoded": str(dec),
                                             "detail": "special-value round-trip broken"})
                if roundtrip_ok is not None:
                    continue  # fully handled as a special-value row

            # ---- finite-value row (the meaningful arithmetic case) ----
            finite_rows += 1
            # D: re-derive abs_error from stored decoded vs input
            rederived = abs(dec - inp)
            if abs(rederived - ae_stored) > REDERIVE_TOL:
                rederive_mismatch.append({"pack": p["id"], "row": r.get("name"),
                                          "stored": ae_stored, "rederived": rederived})
            # E: honesty allow-list for any finite nonzero abs_error
            is_nonzero = (math.isinf(ae_stored) or ae_stored != 0.0)
            if is_nonzero:
                key = (p["id"], r.get("name"))
                if key in allow:
                    nonzero_disclosed.append({"pack": p["id"], "row": r.get("name"),
                                              "abs_error": str(ae_stored), "reason": allow[key]})
                else:
                    undisclosed_nonzero.append({"pack": p["id"], "row": r.get("name"),
                                                "abs_error": str(ae_stored)})
    report["checks"]["D_rederive_abs_error"] = {
        "rows_checked": rows_checked,
        "finite_rows": finite_rows,
        "special_rows": special_rows,
        "mismatch_count": len(rederive_mismatch),
        "mismatch": rederive_mismatch[:20],
        "ok": not rederive_mismatch,
    }
    report["checks"]["D2_special_value_roundtrip"] = {
        "special_rows": special_rows,
        "broken_count": len(special_mismatch),
        "broken": special_mismatch[:10],
        "ok": not special_mismatch,
    }
    report["checks"]["E_honesty_allowlist"] = {
        "disclosed_nonzero": nonzero_disclosed,
        "undisclosed_nonzero": undisclosed_nonzero,
        "ok": not undisclosed_nonzero,
    }
    if rederive_mismatch:
        report["failures"].append({"check": "D", "rederive_mismatch": rederive_mismatch[:20]})
    if special_mismatch:
        report["failures"].append({"check": "D2", "special_mismatch": special_mismatch[:10]})
    if undisclosed_nonzero:
        report["failures"].append({"check": "E", "undisclosed_nonzero": undisclosed_nonzero})

    exit_code = 0 if not report["failures"] else 2
    report["verdict"] = "CLEAN" if exit_code == 0 else "DRIFT_OR_LEAK"
    return exit_code, report


def main(argv=None):
    ap = argparse.ArgumentParser(description="WP-18 conformance-corpus integrity gate")
    ap.add_argument("--ssot", required=True, help="path to specs/numeric/formats_catalog.t27")
    ap.add_argument("--vectors", required=True, help="path to conformance/vectors directory")
    ap.add_argument("--allowlist", default=None, help="path to abs_error_allowlist.json")
    ap.add_argument("--json", default=None, help="optional path to write the JSON report")
    args = ap.parse_args(argv)

    if not os.path.isfile(args.ssot):
        sys.stderr.write("bad-input: SSOT not found: %s\n" % args.ssot)
        return 3
    if not os.path.isdir(args.vectors):
        sys.stderr.write("bad-input: vectors dir not found: %s\n" % args.vectors)
        return 3

    code, report = run_gate(args.ssot, args.vectors, args.allowlist)
    text = json.dumps(report, indent=2, sort_keys=True)
    if args.json:
        with open(args.json, "w", encoding="ascii") as fh:
            fh.write(text + "\n")
    print(text)
    sys.stderr.write("WP-18 verdict: %s (exit %d)\n" % (report.get("verdict", "BAD_INPUT"), code))
    return code


if __name__ == "__main__":
    sys.exit(main())
