# Wave Loop 418 Report — Variant C fallback: PVT regression, instrument import, and standalone Lean integration

**Issue:** #1353  
**Branch:** `wave-loop-418`  
**Variant:** C (bench still blocked: P12 unwired, DLC10 cable missing, no relay).  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 418 kept the FPGA/formal pipeline moving while the physical bench
remains unavailable. The wave delivered:

- A **PVT-envelope lower-bound regression** in Rust and a matching Lean 4 lemma,
  guarding the N25Q128_3V SCK half-period bound across the operating rectangle.
- **VCD parser hardening** for multi-line `$date`/`$version`/`$comment` header
  sections.
- **Analog CSV voltage-column auto-detection** by header name, fixing multi-channel
  imports where the signal is not the second column.
- A **standalone Lean integration test** that writes a synthetic raw-ns capture,
  generates a self-contained `.lean` file with `--standalone`, and type-checks it
  inside a temporary `lake` package.
- Updated `fpga/HARDWARE_SSOT.md` with a first-real-capture checklist and a recipe
  for replacing the placeholder PVT coefficients once real N25Q128_3V PVT data
  is available.

No new silicon evidence was produced — the physical-evidence gap remains until
P12 is wired and the cable/relay hardware is available.

---

## What changed

### 1. PVT-envelope lower-bound regression

`cli/tri/src/fpga.rs`:

- Added `Clone` derive to `ProcessCorner` so contexts can be iterated in tests.
- Added `test_pvt_half_ns_lower_bound_across_operating_rectangle`, which samples
  temp ∈ {-40, -20, 0, 25, 85} °C, vccint ∈ {900, 950, 1000, 1050, 1100} mV,
  and corners {ff, tt, ss}, then asserts
  `n25q128_min_sck_half_ns_pvt(ctx) >= 6` for every point.

`proofs/lean4/Trinity/TernaryFPGABoot.lean`:

- Added `n25q128_min_sck_half_ns_pvt` as the symmetric half-period bound.
- Added `pvt_half_ns_at_least_nominal`, a symbolic proof that the PVT-aware bound
  is at least the nominal 6 ns across the operating envelope.

### 2. VCD header-section skip

`cli/tri/src/fpga.rs`:

- Added per-line state variables `in_date`, `in_version`, `in_comment` in
  `parse_vcd_to_raw_ns`.
- Multi-line header sections are skipped entirely so their free-form contents
  are never mistaken for `$var` declarations or value changes.
- Added `test_parse_vcd_multiline_header_sections_skipped` to verify a vendor-style
  multi-line header is parsed correctly.

### 3. Analog CSV voltage-column detection

`cli/tri/src/fpga.rs`:

- Extended `parse_cclk_csv_reader` to recognize header tokens `voltage`, `v`,
  and `analog` and use the named column as the signal value.
- Added `header_named_columns` flag so the first-data-row numeric fallback does
  not override an explicitly named voltage column.
- Added `test_parse_cclk_csv_named_voltage_column` with a three-column CSV where
  the voltage signal is the third column.

### 4. Standalone Lean integration test

`cli/tri/src/fpga.rs`:

- Added `test_measured_to_lean_standalone_lake_package_builds`, which:
  1. writes a synthetic `MeasuredCclkRawNs` JSON record,
  2. calls `measured_to_lean` with `--standalone --raw-ns`,
  3. creates a temporary `lake` package that requires the local Trinity library,
  4. copies the generated `.lean` file as the package's main module,
  5. runs `lake build` and asserts success.

This proves the `--standalone` output is consumable outside the monorepo.

### 5. HARDWARE_SSOT.md updates

`fpga/HARDWARE_SSOT.md`:

- Added §3.6.14 "First real CCLK capture checklist" covering wiring, capture,
  PVT context recording, theorem generation, and typechecking.
- Added §3.6.15 "Replacing the placeholder PVT envelope coefficients" with a
  table of current coefficients and a step-by-step recipe for updating them once
  real N25Q128_3V PVT curves are available.

---

## Verification results

| Check | Result |
|-------|--------|
| `cargo check -p tri` | **PASS** |
| `cargo test -p tri pvt` | **PASS** (3 PVT tests) |
| `cargo test -p tri vcd` | **PASS** (11 VCD tests) |
| `cargo test -p tri csv` | **PASS** (10 CSV tests) |
| `cargo test -p tri test_measured_to_lean_standalone_lake_package_builds` | **PASS** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** |
| Full `lake build` | blocked by pre-existing `Trinity.NeutrinoMasses` / `Trinity.H4Lagrangian` failures (unrelated to W418) |

---

## Weak points

1. **Physical evidence gap remains.** P12 is still unwired and the DLC10 cable /
   relay hardware is still missing, so no real CCLK capture or cold-POR log was
   produced.
2. **PVT coefficients are still placeholders.** The operating-rectangle regression
   only guards the lower bound; real Micron curves may require different or larger
   margins.
3. **VCD header handling is best-effort.** It covers the most common single-line
   and multi-line `$date`/`$version`/`$comment` variants, but exotic tool dialects
   may need further hardening.
4. **Analog CSV format diversity.** The parser supports the documented
   Saleae/DSView/PulseView layouts; other instruments may need header-name
   additions.

---

## Competitor scan

No new competitor activity was observed. The t27 differentiation remains the
only open pipeline converting instrument exports (CSV/VCD) into Lean 4 proofs
of flash timing compliance, now strengthened by:

- a falsifiable, monotonic PVT envelope with operating-rectangle regression,
- multi-format VCD header support,
- analog CSV voltage-column auto-detection,
- a self-contained `--standalone` theorem that type-checks in a fresh lake package.

---

## Files touched

- `cli/tri/src/fpga.rs`
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
- `fpga/HARDWARE_SSOT.md`
- `docs/reports/WAVE_LOOP_418_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W418_2026-07-04.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W419_2026-07-04.md`

---

## Next steps

1. Open PR from `wave-loop-418` to `master` with body `Closes #1353`.
2. After merge, create the W419 issue and branch.
3. Evaluate bench status for W419 and pick Variant A/B/C per the cooperation file.

---

*φ² + φ⁻² = 3 | TRINITY*
