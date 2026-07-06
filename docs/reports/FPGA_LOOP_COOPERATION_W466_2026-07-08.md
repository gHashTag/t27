# FPGA Loop Cooperation Plan — Wave Loop 466 (2026-07-08)

**Issue:** #1444 (to create from W465 land commit)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context at the end of W465

Wave Loop 465 closed #1443 by selecting Variant B: compiler-backend hardening.
The wave extended W464 struct-array lowering so that function-local and
bench-local arrays whose element type is a struct emit per-element per-field
registers, field access on numeric indices resolves correctly, generated names
remain keyword-safe, and identical struct-literal array arguments across call
sites share a single anonymous ROM set. The `--fast` suite path is green:
599/599 non-smoke PASS, 79/79 yosys smoke PASS.

The physical bench remains blocked: `dlc10 idcode` reports "DLC10 cable not
found (VID=0x03FD)", P12 is unwired, and no automated cold-POR relay gate exists.
The default `./scripts/tri test` still cannot complete in this environment
because Phase 3c-standalone stalls while `lake` downloads the `batteries`
dependency from `reservoir.lean-lang.org`; the smoke-gate report itself passes.

This cooperation plan proposes three mutually-exclusive W466 execution
strategies. Select **one** at the start of the wave.

---

## Variant A — Live CCLK capture if the bench unblocks

Execute only if the DLC10 cable is found **and** P12 + relay are wired.

### Goal
Run the first live cold-POR CCLK sweep since W434 and mint a hardware-backed
theorem fixture set under the post-W465 bitstream.

### Scope
1. Run `tri fpga cclk-sweep --json` across OSCFSEL 0–7 with the current
   post-W465 bitstream.
2. Persist fixtures under `tests/fixtures/fpga/theorem-matrix/live-w466/` with
   PVT context and raw-ns measurements.
3. Mint `XADC_LIVE_W466_OPERATING_POINT` theorem in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
4. Add a regression gate that rejects fixture drift beyond the documented PVT
   envelope.

### Acceptance
- At least one successful live sweep completes and its fixture is committed.
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test --fast` remains 599/599 non-smoke PASS with yosys smoke OK.

---

## Variant B — Compiler backend hardening: variable-index local arrays of structs + struct-array assignment (default)

Execute when the bench is still blocked. This is the most likely W466 path.

### Goal
Extend the W465 struct-array lowering to support variable-index field access
on local arrays of structs and whole-element assignment to a local array slot.

### Scope
1. **Variable-index field access on local arrays of structs.** Allow
   `pts[i].x` where `i` is a variable by emitting a priority mux chain over
   the per-element per-field registers (`pts_0_x`, `pts_1_x`, ...) selected by
   `i`.
2. **Whole-element assignment to a local array slot.** Allow
   `pts[0] = Pt{.x = ..., .y = ...}` by assigning all fields of the selected
   element.
3. **Variable-index whole-element assignment.** Allow `pts[i] = Pt{...}` by
   emitting an if-else chain that writes each field register when the index
   matches.
4. Add regression specs for each new path and reseal affected specs.

### Acceptance
- `./scripts/tri test --fast` reports 0 failures and `ACCEPTABLE: yes`.
- New scratch specs pass parse, typecheck, gen-verilog, yosys smoke.
- `cargo test -p t27c --bin t27c` remains 1524 passed, 0 failed, 2 ignored.

---

## Variant C — Board-less formal fallback

Execute if Variant B is blocked by a scope/refactor that cannot be completed
safely in one wave.

### Goal
Extend the board-less Lean 4 boot-evidence lattice with theorems that witness
the correctness and robustness of the W465 compiler-backend additions.

### Scope
1. **Synthesizability theorem for W465 struct-array lowering.** Prove in Lean 4
   that the per-element per-field register lowering preserves the field-access
   semantics for numeric indices.
2. **Multi-site literal-deduplication witness.** Add a Rust unit test that
   asserts identical struct-literal array arguments across call sites produce
   exactly one anonymous ROM set in the generated Verilog.
3. **Adversarial keyword-field-name escape theorem.** Add a regression spec and
   proof sketch showing that struct fields named `reg` / `wire` do not collide
   with Verilog keywords after the W465 single-token escape convention.

### Acceptance
- `./scripts/tri test --fast` remains 599/599 non-smoke PASS with yosys smoke OK.
- New Lean theorems build with `lake build Trinity.TernaryFPGABoot`.
- `cargo test -p t27c --bin t27c` remains 1524 passed, 0 failed, 2 ignored.

---

## Recommendation

Select **Variant B** unless the bench becomes available, in which case switch to
**Variant A**. Variant B keeps the compiler-backend hardening line moving on a
well-bounded, regression-safe surface. Variant C is a fallback only if Variant B
is blocked by a larger refactor.

---

*φ² + φ⁻² = 3 | TRINITY*
