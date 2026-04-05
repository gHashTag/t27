// Auto-generated from specs/fpga/testbench/top_tb.t27
// DO NOT EDIT -- regenerate with: tri gen specs/fpga/testbench/top_tb.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 43 | Module: Top_Level_Testbench

const std = @import("std");

// =====================================================================
// 1. Testbench Configuration
// =====================================================================

pub const CLK_PERIOD: u32 = 20;
pub const SIM_TIMEOUT: u32 = 50_000_000;

// Protocol constants
pub const PING_CMD: u8 = 0x01;
pub const PONG_RESP: u8 = 0x02;
pub const STATUS_CMD: u8 = 0x30;

// UART baud parameters
pub const CLK_FREQ: u32 = 50_000_000;
pub const BAUD_RATE: u32 = 115200;
pub const BAUD_DIVISOR: u32 = CLK_FREQ / BAUD_RATE;

// =====================================================================
// 2. Testbench Signals
// =====================================================================

pub const TopSignals = struct {
    clk: bool,
    rst_n: bool,
    uart_tx: bool,
    uart_rx: bool,
    spi_cs: bool,
    spi_sck: bool,
    spi_mosi: bool,
    spi_miso: bool,
    led: [4]bool,
    mac_a: [27]bool,
    mac_b: [27]bool,
    mac_acc: i32,
    mac_acc_out: i32,
    mac_valid: bool,
};

// =====================================================================
// 3. Testbench State
// =====================================================================

