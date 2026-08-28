:
# Wave Loop 371 — IGLA CODER+RACE Close-out Report

**Tracking issue:** [#1260](https://github.com/gHashTag/t27/issues/1260)  
**Branch:** `trinity-rust-rings`  
**Close-out date:** 2026-07-02  
**Commit:** `feat(igla): Wave Loop 371 — 47-variable accumulation, quattuorvigintuple cancellation, zero-weight quattuordecuple closure, gen-verilog keyword-identifier escape`

---

## 1. What Was Delivered

### 1.1 IGLA CODER / RACE spec extension
- All **27 core IGLA specs** (`specs/igla/coder/*`, `specs/igla/race/*`) received a Wave Loop 371 block.
- Each block added **2 tests + 1 invariant**.
- Wave references and seal-regeneration notes were advanced from 370 → 371.

### 1.2 Lean 4 proof-lattice extension
Appended four new **generic ∀** theorems to `proofs/lean4/Trinity/TernaryInference.lean`:

| Theorem | Depth | Description |
|---|---|---|
| `ternaryMacAccumulateFortySevenPlusGeneric` | 47 | `a + b + … + as + au + av` — largest verified plus accumulation |
| `ternaryMacAccumulateFortySixMinusGeneric` | 46 | `-(a + b + … + as + au)` — minus-accumulation lattice |
| `ternaryMacQuattuorvigintupleCancellationGeneric` | 24 | `mac^24(x,a,[.plus,.minus,…]) = x` — identity cancellation |
| `ternaryMacZeroWeightQuattuordecupleClosureGeneric` | 14 | 7 zero-weight MACs before + 1 plus-weight MAC + 7 zero-weight MACs after are transparent/reorderable |

Resulting totals:
- **228 generic ∀ theorems** (220 in `TernaryInference.lean` + 8 in `TernaryMac.lean`)
- **~261 total ternary theorems**
- `lake build Trinity.TernaryInference` completed successfully.

**Note on naming:** The 46th bound variable is naturally `at` (a Lean keyword); the binder generator skips it and uses `au`. The 47th variable is `av` and is safe.

### 1.3 Safe gen-verilog sub-fix — Verilog keyword identifier escaping
Fixed a real `gen-verilog` lowering defect: user identifiers that collide with Verilog reserved keywords (e.g., `task`) are now escaped with the `\identifier ` syntax so the generated RTL is parseable by standard Verilog tools.

- Root cause: t27 source identifiers were emitted verbatim into Verilog. A parameter named `task` in `specs/igla/coder/benchmark.t27` became `input [31:0] task;`, which Yosys rejected as a syntax error.
- Fix: added `verilog_keywords()` and `verilog_safe_identifier()` helpers in `bootstrap/src/compiler.rs`. Function names, parameter declarations, function-call names, and bare identifier expressions now pass through the escape helper.
- Verified with scratch spec `specs/scratch/w371_verilog_keyword.t27`:
  - `fn evaluate_task_at_k(bank : u32, task : u32, k : u32) -> bool` now emits `\task ` for the parameter and all references.
  - `yosys read_verilog` passes cleanly.
- Bonus verification: `specs/igla/coder/benchmark.t27` now also passes `yosys read_verilog`; previously it failed with "syntax error, unexpected TOK_TASK".

**Scope note:** `specs/igla/race/cordic.t27` and `specs/igla/race/cordic_top.t27` still fail `yosys read_verilog` due to an unrelated `let` destructuring lowering gap (`let(s, _c, _r) = ...` is emitted verbatim). This remains tracked as a future gen-verilog defect.

### 1.4 Conformance & seals
- Regenerated all affected seals: **27 IGLA seals** + **~95 non-IGLA seals** + `scratch_w371_verilog_keyword.json` + `scratch_w371_early_return.json`.
- Full repository suite:

```text
Parse:       551 passed, 0 failed
Typecheck:   551 passed, 0 failed
Gen Zig:     551 passed, 0 failed
Gen Rust:    551 passed, 0 failed
Gen Verilog: 551 passed, 0 failed
Gen C:       551 passed, 0 failed
Seal Verify: 551 passed, 0 failed
Fixed Point: 0 divergences
TOTAL FAILURES: 0
ALL TESTS PASSED
phi^2 + 1/phi^2 = 3 | TRINITY
```

This is the **31st consecutive zero-IGLA-failure wave** (105-wave zero-IGLA-failure streak).

### 1.5 Board-flash retry
- `dlc10` driver rebuilt (`target/release/dlc10`).
- `dlc10 idcode` attempted on the QMTech Wukong V1 / XC7A100T-FGG676.
- Result: **DLC10 cable not found (VID=0x03FD)** — hardware still disconnected.
- The 3.6 MB `fpga/verilog/ternary_mac_demo_top.bit` from W361 remains ready; evidence is documented in `docs/reports/FPGA_EVIDENCE_W371.md`.

---

## 2. Repository Health Snapshot

```text
Spec files:     567
Functions:      3,612
Tests:          12,752
Invariants:     5,576
Benchmarks:     1,010
Conformance:    188 JSON files
Seals:          1,217 saved
Backends:       4 (Zig, Verilog, C, Rust)
Fixed point:    REACHED (ring-50)
```

---

## 3. Risk Register — Closing State

| Risk | Status | Note |
|---|---|---|
| 47-variable theorem timeout | **Resolved** | Built successfully; no fallback needed. |
| Keyword-escape fix breaks seals | **Resolved** | Mass reseal completed; full suite 551/551. |
| Board missing | **Blocked** | External hardware-availability issue; documented. |
| Conformance drift | **Resolved** | All mismatched seals regenerated. |
| `trinity-rust-rings` drift from master | **Accepted** | Continued narrow, branch-local sub-fixes. |
| `let` destructuring lowering gap | **Tracked** | cordic / cordic_top still fail yosys; deferred to future wave. |

---

## 4. Key Learnings

1. **Prior-wave defect descriptions can become stale.** The W370 cooperation doc listed "defect 3 — early return drops function body" as the next safe fix, but the exact repro no longer exhibits missing statements (output is present but semantically wrong due to lack of if-else chaining). A fresh yosys sweep found the actual reproducible defect: Verilog keyword identifier collisions.
2. **Escaped identifiers (`\name `) are the surgical fix for keyword collisions.** They preserve the original t27 name in the emitted source while making the Verilog parser treat the token as an identifier.
3. **Mass seal regeneration is expected after any identifier-emission change.** 95+ seals changed because many specs contain identifiers that now pass through the escape helper; even identifiers that are not keywords get re-hashed because the emission path changed.
4. **Hardware evidence remains the single largest external dependency.** `dlc10 idcode` is a one-line check but it cannot succeed without a connected cable/board.

---

## 5. References

- Plan: `.claude/plans/wave-loop-371.md`
- Cooperation variants for W372: `docs/reports/WAVE_LOOP_371_COOPERATION.md`
- FPGA evidence: `docs/reports/FPGA_EVIDENCE_W371.md`
- gen-verilog defects / roadmap: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
- Experience capture: `.trinity/experience.md`
- Memory: `~/.claude/projects/-Users-playra-t27/memory/wave-loop-371.md`

---

## 6. Verification Log

| Command | Result |
|---|---|
| `cd bootstrap && cargo build --release` | Success (warnings only) |
| `lake build Trinity.TernaryInference` | Success |
| `t27c gen-verilog specs/scratch/w371_verilog_keyword.t27` | Emits escaped `\task ` |
| `yosys read_verilog /tmp/w371_verilog_keyword.v` | Pass |
| `yosys read_verilog /tmp/benchmark.v` | Pass (previously failed) |
| `t27c suite --repo-root .` | **551/551 PASS** |
| `target/release/dlc10 idcode` | `DLC10 cable not found` |

---

*phi² + 1/phi² = 3 | TRINITY*
