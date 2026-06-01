// Generated from formats_catalog.t27. Do not edit by hand.
// SPDX-License-Identifier: Apache-2.0
#pragma once
#include <array>
#include <cstdint>
#include <string_view>

namespace t27 {

struct Format {
    std::string_view id;
    std::string_view name;
    std::uint32_t bits;
    std::uint32_t s_bits;
    std::uint32_t e_bits;
    std::uint32_t m_bits;
    std::int64_t  bias;
    double phi_distance; // -1.0 == undefined
    std::string_view storage;
    std::string_view cluster;
    std::string_view status;
    std::string_view standard;
    std::string_view use_case;
    std::string_view gf_relation;
    std::string_view source;
};

inline constexpr std::array<Format, 77> FORMATS = {{
    Format{ "binary16", "binary16 (fp16, half)", 16u, 1u, 5u, 10u, 15, 0.118, "u16", "Ieee754Binary", "Verified", "IEEE 754-2008", "GPU activations, inference", "competitor", "IEEE 754-2008" },
    Format{ "binary32", "binary32 (fp32, single)", 32u, 1u, 8u, 23u, 127, 0.27, "u32", "Ieee754Binary", "Verified", "IEEE 754-1985", "industry default", "competitor", "IEEE 754-1985" },
    Format{ "binary64", "binary64 (fp64, double)", 64u, 1u, 11u, 52u, 1023, 0.406, "u64", "Ieee754Binary", "Verified", "IEEE 754-1985", "scientific computing", "competitor", "IEEE 754-1985" },
    Format{ "binary128", "binary128 (fp128, quad)", 128u, 1u, 15u, 112u, 16383, 0.484, "u128", "Ieee754Binary", "Verified", "IEEE 754-2008", "high-precision simulations", "competitor", "IEEE 754-2008" },
    Format{ "binary256", "binary256 (octuple)", 256u, 1u, 19u, 236u, 262143, 0.538, "u256_software", "Ieee754Binary", "Verified", "IEEE 754-2008", "astronomy, cryptography", "competitor", "IEEE 754-2008" },
    Format{ "decimal32", "decimal32", 32u, 1u, 11u, 20u, 101, -1.0, "u32", "Ieee754Decimal", "Verified", "IEEE 754-2008 (DPD/BID)", "banking, GAAP", "orthogonal", "IEEE 754-2008" },
    Format{ "decimal64", "decimal64", 64u, 1u, 13u, 50u, 398, -1.0, "u64", "Ieee754Decimal", "Verified", "IEEE 754-2008", "financial databases", "orthogonal", "IEEE 754-2008" },
    Format{ "decimal128", "decimal128", 128u, 1u, 17u, 110u, 6176, -1.0, "u128", "Ieee754Decimal", "Verified", "IEEE 754-2008", "audit ledgers", "orthogonal", "IEEE 754-2008" },
    Format{ "x87_fp80", "x87 FP80", 80u, 1u, 15u, 64u, 16383, -1.0, "u80_padded", "ExtendedFloat", "Historical", "Intel x87 (explicit integer bit)", "legacy long double on x86", "orthogonal", "Intel SDM" },
    Format{ "double_double", "double-double", 128u, 2u, 22u, 104u, 0, -1.0, "two_u64", "ExtendedFloat", "Verified", "Bailey/Hida (software)", "software extended precision", "orthogonal", "Bailey-Hida 2001" },
    Format{ "quad_double", "quad-double", 256u, 4u, 44u, 208u, 0, -1.0, "four_u64", "ExtendedFloat", "Verified", "Bailey/Hida (software)", "astrophysics, quad-precision sims", "orthogonal", "Bailey-Hida 2001" },
    Format{ "bfloat16", "bfloat16 (BF16)", 16u, 1u, 8u, 7u, 127, 0.525, "u16", "MlLowPrecision", "Verified", "Google Brain", "training (range > precision)", "competitor", "Wang-Kanwar 2019" },
    Format{ "tf32", "TensorFloat-32 (TF32)", 19u, 1u, 8u, 10u, 127, 0.27, "u32_padded", "MlLowPrecision", "Verified", "NVIDIA Ampere", "A100/H100 mixed precision", "competitor", "NVIDIA Ampere whitepaper" },
    Format{ "fp8_e4m3", "FP8 E4M3", 8u, 1u, 4u, 3u, 7, 0.715, "u8", "MlLowPrecision", "Verified", "OCP / NVIDIA / Arm / Intel", "inference, gradient ranges", "competitor", "Micikevicius 2022 (arXiv:2209.05433)" },
    Format{ "fp8_e5m2", "FP8 E5M2", 8u, 1u, 5u, 2u, 15, 1.882, "u8", "MlLowPrecision", "Verified", "OCP / NVIDIA", "activations, wide range", "competitor", "Micikevicius 2022" },
    Format{ "fp6_e3m2", "FP6 E3M2", 6u, 1u, 3u, 2u, 3, 0.882, "u8_packed", "MlLowPrecision", "Verified", "OCP MX", "aggressive quant inference", "competitor", "OCP MX v1.0 (2023)" },
    Format{ "fp6_e2m3", "FP6 E2M3", 6u, 1u, 2u, 3u, 1, 0.049, "u8_packed", "MlLowPrecision", "Verified", "OCP MX", "mantissa-heavy quant", "ally", "OCP MX v1.0 (2023)" },
    Format{ "fp4_e2m1", "FP4 E2M1", 4u, 1u, 2u, 1u, 1, 1.382, "u8_packed", "MlLowPrecision", "Verified", "OCP MX", "extreme quant inference", "competitor", "OCP MX v1.0 (2023)" },
    Format{ "mxfp8", "MXFP8", 8u, 1u, 4u, 3u, 7, 0.715, "u8_plus_shared_e8m0", "Microscaling", "Verified", "OCP MX v1.0", "LLM inference", "ally", "Rouhani 2023 (arXiv:2310.10537)" },
    Format{ "mxfp6", "MXFP6", 6u, 1u, 3u, 2u, 3, 0.882, "u8_packed_plus_e8m0", "Microscaling", "Verified", "OCP MX v1.0", "aggressive inference", "ally", "Rouhani 2023" },
    Format{ "mxfp4", "MXFP4", 4u, 1u, 2u, 1u, 1, 1.382, "u8_packed_plus_e8m0", "Microscaling", "Verified", "OCP MX v1.0", "extreme quant", "ally", "Rouhani 2023" },
    Format{ "nf4", "NF4 (NormalFloat 4-bit)", 4u, 0u, 0u, 4u, 0, -1.0, "u8_packed", "QuantTuned", "Verified", "Dettmers 2023 (QLoRA)", "LLM weight quantization (quantile-based on N(0,1))", "orthogonal", "Dettmers 2023 (arXiv:2305.14314)" },
    Format{ "afp", "AFP (Adaptive Floating-Point)", 16u, 1u, 8u, 7u, 127, -1.0, "u16_plus_tensor_shift", "QuantTuned", "Verified", "Tambe 2020", "efficient training", "orthogonal", "Tambe 2020 (DAC)" },
    Format{ "posit8", "Posit8", 8u, 1u, 2u, 0u, 0, -1.0, "u8", "PositUnumIII", "Verified", "Posit Standard 2022 (es=2)", "inference", "ally", "Posit Standard 2022 (posithub.org)" },
    Format{ "posit16", "Posit16", 16u, 1u, 2u, 0u, 0, -1.0, "u16", "PositUnumIII", "Verified", "Posit Standard 2022 (es=2)", "mixed-precision training", "ally", "Posit Standard 2022" },
    Format{ "posit32", "Posit32", 32u, 1u, 2u, 0u, 0, -1.0, "u32", "PositUnumIII", "Verified", "Posit Standard 2022 (es=2)", "f32 replacement", "ally", "Posit Standard 2022" },
    Format{ "posit64", "Posit64", 64u, 1u, 2u, 0u, 0, -1.0, "u64", "PositUnumIII", "Verified", "Posit Standard 2022 (es=2)", "f64 replacement", "ally", "Posit Standard 2022" },
    Format{ "takum8", "takum8", 8u, 1u, 0u, 0u, 0, -1.0, "u8", "PositUnumIII", "Verified", "Hunhold 2024 (tapered-precision)", "IEEE-754 backward-compatible tapered", "ally", "Hunhold 2024 (arXiv:2412.20273)" },
    Format{ "takum16", "takum16", 16u, 1u, 0u, 0u, 0, -1.0, "u16", "PositUnumIII", "Verified", "Hunhold 2024", "single-rule ladder counterexample", "ally", "Hunhold 2024 (arXiv:2412.20273)" },
    Format{ "takum32", "takum32", 32u, 1u, 0u, 0u, 0, -1.0, "u32", "PositUnumIII", "Verified", "Hunhold 2024", "tapered fp32-class", "ally", "Hunhold 2024" },
    Format{ "takum64", "takum64", 64u, 1u, 0u, 0u, 0, -1.0, "u64", "PositUnumIII", "Verified", "Hunhold 2024", "tapered fp64-class", "ally", "Hunhold 2024" },
    Format{ "lns8", "LNS-8", 8u, 1u, 7u, 0u, 0, -1.0, "u8", "Lns", "Verified", "Arnold 1990; LNS-Madam (2021)", "DSP, signal processing", "orthogonal", "Alam 2021 (arXiv:2106.13914)" },
    Format{ "lns16", "LNS-16", 16u, 1u, 15u, 0u, 0, -1.0, "u16", "Lns", "Verified", "LNS-Madam (2021)", "log-domain training (mul -> add)", "orthogonal", "Alam 2021" },
    Format{ "lns32", "LNS-32", 32u, 1u, 31u, 0u, 0, -1.0, "u32", "Lns", "Verified", "LNS-Madam (2021)", "log-domain DSP", "orthogonal", "Alam 2021" },
    Format{ "lns64", "LNS-64", 64u, 1u, 63u, 0u, 0, -1.0, "u64", "Lns", "Verified", "LNS-Madam (2021)", "scientific log-domain", "orthogonal", "Alam 2021" },
    Format{ "gfternary", "GFTernary", 2u, 1u, 0u, 2u, 0, 0.0, "u2", "GoldenFloat", "Verified", "this work; {-phi, 0, +phi}", "bulk layers (hybrid)", "self", "BENCH-007" },
    Format{ "gf4", "GF4", 4u, 1u, 1u, 2u, 0, 0.118, "u8", "GoldenFloat", "Experimental", "this work; F0 minimal", "proof-of-concept", "self", "specs/numeric/gf4.t27" },
    Format{ "gf8", "GF8", 8u, 1u, 3u, 4u, 3, 0.132, "u8", "GoldenFloat", "Verified", "this work; L1 Lucas", "edge / sensors", "self", "BENCH-007 (specs/numeric/gf8.t27)" },
    Format{ "gf12", "GF12", 12u, 1u, 4u, 7u, 7, 0.047, "u16", "GoldenFloat", "Verified", "this work; L0/F3", "mid-range / audio", "self", "BENCH-007 (specs/numeric/gf12.t27)" },
    Format{ "gf16", "GF16", 16u, 1u, 6u, 9u, 31, 0.049, "u16", "GoldenFloat", "Verified", "this work; PHI_BIAS=60; FPGA 35/35 at 323 MHz Artix-7", "training and inference (production)", "self", "specs/numeric/gf16.t27; zenodo 10.5281/zenodo.19227877 (HW archive)" },
    Format{ "gf20", "GF20", 20u, 1u, 7u, 12u, 63, 0.035, "u32", "GoldenFloat", "Experimental", "this work; 17-squared empirical PHI_BIAS=289", "high-precision edge", "self", "specs/numeric/gf20.t27 (spec only)" },
    Format{ "gf24", "GF24", 24u, 1u, 9u, 14u, 255, 0.025, "u32", "GoldenFloat", "Experimental", "this work; L15 PHI_BIAS=1364", "server inference", "self", "specs/numeric/gf24.t27 (spec only)" },
    Format{ "gf32", "GF32", 32u, 1u, 12u, 19u, 2047, 0.014, "u32", "GoldenFloat", "Verified", "this work; F0 resolved", "fp32 drop-in", "self", "BENCH-012 (specs/numeric/gf32.t27)" },
    Format{ "gf64", "GF64", 64u, 1u, 24u, 39u, 8388607, 0.003, "u64", "GoldenFloat", "Verified", "this work; EXP_MAX - BIAS", "scientific / double", "self", "BENCH-007b (specs/numeric/gf64.t27)" },
    Format{ "gf6", "GF6 (predicted)", 6u, 1u, 2u, 3u, 1, 0.05, "u8_packed", "GoldenFloat", "Experimental", "this work; e=round(5/phi^2)=2, fills FP6 gap", "OPEN R&D: bridge GF4-GF8; FP6 E2M3 hint", "experimental", "section12.5" },
    Format{ "gf128", "GF128 (predicted)", 128u, 1u, 48u, 79u, 0, 0.008, "u128", "GoldenFloat", "Experimental", "this work; e=round(127/phi^2)=48 (Open: bias TBD)", "OPEN R&D: phi-aligned binary128 alternative", "experimental", "section12.5" },
    Format{ "gf256", "GF256 (predicted)", 256u, 1u, 97u, 158u, 0, 0.005, "u256_software", "GoldenFloat", "Experimental", "this work; e=round(255/phi^2)=97 (Open: bias ~2^71 unconfirmed)", "OPEN R&D: phi-aligned binary256 alternative", "experimental", "section12.5; bias Open per skill" },
    Format{ "gf8_bfp", "GF8-BFP (block FP atop GF8)", 8u, 1u, 3u, 4u, 3, 0.132, "u8_plus_shared_exp", "GoldenFloat", "Experimental", "this work; per-tile shared exponent", "OPEN R&D: LLM-quantization-friendly GF8", "experimental", "section12.5" },
    Format{ "gf_lns_hybrid", "GF + LNS hybrid (dual-space)", 16u, 1u, 6u, 9u, 31, 0.049, "u16_plus_lns_path", "GoldenFloat", "Experimental", "this work; mul in log-space, accumulate Lucas-closed", "OPEN R&D: dual-space arithmetic", "experimental", "section12.5" },
    Format{ "mxgf6", "MXGF6 (microscaling GF6)", 6u, 1u, 2u, 3u, 1, 0.05, "u8_packed_plus_e8m0", "GoldenFloat", "Experimental", "this work; OCP MX block + GF6", "OPEN R&D: phi-aligned MX-6 candidate", "experimental", "section12.5" },
    Format{ "mxgf4", "MXGF4 (microscaling GF4)", 4u, 1u, 1u, 2u, 0, 0.118, "u8_packed_plus_e8m0", "GoldenFloat", "Experimental", "this work; OCP MX block + GF4", "OPEN R&D: phi-aligned MX-4 candidate", "experimental", "section12.5" },
    Format{ "int4", "INT4 / UINT4", 4u, 1u, 0u, 3u, 0, -1.0, "u8_packed", "IntegerFixed", "Verified", "two complement", "aggressive quantization", "competitor", "ISO/IEC 9899" },
    Format{ "int8", "INT8 / UINT8", 8u, 1u, 0u, 7u, 0, -1.0, "u8", "IntegerFixed", "Verified", "two complement", "INT8 inference, per-channel scale", "competitor", "ISO/IEC 9899" },
    Format{ "int16", "INT16 / UINT16", 16u, 1u, 0u, 15u, 0, -1.0, "u16", "IntegerFixed", "Verified", "two complement", "DSP, embedded ML", "competitor", "ISO/IEC 9899" },
    Format{ "int32", "INT32 / UINT32", 32u, 1u, 0u, 31u, 0, -1.0, "u32", "IntegerFixed", "Verified", "two complement", "general CPU integer", "competitor", "ISO/IEC 9899" },
    Format{ "int64", "INT64 / UINT64", 64u, 1u, 0u, 63u, 0, -1.0, "u64", "IntegerFixed", "Verified", "two complement", "databases, timestamps", "competitor", "ISO/IEC 9899" },
    Format{ "int128", "INT128 / UINT128", 128u, 1u, 0u, 127u, 0, -1.0, "u128", "IntegerFixed", "Verified", "two complement", "crypto, big-int", "competitor", "Rust/Clang u128" },
    Format{ "q_format", "Q-format (Qm.n)", 0u, 1u, 0u, 0u, 0, -1.0, "varies", "IntegerFixed", "Verified", "TI fixed-point", "audio DSP, fixed-point ML", "orthogonal", "TI SPRA704" },
    Format{ "bcd", "BCD (binary-coded decimal)", 0u, 0u, 0u, 0u, 0, -1.0, "u4_per_digit", "IntegerFixed", "Historical", "IBM 1959", "calculators, GAAP", "orthogonal", "ISO/IEC 8859" },
    Format{ "ibm_hfp32", "IBM HFP (single)", 32u, 1u, 7u, 24u, 64, -1.0, "u32", "HistoricalVendor", "Historical", "IBM System/360 (1964); base-16 exponent", "legacy mainframe", "orthogonal", "IBM POO" },
    Format{ "ibm_hfp64", "IBM HFP (double)", 64u, 1u, 7u, 56u, 64, -1.0, "u64", "HistoricalVendor", "Historical", "IBM System/360 (1964)", "legacy mainframe", "orthogonal", "IBM POO" },
    Format{ "ibm_hfp128", "IBM HFP (extended)", 128u, 1u, 7u, 120u, 64, -1.0, "u128", "HistoricalVendor", "Historical", "IBM z/Architecture", "legacy mainframe", "orthogonal", "IBM POO" },
    Format{ "ms_mbf32", "Microsoft MBF (single)", 32u, 1u, 8u, 23u, 129, -1.0, "u32", "HistoricalVendor", "Historical", "MS BASIC / MS-DOS (pre-IEEE)", "MS BASIC legacy", "orthogonal", "MS-DOS docs" },
    Format{ "ms_mbf64", "Microsoft MBF (double)", 64u, 1u, 8u, 55u, 129, -1.0, "u64", "HistoricalVendor", "Historical", "MS BASIC", "MS BASIC legacy", "orthogonal", "MS-DOS docs" },
    Format{ "vax_f", "VAX F-float", 32u, 1u, 8u, 23u, 128, -1.0, "u32", "HistoricalVendor", "Historical", "DEC VAX", "DEC legacy", "orthogonal", "VAX Architecture Reference" },
    Format{ "vax_d", "VAX D-float", 64u, 1u, 8u, 55u, 128, -1.0, "u64", "HistoricalVendor", "Historical", "DEC VAX", "DEC legacy double", "orthogonal", "VAX Architecture Reference" },
    Format{ "vax_g", "VAX G-float", 64u, 1u, 11u, 52u, 1024, -1.0, "u64", "HistoricalVendor", "Historical", "DEC VAX (IEEE-like)", "DEC legacy", "orthogonal", "VAX Architecture Reference" },
    Format{ "vax_h", "VAX H-float", 128u, 1u, 15u, 112u, 16384, -1.0, "u128", "HistoricalVendor", "Historical", "DEC VAX", "DEC quad", "orthogonal", "VAX Architecture Reference" },
    Format{ "cray_float", "Cray float", 64u, 1u, 15u, 48u, 16384, -1.0, "u64", "HistoricalVendor", "Historical", "Cray-1 (1976); no NaN/Inf, unrounded mul", "Cray legacy", "orthogonal", "Cray-1 Hardware Reference" },
    Format{ "minifloat", "minifloat (arbitrary E:M, <=16 bits)", 0u, 1u, 0u, 0u, 0, -1.0, "varies", "Theoretical", "Experimental", "parametric framework", "design space of GF4/GF8/GF12/GF16", "ally", "Higham 1996" },
    Format{ "unum_i", "Unum I (tapered + ubound)", 0u, 1u, 0u, 0u, 0, -1.0, "varies", "Theoretical", "Experimental", "Gustafson 2015 (predecessor to posit)", "interval arithmetic", "ally", "Gustafson 2015 (The End of Error)" },
    Format{ "unum_ii", "Unum II (SORN projective)", 0u, 0u, 0u, 0u, 0, -1.0, "lookup_table", "Theoretical", "Experimental", "Gustafson 2016", "lookup-table real arithmetic; not GF-comparable", "orthogonal", "Gustafson 2016" },
    Format{ "tapered_fp", "tapered floating point", 0u, 1u, 0u, 0u, 0, -1.0, "varies", "Theoretical", "Experimental", "Morris 1971; posit ancestor", "variable mantissa via regime bits", "ally", "Morris 1971 (IEEE TC)" },
    Format{ "block_fp", "block floating point (BFP)", 0u, 0u, 0u, 0u, 0, -1.0, "varies", "CompressionTrick", "Verified", "Wilkinson 1965; modern revivals", "per-tile shared exponent", "ally", "Darvish-Rouhani 2020" },
    Format{ "shared_exp", "shared-exponent formats", 0u, 0u, 0u, 0u, 0, -1.0, "varies", "CompressionTrick", "Verified", "generalised BFP", "LLM quantization", "ally", "Darvish-Rouhani 2020" },
    Format{ "per_channel_scale", "INT8 with per-channel scale", 8u, 1u, 0u, 7u, 0, -1.0, "u8_plus_fp32_scale", "CompressionTrick", "Verified", "Jacob 2018 (TFLite)", "standard quant inference", "competitor", "Jacob 2018 (CVPR)" },
    Format{ "stochastic_rounding", "stochastic rounding (technique)", 0u, 0u, 0u, 0u, 0, -1.0, "varies", "CompressionTrick", "Verified", "Gupta 2015", "training small networks at low precision", "ally", "Gupta 2015 (ICML)" },
}};

} // namespace t27