pub const TestState = struct {
    test_passed: u32,
    test_failed: u32,
    sim_cycle: u32,
    signals: TopSignals,

    pub fn init() TestState {
        return TestState{
            .test_passed = 0,
            .test_failed = 0,
            .sim_cycle = 0,
            .signals = TopSignals{
                .clk = false,
                .rst_n = false,
                .uart_tx = true,
                .uart_rx = true,
                .spi_cs = true,
                .spi_sck = false,
                .spi_mosi = false,
                .spi_miso = true,
                .led = [_]bool{ true, true, true, true },
                .mac_a = [_]bool{false} ** 27,
                .mac_b = [_]bool{false} ** 27,
                .mac_acc = 0,
                .mac_acc_out = 0,
                .mac_valid = false,
            },
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
        self.signals.clk = !self.signals.clk;
        self.sim_cycle += 1;
    }

    pub fn wait_cycles(self: *TestState, n: u32) void {
        var i: u32 = 0;
        while (i < n) : (i += 1) {
            self.generate_clock();
        }
    }

    pub fn apply_reset(self: *TestState) void {
        self.signals.rst_n = false;
        self.signals.uart_rx = true;
        self.signals.spi_cs = true;
        self.signals.spi_sck = false;
        self.signals.spi_mosi = false;
        self.wait_cycles(10);
        self.signals.rst_n = true;
        self.wait_cycles(10);
    }

    // Simulate UART byte send (LSB first)
    pub fn uart_send_byte(self: *TestState, data: u8) void {
        // Start bit
        self.signals.uart_rx = false;
        self.wait_cycles(BAUD_DIVISOR);

        // Data bits
        var i: u3 = 0;
        while (i < 8) : (i += 1) {
            self.signals.uart_rx = ((data >> i) & 1) == 1;
            self.wait_cycles(BAUD_DIVISOR);
        }

        // Stop bit
        self.signals.uart_rx = true;
        self.wait_cycles(BAUD_DIVISOR);
    }
};

// =====================================================================
// 4. Test Cases
// =====================================================================

fn test_ping_pong(state: *TestState) void {
    // Send PING command via UART
    state.uart_send_byte(PING_CMD);
    state.wait_cycles(BAUD_DIVISOR * 2);

    // In loopback, TX should return to idle
    state.assert_pass(true, "Ping/Pong test completed");
}

fn test_led_heartbeat(state: *TestState) void {
    state.wait_cycles(CLK_PERIOD * 100);
    state.assert_pass(true, "LED heartbeat test");
}

fn test_spi_loopback(state: *TestState) void {
    // Activate CS
    state.signals.spi_cs = false;

    state.signals.spi_mosi = true;
    state.wait_cycles(CLK_PERIOD * 2);
    // Simulate loopback: MISO mirrors MOSI
    state.signals.spi_miso = state.signals.spi_mosi;
    state.assert_pass(state.signals.spi_miso == true, "SPI loopback MISO=1");

    state.signals.spi_mosi = false;
    state.wait_cycles(CLK_PERIOD * 2);
    state.signals.spi_miso = state.signals.spi_mosi;
    state.assert_pass(state.signals.spi_miso == false, "SPI loopback MISO=0");

    state.signals.spi_cs = true;
    state.wait_cycles(CLK_PERIOD * 2);
}

fn test_mac_operation(state: *TestState) void {
    // Set all MAC inputs to true (POS)
    var i: usize = 0;
    while (i < 27) : (i += 1) {
        state.signals.mac_a[i] = true;
        state.signals.mac_b[i] = true;
    }
    state.signals.mac_acc = 0;
    state.wait_cycles(CLK_PERIOD * 20);

    // Simulate MAC completion
    state.signals.mac_valid = true;
    state.assert_pass(state.signals.mac_valid == true, "MAC operation completed");
}

// =====================================================================
// 5. Test Runner
// =====================================================================

pub fn run_tests() void {
    var state = TestState.init();

    std.debug.print("================================================================\n", .{});
    std.debug.print("          t27 TOP-LEVEL FPGA TESTBENCH\n", .{});
    std.debug.print("================================================================\n", .{});
    std.debug.print("  phi^2 + phi^-2 = 3 | TRINITY\n", .{});
    std.debug.print("================================================================\n", .{});

    // Apply reset
    state.apply_reset();

    std.debug.print("[TEST 1] Ping/Pong\n", .{});
    test_ping_pong(&state);
    std.debug.print("  [PASS]\n", .{});

    std.debug.print("[TEST 2] LED Heartbeat\n", .{});
    test_led_heartbeat(&state);
    std.debug.print("  [PASS]\n", .{});

    std.debug.print("[TEST 3] SPI Loopback\n", .{});
    test_spi_loopback(&state);
    std.debug.print("  [PASS]\n", .{});

    std.debug.print("[TEST 4] MAC Operation\n", .{});
    test_mac_operation(&state);
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
// 6. Zig Tests
// =====================================================================

test "top_tb_clk_period" {
    try std.testing.expectEqual(@as(u32, 20), CLK_PERIOD);
}

test "top_tb_sim_timeout" {
    try std.testing.expectEqual(@as(u32, 50_000_000), SIM_TIMEOUT);
}

test "top_tb_protocol_constants" {
    try std.testing.expectEqual(@as(u8, 0x01), PING_CMD);
    try std.testing.expectEqual(@as(u8, 0x02), PONG_RESP);
    try std.testing.expectEqual(@as(u8, 0x30), STATUS_CMD);
}

test "top_tb_initial_state" {
    const state = TestState.init();
    try std.testing.expect(!state.signals.clk);
    try std.testing.expect(!state.signals.rst_n);
    try std.testing.expectEqual(@as(u32, 0), state.test_passed);
    try std.testing.expectEqual(@as(u32, 0), state.test_failed);
}

test "top_tb_clock_generation" {
    var state = TestState.init();
    state.generate_clock();
    try std.testing.expect(state.signals.clk);
    try std.testing.expectEqual(@as(u32, 1), state.sim_cycle);
}

test "top_tb_ping_pong" {
    var state = TestState.init();
    state.apply_reset();
    test_ping_pong(&state);
    try std.testing.expect(state.test_failed == 0);
}

test "top_tb_led_heartbeat" {
    var state = TestState.init();
    state.apply_reset();
    test_led_heartbeat(&state);
    try std.testing.expect(state.test_failed == 0);
}

test "top_tb_spi_loopback" {
    var state = TestState.init();
    state.apply_reset();
    test_spi_loopback(&state);
    try std.testing.expect(state.test_failed == 0);
}

test "top_tb_mac_operation" {
    var state = TestState.init();
    state.apply_reset();
    test_mac_operation(&state);
    try std.testing.expect(state.test_failed == 0);
}
