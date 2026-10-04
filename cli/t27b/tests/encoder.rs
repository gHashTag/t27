//! Encoder words checked against the system assembler.
//!
//! Every expected word below was produced by assembling the instruction in the
//! comment with `clang -c` (Apple clang, arm64) and reading the result back
//! with `otool -t` / `objdump -d`. Branch offsets are in words, matching the
//! label distances of the assembled file.

use t27b::a64::*;

#[test]
fn encoders_match_clang() {
    let cases: &[(&str, u32, u32)] = &[
        ("add w2, w0, w1", add(false, 2, 0, 1), 0x0b01_0002),
        ("add x2, x0, x1", add(true, 2, 0, 1), 0x8b01_0002),
        ("adds w2, w0, w1", adds(false, 2, 0, 1), 0x2b01_0002),
        ("adds x3, x4, x5", adds(true, 3, 4, 5), 0xab05_0083),
        ("sub w2, w0, w1", sub(false, 2, 0, 1), 0x4b01_0002),
        ("subs x2, x0, x1", subs(true, 2, 0, 1), 0xeb01_0002),
        ("cmp w1, w2", cmp(false, 1, 2), 0x6b02_003f),
        ("cmp x8, x9, asr #63", cmp_shifted(true, 8, 9, Shift::Asr, 63), 0xeb89_fd1f),
        ("negs w0, w2", subs(false, 0, ZR, 2), 0x6b02_03e0),
        ("sub sp, sp, x16, uxtx", addsub_ext(true, true, false, SP, SP, 16, Ext::Uxtx, 0), 0xcb30_63ff),
        ("cmp w9, w9, uxtb", cmp_ext(false, 9, 9, Ext::Uxtb), 0x6b29_013f),
        ("cmp w9, w9, sxth", cmp_ext(false, 9, 9, Ext::Sxth), 0x6b29_a13f),
        ("cmp x9, w9, uxtw", cmp_ext(true, 9, 9, Ext::Uxtw), 0xeb29_413f),
        ("cmp x9, w9, sxtw", cmp_ext(true, 9, 9, Ext::Sxtw), 0xeb29_c13f),
        ("add w2, w0, #1", add_imm(false, 2, 0, 1), 0x1100_0402),
        ("adds w2, w0, #0", addsub_imm(false, false, true, 2, 0, 0, false), 0x3100_0002),
        ("sub sp, sp, #32", sub_imm(true, SP, SP, 32), 0xd100_83ff),
        ("subs x1, x2, #4095", addsub_imm(true, true, true, 1, 2, 4095, false), 0xf13f_fc41),
        ("cmp w1, #8", cmp_imm(false, 1, 8), 0x7100_203f),
        ("cmn x3, #1", cmn_imm(true, 3, 1), 0xb100_047f),
        ("mov x29, sp", mov_sp(29, SP), 0x9100_03fd),
        ("mov sp, x29", mov_sp(SP, 29), 0x9100_03bf),
        ("and w0, w1, w2", logic_reg(false, LogOp::And, false, 0, 1, 2), 0x0a02_0020),
        ("orr x0, x1, x2", logic_reg(true, LogOp::Orr, false, 0, 1, 2), 0xaa02_0020),
        ("eor w2, w9, w10", logic_reg(false, LogOp::Eor, false, 2, 9, 10), 0x4a0a_0122),
        ("ands x0, x1, x2", logic_reg(true, LogOp::Ands, false, 0, 1, 2), 0xea02_0020),
        ("mov x0, x2", mov(true, 0, 2), 0xaa02_03e0),
        ("mov w0, w2", mov(false, 0, 2), 0x2a02_03e0),
        ("mvn w0, w2", mvn(false, 0, 2), 0x2a22_03e0),
        ("mvn x0, x2", mvn(true, 0, 2), 0xaa22_03e0),
        ("and w8, w1, #7", logic_imm(false, LogOp::And, 8, 1, 7).unwrap(), 0x1200_0828),
        ("eor w0, w1, #0xff", logic_imm(false, LogOp::Eor, 0, 1, 0xff).unwrap(), 0x5200_1c20),
        ("eor w0, w1, #0xffff", logic_imm(false, LogOp::Eor, 0, 1, 0xffff).unwrap(), 0x5200_3c20),
        ("eor w0, w1, #1", logic_imm(false, LogOp::Eor, 0, 1, 1).unwrap(), 0x5200_0020),
        ("and x0, x1, #0xffffffff00000000", logic_imm(true, LogOp::And, 0, 1, 0xffff_ffff_0000_0000).unwrap(), 0x9260_7c20),
        ("orr x0, xzr, #0x5555555555555555", logic_imm(true, LogOp::Orr, 0, ZR, 0x5555_5555_5555_5555).unwrap(), 0xb200_f3e0),
        ("and w0, w1, #0xfffffff0", logic_imm(false, LogOp::And, 0, 1, 0xffff_fff0).unwrap(), 0x121c_6c20),
        ("movz w1, #0x7", movz(false, 1, 7, 0), 0x5280_00e1),
        ("movz x1, #0x1234, lsl #16", movz(true, 1, 0x1234, 1), 0xd2a2_4681),
        ("movn x1, #0", movn(true, 1, 0, 0), 0x9280_0001),
        ("movn w1, #0", movn(false, 1, 0, 0), 0x1280_0001),
        ("movk x9, #0xbeef, lsl #48", movk(true, 9, 0xbeef, 3), 0xf2f7_dde9),
        ("movk w9, #0xbeef, lsl #16", movk(false, 9, 0xbeef, 1), 0x72b7_dde9),
        ("udiv w0, w1, w2", udiv(false, 0, 1, 2), 0x1ac2_0820),
        ("sdiv x0, x1, x2", sdiv(true, 0, 1, 2), 0x9ac2_0c20),
        ("lsl w0, w1, w2", lslv(false, 0, 1, 2), 0x1ac2_2020),
        ("lsr x0, x1, x2", lsrv(true, 0, 1, 2), 0x9ac2_2420),
        ("asr w0, w1, w2", asrv(false, 0, 1, 2), 0x1ac2_2820),
        ("madd x0, x1, x2, x3", madd(true, 0, 1, 2, 3), 0x9b02_0c20),
        ("msub w2, w8, w17, w2", msub(false, 2, 8, 17, 2), 0x1b11_8902),
        ("mul w9, w2, w17", mul(false, 9, 2, 17), 0x1b11_7c49),
        ("mul x9, x2, x17", mul(true, 9, 2, 17), 0x9b11_7c49),
        ("smull x9, w2, w17", smull(9, 2, 17), 0x9b31_7c49),
        ("umull x9, w2, w17", umull(9, 2, 17), 0x9bb1_7c49),
        ("smulh x8, x1, x2", smulh(8, 1, 2), 0x9b42_7c28),
        ("umulh x8, x1, x2", umulh(8, 1, 2), 0x9bc2_7c28),
        ("lsl w0, w2, #2", lsl_imm(false, 0, 2, 2), 0x531e_7440),
        ("lsr w10, w2, #2", lsr_imm(false, 10, 2, 2), 0x5302_7c4a),
        ("asr x0, x1, #63", asr_imm(true, 0, 1, 63), 0x937f_fc20),
        ("lsl x0, x1, #1", lsl_imm(true, 0, 1, 1), 0xd37f_f820),
        ("asr w3, w4, #31", asr_imm(false, 3, 4, 31), 0x131f_7c83),
        ("lsr x5, x6, #0", lsr_imm(true, 5, 6, 0), 0xd340_fcc5),
        ("uxtb w0, w1", uxtb(0, 1), 0x5300_1c20),
        ("uxth w0, w1", uxth(0, 1), 0x5300_3c20),
        ("sxtb w0, w1", sxtb(0, 1), 0x1300_1c20),
        ("sxth w0, w1", sxth(0, 1), 0x1300_3c20),
        ("sxtw x0, w1", sxtw(0, 1), 0x9340_7c20),
        ("cset w9, lo", cset(false, 9, Cond::Lo), 0x1a9f_27e9),
        ("cset w0, eq", cset(false, 0, Cond::Eq), 0x1a9f_17e0),
        ("cset x0, gt", cset(true, 0, Cond::Gt), 0x9a9f_d7e0),
        ("csinc w0, w1, w2, ne", csinc(false, 0, 1, 2, Cond::Ne), 0x1a82_1420),
        ("b L1", b(2), 0x1400_0002),
        ("nop", nop(), 0xd503_201f),
        ("b L0", b(-2), 0x17ff_fffe),
        ("bl L0", bl(-3), 0x97ff_fffd),
        ("b.eq L1", b_cond(Cond::Eq, -2), 0x54ff_ffc0),
        ("b.vs L2", b_cond(Cond::Vs, 4), 0x5400_0086),
        ("b.lt L0", b_cond(Cond::Lt, -6), 0x54ff_ff4b),
        ("cbz w1, L2", cbz(false, 1, 2), 0x3400_0041),
        ("cbnz x8, L0", cbnz(true, 8, -8), 0xb5ff_ff08),
        ("ret", ret(), 0xd65f_03c0),
        ("blr x16", blr(16), 0xd63f_0200),
        ("brk #0x1", brk(1), 0xd420_0020),
        ("brk #0x5", brk(5), 0xd420_00a0),
        ("nop", nop(), 0xd503_201f),
        ("str x19, [sp, #16]", str_x(19, SP, 16), 0xf900_0bf3),
        ("ldr x10, [x9]", ldr_x(10, 9, 0), 0xf940_012a),
        ("ldr x16, [sp, #32760]", ldr_x(16, SP, 32760), 0xf97f_fff0),
        ("stur x1, [x2, #-8]", stur_x(1, 2, -8), 0xf81f_8041),
        ("ldur x1, [x2, #-8]", ldur_x(1, 2, -8), 0xf85f_8041),
        ("stp x19, x20, [sp, #16]", stp_x(19, 20, SP, 16), 0xa901_53f3),
        ("ldp x0, x1, [x17, #48]", ldp_x(0, 1, 17, 48), 0xa943_0620),
        ("stp x27, x28, [sp, #80]", stp_x(27, 28, SP, 80), 0xa905_73fb),
        ("stp x29, x30, [sp, #-16]!", stp_x_pre(29, 30, SP, -16), 0xa9bf_7bfd),
        ("ldp x29, x30, [sp], #16", ldp_x_post(29, 30, SP, 16), 0xa8c1_7bfd),
        ("stp x29, x30, [sp, #-96]!", stp_x_pre(29, 30, SP, -96), 0xa9ba_7bfd),
        ("ldp x29, x30, [sp], #96", ldp_x_post(29, 30, SP, 96), 0xa8c6_7bfd),
        ("strb w1, [x2, #3]", ldst_uimm(0, ST, 1, 2, 3), 0x3900_0c41),
        ("ldrb w1, [x2, #4095]", ldst_uimm(0, LD, 1, 2, 4095), 0x397f_fc41),
        ("ldrsb w3, [x4, #1]", ldst_uimm(0, LDS32, 3, 4, 1), 0x39c0_0483),
        ("ldrsb x3, [x4, #1]", ldst_uimm(0, LDS64, 3, 4, 1), 0x3980_0483),
        ("strh w5, [x6, #2]", ldst_uimm(1, ST, 5, 6, 2), 0x7900_04c5),
        ("ldrh w5, [x6, #8190]", ldst_uimm(1, LD, 5, 6, 8190), 0x797f_fcc5),
        ("ldrsh w7, [x8, #4]", ldst_uimm(1, LDS32, 7, 8, 4), 0x79c0_0907),
        ("str w9, [x10, #16380]", ldst_uimm(2, ST, 9, 10, 16380), 0xb93f_fd49),
        ("ldr w9, [sp, #4]", ldst_uimm(2, LD, 9, SP, 4), 0xb940_07e9),
        ("ldrsw x9, [x10, #8]", ldst_uimm(2, LDS64, 9, 10, 8), 0xb980_0949),
        ("str x11, [x12, #8]", ldst_uimm(3, ST, 11, 12, 8), 0xf900_058b),
        ("ldr x11, [x29, #32760]", ldst_uimm(3, LD, 11, 29, 32760), 0xf97f_ffab),
        ("strb wzr, [x1]", ldst_uimm(0, ST, ZR, 1, 0), 0x3900_003f),
        ("ldurb w1, [x29, #-1]", ldst_unscaled(0, LD, 1, 29, -1), 0x385f_f3a1),
        ("sturh w2, [x29, #-256]", ldst_unscaled(1, ST, 2, 29, -256), 0x7810_03a2),
        ("ldur w3, [x29, #255]", ldst_unscaled(2, LD, 3, 29, 255), 0xb84f_f3a3),
        ("stur x4, [x29, #-8]", ldst_unscaled(3, ST, 4, 29, -8), 0xf81f_83a4),
        ("ldursb w5, [x29, #-3]", ldst_unscaled(0, LDS32, 5, 29, -3), 0x38df_d3a5),
        ("ldursh w5, [x29, #-4]", ldst_unscaled(1, LDS32, 5, 29, -4), 0x78df_c3a5),
        ("ldrb w1, [x2, x3]", ldst_reg(0, LD, 1, 2, 3, false), 0x3863_6841),
        ("ldrsh w1, [x2, x3, lsl #1]", ldst_reg(1, LDS32, 1, 2, 3, true), 0x78e3_7841),
        ("str w4, [x5, x6, lsl #2]", ldst_reg(2, ST, 4, 5, 6, true), 0xb826_78a4),
        ("ldr x7, [x8, x9]", ldst_reg(3, LD, 7, 8, 9, false), 0xf869_6907),
        ("ldr x7, [x8, x9, lsl #3]", ldst_reg(3, LD, 7, 8, 9, true), 0xf869_7907),
        ("strb w8, [x16], #1", ldst_post(0, ST, 8, 16, 1), 0x3800_1608),
        ("ldrb w30, [x17], #1", ldst_post(0, LD, 30, 17, 1), 0x3840_163e),
        ("ldr x8, [x17], #8", ldst_post(3, LD, 8, 17, 8), 0xf840_8628),
        ("str x8, [x16], #8", ldst_post(3, ST, 8, 16, 8), 0xf800_8608),
        ("adr x3, L0 (-112)", adr(3, -112), 0x10ff_fc83),
        ("adr x4, L1 (+12)", adr(4, 12), 0x1000_0064),
        ("adrp x5, #8192", adrp(5, 2), 0xd000_0005),
        ("adrp x6, #-4096", adrp(6, -1), 0xf0ff_ffe6),
    ];
    let mut bad = Vec::new();
    for (asm, got, want) in cases {
        if got != want {
            bad.push(format!("{:<36} got {:08x} want {:08x}", asm, got, want));
        }
    }
    assert!(bad.is_empty(), "encoder mismatches:\n{}", bad.join("\n"));
    assert_eq!(cases.len(), 129);
}

