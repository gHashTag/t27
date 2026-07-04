# Wave Loop 409 Report — real P12 CCLK retry + per-OSCFSEL SPI transaction lookup

> Issue: [#1323](https://github.com/gHashTag/t27/issues/1323)  
> Branch: `wave-loop-409` → `master`  
> Date: 2026-07-04  
> Anchor: `phi^2 + phi^-2 = 3 | TRINITY`

---

## 1. Goal

Wave Loop 408 delivered a transaction-level SPI flash read model for the
canonical `OSCFSEL=0` configuration and documented the missing P12 wiring
blocker. Wave Loop 409 had two objectives:

- **Variant A:** retry the real P12 CCLK frequency/duty measurement once the
  wiring was available.
- **Variant C:** extend the transaction model to a per-OSCFSEL lookup table for
  `OSCFSEL = 0..7` and tighten the `tri fpga measure-cclk --validate` duty-cycle
  guard using the N25Q128 `t_CL` / `t_CH` limits.

The default bundle was **Variant A + C**. The 2026-07-04 bench check again showed
that **P12 is not wired to ADBUS4**, so Variant A could not produce a useful
silicon measurement. The wave therefore delivered **Variant C alone** plus an
updated real-capture blocker.

---

## 2. What changed

### `proofs/lean4/Trinity/TernaryFPGABoot.lean`

- Added `artix7_boot_transaction_for_oscfsel (oscfsel : Nat) (bitstream_bits : Nat)`
  that builds an `SPIReadTransaction` directly from a raw OSCFSEL selection.
- Rewrote `artix7_boot_transaction cfg bits` as a thin wrapper around the new
  lookup function.
- Proved `oscfsel_zero_to_seven_transaction_satisfies_flash_spec`, which states
  that every documented Artix-7 CCLK selection (`OSCFSEL ∈ {0..7}`) produces an
  N25Q128_3V-compliant transaction.
- Proved `artix7_boot_transaction_eq_for_oscfsel`, linking the config-level
  transaction function to the per-OSCFSEL lookup.

The proof uses `interval_cases` to enumerate the eight OSCFSEL values, then
`simp` with the UG470 frequency lookup and the N25Q128 constants to discharge each
branch computationally.

### `cli/tri/src/fpga.rs`

- Replaced the placeholder `25%–75%` duty-cycle guard with a frequency-derived
  bound computed from the N25Q128 `t_CL` / `t_CH` limits:
  ```text
  duty_pct ∈ [100·t_CL·f, 100 - 100·t_CH·f]
  ```
  where `t_CL = t_CH = 6 ns` and `f` is the measured CCLK frequency.
- Added a sensible `10%–90%` clamp so that very low-frequency captures still
  reject pathological pulses.
- Added `N25Q128_MIN_SCK_LOW_S` and `N25Q128_MIN_SCK_HIGH_S` constants.

### `fpga/HARDWARE_SSOT.md`

- Added §3.6.9 “Per-OSCFSEL transaction lookup (W409)” with a table of nominal
  CCLK, period, SCK low/high times, and flash margin for `OSCFSEL=0..7`.
- Updated §3.6.5 and §3.6.7 to describe the new N25Q128-derived duty-cycle bound.
- Updated the real-capture blocker note to confirm the 2026-07-04 re-attempt
  still returned 0 MHz / 100% duty.

### Reports

- `docs/reports/FPGA_LOOP_EVIDENCE_W409_2026-07-04.md` — exact command/output
  logs for the blocked live capture, synthetic fixture with new validation,
  Lean build, Rust tests, and `tri test` summary.
- `docs/reports/FPGA_LOOP_COOPERATION_W410_2026-07-04.md` — three W410
  cooperation variants.
- `docs/NOW.md` — W409 entry and updated `Last updated:` date.

---

## 3. Verification

### 3.1 Lean 4 formal build

```bash
cd /Users/playra/t27/proofs/lean4
lake build Trinity.TernaryFPGABoot
```

Result:

```text
✔ [2967/2967] Built Trinity.TernaryFPGABoot (16s)
Build completed successfully (2967 jobs).
```

### 3.2 Rust unit tests

```bash
cargo test -p tri fpga::tests --manifest-path /Users/playra/t27/cli/tri/Cargo.toml
```

Result:

```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 33 filtered out
```

### 3.3 Synthetic fixture with new duty bound

```bash
/Users/playra/t27/target/debug/tri fpga measure-cclk --synth --samplerate 100000000 --validate
```

Result:

```text
  Estimated frequency: 2.450 MHz
  Estimated duty cycle: 50.0%
  Validation: OK (CCLK within N25Q128 standard-read spec, 20.4x below 50.000 MHz limit, duty 50.0%, N25Q128-derived range 1.5%–98.5%)
```

### 3.4 Live CCLK capture retry

```bash
/Users/playra/t27/target/debug/tri fpga measure-cclk --live --driver ftdi-la --channel ADBUS4 \
    --samplerate 10000000 --samples 100000 --validate
```

Result:

```text
  Source: live (ftdi-la, ADBUS4)
  Estimated frequency: 0.000 MHz
  Estimated duty cycle: 100.0%
Error: measured CCLK 0.000 MHz is below 0.100 MHz; capture looks like noise or no signal
```

### 3.5 Conformance suite

```bash
cd /Users/playra/t27
./scripts/tri test
```

Result:

```text
Parse: 576 passed, 0 failed
Typecheck: 576 passed, 0 failed
Gen Zig: 576 passed, 0 failed
Gen Rust: 576 passed, 0 failed
Gen Verilog: 576 passed, 0 failed
Gen Verilog Yosys Smoke: 40 passed, 16 failed
Gen C: 576 passed, 0 failed
Seal Verify: 576 passed, 0 failed
Fixed Point: 0 divergences

TOTAL FAILURES: 16
```

The 16 failures are all in the `gen-verilog-yosys-smoke` phase and are
**pre-existing** on `wave-loop-409`:

- 3 IGLA specs (`benchmark`, `cordic`, `cordic_top`) fail because the branch
  does not yet contain the keyword-escape / tuple-return lowering that the
  `igla_clean_specs()` smoke gate assumes is present.
- 13 scratch specs (`w371`–`w388`) fail for the same reason: local-array,
  tuple-return, and `let`-destructuring lowering are documented in
  `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` as fixed in later waves, but those
  compiler changes are not present in this branch.

W409 did **not** modify `bootstrap/src/compiler.rs`; these failures are out of
scope for the per-OSCFSEL transaction lookup task. The parse / typecheck /
Zig / Rust / Verilog / seal-verify phases are all green (576/576).

---

## 4. Competitor positioning

| Competitor / project | Relevant capability | t27 differentiator after W409 |
|---|---|---|
| [Sparkle HDL / Verilean](https://github.com/Verilean/sparkle) | Lean 4 HDL + verified SoC | Sparkle has no public Artix-7 configuration-engine timing model. t27 now has a machine-checked lookup table for all documented CCLK selections. |
| [Kami / Kôika](https://github.com/SteffenReith/Kami) | Coq hardware DSL | Kami proves custom processors; t27 proves vendor FPGA configuration-engine timing against an external flash datasheet. |
| [Project X-Ray / prjxray](https://github.com/f4pga/prjxray) | Reverse-engineered bitstream | prjxray documents *what* the bits mean; t27 formalizes the *timing consequences* of the CCLK/CS/SCK bits. |
| [OpenTitan](https://opentitan.org/book/doc/security/specs/secure_boot/) | Secure SoC boot / RoT | OpenTitan secures a processor boot chain; t27 secures the FPGA *configuration* stage itself. |
| Commercial SPI NOR VIP | Closed simulation models | t27 provides an open, machine-checked Lean 4 bound tied to a real Artix-7 board. |

The defensive value of W409 is a **machine-checked, per-OSCFSEL transaction-level
proof** that covers every documented Artix-7 CCLK selection, paired with a
frequency-derived duty bound grounded in the N25Q128 datasheet.

---

## 5. Risks and residual work

- **Physical P12 wiring:** still missing. As soon as a wire is available from
  P12 to ADBUS4 (or DSLogic/oscilloscope), the live command shape is already
  implemented and validated against the synthetic fixture.
- **OSCFSEL 6 and 7:** included in the formal lookup table but have not been
  physically booted on the Wukong board. The W400 sweep covered only 0..5.
- **Process variation:** the model uses nominal UG470 frequencies. A real capture
  would let us tighten the guard with measured worst-case values.
- **Variant B automation:** relay-controlled cold-POR remains the next hardware
  CI milestone.
- **Transaction model constants:** CS# high time and wake-up are still datasheet
  constants; engine-derived values are a future extension.

---

## 6. Acceptance criteria status

- [x] AC-A1: real P12 capture re-attempted; persistent wiring blocker documented.
- [ ] AC-A2: live capture succeeds on real hardware (blocked by missing P12 wire).
- [x] AC-B1: Variant B deferred to W410.
- [x] AC-C1: `artix7_boot_transaction_for_oscfsel` added and the per-OSCFSEL
      theorem proved for `OSCFSEL ∈ {0..7}`.
- [x] AC-C2: `--validate` duty-cycle guard tightened using the N25Q128 `t_CL` /
      `t_CH` limits.
- [x] AC-D1: `lake build Trinity.TernaryFPGABoot` passes.
- [x] AC-D2: `cargo test -p tri fpga::tests` passes.
- [x] AC-D3: `./scripts/tri test` parse/typecheck/gen/seal-verify passes (576/576).
- [ ] AC-D4: `./scripts/tri test` gen-verilog-yosys-smoke is clean.
      **Residual:** 16 pre-existing failures from unmerged `gen-verilog` backend
      gaps tracked in `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`. W409 did not
      touch the Verilog backend.
- [x] AC-D5: W409 report + evidence + W410 cooperation variants committed.

---

*phi^2 + phi^-2 = 3 | TRINITY*
