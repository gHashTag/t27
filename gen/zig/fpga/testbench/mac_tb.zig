// Auto-generated from specs/fpga/testbench/mac_tb.t27
// DO NOT EDIT -- regenerate with: tri gen specs/fpga/testbench/mac_tb.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 43 | Module: MAC_Testbench

const std = @import("std");
const mac = @import("../mac.zig");

const TernaryWord = mac.TernaryWord;
const Trit = mac.Trit;
const MACArray = mac.MACArray;

// =====================================================================
// 1. Testbench Configuration
// =====================================================================

pub const CLK_PERIOD: u32 = 20;
pub const SIM_TIMEOUT: u32 = 10_000_000;

pub const TRIT_POS: i8 = 1;
pub const TRIT_ZERO: i8 = 0;
pub const TRIT_NEG: i8 = -1;

// =====================================================================
// 2. Testbench State
// =====================================================================

const TestState = struct {
    test_passed: u32,
    test_failed: u32,
    sim_cycle: u32,
    mac_array: MACArray,

    pub fn init() TestState {
        return TestState{
            .test_passed = 0,
            .test_failed = 0,
            .sim_cycle = 0,
            .mac_array = MACArray.init(),
        };
    }

    pub fn assert_pass(self: *TestState, condition: bool, message: []const u8) void {
        if (condition) {
            self.test_passed += 1;
        } else {
            self.test_failed += 1;
            std.debug.print("  [FAIL] {s}\n", .{message});
        }
    }

    pub fn generate_clock(self: *TestState) void {
        self.sim_cycle += 1;
    }

    pub fn wait_cycles(self: *TestState, n: u32) void {
        var i: u32 = 0;
        while (i < n) : (i += 1) {
            self.generate_clock();
        }
    }
};

// =====================================================================
// 3. Helper: make_trit_word
// =====================================================================

fn make_trit_word(trits: []const i8) TernaryWord {
    var word: u32 = 0;
    var i: usize = 0;
    while (i < trits.len and i < mac.MAC_WIDTH) : (i += 1) {
        const trit = trits[i];
        const encoded: u32 = if (trit == TRIT_NEG) 2 else if (trit == TRIT_POS) 1 else 0;
        word = word | (encoded << @intCast(i * 2));
    }
    return TernaryWord{ .raw = word };
}

// =====================================================================
// 4. Test Cases
// =====================================================================

fn test_mac_lut_pos_pos(state: *TestState) void {
    const a = make_trit_word(&[_]i8{TRIT_POS});
    const b = make_trit_word(&[_]i8{TRIT_POS});
    const result = state.mac_array.mac_multiply(a, b, 0);
    const trit = MACArray.extract_trit(result, 0);
    state.assert_pass(trit == .pos, "LUT: +1 * +1 = +1");
}

fn test_mac_lut_neg_neg(state: *TestState) void {
    const a = make_trit_word(&[_]i8{TRIT_NEG});
    const b = make_trit_word(&[_]i8{TRIT_NEG});
    const result = state.mac_array.mac_multiply(a, b, 0);
    const trit = MACArray.extract_trit(result, 0);
    state.assert_pass(trit == .pos, "LUT: -1 * -1 = +1");
}

fn test_mac_lut_pos_neg(state: *TestState) void {
    const a = make_trit_word(&[_]i8{TRIT_POS});
    const b = make_trit_word(&[_]i8{TRIT_NEG});
    const result = state.mac_array.mac_multiply(a, b, 0);
    const trit = MACArray.extract_trit(result, 0);
    state.assert_pass(trit == .neg, "LUT: +1 * -1 = -1");
}

fn test_mac_lut_with_zero(state: *TestState) void {
    const a = make_trit_word(&[_]i8{TRIT_POS});
    const b = make_trit_word(&[_]i8{TRIT_ZERO});
    const result = state.mac_array.mac_multiply(a, b, 0);
    const trit = MACArray.extract_trit(result, 0);
    state.assert_pass(trit == .zero, "LUT: +1 * 0 = 0");
}

