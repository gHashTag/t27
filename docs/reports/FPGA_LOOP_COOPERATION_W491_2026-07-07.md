# Wave Loop 491 — Cooperation Variants

**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY  
**Previous wave:** W490 closed in `docs/reports/WAVE_LOOP_490_CLOSEOUT.md`.

---

## Common gate

Whatever variant is chosen, the next wave must keep:

- 687 / 687 non-smoke PASS (681 base specs + 6 W490 scratch witnesses).
- 0 yosys smoke failures.
- 0 documented Icarus smoke baseline failures (no new undocumented failures).
- 687 / 687 seal matches.
- `cargo test -p t27c --bin t27c` green (1525 / 0 / 2).
- Zero `UNSUPPORTED_ICARUS` placeholders across all specs.

The W490 branch must land before W491 work begins.

---

## Variant A (default) — Formalize the Icarus-lowerable subset in Lean 4

**Theme:** turn the informal rules now scattered through `gen-verilog` into a
machine-checkable predicate, now that the immediate expression-context gaps
are closed.

### Work

1. Extend the existing `lake/` formalization with an `IsIcarusLowerable`
   predicate that captures the checks now implicit in `bootstrap/src/compiler.rs`:
   - struct locals are lowerable only when their fields do not collide with
     Verilog keywords and the name is not redeclared in the same scope,
   - scalar struct-return calls are lowerable only when every leaf field is
     numeric, `bool`, or a known lowerable constructor call,
   - field access on a scalar struct-return call is lowerable only when the
     leaf field is scalar or a fixed-size array of numeric/bool values,
   - enum variants and `string`/`f32` fields are not lowerable in synthesizable
     contexts,
   - imported constructors are lowerable only when the use declaration resolves
     and the argument count matches.
2. Prove a round-trip lemma for at least one representative witness per
   lowerable class introduced in W490:
   - scalar struct literal,
   - imported constructor in expression context,
   - array-typed field accessed on a scalar struct-return call,
   - test-block local with variable-index array-field access.
3. Add a `tri test --icarus-lowerable` gate that checks every spec that passes
   the Icarus smoke gate against the predicate.

### Target outcome

- A checked contract between the t27 frontend and the Icarus backend.
- 687 / 687 non-smoke PASS, 0 yosys failures, 0 Icarus failures, seals green.

### Pros / cons

- **+** Prevents silent drift between frontend features and backend support.
- **+** Gives users a precise definition of what will simulate in Icarus.
- **−** Does not make additional specs functional.
- **−** Requires reconciling the current AST with the Lean model.

### Risk

Medium. Mostly proof-engineering; low regression risk for generated code.

---

## Variant B — Continue gen-verilog struct/call lowering hardening

**Theme:** close the next layer of expression-context gaps that W490 left on the
module-scope side.

### Work

1. **Nested struct-return calls in field access.**
   - Support patterns such as `make_outer(make_inner(a)).x` where the outer
     function returns a struct whose field is another scalar struct returned by
     an inner call. Currently the inner call is inlined as a packed operand, but
     field access on the outer call result may not resolve the packed layout
     correctly when the field is itself a struct.
2. **Module-scope AOS constants initialized from imported struct-return calls.**
   - Extend `gen_verilog_const` so that `const pts : [N]Pt = make_pts();`, where
     `make_pts` is defined in another module, emits the same multi-dimensional
     per-field memory as a literal initializer.
3. **Host-only propagation across import boundaries.**
   - Ensure that a function classified as host-only in one module does not get
     emitted in a downstream module that imports it, even when the downstream
     module has no direct reference to it.
4. **Adversarial witnesses** for each new path in `specs/scratch/`.

### Target outcome

- More module-level and nested-call struct patterns compile and simulate correctly.
- 687 / 687 non-smoke PASS, 0 yosys failures, 0 Icarus failures, seals green.

### Pros / cons

- **+** Directly continues W487/W488/W489/W490.
- **+** Each sub-fix is bounded and independently testable.
- **−** Touches import-resolution reachability and module-level lowering.
- **−** High regression risk without adversarial witnesses.

### Risk

High-medium. The fixes are bounded but span import resolution, nested call
lowering, and module-level constant initialization.

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

- A new `docs/reports/FPGA_EVIDENCE_W491.md` with measured boot traces and a
  comparison to the formal envelope.
- 687 / 687 non-smoke PASS, 0 yosys failures, 0 Icarus failures (FPGA work does
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

Choose **Variant A** as the default. With W490 closing the most urgent
expression-context lowering gaps, the next high-value move is to lock in the
lowerability contract in Lean so that future frontend features cannot silently
drift past what the Icarus backend can emit. If a concrete lowering bug
surfaces during the formalization work, fold it into the wave as a
proof-driven fix. If the DLC10 cable arrives, pivot to **Variant C** for a
half-wave hardware sprint while keeping the formalization work on a side
branch. Use **Variant B** only if the team wants more functional coverage before
investing in the formal contract.

---

*φ² + φ⁻² = 3 | TRINITY*
