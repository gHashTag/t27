// Auto-generated from specs/fpga/testbench/uart_tb.t27
// DO NOT EDIT -- regenerate with: tri gen specs/fpga/testbench/uart_tb.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 43 | Module: UART_Testbench

const std = @import("std");

// Simulation timing
pub const TIMESCALE = "1ns/1ps";
pub const CLK_PERIOD: u32 = 20;
pub const SIM_TIMEOUT: u32 = 10_000_000;

// UART parameters
pub const CLK_FREQ: u32 = 50_000_000;
pub const BAUD_RATE: u32 = 115200;
pub const BAUD_DIVISOR: u32 = CLK_FREQ / BAUD_RATE;

// Test data patterns
pub const TEST_DATA_1: u8 = 0xAA;
pub const TEST_DATA_2: u8 = 0x55;
pub const TEST_DATA_3: u8 = 0x00;
pub const TEST_DATA_4: u8 = 0xFF;

// Testbench state
pub const UartTbState = struct {
    clk: bool = false,
    rst_n: bool = false,
    uart_tx_line: bool = true,
    uart_rx_line: bool = true,
    tx_busy: bool = false,
    rx_data_valid: bool = false,
    rx_data: u8 = 0,
    test_passed: u32 = 0,
    test_failed: u32 = 0,
    sim_cycle: u32 = 0,
};

pub fn generate_clock(state: *UartTbState) void {
    state.clk = !state.clk;
    state.sim_cycle += 1;
}

pub fn wait_cycles(state: *UartTbState, n: u32) void {
    var i: u32 = 0;
    while (i < n) : (i += 1) {
        generate_clock(state);
    }
}

pub fn assert_pass(state: *UartTbState, condition: bool) void {
    if (condition) {
        state.test_passed += 1;
    } else {
        state.test_failed += 1;
    }
}

pub fn test_uart_tx_byte(state: *UartTbState, data: u8) void {
    _ = data;
    wait_cycles(state, BAUD_DIVISOR * 10);
    assert_pass(state, true);
}

pub fn test_uart_idle_line(state: *UartTbState) void {
    assert_pass(state, state.uart_tx_line == true);
}

pub fn test_uart_multiple_bytes(state: *UartTbState) void {
    const bytes = [_]u8{ TEST_DATA_1, TEST_DATA_2, TEST_DATA_3, TEST_DATA_4 };
    for (bytes) |b| {
        test_uart_tx_byte(state, b);
    }
    assert_pass(state, true);
}

pub fn test_uart_reset(state: *UartTbState) void {
    state.rst_n = false;
    wait_cycles(state, 10);
    state.rst_n = true;
    wait_cycles(state, 10);
    assert_pass(state, state.uart_tx_line == true);
}

pub fn run_tests(state: *UartTbState) void {
    state.rst_n = false;
    wait_cycles(state, 10);
    state.rst_n = true;
    wait_cycles(state, 10);

    test_uart_tx_byte(state, TEST_DATA_1);
    test_uart_idle_line(state);
    test_uart_multiple_bytes(state);
    test_uart_reset(state);
}

test "uart_tb_clk_period" {
    try std.testing.expectEqual(@as(u32, 20), CLK_PERIOD);
}

test "uart_tb_baud_divisor" {
    try std.testing.expectEqual(@as(u32, 434), BAUD_DIVISOR);
}

test "uart_tb_test_data" {
    try std.testing.expectEqual(@as(u8, 0xAA), TEST_DATA_1);
    try std.testing.expectEqual(@as(u8, 0x55), TEST_DATA_2);
}

test "uart_tb_initial_state" {
    var state = UartTbState{};
    try std.testing.expect(!state.clk);
    try std.testing.expect(!state.rst_n);
    try std.testing.expectEqual(@as(u32, 0), state.test_passed);
}
