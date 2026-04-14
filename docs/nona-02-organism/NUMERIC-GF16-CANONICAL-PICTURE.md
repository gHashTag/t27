# NUMERIC-GF16 — Canonical numeric picture (t27)

**Purpose:** One English page that matches **this repository’s** specs and conformance JSON. It restates the “full picture” narrative in a form reviewers can verify from paths below—without placeholder citations or numbers that contradict checked-in artifacts.

**See also:** [`NUMERIC-STANDARD-001.md`](NUMERIC-STANDARD-001.md), [`NUMERIC-GOLDENFLOAT-PALETTE.md`](NUMERIC-GOLDENFLOAT-PALETTE.md) (family + TF3 + draft-vs-repo), [`NUMERIC-WHY-NOT-GF16-EVERYWHERE.md`](NUMERIC-WHY-NOT-GF16-EVERYWHERE.md), [`NUMERIC-GF16-DEBT-INVENTORY.md`](NUMERIC-GF16-DEBT-INVENTORY.md), [`conformance/FORMAT-SPEC-001.json`](../../conformance/FORMAT-SPEC-001.json).

---

## 1. GF16 as the primary format

**GF16 (GoldenFloat 16)** is the **primary** 16-bit format in NUMERIC-STANDARD-001. Layout **1-6-9** (sign, exponent, mantissa) is specified in the standard and in `conformance/gf16_vectors.json` under `format_spec`.

**Industry note (external verification):** A **6-bit exponent / 9-bit significand** layout has appeared in vendor and research lineups (e.g. discussions of DLFloat-style encodings). Independence from the φ-based derivation does not replace conformance tests; it is context only.

---

## 2. Sacred constants (from `specs/math/constants.t27`)

These are the **canonical** `f64` literals in the Trinity math constants module (names as in the spec file):

| Name | Role |
|------|------|
| `PHI` | φ ≈ 1.6180339887498948 |
| `PHI_INV` | 1/φ ≈ 0.6180339887498948 |
| `PHI_SQ` | φ² (product of `PHI`) |
| `PHI_INV_SQ` | (1/φ)² |
| `TRINITY` | 3.0 — identity φ² + 1/φ² = 3 is documented beside these |
| `PI`, `E` | π and e |
| `G_MEASURED`, `LAMBDA_COSMO`, `OMEGA_LAMBDA_MEASURED` | CODATA-scale anchors (see file comments) |

Constants such as **τ**, **√5**, **φ³**, **Berry phase**, **project codenames**, or **Lucas L(10)** are **not** asserted here unless they appear in the same spec file or another linked SSOT.

---

## 3. GF16 numeric facts (from conformance SSOT)

From **`conformance/gf16_vectors.json`** (must stay consistent with [`NUMERIC-STANDARD-001.md`](NUMERIC-STANDARD-001.md)):

| Field | Value |
|-------|--------|
| `phi_distance` | `0.0486326415435630` (≈ **0.049** in the human-readable standard table) |
| `format_spec.sign_bits` | 1 |
| `format_spec.exp_bits` | 6 |
| `format_spec.mant_bits` | 9 |
| `format_spec.exp_bias` | 31 |
| `format_spec.exp_mant_ratio` | 2/3 |

**Representative decoded checks** (from the same file’s `test_vectors`):

- **1.0** — encodes with biased exp 31, mant 0; decoded 1.0 within listed tolerance.
- **Max test value** — scenario named `max_value` uses **65504.0** (IEEE-fp16-scale ceiling used in that artifact).
- **Min positive test** — scenario uses **≈ 6.1035×10⁻⁵** (`0.000061035`), not deep subnormals from a different format story.
- **π** — high-precision π decodes to **3.140625** with documented absolute tolerance.
- **φ** — decodes to **1.6171875** with documented relative error (~0.05%).

If a narrative uses **different** max/min/π/φ decimals, it is describing **another** encoding or a mistake; **this repo** is governed by the JSON above.

---

## 4. φ-distance (definition used here)

\[
\text{phi\_distance} = \left| \frac{E}{M} - \frac{1}{\phi} \right|
\]

with \(E\) = exponent bit count and \(M\) = mantissa bit count (excluding sign). **GF16:** \(|6/9 - 1/\phi| \approx 0.049\). **IEEE binary16:** \(|5/10 - 1/\phi| \approx 0.118\). **bfloat16:** \(|8/7 - 1/\phi| \approx 0.524\).

The file `gf16_vectors.json` also records **bf16** and **MXFP4** comparisons under `benchmark_metrics` for messaging—still **claims**, not substitutes for hardware benchmarks.

---

## 5. GoldenFloat family (same table as the standard)

Full table: [`NUMERIC-STANDARD-001.md`](NUMERIC-STANDARD-001.md) § GoldenFloat Family. Machine-readable layout: **`conformance/FORMAT-SPEC-001.json`**.

---

## 6. Conformance vectors (this repo)

**`conformance/gf16_vectors.json`** currently defines **10** entries in `test_vectors` (e.g. zero, one, max, min positive, π, φ, γ (φ⁻³), consciousness threshold (φ⁻¹), π³, roundtrip). Additional family or cross-format vectors may live in other `conformance/*` files; implementers must pass **all** vectors required by NUMERIC-STANDARD-001 for the formats they ship.

---

## 7. Benchmarks and MNIST-style tables

Structured scenarios live in **`conformance/gf_family_bench.json`** (see also **`docs/GF_FAMILY_BENCH.md`**). **Do not** copy **ad hoc** accuracy percentages into papers or posts without reconciling them to that artifact and to **`docs/nona-03-manifest/CLAIM_TIERS.md`**: some scenario fields are placeholders or need independent replication before **Tier A** claims.

Example of **safe** in-repo comparison: the **`sacred_physics_constants`** scenario reports average φ-related encoding errors for FP32, BF16, GF16, MXFP4—use those numbers when citing this repository.

---

## 8. Storage: `u16` bit pattern vs host `f16`

GF16 is a **defined 16-bit encoding**, not necessarily the host’s IEEE `f16` type. Treat the **raw `u16`** (or equivalent) as the portable interchange form; see **`NUMERIC-WHY-NOT-GF16-EVERYWHERE.md`** for why the wider codebase still uses `f32`/`f64` in many places and how migration is staged. Specific compiler bug counts or SIMD instruction counts are **out of scope** for this doc unless backed by a pinned issue list or measurement in this repo.

---

## 9. Next steps for a “public standard”

Assign a **stable name**, **version string**, and **document identifier** (e.g. DOI) in the release process you choose; keep **`FORMAT-SPEC-001.json`** and **`gf16_vectors.json`** as the executable conformance floor for the Trinity project.

---

**φ² + 1/φ² = 3 | TRINITY**