#[test]
fn disassembler_matches_otool_for_memory_forms() {
    // Text from `otool -tv` of the same assembled file, immediates in decimal.
    for (w, pc, want) in [
        (0x39c0_0483u32, 0usize, "ldrsb w3, [x4, #1]"),
        (0x3980_0483, 0, "ldrsb x3, [x4, #1]"),
        (0xb980_0949, 0, "ldrsw x9, [x10, #8]"),
        (0x3900_003f, 0, "strb wzr, [x1, #0]"),
        (0x7810_03a2, 0, "sturh w2, [x29, #-256]"),
        (0x38df_d3a5, 0, "ldursb w5, [x29, #-3]"),
        (0x78e3_7841, 0, "ldrsh w1, [x2, x3, lsl #1]"),
        (0xf869_6907, 0, "ldr x7, [x8, x9]"),
        (0x3840_163e, 0, "ldrb w30, [x17], #1"),
        (0x10ff_fc83, 0x70, "adr x3, 0x0"),
        (0xd000_0005, 0x78, "adrp x5, 0x2000"),
    ] {
        assert_eq!(disasm(w, pc), want, "{:08x}", w);
    }
    assert_eq!(adrp_patch(adrp(5, 0), 2), 0xd000_0005);
    assert_eq!(add_imm_patch(add_imm(true, 5, 5, 0), 0x123), add_imm(true, 5, 5, 0x123));
}

