# NUMERIC-GF16-DEBT — Non-GF16 numeric inventory (rewrite targets)

**Status:** Active audit list  
**Canon:** `docs/nona-02-organism/NUMERIC-STANDARD-001.md` — **GF16 is PRIMARY** for inference; other GoldenFloat widths are **family members**, not substitutes for “IEEE f32/f64 everywhere.”  
**Tag legend:**
- **`[REFERENCE]`** — Spec intentionally defines multiple formats; keep until family is collapsed by ADR.
- **`[DEBT-f64]`** — Uses IEEE `f64` (or pervasive `f64` math); **should migrate** to GF16 (or explicit GF20/GF24 only where standard allows) per product path.
- **`[DEBT-f32]`** — Uses IEEE `f32`; same as above.
- **`[BRIDGE]`** — Uses `gf16_encode_f32` / `gf16_decode_to_f32`; acceptable short-term, **remove decode to f32** on hot paths when pure GF16 ops exist.

---

## 1. `specs/numeric/` — format definitions

| File | Tag | Notes |
|------|-----|-------|
| `specs/numeric/gf16.t27` | **CANON** | Primary format; target state for product numerics. |
| `specs/numeric/gf4.t27` | `[REFERENCE]` | Smallest GF; only for masks/sparsity stories — not default compute. |
| `specs/numeric/gf8.t27` | `[REFERENCE]` | Compression tier. |
| `specs/numeric/gf12.t27` | `[REFERENCE]` | Was “attention, embeddings” in table — **conflicts** with GF16-primary; treat as **legacy width** unless ADR demotes GF12 from hot path. |
| `specs/numeric/gf20.t27` | `[REFERENCE]` | Training/gradients — if training stays in repo, prefer **GF20 path** over `f64`, not parallel IEEE. |
| `specs/numeric/gf24.t27` | `[REFERENCE]` | High precision GoldenFloat — preferred over `f64` where range allows. |
| `specs/numeric/gf32.t27` | `[REFERENCE]` | Same bit width as FP32 but φ-structured; **still not “use f64 in specs.”** |
| `specs/numeric/goldenfloat_family.t27` | `[REFERENCE]` | Registry of all widths. |
| `specs/numeric/phi_ratio.t27` | `[REFERENCE]` | Derivation helper for bit splits. |
| `specs/numeric/tf3.t27` | `[REFERENCE]` | Ternary float experiment — not GF16; mark **non-primary** for inference. |

