# Wave Loop 413 Plan

**Issue:** #1338  
**Branch:** `wave-loop-413`

## Decision gate

| Bench available? | Pick |
|--------------------|------|
| P12 + analyzer + DLC10 | Variant A + B |
| Analyzer only | Variant A + C |
| Cable only | Variant B + C |
| Nothing | Variant C |

Current state (2026-07-01): **nothing** — P12 not wired, DLC10 cable missing, DSLogic not attached. Execute **Variant C only**.

## Goals for Variant C

1. Close the instrument-to-proof gap: `tri fpga measured-to-lean --raw-ns` must accept sigrok/DSView CSV and VCD exports, not only hand-written JSON.
2. Replace the opaque PVT placeholder with an explicit, falsifiable uncertainty model that documents the conservative derating and the exact assumptions that would invalidate it.
3. Add a deterministic, clearly-labeled relay mock for `tri fpga cold-por` so CI can exercise the cold-POR JSON/log path without hardware.

## Work breakdown

### 1. CSV / VCD import for `measured-to-lean --raw-ns`

Files: `cli/tri/src/fpga.rs`, `cli/tri/Cargo.toml`

- Add a small zero-dependency VCD transition parser (no extra crate; parse `$var` / `$enddefinitions` / timestamp/value lines only).
- Add `parse_csv_to_raw_ns(path, samplerate)` and `parse_vcd_to_raw_ns(path, signal_name)` that return `(period_ns, low_ns, high_ns, source)`.
- Extend `FpgaCmd::MeasuredToLean` with optional `--csv` and `--vcd` args (mutually exclusive with `--file`).
- In `measured_to_lean`, when `--raw-ns` is set and a CSV/VCD is supplied, parse the instrument file, build a `MeasuredCclkRawNs`, then follow the existing JSON-to-Lean path.
- Validation: require `low_ns + high_ns = period_ns`; warn if the instrument's own reported totals disagree.

### 2. PVT uncertainty / falsification model

Files: `proofs/lean4/Trinity/TernaryFPGABoot.lean`, `fpga/HARDWARE_SSOT.md`

- Keep `N25Q128_MIN_SCK_LOW_NS_WC = 12` and `N25Q128_MIN_SCK_HIGH_NS_WC = 12` as the **current conservative 2× derating**.
- Document in Lean that these are placeholders derived from the nominal 6 ns bound multiplied by a worst-case factor of 2.
- Add a raw-ns PVT predicate `measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec` and a chain theorem `measured_cclk_from_raw_ns_with_pvt_implies_transaction_ok`.
- Add explicit falsification conditions: if a future N25Q128 PVT characterization shows `t_CL` or `t_CH` can exceed 12 ns under the operating envelope, the constant must be raised and the seal must be regenerated.
- Add concrete example theorems for 25 MHz / 50% duty under PVT margin and under worst-case PVT context.

### 3. Deterministic relay mock for `tri fpga cold-por`

Files: `cli/tri/src/fpga.rs`

- Add new subcommand `ColdPor { bit, relay_port, repeat, log_dir }`.
- `--relay-port MOCK` writes a deterministic boot log with `STAT=0x401079FC` (DONE=HIGH, MODE=001, no errors) and a `relay_mock: true` flag.
- Any other `--relay-port` value returns `not-implemented-yet` (real relay driver is Variant A/B scope).
- The mock log uses the same JSON schema as `boot-log` / `cclk-sweep` so `sweep-report` and downstream tooling remain compatible.

### 4. Tests

Files: `cli/tri/src/fpga.rs` (unit tests at end of file)

