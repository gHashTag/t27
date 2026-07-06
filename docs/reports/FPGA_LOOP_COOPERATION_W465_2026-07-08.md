# FPGA Loop Cooperation Plan — Wave Loop 465 (2026-07-08)

**Issue:** #1443 (to create from W464 land commit)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context at the end of W464

Wave Loop 464 closed #1441 by selecting Variant B: compiler-backend hardening.
The wave extended the W463 array-parameter clone machinery so that functions with
both direct and indirect array-parameter call sites are merged correctly,
struct-literal array arguments lower to per-field Verilog memories, and
clone-name assignment is deterministic and collision-guarded. The `--fast`
suite path is green: 594/594 non-smoke PASS, 74/74 yosys smoke PASS, FPGA smoke
gate OK, 0 baseline failures, 0 seal mismatches.

The physical bench remains blocked: `dlc10 idcode` reports "DLC10 cable not
found (VID=0x03FD)", P12 is unwired, and no automated cold-POR relay gate exists.
The default `./scripts/tri test` still cannot complete in this environment
because Phase 3c-standalone stalls while `lake` downloads the `batteries`
dependency from `reservoir.lean-lang.org`; the smoke-gate report itself passes.

This cooperation plan proposes three mutually-exclusive W465 execution
strategies. Select **one** at the start of the wave.

---

## Variant A — Live CCLK capture if the bench unblocks

Execute only if the DLC10 cable is found **and** P12 + relay are wired.

### Goal
Run the first live cold-POR CCLK sweep since W434 and mint a hardware-backed
theorem fixture set under the post-W464 bitstream.

### Scope
1. Run `tri fpga cclk-sweep --json` across OSCFSEL 0–7 with the current
   post-W464 bitstream.
2. Persist fixtures under `tests/fixtures/fpga/theorem-matrix/live-w465/` with
   PVT context and raw-ns measurements.
3. Mint `XADC_LIVE_W465_OPERATING_POINT` theorem in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
4. Add a regression gate that rejects fixture drift beyond the documented PVT
   envelope.

### Acceptance
- At least one successful live sweep completes and its fixture is committed.
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test --fast` remains 594/594 non-smoke PASS with yosys smoke OK.

---

## Variant B — Compiler backend hardening: local arrays of structs + keyword-safe field-memory names + multi-site struct literals (default)

Execute when the bench is still blocked. This is the most likely W465 path.

### Goal
Extend the W464 struct-array lowering to function-local arrays of structs,
ensure generated field-memory names remain keyword-safe, and allow the same
struct-literal array to be passed from multiple call sites without duplicate ROM
emission.

### Scope
1. **Function-local arrays of structs.** Allow `const local_pts : [3]Pt = ...`
   or `var local_pts : [3]Pt = ...` inside a function and lower them to
   per-element field registers (`local_pts_0_x`, `local_pts_0_y`, ...) or a
   function-local field memory, matching the existing local-array lowering for
   scalar arrays.
2. **Keyword-safe field-memory names.** Apply `verilog_safe_identifier` to the
   `{base}_{field}` combination used in `gen_verilog_const`,
   `gen_verilog_anon_rom`, and the `ExprFieldAccess` array-of-struct path so that
   struct fields whose names collide with Verilog keywords are escaped.
3. **Multi-site struct-literal array arguments.** Ensure that when the same
   struct-literal array is passed to two call sites, the deterministic signature
   key is shared and only one set of per-field ROMs is emitted.
4. Add regression specs for each new path and reseal affected specs.

### Acceptance
- `./scripts/tri test --fast` reports 0 failures and `ACCEPTABLE: yes`.
- New or updated scratch specs pass `t27c gen-verilog` + yosys
  `read_verilog -sv -DSIMULATION` and are exercised by at least one `assert_eq`.
- `cargo test -p t27c --bin t27c` passes with 0 failures.

---

## Variant C — Formal boot-evidence fallback

Execute if Variant B is blocked by a scope/AST refactor that cannot be completed
safely in one wave.

### Goal
Extend the board-less Lean 4 boot-evidence lattice with synthesizability and
correctness bridge statements that cover the new W464 mixed-call-site and
struct-array paths.

### Scope
1. **Synthesizability theorem block.** Add propositions in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` (or a dedicated
   `TernaryGenVerilog.lean` stub) stating that the W464 regression specs
   (`w464_struct_array_literal.t27` and `w464_mixed_array_param_call_site.t27`)
   produce yosys-clean Verilog, and that the emitted clone/field-memory
   structure matches the intended lowering.
2. **Mixed-call-site correctness lemma.** Relate the W464 merge rule to the
   abstract ternary MAC semantics in `TernaryInference.lean`: if `g(data)` is
   called both directly and through `f(data)`, both paths resolve to the same
   array-parameter binding semantics.
3. **Struct-array lowering witness.** Add a Lean theorem or Rust unit test
   showing that an array of structs with `N` elements and `F` scalar fields is
   lowered to exactly `F` memories of depth `N`.
4. Add matching Rust unit tests in `cli/tri/src/fpga.rs` or
   `bootstrap/src/suite.rs`.

### Acceptance
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test --fast` remains 594/594 non-smoke PASS with yosys smoke OK.
- At least 3 new Lean theorems and 3 new Rust unit tests land.

---

## Recommended selection order

1. **Variant A** if hardware becomes available during the W465 start-of-wave probe.
2. **Variant B** otherwise — it is the natural continuation of the compiler
   hardening line after W455–W464.
3. **Variant C** only if Variant B hits an unresolvable parser/AST scope blocker.

---

*φ² + φ⁻² = 3 | TRINITY*
