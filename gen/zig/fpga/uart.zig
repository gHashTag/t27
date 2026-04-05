// Auto-generated from specs/fpga/uart.t27
// DO NOT EDIT -- regenerate with: tri gen specs/fpga/uart.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 43 | Module: UART_Bridge

const std = @import("std");

// =====================================================================
// 1. UART Configuration
// =====================================================================

pub const CLK_FREQ: u32 = 50_000_000; // 50 MHz system clock
pub const BAUD_RATE: u32 = 115200; // 115200 baud
pub const BAUD_DIVISOR: u32 = CLK_FREQ / BAUD_RATE; // ~434

pub const DATA_BITS: u8 = 8; // 8 data bits
pub const STOP_BITS: u8 = 1; // 1 stop bit
pub const PARITY_BITS: u8 = 0; // No parity

// =====================================================================
// 2. UART State Machine States
// =====================================================================

// TX States
pub const TX_IDLE: u8 = 0;
pub const TX_START: u8 = 1;
pub const TX_DATA: u8 = 2;
pub const TX_STOP: u8 = 3;

// RX States
pub const RX_IDLE: u8 = 0;
pub const RX_START: u8 = 1;
pub const RX_DATA: u8 = 2;
pub const RX_STOP: u8 = 3;

// =====================================================================
// 3. UART TX Unit
// =====================================================================

pub const UART_TX_Unit = struct {
    tx_busy: bool,
    tx_state: u8,
    bit_index: u8,
    shift_reg: u8,
    baud_counter: u32,

    pub fn init() UART_TX_Unit {
        return UART_TX_Unit{
            .tx_busy = false,
            .tx_state = TX_IDLE,
            .bit_index = 0,
            .shift_reg = 0,
            .baud_counter = 0,
        };
    }

    /// Queue a byte for transmission. Returns false if busy.
    pub fn uart_tx_write(self: *UART_TX_Unit, data: u8) bool {
        if (self.tx_busy) {
            return false;
        }
        self.shift_reg = data;
        self.bit_index = 0;
        self.tx_state = TX_START;
        self.tx_busy = true;
        self.baud_counter = 0;
        return true;
    }

    /// Check if TX unit is ready for new data.
    pub fn uart_tx_ready(self: *const UART_TX_Unit) bool {
        return !self.tx_busy;
    }

    /// Get current TX line state (for physical output).
    pub fn uart_tx_get_line(self: *const UART_TX_Unit) bool {
        return switch (self.tx_state) {
            TX_IDLE => true, // Idle high
            TX_START => false, // Start bit low
            TX_DATA => ((self.shift_reg >> @intCast(self.bit_index)) & 1) == 1,
            TX_STOP => true, // Stop bit high
            else => true,
        };
    }

    /// Process one clock cycle of TX state machine.
    pub fn uart_tx_tick(self: *UART_TX_Unit) void {
        self.baud_counter += 1;

        switch (self.tx_state) {
            TX_IDLE => {
                // Do nothing, waiting for write
            },
            TX_START => {
                if (self.baud_counter >= BAUD_DIVISOR) {
                    self.baud_counter = 0;
                    self.tx_state = TX_DATA;
                }
            },
            TX_DATA => {
                if (self.baud_counter >= BAUD_DIVISOR) {
                    self.baud_counter = 0;
                    self.bit_index += 1;
                    if (self.bit_index >= DATA_BITS - 1) {
                        self.tx_state = TX_STOP;
                    }
                }
            },
            TX_STOP => {
                if (self.baud_counter >= BAUD_DIVISOR) {
                    self.tx_state = TX_IDLE;
                    self.tx_busy = false;
                }
            },
            else => {},
        }
    }
};

// =====================================================================
// 4. UART RX Unit
// =====================================================================

