# Wave Loop 465 Plan — Issue #1443

**Branch:** `wave-loop-465`  
**Issue:** #1443  
**Anchor:** φ² + φ⁻² = 3 | TRINITY  
**Date:** 2026-07-08  

## Current context

Wave Loop 464 closed #1441 with three `gen-verilog` array-parameter clone
extensions: mixed direct/indirect call-site merging, struct-literal array
arguments lowered to per-field memories, and a deterministic clone-name collision
guard. The fast suite is green: 594/594 non-smoke PASS, 74/74 yosys smoke PASS.

The physical bench is still blocked (DLC10 cable not found, P12 unwired), so
Variant B from the W465 cooperation plan is the default execution path.

## Weak points discovered

1. **Function-local arrays of structs are not lowered.**
   `var pts : [3]Pt` inside a function is emitted as scalar 32-bit regs
   (`pts_0`, `pts_1`, `pts_2`) because `type_to_width("Pt")` defaults to 32.
   Field access `pts[0].x` resolves to `pts_x` instead of `pts_0_x`, so the
   generated Verilog references non-existent identifiers. Initializers such as
   `var pts : [3]Pt = [3]Pt{...}` degrade to a TODO comment.

2. **Bench-local arrays of structs share the same gap.** The hoisted
   declaration path (`gen_verilog_local_decl_hoisted` / `gen_verilog_local_assign`)
   uses the same scalar per-element logic and will emit incorrect registers for
   struct element types.

3. **Field-memory name keyword safety is verified only for module-level ROMs.**
   The current `verilog_safe_identifier` only escapes exact keyword matches;
   generated names like `words_reg` / `words_wire` are valid single tokens and
   already pass yosys, but function-local struct-array reg names must be
   consistently escaped to preserve this property when the base or field name is
   a keyword.

4. **Multi-site struct-literal array arguments already deduplicate**, but there
   is no regression spec that locks this behavior. The binding pass stores anon
   ROMs keyed by canonical signature, so identical literals across call sites
   share one per-field memory; a dedicated spec prevents silent regression.

## Competitor snapshot

- **Sparkle / Verilean:** last public push 2026-07-03; PR #66 (IP.Net + compiler
  perf) and the FIDO2/crypto burst (PR #97–#100, merged 2026-07-04) remain the
  freshest public signals. No new Lean-native ternary competitor appeared.
- **CIRCT / firtool:** `firtool-1.152.0` (2026-07-04) is still the latest public
  release; no `1.153.0` exists.
- **Ternary-FPGA niche:** continues to validate {-1, 0, +1} compute hardware
  without a Lean-native proof pipeline.

Trinity's differentiation (Lean-native proof + sealed `*.t27 → gen/` pipeline +
physical boot-evidence instrumentation) remains intact. W465 is a compiler
quality wave and does not change the competitive boundary.

## Decomposed work

### Task 1 — Function-local arrays of structs
- Extend `StmtLocal` lowering in `gen_verilog_stmt` to detect struct element
  types and emit per-element per-field registers (`{base}_{i}_{field}`).
- Emit initializers from `ExprArrayLiteral` of `ExprStructLit` children as
  per-field scalar assignments.
- Add a `local_array_elem_types` registry so `gen_verilog_expr` can resolve
  indexed field access on local arrays of structs.
- Extend `ExprFieldAccess` to emit `{base}_{idx}_{field}` when the base is a
  function-local array whose element type is a struct.
- Regression spec: `specs/scratch/w465_local_struct_array.t27`.

### Task 2 — Bench-local arrays of structs
- Extend `gen_verilog_local_decl_hoisted` and `gen_verilog_local_assign` with
  the same per-element per-field register emission used for function-local
  arrays.
- Regression spec: `specs/scratch/w465_bench_local_struct_array.t27`.

### Task 3 — Keyword-safe field-memory names
- Ensure every newly generated field register name is passed through
  `verilog_safe_identifier` as a single token.
- Add a regression spec with struct fields named `reg` and `wire` for both
  module-level and function-local arrays of structs; verify yosys clean.
- Regression spec: `specs/scratch/w465_keyword_field_local_struct_array.t27`.

### Task 4 — Multi-site struct-literal array arguments
- Add a regression spec that passes the same struct-literal array to two
  different functions from two call sites and asserts only one anonymous ROM set
  is emitted in the generated Verilog.
- Regression spec: `specs/scratch/w465_multi_site_struct_array_literal.t27`.

### Task 5 — Documentation and close-out
- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with W465 triage.
- Update `docs/reports/T27_VS_FORMAL_HDL_2026.md` with the W465 competitor
  boundary paragraph.
- Update `docs/NOW.md`.
- Write `docs/reports/WAVE_LOOP_465_REPORT.md`.
- Write `docs/reports/FPGA_LOOP_EVIDENCE_W465_2026-07-08.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W466_2026-07-08.md` with three
  variants for W466.
- Save memory entry for W465 and update `MEMORY.md`.

## Risks

- **Scope creep:** variable-index local arrays of structs would require priority
  muxes for field access and are intentionally out of scope for W465; keep to
  numeric-literal indices.
- **Seal churn:** adding new scratch specs and changing local-array lowering may
  reseal existing specs. Reseal only legitimate output changes.
- **Bench still blocked:** Variant A is not executable; do not attempt hardware
  access.

## Acceptance criteria

- `./scripts/tri test --fast` reports 0 failures, `ACCEPTABLE: yes`.
- New scratch specs pass parse, typecheck, gen-verilog, yosys smoke.
- `cargo test -p t27c --bin t27c` remains 1524 passed, 0 failed, 2 ignored.
- Close-out artifacts (report, evidence, W466 cooperation plan) are committed
  and pushed.

## Cooperation variants for Wave Loop 466

1. **Variant A — Live CCLK capture if the bench unblocks.** Run a live cold-POR
   CCLK sweep on Wukong XC7A100T, persist fixtures under
   `tests/fixtures/fpga/theorem-matrix/live-w466/`, mint `XADC_LIVE_W466_OPERATING_POINT`
   theorem.
2. **Variant B (default) — Compiler backend hardening: variable-index local
   arrays of structs + struct-array assignment.** Extend W465 to support
   `pts[i].x` where `i` is a variable by emitting priority muxes over per-field
   registers, and allow whole-element assignment (`pts[0] = Pt{.x=..., .y=...}`).
3. **Variant C (fallback) — Board-less formal fallback.** If Variant B is blocked,
   add a Lean 4 synthesizability theorem for W465 struct-array lowering, a
   multi-site literal-deduplication witness, and an adversarial keyword-field-name
   escape theorem.
