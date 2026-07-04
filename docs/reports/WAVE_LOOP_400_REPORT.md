# Wave Loop 400 — FPGA SPI boot from flash verified, CCLK sweep closes cold-POR timing

**Issue:** #1300  
**Branch:** `trinity-rust-rings`  
**Date:** 2026-07-04  
**Target:** QMTech Wukong V1 / XC7A200T-FGG676-1, JTAG IDCODE `0x03636093`  

---

## 1. Goal

Execute the default Variant-A plan from [W400 cooperation variants](FPGA_LOOP_COOPERATION_2026-07-05.md):

1. Run a physical cold-POR CCLK sweep for the canonical `ternary_mac_demo_top_200t.bit`.
2. Identify the first working `OSCFSEL` value.
3. Measure/estimate CCLK and capture the working default bitstream.
4. Close W400 and publish W401 cooperation variants.

---

## 2. What was done

### 2.1 Sweep setup

Command (run on the host connected to the board):

```bash
./target/release/tri fpga cclk-sweep \
    /Users/playra/t27/fpga/verilog/ternary_mac_demo_top_200t.bit \
    --values 0,1,2,3,4,5 \
    --wait-seconds 120
```

This generated six COR0-patched variants (`OSCFSEL=0..5`), programmed each into
the on-board N25Q128 SPI flash, and then waited for the operator to perform the
cold-POR protocol:

1. Disconnect the JTAG/programming cable.
2. Disconnect board power.
3. Wait ≥10 s.
4. Reconnect board power, wait ≥2 s.
5. Reconnect the cable and press ENTER.

`STAT` was read with `--pre-jtag-reset` so no JTAG reset or `PROGRAM_B` pulse
was issued before sampling.

### 2.2 Raw result

| OSCFSEL | STAT       | MODE | DONE | EOS | CRC | ID  | Verdict |
|---------|------------|------|------|-----|-----|-----|---------|
| 0       | 0x401079FC | 001  | 1    | 1   | 0   | 0   | **PASS** |
| 1       | 0x401079FC | 001  | 1    | 1   | 0   | 0   | **PASS** |
| 2       | 0x401079FC | 001  | 1    | 1   | 0   | 0   | **PASS** |
| 3       | 0x401079FC | 001  | 1    | 1   | 0   | 0   | **PASS** |
| 4       | 0x401079FC | 001  | 1    | 1   | 0   | 0   | **PASS** |
| 5       | 0x401079FC | 001  | 1    | 1   | 0   | 0   | **PASS** |

All six variants reported `DONE=HIGH`, `MODE=0b001` (Master SPI x1), and no
CRC/ID errors. The clean report is committed at
`build/fpga/sweep-report-w400-clean.md` and the JSON logs are at
`build/fpga/boot-log-20260704-14*-oscfselNN.json`.

### 2.3 Interpretation

The canonical bitstream (`OSCFSEL=0`) already boots from flash when the
proper cold-POR protocol is followed. Earlier observations of `DONE=0`
(`STAT=0x5000190C`) were caused by **incomplete cold-POR or JTAG-cable
interference**, not by CCLK/SPI-startup timing. No COR0 patch is required for
the default.

The actual CCLK frequency was not measured in this wave because it requires a
logic-analyser/oscilloscope capture of pin **P12** (`CFGCLK` / `CCLK_0`). All
variants booting implies the default oscillator setting has enough margin; the
frequency measurement is now documentation work rather than a blocker.

---

## 3. Code / doc changes

- `cli/tri/src/fpga.rs`: `cclk-sweep` learned `--wait-seconds` and `--single`;
  `SweepLog`/`SweepSample` are now serializable/deserializable for
  `sweep-report`.
- `fpga/HARDWARE_SSOT.md`: §3.3 and §4 updated with the W400 result and the
  verified flash-boot status of the canonical bitstream.
- `docs/reports/WAVE_LOOP_400_REPORT.md`: this report.
- `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-08.md`: raw evidence summary.
- `docs/reports/FPGA_LOOP_COOPERATION_2026-07-08.md`: W401 cooperation variants.
- `build/fpga/sweep-report-w400-clean.md`: generated clean report.
- `build/fpga/boot-log-archive/`: stale dry-run/partial logs archived.

---

## 4. Conformance

Run before landing:

```bash
./scripts/tri test
```

Expected: **575/575 PASS**.

---

## 5. Closure

- The W400 issue is closed by this report.
- The canonical `ternary_mac_demo_top_200t.bit` is the working default; no
  separate CCLK-variant bitstream is required.
- CCLK frequency measurement and H2 documentation cleanup are deferred to W401
  (see cooperation variants).

Phase complete: W400 Verify/Synthesize
→ Phase 9: Learn (W401 planning)
