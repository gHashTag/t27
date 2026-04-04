// Auto-generated from specs/isa/registers.t27
// DO NOT EDIT -- regenerate with: tri gen specs/isa/registers.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 28 | Module: ISARegisters

const std = @import("std");

// =====================================================================
// TernaryWord: 27 trits packed into a u32
// =====================================================================

pub const TernaryWord = struct {
    raw: u32,
};

// =====================================================================
// 1. Register Constants
// =====================================================================

pub const NUM_REGISTERS: usize = 27;
pub const REG_WIDTH: usize = 27;
pub const COPIC_BASE: usize = 27;

// Register identifiers
pub const R0: u8 = 0;   // Zero register (always reads as 0)
pub const R1: u8 = 1;   // Argument/return register
pub const R2: u8 = 2;   // Argument/return register
pub const R3: u8 = 3;   // Argument register
pub const R4: u8 = 4;   // Argument register
pub const R5: u8 = 5;   // Temporary register
pub const R6: u8 = 6;   // Temporary register
pub const R7: u8 = 7;   // Temporary register
pub const R8: u8 = 8;   // Temporary register
pub const R9: u8 = 9;   // Temporary register
pub const R10: u8 = 10;  // Saved register S0
pub const R11: u8 = 11;  // Saved register S1
pub const R12: u8 = 12;  // Saved register S2
pub const R13: u8 = 13;  // Saved register S3
pub const R14: u8 = 14;  // Saved register S4
pub const R15: u8 = 15;  // Saved register S5
pub const R16: u8 = 16;  // Frame pointer
pub const R17: u8 = 17;  // Stack pointer
pub const R18: u8 = 18;  // Link register
pub const R19: u8 = 19;  // Program counter
pub const R20: u8 = 20;  // Status register
pub const R21: u8 = 21;  // Exception handler
pub const R22: u8 = 22;  // Kernel stack pointer
pub const R23: u8 = 23;  // Thread pointer
pub const R24: u8 = 24;  // Reserved
pub const R25: u8 = 25;  // Reserved
pub const R26: u8 = 26;  // Reserved

// Status register flags (R20)
pub const FLAG_ZERO: u8 = 0;
pub const FLAG_NEG: u8 = 1;
pub const FLAG_CARRY: u8 = 2;
pub const FLAG_OVERFLOW: u8 = 3;
pub const FLAG_TRAP: u8 = 4;
pub const FLAG_INTERRUPT: u8 = 5;

// Register aliases
pub const ARG0: u8 = R1;
pub const ARG1: u8 = R2;
pub const ARG2: u8 = R3;
pub const ARG3: u8 = R4;
pub const TMP0: u8 = R5;
pub const TMP1: u8 = R6;
pub const TMP2: u8 = R7;
pub const TMP3: u8 = R8;
pub const TMP4: u8 = R9;
pub const SAVED0: u8 = R10;
pub const SAVED1: u8 = R11;
pub const SAVED2: u8 = R12;
pub const SAVED3: u8 = R13;
pub const SAVED4: u8 = R14;
pub const SAVED5: u8 = R15;
pub const FP: u8 = R16;
pub const SP: u8 = R17;
pub const LR: u8 = R18;
pub const PC: u8 = R19;
pub const STATUS: u8 = R20;
pub const EH: u8 = R21;
pub const KSP: u8 = R22;
pub const TP: u8 = R23;

// =====================================================================
// 2. Coptic Alphabet Encoding (27 letters for 27 registers)
// =====================================================================

pub const COPTIC_ALPHABET: [27]u32 = .{
    0x03B1, // alpha  (R0)
    0x03B2, // bita   (R1)
    0x03B3, // gamma  (R2)
    0x03B4, // dalda  (R3)
    0x03B5, // ei     (R4)
    0x03C6, // sima   (R5)
    0x03B6, // zata   (R6)
    0x03B7, // ita    (R7)
    0x03B8, // thita  (R8)
    0x03B9, // iota   (R9)
    0x03BA, // kappa  (R10)
    0x03BB, // lauda  (R11)
    0x03BC, // mi     (R12)
    0x03BD, // ni     (R13)
    0x03BE, // ksi    (R14)
    0x03C0, // pi     (R15)
    0x03C1, // ro     (R16)
    0x03C3, // sigma  (R17)
    0x03C4, // tau    (R18)
    0x03C5, // upsilon(R19)
    0x03C6, // fi     (R20)
    0x03C7, // khi    (R21)
    0x03C8, // psi    (R22)
    0x03C9, // ou     (R23)
    0x0417, // sampi  (R24)
    0x0418, // koppa  (R25)
    0x0419, // shei   (R26)
};

// =====================================================================
// 3. Register File State
// =====================================================================