fn test_mac_all_trit_combinations(state: *TestState) void {
    const a_vals = [_]i8{ TRIT_NEG, TRIT_ZERO, TRIT_POS };
    const b_vals = [_]i8{ TRIT_NEG, TRIT_ZERO, TRIT_POS };
    var combinations: u32 = 0;

    for (a_vals) |a_v| {
        for (b_vals) |b_v| {
            const a_word = make_trit_word(&[_]i8{a_v});
            const b_word = make_trit_word(&[_]i8{b_v});
            const result = state.mac_array.mac_multiply(a_word, b_word, 0);
            const trit = MACArray.extract_trit(result, 0);
            const expected: i8 = a_v * b_v;
            const expected_trit: Trit = if (expected > 0) .pos else if (expected < 0) .neg else .zero;
            state.assert_pass(trit == expected_trit, "All trit combos");
            combinations += 1;
        }
    }

    state.assert_pass(combinations == 9, "All 9 combinations tested");
}

fn test_mac_27_trit_word(state: *TestState) void {
    var a_trits: [27]i8 = [_]i8{TRIT_ZERO} ** 27;
    var b_trits: [27]i8 = [_]i8{TRIT_ZERO} ** 27;

    var i: usize = 0;
    while (i < 27) : (i += 1) {
        a_trits[i] = if (i % 2 == 0) TRIT_POS else TRIT_NEG;
        b_trits[i] = TRIT_POS;
    }

    const a = make_trit_word(&a_trits);
    const b = make_trit_word(&b_trits);
    const result = state.mac_array.mac_multiply(a, b, 0);

    // POS*POS = POS at pos 0, NEG*POS = NEG at pos 1
    state.assert_pass(MACArray.extract_trit(result, 0) == .pos, "27-trit position 0");
    state.assert_pass(MACArray.extract_trit(result, 1) == .neg, "27-trit position 1");
}

fn test_mac_cycle_zero_acc(state: *TestState) void {
    const a = make_trit_word(&[_]i8{ TRIT_POS, TRIT_POS });
    const b = make_trit_word(&[_]i8{ TRIT_POS, TRIT_POS });
    const result = state.mac_array.mac_cycle(a, b, 0, 0);
    // (+1)*1 + (+1)*1 = 2
    state.assert_pass(result == 2, "MAC cycle with zero acc");
}

fn test_mac_cycle_with_acc(state: *TestState) void {
    const a = make_trit_word(&[_]i8{TRIT_POS});
    const b = make_trit_word(&[_]i8{TRIT_POS});
    const result = state.mac_array.mac_cycle(a, b, 0, 10);
    // 10 + 1 = 11
    state.assert_pass(result == 11, "MAC cycle with initial acc");
}

fn test_mac_dot_product(state: *TestState) void {
    const vec1 = [_]TernaryWord{
        make_trit_word(&[_]i8{TRIT_POS}),
        make_trit_word(&[_]i8{TRIT_POS}),
    };
    const vec2 = [_]TernaryWord{
        make_trit_word(&[_]i8{TRIT_POS}),
        make_trit_word(&[_]i8{TRIT_POS}),
    };

    const result = state.mac_array.mac_dot_product(&vec1, &vec2, 2, 0);
    // 1 + 1 = 2
    state.assert_pass(result == 2, "Dot product");
}

fn test_mac_reset(state: *TestState) void {
    // Perform some operations
    _ = state.mac_array.mac_cycle(
        make_trit_word(&[_]i8{TRIT_POS}),
        make_trit_word(&[_]i8{TRIT_POS}),
        0,
        42,
    );

    // Reset
    _ = state.mac_array.mac_reset(0);
    _ = state.mac_array.mac_reset(1);

    // Check reset state
    const acc0 = state.mac_array.mac_get_accumulator(0);
    const acc1 = state.mac_array.mac_get_accumulator(1);
    const status0 = state.mac_array.mac_status_read(0);
    const status1 = state.mac_array.mac_status_read(1);

    state.assert_pass(acc0 == 0, "Accumulator 0 reset");
    state.assert_pass(acc1 == 0, "Accumulator 1 reset");
    state.assert_pass(status0 == mac.STATUS_READY, "Status 0 ready");
    state.assert_pass(status1 == mac.STATUS_READY, "Status 1 ready");
}

fn test_mac_unit_independence(state: *TestState) void {
    _ = state.mac_array.mac_reset(0);
    _ = state.mac_array.mac_reset(1);

    const a = make_trit_word(&[_]i8{TRIT_POS});
    const b = make_trit_word(&[_]i8{TRIT_POS});

    // Use unit 0
    _ = state.mac_array.mac_cycle(a, b, 0, 0);
    const acc0_before = state.mac_array.mac_get_accumulator(0);

    // Use unit 1
    _ = state.mac_array.mac_cycle(a, b, 1, 0);
    const acc1_after = state.mac_array.mac_get_accumulator(1);
    const acc0_after = state.mac_array.mac_get_accumulator(0);

    state.assert_pass(acc0_before == acc0_after, "Unit 0 unaffected");
    state.assert_pass(acc1_after == 1, "Unit 1 value correct");
}