#[test]
fn disassembler_reads_back_the_subset() {
    // Every encoder output must disassemble to something other than `.word`.
    for w in [
        add(false, 2, 0, 1),
        cmp_ext(true, 9, 9, Ext::Sxtw),
        logic_imm(true, LogOp::And, 0, 1, 0xffff_ffff_0000_0000).unwrap(),
        movk(true, 9, 0xbeef, 3),
        smulh(8, 1, 2),
        cset(false, 9, Cond::Lo),
        cbnz(true, 8, -8),
        ldp_x_post(29, 30, SP, 96),
        ldst_uimm(0, LDS32, 3, 4, 1),
        ldst_unscaled(1, ST, 2, 29, -256),
        ldst_reg(1, LDS32, 1, 2, 3, true),
        ldst_post(3, LD, 8, 17, 8),
        adr(3, -112),
        adrp(5, 2),
    ] {
        let d = disasm(w, 0);
        assert!(!d.starts_with(".word"), "{:08x} -> {}", w, d);
    }
}

/// Tiny emulator for the instructions mov_imm may produce.
fn run_mov(words: &[u32], sf: bool) -> u64 {
    let mut x: u64 = 0xdead_beef_dead_beef;
    for &w in words {
        let rd = w & 31;
        assert_eq!(rd, 5, "mov_imm wrote the wrong register: {:08x}", w);
        let hw = (w >> 21) & 3;
        let imm = ((w >> 5) & 0xffff) as u64;
        match (w >> 23) & 0x1ff {
            0x0a5 | 0x1a5 => x = imm << (16 * hw),                       // movz
            0x025 | 0x125 => x = !(imm << (16 * hw)),                    // movn
            0x0e5 | 0x1e5 => x = (x & !(0xffff << (16 * hw))) | (imm << (16 * hw)), // movk
            _ if (w >> 23) & 0xff == 0x64 => {
                // orr rd, zr, #bitmask
                let n = (w >> 22) & 1;
                let immr = (w >> 16) & 0x3f;
                let imms = (w >> 10) & 0x3f;
                assert_eq!((w >> 5) & 31, 31, "orr source must be zr");
                x = decode_bitmask(n, immr, imms, sf);
            }
            _ => panic!("unexpected word {:08x}", w),
        }
        if !sf {
            x &= 0xffff_ffff;
        }
    }
    x
}