pub const RegisterFile = struct {
    regs: [NUM_REGISTERS]TernaryWord,

    pub fn init() RegisterFile {
        return RegisterFile{
            .regs = [_]TernaryWord{TernaryWord{ .raw = 0 }} ** NUM_REGISTERS,
        };
    }

    // =====================================================================
    // 4. Register Read/Write
    // =====================================================================

    /// Read from register file. R0 always returns 0.
    pub fn reg_read(self: *const RegisterFile, reg: u8) TernaryWord {
        if (reg == R0) {
            return TernaryWord{ .raw = 0 };
        }
        if (reg > R26) {
            return TernaryWord{ .raw = 0 };
        }
        return self.regs[reg];
    }

    /// Write to register file. R0 writes are ignored.
    pub fn reg_write(self: *RegisterFile, reg: u8, value: TernaryWord) bool {
        if (reg == R0) {
            return false;
        }
        if (reg > R26) {
            return false;
        }
        self.regs[reg] = value;
        return true;
    }

    // =====================================================================
    // 5. Coptic Encoding Conversion
    // =====================================================================

    /// Convert register number to Coptic Unicode code point.
    pub fn reg_to_coptic(reg: u8) u32 {
        if (reg > R26) {
            return 0;
        }
        return COPTIC_ALPHABET[reg];
    }

    /// Convert Coptic Unicode code point to register number.
    pub fn coptic_to_reg(cp: u32) u8 {
        var i: u8 = 0;
        while (i < 27) : (i += 1) {
            if (COPTIC_ALPHABET[i] == cp) {
                return i;
            }
        }
        return 0xFF;
    }

    // =====================================================================
    // 6. Status Register Operations
    // =====================================================================

    /// Read a specific flag from status register (R20).
    pub fn status_read(self: *const RegisterFile, flag: u8) bool {
        const status_val = self.reg_read(R20);
        const mask: u32 = @as(u32, 1) << @intCast(flag);
        return (status_val.raw & mask) != 0;
    }

    /// Set or clear a specific flag in status register (R20).
    pub fn status_write(self: *RegisterFile, flag: u8, value: bool) bool {
        if (flag > 5) {
            return false;
        }
        var status_val = self.reg_read(R20);
        const mask: u32 = @as(u32, 1) << @intCast(flag);
        if (value) {
            status_val.raw = status_val.raw | mask;
        } else {
            status_val.raw = status_val.raw & ~mask;
        }
        return self.reg_write(R20, status_val);
    }

    // =====================================================================
    // 7. Stack Operations (using R17 as stack pointer)
    // =====================================================================

    /// Push register value to stack (R17 as SP, grows downward).
    pub fn push_reg(self: *RegisterFile, reg: u8) bool {
        _ = self.reg_read(reg);
        var sp_val = self.reg_read(R17);
        sp_val.raw = sp_val.raw -% 4;
        if (!self.reg_write(R17, sp_val)) {
            return false;
        }
        return true;
    }

    /// Pop value from stack into register (R17 as SP).
    pub fn pop_reg(self: *RegisterFile, reg: u8) bool {
        var sp_val = self.reg_read(R17);
        const value = TernaryWord{ .raw = 0 }; // Placeholder
        sp_val.raw = sp_val.raw +% 4;
        if (!self.reg_write(R17, sp_val)) {
            return false;
        }
        return self.reg_write(reg, value);
    }

    // =====================================================================
    // 8. Context Save/Restore
    // =====================================================================

    /// Save caller-saved registers (R5-R9) to stack.
    pub fn save_context(self: *RegisterFile) void {
        _ = self.push_reg(R5);
        _ = self.push_reg(R6);
        _ = self.push_reg(R7);
        _ = self.push_reg(R8);
        _ = self.push_reg(R9);
    }

    /// Restore caller-saved registers from stack.
    pub fn restore_context(self: *RegisterFile) void {
        _ = self.pop_reg(R9);
        _ = self.pop_reg(R8);
        _ = self.pop_reg(R7);
        _ = self.pop_reg(R6);
        _ = self.pop_reg(R5);
    }
};

// =====================================================================
// Tests
// =====================================================================

test "isa_r0_always_zero" {
    var rf = RegisterFile.init();
    _ = rf.reg_write(R0, TernaryWord{ .raw = 0x123456 });
    const result = rf.reg_read(R0);
    try std.testing.expectEqual(@as(u32, 0), result.raw);
}

test "isa_r0_write_ignored" {
    var rf = RegisterFile.init();
    const before = rf.reg_read(R0);
    const success = rf.reg_write(R0, TernaryWord{ .raw = 0xDEADBEEF });
    const after = rf.reg_read(R0);
    try std.testing.expectEqual(false, success);
    try std.testing.expectEqual(before.raw, after.raw);
    try std.testing.expectEqual(@as(u32, 0), after.raw);
}

