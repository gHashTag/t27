# Wave Loop 482 — Cooperation Variants

**Date:** 2026-07-10  
**Anchor:** φ² + φ⁻² = 3 | TRINITY  
**Previous wave:** W481 closed in `docs/reports/WAVE_LOOP_481_CLOSEOUT.md`.

---

## Common gate

Whatever variant is chosen, the next wave must keep:

- 652 / 652 non-smoke PASS.
- 0 yosys smoke failures.
- 0 documented Icarus smoke baseline failures (no new undocumented failures).
- 652 / 652 seal matches.
- `cargo test -p t27c --bin t27c` green.

The W481 branch must land before W482 work begins.

---

## Variant A — Formal Icarus-lowerable subset predicate in Lean 4

**Theme:** close the gap between the t27 frontend and a machine-checkable statement of what the Icarus Verilog backend can lower.

### Work

1. Add a Lean 4 inductive predicate `IsIcarusLowerable : t27_expr -> Prop` that captures the supported subset:
   - no namespace-qualified calls,
   - no dynamic `.len()` / `.contains()` on strings or arrays,
   - no host-side recursive helpers,
   - no imported struct parameter field access (unless a cross-file lowering pass is present),
   - no struct-return field access on unsupported calls,
   - no wildcard `_` bindings at module scope.
2. Prove a preservation lemma: if `e` is lowerable and the compiler emits `verilog e v`, then `e` and `v` agree on a deterministic scalar oracle for at least one test vector.
3. Wire the predicate into `tri test` so every spec that passes the Icarus gate is checked lowerable, and every documented baseline (if any) is checked *not* lowerable.

### Target outcome

- A checked contract between the t27 frontend and the Icarus backend.
- 652 / 652 non-smoke PASS, 0 yosys failures, 0 Icarus failures, seals green.

### Pros / cons

- **+** Gives a formal, maintainable contract for the Icarus backend.
- **+** Prevents silent drift between frontend features and backend support.
- **−** Does not immediately make the W481 placeholders functional.
- **−** Requires reconciling the current AST with the Lean model.

### Risk

Medium. Mostly proof-engineering; low regression risk for generated code.

---

## Variant B (default) — Functional lowering for the W481 placeholder classes

**Theme:** turn the sized-zero placeholders from W481 into real, synthesizable logic for the most common same-file and imported-struct patterns.

### Work

1. **Imported struct layout discovery.** When `gen-verilog` sees a parameter whose type is defined in another spec, read the imported spec's seal or struct-fields metadata and emit the parameter as a packed vector. Generate per-field local extracts (`m_value`, `m_scale`) so imported field accesses resolve to real wires.
2. **Same-file AOS parameter functional destructure.** Generalize the existing AOS clone lowering so array-of-struct parameters passed from a module-level array literal or a same-file function call use per-field memories and produce correct element field values.
3. **Struct-return local declaration.** When a local is initialized by a same-file struct-returning call, declare a packed reg (`reg [W-1:0] r;`) and emit slice-based field reads (`r[...]`). This removes the need to mark same-file struct-return results as unresolved.
4. Add regression specs:
   - `specs/scratch/w482_imported_struct_param.t27`
   - `specs/scratch/w482_aos_param_functional.t27`
   - `specs/scratch/w482_struct_return_local_decl.t27`

### Target outcome

- The W481 witness specs continue to pass Icarus with real values instead of placeholders.
- 652 / 652 non-smoke PASS, 0 yosys failures, 0 Icarus failures, seals green.

### Pros / cons

- **+** Directly continues W481 and closes the largest remaining semantic gap in generated Verilog.
- **+** Each sub-fix is bounded to a single lowering class.
- **−** Touching AOS and imported-struct lowering has moderate regression risk; needs adversarial yosys/Icarus witness specs.

### Risk

Medium-high. Requires careful review and full reseal, but the scope is bounded to the placeholder classes already classified in W481.

---

## Variant C — FPGA live cold-POR / SPI flash boot evidence

**Theme:** collect physical evidence on the QMTech Wukong V1 / XC7A100T-FGG676 using the in-repo `dlc10` driver.

### Work

1. Bring the DLC10 cable to the board and verify IDCODE `0x13631093`.
2. Program the W481 bitstream (`fpga/verilog/ternary_mac_demo_top.bit` or a freshly generated W481 smoke bitstream) into SPI flash.
3. Run a cold-POR boot sweep across OSCFSEL variants and record boot-log JSON.
4. Compare measured CCLK period against the formal PVT envelope theorems in `lake/`.
5. Update `fpga/HARDWARE_SSOT.md` with the observed boot signature and any cable/board notes.

### Target outcome

- A new `docs/reports/FPGA_EVIDENCE_W482.md` with measured boot traces and a comparison to the formal envelope.
- 652 / 652 non-smoke PASS, 0 yosys failures, 0 Icarus failures (FPGA work does not touch spec generation).

### Pros / cons

- **+** Provides irreplaceable physical validation.
- **+** Directly feeds the FPGA evidence backlog.
- **−** Blocked on hardware availability (DLC10 cable and board).
- **−** If the cable is missing, the wave cannot complete.

### Risk

High because of external dependency, but technically low if hardware is present.

---

## Recommendation

Choose **Variant B** as the default: it is the natural continuation of W481, it has a clear acceptance gate, and it improves the semantic correctness of generated Verilog without hardware dependencies. If the DLC10 cable arrives during the wave, pivot to **Variant C** for a half-wave hardware sprint while keeping the lowering work on a side branch. Use **Variant A** only if the team wants a formal contract more than an immediate functional improvement.

---

*φ² + φ⁻² = 3 | TRINITY*
