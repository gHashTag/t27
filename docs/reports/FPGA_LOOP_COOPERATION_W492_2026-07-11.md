# Wave Loop 492 — Cooperation Variants

**Date:** 2026-07-11  
**Anchor:** φ² + φ⁻² = 3 | TRINITY  
**Previous wave:** W491 closes in `docs/reports/WAVE_LOOP_491_CLOSEOUT.md`.

---

## Common gate

Whatever variant is chosen, the next wave must keep:

- 687 / 687 non-smoke PASS (681 base specs + 6 W490 scratch witnesses).
- 0 yosys smoke failures.
- 0 documented Icarus smoke baseline failures.
- 687 / 687 seal matches.
- `cargo test -p t27c --bin t27c` green (1525 / 0 / 2).
- Zero `UNSUPPORTED_ICARUS` placeholders across all specs.
- `t27c suite --repo-root . --fast --icarus-lowerable` green (zero disagreements).

The W491 branch must land before W492 work begins.

---

## Variant A (default) — Extend the Lean 4 lowerability proof

**Theme:** turn the representative lemmas from W491 into a full soundness claim
for the Icarus-lowerable subset.

### Work

1. **Soundness direction:** prove that if the predicate classifies a spec as
   lowerable, then the emitted Verilog contains no `UNSUPPORTED_ICARUS`
   placeholders and no `// TODO: implement` stubs. This requires modeling a
   shallow Verilog abstract syntax and the emitter as a pure function from the
   simplified t27 AST.
2. **Completeness direction for the current corpus:** prove that every one of
   the 166 Icarus-passing specs is accepted by the predicate, by mechanically
   importing the JSON verdicts produced by `t27c icarus-lowerable`.
3. **Extend the AST/predicate** to cover module-scope AOS constants initialized
   from imported calls and nested struct-return field access, as those become
   supported in future waves.
4. **Add `tri verify --lean-lowerable`** gate that runs the Lean soundness proof
   and checks the completeness import.

### Target outcome

- A machine-checked guarantee that the Icarus path cannot silently drift.
- 687 / 687 non-smoke PASS, 0 yosys failures, 0 Icarus failures, seals green,
  lowerability gate green.

### Pros / cons

- **+** Locks in the W491 investment.
- **+** Provides a reusable model for future backends (e.g., native ternary).
- **−** High proof-engineering effort; may need to simplify the Verilog AST.
- **−** Does not make additional specs functional.

### Risk

High-medium. The proof is bounded to the Icarus subset, but modeling the
emitter is still significant work.

---

## Variant B — Continue gen-verilog struct/call lowering hardening

**Theme:** close the next layer of module-scope and nested-call gaps that W491
explicitly left out of scope.

### Work

1. **Nested struct-return calls in field access.**
   - Support `make_outer(make_inner(a)).x` where the outer function returns a
     struct whose field is itself a scalar struct returned by an inner call.
2. **Module-scope AOS constants from imported calls.**
   - Make `const pts : [N]Pt = make_pts();` work when `make_pts` is imported.
3. **Host-only propagation across import boundaries.**
   - Ensure that a function classified as host-only in its home module is also
     skipped in downstream modules that import it, even without a direct
     reference.
4. **Adversarial witnesses** for each new path in `specs/scratch/`.

### Target outcome

- More module-level and nested-call struct patterns compile and simulate correctly.
- 687 / 687 non-smoke PASS, 0 yosys failures, 0 Icarus failures, seals green,
  lowerability gate green.

### Pros / cons

- **+** Directly expands functional coverage.
- **+** Each sub-fix is bounded and independently testable.
- **−** Touches import-resolution reachability and nested call lowering.
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
   `lake/` / `proofs/lean4/`.
5. Update `fpga/HARDWARE_SSOT.md` with the observed boot signature and any
   cable/board notes.

### Target outcome

- A new `docs/reports/FPGA_EVIDENCE_W492.md` with measured boot traces and a
  comparison to the formal envelope.
- 687 / 687 non-smoke PASS, 0 yosys failures, 0 Icarus failures (FPGA work does
  not touch spec generation), lowerability gate green.

### Pros / cons

- **+** Provides irreplaceable physical validation.
- **+** Directly feeds the FPGA evidence backlog.
- **−** Blocked on hardware availability (DLC10 cable and board).
- **−** If the cable is missing, the wave cannot complete.

### Risk

High because of external dependency, but technically low if hardware is present.

---

## Recommendation

Choose **Variant A** as the default. W491 formalized the predicate and the
classifier; W492 should prove that the classifier is sound with respect to the
actual emitter output. If a concrete lowering bug surfaces during the soundness
work, fold it into the wave as a proof-driven fix. If the DLC10 cable arrives,
pivot to **Variant C** for a half-wave hardware sprint while keeping the
formalization work on a side branch. Use **Variant B** only if the team wants
more functional coverage before investing in the full soundness proof.

---

*φ² + φ⁻² = 3 | TRINITY*
