# Wave Loop 553 Report — Gate G1 done: the ternary MAC has a bitstream

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_552_REPORT.md`](WAVE_LOOP_552_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Four waves had recorded gate G1 as blocked. W553 re-tested the premise and
found the blocker was **Docker's** memory ceiling, not the machine's. Running
the memory-heavy step natively produced the chipdb, and from there the whole
flow completed.

**`fpga/verilog/ternary_mac_demo_top_v2_200t.bit` exists** — the design W549
built to be falsifiable and W552 proved invariants about now has a bitstream
for the actual board.

| | |
|---|---|
| Bitstream | 9,730,764 B, header confirms part `xc7a200tfbg676-1` |
| nextpnr-xilinx | **0 errors** |
| **Max frequency `cfgmclk`** | **150.63 MHz** (PASS at 80.00 MHz) — 1.88× margin |
| SLICE_LUTX | 120 / 269,200 (0 %) |
| SLICE_FFX | 60 / 269,200 (0 %) |

**Until now every frequency figure attached to IGLA RACE was a projection from
a model. This one is from place-and-route.**

---

## 1. The blocker was environmental, and the measurement proves it

```
bbaexport peak memory footprint : 7,064,369,664 B   (7.06 GB)
Docker Desktop allocation       :          3.83 GiB
host RAM                        :          8 GB
```

7.06 GB against a 3.83 GiB ceiling. No Docker tuning short of ~7.5 GiB would
have fixed it — and the host had the RAM the whole time. The fix was to split
the pipeline: the one memory-hungry step (`bbaexport.py`, pure Python) runs
natively; every other step stays in the container.

**The general lesson, recorded in the skill:** when a blocker is environmental
— memory, sandbox, a missing tool — ask *which specific environment* imposes it
and whether another is available. Do not carry "blocked" forward across waves
without re-testing the premise. W549 through W552 each restated G1 as blocked
without re-examining why.

## 2. The flow that works

1. **Extract** prjxray-db, nextpnr metadata, python and constids from the
   image into `build/fpga/openxc7/` (gitignored).
2. **`bbaexport` natively** → 981 MB `.bba`. This is the 7 GB step.
3. **`bbasm` in Docker** (small) → 332 MB chipdb `.bin`.
4. **yosys + nextpnr-xilinx + fasm2frames + xc7frames2bit in Docker** →
   bitstream.

Full copy-pasteable recipe: [`docs/fpga/IGLA_FPGA_LAUNCH_PLAN.md`](../fpga/IGLA_FPGA_LAUNCH_PLAN.md) §G1.

### Two gotchas, recorded so they cost nothing next time

- **`bbaexport.py` prints nothing when the OOM killer takes it.** Check `$?` —
  **137 means OOM**. Piping through `tail` hides the exit code, which is
  exactly what produced W549's two wrong diagnoses ("missing prjxray database",
  then "unset `XRAY_DATABASE_DIR`").
- **nextpnr-xilinx's XDC reader supports only `get_ports` and `get_nets`.** The
  Vivado-legal `create_clock … [get_pins startup/CFGMCLK]` errors with *"targets
  other than 'get_ports' or 'get_nets' are not supported"*. Re-addressed to
  `[get_nets cfgmclk]`, which Vivado also accepts.

## 3. Board profile updated

`wukong-a200t` now defaults to the v2 bitstream. v1 is superseded and should
not be flashed: its LEDs are driven at ~10⁸ Hz from a ring oscillator and its
accumulate path is tied off, so flashing it could never distinguish a working
MAC from a dead one.

`t27c fpga-flash --dry-run` passes every pre-flight check against the new
bitstream and reports `BLOCKED` for exactly one reason: `openFPGALoader
--scan-usb` finds no programmer.

---

## 4. Secondary: 13 corrupted module declarations repaired

Found while auditing the seal store, which contains files literally named
`"[]const u8".json`, `Str = "",.json` and `[]const u8.json`. The seal writer
was faithful; the **module names** are corrupted:

```
specs/tri/utils/logger.t27:5          module "[]const u8";
specs/tri/utils/args.t27:5            module []const u8;
specs/tri/agent/agent_run.t27:5       module Str = "",;
specs/tri/agent/memory.t27:5          module ;
specs/tri/agent/governance_agent.t27  module String  # phi, trinity, ... ;
```

Same corruption family as W550's type annotations, landed in the `module`
declaration. **All 13 parsed fine** — the parser accepts a string literal, a
type annotation, an assignment expression and an empty name as a module name.
That laxity is why it survived; the only visible symptom was pathological seal
filenames.

No clean version exists in git history, so the intended names were recovered
from the seal store, which recorded them before the corruption. Verified no
spec `use`s any of them; all 13 reseal cleanly under W552's new gate.

### Seal store, further measured

- **1,714 seals for 1,063 specs.** 548 spec_paths carry **more than one seal**
  (585 redundant files) because the filename convention changed from
  `<Module>.json` to `<parentdir>_<Module>.json` without cleanup.
- **91 orphans**: 89 whose spec was deleted (real hashes left behind), 2 whose
  spec never existed in tracked history.

Neither is repaired here — deleting seals loses provenance and is the
maintainer's call.

---

## 5. Three cooperation variants for W554

### Variant A (recommended) — Flash it, or make the last mile ready

**Hypothesis.** Every software obstacle between the repository and a running
board is now gone: the design is proven (T1–T3), the bitstream exists, the
flash command works and pre-flights cleanly. What remains is one USB cable.

**Deliverables — with the board attached:**
1. G2: `openFPGALoader --scan-usb` shows `0403:6014`; `--detect` reports IDCODE
   `0x03636093`.
2. G3: `t27c fpga-flash --board wukong-a200t --mode sram`.
3. Record the observed LED signature against T3's prediction: `led_r23`
   blinking ≈1 Hz, `led_t23` dark. **Commit the photograph or logic-analyzer
   capture as the witness.**

**Deliverables — without the board** (so the wave is not wasted):
4. Build the second bitstream: the same flow for `ternary_mac_demo_top` (v1) so
   the two can be compared on silicon later, or for a GF16 cell.
5. Script the extraction + native-chipdb steps as a `t27c fpga-chipdb`
   subcommand so the recipe cannot rot.

**What would falsify it.** A lit `led_t23` contradicts T3, which was proved by
temporal induction. That would mean silicon disagrees with a machine-checked
model — the most valuable outcome available anywhere in this project.

### Variant B — Clear the LANG-EN gate, then the syntax gaps

Unchanged since W550 and still **blocked on a human decision**. Six committed
documents violate L3 and are not allowlisted, so `build.rs` panics on any
`compiler.rs` edit. Once cleared: float-cast patch (~16 specs),
block-expression production (~40), struct-literals-in-expression (~28) — about
84 of the 326 remaining parse failures.

This is now the **largest remaining item in the whole project** and it is
purely waiting on an approval.

### Variant C — Resolve the `given`/`when`/`then` question

Unchanged since W550. 327 specs use a test form with no parser production that
is specified in `SOUL.md`, the language RFC and the TDD contract. Whether those
blocks are recognised as tests, skipped, or mis-parsed determines whether every
"N tests in spec X" figure is meaningful — the third integrity claim in the
same family as the vacuity and seal findings, and still unexamined.

---

## Recommendation

**Variant A.** The hardware track has never been this close, and the remaining
step is physical. If the board cannot be attached this wave, its second half
(a `fpga-chipdb` subcommand, a second bitstream) still moves it forward. B is
the biggest software win and needs only an approval to start.

---

*φ² + φ⁻² = 3 | TRINITY*
