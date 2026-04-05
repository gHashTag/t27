// Auto-generated from specs/fpga/mac.t27
// DO NOT EDIT -- regenerate with: tri gen specs/fpga/mac.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 28 | Module: ZeroDSP_MAC

const std = @import("std");

// =====================================================================
// TernaryWord: 27 trits packed into a u32
// =====================================================================

pub const TernaryWord = struct {
    raw: u32,
};

pub const Trit = enum(i8) {
    neg = -1,
    zero = 0,
    pos = 1,
};

// =====================================================================
// 1. MAC Configuration
// =====================================================================

pub const MAC_WIDTH: usize = 27;
pub const MAC_ACC_BITS: usize = 32;
pub const NUM_MAC_UNITS: usize = 8;
pub const PIPELINE_STAGES: usize = 4;

pub const OP_MAC_MUL: u8 = 0;
pub const OP_MAC_MAC: u8 = 1;
pub const OP_MAC_MACC: u8 = 2;
pub const OP_MAC_DOT: u8 = 3;

pub const STATUS_READY: u8 = 0;
pub const STATUS_BUSY: u8 = 1;
pub const STATUS_DONE: u8 = 2;

// =====================================================================
// 2. Ternary Multiplication LUT
// =====================================================================

/// LUT for ternary multiplication: index = (a+1)*3 + (b+1)
/// where a,b in {-1, 0, +1}
pub const MAC_LUT: [9]i8 = .{
    1,   // (-1,-1) -> +1
    0,   // (-1, 0) ->  0
    -1,  // (-1,+1) -> -1
    0,   // ( 0,-1) ->  0
    0,   // ( 0, 0) ->  0
    0,   // ( 0,+1) ->  0
    -1,  // (+1,-1) -> -1
    0,   // (+1, 0) ->  0
    1,   // (+1,+1) -> +1
};

// =====================================================================
// 3. MAC Unit State
// =====================================================================

pub const MACUnit = struct {
    accumulator: i32,
    status: u8,
    pipeline: [PIPELINE_STAGES]TernaryWord,
};

