# Wave Loop 122 Decomposed Plan
## Zero-Bench Regression Fix + Weakness Closure + Competitive Intel

**Trigger:** Canonical request to research weaknesses, scientific literature, decompose, implement, report, and propose three cooperation variants.

---

## Phase 1: OBSERVE

### Weaknesses Identified
1. **Zero-bench regression** — `specs/physics/quantum.t27` and `specs/fpga/verification/build_verify.t27` have **zero bench blocks**, violating the 100% bench coverage milestone achieved in W119.
2. **Lowest-tested RACE specs** — `yosys.t27` (4 tests), `formal.t27` (5 tests), `systolic_array.t27` (5 tests), `adder_tree.t27` (5 tests) — all below 8-test threshold.
3. **Lowest-tested CODER specs** — `bench_proxy.t27` (12 tests), `training.t27` (12 tests) — both below 16-test threshold for critical CODER files.
4. **compiler/optimizer.t27** contains a single `// TODO` comment that has persisted since W52.
5. **Five new competitors untracked** — Ternary Mamba (June 2026), MPX (June 2026), SPARQLe (May 2026), ProtoLang (June 2026), SparDA (June 2026).

### GitHub Issues
- 5 open issues (#1037–#1041), all IGLA-Coder phi-loop roadmap.
- #1038 still OPEN despite W119 commit claiming closure (likely remote sync lag).

### Competitive Intelligence
- **Ternary Mamba** (arXiv:2606.18114v1, HIGH) — ternary BitNet-style SSM accelerator; 3.61× compression.
- **MPX** (arXiv:2606.16394, MEDIUM-HIGH) — dual-mode systolic array for GEMM + polynomial multiplication.
- **SPARQLe** (arXiv:2606.00365, MEDIUM) — sub-precision activation representation for quantized LLMs.
- **ProtoLang** (arXiv:2606.13659, MEDIUM) — DSL for hardware communication protocols with dynamic symbolic execution.
- **SparDA** (arXiv:2606.04511v1, LOW-MEDIUM) — sparse decoupled attention with KV-block prefetching.

---

## Phase 2: PLAN (6 Tracks)

### Track A — Zero-Bench Regression Fix (P0)
- Add bench block to `quantum.t27`
- Add bench block to `build_verify.t27`
- Verify: `find specs/ -name '*.t27' | xargs grep -L 'bench '` returns empty

### Track B — IGLA RACE Weakness Expansion
- Add 4 tests to `yosys.t27` (detect_toolchain, generate_equiv_script, emit_sva, aggregate_coverage)
- Add 3 tests to `formal.t27` (prove_equivalence edge cases, compute_coverage boundaries)
- Add 3 tests to `systolic_array.t27` (booth_mul overflow, systolic_step accumulation, gemm negative inputs)
- Add 3 tests to `adder_tree.t27` (negative inputs, mixed signs, large values)

### Track C — IGLA CODER Weakness Expansion
- Add 4 tests to `bench_proxy.t27` (count_passed, empty list, all pass, all fail)
- Add 4 tests to `training.t27` (opd_distill, neg_log_approx, train_step, gradient clip)

### Track D — Competitive Intelligence Expansion
- Add `ternary_mamba_competitor()`, `mpx_competitor()`, `sparqle_competitor()`, `protolang_competitor()`, `sparda_competitor()` to `benchmark.t27`
- Add 5 tests for name validation
- Update `docs/COMPETITIVE_POSITIONING.md` with W122 appendix

### Track E — Seal Integrity & Suite Verification
- Regenerate seals for all modified specs
- Run `./target/release/t27c suite --repo-root .`
- Target: 564/564 PASS, 0 seal mismatches

### Track F — GitHub Issue Triage
- Assess #1038 status and attempt closure if commit evidence exists
- Verify no stale issues reopened

---

## Estimated Impact
- +2 bench blocks (regression fix)
- +17 tests across 6 files
- +5 competitors tracked (total 120)
- 100% bench coverage restored
- Suite: 564/564 PASS

φ² + 1/φ² = 3 | TRINITY
