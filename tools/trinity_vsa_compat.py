#!/usr/bin/env python3
"""The VSA and numeric compatibility contract of the Trinity facade: the sixteen operations
gHashTag/trinity re-exports, held to the semantics the pinned owner executes, through the spec.

WHY THIS EXISTS
---------------
S04 of gHashTag/trinity#988 (gHashTag/t27#3566): gHashTag/trinity's src/trinity.zig re-exports
bind, bundle, similarity, permutation and sequence operations from the package it pins
(gHashTag/zig-golden-float, through gHashTag/zig-hdc), while specs/vsa/vsa_core.t27 describes
the same names with other semantics for the zero trit, for unequal lengths and for the
sequence helpers. A conformance claim that does not say which of the two it means is not a
claim. specs/vsa/trinity_compat.t27 states the owner's semantics as elementwise t27 functions
(the compiler lowers scalars, not array parameters -- S02) and records, name by name, where
the canonical spec agrees and where it differs. This tool holds the vectors to the spec.

WHAT IS COMPARED WITH WHAT
--------------------------
  model   a Python statement of the owner's semantics, read from src/vsa/core.zig and
          src/ternary/hybrid.zig at OWNER_REVISION: bind multiplies over max(len) with a
          missing trit read as zero; bundle takes the sign of the sum; the dot runs over
          min(len); cosine divides by the norms of the whole vectors; hamming counts over
          max(len); permute rotates right by k mod len; bundleN takes the sign of the sum of
          all vectors. `vectors` writes conformance/vsa_trinity_compat.json from it. The
          pseudo-random trits come from this tool's own LCG, never from the owner's PRNG.
  spec    the t27 functions of specs/vsa/trinity_compat.t27, generated to C by `t27c gen-c`
          and composed into whole-vector operations by a C driver this tool writes; the
          length rules of the composition are the spec's own len_max / len_min / rotate_*
          functions. `run` replays every vector through that driver and records the verdict.
The two are independent statements of one reading of the owner: a vector that fails names a
wrong reading in the model, a wrong function in the spec, or a wrong composition. What is NOT
compared: the owner's Zig itself. The package does not build with the Zig available on this
host (spec: NOT_MEASURED), so a differential run against the owner remains the owner's own
test suite, and the record says so.

Usage:
  python3 tools/trinity_vsa_compat.py vectors [--out conformance/vsa_trinity_compat.json]
  python3 tools/trinity_vsa_compat.py run     [--report conformance/vsa_trinity_compat.json]
  python3 tools/trinity_vsa_compat.py check   [--report conformance/vsa_trinity_compat.json]
  python3 tools/trinity_vsa_compat.py --self-check

Exit codes: 0 no finding; 1 findings; 2 could not run (no t27c, no cc, no spec, no report).
"""
from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import math
import pathlib
import platform
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
SPEC = ROOT / "specs/vsa/trinity_compat.t27"
REPORT = ROOT / "conformance/vsa_trinity_compat.json"
CONST_RE = re.compile(r"^pub const (\w+) : ([^=]+?) = (.*);\s*$")
INT_TYPES = {"i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "usize", "isize"}
CONSTANTS = ["TRITS_PER_BYTE", "MAX_TRITS_PACKED", "MAX_PACKED_BYTES", "MAX_TRITS_HYBRID",
             "HYBRID_PACKED_BYTES", "SIMD_WIDTH", "TRI27_MODULUS"]
REQUIRED = ["KIND", "ID", "OWNER_REPO", "OWNER_REVISION", "OWNER_MODULE", "CONSUMER_REPO",
            "PINNED_REVISION", "FACADE_EXPORTS", "FACADE_OWNER_FN", "FACADE_SPEC_FN",
            "FACADE_FINDING", "FACADE_VERDICT", "VERDICTS", "AGREE_COUNT",
            "AGREE_ON_EQUAL_LENGTHS_COUNT", "DIFFER_COUNT"] + CONSTANTS
FLOAT_TOL = 1e-9
OPERATIONS = {
    "bind": "len_max(la, lb) trits; out[i] = trit_bind(a[i] or 0, b[i] or 0)",
    "unbind": "the same composition as bind",
    "bundle2": "len_max(la, lb) trits; out[i] = trit_bundle2(a[i] or 0, b[i] or 0)",
    "bundle3": "len_max of the three; out[i] = trit_bundle3(a[i] or 0, b[i] or 0, c[i] or 0)",
    "bundle_n": "0 vectors: empty; 1: the vector; 2: bundle2; 3: bundle3; more: trit_sign of the sum over the longest length",
    "cosine_similarity": "dot = sum of trit_bind over len_min; norms = sqrt of the trit_nonzero counts; 0.0 unless cosine_defined",
    "hamming_distance": "sum of trit_differs(a[i] or 0, b[i] or 0) over len_max",
    "hamming_similarity": "1.0 when len_max is 0, else 1 - hamming_distance / len_max",
    "dot_similarity": "0.0 when len_max is 0, else dot / len_max",
    "permute": "out[i] = v[rotate_right(i, k, len)]",
    "inverse_permute": "out[i] = v[rotate_left(i, k, len)]",
    "count_non_zero": "sum of trit_nonzero",
    "vector_norm": "sqrt(count_non_zero)",
    "tri27_bind": "tri27_bind(a, b) on register values",
    "bind_zero_identity": "the variant classified separately: len_min trits; out[i] = trit_bind_zero_identity(a[i], b[i])",
}


def sha256(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def run(cmd: list[str], cwd=None, timeout: int = 300) -> tuple[int, str, str]:
    try:
        p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout)
        return p.returncode, p.stdout, p.stderr
    except (OSError, subprocess.TimeoutExpired) as e:
        return 127, "", str(e)


def t27c_path() -> pathlib.Path | None:
    for p in ("target/release/t27c", "target/debug/t27c"):
        if (ROOT / p).exists():
            return ROOT / p
    w = shutil.which("t27c")
    return pathlib.Path(w) if w else None


def tool_version(cmd: list[str]) -> str | None:
    rc, out, err = run(cmd, timeout=60)
    text = (out or err).strip().split("\n")[0] if (out or err) else ""
    return text[:120] if rc == 0 and text else None


# ---------------------------------------------------------------------------
# The spec.
# ---------------------------------------------------------------------------
def parse_value(raw: str, typ: str):
    raw = raw.strip()
    if typ == "bool":
        return raw == "true"
    if typ in INT_TYPES:
        return int(raw)
    if typ == "str":
        return json.loads(raw)
    m = re.fullmatch(r"\[(\d+)\](\w+)", typ)
    if m:
        items = json.loads("[" + raw[1:-1] + "]") if m.group(2) == "str" else [int(x) for x in raw[1:-1].split(",") if x.strip()]
        if len(items) != int(m.group(1)):
            raise ValueError(f"annotated {typ}, holds {len(items)}")
        return items
    raise ValueError(f"unknown type {typ}")


def load_spec(path: pathlib.Path = SPEC) -> dict:
    text = path.read_text(encoding="utf-8")
    if re.search(r"[^\x00-\x7f]", text):
        raise ValueError(f"{path}: non-ASCII byte (L3)")
    f = {}
    for n, line in enumerate(text.split("\n"), 1):
        m = CONST_RE.match(line)
        if m:
            f[m.group(1)] = parse_value(m.group(3), m.group(2).strip())
        elif line.startswith("pub const"):
            raise ValueError(f"{path}:{n}: cannot read constant line")
    missing = [k for k in REQUIRED if k not in f]
    if missing:
        raise ValueError(f"{path}: missing {', '.join(missing)}")
    return f


def spec_findings(f: dict) -> list[str]:
    out = []
    n = len(f["FACADE_EXPORTS"])
    for k in ("FACADE_OWNER_FN", "FACADE_SPEC_FN", "FACADE_FINDING", "FACADE_VERDICT"):
        if len(f[k]) != n:
            out.append(f"{k}: {len(f[k])} entries for {n} exports")
    for i, v in enumerate(f["FACADE_VERDICT"]):
        if v not in f["VERDICTS"]:
            out.append(f"FACADE_VERDICT[{i}] {v!r} is not one of VERDICTS")
        if not f["FACADE_FINDING"][i].startswith(v):
            out.append(f"FACADE_FINDING[{i}] does not open with its verdict {v!r}")
    counts = {v: f["FACADE_VERDICT"].count(v) for v in f["VERDICTS"]}
    for name, key in (("AGREE_COUNT", "agrees"), ("AGREE_ON_EQUAL_LENGTHS_COUNT", "agrees on equal lengths"), ("DIFFER_COUNT", "differs")):
        if f[name] != counts.get(key, 0):
            out.append(f"{name} = {f[name]}, the verdicts hold {counts.get(key, 0)}")
    if f["MAX_TRITS_PACKED"] != f["MAX_PACKED_BYTES"] * f["TRITS_PER_BYTE"]:
        out.append("MAX_TRITS_PACKED is not MAX_PACKED_BYTES * TRITS_PER_BYTE")
    if f["HYBRID_PACKED_BYTES"] != -(-f["MAX_TRITS_HYBRID"] // f["TRITS_PER_BYTE"]):
        out.append("HYBRID_PACKED_BYTES is not ceil(MAX_TRITS_HYBRID / TRITS_PER_BYTE)")
    return out


# ---------------------------------------------------------------------------
# The model: the owner's semantics as read at OWNER_REVISION.
# ---------------------------------------------------------------------------
def sign(x: int) -> int:
    return 1 if x > 0 else (-1 if x < 0 else 0)


def _pad(v, n):
    return list(v) + [0] * (n - len(v))


def m_bind(a, b):
    n = max(len(a), len(b))
    return [x * y for x, y in zip(_pad(a, n), _pad(b, n))]


def m_bundle2(a, b):
    n = max(len(a), len(b))
    return [sign(x + y) for x, y in zip(_pad(a, n), _pad(b, n))]


def m_bundle3(a, b, c):
    n = max(len(a), len(b), len(c))
    return [sign(x + y + z) for x, y, z in zip(_pad(a, n), _pad(b, n), _pad(c, n))]


def m_bundle_n(vs):
    if not vs:
        return []
    if len(vs) == 1:
        return list(vs[0])
    if len(vs) == 2:
        return m_bundle2(vs[0], vs[1])
    if len(vs) == 3:
        return m_bundle3(vs[0], vs[1], vs[2])
    n = max(len(v) for v in vs)
    sums = [0] * n
    for v in vs:
        for i, x in enumerate(v):
            sums[i] += x
    return [sign(s) for s in sums]


def m_dot(a, b):
    n = min(len(a), len(b))
    return sum(a[i] * b[i] for i in range(n))


def m_count_non_zero(v):
    return sum(1 for x in v if x != 0)


def m_vector_norm(v):
    return math.sqrt(m_dot(v, v))


def m_cosine(a, b):
    na, nb = m_vector_norm(a), m_vector_norm(b)
    if na == 0 or nb == 0:
        return 0.0
    return m_dot(a, b) / (na * nb)


def m_hamming_distance(a, b):
    n = max(len(a), len(b))
    return sum(1 for x, y in zip(_pad(a, n), _pad(b, n)) if x != y)


def m_hamming_similarity(a, b):
    n = max(len(a), len(b))
    if n == 0:
        return 1.0
    return 1.0 - m_hamming_distance(a, b) / n


def m_dot_similarity(a, b):
    n = max(len(a), len(b))
    if n == 0:
        return 0.0
    return m_dot(a, b) / n


def m_permute(v, k):
    n = len(v)
    if n == 0:
        return []
    s = k % n
    r = [0] * n
    for i, x in enumerate(v):
        r[(i + s) % n] = x
    return r


def m_inverse_permute(v, k):
    n = len(v)
    if n == 0:
        return []
    s = k % n
    r = [0] * n
    for i, x in enumerate(v):
        r[(i + n - s) % n] = x
    return r


def m_tri27_bind(a, b, modulus=19683):
    if a == 0:
        return b
    if b == 0:
        return a
    return (a + b) % modulus


def m_bind_zero_identity(a, b):
    n = min(len(a), len(b))
    return [(b[i] if a[i] == 0 else (a[i] if b[i] == 0 else a[i] * b[i])) for i in range(n)]


def lcg_vector(seed: int, n: int) -> list[int]:
    """This tool's generator (a 64-bit LCG); not the owner's std.Random.DefaultPrng."""
    s = seed & 0xFFFFFFFFFFFFFFFF
    out = []
    for _ in range(n):
        s = (s * 6364136223846793005 + 1442695040888963407) & 0xFFFFFFFFFFFFFFFF
        out.append(((s >> 33) % 3) - 1)
    return out


def fl(x: float) -> float:
    return round(float(x), 12)


def build_vectors() -> list[dict]:
    V = []

    def add(vid, op, inputs, expect, note):
        V.append({"id": vid, "op": op, "inputs": inputs, "expect": expect, "note": note})

    ties = [1, -1, 0, 1, 0, -1, 1, -1, 0]
    pairs = [
        ("equal-8", lcg_vector(1, 8), lcg_vector(2, 8), "equal lengths"),
        ("equal-32", lcg_vector(3, 32), lcg_vector(4, 32), "one full SIMD chunk of the owner"),
        ("equal-33", lcg_vector(5, 33), lcg_vector(6, 33), "a chunk plus a remainder trit"),
        ("equal-100", lcg_vector(7, 100), lcg_vector(8, 100), "three chunks and a remainder"),
        ("unequal-5-8", lcg_vector(9, 5), lcg_vector(10, 8), "unequal lengths: the policies differ here"),
        ("unequal-40-32", lcg_vector(11, 40), lcg_vector(12, 32), "unequal lengths across a chunk boundary"),
        ("zero-vs-random-8", [0] * 8, lcg_vector(13, 8), "an all-zero vector: bind annihilates, cosine is 0.0"),
        ("all-zero-8", [0] * 8, [0] * 8, "two all-zero vectors"),
        ("empty-empty", [], [], "two empty vectors: hamming similarity 1.0, dot similarity 0.0"),
        ("empty-3", [], lcg_vector(14, 3), "an empty vector against a short one"),
        ("opposite-9", ties, [-x for x in ties], "a vector against its negation: every bundle2 is a tie"),
        ("same-9", ties, list(ties), "a vector against itself: hamming 0, cosine 1.0 (norm 6)"),
        ("mixed-9", ties, [1, 1, 1, -1, -1, -1, 0, 0, 0], "every pair of trits once"),
    ]
    for name, a, b, note in pairs:
        add(f"bind-{name}", "bind", {"a": a, "b": b}, m_bind(a, b), note)
        add(f"unbind-{name}", "unbind", {"a": a, "b": b}, m_bind(a, b), note)
        add(f"bundle2-{name}", "bundle2", {"a": a, "b": b}, m_bundle2(a, b), note)
        add(f"cosine-{name}", "cosine_similarity", {"a": a, "b": b}, fl(m_cosine(a, b)), note)
        add(f"hamming-distance-{name}", "hamming_distance", {"a": a, "b": b}, m_hamming_distance(a, b), note)
        add(f"hamming-similarity-{name}", "hamming_similarity", {"a": a, "b": b}, fl(m_hamming_similarity(a, b)), note)
        add(f"dot-similarity-{name}", "dot_similarity", {"a": a, "b": b}, fl(m_dot_similarity(a, b)), note)
        add(f"bind-zero-identity-{name}", "bind_zero_identity", {"a": a, "b": b}, m_bind_zero_identity(a, b), note + " (the variant classified separately)")
    triples = [
        ("ties-9", ties, [-x for x in ties], [1, 0, -1, 0, 1, -1, 0, 1, -1], "the third vector decides every tie"),
        ("equal-33", lcg_vector(15, 33), lcg_vector(16, 33), lcg_vector(17, 33), "a chunk plus a remainder"),
        ("unequal-8-5-3", lcg_vector(18, 8), lcg_vector(19, 5), lcg_vector(20, 3), "three lengths"),
        ("empty-empty-empty", [], [], [], "three empty vectors"),
    ]
    for name, a, b, c, note in triples:
        add(f"bundle3-{name}", "bundle3", {"a": a, "b": b, "c": c}, m_bundle3(a, b, c), note)
    groups = [
        ("none", [], "no vectors: empty"),
        ("one-8", [lcg_vector(21, 8)], "one vector: itself"),
        ("two-8", [lcg_vector(22, 8), lcg_vector(23, 8)], "two: bundle2"),
        ("three-8", [lcg_vector(24, 8), lcg_vector(25, 8), lcg_vector(26, 8)], "three: bundle3"),
        ("four-ties", [[1, 1, -1, 0], [1, -1, -1, 0], [-1, 1, 1, 0], [-1, -1, 1, 0]], "four: the sign of the sum, ties are zero"),
        ("five-unequal", [lcg_vector(27, 8), lcg_vector(28, 5), lcg_vector(29, 3), lcg_vector(30, 8), lcg_vector(31, 6)], "five of unequal lengths: the longest wins"),
        ("seven-33", [lcg_vector(40 + i, 33) for i in range(7)], "seven vectors across a chunk boundary"),
    ]
    for name, vs, note in groups:
        add(f"bundle-n-{name}", "bundle_n", {"vectors": vs}, m_bundle_n(vs), note)
    v8, v33 = lcg_vector(32, 8), lcg_vector(33, 33)
    for name, v, k, note in [("8-by-0", v8, 0, "k == 0 is the identity"), ("8-by-1", v8, 1, "one step right"),
                             ("8-by-8", v8, 8, "k == len is the identity"), ("8-by-11", v8, 11, "k > len reduces mod len"),
                             ("33-by-5", v33, 5, "across a chunk boundary"), ("empty-by-3", [], 3, "an empty vector stays empty")]:
        add(f"permute-{name}", "permute", {"v": v, "k": k}, m_permute(v, k), note)
        add(f"inverse-permute-{name}", "inverse_permute", {"v": v, "k": k}, m_inverse_permute(v, k), note)
        add(f"inverse-permute-undoes-permute-{name}", "inverse_permute", {"v": m_permute(v, k), "k": k}, list(v), "inverse_permute(permute(v, k), k) == v")
    for name, v, note in [("8", v8, "eight trits"), ("zeros-8", [0] * 8, "no nonzero trit: norm 0.0"), ("empty", [], "empty: 0 and 0.0"), ("100", lcg_vector(34, 100), "a hundred trits")]:
        add(f"count-non-zero-{name}", "count_non_zero", {"v": v}, m_count_non_zero(v), note)
        add(f"vector-norm-{name}", "vector_norm", {"v": v}, fl(m_vector_norm(v)), note)
    for a, b, note in [(0, 5, "zero is the identity"), (5, 0, "zero is the identity"), (19682, 2, "wraps at the modulus"),
                       (-5, 2, "a negative sum is fixed up"), (1, 1, "plain addition"), (19682, 19682, "two maxima"),
                       (-1, -1, "two minus ones"), (9841, 9842, "sums to the modulus: zero")]:
        add(f"tri27-bind-{a}-{b}", "tri27_bind", {"a": a, "b": b}, m_tri27_bind(a, b), note)
    return V


def build_report(f: dict) -> dict:
    return {
        "schema_version": 1,
        "format_family": "Conformance",
        "vector_name": "Trinity VSA facade compatibility",
        "module": "vsa_trinity_compat",
        "spec_path": "specs/vsa/trinity_compat.t27",
        "description": "Golden vectors for the sixteen VSA operations gHashTag/trinity re-exports, as the pinned owner executes them; replayed through the elementwise functions of the spec, generated to C",
        "generator": "tools/trinity_vsa_compat.py vectors (a Python statement of the owner semantics; the pseudo-random trits come from this tool's LCG, not from the owner's PRNG)",
        "created_at": "2026-09-12",
        "reference": {"owner_repo": f["OWNER_REPO"], "owner_revision": f["OWNER_REVISION"], "owner_module": f["OWNER_MODULE"],
                      "consumer_repo": f["CONSUMER_REPO"], "pinned_revision": f["PINNED_REVISION"]},
        "constants": {k: f[k] for k in CONSTANTS},
        "operations": OPERATIONS,
        "float_tolerance": FLOAT_TOL,
        "vectors": build_vectors(),
    }


# ---------------------------------------------------------------------------
# The replay: the spec's functions, generated to C, composed by a driver.
# ---------------------------------------------------------------------------
DRIVER_HEAD = r"""
#include <stdio.h>
#include <math.h>
#include <string.h>
#include "spec.c"
#define MAXN 1024
static int passes = 0, failures = 0;
static int8_t at(const int8_t *v, uint32_t len, uint32_t i) { return i < len ? v[i] : 0; }
static uint32_t c_bind(const int8_t *a, uint32_t la, const int8_t *b, uint32_t lb, int8_t *out) {
    uint32_t n = len_max(la, lb);
    for (uint32_t i = 0; i < n; i++) out[i] = trit_bind(at(a, la, i), at(b, lb, i));
    return n;
}
static uint32_t c_bind_zero_identity(const int8_t *a, uint32_t la, const int8_t *b, uint32_t lb, int8_t *out) {
    uint32_t n = len_min(la, lb);
    for (uint32_t i = 0; i < n; i++) out[i] = trit_bind_zero_identity(a[i], b[i]);
    return n;
}
static uint32_t c_bundle2(const int8_t *a, uint32_t la, const int8_t *b, uint32_t lb, int8_t *out) {
    uint32_t n = len_max(la, lb);
    for (uint32_t i = 0; i < n; i++) out[i] = trit_bundle2(at(a, la, i), at(b, lb, i));
    return n;
}
static uint32_t c_bundle3(const int8_t *a, uint32_t la, const int8_t *b, uint32_t lb, const int8_t *c, uint32_t lc, int8_t *out) {
    uint32_t n = len_max(len_max(la, lb), lc);
    for (uint32_t i = 0; i < n; i++) out[i] = trit_bundle3(at(a, la, i), at(b, lb, i), at(c, lc, i));
    return n;
}
static uint32_t c_bundle_n(const int8_t **vs, const uint32_t *ls, uint32_t m, int8_t *out) {
    if (m == 0) return 0;
    if (m == 1) { memcpy(out, vs[0], ls[0]); return ls[0]; }
    if (m == 2) return c_bundle2(vs[0], ls[0], vs[1], ls[1], out);
    if (m == 3) return c_bundle3(vs[0], ls[0], vs[1], ls[1], vs[2], ls[2], out);
    uint32_t n = 0;
    for (uint32_t j = 0; j < m; j++) n = len_max(n, ls[j]);
    for (uint32_t i = 0; i < n; i++) {
        long sum = 0;
        for (uint32_t j = 0; j < m; j++) sum += at(vs[j], ls[j], i);
        if (sum > 127) sum = 127;
        if (sum < -127) sum = -127;
        out[i] = trit_sign((int8_t)sum);
    }
    return n;
}
static long long c_dot(const int8_t *a, uint32_t la, const int8_t *b, uint32_t lb) {
    uint32_t n = len_min(la, lb);
    long long d = 0;
    for (uint32_t i = 0; i < n; i++) d += trit_bind(a[i], b[i]);
    return d;
}
static uint32_t c_count_non_zero(const int8_t *v, uint32_t lv) {
    uint32_t c = 0;
    for (uint32_t i = 0; i < lv; i++) c += trit_nonzero(v[i]);
    return c;
}
static double c_vector_norm(const int8_t *v, uint32_t lv) { return sqrt((double)c_count_non_zero(v, lv)); }
static double c_cosine(const int8_t *a, uint32_t la, const int8_t *b, uint32_t lb) {
    uint32_t na = c_count_non_zero(a, la), nb = c_count_non_zero(b, lb);
    if (!cosine_defined(na, nb)) return 0.0;
    return (double)c_dot(a, la, b, lb) / (sqrt((double)na) * sqrt((double)nb));
}
static uint32_t c_hamming_distance(const int8_t *a, uint32_t la, const int8_t *b, uint32_t lb) {
    uint32_t n = len_max(la, lb), d = 0;
    for (uint32_t i = 0; i < n; i++) d += trit_differs(at(a, la, i), at(b, lb, i));
    return d;
}
static double c_hamming_similarity(const int8_t *a, uint32_t la, const int8_t *b, uint32_t lb) {
    uint32_t n = len_max(la, lb);
    if (n == 0) return 1.0;
    return 1.0 - (double)c_hamming_distance(a, la, b, lb) / (double)n;
}
static double c_dot_similarity(const int8_t *a, uint32_t la, const int8_t *b, uint32_t lb) {
    uint32_t n = len_max(la, lb);
    if (n == 0) return 0.0;
    return (double)c_dot(a, la, b, lb) / (double)n;
}
static uint32_t c_permute(const int8_t *v, uint32_t lv, uint32_t k, int8_t *out) {
    for (uint32_t i = 0; i < lv; i++) out[i] = v[rotate_right(i, k, lv)];
    return lv;
}
static uint32_t c_inverse_permute(const int8_t *v, uint32_t lv, uint32_t k, int8_t *out) {
    for (uint32_t i = 0; i < lv; i++) out[i] = v[rotate_left(i, k, lv)];
    return lv;
}
static void check_trits(const char *vid, const int8_t *got, uint32_t n, const int8_t *want, uint32_t wn) {
    if (n != wn) { printf("[VEC] %s : FAILED (length %u, want %u)\n", vid, n, wn); failures++; return; }
    for (uint32_t i = 0; i < n; i++) if (got[i] != want[i]) { printf("[VEC] %s : FAILED (trit %u is %d, want %d)\n", vid, i, got[i], want[i]); failures++; return; }
    printf("[VEC] %s : PASSED\n", vid); passes++;
}
static void check_int(const char *vid, long long got, long long want) {
    if (got != want) { printf("[VEC] %s : FAILED (%lld, want %lld)\n", vid, got, want); failures++; return; }
    printf("[VEC] %s : PASSED\n", vid); passes++;
}
static void check_f64(const char *vid, double got, double want, double tol) {
    if (fabs(got - want) > tol) { printf("[VEC] %s : FAILED (%.12f, want %.12f)\n", vid, got, want); failures++; return; }
    printf("[VEC] %s : PASSED\n", vid); passes++;
}
int main(void) {
    int8_t out[MAXN];
"""
DRIVER_TAIL = r"""
    printf("vectors: %d passed, %d failed\n", passes, failures);
    return failures ? 1 : 0;
}
"""


def c_arr(name: str, v: list[int]) -> str:
    body = ", ".join(str(x) for x in v) if v else "0"
    return f"static const int8_t {name}[] = {{{body}}}; const uint32_t {name}_n = {len(v)};"


def driver_block(i: int, vec: dict, tol: float) -> str:
    op, inp, exp = vec["op"], vec["inputs"], vec["expect"]
    vid = json.dumps(vec["id"])
    lines = ["    {"]
    if op in ("bind", "unbind", "bundle2", "bind_zero_identity"):
        lines += [c_arr(f"a{i}", inp["a"]), c_arr(f"b{i}", inp["b"]), c_arr(f"w{i}", exp)]
        fn = {"bind": "c_bind", "unbind": "c_bind", "bundle2": "c_bundle2", "bind_zero_identity": "c_bind_zero_identity"}[op]
        lines.append(f"uint32_t n = {fn}(a{i}, a{i}_n, b{i}, b{i}_n, out); check_trits({vid}, out, n, w{i}, w{i}_n);")
    elif op == "bundle3":
        lines += [c_arr(f"a{i}", inp["a"]), c_arr(f"b{i}", inp["b"]), c_arr(f"c{i}", inp["c"]), c_arr(f"w{i}", exp)]
        lines.append(f"uint32_t n = c_bundle3(a{i}, a{i}_n, b{i}, b{i}_n, c{i}, c{i}_n, out); check_trits({vid}, out, n, w{i}, w{i}_n);")
    elif op == "bundle_n":
        vs = inp["vectors"]
        for j, v in enumerate(vs):
            lines.append(c_arr(f"v{i}_{j}", v))
        lines.append(c_arr(f"w{i}", exp))
        if vs:
            lines.append("const int8_t *vs[] = {" + ", ".join(f"v{i}_{j}" for j in range(len(vs))) + "}; const uint32_t ls[] = {" + ", ".join(f"v{i}_{j}_n" for j in range(len(vs))) + "};")
            lines.append(f"uint32_t n = c_bundle_n(vs, ls, {len(vs)}, out); check_trits({vid}, out, n, w{i}, w{i}_n);")
        else:
            lines.append(f"uint32_t n = c_bundle_n(NULL, NULL, 0, out); check_trits({vid}, out, n, w{i}, w{i}_n);")
    elif op in ("cosine_similarity", "hamming_similarity", "dot_similarity"):
        lines += [c_arr(f"a{i}", inp["a"]), c_arr(f"b{i}", inp["b"])]
        fn = {"cosine_similarity": "c_cosine", "hamming_similarity": "c_hamming_similarity", "dot_similarity": "c_dot_similarity"}[op]
        lines.append(f"check_f64({vid}, {fn}(a{i}, a{i}_n, b{i}, b{i}_n), {exp!r}, {tol!r});")
    elif op == "hamming_distance":
        lines += [c_arr(f"a{i}", inp["a"]), c_arr(f"b{i}", inp["b"])]
        lines.append(f"check_int({vid}, c_hamming_distance(a{i}, a{i}_n, b{i}, b{i}_n), {int(exp)});")
    elif op in ("permute", "inverse_permute"):
        lines += [c_arr(f"v{i}", inp["v"]), c_arr(f"w{i}", exp)]
        fn = "c_permute" if op == "permute" else "c_inverse_permute"
        lines.append(f"uint32_t n = {fn}(v{i}, v{i}_n, {int(inp['k'])}u, out); check_trits({vid}, out, n, w{i}, w{i}_n);")
    elif op == "count_non_zero":
        lines += [c_arr(f"v{i}", inp["v"]), f"check_int({vid}, c_count_non_zero(v{i}, v{i}_n), {int(exp)});"]
    elif op == "vector_norm":
        lines += [c_arr(f"v{i}", inp["v"]), f"check_f64({vid}, c_vector_norm(v{i}, v{i}_n), {exp!r}, {tol!r});"]
    elif op == "tri27_bind":
        lines.append(f"check_int({vid}, tri27_bind({int(inp['a'])}, {int(inp['b'])}), {int(exp)});")
    else:
        raise ValueError(f"vector {vec['id']}: unknown op {op}")
    lines.append("}")
    return "\n    ".join(lines)


def replay(spec: pathlib.Path, report: dict, t27c: pathlib.Path, work: pathlib.Path) -> dict:
    """gen-c the spec, build the driver over the report's vectors, run it; return the replay record."""
    work.mkdir(parents=True, exist_ok=True)
    rc, out, err = run([str(t27c), "gen-c", str(spec)])
    if rc != 0 or not out.strip():
        return {"ok": False, "stage": "generate", "detail": (err or out).strip()[:400], "passed": 0, "failed": 0, "failures": []}
    spec_c = work / "spec.c"
    spec_c.write_text(out, encoding="utf-8")
    tol = float(report.get("float_tolerance", FLOAT_TOL))
    driver = DRIVER_HEAD + "\n".join(driver_block(i, v, tol) for i, v in enumerate(report["vectors"])) + DRIVER_TAIL
    (work / "driver.c").write_text(driver, encoding="utf-8")
    rc, out, err = run(["cc", "-std=c11", "-I", str(work), "-x", "c", str(work / "driver.c"), "-o", str(work / "driver"), "-lm"])
    if rc != 0:
        return {"ok": False, "stage": "compile", "detail": err.strip()[:600], "passed": 0, "failed": 0, "failures": [], "generated_c": sha256(spec_c.read_bytes())}
    rc, out, err = run([str(work / "driver")], timeout=120)
    results = re.findall(r"^\[VEC\] (.+?) : (PASSED|FAILED)(?: \((.*)\))?$", out, re.M)
    failures = [{"id": vid, "detail": d or ""} for vid, v, d in results if v == "FAILED"]
    passed = sum(1 for _, v, _ in results if v == "PASSED")
    m = re.search(r"^vectors: (\d+) passed, (\d+) failed$", out, re.M)
    ok = rc == 0 and m is not None and int(m.group(2)) == 0 and passed == len(report["vectors"]) and not failures
    if m is None:
        return {"ok": False, "stage": "runtime", "detail": f"driver exit {rc}, no summary line", "passed": passed, "failed": len(failures), "failures": failures[:50], "generated_c": sha256(spec_c.read_bytes())}
    return {"ok": ok, "stage": "runtime", "detail": "", "passed": passed, "failed": len(failures), "failures": failures[:50], "generated_c": sha256(spec_c.read_bytes())}


def replay_record(spec: pathlib.Path, report: dict, t27c: pathlib.Path) -> dict:
    with tempfile.TemporaryDirectory() as tmp:
        r = replay(spec, report, t27c, pathlib.Path(tmp))
    r["at"] = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    r["t27c"] = tool_version([str(t27c), "--version"]) or "t27c"
    r["cc"] = tool_version(["cc", "--version"]) or "cc"
    r["host"] = f"{platform.system()} {platform.machine()}"
    r["spec_sha256"] = sha256(spec.read_bytes())
    r["vectors"] = len(report["vectors"])
    return r


# ---------------------------------------------------------------------------
# check: the report is what the model and the spec say now.
# ---------------------------------------------------------------------------
def check_report(spec_path: pathlib.Path, report: dict) -> list[str]:
    findings = []
    try:
        f = load_spec(spec_path)
    except ValueError as e:
        return [str(e)]
    findings += spec_findings(f)
    fresh = build_report(f)
    for key in ("vectors", "constants", "operations", "reference", "float_tolerance"):
        if json.dumps(report.get(key), sort_keys=True) != json.dumps(fresh[key], sort_keys=True):
            findings.append(f"{key}: the report drifts from what `vectors` writes now")
    ids = [v["id"] for v in report.get("vectors", [])]
    if len(set(ids)) != len(ids):
        findings.append("vectors: duplicate ids")
    ops = {v["op"] for v in report.get("vectors", [])}
    for op in OPERATIONS:
        if op not in ops:
            findings.append(f"vectors: operation {op} has no vector")
    r = report.get("replay")
    if not isinstance(r, dict):
        findings.append("replay: absent; run `run`")
        return findings
    if r.get("spec_sha256") != sha256(spec_path.read_bytes()):
        findings.append("replay: the spec changed since the replay; run `run`")
    if r.get("vectors") != len(report.get("vectors", [])):
        findings.append("replay: the vector count changed since the replay; run `run`")
    if not r.get("ok") or r.get("failed", 1) != 0 or r.get("passed") != len(report.get("vectors", [])):
        findings.append(f"replay: {r.get('passed')} passed, {r.get('failed')} failed at stage {r.get('stage')}: {r.get('detail', '')[:200]}")
    return findings


# ---------------------------------------------------------------------------
# --self-check: the model, the replay and the check each report what they must.
# ---------------------------------------------------------------------------
def self_check() -> int:
    t27c = t27c_path()
    if t27c is None or shutil.which("cc") is None:
        print("trinity_vsa_compat --self-check: needs t27c (target/release/t27c) and cc", file=sys.stderr)
        return 2
    ok = True

    def expect(cond: bool, what: str):
        nonlocal ok
        print(f"  {'ok ' if cond else 'BAD'} {what}")
        ok = ok and cond

    expect(m_bind([0], [1]) == [0] and m_bind_zero_identity([0], [1]) == [1], "model: bind annihilates on zero, the variant keeps the identity")
    expect(m_bind([1, -1], [1]) == [1, 0] and m_bind_zero_identity([1, -1], [1]) == [1], "model: bind pads to max(len), the variant cuts to min(len)")
    expect(m_cosine([0, 0], [1, 1]) == 0.0 and m_hamming_similarity([], []) == 1.0 and m_dot_similarity([], []) == 0.0, "model: the zero-norm and empty rules")
    v = lcg_vector(99, 33)
    expect(m_inverse_permute(m_permute(v, 7), 7) == v and m_permute(v, 33) == v, "model: the rotation law and k == len")
    expect(m_bundle_n([[1], [1], [-1], [-1]]) == [0] and m_bundle_n([[1, 0], [1], [-1]]) == [1, 0], "model: bundleN ties and its bundle3 delegation")
    expect(m_tri27_bind(-5, 2) == 19680 and m_tri27_bind(0, 7) == 7, "model: the TRI27 fix-up and identity")
    f = load_spec(SPEC)
    expect(not spec_findings(f), "spec: the facade tables and the counts agree")
    report = build_report(f)
    expect(len(report["vectors"]) >= 100 and len({x["id"] for x in report["vectors"]}) == len(report["vectors"]), f"vectors: {len(report['vectors'])} unique ids")
    with tempfile.TemporaryDirectory() as tmp:
        work = pathlib.Path(tmp)
        r = replay(SPEC, report, t27c, work / "good")
        expect(r["ok"] and r["failed"] == 0 and r["passed"] == len(report["vectors"]), f"replay: the spec passes every vector ({r['passed']} passed, {r['failed']} failed{', ' + r['detail'] if r['detail'] else ''})")
        # planted: one wrong expectation in the vectors is one named failure
        bad = json.loads(json.dumps(report))
        target = next(x for x in bad["vectors"] if x["op"] == "bind" and x["expect"])
        target["expect"][0] = -target["expect"][0] if target["expect"][0] != 0 else 1
        r = replay(SPEC, bad, t27c, work / "bad")
        expect(r["failed"] == 1 and r["failures"][0]["id"] == target["id"], f"planted: a wrong expected trit fails exactly its vector ({r['failures'][0]['id'] if r['failures'] else 'none'})")
        # planted: a spec whose bind keeps the zero identity fails the bind vectors and only them
        text = SPEC.read_text(encoding="utf-8")
        planted = text.replace("pub fn trit_bind(a: i8, b: i8) -> i8 {\n    return a * b;\n}",
                               "pub fn trit_bind(a: i8, b: i8) -> i8 {\n    if (a == 0) { return b; }\n    if (b == 0) { return a; }\n    return a * b;\n}")
        expect(planted != text, "planted: the bind body was found to replace")
        (work / "spec").mkdir()
        (work / "spec" / "trinity_compat.t27").write_text(planted, encoding="utf-8")
        r = replay(work / "spec" / "trinity_compat.t27", report, t27c, work / "spec")
        failed_ops = {next(x["op"] for x in report["vectors"] if x["id"] == fl_["id"]) for fl_ in r["failures"]}
        expect(r["failed"] > 0 and failed_ops and failed_ops <= {"bind", "unbind", "cosine_similarity", "dot_similarity"},
               f"planted: a zero-identity bind fails only bind-shaped vectors ({r['failed']} failures in {sorted(failed_ops)})")
        # planted: check sees drifted vectors and a stale replay
        good = dict(report, replay=dict(replay_record(SPEC, report, t27c)))
        expect(not check_report(SPEC, good), "check: a fresh report has no finding")
        drifted = json.loads(json.dumps(good))
        drifted["vectors"][0]["expect"] = list(reversed(drifted["vectors"][0]["expect"])) if isinstance(drifted["vectors"][0]["expect"], list) else 0
        expect(any("drifts" in x for x in check_report(SPEC, drifted)), "check: an edited vector is reported as drift")
        stale = json.loads(json.dumps(good))
        stale["replay"]["spec_sha256"] = "sha256:0"
        expect(any("spec changed" in x for x in check_report(SPEC, stale)), "check: a replay of another spec text is reported as stale")
        failed = json.loads(json.dumps(good))
        failed["replay"]["failed"] = 1
        failed["replay"]["ok"] = False
        expect(any("failed" in x for x in check_report(SPEC, failed)), "check: a recorded failure is a finding")
    print("trinity_vsa_compat --self-check:", "ok" if ok else "FAILED")
    return 0 if ok else 1


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("command", nargs="?", choices=["vectors", "run", "check"])
    ap.add_argument("--spec", default=str(SPEC))
    ap.add_argument("--out", default=str(REPORT))
    ap.add_argument("--report", default=str(REPORT))
    ap.add_argument("--self-check", action="store_true")
    a = ap.parse_args()
    if a.self_check:
        return self_check()
    if a.command is None:
        ap.print_help()
        return 2
    spec = pathlib.Path(a.spec)
    if not spec.exists():
        print(f"no spec at {spec}", file=sys.stderr)
        return 2
    try:
        f = load_spec(spec)
    except ValueError as e:
        print(f"finding: {e}")
        return 1
    if a.command == "vectors":
        report = build_report(f)
        out = pathlib.Path(a.out)
        if out.exists():
            old = json.loads(out.read_text(encoding="utf-8"))
            if "replay" in old:
                report["replay"] = old["replay"]
        out.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"wrote {out}: {len(report['vectors'])} vectors over {len(OPERATIONS)} operations")
        return 0
    rp = pathlib.Path(a.report)
    if not rp.exists():
        print(f"no report at {rp}; run `vectors` first", file=sys.stderr)
        return 2
    report = json.loads(rp.read_text(encoding="utf-8"))
    if a.command == "run":
        t27c = t27c_path()
        if t27c is None or shutil.which("cc") is None:
            print("run: needs t27c (target/release/t27c) and cc", file=sys.stderr)
            return 2
        report["replay"] = replay_record(spec, report, t27c)
        rp.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        r = report["replay"]
        print(f"replay: {r['passed']} passed, {r['failed']} failed of {r['vectors']} vectors (stage {r['stage']}) with {r['t27c']} and {r['cc']}")
        for x in r["failures"][:20]:
            print(f"  FAILED {x['id']}: {x['detail']}")
        if r["detail"] and not r["ok"]:
            print(f"  {r['detail'][:400]}")
        return 0 if r["ok"] else 1
    findings = check_report(spec, report) + spec_findings(f)
    seen = []
    for x in findings:
        if x not in seen:
            seen.append(x)
    for x in seen:
        print(f"finding: {x}")
    r = report.get("replay", {})
    print(f"check: {len(report.get('vectors', []))} vectors, {len(OPERATIONS)} operations, replay {r.get('passed')}/{r.get('vectors')} passed at {r.get('at')}; {len(seen)} finding(s)")
    return 1 if seen else 0


if __name__ == "__main__":
    sys.exit(main())
