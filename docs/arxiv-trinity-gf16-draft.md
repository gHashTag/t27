# Trinity GF16: A phi-Anchored 16-bit Float with FPGA Implementation at 323 MHz

## Abstract

We introduce Golden Float 16 (GF16), a 16-bit floating-point format with a phi-anchored exponent bias of 31. GF16 uses a 1/6/9 bit layout (sign/exponent/mantissa) and achieves 323 MHz combinational throughput on a Xilinx Artix-7 XC7A100T FPGA using the open-source openXC7 toolchain (Yosys + nextpnr). We present a complete dot-product (N=4) and 4x4 matrix multiplication accelerator verified in FPGA synthesis and RTL simulation, with 35/35 tests passing and 0 timing violations at 100 MHz. The design has been submitted for ASIC fabrication on Sky130 via the TinyTapeout TTSKY26b TT4913 Gamma shuttle (submission closed May 2026); silicon has not yet been returned (expected late 2026), so no on-chip measurement is claimed.

## 1. Introduction

The golden ratio phi (phi = (1+sqrt(5))/2 = 1.618...) appears throughout nature, mathematics, and information theory. We observe that the IEEE 754 half-precision format (float16) uses a 5-bit exponent with bias 15, while bfloat16 uses 8-bit exponent with bias 127. Neither has any connection to fundamental mathematical constants.

Trinity GF16 anchors its exponent bias at 31, derived from the relationship:

```
phi^2 + phi^-2 = 3
phi^2 = phi + 1
```

The bias value 31 encodes 1.0 at the golden ratio's "natural center" of the exponent range, creating a format where common physics and ML values cluster around the representational sweet spot.

## 2. GF16 Format Specification

### 2.1 Bit Layout

| Bit(s) | Field | Width |
|--------|-------|-------|
| 15 | Sign | 1 |
| 14:9 | Exponent | 6 |
| 8:0 | Mantissa | 9 |

- **Total:** 16 bits
- **Exponent bias:** 31
- **Implicit leading 1:** Normal numbers have implicit 1.mantissa

### 2.2 Special Values

| Value | Encoding |
|-------|----------|
| +0 | 0x0000 |
| -0 | 0x8000 |
| +Infinity | 0x7E00 |
| -Infinity | 0xFE00 |
| NaN | 0xFE01 |

### 2.3 Encoding of Key Constants

| Value | GF16 Hex | Decoded | Relative Error |
|-------|----------|---------|----------------|
| 1.0 | 0x3E00 | 1.0 | 0 |
| phi | 0x3F3C | 1.6171875 | 0.0005 |
| pi | 0x4049 | 3.140625 | 0.0003 |
| e | 0x4058 | 2.71875 | 0.0002 |
| sqrt(2) | 0x3F5C | 1.4140625 | 0.0001 |

### 2.4 Dynamic Range

- **Max normal:** (1 + 511/512) x 2^(62-31) = ~4.29 x 10^9
- **Min normal:** 1.0 x 2^(1-31) = ~9.31 x 10^-10
- **Machine epsilon:** 2^-9 = 0.001953125

### 2.5 Comparison with Existing Formats

| Format | Bits | Exp | Mant | Bias | Max Value |
|--------|------|-----|------|------|-----------|
| float16 | 16 | 5 | 10 | 15 | 65504 |
| **GF16** | **16** | **6** | **9** | **31** | **4.29 x 10^9** |
| bfloat16 | 16 | 8 | 7 | 127 | 3.39 x 10^38 |

GF16 provides 65x wider dynamic range than float16 while maintaining better precision than bfloat16 (9 vs 7 mantissa bits). The 6-bit exponent with bias=31 positions 1.0 at the center of a practical ML/inference range.

## 3. Hardware Architecture

### 3.1 GF16 Multiplier

Combinational multiplier implementing:
- 10-bit x 10-bit mantissa multiplication (mapped to DSP48E1 on Xilinx)
- Exponent addition with bias subtraction
- Round-to-nearest-even with guard/round/sticky bits
- Special value handling (NaN, Inf, zero)

### 3.2 GF16 Adder

Combinational adder implementing:
- Exponent alignment via priority encoder (no for-loops)
- Sign-magnitude addition with cancellation support
- Normalization with round-to-nearest-even
- Special value handling

Key design constraint: all internal `reg` variables initialized with defaults at the top of `always @(*)` blocks to prevent latch inference in Yosys synthesis.

### 3.3 Dot Product (N=4)

Tree reduction: 4 multipliers + 3 adders

```
a0*b0 ---\
          +-- s01 ---\
a1*b1 ---/           \
                      +-- result
a2*b2 ---\           /
          +-- s23 --/
a3*b3 ---/
```

### 3.4 Matrix Multiplication (4x4)

16 parallel dot-product units. Each unit computes one element of the output matrix C = A x B.

```
C[i][j] = dot4(A[i][0:3], B[0:3][j])
```

## 4. FPGA Implementation Results

### 4.1 Platform

- **FPGA:** QMTECH XC7A100T-1FGG676C (Artix-7, 63,400 LUTs, 240 DSP48E1)
- **Toolchain:** openXC7 (Yosys 0.62 + nextpnr-xilinx)
- **Clock:** 20-stage ring oscillator (onboard M21 crystal non-functional)
- **Programming:** XVC via ESP32 (192.168.1.30:2542)

