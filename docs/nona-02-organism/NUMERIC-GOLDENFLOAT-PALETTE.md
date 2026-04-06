# NUMERIC — GoldenFloat full palette (t27 SSOT)

**Purpose:** Single English reference for the **GoldenFloat family**, related **TF3** layout, and how this differs from narrative drafts (Reddit placeholders, out-of-tree Zig paths, or invented formats). Numbers and layouts below match **`conformance/FORMAT-SPEC-001.json`**, **`specs/numeric/gf16.t27`**, **`specs/numeric/tf3.t27`**, and **`specs/math/constants.t27`** unless marked *narrative*.

**See also:** [`NUMERIC-STANDARD-001.md`](NUMERIC-STANDARD-001.md), [`NUMERIC-CORE-PALETTE-REGISTRY.md`](NUMERIC-CORE-PALETTE-REGISTRY.md) (full core registry; fixes draft formula/layout errors), [`NUMERIC-GF16-CANONICAL-PICTURE.md`](NUMERIC-GF16-CANONICAL-PICTURE.md), [`NUMERIC-PALETTE-CROSS-REPO-SYNC.md`](NUMERIC-PALETTE-CROSS-REPO-SYNC.md), [`NUMERIC-WHY-NOT-GF16-EVERYWHERE.md`](NUMERIC-WHY-NOT-GF16-EVERYWHERE.md), [`NUMERIC-GF16-DEBT-INVENTORY.md`](NUMERIC-GF16-DEBT-INVENTORY.md).

---

## 1. Unifying principle

Target exponent / mantissa balance (excluding sign):

\[
E/M \approx 1/\varphi \approx 0.618\ldots
\]

**Trinity identity** (documented across specs): \(\varphi^2 + 1/\varphi^2 = 3\). Numeric checks live in **`specs/math/constants.t27`** and **`FORMAT-SPEC-001.json`** (`sacred_constants`).

---

## 2. GoldenFloat family (normative)

Machine-readable: **`conformance/FORMAT-SPEC-001.json`**. Layout is **`[sign : S][exp : E][mant : M]`** bits.

| Format | Bits | S | E | M | Bias | φ-dist (JSON) | Notes |
|--------|------|---|---|---|------|---------------|--------|
| **GF4** | 4 | 1 | 1 | 2 | 0 | 0.118 | Masks / toy width |
| **GF8** | 8 | 1 | 3 | 4 | 3 | 0.132 | Not “~0.050” in this SSOT |
| **GF12** | 12 | 1 | 4 | 7 | 7 | 0.047 | Not “~0.037” in this SSOT |
| **GF16** | 16 | 1 | 6 | 9 | 31 | 0.049 | **PRIMARY** |
| **GF20** | 20 | 1 | 7 | 12 | 63 | 0.035 | Often omitted in short tables |
| **GF24** | 24 | 1 | 9 | 14 | **255** | 0.025 | Bias **255** here, not 63 |
| **GF32** | 32 | 1 | 12 | 19 | 2047 | 0.014 | “FP32-sized” width |

---

## 3. TF3 (ternary float in **this** repo)

**Normative:** **`specs/numeric/tf3.t27`**.

- **8 bits** total, layout **`[S(1)][E(3)][M(4)]`**, bias **3**, type alias **`TF3 = u8`**.
- This is **not** a checked-in **18-bit “TF3-9”** format. If you use that name elsewhere, treat it as **out-of-tree** or rename to avoid collision with **`tf3.t27`**.

---

## 4. Ternary weights \(\{-1,0,+1\}\) and VSA

- **Balanced ternary** and Kleene logic: **`specs/ar/ternary_logic.t27`**.
- **VSA ops** (bind / bundle / similarity): **`specs/vsa/ops.t27`**.
- **Claim tier / evidence:** Zenodo-linked items in **`docs/nona-03-manifest/RESEARCH_CLAIMS.md`** (e.g. VSA + SIMD). VSA is **experimental / research-adjacent** relative to **GF16-primary** product policy.

*Narrative* details (e.g. “12 000 trits”, “5 trits per byte”) are **not** asserted here unless they appear in those specs.

---

