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
| `FALSIFIED_AS_EXACT` | Cannot be claimed as “exact” vs experiment; may remain an interesting approximation. |
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
| `deprecated` | Superseded; history only. |

---

## 1. Toolchain and repository (engineering)

| Claim | Status | Primary evidence | Repro artifact | How to falsify |
|-------|--------|------------------|----------------|----------------|
| `.t27` specs are SOOT for product math on the critical path | `tested` | `docs/T27-CONSTITUTION.md`, `bootstrap/build.rs` | `cargo build` in `bootstrap/`, `tri parse` | Duplicate formula in verdict script without spec migration. |
| Bootstrap compiler core matches `bootstrap/stage0/FROZEN_HASH` | `tested` | `FROZEN.md`, `build.rs` | `cargo build` | Change `compiler.rs` without M5 seal update → build fails. |
| Zig codegen emits headers marking generated code | `tested` | `t27c validate-gen-headers` | `make -C repro repro-language` | Strip header from `gen/zig/**` → command fails. |
| 34 conformance vectors validate as JSON with vectors | `tested` | `t27c validate-conformance`, `conformance/` | `tri validate-conformance` or `make -C repro repro-numerics` | Break vector → command fails. |
| 48 module seals match `tri seal <spec> --verify` | `tested` | `.trinity/seals/`, CI | `tri seal <spec> --verify` | Intentional seal drift → verify fails. |
| GoldenFloat GF16 is primary numeric format for new product work | `conjectural` (policy) | `docs/nona-02-organism/NUMERIC-STANDARD-001.md` | Specs under `specs/numeric/` | Tracked in `docs/nona-02-organism/NUMERIC-GF16-DEBT-INVENTORY.md`. |
| Sacred / phi-linked physics constants as **exact** fundamental laws | `empirical` / `conjectural` | `specs/math/`, physics docs | Label each row in §2–3 | CODATA/NIST update falsifies “exact” wording. |
| Self-hosting / fixed-point compiler story | `tested` (partial) | `docs/nona-01-foundation/SEED-RINGS.md`, `CANON.md` | `t27c suite` fixed-point phase | Full formal self-host proof not yet `proved` — `docs/STATE_OF_THE_PROJECT.md`. |
| CLARA / AR pipeline soundness | `conjectural` | `specs/ar/`, conformance | AR vectors | Bounded proofs TBD. |
| Cross-backend bit-exact equivalence (Zig vs C vs Verilog) | `conjectural` | — | Ring 39 roadmap | Mismatch allowed today. |

---

## 2. Phi-structures in fundamental constants

