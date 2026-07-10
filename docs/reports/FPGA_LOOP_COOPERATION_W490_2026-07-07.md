# Wave Loop 490 — Cooperation Variants

**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY  
**Previous wave:** W489 closed in `docs/reports/WAVE_LOOP_489_CLOSEOUT.md`.

---

## Common gate

Whatever variant is chosen, the next wave must keep:

- 681 / 681 non-smoke PASS.
- 0 yosys smoke failures.
- 0 documented Icarus smoke baseline failures (no new undocumented failures).
- 681 / 681 seal matches.
- `cargo test -p t27c --bin t27c` green (1525 / 0 / 2).
- Zero `UNSUPPORTED_ICARUS` placeholders across all specs.

The W489 branch must land before W490 work begins.

---

## Variant A — Formalize the Icarus-lowerable subset in Lean 4

**Theme:** turn the informal rules now scattered through `gen-verilog` into a
machine-checkable predicate.

### Work

1. Extend the existing `lake/` formalization with an `IsIcarusLowerable`
   predicate that captures the checks now implicit in `bootstrap/src/compiler.rs`:
   - struct locals are lowerable only when their fields do not collide with
     Verilog keywords and the name is not redeclared in the same scope,
   - struct-return calls are lowerable only when every leaf field is numeric,
     `bool`, or a known lowerable constructor call,
   - enum variants and `string`/`f32` fields are not lowerable in synthesizable
     contexts,
   - imported constructors are lowerable only when the use declaration resolves
     and the argument count matches.
2. Prove a round-trip lemma for at least one representative witness per
   lowerable class (scalar struct literal, imported constructor, array-typed
   field, test-block local).
3. Add a `tri test --icarus-lowerable` gate that checks every spec that passes
   the Icarus smoke gate against the predicate.

### Target outcome

- A checked contract between the t27 frontend and the Icarus backend.
- 681 / 681 non-smoke PASS, 0 yosys failures, 0 Icarus failures, seals green.

### Pros / cons

- **+** Prevents silent drift between frontend features and backend support.
- **+** Gives users a precise definition of what will simulate in Icarus.
- **−** Does not make additional specs functional.
- **−** Requires reconciling the current AST with the Lean model.

### Risk

Medium. Mostly proof-engineering; low regression risk for generated code.

---

## Variant B (default) — Continue gen-verilog struct/call lowering hardening

**Theme:** close the remaining expression-context gaps that W489 deliberately
left on the statement-local side.

### Work

1. **Imported constructors in arbitrary expression context.**
   - W489 inlines imported scalar-struct constructors when the result is bound to
     a local or passed as an argument. Extend this to bare expression contexts
     such as field access on a returned struct call (`make_pt(a, b).x`) and to
     calls whose struct type has array-typed fields used directly in an
     expression.
2. **Module-scope array-of-struct constants with array-typed fields.**
   - Add a lowering path for `const pts : [N]Pt = [N]Pt{...}` where `Pt` has an
     array-typed field, emitting a multi-dimensional per-field memory and an
     `initial` copy block.
3. **Host-only enum/string helper hardening.**
   - Make the host-only classifier more aggressive for functions whose only use
     is string/enum manipulation, so IGLA specs do not emit dead-but-unparsable
     Verilog functions.
4. **Adversarial witnesses** for each new path in `specs/scratch/`.

### Target outcome

- More struct-return and AOS constant patterns compile and simulate correctly.
- 681 / 681 non-smoke PASS, 0 yosys failures, 0 Icarus failures, seals green.

### Pros / cons

- **+** Directly continues W487/W488/W489.
- **+** Each sub-fix is bounded and independently testable.
- **−** Touches the import-resolution and host-only-detection heuristics.
- **−** High regression risk without adversarial witnesses.

### Risk

High-medium. The fixes are bounded but span import resolution, expression
lowering, and host-only classification.

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

- A new `docs/reports/FPGA_EVIDENCE_W490.md` with measured boot traces and a
  comparison to the formal envelope.
- 681 / 681 non-smoke PASS, 0 yosys failures, 0 Icarus failures (FPGA work does
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

Choose **Variant B** as the default: it is the natural continuation of
W487/W488/W489, it has a clear acceptance gate, and it closes expression-context
lowering gaps that are already latent in the source specs. If the DLC10 cable
arrives during the wave, pivot to **Variant C** for a half-wave hardware sprint
while keeping the lowering work on a side branch. Use **Variant A** only if the
team wants a formal lowerability contract more than an immediate functional
improvement.

---

*φ² + φ⁻² = 3 | TRINITY*
