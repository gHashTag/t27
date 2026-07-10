# Wave Loop 476 — Close-out Report (2026-07-07)

**Issue:** (to be opened)  
**Branch:** `wave-loop-476`  
**Variant selected:** B — compiler-backend aggregate tail (bench still blocked)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 476 closed the remaining user-facing aggregate-lowering tail deferred
from Wave Loop 475. The work was smaller than expected: the packed-vector and
value-semantics infrastructure landed in W475 already handled the three
outstanding cases correctly, so W476 was primarily a specification, verification,
and sealing wave.

What closed:

1. **Local array-of-struct copy initializers.** `var c : [2]Shape = b;` where `b`
   is another function-local array of structs with array-typed fields now works
   through the compiler's existing copy-propagation / value-semantics path. The
   generated Verilog initializes the destination from the source's per-field
   memories and, when the copy is unmodified, folds subsequent reads back to the
   source.

2. **Module-level arrays of structs passed as packed-vector array parameters.**
   A module-level const/var array of structs can be passed to a function that
   takes the same array type; the call site packs the per-field memories into a
   scalar packed-vector input, and the callee slices it with the same arithmetic
   used for function-local packed parameters.

3. **Whole-struct assignment for nested structs with array-typed fields.**
   Scalar struct variables (`shape_a = shape_b`) and array elements
   (`shapes_a[i] = shapes_b[i]`) whose element struct contains array-typed fields
   are lowered using packed-vector temporaries and per-field memory copies.

4. **Adversarial yosys-elaboration witness.** A single scratch spec exercises the
   intersection of local AOS copy initializers, module-array packed parameters,
   nested whole-struct assignment, and variable-index reads/writes.

The physical bench remains blocked, so Variant B was selected by default. The
conformance suite is green at **644/644** non-smoke specs and **124/124** yosys
smoke targets, with **zero** gen-verilog smoke failures and **zero** seal
mismatches.

---

## What landed

### Regression specs

- `specs/scratch/w476_local_aos_copy_init.t27`  
  Function-local `[2]Shape` arrays initialized from another local array variable,
  including equality, literal-index reads, variable-index reads, and chained
  copies.

- `specs/scratch/w476_module_aos_param.t27`  
  Module-level const `[2]Shape` passed to scalar-struct and array-of-struct
  parameter functions.

- `specs/scratch/w476_nested_whole_struct_assign.t27`  
  Whole-struct assignment for scalar nested structs and array-element assignment
  for nested AOS, including a variable-index element assignment.

- `specs/scratch/w476_adversarial_aggregate_tail.t27`  
  Integration witness combining local AOS copy init, module-array parameter
  passing, nested whole-struct assignment, and variable-index packed-vector
  slicing.

### Seals and stage-0 hash

- Added seals for the four new scratch specs:
  - `.trinity/seals/scratch_w476_local_aos_copy_init.json`
  - `.trinity/seals/scratch_w476_module_aos_param.json`
  - `.trinity/seals/scratch_w476_nested_whole_struct_assign.json`
  - `.trinity/seals/scratch_w476_adversarial_aggregate_tail.json`
- Resealed `.trinity/seals/scratch_w469_struct_field_array_2d.json` because the
  W475 memory-mode lowering legitimately changed its generated Verilog.
- `bootstrap/stage0/FROZEN_HASH` remains stable at  
  `117d7f3db91b15e1f0e52abe3a8c68005b103e73d44d71de5e44745043ce75b0`.

### Compiler notes

No new backend code was required for the three W476 targets. The W475 changes to
`bootstrap/src/compiler.rs` (packed-vector local array-parameter passing,
`try_emit_local_packed_array_param_field`, `gen_verilog_pack_array_of_struct_expr`
for memory-mode/module arrays with array-typed fields, and the value-semantics
assignment paths) already composed to cover them. The wave therefore validates
that the W475 infrastructure is complete for this aggregate-lowering line rather
than adding more backend surface.

---

## Weak spots and related work

### Project weak spots

