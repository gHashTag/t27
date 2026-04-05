// Auto-generated from specs/fpga/top_level.t27
// DO NOT EDIT -- regenerate with: tri gen specs/fpga/top_level.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 43 | Module: Trinity_FPGA_Top

const std = @import("std");

// =====================================================================
// 1. Device Configuration
// =====================================================================

pub const TARGET_DEVICE: []const u8 = "xc7a100t";
pub const TARGET_FAMILY: []const u8 = "artix7";
pub const PACKAGE: []const u8 = "fgg676c";

pub const CLK_FREQ: u32 = 50_000_000;
pub const CLK_PIN: []const u8 = "E19";

pub const RESET_PIN: []const u8 = "C12";
pub const RESET_ACTIVE_LOW: bool = true;

// =====================================================================
// 2. I/O Pin Configuration
// =====================================================================

pub const UART_TX_PIN: []const u8 = "K20";
pub const UART_RX_PIN: []const u8 = "L20";
pub const UART_BAUD: u32 = 115200;

pub const SPI_CS_PIN: []const u8 = "G13";
pub const SPI_SCK_PIN: []const u8 = "K13";
pub const SPI_MOSI_PIN: []const u8 = "H13";
pub const SPI_MISO_PIN: []const u8 = "J13";

pub const LED_PINS: [4][]const u8 = .{ "R5", "T5", "T8", "T9" };
pub const LED_ACTIVE_LOW: bool = true;

pub const MAC_DATA_WIDTH: u8 = 27;

// =====================================================================
// 3. Top-Level Ports
// =====================================================================

pub const TopPorts = struct {
    clk: bool,
    rst_n: bool,
    uart_rx_in: bool,
    uart_tx_out: bool,
    spi_miso_in: bool,
    spi_cs_out: bool,
    spi_sck_out: bool,
    spi_mosi_out: bool,
    led_out: [4]bool,
    mac_a_in: [27]bool,
    mac_b_in: [27]bool,
    mac_acc_in: i32,
    mac_acc_out: i32,
    mac_valid_out: bool,
};

// =====================================================================
// 4. Top-Level State
// =====================================================================

pub const TopState = struct {
    reset_pending: bool,
    led_state: [4]bool,
    heartbeat_cnt: u32,
};

// =====================================================================
// 5. LED Control
// =====================================================================

pub const LED_UART_TX: u8 = 0;
pub const LED_SPI_CS: u8 = 1;
pub const LED_MAC_BUSY: u8 = 2;
pub const LED_HEARTBEAT: u8 = 3;

pub const TopLevel = struct {
    state: TopState,
    ports: TopPorts,

    pub fn init() TopLevel {
        return TopLevel{
            .state = TopState{
                .reset_pending = false,
                .led_state = .{ true, true, true, true },
                .heartbeat_cnt = 0,
            },
            .ports = TopPorts{
                .clk = false,
                .rst_n = true,
                .uart_rx_in = true,
                .uart_tx_out = true,
                .spi_miso_in = true,
                .spi_cs_out = true,
                .spi_sck_out = true,
                .spi_mosi_out = true,
                .led_out = .{ true, true, true, true },
                .mac_a_in = .{false} ** 27,
                .mac_b_in = .{false} ** 27,
                .mac_acc_in = 0,
                .mac_acc_out = 0,
                .mac_valid_out = false,
            },
        };
    }

    // -----------------------------------------------------------------
    // LED helpers
    // -----------------------------------------------------------------

    /// Set LED state (accounting for active-low).
    pub fn led_set(self: *TopLevel, led: u8, state: bool) void {
        if (led >= 4) return;
        const active_state = if (LED_ACTIVE_LOW) !state else state;
        self.state.led_state[led] = active_state;
    }

    /// Get LED state (accounting for active-low).
    pub fn led_get(self: *const TopLevel, led: u8) bool {
        if (led >= 4) return false;
        const active_state = self.state.led_state[led];
        return if (LED_ACTIVE_LOW) !active_state else active_state;
    }

    /// Update LED output ports from internal state.
    pub fn led_update_outputs(self: *TopLevel) void {
        var i: u8 = 0;
        while (i < 4) : (i += 1) {
            self.ports.led_out[i] = self.state.led_state[i];
        }
    }

    // -----------------------------------------------------------------
    // 6. Top-Level Control
    // -----------------------------------------------------------------

    /// Reset all subsystems.
    pub fn top_reset(self: *TopLevel) void {
        // Reset LEDs (all off -- active low means true = off)
        var i: u8 = 0;
        while (i < 4) : (i += 1) {
            self.led_set(i, false);
        }

        // Reset heartbeat counter
        self.state.heartbeat_cnt = 0;

        // Reset MAC outputs
        self.ports.mac_acc_out = 0;
        self.ports.mac_valid_out = false;

        // Reset SPI outputs
        self.ports.spi_cs_out = true;
        self.ports.spi_sck_out = true;
        self.ports.spi_mosi_out = true;

        // Reset UART output
        self.ports.uart_tx_out = true;
    }

    /// Update heartbeat LED (blinks at 1 Hz).
    pub fn top_heartbeat(self: *TopLevel) void {
        self.state.heartbeat_cnt += 1;
        const blink_threshold: u32 = CLK_FREQ / 2;

        if (self.state.heartbeat_cnt >= blink_threshold) {
            self.state.heartbeat_cnt = 0;
            const current = self.led_get(LED_HEARTBEAT);
            self.led_set(LED_HEARTBEAT, !current);
        }
    }

    /// Process one clock cycle.
    pub fn top_tick(self: *TopLevel) void {
        // Handle reset
        if (RESET_ACTIVE_LOW) {
            if (!self.ports.rst_n) {
                if (!self.state.reset_pending) {
                    self.top_reset();
                    self.state.reset_pending = true;
                }
                return;
            }
        } else {
            if (self.ports.rst_n) {
                if (!self.state.reset_pending) {
                    self.top_reset();
                    self.state.reset_pending = true;
                }
                return;
            }
        }
        self.state.reset_pending = false;

        // Update LED outputs
        self.led_update_outputs();

        // Heartbeat
        self.top_heartbeat();
    }
};

