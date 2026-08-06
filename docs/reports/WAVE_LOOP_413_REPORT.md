# Wave Loop 413 Report — Variant C fallback (CSV/VCD import, PVT falsification model, relay mock)

**Issue:** #1338  
**Branch:** `wave-loop-413`  
**Status:** closed via PR #1339 (W413)  
**Date:** 2026-07-04  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was requested

Continue the FPGA boot-evidence line from W412. Default plan was Variant A + B
(physical CCLK capture + relay CI gate) if the bench became available;
otherwise deliver Variant C:

1. Replace/document the PVT placeholder derating in Lean with a falsifiable
   uncertainty model.
2. Add sigrok / DSView / VCD import to `tri fpga measured-to-lean --raw-ns`.
3. Add a deterministic relay-mock CI path for `tri fpga cold-por`.

Bench state at start of W413: P12 unwired, DSLogic not attached, Digilent DLC10
JTAG cable still missing. Variant C was executed.

---

## What landed

### 1. CSV / VCD import for `tri fpga measured-to-lean --raw-ns`

- `cli/tri/src/fpga.rs`
  - New `--csv` and `--vcd` options on `FpgaCmd::MeasuredToLean`.
  - `parse_csv_to_raw_ns` reuses the existing logic/analog CSV parsers and
    converts the (freq, duty) estimate into a conservative integer
    `(period_ns, low_ns, high_ns)` triple.
  - `parse_vcd_to_raw_ns` is a small zero-dependency VCD transition parser that
    handles `$timescale`, single-bit `$var` declarations, timestamp lines, and
    scalar value changes. The first scalar net is used unless `--vcd-signal`
    is given.
  - Instrument imports are validated only for file presence and for having
    enough transitions; the generated Lean theorem is still the user's
    responsibility to check (the `decide` tactic will fail if the predicate
    is false).

- Examples:

  ```bash
  tri fpga measured-to-lean --csv cclk_capture.csv --raw-ns --standalone --out MeasuredCsv.lean
  tri fpga measured-to-lean --vcd cclk.vcd --raw-ns --standalone --out MeasuredVcd.lean
  tri fpga measured-to-lean --vcd cclk.vcd --vcd-signal cclk --raw-ns --standalone --out MeasuredVcd.lean
  ```

### 2. PVT falsification / uncertainty model

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - `N25Q128_MIN_SCK_LOW_NS_WC` / `N25Q128_MIN_SCK_HIGH_NS_WC` kept at 12 ns
    (2× the nominal 6 ns bound).
  - Added explicit comments describing the placeholder as a conservative
    worst-case derating and stating the falsification condition: raise the
    constants if real N25Q128_3V PVT characterization shows `t_CL`/`t_CH` can
    exceed 12 ns under the operating envelope.
  - Added `measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec` and the
    chain theorem `measured_cclk_from_raw_ns_with_pvt_implies_transaction_ok`.
  - Added concrete example theorems for 40/20/20 raw-ns captures under PVT
    margin and under an industrial-corner placeholder context.

### 3. Deterministic relay mock for `tri fpga cold-por`

- `cli/tri/src/fpga.rs`
  - New `FpgaCmd::ColdPor { bit, relay_port, repeat, wait_seconds, log_dir }`.
  - `--relay-port MOCK` writes a deterministic boot log with the canonical
    W400 success signature `STAT=0x401079FC`, `relay_mock: true`, and a
    clearly labeled conclusion. No hardware is touched.
  - Any non-`MOCK` relay port returns `not-implemented-yet` (real relay
    driver is Variant A/B scope).

- Example:

  ```bash
  tri fpga cold-por fpga/verilog/ternary_mac_demo_top_200t.bit --relay-port MOCK --log-dir build/fpga
  ```

### 4. Tests

- Added Rust unit tests:
  - `test_measured_to_lean_csv_raw_ns`
  - `test_parse_vcd_to_raw_ns_25mhz`
  - `test_measured_to_lean_vcd_raw_ns`
  - `test_cold_por_mock_relay`
- `cargo test -p tri fpga::tests` → 20/20 pass.

### 5. Documentation

- `fpga/HARDWARE_SSOT.md` §3.6.12 updated to cover CSV/VCD import, PVT
  falsification model, and the `cold-por --relay-port MOCK` CI path.
- `.claude/plans/wave-loop-413.md` expanded with decomposed plan, weak points,
  and competitor scan.

---

## Verification

| Check | Result |
|-------|--------|
| `cargo test -p tri fpga::tests` | 20/20 pass |
| `lake build Trinity.TernaryFPGABoot` | green (2967 jobs) |
| `./scripts/tri test` parse/typecheck/gen/seal-verify | pass |
| `./scripts/tri test` gen-verilog-yosys-smoke | 40 pass, 16 pre-existing failures |
| `tri fpga measured-to-lean --csv <synth.csv> --raw-ns --standalone` | produces valid Lean snippet |
| `tri fpga measured-to-lean --vcd <clock.vcd> --raw-ns --standalone` | produces valid Lean snippet |
| `tri fpga cold-por ... --relay-port MOCK` | deterministic success log |

---

## Blockers still open

- P12 is not wired to a logic-analyzer channel.
- Digilent DLC10 cable (`VID=0x03FD`) still not detected.
- No real cold-POR traces or relay hardware integration.

These remain for Variant A + B in a future wave when the bench is available.

---

## Learnings

- Instrument-to-proof bridges must tolerate the actual export formats users
  have (sigrok CSV, DSView, PulseView, Saleae, VCD) rather than forcing a
  manual JSON conversion step.
- PVT placeholders should expose their falsification conditions explicitly
  in both code comments and formal model comments; otherwise reviewers cannot
  tell whether the margin is data or convention.
- A deterministic mock is valuable for CI only if it is loudly labeled
  (`relay_mock: true`) and uses the same JSON schema as real logs so
  downstream report tooling stays compatible.

---

*φ² + φ⁻² = 3 | TRINITY*
