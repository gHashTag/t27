# NUMERIC-GF16-DEBT-INVENTORY.md — Numeric Debt Sprint (Issue #167)

**Status:** Ring 47 P2 — Active
**Created:** 2026-04-07
**Purpose:** Tie every line item to `RESEARCH_CLAIMS.md` and L4 TESTABILITY
**Rule:** No unlabeled scientific claims; every debt item has a clear migration path

---

## Tag Legend

| Tag | Meaning |
|-----|---------|
| **`[REFERENCE]`** | Spec intentionally defines multiple formats; keep until family is collapsed by ADR |
| **`[DEBT-f64]`** | Uses IEEE `f64`; **should migrate** to GF16 (or explicit GF20/GF24) per product path |
| **`[DEBT-f32]`** | Uses IEEE `f32`; same as above |
| **`[BRIDGE]`** | Uses `gf16_encode_f32`/`gf16_decode_to_f32`; acceptable short-term, **remove decode to f32** on hot paths |

## Tier Legend (from RESEARCH_CLAIMS.md)

| Tier | Meaning |
|------|---------|
| `proven` | Theorem or machine-checked proof in-repo |
| `tested` | Automated test / conformance / CI fails if violated |
| `claimed` | Claim made without full proof/test coverage |
| `speculative` | Hypothesis; insufficient verification |

## L4 Test Hook Legend

| Hook | Meaning |
|------|---------|
| `N/A` | Not applicable (e.g., policy, reference formats) |
| `PENDING` | Test hook to be added in future ring |
| `<test_name>` | Existing test in `.t27` spec |
| `#NNN` | GitHub issue for test hook |

---

## 1. `specs/numeric/` — Format Definitions

| File | Tag | Claim ID | Tier | L4 Test Hook | Notes |
|------|-----|----------|------|--------------|-------|
| `specs/numeric/gf16.t27` | **CANON** | C-gf-003 | `tested` | `gf16_roundtrip_phi` | Primary format; φ constants validated |
| `specs/numeric/trinity_numeric_surface.t27` | **POLICY** | — | `speculative` | `N/A` | Declares GF raw words as preferred public interchange |
| `specs/numeric/gf4.t27` | `[REFERENCE]` | — | `tested` | `gf4_roundtrip` | Smallest GF; masks/sparsity only |
| `specs/numeric/gf8.t27` | `[REFERENCE]` | — | `tested` | `gf8_roundtrip` | Compression tier |
| `specs/numeric/gf12.t27` | `[REFERENCE]` | — | `tested` | `gf12_roundtrip` | Legacy width; GF16-primary for inference |
| `specs/numeric/gf20.t27` | `[REFERENCE]` | — | `tested` | `gf20_roundtrip` | Training/gradients — prefer over `f64` |
| `specs/numeric/gf24.t27` | `[REFERENCE]` | — | `tested` | `gf24_roundtrip` | High precision — preferred over `f64` |
| `specs/numeric/gf32.t27` | `[REFERENCE]` | — | `tested` | `gf32_roundtrip` | Same bit width as FP32 but φ-structured |
| `specs/numeric/goldenfloat_family.t27` | `[REFERENCE]` | — | `proven` | `N/A` | Registry of all widths |
| `specs/numeric/phi_ratio.t27` | `[REFERENCE]` | C-phi-001 | `proven` | `phi_identity_exact` | φ² = φ + 1 proven in Coq (Ring 45) |
| `specs/numeric/tf3.t27` | `[REFERENCE]` | — | `claimed` | `N/A` | Ternary float experiment — non-primary |

---

## 2. Core Math & Physics — **[DEBT-f64] — Phase 3 Blockers**

| File | Tag | Claim ID | Tier | L4 Test Hook | Phase 3 Blocker | Notes |
|------|-----|----------|------|--------------|-----------------|-------|
| `specs/math/constants.t27` | `[DEBT-f64]` | C-gf-004 | `untested` | `#168` | **#142 (radix economy)** | All sacred constants as **`f64`**; target: GF16-packed constants |
| `specs/math/sacred_physics.t27` | `[DEBT-f64]` | C-phi-003 | `untested` | `#169` | **#142** | Entire pipeline `f64` (gravity, Ω_Λ, tolerances) |
| `specs/math/e8_lie_algebra.t27` | `[DEBT-f64]` | — | `speculative` | `#170` | **#143 (K3 truth table)** | Eigenvalues, cosines, errors in **`f64`** |
| `specs/physics/su2_chern_simons.t27` | `[DEBT-f64]` | — | `speculative` | `PENDING` | **#143** | Coupling, quantum dimension, trig in **`f64`** |
| `specs/physics/sacred_verification.t27` | `[DEBT-f64]` | — | `speculative` | `PENDING` | **#143** | Verification structs and scalars **`f64`** |

