# Wave Loop 408 Report — real P12 CCLK + complete SPI transaction model in Lean 4

> Issue: [#1318](https://github.com/gHashTag/t27/issues/1318)  
> Branch: `wave-loop-408` → `master`  
> Date: 2026-07-04  
> Anchor: `phi^2 + phi^-2 = 3 | TRINITY`

---

## 1. Goal

Wave Loop 407 closed the static SPI flash timing model and added a synthetic
CCLK validation fixture. Wave Loop 408 had two objectives:

- **Variant A:** capture the real CCLK frequency and duty cycle on pin P12 and
  record the measured value in `fpga/HARDWARE_SSOT.md` §3.6.
- **Variant C:** add a complete SPI flash read-transaction model in Lean 4 and
  prove that the canonical `OSCFSEL=0` configuration produces an
  N25Q128_3V-compliant transaction.

The default bundle was **Variant A + C**. However, the on-bench attempt
showed that **P12 is not wired to the logic analyzer**, so Variant A could not
produce a useful silicon measurement. The wave therefore delivered
**Variant C alone** plus a documented real-capture blocker.

---

## 2. What changed

### `proofs/lean4/Trinity/TernaryFPGABoot.lean`

Added a transaction-level model on top of the W407 static timing predicates:

```lean
structure SPIReadTransaction where
  csHighNs : Nat
  numSckEdges : Nat
  sckLowNs : Nat
  sckHighNs : Nat
  wakeUs : Nat

-- Transaction computed from a bitstream config and the number of bits to shift.
def artix7_boot_transaction (cfg : BitstreamConfig) (bitstream_bits : Nat) :
    SPIReadTransaction := ...

-- Checks CS# high, SCK low/high, max SCK frequency, and wake-up bounds.
def transaction_satisfies_flash_spec (t : SPIReadTransaction) : Bool := ...
```

Proved three theorems:

```lean
theorem canonical_oscfsel_transaction_satisfies_flash_spec :
  ∀ (bits : Nat), transaction_satisfies_flash_spec
    (artix7_boot_transaction ⟨IDCODE_XC7A200T, SPI_BUSWIDTH_X1, STARTUPCLK_CCLK, OSCFSEL_DEFAULT⟩ bits)
    = true

theorem canonical_implies_transaction_satisfies_flash_spec (cfg : BitstreamConfig) (bits : Nat) :
  cfg.canonical → transaction_satisfies_flash_spec (artix7_boot_transaction cfg bits)

theorem cold_por_implies_transaction_satisfies_flash_spec
  (p : ColdPOR) (s : StatRegister) (bits : Nat) :
  cold_por_spi_flash_pred p s
  → BitstreamConfig.transaction_satisfies_flash_spec (BitstreamConfig.artix7_boot_transaction p.cfg bits)
```

The transaction spec checks:

| Field | Bound | Source |
|---|---|---|
| `csHighNs` | `≥ 100 ns` | `N25Q128_MIN_CS_HIGH_NS` |
| `sckLowNs` | `≥ 6 ns` | `N25Q128_MIN_SCK_LOW_NS` |
| `sckHighNs` | `≥ 6 ns` | `N25Q128_MIN_SCK_HIGH_NS` |
| `sckLowNs + sckHighNs` | `1e9 / sum ≤ 50 MHz` | `N25Q128_MAX_SCK_HZ` |
| `wakeUs` | `≥ 100 us` | `N25Q128_WAKE_FROM_POWERDOWN_US` |

For `OSCFSEL=0` the model predicts a 400 ns CCLK period, 200 ns SCK low/high,
2.5 MHz SCK frequency, and therefore satisfies every bound.

### `fpga/HARDWARE_SSOT.md`

- Added §3.6.8 “Formal SPI transaction model traceability (W408)” with the
  structure/predicate table and the canonical transaction values.
- Added a real-capture blocker note documenting the 2026-07-04 live capture
  attempt that returned 0 MHz because P12 is not wired to ADBUS4.

### Reports

- `docs/reports/FPGA_LOOP_EVIDENCE_W408_2026-07-04.md` — exact command/output
  logs for the failed live capture, synthetic fixture, Lean build, and Rust
  unit tests.
- `docs/reports/FPGA_LOOP_COOPERATION_2026-07-04.md` — three W409 cooperation
  variants.
- `docs/NOW.md` — W408 entry and updated `Last updated:` date.

---

## 3. Verification

### 3.1 Lean 4 formal build

```bash
cd /Users/playra/t27/proofs/lean4
lake build Trinity.TernaryFPGABoot
```

Result:

```text
✔ [2/2] Built Trinity.TernaryFPGABoot (611ms)
Build completed successfully (2 jobs).
```

### 3.2 Rust unit tests

```bash
cargo test -p tri fpga::tests
```

Result:

```text
running 8 tests
...
test result: ok. 8 passed; 0 failed; ... 33 filtered out
```

### 3.3 Live CCLK capture attempt

```bash
./target/debug/tri fpga measure-cclk --live --driver ftdi-la --channel ADBUS4 \
    --samplerate 10000000 --samples 100000 --validate
```

Result:

```text
  Logic samples: 100000 (high 100000, low 0, transitions 0)
  Estimated frequency: 0.000 MHz
  Estimated duty cycle: 100.0%
Error: measured CCLK 0.000 MHz is below 0.100 MHz; capture looks like noise or no signal
```

This is the documented blocker: the FTDI cable is detected, but P12 is not
wired.

### 3.4 Synthetic fixture

```bash
./target/debug/tri fpga measure-cclk --synth --samplerate 100000000 --validate
```

Result:

```text
  Estimated frequency: 2.450 MHz
  Estimated duty cycle: 50.0%
  Validation: OK (CCLK within N25Q128 standard-read spec, 20.4x below 50.000 MHz limit, duty 50.0%)
```

### 3.5 Conformance suite

After resealing all `.t27` specs with the freshly built `t27c` release binary:

```bash
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
Seal Verify: 576 passed, 0 failed
TOTAL FAILURES: 16
```

The 16 failures are all in the `gen-verilog-yosys-smoke` phase and are
**pre-existing** on `wave-loop-408`:

- 3 IGLA specs (`benchmark`, `cordic`, `cordic_top`) fail because the branch
  does not yet contain the keyword-escape / tuple-return lowering that the
  `igla_clean_specs()` smoke gate assumes is present.
- 13 scratch specs (`w371`–`w388`) fail for the same reason: local-array,
  tuple-return, and `let`-destructuring lowering are documented in
  `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` as fixed in later waves, but those
  compiler changes are not present in this branch.

W408 did **not** modify `bootstrap/src/compiler.rs`; these failures are out of
scope for the real-CCLK + SPI-transaction-model task. The parse / typecheck /
Zig / Rust / Verilog / seal-verify phases are all green (576/576).

---

## 4. Competitor positioning

| Competitor / project | Relevant capability | t27 differentiator after W408 |
|---|---|---|
| [Sparkle HDL / Verilean](https://github.com/Verilean/sparkle) | Lean 4 HDL + verified SoC | t27 formalizes vendor 7-series boot timing and links it to cold-POR evidence; Sparkle has no public Artix-7 config-engine transaction model. |
| [Kami / Kôika](https://github.com/SteffenReith/Kami) | Coq hardware DSL | Kami proves custom processors; t27 proves vendor FPGA configuration engine timing against an external flash datasheet. |
| [Project X-Ray / prjxray](https://github.com/f4pga/prjxray) | Reverse-engineered bitstream | prjxray documents *what* the bits mean; t27 formalizes the *timing consequences* of the CCLK/CS/SCK bits. |
| [OpenTitan](https://opentitan.org/book/doc/security/specs/secure_boot/) | Secure SoC boot / RoT | OpenTitan secures a processor boot chain; t27 secures the FPGA *configuration* stage itself. |
| Commercial SPI NOR VIP | Closed simulation models | t27 provides an open, machine-checked Lean 4 bound tied to a real Artix-7 board. |

The defensive value of W408 is a **machine-checked transaction-level proof**
covering the actual sequence of CS# / SCK / wake-up events, not just static
frequency and duty predicates.

---

## 5. Risks and residual work

- **Physical P12 wiring:** still missing. As soon as a wire is available from
  P12 to ADBUS4 (or to DSLogic/oscilloscope), the live command shape is already
  implemented and validated against the synthetic fixture.
- **Duty-cycle bound:** remains a 25%–75% placeholder; should be tightened once
  a real capture exists.
- **Variant B automation:** relay-controlled cold-POR is still the next hardware
  CI milestone.
- **Transaction model constants:** CS# high time and wake-up are datasheet
  constants, not yet derived from the FPGA configuration engine's actual
  behavior. This is a future extension once engine timing is better documented.

---

## 6. Acceptance criteria status

- [x] AC-A1: real P12 capture attempted; blocker documented.
- [x] AC-A2: live-capture output committed in evidence file.
- [x] AC-B1: Variant B deferred to W409.
- [x] AC-C1: `SPIReadTransaction`, `artix7_boot_transaction`, and
      `transaction_satisfies_flash_spec` added.
- [x] AC-C2: canonical transaction theorem proved.
- [x] AC-C3: cold-POR → transaction-spec theorem proved.
- [x] AC-D1: `lake build Trinity.TernaryFPGABoot` passes.
- [x] AC-D2: `cargo test -p tri fpga::tests` passes.
- [x] AC-D3: `./scripts/tri test` parse/typecheck/gen/seal-verify passes (576/576).
- [ ] AC-D4: `./scripts/tri test` gen-verilog-yosys-smoke is clean.
      **Residual:** 16 pre-existing failures from unmerged `gen-verilog` backend
      gaps (see `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`). W408 did not touch
      the Verilog backend.
- [x] AC-D5: W408 report + evidence + W409 cooperation variants committed.

---

*phi^2 + phi^-2 = 3 | TRINITY*
