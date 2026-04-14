# Trinity / t27 — research and engineering claims registry

**Maintainer / lead author:** Dmitrii Vasilev — [ORCID 0009-0008-4294-6159](https://orcid.org/0009-0008-4294-6159) (Trinity Project / Trinity Framework Publications).

**Status:** Living document — extend when semantics, physics overlays, papers, or Zenodo releases change.
**Goal:** Make Trinity / t27 **falsifiable**, **auditable**, and **honest** about what is proved vs fitted vs conjectural.

**Rule:** Every **strong** statement in README, papers, or marketing should appear here (with an ID) or be downgraded to informal narrative.

**See also:** **`docs/nona-03-manifest/CLAIM_TIERS.md`** (spec / physics tier policy), **`docs/nona-03-manifest/T27-UNIFIED-AXIOM-THEOREM-FORMAT-SYSTEM.md`** (catalog + FORMAT-SPEC), **`conformance/axiom_system.json`** (machine-readable seed).

---

## Status vocabularies

### A — Research / physics (epistemic)

Use these for **domain-science** rows (sections 2–5).

| Status | Meaning |
|--------|---------|
| `EXACT` | Mathematically exact identity from definitions. |
| `WITHIN_UNCERTAINTY` | Numeric agreement within **stated** experimental uncertainty (e.g. CODATA). |
| `EMPIRICAL_FIT` | Empirical formula; good accuracy; **not** a first-principles derivation. |
| `APPROXIMATION` | Approximation; deviation **materially larger** than the relevant experimental uncertainty. |
| `FALSIFIED_AS_EXACT` | Cannot be claimed as "exact" vs experiment; may remain an interesting approximation. |
| `CONJECTURAL` | Hypothesis; insufficient verification. |
| `UNTESTED` | Not yet checked quantitatively in-repo or in linked artifact. |

### B — Toolchain / repository (engineering)

Use these for **build, CI, and SSOT** rows (section 1).

| Status | Meaning |
|--------|---------|
| `proved` | Theorem or machine-checked proof in-repo. |
| `tested` | Automated test / conformance / CI fails if violated. |
| `empirical` | Observed in practice; not a formal proof. |
| `conjectural` | Open or partial. |
| `untested` | Not yet covered by tests. |
| `deprecated` | Superseded; history only. |
| `falsified` | Claim demonstrated false; kept for audit trail. |

---

## 1. Toolchain and repository (engineering)

| Claim | Status | Primary evidence | Repro artifact | How to falsify |
|-------|--------|------------------|----------------|----------------|
| `.t27` specs are SOOT for product math on the critical path | `tested` | `docs/T27-CONSTITUTION.md`, `bootstrap/build.rs` | `cargo build` in `bootstrap/`, `tri parse` | Duplicate formula in verdict script without spec migration. |
| Bootstrap compiler core matches `bootstrap/stage0/FROZEN_HASH` | `tested` | `FROZEN.md`, `build.rs` | `cargo build` | Change `compiler.rs` without M5 seal update → build fails. |
| Zig codegen emits headers marking generated code | `tested` | `t27c validate-gen-headers` | `make -C repro repro-language` | Strip header from `gen/zig/**` → command fails. |
| 34 conformance vectors validate as JSON with vectors | `tested` | `t27c validate-conformance`, `conformance/` | `tri validate-conformance` or `make -C repro repro-numerics` | Break vector → command fails. |
| 48 module seals match `tri seal <spec> --verify` | `tested` | `.trinity/seals/`, CI | `tri seal <spec> --verify` | Intentional seal drift → verify fails. |
| GoldenFloat GF16 is primary numeric format for new product work | `tested` | `docs/NUMERIC-STANDARD-001.md` | Specs under `specs/numeric/` | Product path violation. |
| GF16 roundtrip accuracy meets 0.001% error tolerance | `tested` | C-gf-003, `conformance/gf16_vectors.json` | `t27c validate-conformance` | Introduce format drift > 0.001%. |
| L5 IDENTITY φ² = φ + 1 holds in f64 with tolerance 1e-15 | `tested` | C-phi-001, `coq/Kernel/PhiFloat.v`, Ring 45 | `t27c validate-phi-identity` | Violate identity tolerance in `FORMAT-SPEC-001.json`. |
| Sacred / phi-linked physics constants as **exact** fundamental laws | `empirical` / `conjectural` | `specs/math/`, physics docs | Label each row in §2–3 | CODATA/NIST update falsifies "exact" wording. |
| Self-hosting / fixed-point compiler story | `tested` (partial) | `docs/SEED-RINGS.md`, `CANON.md` | `t27c suite` fixed-point phase | Full formal self-host proof not yet `proved`. |
| CLARA / AR pipeline soundness | `conjectural` | `specs/ar/`, conformance | AR vectors | Bounded proofs TBD. |
| Cross-backend bit-exact equivalence (Zig vs C vs Verilog) | `conjectural` | — | Ring 39 roadmap | Mismatch allowed today. |

---

## 2. Phi-structures in fundamental constants

**Source:** Vasilev & Pellis, 2026, *Polynomial vs Monomial phi-Structures in Fundamental Constants* — Zenodo [10.5281/zenodo.18950696](https://doi.org/10.5281/zenodo.18950696); concept DOI [10.5281/zenodo.18947017](https://doi.org/10.5281/zenodo.18947017).
The paper states explicitly that many relations are **empirical approximations**, not physical derivations from first principles.

| ID | Claim (short) | Domain | Status | Rationale | Artifacts |
|----|---------------|--------|--------|-----------|-----------|
| C-phi-001 | Trinity identity φ² + φ⁻² = 3 and interpretation tying to N_gen = 3 | Math / SM generations | `EXACT` (identity); `CONJECTURAL` (physics reading) | Identity follows from the definition of φ; reading as "explaining" three generations is speculative. | Paper; t27 specs; Coq proof (Ring 45). |
| C-phi-002 | Pellis formula for 1/α: 360²φ⁻² − 2φ⁻³ + 3φ⁻⁵ — ~0.09 ppb deviation vs reference; within CODATA 2022 uncertainty band | EM / α | `WITHIN_UNCERTAINTY`; `EMPIRICAL_FIT` | Paper: deviation vs stated reference within relative uncertainty; still empirical fit, not Lagrangian derivation. | Paper; high-precision scripts. |
| C-phi-003 | Trinity monomial for α_s(M_Z) ~48 ppm vs reference; inside experimental uncertainty | QCD | `EMPIRICAL_FIT` | Treated as empirical template, not derived from QCD Lagrangian. | Paper; `specs/math/**` sacred-formula specs. |
| C-phi-004 | Monomial for m_p/m_e ~19 ppm vs reference but **not** within relative CODATA uncertainty → not "exact" | Particle physics | `FALSIFIED_AS_EXACT`; `APPROXIMATION` | Paper: fails as an "exact" relation; may remain a numerical curiosity. | Paper; deviation tables. |
| C-phi-005 | ~16 Trinity monomials for many constants (mixing angles, EW masses, T_CMB, …) with deviations ≤ ~1000 ppm | Multi-domain | `EMPIRICAL_FIT` | Catalog of fits; some near uncertainty bands, some much coarser. | Paper; sacred-formula catalog. |

---

## 3. GoldenFloat and numeric representations

| ID | Claim | Domain | Status | Rationale | Artifacts | L4 Test Hook |
|----|-------|--------|--------|-----------|-----------|---------------|
| C-gf-001 | GoldenFloat GF16/GF32 meets stated effective accuracy vs bit width | Numerics / HW | `UNTESTED` | Needs differential testing vs IEEE fp16/fp32/bfloat16 and a high-precision reference. | `docs/NUMERIC-STANDARD-001.md` | `#168` |
| C-gf-002 | GF widths improve accuracy–energy trade-off on FPGA vs IEEE fp32 | HW / energy | `CONJECTURAL` | Needs published FPGA methodology and benchmarks. | `docs/NUMERIC-STANDARD-001.md` | `#171` |
| C-gf-003 | GF16 roundtrip accuracy meets 0.001% error tolerance for sacred constants | Numerics | `tested` | Conformance vectors pass; L5 IDENTITY validated. | `conformance/gf16_vectors.json` | `gf16_roundtrip_phi` |
| C-gf-004 | Sacred constants (PHI, PI, G, etc.) can be represented in GF16 with < 0.1% error | Numerics | `untested` | Need GF16 constant bank; currently in `f64` in `constants.t27`. | `specs/math/constants.t27` | `#168` |
| C-gf-005 | Attention RoPE/softmax maintains quality in GF16 vs f64 | ML / attention | `speculative` | Requires benchmark comparing perplexity/accuracy. | `specs/nn/attention.t27` | `#171` |
| C-gf-006 | VSA operations (dot, similarity) have acceptable error in GF16 | VSA / numerics | `speculative` | Requires stability tests vs binary VSA baselines. | `specs/vsa/ops.t27` | `#173` |
| C-gf-007 | AR composition logic correctness preserved in GF16 vs f32 | AR / numerics | `speculative` | Requires testing of composition operators. | `specs/ar/composition.t27` | `#176` |

---

## 4. Ternary LLM / Trinity hardware stack (Zenodo)

These Zenodo records describe **architectures and artifacts**, not theorems. Claims below should be tightened as independent benchmarks and papers appear.

| ID | Claim | Domain | Status | Rationale | Artifacts |
|----|-------|--------|--------|-----------|-----------|
| C-ternary-001 | FPGA autoregressive ternary LLM runs inference in balanced-ternary arithmetic | HW / ML | `EMPIRICAL_FIT` | Zenodo describes design/code; independent replication + benchmarks needed. | 10.5281/zenodo.18939352 |
| C-ternary-002 | Self-Evolving Ouroboros demonstrates a self-hosting / self-evolving cycle | Systems | `CONJECTURAL` | Need formal criteria and reproducible experiment logs. | 10.5281/zenodo.19020211 |
| C-ternary-003 | VSA balanced ternary + SIMD gives stable high-dimensional VSA ops | VSA / numerics | `EMPIRICAL_FIT` | Zenodo description; needs stability tests vs binary VSA baselines. | 10.5281/zenodo.19020213, 10.5281/zenodo.19227877 |
| C-ternary-004 | phi-RoPE improves quality/stability vs standard RoPE on binary models | ML / attention | `CONJECTURAL` | Need public perplexity / stability / spectral comparisons. | 10.5281/zenodo.19020215 |
| C-ternary-005 | Sparse ternary matmul wins FLOPs/W and/or latency on FPGA vs dense binary matmul | HW | `CONJECTURAL` | Need published measurement methodology. | 10.5281/zenodo.19020217 |

---

## 5. Meta-claims about the t27 language and ecosystem

| ID | Claim | Domain | Status | Rationale | Artifacts |
|----|-------|--------|--------|-----------|-----------|
| C-meta-001 | Trinity / t27 is a spec-first ternary stack; Zig/C/Verilog backends are generated from `.t27` | PL / compilers | `EMPIRICAL_FIT` | Repo layout + CI (gen headers, conformance) demonstrate discipline; full `LANGUAGE_SPEC.md` + backend contracts still incomplete. | This repo; `docs/LANGUAGE_SPEC.md`, `BACKEND_CONTRACT.md`. |
| C-meta-002 | Trinity / t27 is self-hosting / self-evolving | Systems | `CONJECTURAL` | Define terms precisely + reproducible pipeline; partial story in rings + Ouroboros Zenodo. | 10.5281/zenodo.19020211; `CANON.md`, `docs/SEED-RINGS.md`. |

---

## 6. Maintenance rules

1. Every new paper, Zenodo release, or major benchmark adds or updates rows with a stable **ID** (`C-phi-*`, `C-gf-*`, …).
2. When CODATA (or other reference data) updates, **re-evaluate** statuses; old reasoning stays in Git history.
3. Any claim that fails as "exact" against experiment must move to **`FALSIFIED_AS_EXACT`** or **`APPROXIMATION`**.
4. The point is **not** to "prove we are right" but to make Trinity / t27 **transparent and falsifiable**.

---

## 7. Adding a row (checklist)

1. One-sentence **claim**.
2. **Status** from § vocabularies (A or B).
3. **Evidence**: spec path, test name, paper DOI, or Zenodo record.
4. **Falsification**: what observation would count against you.
5. **L4 Test Hook**: test name or issue reference (from `NUMERIC-GF16-DEBT-INVENTORY.md`).

---

## 8. Trinity Framework Publications — DOI index

| DOI | Record (short) | Date |
|-----|----------------|------|
| [10.5281/zenodo.18947017](https://doi.org/10.5281/zenodo.18947017) | Concept DOI (all versions) | 2026-03-10 |
| [10.5281/zenodo.18950696](https://doi.org/10.5281/zenodo.18950696) | Latest Trinity Framework version | 2026-03-10 |
| [10.5281/zenodo.18939352](https://doi.org/10.5281/zenodo.18939352) | FPGA Autoregressive Ternary LLM | 2026-03-10 |
| [10.5281/zenodo.19020211](https://doi.org/10.5281/zenodo.19020211) | Self-Evolving Ouroboros | 2026-03-14 |
| [10.5281/zenodo.19020213](https://doi.org/10.5281/zenodo.19020213) | VSA Balanced Ternary + SIMD | 2026-03-14 |
| [10.5281/zenodo.19020215](https://doi.org/10.5281/zenodo.19020215) | phi-RoPE Attention | 2026-03-14 |
| [10.5281/zenodo.19020217](https://doi.org/10.5281/zenodo.19020217) | Sparse Ternary MatMul | 2026-03-14 |
| [10.5281/zenodo.19227877](https://doi.org/10.5281/zenodo.19227877) | VSA Operations for Ternary Computing | — |

---

*φ² + 1/φ² = 3 | TRINITY — claims without falsification criteria are not science.*
