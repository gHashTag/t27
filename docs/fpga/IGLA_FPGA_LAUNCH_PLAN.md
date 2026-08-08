# IGLA on real FPGA — improvement and launch plan

**Wave:** 549
**Date:** 2026-08-09
**Anchor:** φ² + φ⁻² = 3 | TRINITY
**Board SSOT:** [`fpga/HARDWARE_SSOT.md`](../../fpga/HARDWARE_SSOT.md) — when that file and this one disagree, it wins.

> Every number in §1 was measured on this host during Wave 549, not carried
> over from an earlier document. Where a claim could not be measured, it is
> labelled **UNVERIFIED** rather than asserted.

---

## 1. Measured starting state

### 1.1 Host toolchain

| Tool | State | Consequence |
|------|-------|-------------|
| `yosys` 0.63 | present | synthesis works locally |
| `iverilog` / `vvp` | present | RTL simulation works locally |
| `openFPGALoader` | present | programming works — **once a cable exists** |
| `openocd` | present | alternate programming path |
| `docker` | present, no containers running | openXC7 P&R path available |
| `nextpnr-xilinx` | absent locally; present in the `regymm/openxc7` image | P&R via Docker — **working, G1 done** |
| `vivado` | **absent** | no local Vivado flow |

### 1.2 Hardware

```
$ openFPGALoader --scan-usb
empty
No USB devices found
```

**No programmer and no board are attached to this host.** Everything in
gate **G2** and beyond is therefore blocked on physically connecting the
QMTech Wukong V1 and its Digilent HS2 cable. This is the single gating
dependency for the entire "run on real silicon" claim.

Target per the SSOT:

| Field | Value |
|-------|-------|
| Board | QMTech Wukong V1 |
| FPGA | XC7A200T-FGG676 |
| Part string | `xc7a200tfgg676-1` |
| JTAG IDCODE | `0x03636093` |
| Cable | Digilent FTDI `0x0403:0x6014` → openFPGALoader profile `digilent_hs2` |
| Serial node | none — there is no `/dev/cu.usb*` on this host |

### 1.3 Artifacts that already exist

| Artifact | Size | Note |
|----------|------|------|
| `fpga/verilog/ternary_mac_demo_top_200t.bit` | 9,730,768 B | XC7A200T — matches the connected board |
| `fpga/verilog/ternary_mac_demo_top.bit` | 3,825,907 B | XC7A100T — legacy, **wrong part for this board** |
| `fpga/verilog/ternary_mac_synth.v` | — | the MAC cell itself, hand-written and synthesis-ready |

A loadable bitstream for the right part already exists. The bottleneck has
never been synthesis — it is that nothing has ever been flashed and checked.

---

## 2. The finding that reshapes this plan

**The v1 demo cannot demonstrate that the ternary MAC works, even if flashed
successfully.** Read `fpga/verilog/ternary_mac_demo_top.v` against
`ternary_mac_synth.v` and three independent defects fall out:

1. **The clock is a ring oscillator.** A 20-stage `LUT1` inverter chain closed
   on itself, with `ALLOW_COMBINATORIAL_LOOPS TRUE` and
   `CLOCK_DEDICATED_ROUTE FALSE`. Its frequency is process/voltage/temperature
   dependent and unconstrained, so no Fmax can be reported and no timing claim
   survives review.

2. **The output is not observable.** `led_r23 = ~acc_out[0]` and
   `led_t23 = ~acc_out[1]`. With `acc_in` tied to 0 and `w_code` tied to `+1`,
   `acc_out` tracks the free-running counter, so those bits toggle at
   `f_osc/2` and `f_osc/4` — on the order of 10⁸ Hz. **Both LEDs sit at
   roughly 50 % brightness.** A working board and a dead datapath look
   identical to the eye.

3. **The interesting logic is never exercised.** `w_code` is constant `2'b01`,
   so the `is_minus` and zero-decode branches never activate; `acc_in` is
   constant `0`, so the accumulator never accumulates. Synthesis is entitled
   to constant-fold both away. The design proves the toolchain emits a
   loadable bitstream — nothing about ternary arithmetic.

Flashing v1 would therefore produce a photograph, not evidence.

### 2.1 What Wave 549 built instead

