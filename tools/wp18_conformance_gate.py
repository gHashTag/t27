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
import struct
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


def _f64_from_hex(h):
    """The IEEE-754 double a `*_hex` field names, or None if it is not one."""
    if h is None:
        return None
    t = str(h).strip().lower()
    if t.startswith("0x"):
        t = t[2:]
    if len(t) > 16:
        return None
    try:
        return struct.unpack(">d", bytes.fromhex(t.rjust(16, "0")))[0]
    except Exception:
        return None


def _same_f64(a, b):
    """Exact identity, with NaN == NaN and -0.0 != 0.0.

    Not `==`: NaN != NaN would report every nan row as a disagreement, and
    0.0 == -0.0 would hide a sign flip in a field whose whole job is to be
    exact.
    """
    if a is None or b is None:
        return None
    if math.isnan(a) and math.isnan(b):
        return True
    if math.isnan(a) or math.isnan(b):
        return False
    if a == 0.0 and b == 0.0:
        return math.copysign(1.0, a) == math.copysign(1.0, b)
    return a == b


def _is_wide_row(r):
    """The wide-GF shape: an integer code and its exact rational/decimal value.

    These packs carry no f64 round-trip because the format is wider than f64;
    the honest check for them is that the stored error is exactly zero.
    """
    return "bits" in r and "value" in r


def _is_exact_zero(v):
    """True for the several spellings of exact zero these packs use.

    NOT `_to_float`: that helper answers only inf/nan and returns None for the
    plain "0" these rows store, so reusing it would classify every wide row as
    a violation on the clean tree.
    """
    if v is None:
        return True
    if isinstance(v, (int, float)):
        return v == 0
    t = str(v).strip()
    if not t:
        return True
    try:
        return float(t) == 0.0
    except ValueError:
        return False


