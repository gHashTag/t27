# FPGA Loop Cooperation Plan — Wave Loop 460 (2026-07-01)

**Issue:** #1433 (to create)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context at the end of W459

Wave Loop 459 closed #1431 by selecting Variant B: `gen-verilog` backend
hardening. The wave added (1) array-parameter binding for test/invariant/bench
call sites, (2) real assertion/function-call emission inside `` `ifndef
SIMULATION `` test blocks, (3) a known-warnings gate in the yosys smoke runner
that now defines `SIMULATION` during parsing, and (4) `rom_style` pragma support
for `const [N]T` ROM declarations. The `--fast` suite path is green:
583/583 non-smoke PASS, 63/63 yosys smoke PASS, FPGA smoke gate OK, 0 baseline
failures, 0 seal mismatches.

The physical bench remains blocked: `dlc10 idcode` reports "DLC10 cable not
found (VID=0x03FD)", P12 is unwired, and no automated cold-POR relay gate exists.
The default `./scripts/tri test` also cannot complete in this environment
because Phase 3c-standalone stalls while `lake` downloads the `batteries`
dependency from `reservoir.lean-lang.org`; the smoke-gate report itself passes.

This cooperation plan proposes three mutually-exclusive W460 execution
strategies. Select **one** at the start of the wave.

---

## Variant A — Live CCLK capture if the bench unblocks

Execute only if the DLC10 cable is found **and** P12 + relay are wired.

### Goal
Run the first live cold-POR CCLK sweep since W434 and mint a hardware-backed
theorem fixture set under the post-W459 bitstream.

### Scope
1. Run `tri fpga cclk-sweep --json` across OSCFSEL 0–7 with the current
   post-W459 bitstream.
2. Persist fixtures under `tests/fixtures/fpga/theorem-matrix/live-w460/` with
   PVT context and raw-ns measurements.
3. Mint `XADC_LIVE_W460_OPERATING_POINT` theorem in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
4. Add a regression gate that rejects fixture drift beyond the documented PVT
   envelope.

### Acceptance
- At least one successful live sweep completes and its fixture is committed.
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test --fast` remains 583/583 non-smoke PASS with yosys smoke OK.

---

## Variant B — Compiler backend hardening: generalize array parameters + clean bench lowering (default)

Execute when the bench is still blocked. This is the most likely W460 path.

### Goal
Generalize the W459 array-parameter work so functions with array parameters can
be bound from arbitrary call sites and bench-block local state is lowered
 cleanly, then clear the three pre-existing `let_binding` cargo-test failures.

### Scope
1. **Multiple / non-identifier array-parameter call sites.** Extend the binding
   analysis so a function with an array parameter can be called from more than one
   module-level site and from sites that pass a module-level array identifier.
   All sites must still agree on the same array. Add scratch specs that exercise
   multi-site binding and `assert_eq` checks.
2. **Bench-block local-variable lowering.** Bench blocks currently emit local
   variable assignments as implicit procedural wires, producing yosys warnings.
   Lower bench-local variables to properly declared module-scope registers inside
   `` `ifndef SIMULATION `` guards so the smoke gate stays warning-clean without
   relying on the allow-list.
3. **Clear the three pre-existing cargo-test failures.** Investigate and fix
   `let_binding_is_lowered_1401`, `test_let_binding_emitted_c_1401`, and
   `test_let_binding_emitted_rust_1401` in `bootstrap/src/compiler.rs` so the
   full `cargo test -p t27c --bin t27c` suite is green.
4. Reseal all affected specs and shrink the `YOSYS_ALLOWED_WARNINGS` allow-list
   as the new lowering removes the need for some entries.

### Acceptance
- `./scripts/tri test --fast` reports 0 failures and `ACCEPTABLE: yes`.
- New scratch specs pass `t27c gen-verilog` + `yosys read_verilog -sv -DSIMULATION`
  and are exercised by at least one `assert_eq`.
- `cargo test -p t27c --bin t27c` passes with **0 failures**.
- The yosys smoke allow-list is updated to reflect the cleaner output.

---

## Variant C — Formal boot-evidence fallback

Execute if Variant B is blocked by an AST/scope refactor that cannot be
completed safely in one wave.

### Goal
Extend the board-less Lean 4 boot-evidence lattice with compiler-correctness
bridge statements that treat the now-cleaner `gen-verilog` backend as a trusted
source of synthesizable artifacts.

### Scope
1. **Synthesizability theorem block.** Add propositions in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` stating that the W458 and W459
   regression specs (`w458_array_param_read`, `w458_array_param_write`,
   `w459_array_param_test_call`, `w459_rom_style_block`) produce yosys-clean
   Verilog, expressed over seal hashes and the yosys smoke report.
2. **Adversarial clock-jitter envelope.** Quantify worst-case raw-ns predicate
   preservation under ±2 ns bounded jitter across all OSCFSEL selections and
   all four PVT corners.
3. **Compiler-correctness bridge lemma.** Relate the cleared test-block
   assertion emission and ROM-style pragma backend to the abstract ternary MAC
   semantics in `TernaryInference.lean`.
4. Add matching Rust unit tests in `cli/tri/src/fpga.rs`.

### Acceptance
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test --fast` remains 583/583 non-smoke PASS with yosys smoke OK.
- At least 3 new Lean theorems and 3 new Rust unit tests land.

---

## Recommended selection order

1. **Variant A** if hardware becomes available during the W460 start-of-wave probe.
2. **Variant B** otherwise — it is the natural continuation of the compiler
   hardening line after W455/W456/W457/W458/W459.
3. **Variant C** only if Variant B hits an unresolvable parser/AST scope blocker.

---

*φ² + φ⁻² = 3 | TRINITY*