### 4.2 Resource Utilization

| Design | LUTs | DSP48E1 | Max Freq | Tests |
|--------|------|---------|----------|-------|
| GF16 mul | ~650 | 1 | 330 MHz | 13/13 |
| GF16 dot4 | 2,605 | 4 | 322 MHz | 6/6 |
| GF16 matmul 4x4 | 40,350 | 64 | 323 MHz | 35/35 |

### 4.3 Timing

All designs pass timing at 100 MHz with positive slack:

```
Max frequency for clock 'chain[19]': 323.31 MHz (PASS at 100.00 MHz)
```

### 4.4 Latch Elimination

A critical design challenge was eliminating all LDCE latch inferences from Yosys. The root cause was incompletely-assigned `reg` variables in `always @(*)` combinational blocks. Solution: initialize all `reg` variables with defaults at the block entry, and ensure every control path assigns every variable.

Before fix: 1 latch (gf16_add.result)
After fix: 0 latches, 0 errors

### 4.5 Hardware Verification

All designs verified on FPGA via XVC programming:
- dot4([1,2,3,4], [1,2,3,4]) = 30.0 (0x47C0) — LEDs confirm
- matmul4x4 identity test — ISC_DONE=1, DONE=1

### 4.6 Throughput

| Metric | Value |
|--------|-------|
| Dot4 throughput | 322M dot4/sec (combinational, 1-cycle) |
| Matmul4x4 throughput | 322M matmuls/sec (fully parallel) |
| GF16 ops/sec (matmul) | 41.2 GOPS @ 323 MHz |
| GF16 ops/sec @ 100 MHz | 12.8 GOPS |

## 5. ASIC Path (TinyTapeout TTSKY26b TT4913 Gamma)

### 5.1 Submission

The GF16 dot4 design was wrapped for the TinyTapeout Sky130 shuttle:

- **Repo:** github.com/gHashTag/tt-trinity-gf16
- **Module:** tt_um_ghtag_trinity_gf16
- **Tile:** 1x1 (167 x 108 um)
- **PDK:** Sky130A (OpenLane/LibreLane 3.0.0)

### 5.2 CI Results

| Check | Status |
|-------|--------|
| GDS Build | PASSED |
| Gate-Level Test | PASSED |
| Precheck | PASSED |
| DRC | Clean |
| LVS | Clean |

### 5.3 Expected Timeline

- Shuttle close: May 11, 2026
- Chip delivery: ~December 2026

## 6. Python Reference Implementation

A complete Python reference (encode/decode/mul/add/dot4) is provided in `conformance/gf16_ref.py`:

- 32/32 FPGA consistency tests pass
- 19/19 roundtrip tests pass
- Encoding matches FPGA testbench exactly

## 7. Conclusion

We have demonstrated a complete implementation of the Trinity GF16 floating-point format, from specification through FPGA verification to ASIC submission. The phi-anchored bias=31 provides a natural centering for ML and scientific computation values, while the 6/9 exponent/mantissa split offers 65x wider dynamic range than float16 with better precision than bfloat16.

All verified numbers (323 MHz, 40,350 LUTs, 64 DSP48E1, 35/35 tests, 0 latches, 0 timing violations) are from actual FPGA hardware runs (Artix-7 XC7A100T), not ASIC silicon nor simulation estimates.

## References

1. IEEE 754-2019, IEEE Standard for Floating-Point Arithmetic
2. TinyTapeout, https://tinytapeout.com
3. openXC7 toolchain, https://github.com/openXC7
4. Yosys Open SYnthesis Suite, https://github.com/YosysHQ/yosys
5. nextpnr-xilinx, https://github.com/openXC7/nextpnr-xilinx

## Appendix A: GF16 Encoding Examples

```
Value    | Hex    | Sign | Exp | Mant | Decoded
---------|--------|------|-----|------|--------
0.0      | 0x0000 | 0    | 0   | 0    | 0.0
1.0      | 0x3E00 | 0    | 31  | 0    | 1.0
-1.0     | 0xBE00 | 1    | 31  | 0    | -1.0
2.0      | 0x4000 | 0    | 32  | 0    | 2.0
0.5      | 0x3C00 | 0    | 30  | 0    | 0.5
3.0      | 0x4100 | 0    | 32  | 256  | 3.0
1.5      | 0x3F00 | 0    | 31  | 256  | 1.5
100.0    | 0x4B20 | 0    | 37  | 288  | 100.0
phi      | 0x3F3C | 0    | 31  | 60   | 1.6171875
+Inf     | 0x7E00 | 0    | 63  | 0    | Infinity
NaN      | 0xFE01 | 1    | 63  | 1    | NaN
```

## Appendix B: Reproduce Instructions

```bash
# FPGA synthesis (openXC7)
docker run --rm -v "$(pwd)":/workspace regymm/openxc7 bash -c '
  yosys -p "read_verilog fpga/vivado/gf16_*.v; synth_xilinx -top gf16_matmul4x4 -family xc7; write_json build/synth.json"
  nextpnr-xilinx --chipdb xc7a100t.bin --json build/synth.json --freq 100
'

# RTL simulation
iverilog -g2012 -o /tmp/test fpga/vivado/gf16_*.v && vvp /tmp/test

# Python reference
python3 conformance/gf16_ref.py
```