def _count_value_rows(o):
    """Rows a check would read -- BOTH shapes.

    T78: this counted only the f64 shape, so the demotion guard added with it
    was blind to exactly the 2110 wide rows that the D/D2/E loop was already
    dropping. Relabelling gf256 `bitexact` -> `structural` passed with 2021
    rows in the file. A guard against a mislabel must know every shape the
    corpus actually stores, not the one its author had in mind.
    """
    n = 0
    if isinstance(o, dict):
        if ("abs_error" in o and "input_f64" in o and "decoded_f64" in o) or _is_wide_row(o):
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
    hex_disagree = []
    hex_unparseable = []
    overflow_rows = 0
    wide_rows = 0
    wide_nonzero = []
    unrecognised = []
    unparseable = []
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
            # T78: a row SHAPE is not an excuse. These two `continue`s dropped
            # every row that is not the f64 shape, and said nothing. Measured on
            # the live corpus: 2110 of 5905 rows (35.7%) leave here, across
            # SEVEN packs the INDEX labels `bitexact` -- gf14/48/96/128/256/512/
            # 1024, with gf256 alone contributing 2021. Setting `abs_error` to
            # "1e300" on all 2021 gf256 rows and refreshing the pack digest
            # returns exit 0 CLEAN. That field is validated by nothing: the wide
            # witness that covers those packs re-derives from `bits` and
            # compares `value`, and never reads abs_error.
            #
            # Every row is now classified into F, so a shape nobody thought
            # about is reported instead of skipped.
            if _is_wide_row(r):
                wide_rows += 1
                if not _is_exact_zero(r.get("abs_error")):
                    wide_nonzero.append({
                        "file": p["file"],
                        "name": r.get("name") or r.get("label"),
                        "abs_error": r.get("abs_error"),
                    })
                continue
            if "abs_error" not in r or "input_f64" not in r or "decoded_f64" not in r:
                unrecognised.append({"file": p["file"], "keys": sorted(r)[:8]})
                continue
            ae_stored = _to_float(r.get("abs_error"))
            inp = _to_float(r.get("input_f64"))
            dec = _to_float(r.get("decoded_f64"))
            if ae_stored is None or inp is None or dec is None:
                unparseable.append({
                    "file": p["file"],
                    "name": r.get("name"),
                    "abs_error": r.get("abs_error"),
                    "input_f64": r.get("input_f64"),
                    "decoded_f64": r.get("decoded_f64"),
                })
                continue
            rows_checked += 1

            # ---- G: each decimal against its own hex twin ----
            # T80: Check D cannot constrain an overflow row. `abs(inf - x)` is
            # `inf` for every finite x, and `NaN > tol` is False, so changing
            # gf16::overflow_to_inf's input_f64 from 1e+40 to 1.0 and refreshing
            # the digest returned exit 0 CLEAN -- a row asserting that gf16
            # encodes 1.0 as the +inf code, contradicted by its OWN
            # input_f64_hex, with nothing firing.
            #
            # The row already carries the oracle. Measured across the corpus:
            # all 3795 rows carry both hex twins and all 7590 pairs agree, so
            # this is free, in-corpus, and 100% covering -- no second tool, no
            # new data. gf16/gf32/gf64 have no independent witness at all
            # (gf-wide-conformance covers only the wide rungs), so for those
            # rungs this is the only thing that can see such an edit.
            for _dec_key, _hex_key in (("input_f64", "input_f64_hex"),
                                       ("decoded_f64", "decoded_f64_hex")):
                if _hex_key not in r:
                    continue
                _stated = _to_float(r.get(_dec_key))
                _twin = _f64_from_hex(r.get(_hex_key))
                if _stated is None or _twin is None:
                    hex_unparseable.append({"pack": p["id"], "row": r.get("name"),
                                            "field": _dec_key})
                    continue
                if _same_f64(_stated, _twin) is False:
                    hex_disagree.append({"pack": p["id"], "row": r.get("name"),
                                         "field": _dec_key,
                                         "decimal": str(_stated),
                                         "hex_decodes_to": str(_twin)})

            # ---- D2: special-value rows (inf-code / nan-code) ----
            if _is_special_row(r, inp, dec):
                special_rows += 1
                # Correctness = same-class round-trip.
                if math.isnan(inp):
                    roundtrip_ok = (math.isnan(dec))
                elif math.isinf(inp):
                    # mutant-equivalent: both guards force infinity, so >= is >
                    #
                    # T107. The boundary operator reports two surviving mutants
                    # on this line and both are functional equivalences, proven
                    # by the two guards rather than assumed. `inp` is infinite
                    # by the elif one line up; `dec` is infinite by the `and`
                    # that short-circuits before the comparison runs. For a
                    # value in {+inf, -inf}, `x > 0` and `x >= 0` agree, because
                    # the only input that separates them is zero and neither
                    # value can be zero here.
                    #
                    # Recorded because it was first classified as a CANDIDATE
                    # theorem "resting on a property of the codec that has not
                    # been checked" -- written after reading the comparison and
                    # not the elif above it. It rests on nothing but this line's
                    # own guards.
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
                # T80: a finite input that overflowed to inf is NEITHER special
                # nor finite -- it fell through both. It was counted in
                # special_rows above and again in finite_rows below, so
                # 3591 + 205 = 3796 against rows_checked 3795. Its own class,
                # counted once, and the partition is asserted below.
                overflow_rows += 1
                special_rows -= 1
                if math.isinf(dec):
                    continue

            # ---- finite-value row (the meaningful arithmetic case) ----
            finite_rows += 1
            # D: re-derive abs_error from stored decoded vs input
            rederived = abs(dec - inp)
            # T80: `NaN > tol` is False, so a NaN on either side used to make
            # this comparison assert NOTHING and pass. Setting abs_error to NaN
            # on an allowlisted finite row returned exit 0 CLEAN: E excuses the
            # row by name and D could not see it. A comparison that cannot be
            # evaluated is a FAILURE, not a match.
            _diff = abs(rederived - ae_stored)
            if math.isnan(_diff) or math.isnan(ae_stored) or math.isnan(rederived):
                rederive_mismatch.append({"pack": p["id"], "row": r.get("name"),
                                          "stored": str(ae_stored),
                                          "rederived": str(rederived),
                                          "detail": "comparison is not a number"})
            elif _diff > REDERIVE_TOL:
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
    report["checks"]["G_decimal_matches_hex"] = {
        "rows_with_twins": rows_checked,
        "disagree_count": len(hex_disagree),
        "disagree": hex_disagree[:10],
        "unparseable_count": len(hex_unparseable),
        "unparseable": hex_unparseable[:10],
        "ok": not hex_disagree and not hex_unparseable,
    }
    if hex_disagree or hex_unparseable:
        report["failures"].append({"check": "G", "disagree": hex_disagree[:10],
                                   "unparseable": hex_unparseable[:10]})

    # T80: the partition must hold, or the three numbers a reader quotes for
    # coverage do not describe one set of rows. It did not: one row was in two
    # buckets and the sum overshot by exactly 1.
    _partition_ok = (finite_rows + special_rows + overflow_rows) == rows_checked
    report["checks"]["H_row_partition"] = {
        "rows_checked": rows_checked,
        "finite_rows": finite_rows,
        "special_rows": special_rows,
        "overflow_rows": overflow_rows,
        "sum": finite_rows + special_rows + overflow_rows,
        "ok": _partition_ok,
    }
    if not _partition_ok:
        report["failures"].append({
            "check": "H",
            "detail": "finite + special + overflow != rows_checked",
            "sum": finite_rows + special_rows + overflow_rows,
            "rows_checked": rows_checked,
        })

    report["checks"]["F_row_schema_recognised"] = {
        "wide_rows": wide_rows,
        "wide_nonzero_count": len(wide_nonzero),
        "wide_nonzero": wide_nonzero[:10],
        "unrecognised_count": len(unrecognised),
        "unrecognised": unrecognised[:10],
        "unparseable_count": len(unparseable),
        "unparseable": unparseable[:10],
        "ok": not wide_nonzero and not unrecognised and not unparseable,
    }
    if wide_nonzero or unrecognised or unparseable:
        report["failures"].append({
            "check": "F",
            "wide_nonzero": wide_nonzero[:10],
            "unrecognised": unrecognised[:10],
            "unparseable": unparseable[:10],
        })

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