**Phase 3 Impact:** The `f64` debt in `constants.t27` and `sacred_physics.t27` directly blocks #142 (radix economy proof) which requires exact rational representation of sacred constants.

---

## 3. Neural & VSA — **[DEBT-f64] / [DEBT-f32] — Phase 3 Blockers**

| File | Tag | Claim ID | Tier | L4 Test Hook | Phase 3 Blocker | Notes |
|------|-----|----------|------|--------------|-----------------|-------|
| `specs/nn/attention.t27` | `[DEBT-f64]` | C-gf-002 | `untested` | `#171` | **#143** | RoPE tables, buffers, softmax, sacred scale — all **`f64`** |
| `specs/nn/hslm.t27` | `[DEBT-f64]` | — | `speculative` | `#172` | **#143** | Activations, norms, caches, gradients in **`f64`** |
| `specs/vsa/ops.t27` | `[DEBT-f64]` | — | `speculative` | `#173` | — | Similarity/dot/norm return **`f64`** instead of GF16 |
| `specs/vsa/core.t27` | `[DEBT-f64]` | — | `speculative` | `#174` | — | Thresholds and best similarity in **`f64`** |

**Phase 3 Impact:** Attention and VSA debt blocks #143 (K3 truth table) which requires K3-compatible numeric representation.

---

## 4. AR / Composition — Mixed GF16 + IEEE Leakage

| File | Tag | Claim ID | Tier | L4 Test Hook | Notes |
|------|-----|----------|------|--------------|-------|
| `specs/ar/proof_trace.t27` | `[BRIDGE]` | — | `tested` | `proof_trace_gf16_mul` | Confidence GF16 OK; replace bridge ops |
| `specs/ar/restraint.t27` | `[BRIDGE]` | — | `tested` | `restraint_gf16_confidence` | Verify no hidden IEEE in helpers |
| `specs/ar/explainability.t27` | **`[DEBT-f32]`** | — | `speculative` | `#175` | `fact_confs` **`[MAX]f32`**, `conf_f` **`f32`** |
| `specs/ar/composition.t27` | **`[DEBT-f32]`** | — | `speculative` | `#176` | **Largest AR debt** — ML tensors, Bayesian, simulators **`f32`** |
| `specs/ar/datalog_engine.t27` | `[BRIDGE]` | — | `tested` | `datalog_gf16_confidence` | Mostly GF16; verify literals |
| `specs/ar/asp_solver.t27` | `[BRIDGE]` | — | `tested` | `asp_gf16_confidence` | GF16 confidence path |

---

## 5. Orchestration & Demos

| File | Tag | Claim ID | Tier | L4 Test Hook | Notes |
|------|-----|----------|------|--------------|-------|
| `specs/queen/lotus.t27` | `[DEBT-f64]` | — | `speculative` | `#177` | `system_health`, `confidence`, ratios **`f64`** |
| `specs/demos/jones_vsa_demo.t27` | `[DEBT-f64]` | — | `untested` | `#178` | `JonesSignature` **`f64`**, thresholds **`f64`** |
| `specs/demos/jones_topology_filter.t27` | `[DEBT-f64]` | — | `untested` | `#179` | Same + local **`abs(f64)`** |

---

## 6. Conformance / JSON

| File | Tag | Claim ID | Tier | L4 Test Hook | Notes |
|------|-----|----------|------|--------------|-------|
| `conformance/gf16_bench_results.json` | `[DEBT-REF]` | C-gf-002 | `tested` | `N/A` | References **`f32`**/BF16 — OK for benchmark narrative |
| `conformance/phi_ratio_vectors.json` | `[REFERENCE]` | — | `tested` | `N/A` | Tests all GF widths |
| `conformance/goldenfloat_family_vectors.json` | `[REFERENCE]` | — | `tested` | `N/A` | Family queries incl. GF32/GF8 |
| `conformance/math_constants.json` | `[DEBT-f64]` | — | `tested` | `N/A` | Text references **`f64`** floor invariant |
| `conformance/clara_spec_coverage.json` | — | — | `tested` | `N/A` | Lists coverage — not debt |

