# Wave Loop 419 Report — Variant C fallback: VCD/CSV hardening, PVT envelope monotonicity, standalone lake workflow

**Issue:** #1357  
**Branch:** `wave-loop-419`  
**Variant:** C (bench still blocked: P12 unwired, DLC10 cable missing, no relay).  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 419 continued the FPGA/formal evidence line while the physical bench
remains unavailable. The wave delivered:

- **VCD `$comment` hardening**: the parser now treats only an exact `$end` token
  as the section terminator, so embedded `$end`-like strings inside comments no
  longer corrupt the signal dictionary. Added regression test for a comment that
  contains an embedded `$end` token.
- **CSV multi-channel support**: header-based voltage-column auto-detection now
  recognizes `cclk`, `vccint`, `vccaux`, `ain`, `a0`, `channel0` and the new
  `--csv-channel <name>` option lets the user explicitly select the active
  signal column in multi-channel instrument exports.
- **PVT envelope monotonicity/antitonicity proofs**: added Rust tests and Lean 4
  lemmas proving that the PVT-derated N25Q128_3V SCK half-period bound is monotone
  non-decreasing in temperature and antitone non-increasing in VCCINT inside the
  operating envelope.
- **Standalone lake-package workflow documentation**: added §3.6.16 to
  `fpga/HARDWARE_SSOT.md` with a complete worked example of generating a theorem
  and type-checking it in a minimal external `lake` package.
- **Bug fix in `--standalone` output**: removed the invalid
  `import Trinity.BitstreamConfig` line; the generated file now imports only
  `Trinity.TernaryFPGABoot` and type-checks in a fresh lake package.

No new silicon evidence was produced — the physical-evidence gap remains until
P12 is wired and the cable/relay hardware is available.

---

## What changed

### 1. VCD `$comment` hardening

`cli/tri/src/fpga.rs`:

- Replaced the heuristic section terminator (`ends_with("$end")` /
  `contains(" $end")`) with a per-token check that only flags a bare `$end`
  token as the end of a multi-line `$date`/`$version`/`comment` section.
- If the section starts and terminates on the same line, the in-section flag is
  cleared immediately so single-line headers such as `$date today $end` still
  work.
- Added `test_parse_vcd_comment_with_embedded_end_token`, which exercises a
  `$comment` block containing the literal text `$end` before the real terminator.

### 2. CSV multi-channel import

`cli/tri/src/fpga.rs`:

- Extended header-name auto-detection in `parse_cclk_csv_reader` to recognize
  `cclk`, `vccint`, `vccaux`, `ain`, `a0`, and `channel0` in addition to the
  existing `voltage`/`v`/`analog` names.
- Added `--csv-channel <name>` to `FpgaCmd::MeasuredToLean` and propagated it
  through `measured_to_lean`, `parse_csv_to_raw_ns`, `parse_cclk_csv`, and
  `parse_cclk_csv_reader`.
- When `--csv-channel` is supplied, the parser case-insensitively matches the
  requested substring against header names and uses that exact column as the
  signal value.
- Added `test_parse_cclk_csv_named_voltage_channel` (via the updated CSV test
  suite) to verify explicit channel selection on a multi-column export.

### 3. PVT envelope monotonicity / antitonicity

`cli/tri/src/fpga.rs`:

- Added `test_pvt_half_ns_monotone_in_temp`, which checks that for fixed VCCINT
  and process corner, `n25q128_min_sck_half_ns_pvt` does not decrease when
  temperature increases inside the operating envelope.
- Added `test_pvt_half_ns_antitone_in_vccint`, which checks that for fixed
  temperature and process corner, the bound does not increase when VCCINT
  increases inside the envelope.

`proofs/lean4/Trinity/TernaryFPGABoot.lean`:

- Added `pvt_half_ns_monotone_in_temp`: symbolic proof that the PVT half-period
  bound is monotone non-decreasing in temperature.
- Added `pvt_half_ns_antitone_in_vccint`: symbolic proof that the bound is
  antitone non-increasing in VCCINT.

### 4. Standalone lake-package workflow documentation

`fpga/HARDWARE_SSOT.md`:

- Added §3.6.16 "Standalone lake-package workflow for generated theorems (W419)".
- Documents the full flow: `tri fpga measure-cclk --synth --validate --json`,
  `tri fpga measured-to-lean --standalone --out ...`, creating a `lakefile.lean`
  that requires the local Trinity package, and `lake build`.
- Includes both absolute-path and relative-path `require` examples, a `--raw-ns`
  variant, and a warning not to drop `--standalone` files inside the
  `Trinity/` module path.

