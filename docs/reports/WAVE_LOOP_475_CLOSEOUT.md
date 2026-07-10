# Wave Loop 475 — Close-out Report (2026-07-07)

**Issue:** (to be opened)  
**Branch:** `wave-loop-475`  
**Variant selected:** B — compiler-backend aggregate hardening (bench still blocked)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 475 closed the last two gaps in the function-local / array-of-struct
aggregate-lowering line:

1. **Array-of-struct equality for structs with array-typed fields.** The W474
   equality lowering only handled element structs whose leaves were scalar. W475
   generalized the packer so it can read memory-mode local arrays and module-level
   per-field memories that contain array-typed fields, then compares the two packed
   vectors with `==` / `!=`.

2. **Function-local arrays of structs passed as array parameters.** A function
   like `sum_pts(pts: [3]Pt)` can now be called from another function with a
   function-local `[3]Pt` array. The array parameter is passed as a single packed
   vector input, and field access inside the callee (`pts[i].x`) is lowered to a
   packed-vector slice (literal index) or a priority mux (variable index).

The physical bench remains blocked, so Variant B was selected by default. The
conformance suite is green at **640/640** non-smoke specs and **120/120** yosys
smoke targets, with **zero** gen-verilog smoke failures and **zero** seal mismatches.

---

## What landed

### `bootstrap/src/compiler.rs`

- Extended the W461/W463 array-parameter binding pass to recognize function-local
  array identifiers as packed-vector arguments:
  - `is_fn_local_array`, `fn_local_array_type`, `find_fn_local_array_type`
  - `array_param_clone_origins`, `array_param_local_packed_indices`
  - `fn_array_param_types`, `fn_array_param_names`
- Added a `__local__` signature marker so that all call sites that pass a local
  array to the same array parameter share a single packed-vector clone.
- Emitted local-packed array parameters as scalar inputs whose width is the total
  packed bit width of the declared array type (size × element width, recursively).
- Added `try_emit_local_packed_array_param_field` to lower field access on a
  packed-vector array parameter to a direct bit slice when indices are literals, or
  to a priority mux over every element position when any index is variable.
- Updated `ExprCall` to pack local-array arguments with
  `gen_verilog_pack_array_of_struct_expr` when the target array parameter is
  local-packed, instead of dropping the argument like a module-level binding.
- Generalized `gen_verilog_pack_array_of_struct_expr` to pack memory-mode local
  arrays and module-level arrays whose element struct has array-typed fields,
  matching the order used by array-literal and function-return packers so equality
  comparisons are bit-exact.
- Fixed scalar-struct equality for function parameters whose fields are arrays:
  the identifier is now packed directly instead of being expanded into non-existent
  per-field registers.
- Updated `array_param_bound_name` to ignore the `__local__` marker and let the
  new packed-vector field-access path handle those parameters.

### Regression specs

- `specs/scratch/w475_local_aos_param.t27`  
  Function-local `[3]Pt` arrays passed to `[3]Pt` array-parameter functions
  `sum_pts` and `sum_pts_y`, including a twice-called local array.

- `specs/scratch/w475_nested_field_equality.t27`  
  Scalar-struct equality and AOS equality where the element struct has an
  array-typed field (`Shape { pts: [3]Pt }`).

- `specs/scratch/w475_adversarial_nested_equality.t27`  
  Adversarial yosys-elaboration witness combining nested AOS equality, local-array
  parameter passing, and variable-index field access on a packed-vector parameter.

### Seals and stage-0 hash

- All affected `.trinity/seals/*.json` files were resealed to the new gen-verilog
  output.
- `bootstrap/stage0/FROZEN_HASH` was refrozen to  
  `6a99d79b4925fe4da44b593260bb217792b33213ad12b3a9d19830e7048f3c4b`.

---

## Weak spots and related work

### Project weak spots

- **Physical boot-evidence gap.** The strongest differentiation — live cold-POR
  CCLK sweeps on the Wukong XC7A100T — is still gated by missing hardware (DLC10
  cable / unwired P12 relay). This has been the dominant blocker for many
  consecutive waves.
