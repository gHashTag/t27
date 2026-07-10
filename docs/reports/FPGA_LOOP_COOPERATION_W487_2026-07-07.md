# Wave Loop 487 — Cooperation Variants

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY
**Previous wave:** W486 closed in `docs/reports/WAVE_LOOP_486_CLOSEOUT.md`.

---

## Common gate

Whatever variant is chosen, the next wave must keep:

- 667 / 667 non-smoke PASS.
- 0 yosys smoke failures.
- 0 documented Icarus smoke baseline failures (no new undocumented failures).
- 667 / 667 seal matches.
- `cargo test -p t27c --bin t27c` green (1525 / 0 / 2).
- Zero `UNSUPPORTED_ICARUS` placeholders across all specs.

The W486 branch must land before W487 work begins.

---

## Variant A — Formalize the Icarus-lowerable t27 subset in Lean 4

**Theme:** close the gap between the t27 frontend and a machine-checkable
statement of what the Icarus Verilog backend can now lower after W486.

### Work

1. Extend the Lean 4 `IsIcarusLowerable` predicate (or create one under `lake/`)
   to capture the additional restrictions now enforced in W486:
   - no namespace-qualified calls used in synthesizable contexts (after W486,
     host-only namespace calls are erased, but synthesizable namespace calls
     still produce placeholders),
   - no bench-local arrays passed to array-parameter functions unless the lowering
     path is explicitly marked supported (W486),
   - no module-scope wildcard `_` bindings with struct-literal initializers
     (parser-blocked, W486).
2. Prove a preservation lemma: if `e` is lowerable and the compiler emits
   `verilog e v`, then `e` and `v` agree on a deterministic scalar oracle for
   at least one test vector.
3. Wire the predicate into `tri test` so every spec that passes the Icarus gate
   is checked lowerable, and every documented baseline (if any) is checked
   *not* lowerable.

### Target outcome

- A checked contract between the t27 frontend and the Icarus backend that now
  covers W486 erasure rules.
- 667 / 667 non-smoke PASS, 0 yosys failures, 0 Icarus failures, seals green.

### Pros / cons

- **+** Gives a formal, maintainable contract for the Icarus backend.
- **+** Prevents silent drift between frontend features and backend support.
- **−** Does not make additional specs functional.
- **−** Requires reconciling the current AST with the Lean model.

### Risk

Medium. Mostly proof-engineering; low regression risk for generated code.

---

## Variant B (default) — Continue backend hardening for the remaining lowering gaps

**Theme:** close the gaps deliberately left open at W486 so that more IGLA and
bench specs simulate cleanly without reintroducing `UNSUPPORTED_ICARUS`
placeholders.

### Work

1. **Module-scope wildcard struct-literal bindings.**
   - Fix the parser so `let _ = Pt{...};` at module scope does not stop consuming
     subsequent declarations, or add a dedicated recovery path.
   - Emit an anonymous packed temporary for the struct literal without inventing
     a named `_` reg.
2. **Module-scope wildcard array aliases.**
   - Generalize the anonymous ROM logic so `let _ = existing_array;` at module
     scope emits an anonymous copy memory (or a safe alias comment with a clear
     reason string).
3. **2-D / struct bench-local arrays crossing function boundaries.**
   - Extend the bench-local packed-vector path to multi-dimensional scalar arrays
     and arrays of structs, including correct packed-vector slicing for variable
     and multi-index access inside the callee.

### Target outcome

- Additional IGLA and bench specs become Icarus-simulatable without new
  `UNSUPPORTED_ICARUS` placeholders.
- 667 / 667 non-smoke PASS, 0 yosys failures, 0 Icarus failures, seals green.

### Pros / cons

- **+** Directly continues W486 and closes the next largest semantic gaps.
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

- A new `docs/reports/FPGA_EVIDENCE_W487.md` with measured boot traces and a
  comparison to the formal envelope.
- 667 / 667 non-smoke PASS, 0 yosys failures, 0 Icarus failures (FPGA work does
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

Choose **Variant B** as the default: it is the natural continuation of W486, it
has a clear acceptance gate, and it improves the semantic correctness of
generated Verilog without hardware dependencies. If the DLC10 cable arrives
during the wave, pivot to **Variant C** for a half-wave hardware sprint while
keeping the lowering work on a side branch. Use **Variant A** only if the team
wants a formal contract more than an immediate functional improvement.

---

*φ² + φ⁻² = 3 | TRINITY*