`fpga/verilog/ternary_mac_demo_top_v2.v`, with a self-checking testbench and a
constraints file. It fixes all three defects:

- Clock is `STARTUPE2`/`CFGMCLK` — the same characterized primitive the working
  `phi_temporal` heartbeat uses — so the clock net can be constrained
  (`ternary_mac_demo_top_v2.xdc` sets a deliberately pessimistic 12.5 ns).
- A 24-bit prescaler steps the datapath at ≈3.9 Hz, so state changes are
  visible to a human.
- The weight sequence cycles `{+1, 0, −1, 0}` — covering both zero encodings —
  and `acc_out` feeds back into `acc_in`, so the accumulator genuinely
  accumulates and every decode branch stays live.
- LEDs encode whole-accumulator predicates (`acc != 0`, `acc < 0`) rather than
  raw low bits, so a stuck or folded datapath is immediately visible.

**Measured, this host, Wave 549:**

| Check | Result |
|-------|--------|
| `iverilog` + `vvp` self-check | **12 / 12 PASS** |
| `yosys synth_xilinx -abc9 -nocarry -arch xc7` | clean |
| Resources | 113 LUT (3×LUT1, 21×LUT2, 19×LUT3, 24×LUT4, 19×LUT5, 27×LUT6), 60 FF (32 FDCE + 28 FDRE), 1 STARTUPE2, 1 BUFG, 2 OBUF, 190 cells total |

Reproduce:

```bash
cd fpga/verilog && iverilog -g2005 -o /tmp/tb_v2.vvp tb_ternary_mac_demo_v2.v ternary_mac_demo_core.v ternary_mac_synth.v && vvp /tmp/tb_v2.vvp
```

**Done in W553:** v2 has a bitstream — `ternary_mac_demo_top_v2_200t.bit`,
placed and routed with 0 errors at **150.63 MHz** against an 80 MHz constraint.
See gate **G1**.

---

## 3. The launch gates

Each gate has an entry condition, an exact command, and a pass criterion that
someone other than the author could check. A gate is not "done" because the
command ran; it is done when the pass criterion is observed.

### G0 — Simulation parity *(status: **DONE**, Wave 549)*

- **Command:** the `iverilog`/`vvp` line above.
- **Pass:** `=== ALL TESTS PASSED ===`, 12 checks.
- **Why it matters:** the on-board signature in G3 is only meaningful because
  simulation already fixed what the correct signature *is*.

### G1 — Bitstream for v2 *(status: **DONE**, Wave 553)*

The `regymm/openxc7` image (11.3 GB) has been pulled and **verified to
synthesize this design**: 188 cells, 115 LUT, 60 FF, 1 STARTUPE2, under
yosys 0.62 inside the container.

**Solved in W553 by running the memory-heavy step natively instead of in
Docker.** The measurement settles the diagnosis:

| | |
|---|---|
| `bbaexport` peak memory | **7,064,369,664 B (7.06 GB)** |
| Docker Desktop allocation | 3.83 GiB |
| Host RAM | 8 GB |

7.06 GB against a 3.83 GiB ceiling — the OOM was not a misconfiguration, and no
Docker tuning short of ~7.5 GiB would have fixed it. Natively it completes.

**The flow that works** (artifacts land in `build/fpga/openxc7/`, gitignored):

