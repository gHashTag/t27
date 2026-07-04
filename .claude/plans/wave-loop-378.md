# Plan: Wave Loop 378 — IGLA CODER+RACE + `let` destructuring + full IGLA yosys smoke gate

**Date:** 2026-07-03 (planned from W377 close-out)  
**Issue target:** #1268 (created and tracked)  
**Branch:** `trinity-rust-rings`  
**Recommended variant:** Variant B from `docs/reports/WAVE_LOOP_377_COOPERATION.md`

---

## 1. Goal

Extend the 37-wave zero-IGLA-failure streak by pushing the Lean 4 generic ∀ proof lattice to **256**, land the last wave-safe `gen-verilog` sub-fix (**Defect 6: `let` destructuring**), and expand the in-runner CI smoke gate to cover **all 27 IGLA specs**. Keep the QMTech Wukong V1 / DLC10 bitstream path ready.

Target metrics:

| Metric | W377 | W378 | Δ |
|---|---|---|---|
| Lean 4 generic ∀ | 252 | **256** | +4 |
| Pool A floor | 121 | **122** | +1 |
| CODER minimum | 112 | **113** | +1 |
| Pool B depth | 139 | **140** | +1 |
| Integration depth | 120 | **121** | +1 |
| Tests | 13,083 | **13,138** | +55 |
| Invariants | 5,742 | **5,769** | +27 |
| Conformance specs | 557 | **558** | +1 (scratch) |
| Conformance pass rate | 557/557 | **558/558** | 100% |
| Gen-verilog yosys smoke targets | 36 | **38** | +2 (full IGLA) |
| Zero-IGLA-failure streak | 111 waves | **112 waves** | +1 |

---

## 2. Issue landscape

- **#1267** — Wave Loop 377 (closed).
- **#1268** — Wave Loop 378 (created for this work).
- **#1258** — `gen-verilog: incremental array/RAM lowering for datapath specs (fifo/memory)`. Still too broad for one wave.
- **#1265-#1253** — older wave issues; closed as waves land, some still open due to duplicate/tracking semantics.

---

## 3. Scientific / competitive landscape

Key 2025–2026 work on formal/ternary hardware:

1. **Sparkle HDL / Verilean** ([github.com/Verilean/sparkle](https://github.com/Verilean/sparkle)) — formally verifiable HDL in Lean 4 with BitNet b1.58 accelerator, **60+ theorems**. Strongest direct competitor.
2. **TorchLean** ([lean-dojo/TorchLean](https://github.com/lean-dojo/TorchLean), [arXiv:2602.22631](https://arxiv.org/abs/2602.22631)) — Lean 4 NN formalization with PyTorch interop; software focus.
3. **TerEffic** ([arXiv:2502.16473](https://arxiv.org/abs/2502.16473)) and **TeLLMe** ([arXiv:2504.16266](https://arxiv.org/abs/2504.16266)) — 2025 ternary LLM FPGA accelerators; simulation/test verification.
4. **KULeuven-MICAS/ternary-lut-dse** and **TernaryCore** — open ternary hardware, testbench/simulation verification.
5. **Trinity B002** (Zenodo 10.5281/zenodo.19224235) — 2026 defensive publication for zero-DSP ternary inference.

**Takeaway:** Sparkle HDL is the only credible formal competitor. W378 widens the generic ∀ gap from **252× to 256×** while closing the last tracked gen-verilog defect.

---

## 4. Decomposed work breakdown

### 4.1 IGLA spec batch (+55 tests, +27 invariants)

- Copy `scripts/gen_w377.py` → `scripts/gen_w378.py`.
- Update last-wave check from 377 → 378 and all `w377_` / `W377` placeholders to `w378_` / `W378`.
- Run over `specs/igla/coder/*.t27` and `specs/igla/race/*.t27`.
- Verify diff with `git diff --stat` and spot-check two specs.

### 4.2 Lean 4 proof-lattice extension (+4 generic ∀)

Copy `scripts/gen_w377_lean.py` → `scripts/gen_w378_lean.py`, then append:

1. `ternaryMacAccumulateFiftyFourPlusGeneric` — `a+b+...+as+au+...+bb+bc` (54 variables).  
   Watch elaboration time; fallback to 53-plus/52-minus if timeout.
2. `ternaryMacAccumulateFiftyThreeMinusGeneric` — 53-variable minus lattice.
3. `ternaryMacUntrigintupleCancellationGeneric` — depth-31 alternating plus/minus with residual `= mac(x, a, .plus)`.
4. `ternaryMacZeroWeightDuovigintupleClosureGeneric` — 12 zero + 1 plus + 12 zero (37th proof-lattice dimension).

### 4.3 gen-verilog Defect 6 — `let` destructuring

- Find `StmtLocal` lowering path in `bootstrap/src/compiler.rs`.
- Detect tuple/destructuring pattern: `let(a, b, c) = expr`.
- For W378, emit a packed-vector temporary for the RHS call result and scalar `reg` declarations + slice assignments for each binding.
- Document that full tuple-return function generation is still required for semantic completeness; the W378 fix targets syntax-level yosys cleanliness.
- Regression spec: `specs/scratch/w378_let_destructuring.t27`.

### 4.4 CI smoke gate expansion

- Update `bootstrap/src/suite.rs` `igla_clean_specs()` to include `specs/igla/race/cordic.t27` and `specs/igla/race/cordic_top.t27` once they parse cleanly.
- Full IGLA coverage: 27 specs + scratch specs = 38 targets.

### 4.5 Seal regeneration and verification

- Build `t27c` release.
- Run `t27c suite --repo-root .`; expect seal mismatches from compiler change.
- Capture list of specs with hash changes and batch reseal.
- Run suite again until 0 failures.

### 4.6 Documentation and learnings

- Write `docs/reports/WAVE_LOOP_378_REPORT.md`.
- Write `docs/reports/WAVE_LOOP_378_COOPERATION.md` (three variants for W379).
- Write `docs/reports/FPGA_EVIDENCE_W378.md`.
- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.
- Update `.trinity/experience.md`.
- Save memory file and update `MEMORY.md`.

---

## 5. Risk and fallback

- **54-variable theorem** may push Lean elaboration >30 s. Fallback: reduce plus accumulation to 53 and minus to 52, accepting **255 generic ∀**.
- **Defect 6** may require more than syntax-level change if tuple-return lowering is deeply missing. Fallback: emit scalar regs and dummy assignments to make Verilog parse, document semantic gap, and defer full tuple-return work to W379.

---

*phi² + 1/phi² = 3 | TRINITY*