- `test_measured_to_lean_csv_raw_ns`: generate a synthetic 2.5 MHz logic CSV, call `measured_to_lean` via `--csv --raw-ns --standalone`, assert output contains the expected theorem names and `decide` proofs.
- `test_measured_to_lean_vcd_raw_ns`: generate a minimal VCD with a 25 MHz clock, call `--vcd --raw-ns --standalone`, assert expected theorem names.
- `test_cold_por_mock_relay`: invoke `cold-por --relay-port MOCK`, assert JSON log exists, `conclusion` is success, and `relay_mock` is `true`.
- Add Lean test build: `lake build Trinity.TernaryFPGABoot` must remain green.

### 5. Docs and close-out

Files: `docs/reports/WAVE_LOOP_413_REPORT.md`, `docs/reports/FPGA_LOOP_EVIDENCE_W413_*.md`, `docs/reports/FPGA_LOOP_COOPERATION_W414_*.md`, `docs/NOW.md`, `.trinity/experience.md`, `.trinity/current-issue.md`

- W413 report summarizing Variant C deliverables, verification, and blockers.
- Evidence doc with command transcripts (CSV/VCD import, mock relay, lake build, cargo test, tri test).
- Cooperation variants for W414 (A physical, B relay real, C further formal tooling).
- Update `NOW.md` and `current-issue.md` for W414.

## Weak points

1. **Physical evidence gap remains.** Variant C produces no real CCLK capture or cold-POR trace; it only hardens the toolchain path that will consume such evidence once the bench is available.
2. **PVT derating is still an assumption, not data.** The 2× factor is conservative but arbitrary. Real N25Q128 PVT curves may require larger margins at extreme temperature/voltage corners.
3. **VCD parsing is minimal.** Full VCD supports buses, real-valued analog traces, and multiple scopes. The initial parser handles single-bit scalar nets only; complex exports still need manual JSON conversion.
4. **Mock must never be confused with evidence.** The mock log carries an explicit `relay_mock: true` flag, but reviewers must still distinguish mock-generated reports from real boot-log reports.
5. **Branch is behind `master` after W412 merge.** Need to rebase or merge `master` into `wave-loop-413` before opening the PR.

## Competitor scan

- **Sparkle HDL / Verilean:** nearest formal-HDL competitors. They verify RTL against specs but do not publish a measured-to-Lean bridge for FPGA boot timing or PVT-aware flash constraints.
- **Koika / Kami (Bluespec in Coq):** focus on processor correctness, not 7-series configuration/boot timing.
- **SymbiYosys / Yosys formal:** bounded property checking over Verilog; unrelated to linking real logic-analyzer captures to datasheet timing.
- **Project Everest (HACL*, Vale):** verified cryptographic software/assembly, no FPGA boot timing.
- **t27 differentiation:** the only open pipeline that turns a sigrok/DSView/VCD instrument export into a machine-checked Lean 4 proof that the measured CCLK satisfies the flash timing spec, with explicit PVT uncertainty.

## Verification checklist

- [ ] `cargo test -p tri fpga::tests` passes (new + existing tests).
- [ ] `lake build Trinity.TernaryFPGABoot` passes from `proofs/lean4/`.
- [ ] `./scripts/tri test` passes (parse/typecheck/gen/seal-verify).
- [ ] `tri fpga measured-to-lean --csv <synth.csv> --raw-ns --standalone --out /tmp/M.lean` produces a buildable Lean file.
- [ ] `tri fpga measured-to-lean --vcd <clock.vcd> --raw-ns --standalone --out /tmp/M.lean` produces a buildable Lean file.
- [ ] `tri fpga cold-por --bit fpga/verilog/ternary_mac_demo_top_200t.bit --relay-port MOCK` writes a deterministic success log.

## Acceptance criteria

- `measured-to-lean --raw-ns` accepts CSV and VCD inputs and emits the same Lean theorem shape as JSON input.
- PVT model documents the placeholder nature and falsification conditions; theorems still close the transaction proof.
- `tri fpga cold-por --relay-port MOCK` runs without hardware and produces a deterministic, labeled mock boot log.
- All verification green; PR closes #1338.

---

*φ² + φ⁻² = 3 | TRINITY*
