# FPGA Evidence — Wave Loop 359 Hand-Written Ternary MAC

**Date:** 2026-07-02
**Branch:** trinity-rust-rings
**Target board:** QMTech Wukong V1 / XC7A100T-FGG676 (per `fpga/HARDWARE_SSOT.md`)
**Module:** `fpga/verilog/ternary_mac_synth.v` — `ternary_mac_top`

---

## 1. Motivation

The `t27c gen-verilog` backend emits structurally broken Verilog for the ternary MAC (`gen/verilog/fpga/mac.v` and 27/36 generated `.v` files contain placeholder/broken syntax). Until the backend is fixed, no FPGA evidence can be produced from generated code.

This report captures the **first hand-written, synthesis-ready ternary MAC** in the repo, plus a self-checking testbench and `yosys` synthesis metrics. It establishes a baseline for silicon evidence independent of the broken Verilog backend.

---

## 2. Module Design

| Port | Width | Description |
|------|-------|-------------|
| `clk` | 1 | Clock |
| `rst_n` | 1 | Active-low reset |
| `en` | 1 | Accumulate enable |
| `a` | 8 | Signed activation input |
| `w_code` | 2 | Ternary weight code |
| `acc_in` | 32 | Signed accumulator input |
| `acc_out` | 32 | Registered accumulator output |

**Weight encoding:**
- `2'b01` → +1
- `2'b10` → -1
- `2'b00`, `2'b11` → 0

**Operation:** `acc_out = acc_in + (a * decode(w_code))`, registered on `clk`.

---

## 3. Testbench Results

```bash
cd fpga/verilog
iverilog -o tb_ternary_mac.vvp tb_ternary_mac.v ternary_mac_synth.v
vvp tb_ternary_mac.vvp
```

Output:
```
=== ternary_mac_top self-check ===
PASS: a=3 w=1 acc_in=10 out=13
PASS: a=5 w=10 acc_in=20 out=15
PASS: a=7 w=0 acc_in=30 out=30
PASS: a=9 w=11 acc_in=40 out=40
PASS: a=252 w=1 acc_in=100 out=96
PASS: a=250 w=10 acc_in=50 out=56
=== ALL TESTS PASSED ===
```

(Note: negative values are printed as unsigned 8-bit two's-complement by `%0d` formatting; the signed arithmetic is correct.)

---

## 4. Synthesis Metrics (`yosys`)

```bash
yosys -p 'read_verilog ternary_mac_synth.v; synth_xilinx -top ternary_mac_top; stat'
```

| Resource | Count |
|----------|-------|
| LUT5 | **32** |
| FDCE (flip-flops) | **32** |
| CARRY4 | **11** |
| IBUF | 45 |
| OBUF | 32 |
| INV | 40 |
| BUFG | 1 |
| **Estimated logic cells** | **32** |

Interpretation for XC7A100T:
- The ternary MAC consumes ~32 LUTs + 32 FFs + 11 carry chains for the 32-bit accumulator adder.
- This is a reasonable starting point for a single MAC cell; a full systolic tile will scale with operand width and tile geometry.
- No DSP blocks are used — the multiply is free (ternary decode selects `+a`, `-a`, or `0`).

---

## 5. OpenXC7 Bitstream Status

`nextpnr-himbaechel`, `fasm2frames`, and `xc7frames2bit` are **not installed** on this machine. Therefore a `.bit` file could not be generated in this wave.

Next step for W360/W361:
1. Install OpenXC7 toolchain per `fpga/HARDWARE_SSOT.md` §8.
2. Create `ternary_mac_top.xdc` with clock and I/O constraints for the QMTech Wukong V1.
3. Run `yosys` → `nextpnr-xilinx` → `fasm2frames` → `xc7frames2bit`.
4. Flash via `cli/dlc10 sram ternary_mac_top.bit` and verify loopback on the board.

---

## 6. Comparison with Generated Verilog

| Aspect | Generated `mac.v` | Hand-written `ternary_mac_synth.v` |
|--------|-------------------|------------------------------------|
| Syntactically valid | ❌ placeholders, broken struct syntax | ✅ passes `read_verilog` |
| Synthesizable | ❌ | ✅ `synth_xilinx` succeeds |
| Self-checking testbench | ❌ | ✅ `tb_ternary_mac.v` |
| Synthesis metrics | ❌ | ✅ 32 LUTs, 32 FFs, 11 CARRY4 |

---

## 7. Conclusion

Wave Loop 359 produces the **first measurable synthesis evidence** for a Trinity ternary MAC: a hand-written 32-bit accumulator cell using 32 LUTs and 32 flip-flops. This begins to close the silicon credibility gap while the Verilog backend is scheduled for repair.