## 5. Numeric canon (where constants live in **t27**)

| Role | Path |
|------|------|
| Sacred math constants (`PHI`, `PHI_INV`, `PI`, `E`, `TRINITY`, CODATA anchors, …) | **`specs/math/constants.t27`** |
| φ / Trinity tolerances for interchange | **`FORMAT-SPEC-001.json`** → `sacred_constants` |

This repository **does not** contain **`src/math/constants.zig`** or **`sacred/constants.zig`** at the paths from your draft. Downstream **codegen** or the **Trinity kernel repo** may mirror names there; cite those repos separately.

*Narrative-only* symbols (Berry phase, `MU`, `PHOENIX`, Lucas checksum, `SU3`, extra `LAMBDA` scalings) belong in docs only after they are added to **`specs/math/constants.t27`** (or another declared SSOT).

---

## 6. GF16 — special bit patterns (from **`specs/numeric/gf16.t27`**)

These are **normative** in the spec module:

| Role | Raw `u16` |
|------|-----------|
| +0 | `0x0000` |
| −0 | `0x8000` |
| +∞ | `0x7E00` |
| −∞ | `0xFE00` |
| **NaN (canonical in spec)** | **`0xFE01`** |
| Components for **1.0**: sign=0, exp=0, mant=0 → **`gf16_from_components` = `0x3C00`** |

**Important:** Some drafts list NaN as `0x7E01`. In **`gf16.t27`**, positive/all-ones-exponent NaN uses **`0xFE01`** (sign set). Interop tests must follow the **spec**, not IEEE half tables blindly.

For **π, e, √2, φ**, the authoritative checks in-tree are **`conformance/gf16_vectors.json`** (`test_vectors`) and the **`gf16_encode_f32` / decode tests** in **`gf16.t27`**. Do not copy hex from external blogs without reconciling to those.

---

## 7. Conformance files (this repo)

| File | Role |
|------|------|
| **`conformance/gf16_vectors.json`** | GF16 conformance; currently **10** `test_vectors` entries |
| **`conformance/FORMAT-SPEC-001.json`** | All GoldenFloat widths + sacred constant tolerances |

There is **no** root **`conformance/vectors.json`** in **t27**. A **“33 vectors in five categories”** checklist is a **reasonable target** for a merged suite but is **not** the current single-file contract here.

---

## 8. C ABI / multi-language — status in **t27**

**`libgoldenfloat.{so,dylib,dll}`** is **not built or vendored** in this repository.

- **Reference surface** for behavior is **`specs/numeric/gf16.t27`** (`gf16_encode_f32`, `gf16_decode_to_f32`, arithmetic helpers, predicates, etc.).
- A C header listing `uint16_t` APIs is a **sensible FFI target** generated from that spec later; treat draft headers as **proposed**, not shipped.

**Language bindings (Rust / Python / Go / Node, “13/13 tests”, …):** **not present** under this tree (no `*.rs` / `*.py` / `*.go` GF16 crates here). Track them in whichever repo actually hosts the bindings, and point conformance back to **`gf16_vectors.json`**.

---

## 9. Mapping your draft to this repo

| Draft item | t27 correction |
|------------|----------------|
| Reddit links as citations | Remove; use spec paths and Zenodo DOIs from **`RESEARCH_CLAIMS.md`** where needed |
| GF8 / GF12 φ-distance “~0.05 / ~0.037” | Use **FORMAT-SPEC** values (0.132 / 0.047) |
| “TF3-9” 18-bit | Not in repo; **TF3 = 8-bit** in **`tf3.t27`** |
| GF24 bias 63 | **255** in **FORMAT-SPEC-001.json** |
| `vectors.json` + 33 vectors | Use **`gf16_vectors.json`** (10 vectors today); grow suite explicitly |
| Zig `SacredConstants` paths | Use **`constants.t27`** names / **`FORMAT-SPEC`** |
| NaN `0x7E01` | Prefer **`0xFE01`** per **`gf16.t27`** |
| Multi-language ✅ tables | **Out-of-tree** until a binding repo is linked from README |

---

**φ² + 1/φ² = 3 | TRINITY**