```bash
# 0. one-time: extract prjxray-db + nextpnr metadata + python from the image
cid=$(docker create regymm/openxc7)
docker cp "$cid:/nextpnr-xilinx/xilinx/python"                              build/fpga/openxc7/
docker cp "$cid:/nextpnr-xilinx/xilinx/constids.inc"                        build/fpga/openxc7/
docker cp "$cid:/nextpnr-xilinx/xilinx/external/prjxray-db/artix7"          build/fpga/openxc7/prjxray-db/
docker cp "$cid:/nextpnr-xilinx/xilinx/external/nextpnr-xilinx-meta/artix7" build/fpga/openxc7/meta/
docker rm "$cid"

# 1. chipdb -- NATIVELY, this is the step that needs 7 GB
cd build/fpga/openxc7
python3 python/bbaexport.py --xray prjxray-db/artix7 --metadata meta/artix7 \
        --constids constids.inc --device xc7a200tfbg676-1 --bba xc7a200tfbg676.bba
# 981 MB .bba

# 2. assemble -- Docker is fine, this step is small
docker run --rm -v "$PWD:/out" regymm/openxc7 \
  bbasm --le /out/xc7a200tfbg676.bba /out/xc7a200tfbg676.bin      # 332 MB

# 3. synth + P&R + bitstream -- Docker
docker run --rm -v "$REPO:/work" -w /work regymm/openxc7 bash -c '
  source /prjxray/env/bin/activate
  /yosys/yosys -q -p "read_verilog fpga/verilog/ternary_mac_demo_top_v2.v \
        fpga/verilog/ternary_mac_demo_core.v fpga/verilog/ternary_mac_synth.v; \
        synth_xilinx -abc9 -nocarry -arch xc7 -top ternary_mac_demo_top_v2; \
        write_json build/synth-gate/v2_openxc7.json"
  /nextpnr-xilinx/nextpnr-xilinx --chipdb build/fpga/openxc7/xc7a200tfbg676.bin \
        --json build/synth-gate/v2_openxc7.json \
        --xdc fpga/verilog/ternary_mac_demo_top_v2.xdc \
        --write build/fpga/v2_routed.json --fasm build/fpga/v2.fasm
  python3 /prjxray/utils/fasm2frames.py \
        --db-root /nextpnr-xilinx/xilinx/external/prjxray-db/artix7 \
        --part xc7a200tfbg676-1 build/fpga/v2.fasm > build/fpga/v2.frames
  xc7frames2bit --part_name xc7a200tfbg676-1 \
        --part_file /nextpnr-xilinx/xilinx/external/prjxray-db/artix7/xc7a200tfbg676-1/part.yaml \
        --frm_file build/fpga/v2.frames \
        --output_file fpga/verilog/ternary_mac_demo_top_v2_200t.bit'
```

**Result — measured, Wave 553:**

| | |
|---|---|
| Bitstream | `fpga/verilog/ternary_mac_demo_top_v2_200t.bit`, 9,730,764 B |
| Header | part `xc7a200tfbg676-1`, generator `xc7frames2bit` |
| nextpnr | 0 errors |
| **Max frequency `cfgmclk`** | **150.63 MHz (PASS at 80.00 MHz)** — 1.88× margin |
| SLICE_LUTX | 120 / 269,200 (0 %) |
| SLICE_FFX | 60 / 269,200 (0 %) |

Until now every frequency figure attached to IGLA RACE was a projection from a
model. This one is from place-and-route.

**Two gotchas, recorded:**

- `bbaexport.py` prints nothing when the OOM killer takes it. **Check `$?`;
  137 means OOM.** Piping through `tail` hides it, which cost W549 two
  misdiagnoses.
- nextpnr-xilinx's XDC reader supports only `get_ports`/`get_nets`. The
  original `create_clock … [get_pins startup/CFGMCLK]` errored with *"targets
  other than 'get_ports' or 'get_nets' are not supported"*; re-addressed to
  `[get_nets cfgmclk]`, which Vivado also accepts.

### G2 — Cable and board detected *(status: BLOCKED on hardware)*

```bash
openFPGALoader --scan-usb
openFPGALoader --cable digilent_hs2 --detect
```

- **Pass:** scan lists `0403:6014`, and `--detect` reports IDCODE
  `0x03636093`.
- **Failure mode to expect:** an IDCODE of `0x13631093` means the board is a
  100T, and the 200T bitstream must not be loaded — switch to
  `--board wukong-a100t`.

### G3 — Volatile load and signature check *(status: BLOCKED on G1+G2)*

```bash
./target/release/t27c fpga-flash --board wukong-a200t \
    --bitstream fpga/verilog/ternary_mac_demo_top_v2_200t.bit --mode sram
```

`t27c fpga-flash` is new in Wave 549 (it was documented in
`QMTECH_A100T_SMOKE.md` and `TASK.md` for months but did not exist). It runs
pre-flight checks — bitstream present and non-empty, loader on `PATH`,
programmer actually attached — and refuses with an actionable message rather
than a driver error. `--dry-run` performs every check except programming, so
the command is testable with no hardware:

```bash
./target/release/t27c fpga-flash --dry-run
```

- **Pass criterion (the whole point of v2):** `led_r23` blinks at ≈1 Hz with a
  50 % duty cycle; `led_t23` stays dark.
