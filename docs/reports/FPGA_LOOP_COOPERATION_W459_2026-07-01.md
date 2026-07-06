# FPGA Loop Cooperation Plan — Wave Loop 459 (2026-07-01)

**Issue:** #1431
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context at the end of W458

Wave Loop 458 closed #1429 by selecting Variant B: `gen-verilog` backend
hardening. The wave added (1) module-level array access from functions and a
minimal array-parameter binding pass, and (2) yosys warning hygiene
(`` `ifndef SIMULATION `` guards, `parameter real` for `f32`/`f64`, and escaped
string literals). The `--fast` suite path is green: 581/581 non-smoke PASS,
61/61 yosys smoke PASS, FPGA smoke gate OK, 0 seal mismatches.

The physical bench remains blocked: `dlc10 idcode` reports "DLC10 cable not found
(VID=0x03FD)", P12 is unwired, and no automated cold-POR relay gate exists.
The default `./scripts/tri test` also cannot complete in this environment
because Phase 3c-standalone stalls while `lake` downloads the `batteries`
dependency from `reservoir.lean-lang.org`; the smoke-gate report itself passes.

This cooperation plan proposes three mutually-exclusive W459 execution
strategies. Select **one** at the start of the wave.

---

## Variant A — Live CCLK capture if the bench unblocks

Execute only if the DLC10 cable is found **and** P12 + relay are wired.

### Goal
Run the first live cold-POR CCLK sweep since W434 and mint a hardware-backed
 theorem fixture set under the post-W458 bitstream.

### Scope
1. Run `tri fpga cclk-sweep --json` across OSCFSEL 0–7 with the current
   post-W458 bitstream.
2. Persist fixtures under `tests/fixtures/fpga/theorem-matrix/live-w459/` with
   PVT context and raw-ns measurements.
3. Mint `XADC_LIVE_W459_OPERATING_POINT` theorem in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
4. Add a regression gate that rejects fixture drift beyond the documented PVT
   envelope.

### Acceptance
- At least one successful live sweep completes and its fixture is committed.
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test --fast` remains 581/581 non-smoke PASS with yosys smoke OK.

---

## Variant B — Compiler backend hardening: complete array parameters + warning gate (default)

Execute when the bench is still blocked. This is the most likely W459 path.

### Goal
Generalize the W458 array-parameter work so functions with array parameters can
be exercised from test/invariant/bench blocks, and add a known-warnings gate so
that the yosys smoke baseline stays clean.

### Scope
1. **Array parameters from any call site.** Extend the binding analysis so a
   function with an array parameter can be called from module-level statements,
   test blocks, invariant blocks, and bench blocks. All call sites must agree on
   the same module-level array identifier. Add scratch specs that pass arrays as
   actual function parameters and test the results with `assert_eq`.
2. **Known-warnings gate.** In `bootstrap/src/suite.rs`, add an allow-list of
   expected yosys warnings (e.g. deep-recursion on the large IGLA specs) and make
   `cmd_gen_verilog_yosys_smoke` fail if any unrecognized warning appears. This
   locks in the W458 hygiene wins.
3. **ROM style pragma (stretch).** Extend the W457 pragma parser to also accept
   `rom_style = "block"` / `rom_style = "distributed"` and emit the corresponding
   attribute on `const [N]T` ROM declarations.
4. Reseal all affected specs.

### Acceptance
- `./scripts/tri test --fast` reports 0 failures and `ACCEPTABLE: yes`.
- New scratch specs pass `t27c gen-verilog` + `yosys read_verilog -sv` and are
  exercised by at least one `assert_eq`.
- At least one unit test verifies array-parameter lowering from a test block.
- Known-warnings gate is documented and active in the smoke runner.

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
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` stating that the W458 regression
   specs (`w458_array_param_read`, `w458_array_param_write`) and the W457
   RAM-style specs produce yosys-clean Verilog, expressed over seal hashes and
   the yosys smoke report.
2. **Adversarial clock-jitter envelope.** Quantify worst-case raw-ns predicate
   preservation under ±2 ns bounded jitter across all OSCFSEL selections and
   all four PVT corners.
3. **Compiler-correctness bridge lemma.** Relate the cleared translate-off / real
   / string-literal backend to the abstract ternary MAC semantics in
   `TernaryInference.lean`.
4. Add matching Rust unit tests in `cli/tri/src/fpga.rs`.

### Acceptance
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test --fast` remains 581/581 non-smoke PASS with yosys smoke OK.
- At least 3 new Lean theorems and 3 new Rust unit tests land.

---

## Recommended selection order

1. **Variant A** if hardware becomes available during the W459 start-of-wave probe.
2. **Variant B** otherwise — it is the natural continuation of the compiler
   hardening line after W455/W456/W457/W458.
3. **Variant C** only if Variant B hits an unresolvable parser/AST scope blocker.

---

*φ² + φ⁻² = 3 | TRINITY*
