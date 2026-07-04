# Wave Loop 410 Report — real P12 CCLK capture or physical OSCFSEL 6/7 boot + measured-duty formal link

> Issue: [#1325](https://github.com/gHashTag/t27/issues/1325)  
> Branch: `wave-loop-410`  
> Closes #1325

---

## Summary

Wave Loop 410 was scoped as a follow-up to W409 with the default bundle
**Variant A + C**: finally capture the real P12 CCLK frequency/duty cycle, and
physically verify `OSCFSEL=6,7` on the Wukong board, while adding a formal
lemma that turns a measured `(frequency, duty)` pair into a
`transaction_satisfies_flash_spec` proof.

Both physical halves are still blocked by the same bench wiring/cable issues
that have persisted since W406:

- P12 is not wired to a logic-analyzer channel, so a real CCLK measurement is
  impossible.
- The Digilent DLC10 JTAG cable is not detected by the host (`VID=0x03FD`), so
  flash programming and `cclk-sweep` are impossible.

W410 therefore delivered the **formal-only half of Variant C**: a
`measured_cclk_satisfies_flash_spec` predicate and the
`measured_cclk_satisfies_flash_spec_implies_transaction_ok` theorem in Lean 4,
plus a Rust `MeasuredCclk` record and `--json` output that can feed the formal
predicate. This makes the CCLK validation pipeline itself formally traceable,
and once the bench is wired, a real capture can be turned into a proof with no
additional model work.

---

## What changed

### `proofs/lean4/Trinity/TernaryFPGABoot.lean`

- Added measured-CCLK definitions that mirror the conservative
  frequency-to-period conversion used by the Rust CLI:
  `measured_cclk_period_ns`, `measured_cclk_low_ns`,
  `measured_cclk_high_ns`, `measured_boot_transaction`.
- Added `measured_cclk_satisfies_flash_spec (freq_hz : Nat) (duty_pct : Nat)`
  as the formal counterpart of `tri fpga measure-cclk --validate`.
- Proved `measured_cclk_satisfies_flash_spec_implies_transaction_ok`, which
  closes the formal link between a real capture and the existing
  `transaction_satisfies_flash_spec` model.
- Added helper lemmas `measured_cclk_low_le_period` and
  `measured_cclk_period_at_least_min_sck_period`.
- Added concrete examples for the synthetic 2.5 MHz fixture and the nominal
  `OSCFSEL=6,7` rates (`25 MHz` and `33.3 MHz`, both at 50% duty).

### `cli/tri/src/fpga.rs`

- Added `MeasuredCclk` struct with conservative `sck_low_ns` / `sck_high_ns`
  and JSON round-trip support.
- Added `tri fpga measure-cclk --json` to emit the formal-link JSON record.
- Added unit tests for 2.5 MHz, 25 MHz, and JSON round-trip.

### `fpga/HARDWARE_SSOT.md`

- §3.6.1 now notes that the real P12 capture is still blocked and that the
  synthetic fixture remains the only validated path until the wiring is fixed.
- §3.6.9 notes that `OSCFSEL=6,7` are nominally compliant by the W409 lookup
  proof, but physically unverified because the DLC10 cable is unavailable.
- Added a reference to the new `measured_cclk_satisfies_flash_spec` formal link.

### Reports

- `docs/reports/FPGA_LOOP_EVIDENCE_W410_2026-07-04.md` — build outputs, blocked
  hardware state, and verification summary.
- `docs/reports/FPGA_LOOP_COOPERATION_W411_2026-07-04.md` — three cooperation
  variants for W411.

---

## Acceptance criteria status

| ID | Criterion | Status |
|----|-----------|--------|
| AC-A1 | Real CCLK capture CSV exists | ❌ blocked — P12 not wired |
| AC-A2 | `fpga/HARDWARE_SSOT.md` §3.6.1 contains measured frequency/duty | ❌ blocked |
| AC-A3 | `tri fpga measure-cclk --live ... --validate` passes on hardware | ❌ blocked |
| AC-B1 | Relay auto-power-cycle deferred | ⏸ deferred to W411 |
| AC-C1 | `OSCFSEL=6,7` physically booted and logged | ❌ blocked — DLC10 cable missing |
| AC-C2 | `measured_cclk_satisfies_flash_spec` predicate and lemma link `(freq, duty)` to transaction spec | ✅ completed |
| AC-D1 | `lake build Trinity.TernaryFPGABoot` passes | ✅ |
| AC-D2 | `cargo test -p tri fpga::tests` passes | ✅ (11/11) |
| AC-D3 | `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass | ✅ |
| AC-D4 | gen-verilog-yosys-smoke clean or failures tracked | ⚠️ 16 pre-existing failures tracked in `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` |
| AC-D5 | W410 report + evidence + W411 cooperation committed | ✅ |

---

## Competitor scan (2026 formal-HDL / FPGA-boot space)

The same 2026 competitors identified in W409 are still the relevant reference
points; W410 does not change the competitive map, but it removes another
reproducibility barrier:

- **Sparkle HDL / VeriLean** — Lean 4/Verilog bridges that can state and prove
  FPGA-boot timing claims. They do not yet have a public end-to-end proof that
  ties a measured CCLK duty cycle to a flash datasheets via an automated
  pipeline.
- **Kami / Bluespec** — strong semantics for hardware blocks, but the boot
  configuration-engine + flash timing link is not their focus.
- **prjxray / OpenXC7** — bitstream-level knowledge; complementary to t27's
  spec-first proof, not a direct competitor.
- **OpenTitan** — has strong lifecycle/ROM boot evidence, but targets a
  different class of parts and does not publish a 7-series CCLK transaction
  proof.

W410's measured-duty formal link is another step that is straightforward to
state but hard to reproduce end-to-end because it requires both a real board
and a spec-first formal model.

---

## Risks and weak points

1. **Physical evidence gap:** The bitstream is ready, but neither P12 nor the
   DLC10 cable are connected. Until this is fixed, the formal model is not
   anchored to fresh silicon measurements for OSCFSEL 6/7.
2. **Board/wiring dependency:** Real progress is gated by physical bench
   access. No amount of model work can close this.
3. **Gen-verilog-yosys-smoke backlog:** 16 scratch/IGLA specs still fail
   yosys smoke. These are tracked and do not affect the FPGA boot work, but
   they are a recurring distraction in conformance reports.
4. **Copy-paste formal link:** The Rust JSON output is designed to be pasted
   into the Lean predicate. A future improvement is to generate the Lean
   `measured_cclk_satisfies_flash_spec` call automatically from the JSON.

---

## Verification results

- `lake build Trinity.TernaryFPGABoot` — green.
- `cargo test -p tri fpga::tests` — 11/11 pass.
- `tri fpga measure-cclk --synth --validate --json` — green, JSON output
  matches Lean predicate fields.
- `./scripts/tri test` — parse/typecheck/gen Zig/gen Rust/gen Verilog/seal
  verify/gen C/fixed-point all green; yosys smoke 40 pass, 16 fail
  (pre-existing).

---

## Recommended next wave (W411)

See `docs/reports/FPGA_LOOP_COOPERATION_W411_2026-07-04.md` for the three
variants. The default recommendation is **Variant A + C again**: wire P12 and
the DLC10 cable, capture the real CCLK, and physically boot `OSCFSEL=6,7`.
If the bench is still unavailable, **Variant B** (relay-controlled cold-POR)
can advance CI automation independently.

---

*phi^2 + 1/phi^2 = 3 | TRINITY*