- **Copy initializers for arrays of structs with array-typed fields.**
  `var c : [2]Shape = b;` where `b` is another local array variable is not yet
  lowered; initialization from array literals and function calls is covered.
- **Module arrays as packed-vector array arguments.** A module-level array of
  structs can be bound to an array parameter by name, but passing it through the
  packed-vector path is not yet unified with the local-array path.
- **Lean ↔ Verilog semantic bridge.** The per-field memory model is tested by
  simulation and yosys elaboration, but there is still no formal proof that the
  packed-vector slice arithmetic preserves source read/write semantics.
- **Master-merge divergence.** A related fix set exists on `master` (`701d79b3b`)
  for earlier gen-verilog defects. It remains deferred; plan it as its own small
  wave rather than merging opportunistically.

### Scientific / engineering context

- The ternary/ternary-trit HDL space remains thin in the literature. Sparkle HDL
  and Verilean are the closest public Lean-native hardware-description
  experiments. No published work has demonstrated a spec-to-bitstream pipeline
  for ternary-weighted neural accelerators with sealed numeric conformance, which
  is t27's core claim.
- t27's backend now supports a full struct-of-arrays / array-of-structs
  decomposition: scalar fields become per-element per-field registers, array-typed
  fields become per-field unpacked memories, and arrays of structs are passed
  between functions as packed vectors. This matches the register/memory model of
  Verilog and avoids packed arrays of structs that most synthesizers reject.
- The packed-vector slice arithmetic for local-packed parameters is the same
  arithmetic used for array-literal and function-return packers, which makes
  equality comparisons and parameter passing bit-exact.

---

## Not done (blocked or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Live-capture `XADC_LIVE_W475_OPERATING_POINT` — bench unavailable.
- Copy initializers from one local array-of-struct variable to another —
  deferred.
- Unified module-array → packed-vector argument path — deferred.
- Lean 4 synthesizability/correctness lemmas for the packed-vector memory model —
  deferred to a future Variant C wave.
- Master-merge of the `master` gen-verilog fix set — still deferred; should be
  planned as its own small wave.

---

## Verification

- `cargo build --release`: **PASS**.
- `cargo test -p t27c --bin t27c`: **1524 passed, 0 failed, 2 ignored**.
- `./scripts/tri test --fast`: **ALL TESTS PASSED**
  - Parse / Typecheck / Gen Zig / Gen Rust / Gen Verilog / Gen C / Seal Verify:
    **640/640 PASS**.
  - Gen Verilog Yosys Smoke: **120 passed, 0 failed**.
  - FPGA Board-Less Smoke Gate: **OK**.
  - Fixed Point: 0 divergences.
  - **TOTAL FAILURES: 0** — `BASELINE FAILURES: 0`, `ACCEPTABLE: yes`.
- Full `./scripts/tri test`: **ALL TESTS PASSED**
  - 640/640 parse/typecheck/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify PASS.
  - Gen Verilog Yosys Smoke: **120 passed, 0 failed**.
  - FPGA Board-Less Smoke Gate: **OK**.
  - FPGA Standalone Lake-Package Build: **OK**.
  - Fixed Point: 0 divergences.
  - **TOTAL FAILURES: 0** — `BASELINE FAILURES: 0`, `ACCEPTABLE: yes`.

---

## Close-out artifacts

- `docs/reports/WAVE_LOOP_475_CLOSEOUT.md` (this file)
- `docs/reports/FPGA_LOOP_COOPERATION_W476_2026-07-08.md`
- `.trinity/ring-475.md`
- `.trinity/experience.md` (appended)
- `~/.claude/projects/-Users-playra-t27/memory/wave-loop-475.md`

---

## Next wave

- **Branch:** `wave-loop-476`
- **Plan:** `docs/reports/FPGA_LOOP_COOPERATION_W476_2026-07-08.md`

---

*φ² + φ⁻² = 3 | TRINITY*