### 5. Bug fix: `--standalone` import

`cli/tri/src/fpga.rs`:

- Removed the invalid `import Trinity.BitstreamConfig` line from the
  `--standalone` template. `Trinity.BitstreamConfig` is a namespace inside
  `Trinity.TernaryFPGABoot`, not a standalone module.
- Updated `test_measured_to_lean_output_standalone` to assert the correct import.
- The existing `test_measured_to_lean_standalone_lake_package_builds` now
  type-checks the corrected generated file in a temporary lake package.

---

## Verification results

| Check | Result |
|-------|--------|
| `cargo check -p tri` | **PASS** |
| `cargo test -p tri vcd` | **PASS** (11 tests) |
| `cargo test -p tri csv` | **PASS** (11 tests) |
| `cargo test -p tri pvt` | **PASS** (9 tests) |
| `cargo test -p tri fpga::tests` | **PASS** (45 tests) |
| `cargo test -p tri test_measured_to_lean_standalone_lake_package_builds` | **PASS** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |
| Full `./scripts/tri test` | see `FPGA_LOOP_EVIDENCE_W419_2026-07-05.md` |

---

## Weak points

1. **Physical evidence gap remains.** P12 is still unwired, the DLC10 cable /
   relay hardware is still missing, so no real CCLK capture or automated
   cold-POR log was produced.
2. **PVT coefficients are still placeholders.** Monotonicity/antitonicity only
   guards the *shape* of the linear envelope; real Micron N25Q128_3V PVT curves
   may require different coefficients.
3. **VCD/CSV format diversity is best-effort.** The parsers now cover the
   observed `$comment` and multi-channel cases, but new vendor dialects may
   require further header-name additions.
4. **Gen-Verilog weak point #1245 remains partially unmerged.** The full fix set
   exists on `master` (commit `701d79b3b`) but has not been merged into the
   `wave-loop-419` branch; only safe sub-fixes are being back-ported per wave.
5. **Competitor velocity.** Sparkle HDL / Verilean continues to ship formally
   verified Lean 4 hardware blocks (RV32 divider, AXI4-Lite, crypto cores) and
   remains the closest competitor in the same design space.

---

## Competitor scan

The credible competition in the formally verifiable HDL space has not stood
still. The main entities and how t27 differs:

- **Sparkle HDL ([Verilean/sparkle](https://github.com/Verilean/sparkle))** —
  a Lean 4-embedded, type-safe, formally verifiable HDL compiler. It offers
  native theorem proving, dependent-type width safety, combinational-loop
  prevention by construction, and SystemVerilog generation. Recent commits prove
  the RV32 divider correct against its model and circuit. Sparkle is the closest
  direct competitor because it also uses Lean 4 as the proof host.
- **Clash** — Haskell-embedded HDL with strong types and built-in simulation; the
  explicit inspiration for Sparkle. Formal verification is external, not
  native inside the host language.
- **Chisel / FIRRTL** — mainstream Scala-embedded HDL. Stronger types than
  Verilog, but formal verification is external and FIRRTL transformations can
  obscure the generated RTL.
- **Bluespec SystemVerilog** — high-level HDL with strong semantics and
  correct-by-construction RTL generation; formal proofs require external tools.
- **Coq-based frameworks (Kami, Silver Oak / Project Oak)** — proof-assistant
  hardware specification and verification. Mature but not Lean-4-native and
  with smaller ecosystem traction in this specific space.
- **ACL2** — industrial hardware/software verification used by AMD/Intel; not a
  direct competitor for an open Lean 4 HDL stack.

t27's differentiation remains the only open pipeline that converts real
instrument exports (CSV/VCD) into Lean 4 proofs of FPGA flash-timing compliance,
anchored to a physical Artix-7 board and a falsifiable PVT envelope.

---

## Files touched

- `cli/tri/src/fpga.rs`
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
- `fpga/HARDWARE_SSOT.md`
- `docs/reports/WAVE_LOOP_419_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W419_2026-07-05.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W420_2026-07-05.md`
- `docs/NOW.md`
- `.trinity/current-issue.md`
- `.trinity/experience.md`

---

## Next steps

1. Open PR from `wave-loop-419` to `master` with body `Closes #1357` — **PR #1360**.
2. Run `./scripts/tri test` and `lake build Trinity.TernaryFPGABoot` in CI-like
   mode; merge when green.
3. After merge, create the W420 issue and branch `wave-loop-420`, choosing the
   cooperation variant per `FPGA_LOOP_COOPERATION_W420_2026-07-05.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
