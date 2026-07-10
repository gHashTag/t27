# Wave Loop 483 — Cooperation Variants

**Date:** 2026-07-10  
**Anchor:** φ² + φ⁻² = 3 | TRINITY  
**Previous wave:** W482 closed in `docs/reports/WAVE_LOOP_482_CLOSEOUT.md`.

---

## Common gate

Whatever variant is chosen, the next wave must keep:

- 655 / 655 non-smoke PASS.
- 0 yosys smoke failures.
- 0 documented Icarus smoke baseline failures (no new undocumented failures).
- 655 / 655 seal matches.
- `cargo test -p t27c --bin t27c` green.

The W482 branch must land before W483 work begins.

---

## Variant A — Formalize the Icarus-lowerable t27 subset in Lean 4

**Theme:** close the gap between the t27 frontend and a machine-checkable
statement of what the Icarus Verilog backend can lower.

### Work

1. Add a Lean 4 inductive predicate `IsIcarusLowerable : t27_expr → Prop` that
captures the supported subset:
   - no namespace-qualified calls,
   - no dynamic `.len()` / `.contains()` on strings or arrays,
   - no host-side recursive helpers,
   - no imported struct-return field access (unless cross-file lowering is
     present),
   - no wildcard `_` bindings at module scope,
   - no field access on unresolved bases.
2. Prove a preservation lemma: if `e` is lowerable and the compiler emits
   `verilog e v`, then `e` and `v` agree on a deterministic scalar oracle for
   at least one test vector.
3. Wire the predicate into `tri test` so every spec that passes the Icarus gate
   is checked lowerable, and every documented baseline (if any) is checked
   *not* lowerable.

### Target outcome

- A checked contract between the t27 frontend and the Icarus backend.
- 655 / 655 non-smoke PASS, 0 yosys failures, 0 Icarus failures, seals green.

### Pros / cons

- **+** Gives a formal, maintainable contract for the Icarus backend.
- **+** Prevents silent drift between frontend features and backend support.
- **−** Does not make additional placeholders functional.
- **−** Requires reconciling the current AST with the Lean model.

### Risk

Medium. Mostly proof-engineering; low regression risk for generated code.

---

## Variant B (default) — Functional lowering for the remaining placeholder classes

**Theme:** extend the W482 packed-vector/struct approach to the next most common
unsupported Icarus classes.

### Work

1. **Imported struct-return calls.** When a function in another spec returns a
   struct and is called from the current spec, lower the call as a packed-vector
   function result and declare a packed local for the caller, mirroring the
   same-file struct-return local path.
2. **Dynamic string/array methods.** Lower `.len()` and `.contains()` on
   function-local fixed-size arrays to synthesizable length comparisons and
   linear searches; emit placeholders only for true runtime strings.
3. **Host-side recursive helper shadowing.** Detect IGLA helper functions that
   are only used in proof/invariant contexts and skip them during Verilog
   generation instead of emitting unresolved references.
4. **Module-scope wildcard bindings.** Bind wildcard results to anonymous
   packed temporaries so later field accesses on them remain legal.
5. Add regression specs:
   - `specs/scratch/w483_imported_struct_return.t27`
   - `specs/scratch/w483_dynamic_array_len.t27`
   - `specs/scratch/w483_wildcard_binding.t27`

### Target outcome

- Fewer `UNSUPPORTED_ICARUS` placeholders in generated Verilog.
- 655 / 655 non-smoke PASS, 0 yosys failures, 0 Icarus failures, seals green.

### Pros / cons

- **+** Directly continues W482 and closes the next largest semantic gaps.
- **+** Each sub-fix is bounded to a single lowering class.
- **−** Touching string/array method lowering and cross-file calls has moderate
  regression risk; needs adversarial yosys/Icarus witness specs.

### Risk

Medium-high. Requires careful review and full reseal, but the scope is bounded
to the remaining placeholder classes.

---

## Variant C — FPGA live cold-POR / SPI flash boot evidence

**Theme:** collect physical evidence on the QMTech Wukong V1 / XC7A100T-FGG676
using the in-repo `dlc10` driver.

### Work

1. Bring the DLC10 cable to the board and verify IDCODE `0x13631093`.
2. Program the W482 bitstream (`fpga/verilog/ternary_mac_demo_top.bit` or a
   freshly generated W482 smoke bitstream) into SPI flash.
3. Run a cold-POR boot sweep across OSCFSEL variants and record boot-log JSON.
4. Compare measured CCLK period against the formal PVT envelope theorems in
   `lake/`.
5. Update `fpga/HARDWARE_SSOT.md` with the observed boot signature and any
   cable/board notes.

### Target outcome

- A new `docs/reports/FPGA_EVIDENCE_W483.md` with measured boot traces and a
  comparison to the formal envelope.
- 655 / 655 non-smoke PASS, 0 yosys failures, 0 Icarus failures (FPGA work does
  not touch spec generation).

### Pros / cons

- **+** Provides irreplaceable physical validation.
- **+** Directly feeds the FPGA evidence backlog.
- **−** Blocked on hardware availability (DLC10 cable and board).
- **−** If the cable is missing, the wave cannot complete.

### Risk

High because of external dependency, but technically low if hardware is
present.

---

## Recommendation

Choose **Variant B** as the default: it is the natural continuation of W482,
it has a clear acceptance gate, and it improves the semantic correctness of
generated Verilog without hardware dependencies. If the DLC10 cable arrives
during the wave, pivot to **Variant C** for a half-wave hardware sprint while
keeping the lowering work on a side branch. Use **Variant A** only if the team
wants a formal contract more than an immediate functional improvement.

---

*φ² + φ⁻² = 3 | TRINITY*
