# FPGA Loop Cooperation Plan — Wave Loop 456 (2026-07-01)

**Issue:** #1427  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context at the end of W455

Wave Loop 455 cleared the 7 residual `gen-verilog` yosys smoke failures by
porting tuple-return / `let` destructuring / ROM / function-local-array lowering
into the current branch. Full `./scripts/tri test` is now green (576/576 non-smoke
PASS, 56/56 yosys smoke PASS, FPGA smoke gate OK, 0 seal mismatches). The physical
bench remains blocked (DLC10 cable not detected, P12 unwired).

This cooperation plan proposes three mutually-exclusive W456 execution strategies.
Select **one** at the start of the wave.

---

## Variant A — Live CCLK capture if the bench unblocks

Execute only if the DLC10 cable is found **and** P12 + relay are wired.

### Goal
Run live cold-POR CCLK sweeps on the Wukong XC7A100T board and mint the first
hardware-backed theorem fixture set since W434.

### Scope
1. Run `tri fpga cclk-sweep --json` across OSCFSEL 0–7 with the current
   post-W455 bitstream.
2. Capture `tests/fixtures/fpga/theorem-matrix/live-w456/` with PVT context and
   raw-ns measurements.
3. Mint `XADC_LIVE_W456_OPERATING_POINT` theorem in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
4. Add a regression gate that rejects fixture drift beyond the documented PVT
   envelope.

### Acceptance
- At least one successful live sweep completes and its fixture is committed.
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test` remains 576/576 non-smoke PASS with FPGA smoke gate OK.

---

## Variant B — Compiler backend hardening (default)

Execute when the bench is still blocked. This is the most likely W456 path.

### Goal
Close the remaining `gen-verilog` / conformance gaps that are now the weakest
points after the 7-failure backlog is gone.

### Scope
1. **RAM style inference.** Add pragma-driven block-vs-distributed RAM lowering
   for module-level `var mem : [N]T` and verify with yosys + Vivado/OpenXC7
   inference checks.
2. **Module-level array parameters.** Support `fn foo(mem : [N]T)` array
   parameter passing and indexing without lowering to a scalar.
3. **ROM read-only enforcement.** Emit `$readmemh`-style or `localparam` ROM
   patterns and add a test that writes to a `const` ROM are rejected at
   typecheck or emit no write path in Verilog.
4. **Warning hygiene.** Address non-fatal yosys warnings on `arch.t27` real-value
   conversions and `translate_off` hot comments where feasible without breaking
   existing semantics.
5. Add at least two new scratch regression specs covering (a) parameterized
   module-level RAM and (b) ROM read-only behavior.

### Acceptance
- `./scripts/tri test` reports 0 failures and `ACCEPTABLE: yes`.
- Gen-verilog yosys smoke gate remains 0 failures.
- New scratch specs pass `t27c gen-verilog` + `yosys read_verilog -sv`.
- All affected seals resealed.

---

## Variant C — Formal boot-evidence fallback

Execute if Variant B is blocked by a compiler refactor that cannot be completed
safely in one wave.

### Goal
Extend the board-less Lean 4 boot-evidence lattice using the now-cleared
`gen-verilog` backend as a trusted generator.

### Scope
1. **Synthesizability theorem block.** Add theorems in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` that state the 7 previously failing
   specs now produce yosys-clean Verilog, expressed as propositions over the
   generated artifact metadata (seal hashes + yosys smoke report).
2. **Adversarial clock-jitter envelope.** Quantify worst-case raw-ns predicate
   preservation under ±2 ns bounded jitter across all OSCFSEL selections and
   all four PVT corners.
3. **Compiler-correctness bridge lemma.** Relate the cleared `let` / tuple /
   ROM / array backend to the abstract ternary MAC semantics in
   `TernaryInference.lean`.
4. Add matching Rust unit tests in `cli/tri/src/fpga.rs`.

### Acceptance
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test` remains 576/576 non-smoke PASS with FPGA smoke gate OK.
- At least 3 new Lean theorems and 3 new Rust unit tests land.

---

## Recommended selection order

1. **Variant A** if hardware becomes available during W456 start-of-wave probe.
2. **Variant B** otherwise — it is the natural continuation after clearing the
   yosys smoke backlog.
3. **Variant C** only if Variant B hits an unresolvable compiler refactor blocker.

---

*φ² + φ⁻² = 3 | TRINITY*
