# Wave Loop 411 Report — real P12 + OSCFSEL 6/7 retry, relay CI gate, or auto-proof tooling

> Issue: [#1329](https://github.com/gHashTag/t27/issues/1329)  
> Branch: `wave-loop-411`  
> Closes #1329

---

## Summary

Wave Loop 411 was scoped with the default bundle **Variant A + C**: finally
connect the bench, capture real P12 CCLK, physically boot `OSCFSEL=6,7`, and
add auto-proof tooling that turns a measurement into a Lean theorem.

The physical blockers from W410 persisted:

- P12 is still not wired to a logic-analyzer channel.
- The Digilent DLC10 cable is still not detected (`VID=0x03FD`).

W411 therefore delivered **Variant C alone**: a `tri fpga measured-to-lean`
subcommand that generates a type-correct Lean theorem from a `--json`
measurement, plus a PVT-margin predicate in Lean 4 that uses conservative 2×
derated SCK low/high limits. Variant A and Variant B are documented as blocked
and deferred to W412.

---

## What changed

### `proofs/lean4/Trinity/TernaryFPGABoot.lean`

- Added worst-case SCK timing constants:
  `N25Q128_MIN_SCK_LOW_NS_WC` and `N25Q128_MIN_SCK_HIGH_NS_WC` (12 ns each,
  a conservative 2× derating of the nominal 6 ns limits).
- Added `measured_cclk_with_margin_satisfies_flash_spec` predicate.
- Proved `measured_cclk_with_margin_implies_measured_cclk_satisfies_flash_spec`
  and `measured_cclk_with_margin_implies_transaction_ok`.
- Added concrete PVT-margin examples for the synthetic 2.5 MHz fixture and the
  nominal `OSCFSEL=6,7` rates (25 MHz and 33.3 MHz).

### `cli/tri/src/fpga.rs`

- Added `FpgaCmd::MeasuredToLean` subcommand with `--file`, `--out`, `--name`,
  and `--margin` options.
- Implemented `measured_to_lean`: reads a `MeasuredCclk` JSON record and emits a
  Lean theorem snippet that proves both the flash predicate and the
  transaction-level consequence.
- Added `sanitize_lean_ident` helper for turning the source string into a valid
  Lean identifier suffix.
- Added unit tests for identifier sanitization, nominal output, and margin output.

### `fpga/HARDWARE_SSOT.md`

- Added §3.6.11 documenting the `measured-to-lean` pipeline and the PVT-margin
  model, with copy-paste examples for both nominal and `--margin` modes.

### Reports

- `docs/reports/FPGA_LOOP_EVIDENCE_W411_2026-07-04.md` — command outputs,
  build results, and conformance summary.
- `docs/reports/WAVE_LOOP_411_REPORT.md` — this close-out report.
- `docs/reports/FPGA_LOOP_COOPERATION_W412_2026-07-04.md` — three W412
  cooperation variants.

---

## Acceptance criteria status

| ID | Criterion | Status |
|----|-----------|--------|
| AC-A1 | Real P12 CCLK capture CSV exists (or blocker documented) | ❌ blocked — P12 not wired |
| AC-A2 | `OSCFSEL=6,7` boot logs exist (PASS or documented failure) | ❌ blocked — DLC10 cable missing |
| AC-A3 | Lean theorem links captured `(frequency, duty)` to `transaction_satisfies_flash_spec` | ✅ delivered via `measured-to-lean` on synthetic data |
| AC-B1 | Relay auto-power-cycle trait + mock path, or deferred | ⏸ deferred to W412 |
| AC-C1 | `measured-to-lean` generates type-correct theorem skeleton | ✅ |
| AC-C2 | PVT margin predicate exists and implies nominal predicate | ✅ |
| AC-C3 | PVT margin predicate implies `transaction_satisfies_flash_spec` | ✅ |
| AC-D1 | `lake build Trinity.TernaryFPGABoot` passes | ✅ |
| AC-D2 | `cargo test -p tri fpga::tests` passes | ✅ (14/14) |
| AC-D3 | `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass | ✅ |
| AC-D4 | gen-verilog-yosys-smoke clean or failures tracked | ⚠️ 16 pre-existing failures tracked |
| AC-D5 | W411 report + evidence + W412 cooperation committed | ✅ |

---

## Competitor scan (2026 formal-HDL / FPGA-boot space)

The same competitor set from W410 remains relevant; W411 narrows the gap in
measured-to-formal tooling:

- **Sparkle HDL / VeriLean** — strong Lean 4 HDL semantics, but no public
  pipeline from a `sigrok-cli` capture to a flash-datasheet theorem.
- **Kami / Kôika** — Coq-based hardware DSL; no 7-series configuration-engine
  timing model.
- **prjxray / OpenXC7** — bitstream-level knowledge, complementary to t27's
  spec-first proofs.
- **OpenTitan** — SoC boot security, not FPGA configuration-stage timing.
- **spispy / SPI flash emulators** — useful for protocol analysis, but do not
  produce machine-checked timing bounds.
- **Commercial SPI NOR VIP** — closed reference models, not tied to live
  measurement.

W411's auto-proof tooling is another step that is easy to describe but hard to
reproduce end-to-end because it requires both a real board and a formal model
that matches the conservative integer conversions in the measurement pipeline.

---

## Risks and weak points

1. **Physical evidence gap:** The model is still not anchored to fresh silicon
   measurements for OSCFSEL 6/7. The `measured-to-lean` pipeline is ready, but
   the input is synthetic until P12 is wired.
2. **PVT margin is a placeholder:** The 2× derating is conservative but not
   derived from actual N25Q128 PVT characterization data. It should be replaced
   with real derating curves once available.
3. **Bench dependency:** All physical progress is gated by wiring and cable
   access. No tooling can substitute for this.
4. **Gen-verilog-yosys-smoke backlog:** 16 failures remain tracked separately.
5. **Generated snippet namespace:** The emitted theorem references functions
   inside `Trinity.BitstreamConfig`; the user must paste it into a Lean file
   with `namespace Trinity` and `open BitstreamConfig` (documented in
   `fpga/HARDWARE_SSOT.md` §3.6.11).

---

## Verification results

- `lake build Trinity.TernaryFPGABoot` — green.
- `cargo test -p tri fpga::tests` — 14/14 pass.
- `tri fpga measured-to-lean --file /tmp/measured.json` — produces type-correct
  theorem snippet matching the existing Lean examples.
- `./scripts/tri test` — parse/typecheck/gen/seal-verify all green; yosys smoke
  40 pass, 16 fail (pre-existing).

---

## Recommended next wave (W412)

See `docs/reports/FPGA_LOOP_COOPERATION_W412_2026-07-04.md` for the three
variants. The default recommendation is **Variant A + B bundle**: finally wire P12
and the DLC10 cable, capture the real CCLK and boot `OSCFSEL=6,7`, while also
building the relay-controlled cold-POR CI gate. If the bench is still
unavailable, **Variant C alone** (refine PVT margins with real datasheet data, or
extend `measured-to-lean` to emit a full standalone `.lean` file) remains the
fallback.

---

*phi^2 + 1/phi^2 = 3 | TRINITY*