// =====================================================================
// Tests
// =====================================================================

test "top_device_configured" {
    try std.testing.expect(std.mem.eql(u8, TARGET_DEVICE, "xc7a100t"));
    try std.testing.expect(std.mem.eql(u8, TARGET_FAMILY, "artix7"));
}

test "top_clk_frequency" {
    try std.testing.expectEqual(@as(u32, 50_000_000), CLK_FREQ);
}

test "top_clk_pin_defined" {
    try std.testing.expect(std.mem.eql(u8, CLK_PIN, "E19"));
}

test "top_reset_pin_defined" {
    try std.testing.expect(std.mem.eql(u8, RESET_PIN, "C12"));
    try std.testing.expectEqual(true, RESET_ACTIVE_LOW);
}

test "top_uart_pins_defined" {
    try std.testing.expect(std.mem.eql(u8, UART_TX_PIN, "K20"));
    try std.testing.expect(std.mem.eql(u8, UART_RX_PIN, "L20"));
    try std.testing.expectEqual(@as(u32, 115200), UART_BAUD);
}

test "top_spi_pins_defined" {
    try std.testing.expect(std.mem.eql(u8, SPI_CS_PIN, "G13"));
    try std.testing.expect(std.mem.eql(u8, SPI_SCK_PIN, "K13"));
    try std.testing.expect(std.mem.eql(u8, SPI_MOSI_PIN, "H13"));
    try std.testing.expect(std.mem.eql(u8, SPI_MISO_PIN, "J13"));
}

test "top_led_pins_defined" {
    try std.testing.expectEqual(@as(usize, 4), LED_PINS.len);
    try std.testing.expect(std.mem.eql(u8, LED_PINS[0], "R5"));
}

test "top_leds_active_low" {
    try std.testing.expectEqual(true, LED_ACTIVE_LOW);
}

test "top_mac_data_width" {
    try std.testing.expectEqual(@as(u8, 27), MAC_DATA_WIDTH);
}

test "top_initially_not_resetting" {
    const top = TopLevel.init();
    try std.testing.expectEqual(false, top.state.reset_pending);
}

test "top_leds_initially_off" {
    const top = TopLevel.init();
    // All LEDs are logically off (active-low: state=true means off)
    try std.testing.expectEqual(false, top.led_get(LED_UART_TX));
    try std.testing.expectEqual(false, top.led_get(LED_SPI_CS));
    try std.testing.expectEqual(false, top.led_get(LED_MAC_BUSY));
    try std.testing.expectEqual(false, top.led_get(LED_HEARTBEAT));
}

test "top_led_set_inverts_active_low" {
    var top = TopLevel.init();
    top.led_set(LED_UART_TX, true);
    // Active-low: setting true stores false
    try std.testing.expectEqual(false, top.state.led_state[LED_UART_TX]);
}

test "top_led_get_inverts_active_low" {
    var top = TopLevel.init();
    top.state.led_state[LED_HEARTBEAT] = true;
    // Active-low: stored true reads as false
    try std.testing.expectEqual(false, top.led_get(LED_HEARTBEAT));
}

test "top_heartbeat_threshold" {
    const threshold: u32 = CLK_FREQ / 2;
    try std.testing.expectEqual(@as(u32, 25_000_000), threshold);
}

test "top_led_count_4" {
    try std.testing.expectEqual(@as(usize, 4), LED_PINS.len);
}

test "top_led_indices_valid" {
    try std.testing.expectEqual(@as(u8, 0), LED_UART_TX);
    try std.testing.expectEqual(@as(u8, 1), LED_SPI_CS);
    try std.testing.expectEqual(@as(u8, 2), LED_MAC_BUSY);
    try std.testing.expectEqual(@as(u8, 3), LED_HEARTBEAT);
}

test "top_mac_operands_27_trits" {
    const top = TopLevel.init();
    try std.testing.expectEqual(@as(usize, 27), top.ports.mac_a_in.len);
}
