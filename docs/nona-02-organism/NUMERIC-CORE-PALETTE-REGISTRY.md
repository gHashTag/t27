# NUMERIC — Core palette registry (t27 English SSOT)

**Status:** Normative **in-tree** registry. **Language:** English (**LANG-EN**).  
**Non-English research drafts** must be reconciled here; do not treat them as SSOT when they conflict with **`conformance/FORMAT-SPEC-001.json`** or **`specs/numeric/*.t27`**.

**Companions:** [`NUMERIC-STANDARD-001.md`](NUMERIC-STANDARD-001.md), [`NUMERIC-GOLDENFLOAT-PALETTE.md`](NUMERIC-GOLDENFLOAT-PALETTE.md), [`NUMERIC-PALETTE-CROSS-REPO-SYNC.md`](NUMERIC-PALETTE-CROSS-REPO-SYNC.md), [`CLAIM_TIERS.md`](../nona-03-manifest/CLAIM_TIERS.md).

---

## Axiom 0 — Trinity identity (algebraic)

Let \(\varphi = (1+\sqrt5)/2\). Then

\[
\varphi^2 = \frac{3+\sqrt5}{2},\quad \frac{1}{\varphi^2} = \frac{3-\sqrt5}{2},\quad \varphi^2 + \frac{1}{\varphi^2} = 3.
\]

**t27:** Constants and commentary in **`specs/math/constants.t27`**; interchange tolerances in **`FORMAT-SPEC-001.json`** (`TRINITY`). **Floating-point** equality at runtime is **not** exact—use stated tolerances or integer/rational proofs in docs.

**Bridge to ternary:** The scalar **3** aligns with **three-valued** logic / trits as a **design metaphor**; the formal link is specified in **`specs/ar/ternary_logic.t27`** and related modules—not via floating-point identity alone.

**“Master formula”** \(V = n \times 3^k \times \pi^m \times \varphi^p \times e^q\) (and extensions): appears as a **speculative / physics mapping** in **`specs/physics/zamolodchikov_4d_conjecture.t27`**. Tag with **`claim_tier`** per **`CLAIM_TIERS.md`** (`conjecture` / `empirical_fit` unless upgraded with proof artifacts).

---

## 1. GoldenFloat family

### 1.1 φ-distance (normative)

Use **exponent-bit count** \(E\) and **mantissa-bit count** \(M\) (**excluding sign**):

\[
\text{phi\_distance} = \left| \frac{E}{M} - \frac{1}{\varphi} \right|.
\]

**Do not** use \(|M/\text{total\_bits} - 1/\varphi|\)—that is **not** the project metric.

### 1.2 Format table (must match JSON)

Source: **`conformance/FORMAT-SPEC-001.json`** and [`NUMERIC-STANDARD-001.md`](NUMERIC-STANDARD-001.md).

| Format | Bits | S | E | M | Bias | φ-dist |
|--------|------|---|---|---|------|--------|
| GF4 | 4 | 1 | 1 | 2 | 0 | 0.118 |
| GF8 | 8 | 1 | **3** | **4** | 3 | 0.132 |
| GF12 | 12 | 1 | **4** | **7** | 7 | 0.047 |
| **GF16** | 16 | 1 | **6** | **9** | 31 | **0.049** (primary) |
| GF20 | 20 | 1 | 7 | 12 | 63 | 0.035 |
| GF24 | 24 | 1 | **9** | **14** | **255** | 0.025 |
| GF32 | 32 | 1 | **12** | **19** | **2047** | 0.014 |

**Specs:** `specs/numeric/gf4.t27` … `gf32.t27` (including **GF8 / GF24 / GF32**—present as **`.t27`**; any “missing Zig” issue is **codegen/hand-Zig debt**, not missing spec files).

**“TF3-9” (18-bit)** and **φ-dist 0.018** are **not** in **FORMAT-SPEC-001.json**. In-tree **TF3** is **`specs/numeric/tf3.t27`**: **8 bits**, `[1][3][4]`, **`TF3 = u8`**.

---

## 2. GF16 — value and specials

**Normative decode** for finite values (**`specs/numeric/gf16.t27`** — `gf16_decode_to_f32`):

\[
\text{value} = (-1)^{s} \times \left(1 + \frac{m}{512}\right) \times 2^{(e - 31)}
\]