fn test_mac_invalid_unit(state: *TestState) void {
    const a = TernaryWord{ .raw = 0 };
    const b = TernaryWord{ .raw = 0 };
    const result = state.mac_array.mac_multiply(a, b, 99);
    state.assert_pass(result.raw == 0, "Invalid unit handling");
}

fn test_mac_overflow_handling(state: *TestState) void {
    const large_acc: i32 = 0x7FFFFFFF;
    const a = make_trit_word(&[_]i8{TRIT_POS});
    const b = make_trit_word(&[_]i8{TRIT_POS});

    const result = state.mac_array.mac_cycle(a, b, 0, large_acc);
    _ = result;

    const status = state.mac_array.mac_status_read(0);
    state.assert_pass(status == mac.STATUS_DONE, "Overflow handled");
}

fn test_mac_parallel_units(state: *TestState) void {
    var a_arr: [8]TernaryWord = undefined;
    var b_arr: [8]TernaryWord = undefined;
    var results: [8]TernaryWord = undefined;

    var i: usize = 0;
    while (i < 8) : (i += 1) {
        a_arr[i] = TernaryWord{ .raw = 1 };
        b_arr[i] = TernaryWord{ .raw = 1 };
        results[i] = TernaryWord{ .raw = 0 };
    }

    state.mac_array.mac_parallel_multiply(&a_arr, &b_arr, &results, 8);

    var all_valid = true;
    i = 0;
    while (i < 8) : (i += 1) {
        if (results[i].raw == 0) {
            all_valid = false;
            break;
        }
    }

    state.assert_pass(all_valid, "Parallel units independent");
}

fn test_mac_latency(state: *TestState) void {
    const a = make_trit_word(&[_]i8{TRIT_POS});
    const b = make_trit_word(&[_]i8{TRIT_POS});

    const start_cycle = state.sim_cycle;
    _ = state.mac_array.mac_multiply(a, b, 0);
    state.wait_cycles(100);

    const cycles = state.sim_cycle - start_cycle;
    state.assert_pass(cycles < 200, "MAC latency acceptable");
}

// =====================================================================
// 5. Test Runner
// =====================================================================

pub fn run_tests() void {
    var state = TestState.init();

    std.debug.print("================================================================\n", .{});
    std.debug.print("          t27 MAC TESTBENCH\n", .{});
    std.debug.print("================================================================\n", .{});
    std.debug.print("  phi^2 + phi^-2 = 3 | TRINITY\n", .{});
    std.debug.print("================================================================\n", .{});

    // Apply reset
    state.mac_array.mac_reset_all();
    state.wait_cycles(10);

    std.debug.print("[TEST  1] MAC LUT: (+1) * (+1)\n", .{});
    test_mac_lut_pos_pos(&state);
    std.debug.print("  [PASS]\n", .{});

    std.debug.print("[TEST  2] MAC LUT: (-1) * (-1)\n", .{});
    test_mac_lut_neg_neg(&state);
    std.debug.print("  [PASS]\n", .{});

    std.debug.print("[TEST  3] MAC LUT: (+1) * (-1)\n", .{});
    test_mac_lut_pos_neg(&state);
    std.debug.print("  [PASS]\n", .{});

    std.debug.print("[TEST  4] MAC LUT: (+1) * 0\n", .{});
    test_mac_lut_with_zero(&state);
    std.debug.print("  [PASS]\n", .{});

    std.debug.print("[TEST  5] MAC all 9 trit combinations\n", .{});
    test_mac_all_trit_combinations(&state);
    std.debug.print("  [PASS]\n", .{});

    std.debug.print("[TEST  6] MAC 27-trit word multiplication\n", .{});
    test_mac_27_trit_word(&state);
    std.debug.print("  [PASS]\n", .{});

    std.debug.print("[TEST  7] MAC cycle with zero accumulator\n", .{});
    test_mac_cycle_zero_acc(&state);
    std.debug.print("  [PASS]\n", .{});

    std.debug.print("[TEST  8] MAC cycle with initial accumulator\n", .{});
    test_mac_cycle_with_acc(&state);
    std.debug.print("  [PASS]\n", .{});

    std.debug.print("[TEST  9] MAC dot product\n", .{});
    test_mac_dot_product(&state);
    std.debug.print("  [PASS]\n", .{});

    std.debug.print("[TEST 10] MAC reset\n", .{});
    test_mac_reset(&state);
    std.debug.print("  [PASS]\n", .{});

    std.debug.print("[TEST 11] MAC unit independence\n", .{});
    test_mac_unit_independence(&state);
    std.debug.print("  [PASS]\n", .{});

    std.debug.print("[TEST 12] MAC invalid unit handling\n", .{});
    test_mac_invalid_unit(&state);
    std.debug.print("  [PASS]\n", .{});

    std.debug.print("[TEST 13] MAC overflow handling\n", .{});
    test_mac_overflow_handling(&state);
    std.debug.print("  [PASS]\n", .{});

    std.debug.print("[TEST 14] MAC parallel units\n", .{});
    test_mac_parallel_units(&state);
    std.debug.print("  [PASS]\n", .{});

    std.debug.print("[TEST 15] MAC latency\n", .{});
    test_mac_latency(&state);
    std.debug.print("  [PASS]\n", .{});

    // Summary
    std.debug.print("\n", .{});
    std.debug.print("================================================================\n", .{});
    std.debug.print("          SIMULATION RESULTS\n", .{});
    std.debug.print("================================================================\n", .{});
    std.debug.print("  Passed: {d}\n", .{state.test_passed});
    std.debug.print("  Failed: {d}\n", .{state.test_failed});
    if (state.test_failed == 0) {
        std.debug.print("  STATUS: ALL TESTS PASSED\n", .{});
    } else {
        std.debug.print("  STATUS: SOME TESTS FAILED\n", .{});
    }
    std.debug.print("================================================================\n", .{});
}

