# Wave Loop 489 — Cooperation Variants

**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY  
**Previous wave:** W488 closed in `docs/reports/WAVE_LOOP_488_CLOSEOUT.md`.

---

## Common gate

Whatever variant is chosen, the next wave must keep:

- 673 / 673 non-smoke PASS.
- 0 yosys smoke failures.
- 0 documented Icarus smoke baseline failures (no new undocumented failures).
- 673 / 673 seal matches.
- `cargo test -p t27c --bin t27c` green (1525 / 0 / 2).
- Zero `UNSUPPORTED_ICARUS` placeholders across all specs.

The W488 branch must land before W489 work begins.

---

## Variant A — Formalize the lowerable t27 subset in Lean 4

**Theme:** close the semantic gap between frontend features and the Icarus
backend with a machine-checkable predicate.

### Work

1. Extend (or create) an `IsIcarusLowerable` predicate under `lake/` that
   captures the rules now implicit in `bootstrap/src/compiler.rs`:
   - module-scope wildcard `_` aliases are lowerable only when every scalar/array
     leaf field of the element struct is numeric or bool,
   - function-local struct variables are lowerable only when their fields do
     not collide with Verilog keywords and the function does not redeclare the
   - same local name,
   - struct literals are lowerable only when every leaf is numeric, bool, or a
     supported call (no `string`, `f32`, or namespace-qualified enum variants).
2. Prove a preservation lemma for at least one deterministic test vector per
   lowerable class.
3. Wire the predicate into a new `tri test --icarus-lowerable` check so every
   spec that passes the Icarus gate is checked lowerable.

### Target outcome

- A checked contract between the t27 frontend and the Icarus backend.
- 673 / 673 non-smoke PASS, 0 yosys failures, 0 Icarus failures, seals green.

### Pros / cons

- **+** Prevents silent drift between frontend features and backend support.
- **+** Gives a precise definition of "supported" for users.
- **−** Does not make additional specs functional.
- **−** Requires reconciling the current AST with the Lean model.

### Risk

Medium. Mostly proof-engineering; low regression risk for generated code.

---

## Variant B (default) — Complete the colon struct-literal / struct-local lowering gaps

**Theme:** finish the W488 sub-fixes that were rolled back because they exposed
latent `gen-verilog` issues.

### Work

1. **Function-scope struct-local deduplication and keyword escaping.**
   - Track local struct variables already declared in the current procedural
     scope and emit the `reg` declaration only once.
   - Rename or escape local struct variable names that collide with Verilog
     keywords or generated block labels.
   - Add adversarial witness specs for `let assign = Struct{...}` and
     `let body = Struct{...}` patterns.
2. **Array-typed fields of packed scalar struct locals.**
   - When a function returns a struct with an array-typed field and that result
     is bound to a packed scalar struct local, emit per-field unpacked memories
     instead of a single packed vector so that `q.coords[i]` lowers to
     `q_coords[i]` rather than the illegal `q[...][i]`.
3. **Re-introduce colon-style struct-literal separators.**
   - Enable `field: value` in `parse_struct_literal` with the same guarded
     recovery path prototyped in W488.
   - Add witness specs for colon struct literals in module constants,
     function returns, and test blocks.

### Target outcome

- Existing `igla/` and `scratch/` specs that already contain colon struct
  literals compile and simulate correctly.
- 673 / 673 non-smoke PASS, 0 yosys failures, 0 Icarus failures, seals green.

### Pros / cons

- **+** Closes the largest remaining semantic gap in struct lowering.
- **+** Directly continues W487/W488.
- **−** High regression risk; needs adversarial yosys/Icarus witnesses.
- **−** Touches parser, local-variable tracking, and struct-return lowering.

### Risk

High-medium. The fixes are bounded but span parser and multiple codegen paths.

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

- A new `docs/reports/FPGA_EVIDENCE_W489.md` with measured boot traces and a
  comparison to the formal envelope.
- 673 / 673 non-smoke PASS, 0 yosys failures, 0 Icarus failures (FPGA work does
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

Choose **Variant B** as the default: it is the natural continuation of W487/W488,
it has a clear acceptance gate, and it closes the colon struct-literal lowering
gap that is already present in the source specs. If the DLC10 cable arrives
during the wave, pivot to **Variant C** for a half-wave hardware sprint while
keeping the lowering work on a side branch. Use **Variant A** only if the team
wants a formal lowerability contract more than an immediate functional
improvement.

---

*φ² + φ⁻² = 3 | TRINITY*
