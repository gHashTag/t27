# Wave Loop 488 — Cooperation Variants

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY
**Previous wave:** W487 closed in `docs/reports/WAVE_LOOP_487_CLOSEOUT.md`.

---

## Common gate

Whatever variant is chosen, the next wave must keep:

- 672 / 672 non-smoke PASS.
- 0 yosys smoke failures.
- 0 documented Icarus smoke baseline failures (no new undocumented failures).
- 672 / 672 seal matches.
- `cargo test -p t27c --bin t27c` green (1525 / 0 / 2).
- Zero `UNSUPPORTED_ICARUS` placeholders across all specs.

The W487 branch must land before W488 work begins.

---

## Variant A — Formalize the Icarus-lowerable t27 subset in Lean 4

**Theme:** close the gap between the t27 frontend and a machine-checkable
statement of what the Icarus Verilog backend can now lower after W487.

### Work

1. Extend the Lean 4 `IsIcarusLowerable` predicate (or create one under `lake/`)
   to capture the additional restrictions now enforced in W487:
   - module-scope wildcard `_` bindings with struct-literal or array-identifier
     initializers are lowerable only when every leaf field is numeric or bool,
   - bench-local 2-D scalar arrays and arrays of structs are lowerable only when
     passed to array-parameter functions through the `__local__` packed-vector
     path,
   - function-return struct literals are lowerable only when they contain no
     `string`, `f32`, or namespace-qualified (`::`) leaf values.
2. Prove a preservation lemma: if `e` is lowerable and the compiler emits
   `verilog e v`, then `e` and `v` agree on a deterministic scalar oracle for
   at least one test vector.
3. Wire the predicate into `tri test` so every spec that passes the Icarus gate
   is checked lowerable, and every documented baseline (if any) is checked
   *not* lowerable.

### Target outcome

- A checked contract between the t27 frontend and the Icarus backend that now
  covers W487 lowering rules.
- 672 / 672 non-smoke PASS, 0 yosys failures, 0 Icarus failures, seals green.

### Pros / cons

- **+** Gives a formal, maintainable contract for the Icarus backend.
- **+** Prevents silent drift between frontend features and backend support.
- **−** Does not make additional specs functional.
- **−** Requires reconciling the current AST with the Lean model.

### Risk

Medium. Mostly proof-engineering; low regression risk for generated code.

---

## Variant B (default) — Continue backend hardening for the remaining lowering gaps

**Theme:** close the gaps deliberately left open at W487 so that more IGLA and
bench specs simulate cleanly without reintroducing `UNSUPPORTED_ICARUS`
placeholders.

### Work

1. **Colon-style struct-literal field separators.**
   - Re-introduce `field: value` parsing in `parse_struct_literal` with a
     guarded recovery path so a malformed literal does not swallow the rest of
     the module.
   - Lower namespace-qualified enum variants (`Enum::Variant`) and other
     non-synthesizable leaf expressions to numeric constants or safe zero
     placeholders instead of invalid Verilog identifiers.
2. **Non-synthesizable struct fields beyond zero placeholders.**
   - Decide whether `string` and `f32` struct fields should be stripped from
     packed vectors, emitted as parallel `real`/`reg [8*len-1:0]` signals, or
     kept as host-only. Implement the chosen policy and add adversarial witness
     specs.
3. **Wildcard array-of-struct aliases with array-typed fields.**
   - Extend the AOS alias branch in `gen_verilog_const` to handle element structs
     whose fields are themselves arrays, emitting multi-dimensional per-field
     memories and copying them element-by-element.

### Target outcome

- Additional IGLA and bench specs become Icarus-simulatable without new
  `UNSUPPORTED_ICARUS` placeholders.
- 672 / 672 non-smoke PASS, 0 yosys failures, 0 Icarus failures, seals green.

### Pros / cons

- **+** Directly continues W487 and closes the next largest semantic gaps.
- **+** Each sub-fix is bounded to a single lowering class.
- **−** Touching IGLA/bench lowering has moderate regression risk; needs
  adversarial yosys/Icarus witness specs.

### Risk

Medium-high. Requires careful review and full reseal, but the scope is bounded.

---

## Variant C — FPGA live cold-POR / SPI flash boot evidence

**Theme:** collect physical evidence on the QMTech Wukong V1 / XC7A100T-FGG676
using the in-repo `dlc10` driver.

### Work

1. Bring the DLC10 cable to the board and verify IDCODE `0x13631093` with
   `dlc10 idcode`.
2. Program a freshly generated smoke bitstream into SPI flash with `dlc10 flash`.
3. Run a cold-POR boot sweep across OSCFSEL variants and record boot-log JSON.
4. Compare measured CCLK period against the formal PVT envelope theorems in
   `lake/`.
5. Update `fpga/HARDWARE_SSOT.md` with the observed boot signature and any
   cable/board notes.

### Target outcome

- A new `docs/reports/FPGA_EVIDENCE_W488.md` with measured boot traces and a
  comparison to the formal envelope.
- 672 / 672 non-smoke PASS, 0 yosys failures, 0 Icarus failures (FPGA work does
  not touch spec generation).

### Pros / cons

- **+** Provides irreplaceable physical validation.
- **+** Directly feeds the FPGA evidence backlog.
- **−** Blocked on hardware availability (DLC10 cable and board).
- **−** If the cable is missing, the wave cannot complete.

### Risk

High because of external dependency, but technically low if hardware is present.

---

## Recommendation

Choose **Variant B** as the default: it is the natural continuation of W487, it
has a clear acceptance gate, and it improves the semantic correctness of
generated Verilog without hardware dependencies. If the DLC10 cable arrives
during the wave, pivot to **Variant C** for a half-wave hardware sprint while
keeping the lowering work on a side branch. Use **Variant A** only if the team
wants a formal contract more than an immediate functional improvement.

---

*φ² + φ⁻² = 3 | TRINITY*
