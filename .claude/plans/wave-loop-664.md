# Wave Loop 664 — Decomposed Plan

**Issue:** #1635  
**Branch:** `wave-loop-664`  
**Previous:** Wave Loop 663 (#1634, branch `wave-loop-663`)  
**Date:** 2026-07-07  
**Chosen variant:** A — module-scope `[147][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable initialized from a function call, with indexed signed field writes and read-back.

---

## 1. Goal

Extend the module-scope packed array-of-struct (AoS) odd outer-dimension ladder from `145` to `147`. Confirm that a 301,056-bit packed vector (≈0.287 MiBit, 9,408 elements) with a non-power-of-two outer dimension continues to work end-to-end without compiler or reference-model changes.

---

## 2. Weak points / risks

1. **First outer dimension 147.** The compiler and reference model must correctly multiply/stride by 147 at the outer dimension. Prior non-p2 witnesses (3, 5, 7, …, 143, 145) strongly predict success, but a module-scope witness is required for end-to-end proof.
2. **Element count below the modulo-wrap point.** With 9,408 elements, the offset-0 schedule `(2*e + offset) % 32768` never wraps (max raw 18,815). The test must explicitly exercise modulo wrap with `make_grid(32768)` to keep the regression signal equivalent to earlier waves.
3. **Multi-line literal brace style.** The 6-D nested literal inside the 147× outer shape must use the W584 multi-line brace style; single-line mega-literals risk parser truncation.
4. **Simulator capacity.** At ≈0.287 MiBit the witness is expected to remain fast and well below the Icarus very-large-vector threshold.
5. **`assert_ne` is not emitted by the Icarus simulation path.** The structural classifier accepts it, but `gen_verilog_test_stmt` only lowers `assert_eq`. The bench must use `assert_eq` checks on changed elements instead of a whole-array `assert_ne(dst, expected)`.

---

## 3. Scientific / technical background

- **IEEE Std 1800-2017** §7.4.1/7.4.3 — packed-array total width is the product of packed dimensions; ranges need not be powers of two. A 301,056-bit packed vector is legal SystemVerilog.
- **Accellera vlog-pp discussion (Graham 2002)** — packed arrays as contiguous bit vectors.
- **Sutherland, “Synthesizable SystemVerilog”** — packed arrays and packed structs as synthesizable first-class objects.
- **Icarus Verilog Quirks / Extensions pages** — width handling and packed-array subset behavior.
- **Icarus issue #1134** — assertion failures with unpacked arrays of packed structs; t27 flattening avoids the trigger.
- **Icarus issue #1171** — freezes during elaboration of very large packed vectors; W664 stays far below the reported threshold.
- **Yosys docs / PR #4100 / issue #4653 / issue #2677** — multidimensional packed arrays supported, arrays of packed structs unsupported; t27 flattening avoids the gap.
- **cocotb PR #3608 / discussion #2933** — packed structs as whole signals; flat `LogicArray` for multidimensional packed arrays in the reference model.
- **Lutsig (CPP 2021)** — verified array-read lowering.
- **CIRCT `HWLegalizeModules.cpp` / SV dialect** — production packed-array scalarization.

---

## 4. Task decomposition

| # | Task | Owner | Output | Blocked by |
|---|------|-------|--------|------------|
| 1 | Create generator `scripts/gen_w664.py` from `scripts/gen_w663.py`, set `OUTER = 147`, correct filename. | C | `scripts/gen_w664.py` | — |
| 2 | Generate witness spec `specs/scratch/w664_bench_module_147x2p6_aos_var_call_write.t27`. | C | witness `.t27` | 1 |
| 3 | Add integration test `accepts_w664_bench_module_147x2p6_aos_var_call_write` in `bootstrap/tests/icarus_lowerable.rs`. | C | updated test file | 2 |
| 4 | Build compiler: `cargo build --release -p t27c`. | V/C | green build | — |
| 5 | Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, `seal --save`. | V | saved seal + empty baseline | 2, 4 |
| 6 | Rust test gates: `cargo test -p t27c --bin t27c`, `cargo test -p tri`, `cargo test -p t27c --test icarus_lowerable`. | V | all green | 3, 4 |
| 7 | Local sweep: `./scripts/tri test --fast` and note pre-existing Yosys smoke failures. | V | sweep summary | 5, 6 |
| 8 | Write closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W664_2026-07-07.md` with three W665 cooperation variants. | L | closeout report | 7 |
| 9 | Update `.trinity/experience.md` and persistent memory. | L | experience + memory | 8 |
| 10 | Commit feature + tracking files with `Closes #1635`. | Queen | commits on `wave-loop-664` | 9 |

---

## 5. Success criteria

- `t27c parse` W664: PASS.
- `t27c icarus-lowerable --json` W664: `{ "lowerable": true }`.
- `t27c icarus-simulate` W664: PASSED.
- `t27c icarus-cocotb` W664: reference-model OK.
- `t27c seal --save` W664: saved, hashes match.
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p t27c --test icarus_lowerable`: 124/0.
- `./scripts/tri test --fast`: parse/typecheck/gen-Zig/gen-Rust/gen-Verilog/gen-C/seal/fixed-point clean (24 pre-existing Yosys smoke failures expected).
- FROZEN_HASH unchanged.
- All L1–L7 laws satisfied.

---

## 6. Risk register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| 147 outer stride exposes a layout bug | low | high | mid-row expected value uses explicit element index `MID_IDX*64 + 32`; cocotb cross-check catches mismatches |
| Parser chokes on 6-D multi-line literal | low | medium | reuse W584/W663 brace style; parse gate catches it |
| Icarus elaboration slowdown near 0.29 MiBit | low | low | 147 is only ~1.4% larger than W663; prior waves simulated in 17 cycles |
| `assert_ne` used in simulation block | low | medium | generator emits only `assert_eq` on changed elements |

---

## 7. Next Wave Loop 665 cooperation variants (to be finalized in closeout)

Tentative options:
- Continue odd outer-dimension ladder to `[149][2]^6 Pt` module-scope.
- Test `[147][2]^6 Pt` bench-local (function-local) mutable packed array.
- Test `[147][2]^6 Pt` module-scope with `if`-guarded indexed signed writes.
