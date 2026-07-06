# Wave Loop 462 Plan — Variant B (Compiler backend hardening)

**Issue:** #1437  
**Branch:** `wave-loop-462`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Weak points investigated

After W461 the `gen-verilog` backend still has three related gaps around
array parameters:

1. **Literal array arguments.** A call like `sum_pair([4]u16{1,2,3,4}, 0, 1)` is
   rejected because the binding pass only accepts module-level identifiers for
   array-parameter slots. Users must declare a named module-level ROM first.
2. **Void-return bare module-level calls waste dummy registers.** A module-level
   bare call to a `void` function currently falls back to a 32-bit dummy register
   because `fn_return_types` does not expose the void return type. Verilog-2001
   allows `task`-style bare enables for void calls, so the dummy register is
   unnecessary and confusing.
3. **Bench-local hoisting + array parameters.** Bench blocks can declare local
   variables and can call array-parameter functions, but no regression spec
   exercises both at once. The interaction between hoisted bench-local names
   and clone-name derivation is untested.

Other tracked gen-verilog defects are either already fixed (keyword escape,
`let` preservation, bench-local hoisting, multi-array cloning) or belong to the
broad master-merge debt that is explicitly out of scope for a single wave.

---

## Competitor snapshot (input for T27_VS_FORMAL_HDL_2026.md)

- **Sparkle / Verilean:** last public push on **2026-07-03**, stable cache-key
  fix for multi-output sub-modules. The repository still shows the FIDO2/crypto
  burst from 2026-07-04 and the 関数型まつり2026 talk on 2026-07-11 remains the
  most recent public competitive checkpoint.
- **CIRCT / firtool:** `firtool-1.152.0` shipped on **2026-07-04**. No newer
  public release has appeared between W461 and W462 boundaries.
- **Ternary-FPGA ecosystem:** activity continues around TernaryCore,
  BitNet-RISCV-Multicore, and related projects, validating the {-1,0,+1}
  compute niche but not combining it with a Lean-native proof pipeline.

No new public competitor signals surfaced since the W461 close-out.

---

## Decomposed tasks

### Task A — Literal array arguments for array parameters

- Extend the array-parameter binding pass so that when a call site passes an
  `ExprArrayLiteral` to an array-parameter slot, the literal is lowered to a
  module-level anonymous ROM (or reused if the same literal appears in another
  call site).
- Mint a binding signature that includes the literal's contents so identical
  literals share a clone while different literals get different clones.
- Emit the anonymous ROM in `gen_verilog_module` with the same style as
  `const [N]T{...}` ROMs.
- Add regression spec `specs/scratch/w462_array_param_literal.t27`.

### Task B — Void-return bare module-level calls skip dummy registers

- Track void-return functions in `fn_return_types` (value `"void"`).
- In `dummy_reg_width_for_call`, return width `0` for void callees.
- In `gen_verilog_module`, when `dummy_reg_width_for_call` is `0`, emit the
  call as a bare `task`-enable statement inside `always @(*)` instead of
  declaring a dummy register and assigning to it.
- Add regression spec `specs/scratch/w462_void_bare_call.t27` with a void
  function called at module level.

### Task C — Bench-local + array-parameter integration spec

- Add a regression spec `specs/scratch/w462_array_param_bench_local.t27` that
  declares bench-local variables and calls an array-parameter function from
  the bench block, exercising the W460 hoisting and W461 clone paths
  together.
- No compiler change expected; this is a coverage/verification task.

### Task D — Verification and seal refresh

- Build t27c release (`cargo build --release`).
- Reseal all affected specs.
- Run `./scripts/tri test --fast` with 0 failures.
- Run `cargo test -p t27c --bin t27c` with 0 failures.
- Run `t27c check-now`.

### Task E — Close-out artifacts

- Write `docs/reports/WAVE_LOOP_462_REPORT.md`.
- Write `docs/reports/FPGA_LOOP_EVIDENCE_W462_2026-07-06.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W463_2026-07-06.md` with three
  variants for W463.
- Update `docs/NOW.md` and `docs/reports/T27_VS_FORMAL_HDL_2026.md`.
- Save memory entry `memory/wave-loop-462.md` and update `MEMORY.md`.
- Commit and push `wave-loop-462`; create and push `wave-loop-463`.

---

## Acceptance

- `./scripts/tri test --fast`: 589/589 non-smoke PASS, yosys smoke 0 failures,
  0 seal mismatches, `ACCEPTABLE: yes`.
- `cargo test -p t27c --bin t27c`: 1524+ passed, 0 failed.
- New scratch specs pass `t27c gen-verilog` + `yosys read_verilog -sv -DSIMULATION`.

---

*φ² + φ⁻² = 3 | TRINITY*