pub const MACArray = struct {
    units: [NUM_MAC_UNITS]MACUnit,

    pub fn init() MACArray {
        var arr: MACArray = undefined;
        var i: usize = 0;
        while (i < NUM_MAC_UNITS) : (i += 1) {
            arr.units[i] = MACUnit{
                .accumulator = 0,
                .status = STATUS_READY,
                .pipeline = [_]TernaryWord{TernaryWord{ .raw = 0 }} ** PIPELINE_STAGES,
            };
        }
        return arr;
    }

    // =================================================================
    // 4. Trit Extraction
    // =================================================================

    /// Extract trit at given index from TernaryWord.
    /// Encoding: 0->zero, 1->pos, 2->neg.
    pub fn extract_trit(word: TernaryWord, index: usize) Trit {
        const bit_pos = index * 2;
        const encoded: u32 = (word.raw >> @intCast(bit_pos)) & 3;
        if (encoded == 2) return .neg;
        if (encoded == 1) return .pos;
        return .zero;
    }

    /// Pack a trit into a u32 at the given trit position.
    pub fn pack_trit(trit: Trit, index: usize) u32 {
        const bit_pos = index * 2;
        const encoded: u32 = switch (trit) {
            .neg => 2,
            .pos => 1,
            .zero => 0,
        };
        return encoded << @intCast(bit_pos);
    }

    // =================================================================
    // 5. MAC Operations
    // =================================================================

    /// Ternary multiplication using LUT.
    pub fn mac_multiply(self: *MACArray, a: TernaryWord, b: TernaryWord, unit: u8) TernaryWord {
        if (unit >= NUM_MAC_UNITS) {
            return TernaryWord{ .raw = 0 };
        }

        self.units[unit].status = STATUS_BUSY;
        var result: u32 = 0;
        var i: usize = 0;
        while (i < MAC_WIDTH) : (i += 1) {
            const a_trit = extract_trit(a, i);
            const b_trit = extract_trit(b, i);
            const a_idx: usize = @intCast(@as(i16, @intFromEnum(a_trit)) + 1);
            const b_idx: usize = @intCast(@as(i16, @intFromEnum(b_trit)) + 1);
            const lut_idx = a_idx * 3 + b_idx;
            const product = MAC_LUT[lut_idx];

            if (product == 1) {
                result = result | pack_trit(.pos, i);
            } else if (product == -1) {
                result = result | pack_trit(.neg, i);
            }
        }

        self.units[unit].status = STATUS_DONE;
        return TernaryWord{ .raw = result };
    }

    /// Single MAC cycle: acc = acc + dot(a, b).
    pub fn mac_cycle(self: *MACArray, a: TernaryWord, b: TernaryWord, unit: u8, acc: i32) i32 {
        if (unit >= NUM_MAC_UNITS) {
            return 0;
        }

        self.units[unit].status = STATUS_BUSY;
        var dot: i32 = 0;
        var i: usize = 0;
        while (i < MAC_WIDTH) : (i += 1) {
            const a_trit = extract_trit(a, i);
            const b_trit = extract_trit(b, i);
            const product = @as(i32, @intFromEnum(a_trit)) * @as(i32, @intFromEnum(b_trit));
            dot += product;
        }

        self.units[unit].accumulator = acc + dot;
        self.units[unit].status = STATUS_DONE;
        return self.units[unit].accumulator;
    }

    /// Full vector dot product using MAC unit.
    pub fn mac_dot_product(self: *MACArray, a: []const TernaryWord, b: []const TernaryWord, len: usize, unit: u8) i32 {
        if (unit >= NUM_MAC_UNITS) {
            return 0;
        }

        self.units[unit].status = STATUS_BUSY;
        self.units[unit].accumulator = 0;

        var i: usize = 0;
        while (i < len) : (i += 1) {
            self.units[unit].accumulator = self.mac_cycle(
                a[i],
                b[i],
                unit,
                self.units[unit].accumulator,
            );
        }

        self.units[unit].status = STATUS_DONE;
        return self.units[unit].accumulator;
    }

    /// Matrix-vector multiplication using MAC units.
    pub fn mac_matrix_vector(
        self: *MACArray,
        mat: []const TernaryWord,
        vec: []const TernaryWord,
        rows: usize,
        cols: usize,
        result: []i32,
        unit_assign: []const u8,
    ) void {
        var row: usize = 0;
        while (row < rows) : (row += 1) {
            const unit = unit_assign[row];
            self.units[unit].accumulator = 0;

            var col: usize = 0;
            while (col < cols) : (col += 1) {
                const mat_idx = row * cols + col;
                self.units[unit].accumulator = self.mac_cycle(
                    mat[mat_idx],
                    vec[col],
                    unit,
                    self.units[unit].accumulator,
                );
            }

            result[row] = self.units[unit].accumulator;
        }
    }

    // =================================================================
    // 6. MAC Unit Management
    // =================================================================

    pub fn mac_status_read(self: *const MACArray, unit: u8) u8 {
        if (unit >= NUM_MAC_UNITS) return 0xFF;
        return self.units[unit].status;
    }

    pub fn mac_status_write(self: *MACArray, unit: u8, status: u8) bool {
        if (unit >= NUM_MAC_UNITS) return false;
        self.units[unit].status = status;
        return true;
    }

    pub fn mac_reset(self: *MACArray, unit: u8) bool {
        if (unit >= NUM_MAC_UNITS) return false;
        self.units[unit].accumulator = 0;
        self.units[unit].status = STATUS_READY;
        self.units[unit].pipeline = [_]TernaryWord{TernaryWord{ .raw = 0 }} ** PIPELINE_STAGES;
        return true;
    }

    pub fn mac_reset_all(self: *MACArray) void {
        var i: usize = 0;
        while (i < NUM_MAC_UNITS) : (i += 1) {
            _ = self.mac_reset(@intCast(i));
        }
    }

    pub fn mac_get_accumulator(self: *const MACArray, unit: u8) i32 {
        if (unit >= NUM_MAC_UNITS) return 0;
        return self.units[unit].accumulator;
    }

    pub fn mac_set_accumulator(self: *MACArray, unit: u8, value: i32) bool {
        if (unit >= NUM_MAC_UNITS) return false;
        self.units[unit].accumulator = value;
        return true;
    }

    // =================================================================
    // 7. Parallel MAC Operations
    // =================================================================

    pub fn mac_parallel_multiply(
        self: *MACArray,
        a: []const TernaryWord,
        b: []const TernaryWord,
        results: []TernaryWord,
        count: usize,
    ) void {
        var i: usize = 0;
        while (i < count) : (i += 1) {
            const unit: u8 = @intCast(i % NUM_MAC_UNITS);
            results[i] = self.mac_multiply(a[i], b[i], unit);
        }
    }
};

// =====================================================================
// Tests
// =====================================================================

test "mac_lut_multiply_pos_pos" {
    var mac = MACArray.init();
    const set_trit = MACArray.pack_trit(.pos, 0);
    const a = TernaryWord{ .raw = set_trit };
    const b = TernaryWord{ .raw = set_trit };
    const result = mac.mac_multiply(a, b, 0);
    const result_trit = MACArray.extract_trit(result, 0);
    try std.testing.expectEqual(Trit.pos, result_trit);
}

test "mac_lut_multiply_neg_neg" {
    var mac = MACArray.init();
    const set_trit = MACArray.pack_trit(.neg, 0);
    const a = TernaryWord{ .raw = set_trit };
    const b = TernaryWord{ .raw = set_trit };
    const result = mac.mac_multiply(a, b, 0);
    const result_trit = MACArray.extract_trit(result, 0);
    try std.testing.expectEqual(Trit.pos, result_trit);
}

