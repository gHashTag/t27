# FPGA Loop — Cooperation Variants for Wave Loop 412

> Prepared from Wave Loop 411 close-out  
> Target issue: #1330 (branch `wave-loop-412`)  
> Date: 2026-07-04

---

## Blockers inherited from W410–W411

1. **P12 not wired to a logic-analyzer channel.**
   - Without this we cannot capture real `CCLK` frequency/duty from the board.
2. **Digilent DLC10 JTAG cable not detected (`VID=0x03FD`).**
   - Without this we cannot program/flash the Artix-7, nor can we read `STAT` or
     boot logs from the board.

Until both blockers are resolved, all physical boot evidence is synthetic.
W411 delivered the **Variant C** fallback: `measured-to-lean` auto-proof
pipeline and conservative PVT-margin predicate.

---

## Variant A — Physical capture + `OSCFSEL=6,7` real boot

### Goal
Finally wire the bench, capture the real P12 CCLK waveform, boot from flash
with `OSCFSEL=6` and `OSCFSEL=7`, and feed the live `(frequency, duty)` pair
into `tri fpga measured-to-lean --json`.

### Tasks
- Wire P12 to the nearest available logic-analyzer channel.
- Verify DLC10 cable detection with `dlc10 idcode`.
- Run `tri fpga measure-cclk --json` on cold POR.
- Run `tri fpga flash ...` and `tri fpga boot-log --json` for `OSCFSEL=6`.
- Repeat for `OSCFSEL=7`.
- Run `tri fpga measured-to-lean --file measured.json --name w412_oscfsel6`.
- Commit the generated Lean theorem into `proofs/lean4/Trinity/TernaryFPGABoot.lean`.

### Acceptance criteria
- `docs/fpga/evidence/w412_oscfsel6_cclk.csv` exists.
- `docs/fpga/evidence/w412_oscfsel7_cclk.csv` exists.
- Two new `measured_...` theorems in `TernaryFPGABoot.lean` are `lake build`-green.
- `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.

### Complexity
**High** — blocked on hardware access; simple once the bench is wired.

---

## Variant B — Relay CI cold-POR gate

### Goal
Build a relay-controlled cold-POR automation layer so that the SPI boot gate
can be exercised in CI without manual board power cycling.

### Tasks
- Add `fpga/src/relay.rs` with a `PowerController` trait.
- Provide two implementations:
  - `HardwareRelay` (USB relay board via serial/hidraw).
  - `MockPowerController` for board-less CI.
- Add `tri fpga cold-por --relay-port PORT --oscfsel 6|7` subcommand.
- Integrate mock path into `./scripts/tri test --fpga-smoke`.
- Document relay wiring in `fpga/HARDWARE_SSOT.md` §3.6.12.

### Acceptance criteria
- `cargo test -p tri fpga::relay_tests` passes.
- `tri fpga cold-por --oscfsel 6 --relay-port MOCK` produces a JSON boot log.
- CI smoke gate runs on `MockPowerController` when no DLC10 is present.
- `./scripts/tri test` passes.

### Complexity
**Medium** — mostly Rust plumbing plus CI integration.

---

## Variant C — PVT-margin refinement + standalone measured-to-lean output

### Goal
Improve the formal tooling so that it is useful even without physical access.

### Tasks
- Replace the conservative 2× placeholder `N25Q128_MIN_SCK_*_NS_WC` constants
  with real Micron PVT derating curves if a public PVT table is found; otherwise
  document the assumption more precisely.
- Extend `tri fpga measured-to-lean` to emit a self-contained `.lean` file with
  imports, namespace, and theorem block, instead of just a snippet.
- Add a `--derive-period` mode that reads raw `(period_ns, low_ns, high_ns)`
  from a sigrok CSV instead of computing from frequency/duty.
- Add `measured_cclk_from_raw_ns_satisfies_flash_spec` predicate and chain it
  through the same implication theorems.

### Acceptance criteria
- New theorem examples for raw-ns input are `lake build`-green.
- `measured-to-lean --out TernaryFPGABootMeasured.lean` generates a
  self-contained file.
- `./scripts/tri test` passes.
- PVT-margin reasoning is documented in `fpga/HARDWARE_SSOT.md`.

### Complexity
**Low–Medium** — fully software-side, can be done while the bench is blocked.

---

## Recommended bundle

**Variant A + B together** when the bench becomes available.  
**Variant C alone** as the safe fallback if P12 or the DLC10 cable is still
unreachable.

---

*phi^2 + 1/phi^2 = 3 | TRINITY*
