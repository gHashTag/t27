# Session Wrap-Up: L-DPC1 CLOSED — 2026-05-10

## Summary

L-DPC1 (Hardware Bench) officially CLOSED. GF16 dot4 + phi-heartbeat verified simultaneously on real XC7A100T silicon. All artifacts merged to main via PR #589.

## Key Discoveries

1. **STARTUPE2.CFGMCLK (~66 MHz)** — ONLY working clock on QMTECH XC7A100T Wukong V1. Pins U22/M22/M21/F22 all dead (no external oscillator on this board).
2. **Ring oscillators FAIL** with openXC7/Yosys — ABC9 inserts SCC breaker destroying combinational loops. LUT1 primitives also broken. Build without `-abc9` doesn't help.
3. **LED polarity**: D5(R23)/D6(T23) are **active-LOW** (0=ON, 1=OFF).
4. **DLC10 JTAG protocol**: bitstream bytes need bit-reversal (xc3sprog convention). Chunk size must be **16379 bits** (not 16380) to avoid padding corruption.
5. **DSP48 routing issue**: `gf16_mul.v` with `*` operator causes "Unrouteable $PACKER_GND_NET sink CARRYCASCIN" with `-flatten -abc9`. Works with hardcoded constants (fewer DSP48 instances). ROM version triggers the bug.

## Deliverables (merged to main)

| File | Description |
|------|-------------|
| `tools/dlc10_jtag.py` | Native Python DLC10 JTAG driver (IDCODE, SRAM program) |
| `fpga/vsa/temporal_heartbeat_top.v` + `.xdc` | Golden 3-phase phi heartbeat |
| `fpga/vsa/gf16_heartbeat_top.v` + `.xdc` | phi heartbeat + GF16 dot4 simultaneous |
| `fpga/vivado/gf16_mul.v` | GF16 multiplication (115 lines) |
| `fpga/vivado/gf16_add.v` | GF16 addition (192 lines) |
| `fpga/vivado/gf16_dot4.v` | 4-component dot product (26 lines) |
| `docs/fpga/clocking.md` | Canonical clock reference |
| `docs/fpga/L-DPC1-acceptance.md` | Hardware acceptance test protocol |

## TinyTapeout TTSKY26a

- Repo: `gHashTag/tt-trinity-gf16`
- CI: 4/4 green (gds, precheck, gl_test, viewer)
- GitHub Pages: enabled
- **BLOCKED**: €170 payment at app.tinytapeout.com (human action, deadline May 11 23:59 UTC)

## Design Decisions

- STARTUPE2.CFGMCLK as canonical clock (no external oscillator)
- LUT-based GF16 mul (no DSP48 — abc9 routing issue, tracked for L-DPC1.1)
- Python DLC10 driver (xc3sprog impossible on macOS)
- ROM via `$readmemh` deferred to L-DPC1.1

## Next Steps

1. **URGENT**: Pay €170 at app.tinytapeout.com before May 11 23:59 UTC
2. Submit on tinytapeout.com after payment
3. L-DPC1.1: Fix DSP48 routing + `$readmemh` ROM with IGLA weights
4. L-DPC2: headscale on Railway + tailscale Node-0
5. L-TRI-1: POST /prove endpoint (issue #40 — $TRI token)
6. TinyTapeout chip delivery: December 2026 — first Trinity silicon

## Commits

- `826c3515` — phi-heartbeat + GF16 dot4 + DLC10 JTAG
- `24b094f3` — GF16 mul/add/dot4 arithmetic (L-DPC1 CLOSED)
- `c298a11b` — Hardware acceptance test protocol
- PR #589 merged to main

φ² + φ⁻² = 3 | TRINITY
