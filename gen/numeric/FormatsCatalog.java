// Generated from formats_catalog.t27. Do not edit by hand.
// SPDX-License-Identifier: Apache-2.0
package ai.t27.numeric;

import java.util.Arrays;
import java.util.Collections;
import java.util.List;

public final class FormatsCatalog {
    private FormatsCatalog() {}

    public static final class Format {
        public final String id;
        public final String name;
        public final long bits;
        public final long sBits;
        public final long eBits;
        public final long mBits;
        public final long bias;
        public final double phiDistance;
        public final String storage;
        public final String cluster;
        public final String status;
        public final String standard;
        public final String useCase;
        public final String gfRelation;
        public final String source;
        public Format(String id, String name,
                long bits, long sBits, long eBits, long mBits,
                long bias, double phiDistance,
                String storage, String cluster, String status,
                String standard, String useCase,
                String gfRelation, String source) {
            this.id = id; this.name = name;
            this.bits = bits; this.sBits = sBits;
            this.eBits = eBits; this.mBits = mBits;
            this.bias = bias; this.phiDistance = phiDistance;
            this.storage = storage; this.cluster = cluster;
            this.status = status; this.standard = standard;
            this.useCase = useCase; this.gfRelation = gfRelation;
            this.source = source;
        }
    }

    public static final List<Format> FORMATS;
    static {
        Format[] arr = new Format[] {
            new Format("binary16", "binary16 (fp16, half)", 16L, 1L, 5L, 10L, 15L, 0.118, "u16", "Ieee754Binary", "Verified", "IEEE 754-2008", "GPU activations, inference", "competitor", "IEEE 754-2008"),
            new Format("binary32", "binary32 (fp32, single)", 32L, 1L, 8L, 23L, 127L, 0.27, "u32", "Ieee754Binary", "Verified", "IEEE 754-1985", "industry default", "competitor", "IEEE 754-1985"),
            new Format("binary64", "binary64 (fp64, double)", 64L, 1L, 11L, 52L, 1023L, 0.406, "u64", "Ieee754Binary", "Verified", "IEEE 754-1985", "scientific computing", "competitor", "IEEE 754-1985"),
            new Format("binary128", "binary128 (fp128, quad)", 128L, 1L, 15L, 112L, 16383L, 0.484, "u128", "Ieee754Binary", "Verified", "IEEE 754-2008", "high-precision simulations", "competitor", "IEEE 754-2008"),
            new Format("binary256", "binary256 (octuple)", 256L, 1L, 19L, 236L, 262143L, 0.538, "u256_software", "Ieee754Binary", "Verified", "IEEE 754-2008", "astronomy, cryptography", "competitor", "IEEE 754-2008"),
            new Format("decimal32", "decimal32", 32L, 1L, 11L, 20L, 101L, -1.0, "u32", "Ieee754Decimal", "Verified", "IEEE 754-2008 (DPD/BID)", "banking, GAAP", "orthogonal", "IEEE 754-2008"),
            new Format("decimal64", "decimal64", 64L, 1L, 13L, 50L, 398L, -1.0, "u64", "Ieee754Decimal", "Verified", "IEEE 754-2008", "financial databases", "orthogonal", "IEEE 754-2008"),
            new Format("decimal128", "decimal128", 128L, 1L, 17L, 110L, 6176L, -1.0, "u128", "Ieee754Decimal", "Verified", "IEEE 754-2008", "audit ledgers", "orthogonal", "IEEE 754-2008"),
            new Format("x87_fp80", "x87 FP80", 80L, 1L, 15L, 64L, 16383L, -1.0, "u80_padded", "ExtendedFloat", "Historical", "Intel x87 (explicit integer bit)", "legacy long double on x86", "orthogonal", "Intel SDM"),
            new Format("double_double", "double-double", 128L, 2L, 22L, 104L, 0L, -1.0, "two_u64", "ExtendedFloat", "Verified", "Bailey/Hida (software)", "software extended precision", "orthogonal", "Bailey-Hida 2001"),
            new Format("quad_double", "quad-double", 256L, 4L, 44L, 208L, 0L, -1.0, "four_u64", "ExtendedFloat", "Verified", "Bailey/Hida (software)", "astrophysics, quad-precision sims", "orthogonal", "Bailey-Hida 2001"),
            new Format("bfloat16", "bfloat16 (BF16)", 16L, 1L, 8L, 7L, 127L, 0.525, "u16", "MlLowPrecision", "Verified", "Google Brain", "training (range > precision)", "competitor", "Wang-Kanwar 2019"),
            new Format("tf32", "TensorFloat-32 (TF32)", 19L, 1L, 8L, 10L, 127L, 0.27, "u32_padded", "MlLowPrecision", "Verified", "NVIDIA Ampere", "A100/H100 mixed precision", "competitor", "NVIDIA Ampere whitepaper"),
            new Format("fp8_e4m3", "FP8 E4M3", 8L, 1L, 4L, 3L, 7L, 0.715, "u8", "MlLowPrecision", "Verified", "OCP / NVIDIA / Arm / Intel", "inference, gradient ranges", "competitor", "Micikevicius 2022 (arXiv:2209.05433)"),
            new Format("fp8_e5m2", "FP8 E5M2", 8L, 1L, 5L, 2L, 15L, 1.882, "u8", "MlLowPrecision", "Verified", "OCP / NVIDIA", "activations, wide range", "competitor", "Micikevicius 2022"),
            new Format("fp6_e3m2", "FP6 E3M2", 6L, 1L, 3L, 2L, 3L, 0.882, "u8_packed", "MlLowPrecision", "Verified", "OCP MX", "aggressive quant inference", "competitor", "OCP MX v1.0 (2023)"),
            new Format("fp6_e2m3", "FP6 E2M3", 6L, 1L, 2L, 3L, 1L, 0.049, "u8_packed", "MlLowPrecision", "Verified", "OCP MX", "mantissa-heavy quant", "ally", "OCP MX v1.0 (2023)"),
            new Format("fp4_e2m1", "FP4 E2M1", 4L, 1L, 2L, 1L, 1L, 1.382, "u8_packed", "MlLowPrecision", "Verified", "OCP MX", "extreme quant inference", "competitor", "OCP MX v1.0 (2023)"),
            new Format("mxfp8", "MXFP8", 8L, 1L, 4L, 3L, 7L, 0.715, "u8_plus_shared_e8m0", "Microscaling", "Verified", "OCP MX v1.0", "LLM inference", "ally", "Rouhani 2023 (arXiv:2310.10537)"),
            new Format("mxfp6", "MXFP6", 6L, 1L, 3L, 2L, 3L, 0.882, "u8_packed_plus_e8m0", "Microscaling", "Verified", "OCP MX v1.0", "aggressive inference", "ally", "Rouhani 2023"),
            new Format("mxfp4", "MXFP4", 4L, 1L, 2L, 1L, 1L, 1.382, "u8_packed_plus_e8m0", "Microscaling", "Verified", "OCP MX v1.0", "extreme quant", "ally", "Rouhani 2023"),
            new Format("nf4", "NF4 (NormalFloat 4-bit)", 4L, 0L, 0L, 4L, 0L, -1.0, "u8_packed", "QuantTuned", "Verified", "Dettmers 2023 (QLoRA)", "LLM weight quantization (quantile-based on N(0,1))", "orthogonal", "Dettmers 2023 (arXiv:2305.14314)"),
            new Format("afp", "AFP (Adaptive Floating-Point)", 16L, 1L, 8L, 7L, 127L, -1.0, "u16_plus_tensor_shift", "QuantTuned", "Verified", "Tambe 2020", "efficient training", "orthogonal", "Tambe 2020 (DAC)"),
            new Format("posit8", "Posit8", 8L, 1L, 2L, 0L, 0L, -1.0, "u8", "PositUnumIII", "Verified", "Posit Standard 2022 (es=2)", "inference", "ally", "Posit Standard 2022 (posithub.org)"),
            new Format("posit16", "Posit16", 16L, 1L, 2L, 0L, 0L, -1.0, "u16", "PositUnumIII", "Verified", "Posit Standard 2022 (es=2)", "mixed-precision training", "ally", "Posit Standard 2022"),
            new Format("posit32", "Posit32", 32L, 1L, 2L, 0L, 0L, -1.0, "u32", "PositUnumIII", "Verified", "Posit Standard 2022 (es=2)", "f32 replacement", "ally", "Posit Standard 2022"),
            new Format("posit64", "Posit64", 64L, 1L, 2L, 0L, 0L, -1.0, "u64", "PositUnumIII", "Verified", "Posit Standard 2022 (es=2)", "f64 replacement", "ally", "Posit Standard 2022"),
            new Format("takum8", "takum8", 8L, 1L, 0L, 0L, 0L, -1.0, "u8", "PositUnumIII", "Verified", "Hunhold 2024 (tapered-precision)", "IEEE-754 backward-compatible tapered", "ally", "Hunhold 2024 (arXiv:2412.20273)"),
            new Format("takum16", "takum16", 16L, 1L, 0L, 0L, 0L, -1.0, "u16", "PositUnumIII", "Verified", "Hunhold 2024", "single-rule ladder counterexample", "ally", "Hunhold 2024 (arXiv:2412.20273)"),
            new Format("takum32", "takum32", 32L, 1L, 0L, 0L, 0L, -1.0, "u32", "PositUnumIII", "Verified", "Hunhold 2024", "tapered fp32-class", "ally", "Hunhold 2024"),
            new Format("takum64", "takum64", 64L, 1L, 0L, 0L, 0L, -1.0, "u64", "PositUnumIII", "Verified", "Hunhold 2024", "tapered fp64-class", "ally", "Hunhold 2024"),
            new Format("lns8", "LNS-8", 8L, 1L, 7L, 0L, 0L, -1.0, "u8", "Lns", "Verified", "Arnold 1990; LNS-Madam (2021)", "DSP, signal processing", "orthogonal", "Alam 2021 (arXiv:2106.13914)"),
            new Format("lns16", "LNS-16", 16L, 1L, 15L, 0L, 0L, -1.0, "u16", "Lns", "Verified", "LNS-Madam (2021)", "log-domain training (mul -> add)", "orthogonal", "Alam 2021"),
            new Format("lns32", "LNS-32", 32L, 1L, 31L, 0L, 0L, -1.0, "u32", "Lns", "Verified", "LNS-Madam (2021)", "log-domain DSP", "orthogonal", "Alam 2021"),
            new Format("lns64", "LNS-64", 64L, 1L, 63L, 0L, 0L, -1.0, "u64", "Lns", "Verified", "LNS-Madam (2021)", "scientific log-domain", "orthogonal", "Alam 2021"),
            new Format("gfternary", "GFTernary", 2L, 1L, 0L, 2L, 0L, 0.0, "u2", "GoldenFloat", "Verified", "this work; {-phi, 0, +phi}", "bulk layers (hybrid)", "self", "BENCH-007"),
            new Format("gf4", "GF4", 4L, 1L, 1L, 2L, 0L, 0.118, "u8", "GoldenFloat", "Experimental", "this work; F0 minimal", "proof-of-concept", "self", "specs/numeric/gf4.t27"),
            new Format("gf8", "GF8", 8L, 1L, 3L, 4L, 3L, 0.132, "u8", "GoldenFloat", "Verified", "this work; L1 Lucas", "edge / sensors", "self", "BENCH-007 (specs/numeric/gf8.t27)"),
            new Format("gf12", "GF12", 12L, 1L, 4L, 7L, 7L, 0.047, "u16", "GoldenFloat", "Verified", "this work; L0/F3", "mid-range / audio", "self", "BENCH-007 (specs/numeric/gf12.t27)"),
            new Format("gf16", "GF16", 16L, 1L, 6L, 9L, 31L, 0.049, "u16", "GoldenFloat", "Verified", "this work; PHI_BIAS=60; FPGA 35/35 at 323 MHz Artix-7", "training and inference (production)", "self", "specs/numeric/gf16.t27; zenodo 10.5281/zenodo.19227877 (HW archive)"),
            new Format("gf20", "GF20", 20L, 1L, 7L, 12L, 63L, 0.035, "u32", "GoldenFloat", "Experimental", "this work; 17-squared empirical PHI_BIAS=289", "high-precision edge", "self", "specs/numeric/gf20.t27 (spec only)"),
            new Format("gf24", "GF24", 24L, 1L, 9L, 14L, 255L, 0.025, "u32", "GoldenFloat", "Experimental", "this work; L15 PHI_BIAS=1364", "server inference", "self", "specs/numeric/gf24.t27 (spec only)"),
            new Format("gf32", "GF32", 32L, 1L, 12L, 19L, 2047L, 0.014, "u32", "GoldenFloat", "Verified", "this work; F0 resolved", "fp32 drop-in", "self", "BENCH-012 (specs/numeric/gf32.t27)"),
            new Format("gf64", "GF64", 64L, 1L, 24L, 39L, 8388607L, 0.003, "u64", "GoldenFloat", "Verified", "this work; EXP_MAX - BIAS", "scientific / double", "self", "BENCH-007b (specs/numeric/gf64.t27)"),
            new Format("gf6", "GF6 (predicted)", 6L, 1L, 2L, 3L, 1L, 0.05, "u8_packed", "GoldenFloat", "Experimental", "this work; e=round(5/phi^2)=2, fills FP6 gap", "OPEN R&D: bridge GF4-GF8; FP6 E2M3 hint", "experimental", "section12.5"),
            new Format("gf128", "GF128 (predicted)", 128L, 1L, 48L, 79L, 0L, 0.008, "u128", "GoldenFloat", "Experimental", "this work; e=round(127/phi^2)=48 (Open: bias TBD)", "OPEN R&D: phi-aligned binary128 alternative", "experimental", "section12.5"),
            new Format("gf256", "GF256 (predicted)", 256L, 1L, 97L, 158L, 0L, 0.005, "u256_software", "GoldenFloat", "Experimental", "this work; e=round(255/phi^2)=97 (Open: bias ~2^71 unconfirmed)", "OPEN R&D: phi-aligned binary256 alternative", "experimental", "section12.5; bias Open per skill"),
            new Format("gf8_bfp", "GF8-BFP (block FP atop GF8)", 8L, 1L, 3L, 4L, 3L, 0.132, "u8_plus_shared_exp", "GoldenFloat", "Experimental", "this work; per-tile shared exponent", "OPEN R&D: LLM-quantization-friendly GF8", "experimental", "section12.5"),
            new Format("gf_lns_hybrid", "GF + LNS hybrid (dual-space)", 16L, 1L, 6L, 9L, 31L, 0.049, "u16_plus_lns_path", "GoldenFloat", "Experimental", "this work; mul in log-space, accumulate Lucas-closed", "OPEN R&D: dual-space arithmetic", "experimental", "section12.5"),
            new Format("mxgf6", "MXGF6 (microscaling GF6)", 6L, 1L, 2L, 3L, 1L, 0.05, "u8_packed_plus_e8m0", "GoldenFloat", "Experimental", "this work; OCP MX block + GF6", "OPEN R&D: phi-aligned MX-6 candidate", "experimental", "section12.5"),
            new Format("mxgf4", "MXGF4 (microscaling GF4)", 4L, 1L, 1L, 2L, 0L, 0.118, "u8_packed_plus_e8m0", "GoldenFloat", "Experimental", "this work; OCP MX block + GF4", "OPEN R&D: phi-aligned MX-4 candidate", "experimental", "section12.5"),
            new Format("int4", "INT4 / UINT4", 4L, 1L, 0L, 3L, 0L, -1.0, "u8_packed", "IntegerFixed", "Verified", "two complement", "aggressive quantization", "competitor", "ISO/IEC 9899"),
            new Format("int8", "INT8 / UINT8", 8L, 1L, 0L, 7L, 0L, -1.0, "u8", "IntegerFixed", "Verified", "two complement", "INT8 inference, per-channel scale", "competitor", "ISO/IEC 9899"),
            new Format("int16", "INT16 / UINT16", 16L, 1L, 0L, 15L, 0L, -1.0, "u16", "IntegerFixed", "Verified", "two complement", "DSP, embedded ML", "competitor", "ISO/IEC 9899"),
            new Format("int32", "INT32 / UINT32", 32L, 1L, 0L, 31L, 0L, -1.0, "u32", "IntegerFixed", "Verified", "two complement", "general CPU integer", "competitor", "ISO/IEC 9899"),
            new Format("int64", "INT64 / UINT64", 64L, 1L, 0L, 63L, 0L, -1.0, "u64", "IntegerFixed", "Verified", "two complement", "databases, timestamps", "competitor", "ISO/IEC 9899"),
            new Format("int128", "INT128 / UINT128", 128L, 1L, 0L, 127L, 0L, -1.0, "u128", "IntegerFixed", "Verified", "two complement", "crypto, big-int", "competitor", "Rust/Clang u128"),
            new Format("q_format", "Q-format (Qm.n)", 0L, 1L, 0L, 0L, 0L, -1.0, "varies", "IntegerFixed", "Verified", "TI fixed-point", "audio DSP, fixed-point ML", "orthogonal", "TI SPRA704"),
            new Format("bcd", "BCD (binary-coded decimal)", 0L, 0L, 0L, 0L, 0L, -1.0, "u4_per_digit", "IntegerFixed", "Historical", "IBM 1959", "calculators, GAAP", "orthogonal", "ISO/IEC 8859"),
            new Format("ibm_hfp32", "IBM HFP (single)", 32L, 1L, 7L, 24L, 64L, -1.0, "u32", "HistoricalVendor", "Historical", "IBM System/360 (1964); base-16 exponent", "legacy mainframe", "orthogonal", "IBM POO"),
            new Format("ibm_hfp64", "IBM HFP (double)", 64L, 1L, 7L, 56L, 64L, -1.0, "u64", "HistoricalVendor", "Historical", "IBM System/360 (1964)", "legacy mainframe", "orthogonal", "IBM POO"),
            new Format("ibm_hfp128", "IBM HFP (extended)", 128L, 1L, 7L, 120L, 64L, -1.0, "u128", "HistoricalVendor", "Historical", "IBM z/Architecture", "legacy mainframe", "orthogonal", "IBM POO"),
            new Format("ms_mbf32", "Microsoft MBF (single)", 32L, 1L, 8L, 23L, 129L, -1.0, "u32", "HistoricalVendor", "Historical", "MS BASIC / MS-DOS (pre-IEEE)", "MS BASIC legacy", "orthogonal", "MS-DOS docs"),
            new Format("ms_mbf64", "Microsoft MBF (double)", 64L, 1L, 8L, 55L, 129L, -1.0, "u64", "HistoricalVendor", "Historical", "MS BASIC", "MS BASIC legacy", "orthogonal", "MS-DOS docs"),
            new Format("vax_f", "VAX F-float", 32L, 1L, 8L, 23L, 128L, -1.0, "u32", "HistoricalVendor", "Historical", "DEC VAX", "DEC legacy", "orthogonal", "VAX Architecture Reference"),
            new Format("vax_d", "VAX D-float", 64L, 1L, 8L, 55L, 128L, -1.0, "u64", "HistoricalVendor", "Historical", "DEC VAX", "DEC legacy double", "orthogonal", "VAX Architecture Reference"),
            new Format("vax_g", "VAX G-float", 64L, 1L, 11L, 52L, 1024L, -1.0, "u64", "HistoricalVendor", "Historical", "DEC VAX (IEEE-like)", "DEC legacy", "orthogonal", "VAX Architecture Reference"),
            new Format("vax_h", "VAX H-float", 128L, 1L, 15L, 112L, 16384L, -1.0, "u128", "HistoricalVendor", "Historical", "DEC VAX", "DEC quad", "orthogonal", "VAX Architecture Reference"),
            new Format("cray_float", "Cray float", 64L, 1L, 15L, 48L, 16384L, -1.0, "u64", "HistoricalVendor", "Historical", "Cray-1 (1976); no NaN/Inf, unrounded mul", "Cray legacy", "orthogonal", "Cray-1 Hardware Reference"),
            new Format("minifloat", "minifloat (arbitrary E:M, <=16 bits)", 0L, 1L, 0L, 0L, 0L, -1.0, "varies", "Theoretical", "Experimental", "parametric framework", "design space of GF4/GF8/GF12/GF16", "ally", "Higham 1996"),
            new Format("unum_i", "Unum I (tapered + ubound)", 0L, 1L, 0L, 0L, 0L, -1.0, "varies", "Theoretical", "Experimental", "Gustafson 2015 (predecessor to posit)", "interval arithmetic", "ally", "Gustafson 2015 (The End of Error)"),
            new Format("unum_ii", "Unum II (SORN projective)", 0L, 0L, 0L, 0L, 0L, -1.0, "lookup_table", "Theoretical", "Experimental", "Gustafson 2016", "lookup-table real arithmetic; not GF-comparable", "orthogonal", "Gustafson 2016"),
            new Format("tapered_fp", "tapered floating point", 0L, 1L, 0L, 0L, 0L, -1.0, "varies", "Theoretical", "Experimental", "Morris 1971; posit ancestor", "variable mantissa via regime bits", "ally", "Morris 1971 (IEEE TC)"),
            new Format("block_fp", "block floating point (BFP)", 0L, 0L, 0L, 0L, 0L, -1.0, "varies", "CompressionTrick", "Verified", "Wilkinson 1965; modern revivals", "per-tile shared exponent", "ally", "Darvish-Rouhani 2020"),
            new Format("shared_exp", "shared-exponent formats", 0L, 0L, 0L, 0L, 0L, -1.0, "varies", "CompressionTrick", "Verified", "generalised BFP", "LLM quantization", "ally", "Darvish-Rouhani 2020"),
            new Format("per_channel_scale", "INT8 with per-channel scale", 8L, 1L, 0L, 7L, 0L, -1.0, "u8_plus_fp32_scale", "CompressionTrick", "Verified", "Jacob 2018 (TFLite)", "standard quant inference", "competitor", "Jacob 2018 (CVPR)"),
            new Format("stochastic_rounding", "stochastic rounding (technique)", 0L, 0L, 0L, 0L, 0L, -1.0, "varies", "CompressionTrick", "Verified", "Gupta 2015", "training small networks at low precision", "ally", "Gupta 2015 (ICML)")
        };
        FORMATS = Collections.unmodifiableList(Arrays.asList(arr));
    }
}