**Agent rule:** do **not** add **new** `f32`/`f64` fields in **nn/**, **vsa/**, **math/** when GF16 (or allowed GF20/24) can carry the quantity.

---

## 2. Core math & physics — heavy `[DEBT-f64]`

| File | Tag | What is wrong |
|------|-----|----------------|
| `specs/math/constants.t27` | `[DEBT-f64]` | All sacred constants (`PHI`, `PI`, `G_MEASURED`, scales) as **`f64`** + `pow`/`ln`/`exp` approximations in **`f64`**. **Rewrite target:** GF16-packed constants + promoted arithmetic, or fixed-point spec. |
| `specs/math/sacred_physics.t27` | `[DEBT-f64]` | Entire pipeline `f64` (gravity, Ω_Λ, tolerances, structs). |
| `specs/math/e8_lie_algebra.t27` | `[DEBT-f64]` | Eigenvalues, cosines, errors in **`f64`**. |
| `specs/physics/su2_chern_simons.t27` | `[DEBT-f64]` | Coupling, quantum dimension, Jones helper, trig in **`f64`**. |
| `specs/physics/sacred_verification.t27` | `[DEBT-f64]` | Formula verification structs and scalars **`f64`**. |

---

## 3. Neural & VSA — `[DEBT-f64]` / `[DEBT-f32]`

| File | Tag | What is wrong |
|------|-----|----------------|
| `specs/nn/attention.t27` | `[DEBT-f64]` | RoPE tables, buffers, softmax path, sacred scale — all **`f64`**. **High priority** vs NUMERIC-STANDARD (GF16 primary for inference). |
| `specs/nn/hslm.t27` | `[DEBT-f64]` | Activations, norms, caches, gradients narrative in **`f64`**. |
| `specs/vsa/ops.t27` | `[DEBT-f64]` | Similarity/dot/norm return **`f64`** (IEEE) instead of GF16 accumulators. |
| `specs/vsa/core.t27` | `[DEBT-f64]` | Thresholds and best similarity in **`f64`**. |

---

## 4. AR / composition — mixed GF16 + IEEE leakage

| File | Tag | What is wrong |
|------|-----|----------------|
| `specs/ar/proof_trace.t27` | `[BRIDGE]` | Confidence **`GF16`** OK; **`gf16_decode_to_f32` / `gf16_encode_f32`** for multiply — replace with native GF16 mul when specified. |
| `specs/ar/restraint.t27` | `[BRIDGE]` | Same pattern + **`f32` comparison** comments. |
| `specs/ar/explainability.t27` | **`[DEBT-f32]` + `[BRIDGE]`** | `fact_confs` **`[MAX]f32`**, `conf_f` **`f32`**; rest GF16. |
| `specs/ar/composition.t27` | **`[DEBT-f32]` + `[BRIDGE]`** | ML tensors **`[]const f32`**, Bayesian **`f32`**, simulators **`f32`**, `f32_to_trit`, struct fields **`f32`** for probabilities/scores. **Largest AR debt.** |
| `specs/ar/datalog_engine.t27` | `[BRIDGE]` | Mostly GF16; **`gf16_encode_f32(1.0)`** literals. |
| `specs/ar/asp_solver.t27` | `[BRIDGE]` | GF16 confidence path; verify no hidden IEEE in helpers. |

---

## 5. Orchestration & demos

| File | Tag | What is wrong |
|------|-----|----------------|
| `specs/queen/lotus.t27` | `[DEBT-f64]` | `system_health`, `confidence`, ratios **`f64`**. |
| `specs/demos/jones_vsa_demo.t27` | `[DEBT-f64]` | `JonesSignature` **`f64`**, thresholds **`f64`**. |
| `specs/demos/jones_topology_filter.t27` | `[DEBT-f64]` | Same + local **`abs(f64)`**. |

---

## 6. Conformance / JSON (mentions non-GF16)

| File | Tag | Notes |
|------|-----|-------|
| `conformance/gf16_bench_results.json` | `[DEBT-REF]` | References **`f32`** / BF16 comparison — OK for benchmark narrative; **do not** use as excuse to spec **`f32`** in product modules. |
| `conformance/phi_ratio_vectors.json` | `[REFERENCE]` | Tests all GF widths — keep aligned with numeric specs. |
| `conformance/goldenfloat_family_vectors.json` | `[REFERENCE]` | Family queries incl. GF32/GF8. |
| `conformance/math_constants.json` | `[DEBT-f64]` | Text references **`f64`** floor invariant — tied to `constants.t27` debt. |
| `conformance/clara_spec_coverage.json` | — | Lists **`gf32.t27`** etc. as coverage — not debt by itself. |

---

## 7. Off-spec but stinky (IEEE / high-precision outside GF16)

| Path | Tag | Notes |
|------|-----|-------|
| `conformance/kepler_newton_tests.py` | `[DEBT-extreme]` | **`mpmath` / high-precision float** — violates SSOT-MATH; must become `.t27` + allowed numeric story. |
| `research/tba/*.py` | `[DEBT-extreme]` | Floating research; quarantine from product GF16 path. |

---

## 8. Clean vs dirty summary (counts)

| Category | Approx. spec files | Status |
|----------|-------------------|--------|
| GF16-primary or GF16-confidence core | `gf16.t27`, most of `restraint.t27`, parts of `proof_trace`, `datalog_engine`, `asp_solver` | **Partial good** |
| Pure **`f64`** domain specs | `constants`, `sacred_physics`, `e8`, `su2_chern_simons`, `sacred_verification`, `nn/*`, `vsa/*`, `queen/lotus`, demos | **Major rewrite** |
| **`f32`** leakage | `composition.t27`, `explainability.t27` | **Major rewrite** |
| Multi-width GF reference specs | `gf4`–`gf32`, `goldenfloat_family`, `phi_ratio`, `tf3` | **Keep** as `[REFERENCE]` until ADR collapses |

---

## 9. Recommended rewrite order (for agents)

1. **`specs/nn/attention.t27` + `hslm.t27`** → GF16 tensors + documented promotion rules (GF20 for acc if needed).  
2. **`specs/ar/composition.t27`** → replace **`f32`** feature/state tensors with GF16 (or trit + GF16 logits).  
3. **`specs/math/constants.t27` + `sacred_physics.t27`** → GF16 constant bank + error bounds spec.  
4. **Physics stack** (`su2_chern_simons`, `sacred_verification`, `e8`) → align with chosen promoted format (likely **GF24** or **GF32** per family, **not raw `f64`** if avoidable).  
5. **VSA ops** → dot/similarity in GF16 accumulator type.  
6. **Queen Lotus** metrics → GF16 health/confidence encoding.

---

## 10. Cross-links

- `docs/nona-02-organism/NUMERIC-STANDARD-001.md` — primary format authority.  
- `docs/nona-01-foundation/QUEEN-LOTUS-SEED-LANGUAGE-PURGE.md` — stop adding parallel Python/IEEE paths.  
- `docs/T27-CONSTITUTION.md` — SSOT-MATH.  
- `docs/nona-01-foundation/GOLDEN-RINGS-CANON.md` — **REFACTOR-HEAP** vs ring-sealed **GOLD** (this inventory is debt, not canon).

---

*phi^2 + 1/phi^2 = 3 | TRINITY*
