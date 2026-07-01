# FPGA Evidence — Wave Loop 360

**Date:** 2026-07-02  
**Target board:** QMTech Wukong V1 / XC7A100T-FGG676 (`xc7a100tfgg676-1`)  
**Design:** `ternary_mac_demo_top` → wraps `ternary_mac_top`  
**Toolchain:** yosys 0.63 + attempted OpenXC7 (`nextpnr-xilinx`)

---

## Status

| Step | Result |
|------|--------|
| Hand-written ternary MAC RTL | ✅ `fpga/verilog/ternary_mac_synth.v` |
| Self-checking testbench | ✅ 6/6 vectors pass (W359) |
| yosys `synth_xilinx` | ✅ 34 cells (demo top), 12 CARRY4 total |
| yosys JSON netlist | ✅ `fpga/verilog/ternary_mac_demo_top.json` |
| Pin constraints (LEDs R23/T23) | ✅ `fpga/verilog/ternary_mac_demo_top.xdc` |
| `nextpnr-xilinx` placement/routing | ❌ **Toolchain not installed** |
| FASM → frames → `.bit` | ❌ Blocked on missing nextpnr output |
| Board flash via `dlc10` | ❌ Blocked on missing `.bit` |

---

## yosys synthesis log (demo top)

Command:

```sh
cd fpga/verilog
yosys -p 'read_verilog ternary_mac_synth.v ternary_mac_demo_top.v; \
          synth_xilinx -family xc7 -top ternary_mac_demo_top -flatten; \
          stat; write_json ternary_mac_demo_top.json'
```

Result summary:

```
   34 cells:
        1   $scopeinfo
        1   BUFG
        1   CARRY4
        6   FDRE
        4   INV
       19   LUT1
        2   OBUF

   Estimated number of LCs: 10
```

Including the nested `ternary_mac_top` instance, the design uses:

- **32 LUT5** (8-bit signed × 2-bit weight product)
- **32 FDCE** (32-bit accumulator register)
- **11 CARRY4** inside the MAC + 1 CARRY4 in the demo counter = **12 CARRY4**
- **6 FDRE** in the demo counter/ring-oscillator divider
- **19 LUT1** in the ring-oscillator inverter chain
- **2 OBUF** for LEDs
- **1 BUFG** inferred on the oscillator net

---

## OpenXC7 blocker

`nextpnr-xilinx` is not on `PATH`:

```
$ nextpnr-xilinx --chipdb xc7a100tfgg676.bin ...
(eval):1: command not found: nextpnr-xilinx
```

Homebrew only provides `nextpnr-ice40`; the Xilinx backend must be built from the `openXC7/nextpnr-xilinx` `stable-backports` branch. The verified recipe is in `fpga/HARDWARE_SSOT.md` §8:

1. Install dependencies: `brew install yosys boost boost-python3 eigen cmake`
2. Clone and build `openXC7/nextpnr-xilinx` with `-DUSE_OPENMP=OFF`
3. Generate `xc7a100tfgg676.bin` chipdb (~159 MB)
4. Build `f4pga/prjxray` `xc7frames2bit`
5. Create a Python venv for `fasm2frames.py`

Because this build takes 10–30 minutes and is not yet present, W360 captures the **ready-to-route** netlist and constraints and records the toolchain gap. W361 should either complete the OpenXC7 install or switch to a Vivado-in-Docker proxy if the install fails.

---

## Next steps

1. Run the OpenXC7 recipe to obtain `nextpnr-xilinx` and chipdb.
2. Place/route with `--ignore-loops` (ring oscillator).
3. Produce `m.fasm` → `m.frames` → `m.bit`.
4. Validate on board: `cargo build --release -p dlc10 && dlc10 idcode && dlc10 sram m.bit`.
