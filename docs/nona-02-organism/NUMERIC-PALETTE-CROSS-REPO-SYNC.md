# NUMERIC — Cross-repo palette notes (t27 vs sibling claims)

**Purpose:** Record “full palette” items that appear when **two repositories** are read together. This file **does not** cite unrelated web URLs as evidence. Each item states what is **in-tree** in **t27** vs **outside this clone**.

**English SSOT registry (corrects common draft errors):** [`NUMERIC-CORE-PALETTE-REGISTRY.md`](NUMERIC-CORE-PALETTE-REGISTRY.md).

**Companions:** [`NUMERIC-GOLDENFLOAT-PALETTE.md`](NUMERIC-GOLDENFLOAT-PALETTE.md), [`CLAIM_TIERS.md`](../nona-03-manifest/CLAIM_TIERS.md), [`T27-UNIFIED-AXIOM-THEOREM-FORMAT-SYSTEM.md`](../nona-03-manifest/T27-UNIFIED-AXIOM-THEOREM-FORMAT-SYSTEM.md).

---

## 1. HybridBigInt (SIMD ternary, 3¹⁰, “32 trits per cycle”)

**t27:** No `HybridBigInt`, `59049`, or `3^10` width appears under **`specs/`** or **`docs/`** in searches.

**Status:** Treat as **sibling-repo or roadmap** content until a spec path (e.g. `specs/base/` or `specs/nn/`) lands with tests/invariants.

---

## 2. PackedTrit VSA — “12 000 trits = 2 400 bytes, 5:1 base-3”

**t27 SSOT:** **`specs/base/types.t27`** defines **`PackedTrit = u8`** packing **eight** trits per byte (`pack_trit` / `unpack_trit` for positions 0–7). **`TernaryWord`** uses **`WORD_BYTES = 5`** for **27** trits (ceil(27/8) style packing narrative in comments).

**Gap vs narrative:** A **5 trits → 1 byte** scheme (log₂(3⁵) < 8) is **not** the same as the **8-trits-in-u8** layout above. If the other repo documents 5:1 and 12 000-trit vectors, that is a **separate encoding** — merge only with an ADR and updated **`types.t27`** or a new module.

**VSA:** Operations live in **`specs/vsa/ops.t27`**; hypervector **sizes** must be stated there (or in nn specs) before citing fixed byte counts.

---

## 3. WASM modules (`phi_core`, `trit_wasm`, …)

**t27:** No checked-in **`.wasm`** / wasm build targets were found at repo root patterns.

**Status:** Plausible as **codegen output** or a **sibling build**; track under **`tri gen`** / backend issues. List concrete paths when artifacts exist.

---

## 4. Master formula \(V = n \times 3^k \times \pi^m \times \varphi^p \times e^q\) (and extensions)

**t27:** **`specs/physics/zamolodchikov_4d_conjecture.t27`** records the **Sacred Formula** as  
`V = n * 3^k * pi^m * phi^p * e^q * gamma^r` in a **prediction / speculative mapping** block (BPS spectrum narrative).

**Epistemics:** Classify with **`claim_tier`** per [`CLAIM_TIERS.md`](../nona-03-manifest/CLAIM_TIERS.md) — this is **not** `exact_algebraic` without a proof module; default posture is **`conjecture`** / **`empirical_fit`** until tightened.

---

## 5. Lucas L(10) = 123 and φ¹⁰ + φ⁻¹⁰

**Algebra (standard):** For \(\varphi = (1+\sqrt5)/2\), Lucas numbers satisfy \(L_n = \varphi^n + (-\varphi)^{-n} = \varphi^n + (-1)^n \varphi^{-n}\). For **even** \(n\), \(L_n = \varphi^n + \varphi^{-n}\). Hence **\(L_{10} = \varphi^{10} + \varphi^{-10} = 123\)** (integer).

**t27:** [`T27-UNIFIED-AXIOM-THEOREM-FORMAT-SYSTEM.md`](../nona-03-manifest/T27-UNIFIED-AXIOM-THEOREM-FORMAT-SYSTEM.md) lists **THM-002** (Lucas / φ identity). A named `LAMBDA_10` constant is **not** required in product code until added to **`specs/math/constants.t27`** with tests.

---

## 6. “37 × 3n = nnn” (repdigit pattern)

**t27:** No normative reference to **37** or that pattern under **`specs/*.t27`** in current search.

**Status:** **Out of tree** unless added to math specs with **`exact_algebraic`** proof sketch (modular arithmetic) and **`claim_tier`**.

---

## 7. GF8 / GF24 / GF32 — “specified but not in Zig”

**Correction for t27:** Widths **are** specified as **`.t27`** modules:

- `specs/numeric/gf8.t27`
- `specs/numeric/gf24.t27`
- `specs/numeric/gf32.t27`

**Gap:** **Hand-written Zig** parity or **generated** Zig from **`tri gen`** may still lag; that is **codegen/debt**, not missing specs. See [`NUMERIC-GF16-DEBT-INVENTORY.md`](NUMERIC-GF16-DEBT-INVENTORY.md) and root **`SOUL.md`** (no new handwritten domain Zig).

---

## 8. Physical constants — “empirical” epistemic status

**t27 policy:** [`CLAIM_TIERS.md`](../nona-03-manifest/CLAIM_TIERS.md) already distinguishes **`exact_experimental`**, **`empirical_fit`**, **`conjecture`**, etc. CODATA-scale anchors in **`specs/math/constants.t27`** are **measured** quantities, not theorems.

**Wording:** Prefer **tier labels** over informal “empirical” alone.

---

## Summary table

| Claim | In t27 now? | Next step |
|-------|-------------|-----------|
| HybridBigInt / 3¹⁰ SIMD | No | Spec + issue if product |
| 5:1 trit packing, 12 k trits | Not as SSOT | ADR vs `types.t27` 8-pack |
| WASM phi/trit modules | No artifacts | Wire build outputs + paths |
| Sacred Formula V = … | Yes (Zamolodchikov spec) | `claim_tier` + tests |
| L(10)=123 = φ¹⁰+φ⁻¹⁰ | Theorem row (THM-002) | Optional `constants.t27` |
| 37 / nnn | No | Prove + add or drop |
| GF8/24/32 | Yes (`.t27`) | Close Zig/gen gap |
| Empirical constants | Policy exists | Tag specs consistently |

---

**φ² + 1/φ² = 3 | TRINITY**
