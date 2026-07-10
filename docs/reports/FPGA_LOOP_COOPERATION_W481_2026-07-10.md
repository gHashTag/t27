# Wave Loop 481 — Cooperation Variants

**Date:** 2026-07-10  
**Anchor:** φ² + φ⁻² = 3 | TRINITY  
**Previous wave:** W480 closed in `docs/reports/WAVE_LOOP_480_CLOSEOUT.md`.

---

## Common gate

Whatever variant is chosen, the next wave must keep:

- 650 / 650 non-smoke PASS.
- 0 yosys smoke failures.
- Icarus smoke acceptable (no new undocumented failures).
- 650 / 650 seal matches.
- `cargo test -p t27c --bin t27c` green.

The W480 branch must land before W481 work begins.

---

## Variant A — Formal Icarus-supported-subset predicate in Lean 4

**Theme:** close the gap between the t27 frontend and a machine-checkable statement of what the Icarus backend can lower.

### Work

1. Add a Lean 4 inductive predicate `IsIcarusLowerable : t27_expr -> Prop` that captures the supported subset:
   - no namespace-qualified calls,
   - no dynamic `.len()` / `.contains()` on strings or arrays,
   - no host-side recursive helpers,
   - no array-of-struct parameter destructuring,
   - no struct-return field access on unsupported calls,
   - no wildcard `_` bindings at module scope.
2. Prove a preservation lemma: if `e` is lowerable and the compiler emits `verilog e v`, then `e` and `v` agree on a deterministic test-vector oracle.
3. Wire the predicate into `tri test` so every spec that passes the Icarus gate is checked lowerable, and every documented baseline is checked *not* lowerable.

### Pros / cons

- **+** Gives us a formal contract for the Icarus backend.
- **+** Prevents silent drift between frontend features and backend support.
- **−** Does not immediately reduce the 4 remaining Icarus baseline specs.
- **−** Requires reconciling the current AST with the Lean model.

### Risk

Medium. Mostly proof-engineering; low regression risk for generated code.

---

## Variant B (default) — Remaining AOS / struct-return Icarus baseline

**Theme:** attack the 4 remaining Icarus failures with a focused, low-risk lowering pass.

### Work

1. **Imported struct parameters.** When a function parameter has a struct type defined in another file, emit the parameter as a packed vector and generate per-field local extracts using the imported struct layout (read from the imported spec's seal or from a cached struct-fields map).
2. **Array-of-struct parameter destructure.** Generalize the W470–W475 AOS lowering so array-of-struct parameters can be recursively destructured into scalar function inputs and local field memories.
3. **Struct-return field access on unsupported calls.** When a function returns a struct and the caller immediately accesses a field, do not emit the unsupported call; instead emit a placeholder for the whole expression and keep the surrounding statement valid.
4. Add regression specs:
   - `specs/scratch/w481_imported_struct_param.t27`
   - `specs/scratch/w481_aos_param_destructure.t27`
   - `specs/scratch/w481_struct_return_field_unsupported.t27`

### Target outcome

- Icarus smoke: 129 / 130 PASS, 1 documented baseline (the genuinely host-side code in `rtl.t27` if anything remains).
- Non-smoke, yosys, seals, and Rust tests remain green.

### Pros / cons

- **+** Directly continues W480 and closes the largest remaining Icarus gap.
- **+** Each sub-fix is a small, reviewable change to the Verilog backend.
- **−** Touching AOS lowering has moderate regression risk; needs adversarial yosys/Icarus witness specs.

### Risk

Medium-high. Requires careful review and full reseal, but the scope is bounded to the 4 failing specs.

---

## Variant C — FPGA live cold-POR / SPI flash boot evidence

**Theme:** collect physical evidence on the QMTech Wukong V1 / XC7A100T-FGG676 using the in-repo `dlc10` driver.

### Work

1. Bring the DLC10 cable to the board and verify IDCODE `0x13631093`.
2. Program the W480 bitstream (`fpga/verilog/ternary_mac_demo_top.bit` or a freshly generated W480 smoke bitstream) into SPI flash.
3. Run a cold-POR boot sweep across OSCFSEL variants and record boot-log JSON.
4. Compare measured CCLK period against the formal PVT envelope theorems in `lake/`.
5. Update `fpga/HARDWARE_SSOT.md` with the observed boot signature and any cable/board notes.

### Pros / cons

- **+** Provides irreplaceable physical validation.
- **+** Directly feeds the FPGA evidence backlog.
- **−** Blocked on hardware availability (DLC10 cable and board).
- **−** If the cable is missing, the wave cannot complete.

### Risk

High because of external dependency, but technically low if hardware is present.

---

## Recommendation

Choose **Variant B** as the default: it is the natural continuation of W480, it has a clear acceptance gate, and it reduces the Icarus baseline further without hardware dependencies. If the DLC10 cable arrives during the wave, pivot to **Variant C** for a half-wave hardware sprint while keeping the AOS fixes on a side branch. Use **Variant A** only if the team wants a formal contract more than an immediate gate improvement.

---

*φ² + φ⁻² = 3 | TRINITY*