pub const UART_RX_Unit = struct {
    rx_state: u8,
    bit_index: u8,
    shift_reg: u8,
    baud_counter: u32,
    rx_sync: [3]bool,
    framing_error: bool,

    pub fn init() UART_RX_Unit {
        return UART_RX_Unit{
            .rx_state = RX_IDLE,
            .bit_index = 0,
            .shift_reg = 0,
            .baud_counter = 0,
            .rx_sync = .{ true, true, true },
            .framing_error = false,
        };
    }

    /// Synchronize async UART input.
    pub fn uart_rx_sync(self: *UART_RX_Unit, rx_input: bool) void {
        self.rx_sync[2] = self.rx_sync[1];
        self.rx_sync[1] = self.rx_sync[0];
        self.rx_sync[0] = rx_input;
    }

    /// Check if data is ready to read.
    pub fn uart_rx_has_data(self: *const UART_RX_Unit) bool {
        return self.rx_state == RX_IDLE and self.framing_error;
    }

    /// Read received data byte.
    pub fn uart_rx_read_data(self: *UART_RX_Unit) u8 {
        self.framing_error = false;
        return self.shift_reg;
    }

    /// Check if framing error occurred.
    pub fn uart_rx_has_framing_error(self: *const UART_RX_Unit) bool {
        return self.framing_error;
    }

    /// Get synchronized RX input.
    pub fn uart_rx_get_line(self: *const UART_RX_Unit) bool {
        return self.rx_sync[1];
    }

    /// Process one clock cycle of RX state machine.
    pub fn uart_rx_tick(self: *UART_RX_Unit) void {
        self.baud_counter += 1;

        switch (self.rx_state) {
            RX_IDLE => {
                if (!self.rx_sync[1]) {
                    self.baud_counter = BAUD_DIVISOR / 2;
                    self.rx_state = RX_START;
                }
            },
            RX_START => {
                if (self.baud_counter >= BAUD_DIVISOR) {
                    self.baud_counter = 0;
                    if (!self.rx_sync[1]) {
                        self.rx_state = RX_DATA;
                        self.bit_index = 0;
                    } else {
                        self.rx_state = RX_IDLE;
                    }
                }
            },
            RX_DATA => {
                if (self.baud_counter >= BAUD_DIVISOR) {
                    self.baud_counter = 0;
                    const bit_val: u8 = if (self.rx_sync[1]) 1 else 0;
                    self.shift_reg = (self.shift_reg & ~(@as(u8, 1) << @intCast(self.bit_index))) |
                        (bit_val << @intCast(self.bit_index));
                    self.bit_index += 1;
                    if (self.bit_index >= DATA_BITS - 1) {
                        self.rx_state = RX_STOP;
                    }
                }
            },
            RX_STOP => {
                if (self.baud_counter >= BAUD_DIVISOR) {
                    if (!self.rx_sync[1]) {
                        self.framing_error = true;
                    } else {
                        self.framing_error = false;
                    }
                    self.rx_state = RX_IDLE;
                }
            },
            else => {},
        }
    }
};

// =====================================================================
// 5. Bridge Protocol
// =====================================================================

pub const CMD_PING: u8 = 0x01;
pub const CMD_PONG: u8 = 0x02;
pub const CMD_WRITE_REG: u8 = 0x10;
pub const CMD_READ_REG: u8 = 0x11;
pub const CMD_MAC_OP: u8 = 0x20;
pub const CMD_STATUS: u8 = 0x30;

pub const RESP_OK: u8 = 0x00;
pub const RESP_ERROR: u8 = 0xFF;

// =====================================================================
// Tests
// =====================================================================

test "uart_baud_divisor_calculation" {
    try std.testing.expectEqual(@as(u32, 434), BAUD_DIVISOR);
}

test "uart_tx_initially_not_busy" {
    const tx = UART_TX_Unit.init();
    try std.testing.expectEqual(false, tx.tx_busy);
}

test "uart_tx_write_when_ready" {
    var tx = UART_TX_Unit.init();
    const result = tx.uart_tx_write(0xAA);
    try std.testing.expectEqual(true, result);
}

test "uart_tx_write_when_busy" {
    var tx = UART_TX_Unit.init();
    _ = tx.uart_tx_write(0x55);
    const ready = tx.uart_tx_ready();
    try std.testing.expectEqual(false, ready);
}

test "uart_tx_idle_line_high" {
    const tx = UART_TX_Unit.init();
    const line = tx.uart_tx_get_line();
    try std.testing.expectEqual(true, line);
}

test "uart_tx_start_bit_low" {
    var tx = UART_TX_Unit.init();
    _ = tx.uart_tx_write(0xAA);
    const line = tx.uart_tx_get_line();
    try std.testing.expectEqual(false, line);
}

test "uart_rx_initially_idle" {
    const rx = UART_RX_Unit.init();
    try std.testing.expectEqual(RX_IDLE, rx.rx_state);
}

test "uart_rx_detects_start_bit" {
    var rx = UART_RX_Unit.init();
    rx.uart_rx_sync(false);
    rx.uart_rx_sync(false);
    rx.uart_rx_tick();
    try std.testing.expectEqual(RX_START, rx.rx_state);
}

test "uart_rx_sync_chain" {
    var rx = UART_RX_Unit.init();
    rx.uart_rx_sync(false);
    try std.testing.expectEqual(false, rx.rx_sync[0]);
    try std.testing.expectEqual(true, rx.rx_sync[1]);
    try std.testing.expectEqual(true, rx.rx_sync[2]);
}

test "uart_rx_framing_error_detection" {
    var rx = UART_RX_Unit.init();
    rx.rx_sync = .{ false, false, false };
    rx.rx_state = RX_STOP;
    rx.baud_counter = BAUD_DIVISOR;
    rx.uart_rx_tick();
    try std.testing.expectEqual(true, rx.uart_rx_has_framing_error());
}

test "uart_protocol_ping_defined" {
    try std.testing.expectEqual(@as(u8, 0x01), CMD_PING);
}

test "uart_protocol_pong_defined" {
    try std.testing.expectEqual(@as(u8, 0x02), CMD_PONG);
}