#[test]
fn mov_imm_materialises_every_value() {
    let mut vals: Vec<u64> = vec![
        0, 1, 7, 0xffff, 0x1_0000, 0x7fff_ffff, 0x8000_0000, 0xffff_ffff, 0x1_0000_0000,
        0x5555_5555_5555_5555, 0x8000_0000_0000_0000, 0x7fff_ffff_ffff_ffff, u64::MAX,
        u64::MAX - 1, 0xffff_ffff_0000_0000, 0x0000_ffff_ffff_0000, 0x1234_5678_9abc_def0,
        0xffff_1234_ffff_ffff, 0xfffe_0000_0000_0000,
    ];
    let mut s: u64 = 0x2545_f491_4f6c_dd1d;
    for _ in 0..20000 {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        vals.push(s);
        vals.push(s & 0xffff_0000_ffff);
        vals.push(!(s & 0xffff));
        vals.push(s >> (s % 64));
    }
    for &v in &vals {
        for sf in [true, false] {
            let want = if sf { v } else { v & 0xffff_ffff };
            let mut out = Vec::new();
            mov_imm(sf, 5, want, &mut out);
            assert!(!out.is_empty() && out.len() <= 4, "{:#x}: {} words", want, out.len());
            assert_eq!(run_mov(&out, sf), want, "mov_imm({}, {:#x}) -> {:08x?}", sf, want, out);
        }
    }
}