test "mac_lut_multiply_pos_neg" {
    var mac = MACArray.init();
    const a = TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) };
    const b = TernaryWord{ .raw = MACArray.pack_trit(.neg, 0) };
    const result = mac.mac_multiply(a, b, 0);
    const result_trit = MACArray.extract_trit(result, 0);
    try std.testing.expectEqual(Trit.neg, result_trit);
}

test "mac_lut_multiply_with_zero" {
    var mac = MACArray.init();
    const a = TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) };
    const b = TernaryWord{ .raw = MACArray.pack_trit(.zero, 0) };
    const result = mac.mac_multiply(a, b, 0);
    const result_trit = MACArray.extract_trit(result, 0);
    try std.testing.expectEqual(Trit.zero, result_trit);
}

test "mac_lut_size_9" {
    try std.testing.expectEqual(@as(usize, 9), MAC_LUT.len);
}

test "mac_num_units_8" {
    try std.testing.expectEqual(@as(usize, 8), NUM_MAC_UNITS);
}

test "mac_width_27" {
    try std.testing.expectEqual(@as(usize, 27), MAC_WIDTH);
}

test "mac_pipeline_stages_4" {
    try std.testing.expectEqual(@as(usize, 4), PIPELINE_STAGES);
}

test "mac_cycle_with_zero_accumulator" {
    var mac = MACArray.init();
    const a = TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) | MACArray.pack_trit(.pos, 1) };
    const b = TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) | MACArray.pack_trit(.pos, 1) };
    const result = mac.mac_cycle(a, b, 0, 0);
    try std.testing.expectEqual(@as(i32, 2), result);
}

test "mac_cycle_with_initial_accumulator" {
    var mac = MACArray.init();
    const a = TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) };
    const b = TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) };
    const result = mac.mac_cycle(a, b, 0, 5);
    try std.testing.expectEqual(@as(i32, 6), result);
}

test "mac_dot_product_simple" {
    var mac = MACArray.init();
    const a = [_]TernaryWord{
        TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) },
        TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) },
    };
    const b = [_]TernaryWord{
        TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) },
        TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) },
    };
    const result = mac.mac_dot_product(&a, &b, 2, 0);
    try std.testing.expectEqual(@as(i32, 2), result);
}

test "mac_dot_product_with_negatives" {
    var mac = MACArray.init();
    const a = [_]TernaryWord{
        TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) },
        TernaryWord{ .raw = MACArray.pack_trit(.neg, 0) },
    };
    const b = [_]TernaryWord{
        TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) },
        TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) },
    };
    const result = mac.mac_dot_product(&a, &b, 2, 0);
    try std.testing.expectEqual(@as(i32, 0), result);
}

test "mac_status_initially_ready" {
    const mac = MACArray.init();
    try std.testing.expectEqual(STATUS_READY, mac.mac_status_read(0));
}

test "mac_status_after_operation_is_done" {
    var mac = MACArray.init();
    const a = TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) };
    const b = TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) };
    _ = mac.mac_multiply(a, b, 0);
    try std.testing.expectEqual(STATUS_DONE, mac.mac_status_read(0));
}

test "mac_reset_clears_accumulator" {
    var mac = MACArray.init();
    const a = TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) };
    const b = TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) };
    _ = mac.mac_cycle(a, b, 0, 0);
    _ = mac.mac_reset(0);
    try std.testing.expectEqual(@as(i32, 0), mac.mac_get_accumulator(0));
}

test "mac_reset_clears_status" {
    var mac = MACArray.init();
    const a = TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) };
    const b = TernaryWord{ .raw = MACArray.pack_trit(.pos, 0) };
    _ = mac.mac_multiply(a, b, 0);
    _ = mac.mac_reset(0);
    try std.testing.expectEqual(STATUS_READY, mac.mac_status_read(0));
}

test "mac_invalid_unit_returns_zero" {
    var mac = MACArray.init();
    const result = mac.mac_multiply(TernaryWord{ .raw = 0 }, TernaryWord{ .raw = 0 }, 99);
    try std.testing.expectEqual(@as(u32, 0), result.raw);
}

test "mac_extract_trit_zero" {
    const word = TernaryWord{ .raw = 0 };
    try std.testing.expectEqual(Trit.zero, MACArray.extract_trit(word, 0));
}

test "mac_extract_trit_pos" {
    const word = TernaryWord{ .raw = 1 };
    try std.testing.expectEqual(Trit.pos, MACArray.extract_trit(word, 0));
}

test "mac_extract_trit_neg" {
    const word = TernaryWord{ .raw = 2 };
    try std.testing.expectEqual(Trit.neg, MACArray.extract_trit(word, 0));
}

test "mac_pack_trit_roundtrip" {
    const original = Trit.pos;
    const packed = MACArray.pack_trit(original, 0);
    const word = TernaryWord{ .raw = packed };
    const extracted = MACArray.extract_trit(word, 0);
    try std.testing.expectEqual(original, extracted);
}