- **Physical boot-evidence gap.** The strongest differentiation — live cold-POR
  CCLK sweeps on the Wukong XC7A100T — is still gated by missing hardware (DLC10
  cable / unwired P12 relay). This has been the dominant blocker for many
  consecutive waves and is the natural top priority whenever hardware appears.
- **Declaration ordering in function bodies.** Memory-mode local arrays (and
  scalar-struct temporaries) are declared after preceding assignments, which
  Yosys accepts but strict Verilog-2001 simulators such as Icarus reject. Fixing
  this requires hoisting all local declarations to the top of each function body
  and is better handled as its own small hygiene wave.
- **Lean ↔ Verilog semantic bridge.** The per-field memory model is tested by
  Yosys elaboration and the conformance suite, but there is still no formal proof
  that the packed-vector slice arithmetic preserves source read/write semantics.
- **Master-merge divergence.** A related fix set exists on `master` (`701d79b3b`)
  for earlier gen-verilog defects. It remains deferred; plan it as its own small
  wave rather than merging opportunistically.

### Scientific / engineering context

- The ternary/ternary-trit HDL space remains thin in the literature. Sparkle HDL
  and Verilean are the closest public Lean-native hardware-description
  experiments. The CktFormalizer arXiv preprint shows that dependently-typed
  Lean-to-silicon pipelines can dramatically improve backend realizability.
- t27's backend now supports a full struct-of-arrays / array-of-structs
  decomposition plus packed-vector parameter passing and equality. This matches
  the register/memory model of Verilog and avoids packed arrays of structs that
  most synthesizers reject, which is the same design point Vitis HLS uses when
  it disaggregates AoS into SoA internally.
- No published competitor has demonstrated a spec-to-bitstream pipeline for
  ternary-weighted neural accelerators with sealed numeric conformance across
  644 specs and 124 yosys smoke targets.

---

## Not done (blocked or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Live-capture `XADC_LIVE_W476_OPERATING_POINT` — bench unavailable.
- Function-body local-declaration hoisting for strict Verilog-2001 simulators —
  deferred to a hygiene wave.
- Lean 4 synthesizability/correctness lemmas for the packed-vector memory model —
  deferred to a future Variant C wave.
- Master-merge of the `master` gen-verilog fix set — still deferred.

---

## Verification

- `cargo build --release`: **PASS**.
- `cargo test -p t27c --bin t27c`: **1524 passed, 0 failed, 2 ignored**.
- `./scripts/tri test --fast`: **ALL TESTS PASSED**
  - Parse / Typecheck / Gen Zig / Gen Rust / Gen Verilog / Gen C / Seal Verify:
    **644/644 PASS**.
  - Gen Verilog Yosys Smoke: **124 passed, 0 failed**.
  - FPGA Board-Less Smoke Gate: **OK**.
  - Fixed Point: 0 divergences.
  - **TOTAL FAILURES: 0** — `BASELINE FAILURES: 0`, `ACCEPTABLE: yes`.
- Full `./scripts/tri test`: **ALL TESTS PASSED**
  - 644/644 parse/typecheck/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify PASS.
  - Gen Verilog Yosys Smoke: **124 passed, 0 failed**.
  - FPGA Board-Less Smoke Gate: **OK**.
  - FPGA Standalone Lake-Package Build: **OK**.
  - Fixed Point: 0 divergences.
  - **TOTAL FAILURES: 0** — `BASELINE FAILURES: 0`, `ACCEPTABLE: yes**.

---

## Close-out artifacts

- `docs/reports/WAVE_LOOP_476_CLOSEOUT.md` (this file)
- `docs/reports/FPGA_LOOP_COOPERATION_W477_2026-07-08.md`
- `.trinity/ring-476.md`
- `.trinity/experience.md` (appended)
- `~/.claude/projects/-Users-playra-t27/memory/wave-loop-476.md`

---

## Next wave

- **Branch:** `wave-loop-477`
- **Plan:** `docs/reports/FPGA_LOOP_COOPERATION_W477_2026-07-08.md`

---

*φ² + φ⁻² = 3 | TRINITY*