---

## 7. Off-Spec (Non-Compliant)

| Path | Tag | Claim ID | Tier | L4 Test Hook | Notes |
|------|-----|----------|------|--------------|-------|
| `conformance/kepler_newton_tests.py` | `[DEBT-extreme]` | — | `falsified` | `N/A` | **`mpmath`** — violates SSOT-MATH; quarantine from product |
| `research/tba/*.py` | `[DEBT-extreme]` | — | `speculative` | `N/A` | Floating research; quarantine from product path |

---

## 8. Summary Statistics

| Category | Approx. Files | Status |
|----------|---------------|--------|
| GF16-primary or GF16-confidence core | 6 | **Partial good** |
| Pure **`f64`** domain specs | 9 | **Major rewrite** |
| **`f32`** leakage | 2 | **Major rewrite** |
| Multi-width GF reference specs | 7 | **Keep** as `[REFERENCE]` |
| Off-spec non-compliant | 2 | **Quarantine** |

---

## 9. Phase 3 Blocker Analysis

### #142 — Radix Economy Proof
**Blocked by:**
- `specs/math/constants.t27` (C-gf-004) — sacred constants in `f64`
- `specs/math/sacred_physics.t27` (C-phi-003) — physics pipeline in `f64`

**Migration Path:** Create GF16-packed constant bank before radix economy proof.

### #143 — K3 Truth Table
**Blocked by:**
- `specs/math/e8_lie_algebra.t27` — eigenvalues in `f64`
- `specs/nn/attention.t27` — RoPE/softmax in `f64`
- `specs/vsa/ops.t27`, `vsa/core.t27` — operations in `f64`

**Migration Path:** Define K3-compatible numeric representation (trit+GF16) before truth table.

---

## 10. Recommended Rewrite Order

1. **`specs/math/constants.t27`** (C-gf-004) → GF16 constant bank + error bounds (unblocks #142)
2. **`specs/math/sacred_physics.t27`** (C-phi-003) → GF16 physics pipeline (unblocks #142)
3. **`specs/nn/attention.t27`** → GF16 tensors + documented promotion rules (unblocks #143)
4. **`specs/vsa/ops.t27` + `core.t27`** → dot/similarity in GF16 accumulator
5. **`specs/ar/composition.t27`** → replace **`f32`** with GF16 (or trit + GF16 logits)
6. **Physics stack** (`su2_chern_simons`, `sacred_verification`) → align with chosen format
7. **Queen Lotus** metrics → GF16 health/confidence encoding

---

## 11. Cross-Links

- `docs/NUMERIC-STANDARD-001.md` — primary format authority
- `docs/nona-03-manifest/RESEARCH_CLAIMS.md` — claim tiers
- `docs/nona-03-manifest/T27-MATH-PHYSICS-TEST-FRAMEWORK-SPEC.md` — L4 test framework
- `docs/T27-CONSTITUTION.md` — SSOT-MATH
- `docs/SOUL.md` — TDD-CONTRACT
- `docs/NOW.md` — Phase 2.6 tracking

---

## 12. New Claims to Add to RESEARCH_CLAIMS.md

| Claim ID | Claim | Status | Rationale |
|----------|-------|--------|-----------|
| C-gf-003 | GF16 roundtrip accuracy meets 0.001% error tolerance | `tested` | Conformance vectors pass |
| C-gf-004 | Sacred constants (PHI, PI, G, etc.) can be represented in GF16 with < 0.1% error | `untested` | Need GF16 constant bank |
| C-gf-005 | Attention RoPE/softmax maintains quality in GF16 vs f64 | `speculative` | Requires benchmark |
| C-gf-006 | VSA operations (dot, similarity) have acceptable error in GF16 | `speculative` | Requires benchmark |
| C-gf-007 | AR composition logic correctness preserved in GF16 vs f32 | `speculative` | Requires testing |

---

**φ² + 1/φ² = 3 | TRINITY**