where \(s\) is sign, \(e\) is the **stored exponent field** (0…63), \(m\) is the **9-bit mantissa** (0…511), bias **31**.

**Specials** (same module):

| Pattern | Meaning |
|---------|---------|
| `0x0000` | +0 |
| `0x8000` | −0 |
| `exp == 63`, `mant == 0` | ±∞ (`0x7E00` / `0xFE00`) |
| `exp == 63`, `mant != 0` | NaN (canonical pattern in spec: `0xFE01`) |

**Common mistake:** **`0x3C00`** is the **full 16-bit encoding** for a representable value (including **`1.0`** in the spec’s tests), **not** an “exponent field” value.

---

## 3. TF3 (in-repo semantics)

**`tf3_to_f32`** uses a **binary** radix in the decoded value:

\[
\text{value} = (-1)^{s} \times \left(1 + \frac{m}{16}\right) \times 2^{(e - 3)}
\]

with **4-bit** mantissa, exponent bias **3**, **8-bit** container. This is **not** the same as a base-3 float with bias 31—do not copy that narrative into t27 specs without a new ADR and code change.

---

## 4. PackedTrit and ternary width

**SSOT:** **`specs/base/types.t27`** — **`PackedTrit = u8`** holds **eight** trits (positions 0–7).  
**Alternative “5 trits per byte”** packing lives in **sibling-repo narratives** only—see [`NUMERIC-PALETTE-CROSS-REPO-SYNC.md`](NUMERIC-PALETTE-CROSS-REPO-SYNC.md).

**HybridBigInt / TVC / 59049 / SIMD 32-trit lanes:** **not** specified under **`specs/`** in this repository.

---

## 5. Core constants (t27)

**Declared:** **`specs/math/constants.t27`** — at minimum `PHI`, `PHI_INV`, `PHI_SQ`, `PHI_INV_SQ`, `TRINITY`, `PI`, `E`, and CODATA-scale anchors as documented there.

**Not in-tree until added to a spec:** `MU`, `CHI`, `PHOENIX`, repunit **37**, golden-angle shortcuts, extended Fibonacci/Lucas tables, and **numerology-style** physics rows. Lucas **THM-002** is listed in **`T27-UNIFIED-AXIOM-THEOREM-FORMAT-SYSTEM.md`**.

**Physics coincidence formulas** (fine-structure, mass ratios, etc.): treat as **`empirical_fit` / `conjecture`** per **`CLAIM_TIERS.md`** and **`RESEARCH_CLAIMS.md`**—never as **`exact_algebraic`** without proof modules.

---

## 6. WASM, C ABI, bindings, conformance counts

| Topic | t27 fact |
|-------|-----------|
| **`deploy/runtime/*.wasm`** | **Not** present in this tree (search before claiming). |
| **`libgoldenfloat` / `gf16.h`** | **Proposed** FFI surface; mirror **`specs/numeric/gf16.t27`** when implemented. |
| **Rust / Python / Go / Node / Gleam “✅”** | **No** binding crates/sources here—track in the repo that actually ships them. |
| **`conformance/vectors.json` (33 vectors)** | **No** such file; use **`conformance/gf16_vectors.json`** (10 `test_vectors` at last audit) and grow deliberately. |

---

## 7. FPGA table

Resource counts (LUT/FF/DSP) are **not** verified SSOT in this doc. If cited, they must come from a pinned synthesis log or **`specs/fpga/**`** benchmark artifact.

---

## 8. TODO → t27 issues (mapped)

1. **GF8/GF24/GF32 “missing”** — **False for specs**; **true** for parity of **generated/hand Zig** → follow **`NUMERIC-GF16-DEBT-INVENTORY.md`** / `tri gen`.  
2. **VSA conformance vectors** — add under **`conformance/`** + tests in **`specs/vsa/**`**.  
3. **HybridBigInt** — land **`specs/`** + tests or drop from product claims.  
4. **WASM** — add build outputs + paths or remove from docs.  
5. **Split sacred vs ML constants** — extend **`specs/math/constants.t27`** or separate module with **`claim_tier`**.  
6. **φ-distance / layout tables in non-English drafts** — **replace** with this file + **FORMAT-SPEC** to avoid silent drift.

---

**φ² + 1/φ² = 3 | TRINITY**
