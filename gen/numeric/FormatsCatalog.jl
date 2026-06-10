# Generated from formats_catalog.t27. Do not edit by hand.
# SPDX-License-Identifier: Apache-2.0

module FormatsCatalog

export Format, FORMATS

struct Format
    id::String
    name::String
    bits::UInt32
    s_bits::UInt32
    e_bits::UInt32
    m_bits::UInt32
    bias::Int64
    phi_distance::Float64  # -1.0 == undefined
    storage::String
    cluster::String
    status::String
    standard::String
    use_case::String
    gf_relation::String
    source::String
end

const FORMATS = Format[
    Format("binary16", "binary16 (fp16, half)", UInt32(16), UInt32(1), UInt32(5), UInt32(10), Int64(15), 0.118, "u16", "Ieee754Binary", "Verified", "IEEE 754-2008", "GPU activations, inference", "competitor", "IEEE 754-2008"),
    Format("binary32", "binary32 (fp32, single)", UInt32(32), UInt32(1), UInt32(8), UInt32(23), Int64(127), 0.27, "u32", "Ieee754Binary", "Verified", "IEEE 754-1985", "industry default", "competitor", "IEEE 754-1985"),
    Format("binary64", "binary64 (fp64, double)", UInt32(64), UInt32(1), UInt32(11), UInt32(52), Int64(1023), 0.406, "u64", "Ieee754Binary", "Verified", "IEEE 754-1985", "scientific computing", "competitor", "IEEE 754-1985"),
    Format("binary128", "binary128 (fp128, quad)", UInt32(128), UInt32(1), UInt32(15), UInt32(112), Int64(16383), 0.484, "u128", "Ieee754Binary", "Verified", "IEEE 754-2008", "high-precision simulations", "competitor", "IEEE 754-2008"),
    Format("binary256", "binary256 (octuple)", UInt32(256), UInt32(1), UInt32(19), UInt32(236), Int64(262143), 0.538, "u256_software", "Ieee754Binary", "Verified", "IEEE 754-2008", "astronomy, cryptography", "competitor", "IEEE 754-2008"),
    Format("decimal32", "decimal32", UInt32(32), UInt32(1), UInt32(11), UInt32(20), Int64(101), -1.0, "u32", "Ieee754Decimal", "Verified", "IEEE 754-2008 (DPD/BID)", "banking, GAAP", "orthogonal", "IEEE 754-2008"),
    Format("decimal64", "decimal64", UInt32(64), UInt32(1), UInt32(13), UInt32(50), Int64(398), -1.0, "u64", "Ieee754Decimal", "Verified", "IEEE 754-2008", "financial databases", "orthogonal", "IEEE 754-2008"),
    Format("decimal128", "decimal128", UInt32(128), UInt32(1), UInt32(17), UInt32(110), Int64(6176), -1.0, "u128", "Ieee754Decimal", "Verified", "IEEE 754-2008", "audit ledgers", "orthogonal", "IEEE 754-2008"),
    Format("x87_fp80", "x87 FP80", UInt32(80), UInt32(1), UInt32(15), UInt32(64), Int64(16383), -1.0, "u80_padded", "ExtendedFloat", "Historical", "Intel x87 (explicit integer bit)", "legacy long double on x86", "orthogonal", "Intel SDM"),
    Format("double_double", "double-double", UInt32(128), UInt32(2), UInt32(22), UInt32(104), Int64(0), -1.0, "two_u64", "ExtendedFloat", "Verified", "Bailey/Hida (software)", "software extended precision", "orthogonal", "Bailey-Hida 2001"),
    Format("quad_double", "quad-double", UInt32(256), UInt32(4), UInt32(44), UInt32(208), Int64(0), -1.0, "four_u64", "ExtendedFloat", "Verified", "Bailey/Hida (software)", "astrophysics, quad-precision sims", "orthogonal", "Bailey-Hida 2001"),
    Format("bfloat16", "bfloat16 (BF16)", UInt32(16), UInt32(1), UInt32(8), UInt32(7), Int64(127), 0.525, "u16", "MlLowPrecision", "Verified", "Google Brain", "training (range > precision)", "competitor", "Wang-Kanwar 2019"),
    Format("tf32", "TensorFloat-32 (TF32)", UInt32(19), UInt32(1), UInt32(8), UInt32(10), Int64(127), 0.27, "u32_padded", "MlLowPrecision", "Verified", "NVIDIA Ampere", "A100/H100 mixed precision", "competitor", "NVIDIA Ampere whitepaper"),
    Format("fp8_e4m3", "FP8 E4M3", UInt32(8), UInt32(1), UInt32(4), UInt32(3), Int64(7), 0.715, "u8", "MlLowPrecision", "Verified", "OCP / NVIDIA / Arm / Intel", "inference, gradient ranges", "competitor", "Micikevicius 2022 (arXiv:2209.05433)"),
    Format("fp8_e5m2", "FP8 E5M2", UInt32(8), UInt32(1), UInt32(5), UInt32(2), Int64(15), 1.882, "u8", "MlLowPrecision", "Verified", "OCP / NVIDIA", "activations, wide range", "competitor", "Micikevicius 2022"),
    Format("fp6_e3m2", "FP6 E3M2", UInt32(6), UInt32(1), UInt32(3), UInt32(2), Int64(3), 0.882, "u8_packed", "MlLowPrecision", "Verified", "OCP MX", "aggressive quant inference", "competitor", "OCP MX v1.0 (2023)"),
    Format("fp6_e2m3", "FP6 E2M3", UInt32(6), UInt32(1), UInt32(2), UInt32(3), Int64(1), 0.049, "u8_packed", "MlLowPrecision", "Verified", "OCP MX", "mantissa-heavy quant", "ally", "OCP MX v1.0 (2023)"),
    Format("fp4_e2m1", "FP4 E2M1", UInt32(4), UInt32(1), UInt32(2), UInt32(1), Int64(1), 1.382, "u8_packed", "MlLowPrecision", "Verified", "OCP MX", "extreme quant inference", "competitor", "OCP MX v1.0 (2023)"),
    Format("mxfp8", "MXFP8", UInt32(8), UInt32(1), UInt32(4), UInt32(3), Int64(7), 0.715, "u8_plus_shared_e8m0", "Microscaling", "Verified", "OCP MX v1.0", "LLM inference", "ally", "Rouhani 2023 (arXiv:2310.10537)"),
    Format("mxfp6", "MXFP6", UInt32(6), UInt32(1), UInt32(3), UInt32(2), Int64(3), 0.882, "u8_packed_plus_e8m0", "Microscaling", "Verified", "OCP MX v1.0", "aggressive inference", "ally", "Rouhani 2023"),
    Format("mxfp4", "MXFP4", UInt32(4), UInt32(1), UInt32(2), UInt32(1), Int64(1), 1.382, "u8_packed_plus_e8m0", "Microscaling", "Verified", "OCP MX v1.0", "extreme quant", "ally", "Rouhani 2023"),
    Format("nf4", "NF4 (NormalFloat 4-bit)", UInt32(4), UInt32(0), UInt32(0), UInt32(4), Int64(0), -1.0, "u8_packed", "QuantTuned", "Verified", "Dettmers 2023 (QLoRA)", "LLM weight quantization (quantile-based on N(0,1))", "orthogonal", "Dettmers 2023 (arXiv:2305.14314)"),
    Format("afp", "AFP (Adaptive Floating-Point)", UInt32(16), UInt32(1), UInt32(8), UInt32(7), Int64(127), -1.0, "u16_plus_tensor_shift", "QuantTuned", "Verified", "Tambe 2020", "efficient training", "orthogonal", "Tambe 2020 (DAC)"),
    Format("posit8", "Posit8", UInt32(8), UInt32(1), UInt32(2), UInt32(0), Int64(0), -1.0, "u8", "PositUnumIII", "Verified", "Posit Standard 2022 (es=2)", "inference", "ally", "Posit Standard 2022 (posithub.org)"),
    Format("posit16", "Posit16", UInt32(16), UInt32(1), UInt32(2), UInt32(0), Int64(0), -1.0, "u16", "PositUnumIII", "Verified", "Posit Standard 2022 (es=2)", "mixed-precision training", "ally", "Posit Standard 2022"),
    Format("posit32", "Posit32", UInt32(32), UInt32(1), UInt32(2), UInt32(0), Int64(0), -1.0, "u32", "PositUnumIII", "Verified", "Posit Standard 2022 (es=2)", "f32 replacement", "ally", "Posit Standard 2022"),
    Format("posit64", "Posit64", UInt32(64), UInt32(1), UInt32(2), UInt32(0), Int64(0), -1.0, "u64", "PositUnumIII", "Verified", "Posit Standard 2022 (es=2)", "f64 replacement", "ally", "Posit Standard 2022"),
    Format("takum8", "takum8", UInt32(8), UInt32(1), UInt32(0), UInt32(0), Int64(0), -1.0, "u8", "PositUnumIII", "Verified", "Hunhold 2024 (tapered-precision)", "IEEE-754 backward-compatible tapered", "ally", "Hunhold 2024 (arXiv:2412.20273)"),
    Format("takum16", "takum16", UInt32(16), UInt32(1), UInt32(0), UInt32(0), Int64(0), -1.0, "u16", "PositUnumIII", "Verified", "Hunhold 2024", "single-rule ladder counterexample", "ally", "Hunhold 2024 (arXiv:2412.20273)"),
    Format("takum32", "takum32", UInt32(32), UInt32(1), UInt32(0), UInt32(0), Int64(0), -1.0, "u32", "PositUnumIII", "Verified", "Hunhold 2024", "tapered fp32-class", "ally", "Hunhold 2024"),
    Format("takum64", "takum64", UInt32(64), UInt32(1), UInt32(0), UInt32(0), Int64(0), -1.0, "u64", "PositUnumIII", "Verified", "Hunhold 2024", "tapered fp64-class", "ally", "Hunhold 2024"),
    Format("lns8", "LNS-8", UInt32(8), UInt32(1), UInt32(7), UInt32(0), Int64(0), -1.0, "u8", "Lns", "Verified", "Arnold 1990; LNS-Madam (2021)", "DSP, signal processing", "orthogonal", "Alam 2021 (arXiv:2106.13914)"),
    Format("lns16", "LNS-16", UInt32(16), UInt32(1), UInt32(15), UInt32(0), Int64(0), -1.0, "u16", "Lns", "Verified", "LNS-Madam (2021)", "log-domain training (mul -> add)", "orthogonal", "Alam 2021"),
    Format("lns32", "LNS-32", UInt32(32), UInt32(1), UInt32(31), UInt32(0), Int64(0), -1.0, "u32", "Lns", "Verified", "LNS-Madam (2021)", "log-domain DSP", "orthogonal", "Alam 2021"),
    Format("lns64", "LNS-64", UInt32(64), UInt32(1), UInt32(63), UInt32(0), Int64(0), -1.0, "u64", "Lns", "Verified", "LNS-Madam (2021)", "scientific log-domain", "orthogonal", "Alam 2021"),
    Format("gfternary", "GFTernary", UInt32(2), UInt32(1), UInt32(0), UInt32(2), Int64(0), 0.0, "u2", "GoldenFloat", "Verified", "this work; {-phi, 0, +phi}", "bulk layers (hybrid)", "self", "BENCH-007"),
    Format("gf4", "GF4", UInt32(4), UInt32(1), UInt32(1), UInt32(2), Int64(0), 0.118, "u8", "GoldenFloat", "Experimental", "this work; F0 minimal", "proof-of-concept", "self", "specs/numeric/gf4.t27"),
    Format("gf8", "GF8", UInt32(8), UInt32(1), UInt32(3), UInt32(4), Int64(3), 0.132, "u8", "GoldenFloat", "Verified", "this work; L1 Lucas", "edge / sensors", "self", "BENCH-007 (specs/numeric/gf8.t27)"),
    Format("gf12", "GF12", UInt32(12), UInt32(1), UInt32(4), UInt32(7), Int64(7), 0.047, "u16", "GoldenFloat", "Verified", "this work; L0/F3", "mid-range / audio", "self", "BENCH-007 (specs/numeric/gf12.t27)"),
    Format("gf16", "GF16", UInt32(16), UInt32(1), UInt32(6), UInt32(9), Int64(31), 0.049, "u16", "GoldenFloat", "Verified", "this work; PHI_BIAS=60; FPGA 35/35 at 323 MHz Artix-7", "training and inference (production)", "self", "specs/numeric/gf16.t27; zenodo 10.5281/zenodo.19227877 (HW archive)"),
    Format("gf20", "GF20", UInt32(20), UInt32(1), UInt32(7), UInt32(12), Int64(63), 0.035, "u32", "GoldenFloat", "Experimental", "this work; 17-squared empirical PHI_BIAS=289", "high-precision edge", "self", "specs/numeric/gf20.t27 (spec only)"),
    Format("gf24", "GF24", UInt32(24), UInt32(1), UInt32(9), UInt32(14), Int64(255), 0.025, "u32", "GoldenFloat", "Experimental", "this work; rule e=round(23/phi^2)=9; normative bias=2^(e-1)-1=255; empirical PHI_BIAS=1364 (=L15) OPEN", "server inference", "self", "specs/numeric/gf24.t27 (spec only)"),
    Format("gf32", "GF32", UInt32(32), UInt32(1), UInt32(12), UInt32(19), Int64(2047), 0.014, "u32", "GoldenFloat", "Verified", "this work; F0 resolved", "fp32 drop-in", "self", "BENCH-012 (specs/numeric/gf32.t27)"),
    Format("gf64", "GF64", UInt32(64), UInt32(1), UInt32(24), UInt32(39), Int64(8388607), 0.003, "u64", "GoldenFloat", "Verified", "this work; EXP_MAX - BIAS", "scientific / double", "self", "BENCH-007b (specs/numeric/gf64.t27)"),
    Format("gf6", "GF6 (rule-derived)", UInt32(6), UInt32(1), UInt32(2), UInt32(3), Int64(1), 0.049, "u8_packed", "GoldenFloat", "Open", "this work; rule e=round(5/phi^2)=2; FP6 E2M3 bridge", "OPEN R&D: bridge GF4-GF8; FP6 E2M3 hint", "experimental", "specs/numeric/gf6.t27"),
    Format("gf10", "GF10 (rule-derived)", UInt32(10), UInt32(1), UInt32(3), UInt32(6), Int64(3), 0.118, "u16", "GoldenFloat", "Open", "this work; rule e=round(9/phi^2)=3; bridge GF8-GF12", "OPEN R&D: tight-precision activations", "experimental", "specs/numeric/gf10.t27"),
    Format("gf14", "GF14 (rule-derived)", UInt32(14), UInt32(1), UInt32(5), UInt32(8), Int64(15), 0.007, "u16", "GoldenFloat", "Open", "this work; rule e=round(13/phi^2)=5; bridge GF12-GF16; lowest phi-dist below GF48", "OPEN R&D: drop-in for fp16 with tighter phi alignment", "experimental", "specs/numeric/gf14.t27"),
    Format("gf48", "GF48 (rule-derived)", UInt32(48), UInt32(1), UInt32(18), UInt32(29), Int64(131071), 0.003, "u64_padded", "GoldenFloat", "Open", "this work; rule e=round(47/phi^2)=18", "OPEN R&D: between GF32 and GF64; tightest phi-dist of the wide rungs", "experimental", "specs/numeric/gf48.t27"),
    Format("gf96", "GF96 (rule-derived)", UInt32(96), UInt32(1), UInt32(36), UInt32(59), Int64(34359738367), 0.008, "u128_padded", "GoldenFloat", "Open", "this work; rule e=round(95/phi^2)=36", "OPEN R&D: between GF64 and GF128 (phi-aligned extended)", "experimental", "specs/numeric/gf96.t27"),
    Format("gf128", "GF128 (rule-derived)", UInt32(128), UInt32(1), UInt32(49), UInt32(78), Int64(281474976710655), 0.01, "u128", "GoldenFloat", "Open", "this work; rule e=round(127/phi^2)=49 (corrects v1.1 typo e=48)", "OPEN R&D: phi-aligned binary128 alternative", "experimental", "specs/numeric/gf128.t27"),
    Format("gf256", "GF256 (rule-derived)", UInt32(256), UInt32(1), UInt32(97), UInt32(158), Int64(79228162514264337593543950335), 0.004, "u256_software", "GoldenFloat", "Open", "this work; rule e=round(255/phi^2)=97; normative bias=2^96-1", "OPEN R&D: phi-aligned binary256 alternative", "experimental", "specs/numeric/gf256.t27"),
    Format("gf512", "GF512 (rule-derived)", UInt32(512), UInt32(1), UInt32(195), UInt32(316), Int64(-2), 0.0009, "u512_software", "GoldenFloat", "Open", "this work; rule e=round(511/phi^2)=195", "OPEN R&D: ultra-wide phi-aligned (extrapolation, no RTL)", "experimental", "specs/numeric/gf512.t27"),
    Format("gf1024", "GF1024 (rule-derived)", UInt32(1024), UInt32(1), UInt32(391), UInt32(632), Int64(-2), 0.0006, "u1024_software", "GoldenFloat", "Open", "this work; rule e=round(1023/phi^2)=391; lowest phi-distance in the ladder", "OPEN R&D: limit-of-ladder phi alignment (extrapolation, no RTL)", "experimental", "specs/numeric/gf1024.t27"),
    Format("gf8_bfp", "GF8-BFP (block FP atop GF8)", UInt32(8), UInt32(1), UInt32(3), UInt32(4), Int64(3), 0.132, "u8_plus_shared_exp", "GoldenFloat", "Experimental", "this work; per-tile shared exponent", "OPEN R&D: LLM-quantization-friendly GF8", "experimental", "section12.5"),
    Format("gf_lns_hybrid", "GF + LNS hybrid (dual-space)", UInt32(16), UInt32(1), UInt32(6), UInt32(9), Int64(31), 0.049, "u16_plus_lns_path", "GoldenFloat", "Experimental", "this work; mul in log-space, accumulate Lucas-closed", "OPEN R&D: dual-space arithmetic", "experimental", "section12.5"),
    Format("mxgf6", "MXGF6 (microscaling GF6)", UInt32(6), UInt32(1), UInt32(2), UInt32(3), Int64(1), 0.05, "u8_packed_plus_e8m0", "GoldenFloat", "Experimental", "this work; OCP MX block + GF6", "OPEN R&D: phi-aligned MX-6 candidate", "experimental", "section12.5"),
    Format("mxgf4", "MXGF4 (microscaling GF4)", UInt32(4), UInt32(1), UInt32(1), UInt32(2), Int64(0), 0.118, "u8_packed_plus_e8m0", "GoldenFloat", "Experimental", "this work; OCP MX block + GF4", "OPEN R&D: phi-aligned MX-4 candidate", "experimental", "section12.5"),
    Format("int4", "INT4 / UINT4", UInt32(4), UInt32(1), UInt32(0), UInt32(3), Int64(0), -1.0, "u8_packed", "IntegerFixed", "Verified", "two complement", "aggressive quantization", "competitor", "ISO/IEC 9899"),
    Format("int8", "INT8 / UINT8", UInt32(8), UInt32(1), UInt32(0), UInt32(7), Int64(0), -1.0, "u8", "IntegerFixed", "Verified", "two complement", "INT8 inference, per-channel scale", "competitor", "ISO/IEC 9899"),
    Format("int16", "INT16 / UINT16", UInt32(16), UInt32(1), UInt32(0), UInt32(15), Int64(0), -1.0, "u16", "IntegerFixed", "Verified", "two complement", "DSP, embedded ML", "competitor", "ISO/IEC 9899"),
    Format("int32", "INT32 / UINT32", UInt32(32), UInt32(1), UInt32(0), UInt32(31), Int64(0), -1.0, "u32", "IntegerFixed", "Verified", "two complement", "general CPU integer", "competitor", "ISO/IEC 9899"),
    Format("int64", "INT64 / UINT64", UInt32(64), UInt32(1), UInt32(0), UInt32(63), Int64(0), -1.0, "u64", "IntegerFixed", "Verified", "two complement", "databases, timestamps", "competitor", "ISO/IEC 9899"),
    Format("int128", "INT128 / UINT128", UInt32(128), UInt32(1), UInt32(0), UInt32(127), Int64(0), -1.0, "u128", "IntegerFixed", "Verified", "two complement", "crypto, big-int", "competitor", "Rust/Clang u128"),
    Format("q_format", "Q-format (Qm.n)", UInt32(0), UInt32(1), UInt32(0), UInt32(0), Int64(0), -1.0, "varies", "IntegerFixed", "Verified", "TI fixed-point", "audio DSP, fixed-point ML", "orthogonal", "TI SPRA704"),
    Format("bcd", "BCD (binary-coded decimal)", UInt32(0), UInt32(0), UInt32(0), UInt32(0), Int64(0), -1.0, "u4_per_digit", "IntegerFixed", "Historical", "IBM 1959", "calculators, GAAP", "orthogonal", "ISO/IEC 8859"),
    Format("ibm_hfp32", "IBM HFP (single)", UInt32(32), UInt32(1), UInt32(7), UInt32(24), Int64(64), -1.0, "u32", "HistoricalVendor", "Historical", "IBM System/360 (1964); base-16 exponent", "legacy mainframe", "orthogonal", "IBM POO"),
    Format("ibm_hfp64", "IBM HFP (double)", UInt32(64), UInt32(1), UInt32(7), UInt32(56), Int64(64), -1.0, "u64", "HistoricalVendor", "Historical", "IBM System/360 (1964)", "legacy mainframe", "orthogonal", "IBM POO"),
    Format("ibm_hfp128", "IBM HFP (extended)", UInt32(128), UInt32(1), UInt32(7), UInt32(120), Int64(64), -1.0, "u128", "HistoricalVendor", "Historical", "IBM z/Architecture", "legacy mainframe", "orthogonal", "IBM POO"),
    Format("ms_mbf32", "Microsoft MBF (single)", UInt32(32), UInt32(1), UInt32(8), UInt32(23), Int64(129), -1.0, "u32", "HistoricalVendor", "Historical", "MS BASIC / MS-DOS (pre-IEEE)", "MS BASIC legacy", "orthogonal", "MS-DOS docs"),
    Format("ms_mbf64", "Microsoft MBF (double)", UInt32(64), UInt32(1), UInt32(8), UInt32(55), Int64(129), -1.0, "u64", "HistoricalVendor", "Historical", "MS BASIC", "MS BASIC legacy", "orthogonal", "MS-DOS docs"),
    Format("vax_f", "VAX F-float", UInt32(32), UInt32(1), UInt32(8), UInt32(23), Int64(128), -1.0, "u32", "HistoricalVendor", "Historical", "DEC VAX", "DEC legacy", "orthogonal", "VAX Architecture Reference"),
    Format("vax_d", "VAX D-float", UInt32(64), UInt32(1), UInt32(8), UInt32(55), Int64(128), -1.0, "u64", "HistoricalVendor", "Historical", "DEC VAX", "DEC legacy double", "orthogonal", "VAX Architecture Reference"),
    Format("vax_g", "VAX G-float", UInt32(64), UInt32(1), UInt32(11), UInt32(52), Int64(1024), -1.0, "u64", "HistoricalVendor", "Historical", "DEC VAX (IEEE-like)", "DEC legacy", "orthogonal", "VAX Architecture Reference"),
    Format("vax_h", "VAX H-float", UInt32(128), UInt32(1), UInt32(15), UInt32(112), Int64(16384), -1.0, "u128", "HistoricalVendor", "Historical", "DEC VAX", "DEC quad", "orthogonal", "VAX Architecture Reference"),
    Format("cray_float", "Cray float", UInt32(64), UInt32(1), UInt32(15), UInt32(48), Int64(16384), -1.0, "u64", "HistoricalVendor", "Historical", "Cray-1 (1976); no NaN/Inf, unrounded mul", "Cray legacy", "orthogonal", "Cray-1 Hardware Reference"),
    Format("minifloat", "minifloat (arbitrary E:M, <=16 bits)", UInt32(0), UInt32(1), UInt32(0), UInt32(0), Int64(0), -1.0, "varies", "Theoretical", "Experimental", "parametric framework", "design space of GF4/GF8/GF12/GF16", "ally", "Higham 1996"),
    Format("unum_i", "Unum I (tapered + ubound)", UInt32(0), UInt32(1), UInt32(0), UInt32(0), Int64(0), -1.0, "varies", "Theoretical", "Experimental", "Gustafson 2015 (predecessor to posit)", "interval arithmetic", "ally", "Gustafson 2015 (The End of Error)"),
    Format("unum_ii", "Unum II (SORN projective)", UInt32(0), UInt32(0), UInt32(0), UInt32(0), Int64(0), -1.0, "lookup_table", "Theoretical", "Experimental", "Gustafson 2016", "lookup-table real arithmetic; not GF-comparable", "orthogonal", "Gustafson 2016"),
    Format("tapered_fp", "tapered floating point", UInt32(0), UInt32(1), UInt32(0), UInt32(0), Int64(0), -1.0, "varies", "Theoretical", "Experimental", "Morris 1971; posit ancestor", "variable mantissa via regime bits", "ally", "Morris 1971 (IEEE TC)"),
    Format("block_fp", "block floating point (BFP)", UInt32(0), UInt32(0), UInt32(0), UInt32(0), Int64(0), -1.0, "varies", "CompressionTrick", "Verified", "Wilkinson 1965; modern revivals", "per-tile shared exponent", "ally", "Darvish-Rouhani 2020"),
    Format("shared_exp", "shared-exponent formats", UInt32(0), UInt32(0), UInt32(0), UInt32(0), Int64(0), -1.0, "varies", "CompressionTrick", "Verified", "generalised BFP", "LLM quantization", "ally", "Darvish-Rouhani 2020"),
    Format("per_channel_scale", "INT8 with per-channel scale", UInt32(8), UInt32(1), UInt32(0), UInt32(7), Int64(0), -1.0, "u8_plus_fp32_scale", "CompressionTrick", "Verified", "Jacob 2018 (TFLite)", "standard quant inference", "competitor", "Jacob 2018 (CVPR)"),
    Format("stochastic_rounding", "stochastic rounding (technique)", UInt32(0), UInt32(0), UInt32(0), UInt32(0), Int64(0), -1.0, "varies", "CompressionTrick", "Verified", "Gupta 2015", "training small networks at low precision", "ally", "Gupta 2015 (ICML)"),
]

end # module