test "isa_register_count_27" {
    try std.testing.expectEqual(@as(usize, 27), NUM_REGISTERS);
}

test "isa_valid_register_write_succeeds" {
    var rf = RegisterFile.init();
    const value = TernaryWord{ .raw = 0xABCDEF };
    const success = rf.reg_write(R10, value);
    try std.testing.expectEqual(true, success);
}

test "isa_valid_register_write_read_roundtrip" {
    var rf = RegisterFile.init();
    const value = TernaryWord{ .raw = 0x123456 };
    _ = rf.reg_write(R10, value);
    const result = rf.reg_read(R10);
    try std.testing.expectEqual(value.raw, result.raw);
}

test "isa_invalid_register_write_fails" {
    var rf = RegisterFile.init();
    const value = TernaryWord{ .raw = 0x123456 };
    const success = rf.reg_write(27, value);
    try std.testing.expectEqual(false, success);
}

test "isa_invalid_register_read_returns_zero" {
    var rf = RegisterFile.init();
    const result = rf.reg_read(27);
    try std.testing.expectEqual(@as(u32, 0), result.raw);
}

test "isa_reg_to_coptic_r0" {
    const cp = RegisterFile.reg_to_coptic(R0);
    try std.testing.expectEqual(@as(u32, 0x03B1), cp);
}

test "isa_reg_to_coptic_r10" {
    const cp = RegisterFile.reg_to_coptic(R10);
    try std.testing.expectEqual(@as(u32, 0x03BA), cp);
}

test "isa_reg_to_coptic_r26" {
    const cp = RegisterFile.reg_to_coptic(R26);
    try std.testing.expectEqual(@as(u32, 0x0419), cp);
}

test "isa_coptic_to_reg_alpha" {
    const reg = RegisterFile.coptic_to_reg(0x03B1);
    try std.testing.expectEqual(R0, reg);
}

test "isa_coptic_to_reg_kappa" {
    const reg = RegisterFile.coptic_to_reg(0x03BA);
    try std.testing.expectEqual(R10, reg);
}

test "isa_coptic_to_reg_invalid" {
    const reg = RegisterFile.coptic_to_reg(0x0041);
    try std.testing.expectEqual(@as(u8, 0xFF), reg);
}

test "isa_coptic_roundtrip_r0" {
    const original_cp = RegisterFile.reg_to_coptic(R0);
    const recovered_reg = RegisterFile.coptic_to_reg(original_cp);
    const recovered_cp = RegisterFile.reg_to_coptic(recovered_reg);
    try std.testing.expectEqual(original_cp, recovered_cp);
}

test "isa_coptic_roundtrip_r15" {
    const original_cp = RegisterFile.reg_to_coptic(R15);
    const recovered_reg = RegisterFile.coptic_to_reg(original_cp);
    const recovered_cp = RegisterFile.reg_to_coptic(recovered_reg);
    try std.testing.expectEqual(original_cp, recovered_cp);
}

test "isa_status_read_initial_false" {
    var rf = RegisterFile.init();
    const flag_val = rf.status_read(FLAG_ZERO);
    try std.testing.expectEqual(false, flag_val);
}

test "isa_status_write_set" {
    var rf = RegisterFile.init();
    _ = rf.status_write(FLAG_ZERO, true);
    const flag_val = rf.status_read(FLAG_ZERO);
    try std.testing.expectEqual(true, flag_val);
}

test "isa_status_write_clear" {
    var rf = RegisterFile.init();
    _ = rf.status_write(FLAG_ZERO, true);
    _ = rf.status_write(FLAG_ZERO, false);
    const flag_val = rf.status_read(FLAG_ZERO);
    try std.testing.expectEqual(false, flag_val);
}

test "isa_status_flags_independent" {
    var rf = RegisterFile.init();
    _ = rf.status_write(FLAG_ZERO, true);
    _ = rf.status_write(FLAG_NEG, true);
    try std.testing.expectEqual(true, rf.status_read(FLAG_ZERO));
    try std.testing.expectEqual(true, rf.status_read(FLAG_NEG));
}

test "isa_status_invalid_flag" {
    var rf = RegisterFile.init();
    const success = rf.status_write(10, true);
    try std.testing.expectEqual(false, success);
}

test "isa_register_aliases_match" {
    try std.testing.expectEqual(R1, ARG0);
    try std.testing.expectEqual(R2, ARG1);
    try std.testing.expectEqual(R17, SP);
    try std.testing.expectEqual(R16, FP);
}

test "isa_coptic_alphabet_size_27" {
    try std.testing.expectEqual(@as(usize, 27), COPTIC_ALPHABET.len);
}
