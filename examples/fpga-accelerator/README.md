# FPGA Accelerator Example

This example demonstrates designing an FPGA IP core for GF16 arithmetic operations.

## Background

The GF16 format (6-bit exponent, 9-bit mantissa) is designed for optimal bit allocation following the golden ratio. This IP core provides hardware acceleration for GF16 operations, suitable for AI inference and signal processing.

## The Specification

`gf16-accelerator.t27` generates Verilog for:

1. **GF16 Adder** — φ-aligned adder with carry chain optimization
2. **GF16 Multiplier** — Booth-encoded multiplier
3. **Vector Unit** — Parallel GF16 operations (8 elements)
4. **MAC Unit** — Multiply-accumulate for neural networks
5. **Pipeline Registers** — Clock domain crossing safe
6. **Test Bench** — Verification with golden model

## Hardware Resources (Xilinx Artix-7)

| Component | LUTs | FFs | DSPs | BRAM |
|-----------|-----|-----|------|------|
| GF16 Adder | 85 | 60 | 0 | 0 |
| GF16 Multiplier | 120 | 80 | 1 | 0 |
| Vector Unit (8x) | 960 | 640 | 8 | 0 |
| MAC Pipeline | 205 | 140 | 1 | 0 |

## Running

```bash
# Parse
tri parse gf16-accelerator.t27

# Generate Verilog
tri gen-verilog gf16-accelerator.t27 > gen/gf16_accelerator.v

# Generate for FPGA tools
tri gen-verilog gf16-accelerator.t27 | tee gen/gf16_accelerator.v

# Simulate (with iverilog)
iverilog -o tb gen/gf16_accelerator.v testbench/gf16_tb.v
vvp tb

# Build for FPGA (Vivado)
vivado -mode batch -source scripts/synthesize_gf16.tcl
```

## Synthesis

Target FPGAs:
- **Xilinx Artix-7** (xc7a35t) — 1-2 MACs per unit
- **Intel Cyclone V** (5CGXFC7C7F23C8) — Similar resource usage
- **Lattice iCE40-HX** — Limited DSP, requires LUT-based mul

## Applications

- Edge AI inference (GF16 quantized models)
- DSP filtering (FIR, IIR with GF16 coefficients)
- Control systems (PID controllers)

---

**φ² + 1/φ² = 3 | TRINITY**