#[test]
fn bitmask_immediates_are_complete_and_exact() {
    // Enumerate every logical immediate: element size e, run of s+1 ones,
    // rotation r. 5334 distinct 64-bit values, 1302 32-bit ones.
    for sf in [true, false] {
        let width = if sf { 64 } else { 32 };
        let mut seen = std::collections::HashSet::new();
        let mut e = 2;
        while e <= width {
            for ones in 1..e {
                for rot in 0..e {
                    let elem: u64 = ((1u128 << ones) - 1) as u64;
                    let elem = if rot == 0 {
                        elem
                    } else {
                        ((elem >> rot) | (elem << (e - rot))) & (((1u128 << e) - 1) as u64)
                    };
                    let mut v: u64 = 0;
                    let mut k = 0;
                    while k < width {
                        v |= elem << k;
                        k += e;
                    }
                    if !seen.insert(v) {
                        continue;
                    }
                    let (n, immr, imms) =
                        bitmask_imm(v, sf).unwrap_or_else(|| panic!("{:#x} not encodable (sf={})", v, sf));
                    assert_eq!(decode_bitmask(n, immr, imms, sf), v, "round trip of {:#x}", v);
                }
            }
            e *= 2;
        }
        assert_eq!(seen.len(), if sf { 5334 } else { 1302 });
        for v in [0u64, if sf { u64::MAX } else { 0xffff_ffff }, 0x1234_5678] {
            assert!(bitmask_imm(v, sf).is_none(), "{:#x} must not be encodable", v);
        }
    }
}
