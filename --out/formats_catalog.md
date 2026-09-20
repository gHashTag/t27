# Universal Numeric Format Catalog

Generated from specs/numeric/formats_catalog.t27 by the bootstrap
codegen (tools/gen_formats_catalog.py). Do not edit by hand.

Status labels: Verified | EmpiricalFit | Open | Risk | Retracted |
Experimental | Historical. phi_distance: lower = more phi-aligned;
-1 sentinel = undefined (non-radix-2 or non-FP).

Total formats: 83.

| ID | Bits | S:E:M | Bias | phi_dist | Storage | Cluster | Status | Standard | Use case | GF rel. |
|----|-----:|------|-----:|--------:|---------|---------|--------|----------|----------|---------|
| binary16 | 16 | 1:5:10 | 15 | 0.118 | u16 | Ieee754Binary | Verified | IEEE 754-2008 | GPU activations, inference | competitor |
| binary32 | 32 | 1:8:23 | 127 | 0.270 | u32 | Ieee754Binary | Verified | IEEE 754-1985 | industry default | competitor |
| binary64 | 64 | 1:11:52 | 1023 | 0.406 | u64 | Ieee754Binary | Verified | IEEE 754-1985 | scientific computing | competitor |
| binary128 | 128 | 1:15:112 | 16383 | 0.484 | u128 | Ieee754Binary | Verified | IEEE 754-2008 | high-precision simulations | competitor |
| binary256 | 256 | 1:19:236 | 262143 | 0.538 | u256_software | Ieee754Binary | Verified | IEEE 754-2008 | astronomy, cryptography | competitor |
| decimal32 | 32 | 1:11:20 | 101 | n/a | u32 | Ieee754Decimal | Verified | IEEE 754-2008 (DPD/BID) | banking, GAAP | orthogonal |
| decimal64 | 64 | 1:13:50 | 398 | n/a | u64 | Ieee754Decimal | Verified | IEEE 754-2008 | financial databases | orthogonal |
| decimal128 | 128 | 1:17:110 | 6176 | n/a | u128 | Ieee754Decimal | Verified | IEEE 754-2008 | audit ledgers | orthogonal |
| x87_fp80 | 80 | 1:15:64 | 16383 | n/a | u80_padded | ExtendedFloat | Historical | Intel x87 (explicit integer bit) | legacy long double on x86 | orthogonal |
| double_double | 128 | 2:22:104 | 0 | n/a | two_u64 | ExtendedFloat | Verified | Bailey/Hida (software) | software extended precision | orthogonal |
| quad_double | 256 | 4:44:208 | 0 | n/a | four_u64 | ExtendedFloat | Verified | Bailey/Hida (software) | astrophysics, quad-precision sims | orthogonal |
| bfloat16 | 16 | 1:8:7 | 127 | 0.525 | u16 | MlLowPrecision | Verified | Google Brain | training (range > precision) | competitor |
| tf32 | 19 | 1:8:10 | 127 | 0.270 | u32_padded | MlLowPrecision | Verified | NVIDIA Ampere | A100/H100 mixed precision | competitor |
| fp8_e4m3 | 8 | 1:4:3 | 7 | 0.715 | u8 | MlLowPrecision | Verified | OCP / NVIDIA / Arm / Intel | inference, gradient ranges | competitor |
| fp8_e5m2 | 8 | 1:5:2 | 15 | 1.882 | u8 | MlLowPrecision | Verified | OCP / NVIDIA | activations, wide range | competitor |
| fp6_e3m2 | 6 | 1:3:2 | 3 | 0.882 | u8_packed | MlLowPrecision | Verified | OCP MX | aggressive quant inference | competitor |
| fp6_e2m3 | 6 | 1:2:3 | 1 | 0.049 | u8_packed | MlLowPrecision | Verified | OCP MX | mantissa-heavy quant | ally |
| fp4_e2m1 | 4 | 1:2:1 | 1 | 1.382 | u8_packed | MlLowPrecision | Verified | OCP MX | extreme quant inference | competitor |
| mxfp8 | 8 | 1:4:3 | 7 | 0.715 | u8_plus_shared_e8m0 | Microscaling | Verified | OCP MX v1.0 | LLM inference | ally |
| mxfp6 | 6 | 1:3:2 | 3 | 0.882 | u8_packed_plus_e8m0 | Microscaling | Verified | OCP MX v1.0 | aggressive inference | ally |
| mxfp4 | 4 | 1:2:1 | 1 | 1.382 | u8_packed_plus_e8m0 | Microscaling | Verified | OCP MX v1.0 | extreme quant | ally |
| nf4 | 4 | 0:0:4 | 0 | n/a | u8_packed | QuantTuned | Verified | Dettmers 2023 (QLoRA) | LLM weight quantization (quantile-based on N(0,1)) | orthogonal |
| afp | 16 | 1:8:7 | 127 | n/a | u16_plus_tensor_shift | QuantTuned | Verified | Tambe 2020 | efficient training | orthogonal |
| posit8 | 8 | 1:2:0 | 0 | n/a | u8 | PositUnumIII | Verified | Posit Standard 2022 (es=2) | inference | ally |
| posit16 | 16 | 1:2:0 | 0 | n/a | u16 | PositUnumIII | Verified | Posit Standard 2022 (es=2) | mixed-precision training | ally |
| posit32 | 32 | 1:2:0 | 0 | n/a | u32 | PositUnumIII | Verified | Posit Standard 2022 (es=2) | f32 replacement | ally |
| posit64 | 64 | 1:2:0 | 0 | n/a | u64 | PositUnumIII | Verified | Posit Standard 2022 (es=2) | f64 replacement | ally |
| takum8 | 8 | 1:0:0 | 0 | n/a | u8 | PositUnumIII | Verified | Hunhold 2024 (tapered-precision) | IEEE-754 backward-compatible tapered | ally |
| takum16 | 16 | 1:0:0 | 0 | n/a | u16 | PositUnumIII | Verified | Hunhold 2024 | single-rule ladder counterexample | ally |
| takum32 | 32 | 1:0:0 | 0 | n/a | u32 | PositUnumIII | Verified | Hunhold 2024 | tapered fp32-class | ally |
| takum64 | 64 | 1:0:0 | 0 | n/a | u64 | PositUnumIII | Verified | Hunhold 2024 | tapered fp64-class | ally |
| lns8 | 8 | 1:7:0 | 0 | n/a | u8 | Lns | Verified | Arnold 1990; LNS-Madam (2021) | DSP, signal processing | orthogonal |
| lns16 | 16 | 1:15:0 | 0 | n/a | u16 | Lns | Verified | LNS-Madam (2021) | log-domain training (mul -> add) | orthogonal |
| lns32 | 32 | 1:31:0 | 0 | n/a | u32 | Lns | Verified | LNS-Madam (2021) | log-domain DSP | orthogonal |
| lns64 | 64 | 1:63:0 | 0 | n/a | u64 | Lns | Verified | LNS-Madam (2021) | scientific log-domain | orthogonal |
| gfternary | 2 | 1:0:2 | 0 | 0.000 | u2 | GoldenFloat | Verified | this work; {-phi, 0, +phi} | bulk layers (hybrid) | self |
| gf4 | 4 | 1:1:2 | 0 | 0.118 | u8 | GoldenFloat | Experimental | this work; F0 minimal | proof-of-concept | self |
| gf8 | 8 | 1:3:4 | 3 | 0.132 | u8 | GoldenFloat | Verified | this work; L1 Lucas | edge / sensors | self |
| gf12 | 12 | 1:4:7 | 7 | 0.047 | u16 | GoldenFloat | Verified | this work; L0/F3 | mid-range / audio | self |
| gf16 | 16 | 1:6:9 | 31 | 0.049 | u16 | GoldenFloat | Verified | this work; PHI_BIAS=60; FPGA 35/35 at 323 MHz Artix-7 | training and inference (production) | self |
| gf20 | 20 | 1:7:12 | 63 | 0.035 | u32 | GoldenFloat | Experimental | this work; 17-squared empirical PHI_BIAS=289 | high-precision edge | self |
| gf24 | 24 | 1:9:14 | 255 | 0.025 | u32 | GoldenFloat | Experimental | this work; rule e=round(23/phi^2)=9; normative bias=2^(e-1)-1=255; empirical PHI_BIAS=1364 (=L15) OPEN | server inference | self |
| gf32 | 32 | 1:12:19 | 2047 | 0.014 | u32 | GoldenFloat | Verified | this work; F0 resolved | fp32 drop-in | self |
| gf64 | 64 | 1:24:39 | 8388607 | 0.003 | u64 | GoldenFloat | Verified | this work; EXP_MAX - BIAS | scientific / double | self |
| gf6 | 6 | 1:2:3 | 1 | 0.049 | u8_packed | GoldenFloat | Open | this work; rule e=round(5/phi^2)=2; FP6 E2M3 bridge | OPEN R&D: bridge GF4-GF8; FP6 E2M3 hint | experimental |
| gf10 | 10 | 1:3:6 | 3 | 0.118 | u16 | GoldenFloat | Open | this work; rule e=round(9/phi^2)=3; bridge GF8-GF12 | OPEN R&D: tight-precision activations | experimental |
| gf14 | 14 | 1:5:8 | 15 | 0.007 | u16 | GoldenFloat | Open | this work; rule e=round(13/phi^2)=5; bridge GF12-GF16; lowest phi-dist below GF48 | OPEN R&D: drop-in for fp16 with tighter phi alignment | experimental |
| gf48 | 48 | 1:18:29 | 131071 | 0.003 | u64_padded | GoldenFloat | Open | this work; rule e=round(47/phi^2)=18 | OPEN R&D: between GF32 and GF64; tightest phi-dist of the wide rungs | experimental |
| gf96 | 96 | 1:36:59 | 34359738367 | 0.008 | u128_padded | GoldenFloat | Open | this work; rule e=round(95/phi^2)=36 | OPEN R&D: between GF64 and GF128 (phi-aligned extended) | experimental |
| gf128 | 128 | 1:49:78 | 281474976710655 | 0.010 | u128 | GoldenFloat | Open | this work; rule e=round(127/phi^2)=49 (corrects v1.1 typo e=48) | OPEN R&D: phi-aligned binary128 alternative | experimental |
| gf256 | 256 | 1:97:158 | 79228162514264337593543950335 | 0.004 | u256_software | GoldenFloat | Open | this work; rule e=round(255/phi^2)=97; normative bias=2^96-1 | OPEN R&D: phi-aligned binary256 alternative | experimental |
| gf512 | 512 | 1:195:316 | -2 | 0.001 | u512_software | GoldenFloat | Open | this work; rule e=round(511/phi^2)=195 | OPEN R&D: ultra-wide phi-aligned (extrapolation, no RTL) | experimental |
| gf1024 | 1024 | 1:391:632 | -2 | 0.001 | u1024_software | GoldenFloat | Open | this work; rule e=round(1023/phi^2)=391; lowest phi-distance in the ladder | OPEN R&D: limit-of-ladder phi alignment (extrapolation, no RTL) | experimental |
| gf8_bfp | 8 | 1:3:4 | 3 | 0.132 | u8_plus_shared_exp | GoldenFloat | Experimental | this work; per-tile shared exponent | OPEN R&D: LLM-quantization-friendly GF8 | experimental |
| gf_lns_hybrid | 16 | 1:6:9 | 31 | 0.049 | u16_plus_lns_path | GoldenFloat | Experimental | this work; mul in log-space, accumulate Lucas-closed | OPEN R&D: dual-space arithmetic | experimental |
| mxgf6 | 6 | 1:2:3 | 1 | 0.050 | u8_packed_plus_e8m0 | GoldenFloat | Experimental | this work; OCP MX block + GF6 | OPEN R&D: phi-aligned MX-6 candidate | experimental |
| mxgf4 | 4 | 1:1:2 | 0 | 0.118 | u8_packed_plus_e8m0 | GoldenFloat | Experimental | this work; OCP MX block + GF4 | OPEN R&D: phi-aligned MX-4 candidate | experimental |
| int4 | 4 | 1:0:3 | 0 | n/a | u8_packed | IntegerFixed | Verified | two complement | aggressive quantization | competitor |
| int8 | 8 | 1:0:7 | 0 | n/a | u8 | IntegerFixed | Verified | two complement | INT8 inference, per-channel scale | competitor |
| int16 | 16 | 1:0:15 | 0 | n/a | u16 | IntegerFixed | Verified | two complement | DSP, embedded ML | competitor |
| int32 | 32 | 1:0:31 | 0 | n/a | u32 | IntegerFixed | Verified | two complement | general CPU integer | competitor |
| int64 | 64 | 1:0:63 | 0 | n/a | u64 | IntegerFixed | Verified | two complement | databases, timestamps | competitor |
| int128 | 128 | 1:0:127 | 0 | n/a | u128 | IntegerFixed | Verified | two complement | crypto, big-int | competitor |
| q_format | 0 | 1:0:0 | 0 | n/a | varies | IntegerFixed | Verified | TI fixed-point | audio DSP, fixed-point ML | orthogonal |
| bcd | 0 | 0:0:0 | 0 | n/a | u4_per_digit | IntegerFixed | Historical | IBM 1959 | calculators, GAAP | orthogonal |
| ibm_hfp32 | 32 | 1:7:24 | 64 | n/a | u32 | HistoricalVendor | Historical | IBM System/360 (1964); base-16 exponent | legacy mainframe | orthogonal |
| ibm_hfp64 | 64 | 1:7:56 | 64 | n/a | u64 | HistoricalVendor | Historical | IBM System/360 (1964) | legacy mainframe | orthogonal |
| ibm_hfp128 | 128 | 1:7:120 | 64 | n/a | u128 | HistoricalVendor | Historical | IBM z/Architecture | legacy mainframe | orthogonal |
| ms_mbf32 | 32 | 1:8:23 | 129 | n/a | u32 | HistoricalVendor | Historical | MS BASIC / MS-DOS (pre-IEEE) | MS BASIC legacy | orthogonal |
| ms_mbf64 | 64 | 1:8:55 | 129 | n/a | u64 | HistoricalVendor | Historical | MS BASIC | MS BASIC legacy | orthogonal |
| vax_f | 32 | 1:8:23 | 128 | n/a | u32 | HistoricalVendor | Historical | DEC VAX | DEC legacy | orthogonal |
| vax_d | 64 | 1:8:55 | 128 | n/a | u64 | HistoricalVendor | Historical | DEC VAX | DEC legacy double | orthogonal |
| vax_g | 64 | 1:11:52 | 1024 | n/a | u64 | HistoricalVendor | Historical | DEC VAX (IEEE-like) | DEC legacy | orthogonal |
| vax_h | 128 | 1:15:112 | 16384 | n/a | u128 | HistoricalVendor | Historical | DEC VAX | DEC quad | orthogonal |
| cray_float | 64 | 1:15:48 | 16384 | n/a | u64 | HistoricalVendor | Historical | Cray-1 (1976); no NaN/Inf, unrounded mul | Cray legacy | orthogonal |
| minifloat | 0 | 1:0:0 | 0 | n/a | varies | Theoretical | Experimental | parametric framework | design space of GF4/GF8/GF12/GF16 | ally |
| unum_i | 0 | 1:0:0 | 0 | n/a | varies | Theoretical | Experimental | Gustafson 2015 (predecessor to posit) | interval arithmetic | ally |
| unum_ii | 0 | 0:0:0 | 0 | n/a | lookup_table | Theoretical | Experimental | Gustafson 2016 | lookup-table real arithmetic; not GF-comparable | orthogonal |
| tapered_fp | 0 | 1:0:0 | 0 | n/a | varies | Theoretical | Experimental | Morris 1971; posit ancestor | variable mantissa via regime bits | ally |
| block_fp | 0 | 0:0:0 | 0 | n/a | varies | CompressionTrick | Verified | Wilkinson 1965; modern revivals | per-tile shared exponent | ally |
| shared_exp | 0 | 0:0:0 | 0 | n/a | varies | CompressionTrick | Verified | generalised BFP | LLM quantization | ally |
| per_channel_scale | 8 | 1:0:7 | 0 | n/a | u8_plus_fp32_scale | CompressionTrick | Verified | Jacob 2018 (TFLite) | standard quant inference | competitor |
| stochastic_rounding | 0 | 0:0:0 | 0 | n/a | varies | CompressionTrick | Verified | Gupta 2015 | training small networks at low precision | ally |

## Sources

Per-row citation in the `source` field of the SSOT.

## Honesty contract

This catalog records cluster, status, phi_distance, and use case
only. Per-rung quality claims (better than posit / takum / OCP-MX /
LNS) live ONLY in FL-002 with the F1/F2/F3 falsification protocol.
Default status of any moat claim is Open conjecture.