- **Interpretation:**
  - `led_r23` steady-on or steady-off → the accumulator is not accumulating.
  - `led_t23` lit → the minus-weight path is producing a negative accumulator
    when the sequence says it should not.
  - both LEDs mid-brightness → a v1 bitstream was loaded by mistake.

### G4 — Persistent SPI boot *(status: BLOCKED, upstream)*

```bash
./target/release/t27c fpga-flash --board wukong-a200t --mode flash \
    --bitstream fpga/verilog/ternary_mac_demo_top_v2_200t.bit
```

- **Known blocker:** writing SPI flash needs a JTAG-to-SPI proxy bitstream.
  `OPENXC7_FGG676_STATUS.md` records that nextpnr-xilinx **crashes**
  (`std::out_of_range: dict::at()`) when the dedicated configuration pins
  (`C8`/`B19`/`A18`) are LOC-ed, because openXC7 does not model the
  `STARTUPE2`-driven config-pin path. Upstream `openFPGALoader#663` says to
  regenerate the FGG676 proxy with Vivado. **This is a Vivado-only artifact
  today.** Do not spend another wave on the openXC7 route until
  `pack_clocking_xc7.cc` grows config-pin support.
- **Secondary blocker:** the mode-pin strap state on the Wukong V1 is
  undocumented; `HARDWARE_SSOT.md` §"Cold-POR mode-pin sampling" already
  ranks this as the most likely SPI-boot failure cause.
- **Recommendation:** treat G4 as out of scope until G3 has been observed at
  least once. A persistent boot of an unverified design is not progress.

---

## 4. Weak points in the surrounding claims

Measured during Wave 549; each is a place the repository asserted something
that was not true.

| # | Claim | Reality | Status |
|---|-------|---------|--------|
| 1 | `TASK.md` line 90: `t27c fpga-flash` CLI — "Done" | the subcommand did not exist | **fixed** (implemented) |
| 2 | `QMTECH_A100T_SMOKE.md` step 1 tells the operator to run `t27c fpga-flash` | same | **fixed** |
| 3 | `t27c fpga-build --device` defaults to `xc7a100tcsg324-1` | that is the **Arty A7** package; the SSOT explicitly forbids mixing `csg324` into Wukong flows | **open** — see W550 Variant A |
| 4 | `QMTECH_A100T_SMOKE.md` documents a UART loopback smoke test on `/dev/ttyUSB0` | the SSOT records **no serial node exists** on this host | **open** |
| 5 | The smoke doc targets `qmtech-a100t` | the connected board is a **200T** since 2026-07-03 | **open** |

Items 3–5 are documentation-vs-hardware drift of exactly the kind that made
the v1 demo unfalsifiable. They are cheap to fix and are folded into the
W550 variants.

---

## 5. Honest scope statement

What this plan does **not** claim:

1. **No TOPS/W number.** The v2 demo runs one MAC step every 2²⁴ clocks
   specifically so a human can watch it. It is a correctness witness, not a
   throughput benchmark. Any TOPS/W figure for IGLA RACE remains a projection
   from the Coq/Lean models, not a measurement.
2. **No silicon-verified ternary GEMM.** `ternary_mac_top` is one MAC cell.
   `systolic_ternary.t27` and `ternary_gemm.t27` have never been synthesized,
   let alone flashed.
3. **No timing closure claim.** Until G1 produces a routed design with a
   reported slack against the `cfgmclk` constraint, there is no Fmax.
4. **Nothing here has run on hardware yet.** G0 is simulation. G1 is a build.
   G2–G4 require a board that is not currently connected.

---

## 6. Critical path, shortest first

```
G0 (done) ──► G1 bitstream (done) ──► G3 signature ──► G4 persistent boot
                                          ▲
                       G2 attach board and cable   <-- THE ONLY REMAINING BLOCKER
```

Everything that can be done without hardware is done. `t27c fpga-flash
--dry-run` passes every pre-flight check against the new bitstream and reports
BLOCKED for exactly one reason: `openFPGALoader --scan-usb` finds no
programmer. **Connect the QMTech Wukong V1 and its Digilent HS2 cable and G2/G3
can run immediately.**

---

*φ² + φ⁻² = 3 | TRINITY*