**Source:** Vasilev & Pellis, 2026, *Polynomial vs Monomial phi-Structures in Fundamental Constants* — Zenodo [10.5281/zenodo.18950696](https://doi.org/10.5281/zenodo.18950696); concept DOI [10.5281/zenodo.18947017](https://doi.org/10.5281/zenodo.18947017).  
The paper states explicitly that many relations are **empirical approximations**, not physical derivations from first principles.

| ID | Claim (short) | Domain | Status | Rationale | Artifacts |
|----|---------------|--------|--------|-----------|-----------|
| C-phi-001 | Trinity identity φ² + φ⁻² = 3 and interpretation tying to N_gen = 3 | Math / SM generations | `EXACT` (identity); `CONJECTURAL` (physics reading) | Identity follows from the definition of φ; reading as “explaining” three generations is speculative. | Paper; t27 specs (Trinity identity). |
| C-phi-002 | Pellis formula for 1/α: 360²φ⁻² − 2φ⁻³ + 3φ⁻⁵ — ~0.09 ppb deviation vs reference; within CODATA 2022 uncertainty band | EM / α | `WITHIN_UNCERTAINTY`; `EMPIRICAL_FIT` | Paper: deviation vs stated reference within relative uncertainty; still empirical fit, not Lagrangian derivation. | Paper; high-precision scripts (see paper / Zenodo bundle — migrate into repo repro when pinned). |
| C-phi-003 | Trinity monomial for α_s(M_Z) ~48 ppm vs reference; inside experimental uncertainty | QCD | `EMPIRICAL_FIT` | Treated as empirical template, not derived from QCD Lagrangian. | Paper; `specs/math/**` sacred-formula specs. |
| C-phi-004 | Monomial for m_p/m_e ~19 ppm vs reference but **not** within relative CODATA uncertainty → not “exact” | Particle physics | `FALSIFIED_AS_EXACT`; `APPROXIMATION` | Paper: fails as an “exact” relation; may remain a numerical curiosity. | Paper; deviation tables. |
| C-phi-005 | ~16 Trinity monomials for many constants (mixing angles, EW masses, T_CMB, …) with deviations ≤ ~1000 ppm | Multi-domain | `EMPIRICAL_FIT` | Catalog of fits; some near uncertainty bands, some much coarser. | Paper; sacred-formula catalog. |

---

## 3. GoldenFloat and numeric representations

*Placeholder — extend when differential tests and Zenodo/crate artifacts are pinned.*

| ID | Claim | Domain | Status | Rationale | Artifacts |
|----|-------|--------|--------|-----------|-----------|
| C-gf-001 | GoldenFloat GF16/GF32 meets stated effective accuracy vs bit width | Numerics / HW | `UNTESTED` | Needs differential testing vs IEEE fp16/fp32/bfloat16 and a high-precision reference (e.g. Python `decimal`). | `docs/NUMERICS_VALIDATION.md` §§4–7; Zenodo bundle TBD. |
| C-gf-002 | GF widths improve accuracy–energy trade-off on FPGA vs IEEE fp32 | HW / energy | `CONJECTURAL` | Needs published FPGA methodology and benchmarks. | `docs/NUMERICS_VALIDATION.md` §8 |

---

## 4. Ternary LLM / Trinity hardware stack (Zenodo)

These Zenodo records describe **architectures and artifacts**, not theorems. Claims below should be tightened as independent benchmarks and papers appear.

**Related DOIs:** [10.5281/zenodo.18939352](https://doi.org/10.5281/zenodo.18939352) (FPGA autoregressive ternary LLM), [10.5281/zenodo.19020270](https://doi.org/10.5281/zenodo.19020270) (Ouroboros), [10.5281/zenodo.19020275](https://doi.org/10.5281/zenodo.19020275) (VSA + SIMD), [10.5281/zenodo.19020280](https://doi.org/10.5281/zenodo.19020280) (phi-RoPE), [10.5281/zenodo.19020282](https://doi.org/10.5281/zenodo.19020282) (sparse ternary matmul), [10.5281/zenodo.19227877](https://doi.org/10.5281/zenodo.19227877) (VSA ops); concept [10.5281/zenodo.18947017](https://doi.org/10.5281/zenodo.18947017).

| ID | Claim | Domain | Status | Rationale | Artifacts |
|----|-------|--------|--------|-----------|-----------|
| C-ternary-001 | FPGA autoregressive ternary LLM runs inference in balanced-ternary arithmetic | HW / ML | `EMPIRICAL_FIT` | Zenodo describes design/code; independent replication + benchmarks needed. | 10.5281/zenodo.18939352 |
| C-ternary-002 | Self-Evolving Ouroboros demonstrates a self-hosting / self-evolving cycle | Systems | `CONJECTURAL` | Need formal criteria and reproducible experiment logs. | 10.5281/zenodo.19020270 |
| C-ternary-003 | VSA balanced ternary + SIMD gives stable high-dimensional VSA ops | VSA / numerics | `EMPIRICAL_FIT` | Zenodo description; needs stability tests vs binary VSA baselines. | 10.5281/zenodo.19020275, 10.5281/zenodo.19227877 |
| C-ternary-004 | phi-RoPE improves quality/stability vs standard RoPE on binary models | ML / attention | `CONJECTURAL` | Need public perplexity / stability / spectral comparisons. | 10.5281/zenodo.19020280 |
| C-ternary-005 | Sparse ternary matmul wins FLOPs/W and/or latency on FPGA vs dense binary matmul | HW | `CONJECTURAL` | Need published measurement methodology. | 10.5281/zenodo.19020282 |

---

## 5. Meta-claims about the t27 language and ecosystem

| ID | Claim | Domain | Status | Rationale | Artifacts |
|----|-------|--------|--------|-----------|-----------|
| C-meta-001 | Trinity / t27 is a spec-first ternary stack; Zig/C/Verilog backends are generated from `.t27` | PL / compilers | `EMPIRICAL_FIT` | Repo layout + CI (gen headers, conformance) demonstrate discipline; full `docs/nona-02-organism/LANGUAGE_SPEC.md` + backend contracts still incomplete. | This repo; `docs/nona-02-organism/LANGUAGE_SPEC.md`, `docs/BACKEND_CONTRACT.md`. |
| C-meta-002 | Trinity / t27 is self-hosting / self-evolving | Systems | `CONJECTURAL` | Define terms precisely + reproducible pipeline; partial story in rings + Ouroboros Zenodo. | 10.5281/zenodo.19020270; `CANON.md`, `docs/nona-01-foundation/SEED-RINGS.md`. |

---

## 5a. ML / IGLA optimizer (phi as a falsifiable design prior, Issue #181)

This section tracks the phi->IGLA epic
([gHashTag/trios-trainer-igla#181](https://github.com/gHashTag/trios-trainer-igla/issues/181)).
The governing principle: phi is a **design prior**, not a proven theory --
"the method survives, phi does not (yet)." Only the algebraic identity
phi^2 + phi^-2 = 3 is `EXACT`; every phi performance claim is `CONJECTURAL`
with an explicit falsification path.

Numeric note: weight_decay = phi^-3 ~ 0.2360679774997897 (= 1/(2*phi+1)). A
stale trainer comment reads "~0.118" (= phi^-3/2) and is wrong.

| ID | Claim | Domain | Status | Falsification / rationale | Artifacts |
|----|-------|--------|--------|---------------------------|-----------|
| FL-001 | phi-canonical AdamW anchors (beta1=phi^-1 ~ 0.618034, weight_decay=phi^-3 ~ 0.236068, lr=phi^-3) produce lower BPB than equal-budget tuned AdamW(0.9, 0.01) on the IGLA RACE task (hidden=828, ~81K steps, seed=43; champion BPB=2.1919, sub-Chinchilla / preliminary) | ML / optimizer | `CONJECTURAL` ([Open conjecture]) | FALSIFIED if a non-phi control of equal tuning budget -- tuned-standard AdamW(0.9,0.01) OR random-rational of equal cost -- matches phi-canonical within Monte-Carlo error across N>=5 seeds. Secondary: phi-damped (0.9/phi, 0.999/phi) also matching removes any "phi-family" advantage. Identity phi^2+phi^-2=3 is `EXACT` (L5) and is unaffected by any such falsification. | `trios-trainer-igla src/optimizer.rs`; `t27 specs/ml/optimizer/adamw.t27`, `specs/ml/optimizer/race_config.t27`, `specs/ml/igla_champion_capsule.t27`; Issue #181 |
| FL-002 | QK-Gain = phi^2 ~ 2.618 (INV-9) gives better attention quality than gain=1.0 or a learned scalar gain | ML / attention | `CONJECTURAL` ([Open conjecture]) | FALSIFIED if gain=1.0 or learned gain matches gain=phi^2 within MC error at equal budget. phi^2 = phi + 1 is `EXACT` arithmetic, but its optimality as an attention gain has no theoretical basis. | `t27 specs/nn/attention.t27` (AttentionQKGainAblation); Issue #181 |
| FL-003 | GFTernary weights {-phi, 0, +phi} beat BitNet b1.58 integer ternary {-1, 0, +1} at equal compute | Numerics / ML | `CONJECTURAL` ([Open conjecture]) | FALSIFIED if integer-ternary matches phi-ternary BPB within MC error. Baseline: Ma et al. 2024, arXiv:2402.17764. | `t27 specs/numeric/gfternary.t27`; Issue #181 |
| FL-004 | The GoldenFloat width ladder (one closed rule e=round((N-1)/phi^2) across GF4..GF256, integer-backed via the Lucas-exact accumulator phi^(2n)+phi^(-2n)=L_(2n)) gives a defensible BREADTH / toolchain-coherence advantage -- fewer lossy cross-format conversions at matched bit budget -- over an equally-tuned posit / takum / OCP-MX ladder. The advantage claimed is breadth, NOT per-rung accuracy and NOT base-uniqueness. | Numerics | `CONJECTURAL` ([Open conjecture]) | Sub-leg F1 (the arithmetic) is `EXACT`: phi^(2n)+phi^(-2n)=L_(2n) integer Lucas, verified to 60 digits (max residual 7.1e-56), so phi-scaled accumulation is integer-exact. The MOAT is FALSIFIED if (a) a posit/takum/MX ladder matches breadth (lossy-conversion count) AND per-rung accuracy at matched bit budgets (F2/F3), OR (b) a prior width-spanning float family is shown to derive its E:M split from one closed rule across a 2-256-bit ladder with comparable integer-backed coherence. takum (Hunhold 2024, arXiv:2404.18603) is the closest live counterexample; the posit ladder is the F3 control, encoded with BOTH sourced es schedules -- pre-standard es=0/1/2/3/4 at 8/16/32/64/128 (de Dinechin et al. 2019) and the ratified-2022 fixed es=2 for all widths (Posit Standard 2022, posithub.org). F1 verifies the arithmetic, NEVER the moat. Anchor: Ahlbach/Usatine/Pippenger 2012, arXiv:1207.4497. | `t27 specs/numeric/lucas_accumulator.t27`, `specs/numeric/posit_ladder_control.t27`, `specs/numeric/goldenfloat_family.t27`, `specs/numeric/gf_competitive.t27`; epic in gHashTag/trios-trainer-igla#181 |

**ASHA / Hyperband note (relabel).** The RACE scheduler uses geometric rungs
with factor eta=3 (1k -> 3k -> 9k -> 27k). eta=3 is the **standard field
default** (Li et al. 2018, Hyperband, arXiv:1603.06560), `EMPIRICAL_FIT`. The
numerical coincidence eta=3 == phi^2 + phi^-2 is noted but does **NOT**
constitute a phi derivation. Label: field default, not a phi fact.

**Retraction (2026-05-31).** delta_CP = 3/phi^2 is withdrawn as a physics
claim (`FALSIFIED_AS_EXACT`; no Standard Model derivation; ~65.7 deg vs PMNS
~195 deg). It must not be cited as evidence in any t27 first-party document or
spec. (Note: `specs/physics/formula_discovery.t27` uses a distinct expression
9*phi^-2, which is a separate empirical fit, not this retracted claim.)

---

## 6. Maintenance rules

1. Every new paper, Zenodo release, or major benchmark adds or updates rows with a stable **ID** (`C-phi-*`, `C-gf-*`, …).  
2. When CODATA (or other reference data) updates, **re-evaluate** statuses; old reasoning stays in Git history.  
3. Any claim that fails as “exact” against experiment must move to **`FALSIFIED_AS_EXACT`** or **`APPROXIMATION`**.  
4. The point is **not** to “prove we are right” but to make Trinity / t27 **transparent and falsifiable**.

---

## 7. Adding a row (checklist)

1. One-sentence **claim**.  
2. **Status** from § vocabularies (A or B).  
3. **Evidence**: spec path, test name, paper DOI, or Zenodo record.  
4. **Falsification**: what observation would count against you.

---

## 8. Trinity Framework Publications — DOI index

| DOI | Record (short) | Date |
|-----|----------------|------|
| [10.5281/zenodo.18947017](https://doi.org/10.5281/zenodo.18947017) | Concept DOI (all versions) | 2026-03-10 |
| [10.5281/zenodo.18950696](https://doi.org/10.5281/zenodo.18950696) | Latest Trinity Framework version | 2026-03-10 |
| [10.5281/zenodo.18939352](https://doi.org/10.5281/zenodo.18939352) | FPGA Autoregressive Ternary LLM | 2026-03-10 |
| [10.5281/zenodo.19020270](https://doi.org/10.5281/zenodo.19020270) | Self-Evolving Ouroboros | 2026-03-14 |
| [10.5281/zenodo.19020275](https://doi.org/10.5281/zenodo.19020275) | VSA Balanced Ternary + SIMD | 2026-03-14 |
| [10.5281/zenodo.19020280](https://doi.org/10.5281/zenodo.19020280) | phi-RoPE Attention | 2026-03-14 |
| [10.5281/zenodo.19020282](https://doi.org/10.5281/zenodo.19020282) | Sparse Ternary MatMul | 2026-03-14 |
| [10.5281/zenodo.19227877](https://doi.org/10.5281/zenodo.19227877) | VSA Operations for Ternary Computing | — |

---

*φ² + 1/φ² = 3 | TRINITY — claims without falsification criteria are not science.*
