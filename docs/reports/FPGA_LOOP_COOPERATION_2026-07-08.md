# Wave Loop 401 — Cooperation Variants

**Context:** W400 physically verified that the canonical
`fpga/verilog/ternary_mac_demo_top_200t.bit` boots from flash when the
 disciplined cold-POR protocol is followed. All six `OSCFSEL` variants
(0..5) reached `DONE=1` with `MODE=0b001`, so CCLK frequency is **not** the
blocker. The remaining work is to document the exact CCLK frequency, harden
the cold-POR protocol, and make the success reproducible in CI.

---

## Variant A — Measure CCLK and lock the working default (default)

**Goal:** capture the actual CCLK frequency for the working default bitstream,
commit it as the canonical evidence, and update `fpga/HARDWARE_SSOT.md` so the
cold-POR success is the documented norm.

**Work**
1. Capture pin **P12** (`CFGCLK` / `CCLK_0`, bank 0) on a logic analyser during
the first ~100 µs after cold-POR.
2. Export the trace to CSV and run:
   ```bash
   tri fpga measure-cclk --csv build/fpga/dsview_cclk.csv
   ```
3. Update `fpga/HARDWARE_SSOT.md` §3.5 with the measured frequency and duty
cycle.
4. Confirm the canonical `ternary_mac_demo_top_200t.bit` is the committed
default; archive or delete the now-unnecessary COR0-patched variants.

**Acceptance**
- AC-A1: A measured CCLK frequency is recorded for `OSCFSEL=0`.
- AC-A2: `fpga/HARDWARE_SSOT.md` states that the default bitstream boots from
  flash and documents the cold-POR protocol as mandatory.

---

## Variant B — Harden the cold-POR protocol and make it testable without a board

**Goal:** ensure the cold-POR/JTAG-cable interference lesson is not lost, and
build board-less guards so regressions are caught before the next physical
session.

**Work**
1. Add `tri fpga boot-protocol --checklist` that prints the exact disconnect/
reconnect sequence and refuses to sample `STAT` if the user has not confirmed
each step.
2. Extend `tri fpga smoke-gate` to assert that the canonical bitstream has:
   - `IDCODE=0x03636093`
   - `SPI_BUSWIDTH=x1`
   - `STARTUPCLK=CCLK`
   - `OSCFSEL=0` (no unexpected COR0 patch)
3. Add a CI test that verifies `cclk-sweep --dry-run` and `sweep-report` produce
   the expected markdown structure.
4. Document the JTAG-cable interference failure mode in `fpga/HARDWARE_SSOT.md`
   with the observed `STAT` signature (`0x5000190C`) vs. success signature
   (`0x401079FC`).

**Acceptance**
- AC-B1: `tri fpga smoke-gate` fails if the canonical bitstream is patched or
  misconfigured.
- AC-B2: A new board-less CI guard exercises `cclk-sweep --dry-run` and
  `sweep-report`.

---

## Variant C — Reproduce boot-from-flash on a second board / second bitstream

**Goal:** prove the W400 result is not a one-off by reproducing it on another
physical artifact, and close the loop on the original GF16 matrix design.

**Work**
1. Use the same cold-POR protocol to boot a second QMTech Wukong V1 board from
   the same flash image, or re-flash the current board with the
   `gf16_matmul4x4_top.bit` bitstream and verify boot.
2. If a second board is unavailable, synthesize a fresh
   `ternary_mac_demo_top_200t.bit` from the current source and verify the new
   build also boots from flash.
3. Compare `STAT` values, `bit-config` output, and CCLK capture between the two
   runs.
4. Document any board-to-board variance in `docs/reports/FPGA_LOOP_EVIDENCE_...`.

**Acceptance**
- AC-C1: Boot-from-flash is reproduced on a second board or a freshly built
  bitstream.
- AC-C2: Variance (if any) is documented and bounded.

---

## Recommended choice

**Variant A** is the default because the only unresolved artifact from W400 is
the actual CCLK frequency. Capturing it closes the H2 timing chapter and lets the
repository declare a known-good default bitstream.

If a logic analyser is not immediately available, fall back to **Variant B** to
harden the protocol and CI guards so the W400 finding survives toolchain churn.

**Variant C** is the long-leverage verification step once Variant A is done and
a second board or fresh build is available.
