# FPGA Loop Cooperation Plan — Wave Loop 458 (2026-07-01)

**Issue:** #1429 (to be created)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context at the end of W457

Wave Loop 457 closed #1428 by adding RAM style pragma support for module-level
arrays (`pragma ram_style = "block";` / `pragma ram_style = "distributed";`).
The emitted Verilog `(* ram_style = "..." *)` attribute is preserved through
the yosys smoke gate, and `./scripts/tri test` is green (579/579 non-smoke PASS,
59/59 yosys smoke PASS, FPGA smoke gate OK, 0 seal mismatches).

The physical bench remains blocked: `dlc10 idcode` reports "DLC10 cable not found
(VID=0x03FD)", P12 is unwired, and no automated cold-POR relay gate exists.

This cooperation plan proposes three mutually-exclusive W458 execution
strategies. Select **one** at the start of the wave.

---

## Variant A — Live CCLK capture if the bench unblocks

Execute only if the DLC10 cable is found **and** P12 + relay are wired.

### Goal
Run the first live cold-POR CCLK sweep since W434 and mint a hardware-backed
 theorem fixture set.

### Scope
1. Run `tri fpga cclk-sweep --json` across OSCFSEL 0–7 with the current
   post-W457 bitstream.
2. Persist fixtures under `tests/fixtures/fpga/theorem-matrix/live-w458/` with
   PVT context and raw-ns measurements.
3. Mint `XADC_LIVE_W458_OPERATING_POINT` theorem in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
4. Add a regression gate that rejects fixture drift beyond the documented PVT
   envelope.

### Acceptance
- At least one successful live sweep completes and its fixture is committed.
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test` remains 579/579 non-smoke PASS with FPGA smoke gate OK.

---

## Variant B — Compiler backend hardening: module-level array parameters + warning hygiene (default)

Execute when the bench is still blocked. This is the most likely W458 path.

### Goal
Close the next two `gen-verilog` debt items now that RAM/ROM array lowering and
RAM style pragmas are in place: (1) passing module-level arrays as function
parameters, and (2) cleaning the remaining yosys translate-off / real-value
warnings in the smoke gate so the baseline becomes warning-free.

### Scope
1. **Module-level array parameters.** Extend the parser and Verilog backend so a
   `pub fn` can accept a module-level `var mem : [N]T` (or `const [N]T`) as an
   argument and lower it to a memory port interface (`input [A-1:0] addr`,
   `input wen`, `input [W-1:0] wdata`, `output [W-1:0] rdata`) instead of
   value-copying the whole array.
2. **Regression specs.** Add at least two scratch specs:
   - `w458_array_param_read.t27` — function reads from a passed module array.
   - `w458_array_param_write.t27` — function writes to a passed module array.
3. **Warning hygiene.** Audit the 59 yosys smoke targets; for each remaining
   `translate_off` / real-value warning, either fix the generator or add a
   documented, reviewed waiver so the smoke gate can assert
   `warnings == known_waivers`.
4. **ROM style pragma (optional stretch).** If scope allows, extend the W457
   pragma parser to also accept `rom_style = "block"` / `rom_style = "distributed"`
   and emit the corresponding attribute on `const [N]T` ROM declarations.
5. Reseal all affected specs.

### Acceptance
- `./scripts/tri test` reports 0 failures and `ACCEPTABLE: yes`.
- Gen-verilog yosys smoke gate remains 0 failures.
- New scratch specs pass `t27c gen-verilog` + `yosys read_verilog -sv`.
- At least one unit test verifies array-parameter lowering.

---

## Variant C — Formal boot-evidence fallback

Execute if Variant B is blocked by a parser/AST refactor that cannot be
completed safely in one wave.

### Goal
Extend the board-less Lean 4 boot-evidence lattice using the now-cleared
`gen-verilog` backend as a trusted generator.

### Scope
1. **Synthesizability theorem block.** Add propositions in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` stating that the 7 previously
   failing specs, the W456 ROM spec, and the W457 RAM-style specs produce
   yosys-clean Verilog, expressed over seal hashes and the yosys smoke report.
2. **Adversarial clock-jitter envelope.** Quantify worst-case raw-ns predicate
   preservation under ±2 ns bounded jitter across all OSCFSEL selections and
   all four PVT corners.
3. **Compiler-correctness bridge lemma.** Relate the cleared `let` / tuple /
   ROM / array / RAM-style backend to the abstract ternary MAC semantics in
   `TernaryInference.lean`.
4. Add matching Rust unit tests in `cli/tri/src/fpga.rs`.

### Acceptance
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test` remains 579/579 non-smoke PASS with FPGA smoke gate OK.
- At least 3 new Lean theorems and 3 new Rust unit tests land.

---

## Recommended selection order

1. **Variant A** if hardware becomes available during the W458 start-of-wave probe.
2. **Variant B** otherwise — it is the natural continuation of the compiler
   hardening line after W455/W456/W457.
3. **Variant C** only if Variant B hits an unresolvable parser/AST refactor blocker.

---

*φ² + φ⁻² = 3 | TRINITY*