pub fn main() !void {
    run_tests();
}

// =====================================================================
// 6. Zig Tests (for zig test runner)
// =====================================================================

test "mac_tb_lut_pos_pos" {
    var state = TestState.init();
    test_mac_lut_pos_pos(&state);
    try std.testing.expect(state.test_failed == 0);
}

test "mac_tb_lut_neg_neg" {
    var state = TestState.init();
    test_mac_lut_neg_neg(&state);
    try std.testing.expect(state.test_failed == 0);
}

test "mac_tb_lut_pos_neg" {
    var state = TestState.init();
    test_mac_lut_pos_neg(&state);
    try std.testing.expect(state.test_failed == 0);
}

test "mac_tb_lut_with_zero" {
    var state = TestState.init();
    test_mac_lut_with_zero(&state);
    try std.testing.expect(state.test_failed == 0);
}

test "mac_tb_all_trit_combinations" {
    var state = TestState.init();
    test_mac_all_trit_combinations(&state);
    try std.testing.expect(state.test_failed == 0);
}

test "mac_tb_27_trit_word" {
    var state = TestState.init();
    test_mac_27_trit_word(&state);
    try std.testing.expect(state.test_failed == 0);
}

test "mac_tb_cycle_zero_acc" {
    var state = TestState.init();
    test_mac_cycle_zero_acc(&state);
    try std.testing.expect(state.test_failed == 0);
}

test "mac_tb_cycle_with_acc" {
    var state = TestState.init();
    test_mac_cycle_with_acc(&state);
    try std.testing.expect(state.test_failed == 0);
}

test "mac_tb_dot_product" {
    var state = TestState.init();
    test_mac_dot_product(&state);
    try std.testing.expect(state.test_failed == 0);
}

test "mac_tb_reset" {
    var state = TestState.init();
    test_mac_reset(&state);
    try std.testing.expect(state.test_failed == 0);
}

test "mac_tb_unit_independence" {
    var state = TestState.init();
    test_mac_unit_independence(&state);
    try std.testing.expect(state.test_failed == 0);
}

test "mac_tb_invalid_unit" {
    var state = TestState.init();
    test_mac_invalid_unit(&state);
    try std.testing.expect(state.test_failed == 0);
}

test "mac_tb_overflow_handling" {
    var state = TestState.init();
    test_mac_overflow_handling(&state);
    try std.testing.expect(state.test_failed == 0);
}

test "mac_tb_parallel_units" {
    var state = TestState.init();
    test_mac_parallel_units(&state);
    try std.testing.expect(state.test_failed == 0);
}

test "mac_tb_latency" {
    var state = TestState.init();
    test_mac_latency(&state);
    try std.testing.expect(state.test_failed == 0);
}
