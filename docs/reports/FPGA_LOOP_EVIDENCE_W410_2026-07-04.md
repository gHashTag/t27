# FPGA Loop Evidence — Wave Loop 410 (2026-07-04)

> Issue: [#1325](https://github.com/gHashTag/t27/issues/1325)  
> Branch: `wave-loop-410`  
> Goal: real P12 CCLK capture or physical `OSCFSEL=6,7` boot + measured-duty formal link.

---

## 1. Hardware state at start of W410

The QMTech Wukong V1 / XC7A200T board remains on the bench but the Digilent
DLC10 JTAG cable is still not connected to the host, so no program/flash or
STAT capture is possible. The P12 CCLK pin is also still not wired to a logic
analyzer channel, so a real CCLK measurement is impossible.

```
$ cargo run -p dlc10 -- idcode
Error: open DLC10
Caused by:
    DLC10 cable not found (VID=0x03FD)
```

Because of this, **Variant A (real P12 capture)** and the physical-boot portion
of **Variant C (`OSCFSEL=6,7` cold-POR)** are blocked for W410. The wave
delivers the formal-only half of Variant C: a `measured_cclk_satisfies_flash_spec`
predicate and a lemma that links any captured `(frequency, duty)` pair to
`transaction_satisfies_flash_spec`, plus the Rust infrastructure that produces
the JSON record used by that link.

---

## 2. Formal link added to `proofs/lean4/Trinity/TernaryFPGABoot.lean`

New definitions:

- `measured_cclk_period_ns (freq_hz : Nat)`
- `measured_cclk_low_ns (freq_hz : Nat) (duty_pct : Nat)`
- `measured_cclk_high_ns (freq_hz : Nat) (duty_pct : Nat)`
- `measured_boot_transaction (freq_hz : Nat) (duty_pct : Nat) (bits : Nat)`
- `measured_cclk_satisfies_flash_spec (freq_hz : Nat) (duty_pct : Nat) : Bool`
- `N25Q128_MIN_SCK_PERIOD_NS : Nat` (derived from the 50 MHz limit)

New lemmas:

- `measured_cclk_low_le_period` — low time never exceeds the conservative period.
- `measured_cclk_period_at_least_min_sck_period` — a compliant frequency gives
  a period ≥ 20 ns.
- `measured_cclk_satisfies_flash_spec_implies_transaction_ok` — the main link:
  a measured pair that passes the predicate produces a flash-spec-compliant
  transaction.
- Concrete examples for the synthetic fixture (`2.5 MHz, 50%`) and the nominal
  `OSCFSEL=6,7` rates (`25 MHz, 50%` and `33.3 MHz, 50%`).

Build result:

```
$ lake build Trinity.TernaryFPGABoot
✔ Built Trinity.TernaryFPGABoot
Build completed successfully (2967 jobs)
```

---

## 3. Rust `MeasuredCclk` formal-link export

`cli/tri/src/fpga.rs` now defines:

```rust
struct MeasuredCclk {
    freq_hz: u64,
    duty_pct: f64,
    sck_low_ns: u64,
    sck_high_ns: u64,
    source: String,
}
```

The conservative `sck_low_ns` / `sck_high_ns` values mirror the Lean
`measured_cclk_low_ns` / `measured_cclk_high_ns` definitions. A new
`--json` flag on `tri fpga measure-cclk` emits a JSON object that can be
pasted into the formal predicate.

Synthetic fixture output:

```
$ cargo run -p tri -- fpga measure-cclk --synth --validate --json
...
{
  "freq_hz": 2495000,
  "duty_pct": 50.0,
  "sck_low_ns": 200,
  "sck_high_ns": 200,
  "source": "synthetic (10000000 Hz samplerate)"
}
```

Rust tests:

```
$ cargo test -p tri fpga::tests
running 11 tests
test fpga::tests::test_measured_cclk_25mhz_50duty ... ok
test fpga::tests::test_measured_cclk_conservative_2_5mhz_50duty ... ok
test fpga::tests::test_measured_cclk_json_roundtrip ... ok
...
test result: ok. 11 passed; 0 failed
```

---

## 4. `./scripts/tri test` conformance summary

```
Parse: 576 passed, 0 failed
Typecheck: 576 passed, 0 failed
GF16 conformance: ok
Gen Zig: 576 passed, 0 failed
Gen Rust: 576 passed, 0 failed
Gen Verilog: 576 passed, 0 failed
Gen Verilog Yosys Smoke: 40 passed, 16 failed
FPGA Board-Less Smoke Gate: ok
Gen C: 576 passed, 0 failed
Seal Verify: 576 passed, 0 failed
Fixed Point: 0 divergences
```

The 16 yosys-smoke failures are pre-existing gen-verilog defects tracked in
`docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` and are outside the W410 scope.

---

## 5. What is still physically blocked

- **P12 → ADBUS4 wiring:** no real CCLK capture (Variant A).
- **DLC10 cable:** cannot program flash or run `tri fpga cclk-sweep` for
  `OSCFSEL=6,7` (physical half of Variant C).

These blockers are documented here and in `fpga/HARDWARE_SSOT.md` §3.6.1/§3.6.9.
Once the cable and wiring are available, the formal link added in W410 lets a
real capture immediately produce a `transaction_satisfies_flash_spec` proof.

---

*phi^2 + 1/phi^2 = 3 | TRINITY*
