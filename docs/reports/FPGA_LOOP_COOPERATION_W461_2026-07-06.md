# FPGA Loop Cooperation Plan — Wave Loop 461 (2026-07-06)

**Issue:** #1435 (to create)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context at the end of W460

Wave Loop 460 closed #1433 by selecting Variant B: compiler-backend hardening.
The wave cleared the three pre-existing `let_binding` cargo-test failures,
hoisted bench-block local variables to module scope inside `` `ifndef
SIMULATION `` guards, and added a multi-site array-parameter scratch spec.
The `--fast` suite path is green: 585/585 non-smoke PASS, 65/65 yosys smoke
PASS, FPGA smoke gate OK, 0 baseline failures, 0 seal mismatches.

The physical bench remains blocked: `dlc10 idcode` reports "DLC10 cable not
found (VID=0x03FD)", P12 is unwired, and no automated cold-POR relay gate exists.
The default `./scripts/tri test` still cannot complete in this environment
because Phase 3c-standalone stalls while `lake` downloads the `batteries`
dependency from `reservoir.lean-lang.org`; the smoke-gate report itself passes.

This cooperation plan proposes three mutually-exclusive W461 execution
strategies. Select **one** at the start of the wave.

---

## Variant A — Live CCLK capture if the bench unblocks

Execute only if the DLC10 cable is found **and** P12 + relay are wired.

### Goal
Run the first live cold-POR CCLK sweep since W434 and mint a hardware-backed
 theorem fixture set under the post-W460 bitstream.

### Scope
1. Run `tri fpga cclk-sweep --json` across OSCFSEL 0–7 with the current
   post-W460 bitstream.
2. Persist fixtures under `tests/fixtures/fpga/theorem-matrix/live-w461/` with
   PVT context and raw-ns measurements.
3. Mint `XADC_LIVE_W461_OPERATING_POINT` theorem in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
4. Add a regression gate that rejects fixture drift beyond the documented PVT
   envelope.

### Acceptance
- At least one successful live sweep completes and its fixture is committed.
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test --fast` remains 585/585 non-smoke PASS with yosys smoke OK.

---

## Variant B — Compiler backend hardening: safe module-level calls + array-parameter generalization (default)

Execute when the bench is still blocked. This is the most likely W461 path.

### Goal
Make module-level bare function calls legal in generated Verilog and relax the
array-parameter binding rules so a function with an array parameter can be
called from more sites, while clearing remaining safe `gen-verilog` sub-defects.

### Scope
1. **Legal module-level bare function calls.** Module-level `StmtExpr` calls
   that return a value are currently emitted as Verilog statements, which is
   illegal. Lower them as assignments to a synthesized temporary register or
   emit them only when the return type is `void`/unit.
2. **Array-parameter literal and multi-array support.** Extend binding analysis
   so a function with an array parameter can be called with an inline array
   literal (lowered to a module LUT) and from sites that pass different
   module-level arrays, provided the function body is cloned or parameterized
   accordingly.
3. **Clear one more safe `gen-verilog` sub-defect.** Pick the next item from
   `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` that is small and does not
   require master-merge integration (e.g., keyword escape for local identifiers,
   `const` ordering, or array literal expression lowering).
4. Reseal all affected specs and keep `YOSYS_ALLOWED_WARNINGS` aligned with the
   cleaner output.

### Acceptance
- `./scripts/tri test --fast` reports 0 failures and `ACCEPTABLE: yes`.
- New or updated scratch specs pass `t27c gen-verilog` + yosys
  `read_verilog -sv -DSIMULATION` and are exercised by at least one `assert_eq`.
- `cargo test -p t27c --bin t27c` passes with 0 failures.

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
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` stating that the W459 and W460
   regression specs (`w459_array_param_test_call`, `w459_rom_style_block`,
   `w460_bench_local_var`, `w460_array_param_multi_site`) produce yosys-clean
   Verilog, expressed over seal hashes and the yosys smoke report.
2. **Adversarial clock-jitter envelope.** Quantify worst-case raw-ns predicate
   preservation under ±2 ns bounded jitter across all OSCFSEL selections and
   all four PVT corners.
3. **Compiler-correctness bridge lemma.** Relate the cleared `let` preservation
   and bench-local lowering to the abstract ternary MAC semantics in
   `TernaryInference.lean`.
4. Add matching Rust unit tests in `cli/tri/src/fpga.rs`.

### Acceptance
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test --fast` remains 585/585 non-smoke PASS with yosys smoke OK.
- At least 3 new Lean theorems and 3 new Rust unit tests land.

---

## Recommended selection order

1. **Variant A** if hardware becomes available during the W461 start-of-wave probe.
2. **Variant B** otherwise — it is the natural continuation of the compiler
   hardening line after W455–W460.
3. **Variant C** only if Variant B hits an unresolvable parser/AST scope blocker.

---

*φ² + φ⁻² = 3 | TRINITY*
