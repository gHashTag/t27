# Wave Loop 477 — Plan: compiler-backend hygiene (hoisting + Icarus gate)

**Branch:** `wave-loop-477`  
**Source wave:** Wave Loop 476  
**Variant:** B (default, bench still blocked)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Close the gen-verilog hygiene gap identified in Wave Loop 476:

1. Hoist all local declarations (memory-mode arrays, scalar-struct temps, loop
   variables, packed-vector temps) to the top of each generated Verilog function
   body so the output is strict Verilog-2001 / SystemVerilog compliant.
2. Add an Icarus Verilog compilation and simulation gate to the conformance
   suite so emitted `initial begin assert(...)` test blocks are actually executed.
3. Keep all existing Yosys smoke targets green.
4. Add an adversarial scratch spec that interleaves declarations and statements.

## Scope

- **Variant A** (live cold-POR CCLK sweep) is only viable if the DLC10 cable and
  P12 relay wiring are located.
- **Variant B** (this plan) is the default and has no hardware dependency.
- **Variant C** (Lean 4 synthesizability/correctness lemmas) is the fallback if
  hoisting turns out to be larger than one wave.

## Literature and context

- **LiveHD (Wang et al., CC 2023)** — multi-threaded hardware compiler that
  lowers FIRRTL/Verilog/Pyrope to generated Verilog and resolves module/declaration
  ordering dynamically during IR construction. Relevant to t27's own ordering problem.
- **Icarus Verilog v13 strict declaration-before-use** — IEEE 1800-2012 §6.6
  requires nets/variables to be declared lexically before use. Icarus now enforces
  this by default, making t27's current output non-compliant.
- **AutoBench / CorrectBench (Qiu et al., 2024/2025)** — simulation-based validation
  of generated HDL testbenches with Eval0–Eval2 conformance metrics. Relevant to
  adding an Icarus execution gate.
- **Gu et al., DAC 2018** — specification-driven conformance checking for virtual
  prototype and post-silicon designs; relevant to the long-term spec-to-RTL bridge.
- **Sparkle HDL / Verilean** — Lean-native HDL; strict output hygiene is part of
  the formal-synthesis story t27 is building.

## Decomposed tasks

### Phase 1 — TDD spec

Add one scratch spec under `specs/scratch/`:

1. `w477_hoisting_and_iverilog.t27`
   - Defines `Pt { x: u8, y: u8 }` and `Shape { pts: [3]Pt }`.
   - Interleaves local declarations and assignments in a single function:
     declare a scalar, then a local struct array, then assign the scalar, then
     copy the array, then read from the copy with variable index, then a loop
     using a fresh loop variable.
   - Contains tests and invariants.
   - Must pass both Yosys elaboration and Icarus `iverilog -g2012` + `vvp`.

### Phase 2 — Function-body declaration hoisting

In `bootstrap/src/compiler.rs`, change `gen_verilog_fn_internal` so that local
registers, loop variables, and packed-vector temporaries are emitted before any
statements inside the `begin : ... end` function body.

Approach:
- Introduce a `Vec<String>` of pending declarations per function body.
- When the body generator would emit a declaration (local array, struct var temp,
  `_aos_ret_tmp_*`, `_aos_elem_tmp_*`, `_let_tmp_*`, loop variable `integer`),
  append the declaration string to the pending list instead of writing it to the
  output buffer.
- After generating the body but before writing it, emit all pending declarations
  at the top of the `begin` block, preserving declaration order.
- Then emit the previously buffered body statements.
- This is similar to the existing `aos_tmp_decls` mechanism, but generalized to
  all local declarations.

Important constraints:
- Do not change the observable behavior of generated code (only declaration order).
- Preserve seal stability where possible; many existing seals will require resealing
  because generated text legitimately changes.
- Keep Yosys smoke green.

### Phase 3 — Icarus simulation gate

In `bootstrap/src/suite.rs`:

- Add `iverilog_available()` analogous to `yosys_available()`.
- Add `cmd_gen_verilog_iverilog_smoke(repo, rel)` that:
  1. Calls `cmd_gen_verilog_stdout(repo, rel)` to get the Verilog text.
  2. Writes it to a temp file.
  3. Runs `iverilog -g2012 -o <tmp.vvp> <tmp.v>`.
  4. If compilation succeeds and the spec contains test blocks, runs `vvp <tmp.vvp>`.
  5. Treats any compilation error, runtime assertion failure, or stderr warning as a failure.
- Insert the new phase after `Gen Verilog Yosys Smoke` and before the FPGA gates.
- In `--fast` mode, keep the phase enabled (it is cheap and not FPGA-dependent).
- Update `SuiteSummary` to include `iverilog_smoke_*` fields if needed.

### Phase 4 — Verify and reseal

- `cargo build --release`
- `cargo test -p t27c --bin t27c`
- `./scripts/tri test --fast`
- Full `./scripts/tri test`
- Reseal all affected specs and refreeze `bootstrap/stage0/FROZEN_HASH` if the
  compiler hash changes.
- Target: ≥644/644 non-smoke PASS, Yosys smoke green, Icarus smoke green,
  0 seal mismatches.

### Phase 5 — Close-out and cooperation variants

- Write `docs/reports/WAVE_LOOP_477_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W478_2026-07-08.md` with three
  variants for Wave Loop 478.
- Update `docs/NOW.md`, `.trinity/experience.md`, and persistent memory.
- Create `wave-loop-478` branch.

## Exit criteria

- `w477_hoisting_and_iverilog.t27` passes Yosys and Icarus.
- All 644 non-smoke specs pass.
- All 124 yosys smoke targets pass.
- Icarus gate passes on scratch + IGLA specs.
- Seals match, `FROZEN_HASH` stable.
- Close-out report and cooperation variants written.

---

*φ² + φ⁻² = 3 | TRINITY*
