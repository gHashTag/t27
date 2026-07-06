# FPGA Loop Cooperation Plan — Wave Loop 457 (2026-07-01)

**Issue:** #1428 (to be created)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context at the end of W456

Wave Loop 456 closed #1427 by adding ROM read-only enforcement in the typechecker
and a regression spec. Full `./scripts/tri test` remains green (577/577
non-smoke PASS, 57/57 yosys smoke PASS, FPGA smoke gate OK, 0 seal mismatches).
The physical bench is still blocked.

This cooperation plan proposes three mutually-exclusive W457 execution strategies.
Select **one** at the start of the wave.

---

## Variant A — Live CCLK capture if the bench unblocks

Execute only if the DLC10 cable is found **and** P12 + relay are wired.

### Goal
Run the first live cold-POR CCLK sweep since W434 and mint a hardware-backed
 theorem fixture set.

### Scope
1. Run `tri fpga cclk-sweep --json` across OSCFSEL 0–7 with the current
   post-W456 bitstream.
2. Persist fixtures under `tests/fixtures/fpga/theorem-matrix/live-w457/` with
   PVT context and raw-ns measurements.
3. Mint `XADC_LIVE_W457_OPERATING_POINT` theorem in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
4. Add a regression gate that rejects fixture drift beyond the documented PVT
   envelope.

### Acceptance
- At least one successful live sweep completes and its fixture is committed.
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test` remains 577/577 non-smoke PASS with FPGA smoke gate OK.

---

## Variant B — Compiler backend hardening: RAM style pragmas (default)

Execute when the bench is still blocked. This is the most likely W457 path.

### Goal
Add synthesizer-controllable RAM style attributes for module-level arrays, so
`var mem : [N]T` can be directed to block or distributed RAM and the choice is
covered by regression tests and yosys inference checks.

### Scope
1. **Syntax:** introduce a pragma form such as `#[ram_style("block")]` or
   `pragma ram_style = "block";` attached to module-level `var` array
   declarations. Update the parser to preserve the pragma in the AST.
2. **Backend:** in `gen_verilog_var`, emit the appropriate Verilog attribute:
   - `(* ram_style = "block" *) reg [W-1:0] mem [0:N-1];`
   - `(* ram_style = "distributed" *)` for the alternative.
   - Default (no pragma) keeps current output.
3. **Regression specs:** add at least two scratch specs:
   - `w457_ram_style_block.t27`
   - `w457_ram_style_distributed.t27`
4. **Inference check:** extend the yosys smoke gate to assert that the emitted
   attribute is preserved (e.g., `yosys read_verilog -sv` + `synth_xilinx` does
   not strip the attribute).
5. Reseal all affected specs.

### Acceptance
- `./scripts/tri test` reports 0 failures and `ACCEPTABLE: yes`.
- Gen-verilog yosys smoke gate remains 0 failures.
- New scratch specs pass `t27c gen-verilog` + `yosys read_verilog -sv`.
- At least one unit test verifies pragma parsing and emitted attribute.

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
   failing specs and the new W456 ROM spec produce yosys-clean Verilog,
   expressed over seal hashes and the yosys smoke report.
2. **Adversarial clock-jitter envelope.** Quantify worst-case raw-ns predicate
   preservation under ±2 ns bounded jitter across all OSCFSEL selections and
   all four PVT corners.
3. **Compiler-correctness bridge lemma.** Relate the cleared `let` / tuple /
   ROM / array backend to the abstract ternary MAC semantics in
   `TernaryInference.lean`.
4. Add matching Rust unit tests in `cli/tri/src/fpga.rs`.

### Acceptance
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test` remains 577/577 non-smoke PASS with FPGA smoke gate OK.
- At least 3 new Lean theorems and 3 new Rust unit tests land.

---

## Recommended selection order

1. **Variant A** if hardware becomes available during the W457 start-of-wave probe.
2. **Variant B** otherwise — it is the natural continuation of the compiler
   hardening line after W455/W456.
3. **Variant C** only if Variant B hits an unresolvable parser/AST refactor blocker.

---

*φ² + φ⁻² = 3 | TRINITY*
