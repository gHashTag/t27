# Wave Loop 405 Report — FPGA flash-boot cold-POR smoke gate

> Issue: [#1311](https://github.com/t27/t27/issues/1311)  
> Branch: `wave-loop-405` → `master`  
> Date: 2026-07-10  
> Anchor: `phi^2 + phi^-2 = 3 | TRINITY`

---

## 1. Goal

Close the flash-boot verification gap left by W404. W404 added a cable-connected
SRAM smoke gate (`tri fpga smoke-gate --require-cable`); W405 extends it to a
cold-POR flash-boot gate that programs SPI flash, prompts the operator for a
physical power-cycle, captures the FPGA `STAT` register without JTAG reset, and
asserts the same `boot_success` predicate used by the Lean 4 model.

The default variant chosen was **Variant B**:

- Extend `tri fpga smoke-gate` with `--flash-boot`.
- `--flash-boot` implies `--require-cable`.
- Program the canonical bitstream to SPI flash with verify.
- Prompt for the documented cold-POR protocol.
- Capture `STAT` and assert `DONE=HIGH`, `MODE=0b001`, `EOS=1`, no CRC/ID/DEC
  errors.

Variants A (physical CCLK measurement on P12) and C (formal OSCFSEL/CCLK
bounds in Lean 4) are deferred to W406.

---

## 2. What changed

### `cli/tri/src/fpga.rs`

- `FpgaCmd::SmokeGate` gained `--flash-boot` and `--wait-seconds` flags.
- `smoke_gate()` now accepts `flash_boot` and `wait_seconds`.
- When `--flash-boot` is set:
  - The gate still detects the Digilent FTDI cable and XC7A200T board.
  - It delegates the cold-POR flash-boot verification to `cclk_sweep()` with a
    single `OSCFSEL=0` variant.
  - It verifies that at least one `SweepResult` has `done=true`.
- `cclk_sweep()` now returns `Result<Vec<SweepResult>>` so callers can assert
  on structured results instead of parsing text or reading side files.
- The CLI dispatch for `FpgaCmd::CclkSweep` bails if no variant reaches
  `DONE=HIGH`, preserving the previous user-visible behavior.

### Why delegate to `cclk_sweep`?

The first implementation used a direct `program_flash()` + `capture_stat()`
sequence that looked identical to the `cclk_sweep` cold-POR path. On the bench
it consistently returned `H2_CCLK_TIMING` (`STAT=0x5000190C`) even though the
same operator actions, the same bitstream, and the same openFPGALoader
invocations were used. Running `tri fpga cclk-sweep --single 0` on the same
bench immediately returned `STAT=0x401079FC` and `DONE=HIGH`. Reusing the
empirically-working `cclk_sweep` path inside the smoke gate resolved the
failure. The root cause of the behavioral difference remains subtle (likely
timing/ordering around stdin prompts and helper interaction), so the fix is to
reuse the proven path rather than duplicate it.

### Documentation

- `docs/NOW.md` — added W405 entry.
- `.trinity/experience.md` — captured W405 learnings.
- `.claude/plans/wave-loop-405.md` — acceptance criteria updated.
- This report and the companion evidence / cooperation files.

---

## 3. Verification

### 3.1 Conformance suite

```bash
./scripts/tri test
```

Result:

```text
Gen Verilog Yosys Smoke: 56 passed, 0 failed
Gen C: 576 passed, 0 failed
Seal Verify: 576 passed, 0 failed
TOTAL FAILURES: 0
ALL TESTS PASSED
phi^2 + phi^-2 = 3 | TRINITY
```

### 3.2 Hardware smoke gate

```bash
./target/debug/tri fpga smoke-gate --require-cable --flash-boot --wait-seconds 120
```

Result:

```text
[smoke-gate] cable OK (FPGA detected)
[smoke-gate] flash-boot: verifying cold-POR boot via single-variant CCLK sweep
[program-flash] Write complete. Reset or power-cycle the board to load from flash.
[cclk-sweep] PHYSICAL POWER-CYCLE REQUIRED
...
[stat] sample 1/3: raw=0x401079FC
[stat] sample 2/3: raw=0x401079FC
[stat] sample 3/3: raw=0x401079FC
=> First working variant: OSCFSEL=0 (.../ternary_mac_demo_top_200t_oscfsel00.bit)
[smoke-gate] flash-boot check OK (DONE=HIGH, mode=001, no errors)
[smoke-gate] yosys synthesis OK
[smoke-gate] complete
```

The captured `STAT` value `0x401079FC` decodes to:

| Field | Value |
|---|---|
| DONE | 1 |
| MODE | `0b001` (Master SPI x1) |
| EOS | 1 |
| INIT_COMPLETE | 1 |
| CRC_ERROR | 0 |
| ID_ERROR | 0 |
| DEC_ERROR | 0 |

This is exactly the `boot_success` predicate in `proofs/lean4/Trinity/TernaryFPGABoot.lean`.

Full evidence is recorded in `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-10.md`.

---

## 4. Competitor positioning

| Competitor / project | Relevant capability | t27 differentiator after W405 |
|---|---|---|
| Verilean | Lean 4 hardware proofs | t27 links the same Lean 4 `boot_success` predicate to a real cold-POR STAT capture on a real FPGA, not just a model |
| Sparkle HDL | End-to-end formal + simulation | t27 has a cable-connected smoke gate that now covers both SRAM load (W404) and cold-POR flash boot (W405) |
| openFPGALoader ecosystem | Tooling for flash / SRAM load | t27 wraps it with a spec-first CLI, formal traceability, and reproducible evidence reports |
| Project Trellis / nextpnr | Open-source bitstream tooling | t27 focuses on Artix-7 boot verification, not place-and-route competition |

The defensive value of W405 is that the flash-boot path is now reproducibly
covered by an automated gate. A competitor can match the formal model or the
hardware tooling alone, but the combination of a spec-first CLI, Lean 4
predicates, and physical cold-POR evidence is harder to reproduce.

---

## 5. Risks and residual work

- The smoke-gate `--flash-boot` mode still requires a human operator to
  disconnect/reconnect the JTAG cable and power-cycle the board. Full
  automation would need a relay-controlled power switch and a cable with
  galvanically isolated JTAG lines.
- The implementation reuses `cclk_sweep` because a direct path failed
  empirically; the exact root cause of that failure should be investigated so
  future helpers don't accidentally reintroduce it.
- Physical CCLK measurement (Variant A) and formal CCLK timing bounds (Variant
  C) are deferred to W406.

---

## 6. Acceptance criteria status

- [x] AC-B1: `tri fpga smoke-gate --require-cable --flash-boot` programs flash
      and asserts `boot_success` after a cold POR.
- [x] AC-D1: `./scripts/tri test` passes.
- [x] AC-D2: W405 report, evidence, and W406 cooperation variants committed.
- [ ] AC-A1 / AC-A2: deferred to W406.
- [ ] AC-C1: deferred to W406.

---

*phi^2 + phi^-2 = 3 | TRINITY*
