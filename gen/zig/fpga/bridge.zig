// Auto-generated from specs/fpga/bridge.t27
// DO NOT EDIT -- regenerate with: tri gen specs/fpga/bridge.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 43 | Module: FPGA_Bridge

const std = @import("std");

// =====================================================================
// 1. Bridge Configuration
// =====================================================================

pub const RX_BUFFER_SIZE: usize = 256;
pub const TX_BUFFER_SIZE: usize = 256;
pub const SPI_BUFFER_SIZE: usize = 64;
pub const MAX_PACKET_SIZE: usize = 128;
pub const PACKET_TIMEOUT: u32 = 10_000;

// =====================================================================
// 2. Bridge State
// =====================================================================

pub const BRIDGE_IDLE: u8 = 0;
pub const BRIDGE_RX: u8 = 1;
pub const BRIDGE_PARSE: u8 = 2;
pub const BRIDGE_TX: u8 = 3;
pub const BRIDGE_SPI: u8 = 4;
pub const BRIDGE_MAC: u8 = 5;

// =====================================================================
// 3. Packet Types
// =====================================================================

pub const PKT_UART_DATA: u8 = 0x00;
pub const PKT_SPI_XFER: u8 = 0x10;
pub const PKT_MAC_OP: u8 = 0x20;
pub const PKT_STATUS: u8 = 0x30;
pub const PKT_CONFIG: u8 = 0x40;

// =====================================================================
// 4. Bridge Unit State
// =====================================================================

pub const BridgeUnit = struct {
    state: u8,
    rx_head: usize,
    rx_tail: usize,
    tx_head: usize,
    tx_tail: usize,
    packet_len: u8,
    packet_type: u8,
    timeout_cnt: u32,
    spi_enabled: bool,
    mac_enabled: bool,

    rx_buffer: [RX_BUFFER_SIZE]u8,
    tx_buffer: [TX_BUFFER_SIZE]u8,

    pub fn init() BridgeUnit {
        return BridgeUnit{
            .state = BRIDGE_IDLE,
            .rx_head = 0,
            .rx_tail = 0,
            .tx_head = 0,
            .tx_tail = 0,
            .packet_len = 0,
            .packet_type = 0,
            .timeout_cnt = 0,
            .spi_enabled = true,
            .mac_enabled = true,
            .rx_buffer = [_]u8{0} ** RX_BUFFER_SIZE,
            .tx_buffer = [_]u8{0} ** TX_BUFFER_SIZE,
        };
    }

    // =================================================================
    // 5. Buffer Management
    // =================================================================

    /// Write byte to circular buffer. Returns new head on success, null on full.
    pub fn buffer_write(buf: []u8, size: usize, head: usize, data: u8) ?usize {
        const new_head = (head + 1) % size;
        if (new_head == 0 and head == size - 1) {
            return null;
        }
        buf[head] = data;
        return new_head;
    }

    /// Read byte from circular buffer. Returns (data, new_tail) or null if empty.
    pub fn buffer_read(buf: []const u8, size: usize, tail: usize) ?struct { data: u8, new_tail: usize } {
        if (tail == size) {
            return null;
        }
        const data = buf[tail];
        const new_tail = (tail + 1) % size;
        return .{ .data = data, .new_tail = new_tail };
    }

    /// Count bytes in circular buffer.
    pub fn buffer_count(head: usize, tail: usize, size: usize) usize {
        if (head >= tail) {
            return head - tail;
        } else {
            return head + size - tail;
        }
    }

    /// Get available bytes in RX buffer.
    pub fn bridge_rx_available(self: *const BridgeUnit) usize {
        return buffer_count(self.rx_head, self.rx_tail, RX_BUFFER_SIZE);
    }

    /// Get available space in TX buffer.
    pub fn bridge_tx_space(self: *const BridgeUnit) usize {
        return TX_BUFFER_SIZE - buffer_count(self.tx_head, self.tx_tail, TX_BUFFER_SIZE);
    }

    // =================================================================
    // 6. Packet Protocol
    // =================================================================

    /// Parse packet header from RX buffer.
    pub fn bridge_parse_header(self: *BridgeUnit) bool {
        if (self.bridge_rx_available() < 2) {
            return false;
        }

        const read1 = buffer_read(&self.rx_buffer, RX_BUFFER_SIZE, self.rx_tail) orelse return false;
        const read2 = buffer_read(&self.rx_buffer, RX_BUFFER_SIZE, read1.new_tail) orelse return false;
        self.rx_tail = read2.new_tail;

        self.packet_type = read1.data;
        self.packet_len = read2.data;

        if (self.packet_len > MAX_PACKET_SIZE) {
            return false;
        }

        self.state = BRIDGE_PARSE;
        self.timeout_cnt = 0;
        return true;
    }

    /// Process packet payload based on type.
    pub fn bridge_process_payload(self: *BridgeUnit) bool {
        if (self.bridge_rx_available() < @as(usize, self.packet_len)) {
            self.timeout_cnt = self.timeout_cnt + 1;
            if (self.timeout_cnt > PACKET_TIMEOUT) {
                self.state = BRIDGE_IDLE;
                self.rx_tail = self.rx_head;
            }
            return false;
        }

        switch (self.packet_type) {
            PKT_UART_DATA => self.bridge_handle_uart_data(),
            PKT_SPI_XFER => self.bridge_handle_spi_xfer(),
            PKT_MAC_OP => self.bridge_handle_mac_op(),
            PKT_STATUS => self.bridge_handle_status(),
            PKT_CONFIG => self.bridge_handle_config(),
            else => {
                self.state = BRIDGE_IDLE;
                self.rx_tail = self.rx_head;
                return false;
            },
        }

        self.state = BRIDGE_IDLE;
        return true;
    }

    // =================================================================
    // 7. Packet Handlers
    // =================================================================

    /// Handle UART data packet (echo back).
    pub fn bridge_handle_uart_data(self: *BridgeUnit) void {
        var i: usize = 0;
        while (i < @as(usize, self.packet_len)) : (i += 1) {
            const read_result = buffer_read(&self.rx_buffer, RX_BUFFER_SIZE, self.rx_tail) orelse break;
            self.rx_tail = read_result.new_tail;

            if (self.bridge_tx_space() > 0) {
                self.tx_buffer[self.tx_head] = read_result.data;
                self.tx_head = (self.tx_head + 1) % TX_BUFFER_SIZE;
            }
        }
    }

    /// Handle SPI transfer packet.
    pub fn bridge_handle_spi_xfer(self: *BridgeUnit) void {
        if (!self.spi_enabled) {
            return;
        }

        const read1 = buffer_read(&self.rx_buffer, RX_BUFFER_SIZE, self.rx_tail) orelse return;
        const read2 = buffer_read(&self.rx_buffer, RX_BUFFER_SIZE, read1.new_tail) orelse return;
        const read3 = buffer_read(&self.rx_buffer, RX_BUFFER_SIZE, read2.new_tail) orelse return;
        self.rx_tail = read3.new_tail;

        _ = read1.data; // cs_sel
        _ = read2.data; // data_l
        _ = read3.data; // data_h
    }

    /// Handle MAC operation packet.
    pub fn bridge_handle_mac_op(self: *BridgeUnit) void {
        if (!self.mac_enabled) {
            return;
        }

        const read1 = buffer_read(&self.rx_buffer, RX_BUFFER_SIZE, self.rx_tail) orelse return;
        const read2 = buffer_read(&self.rx_buffer, RX_BUFFER_SIZE, read1.new_tail) orelse return;
        self.rx_tail = read2.new_tail;

        _ = read1.data; // op
        _ = read2.data; // unit
    }

    /// Handle status request.
    pub fn bridge_handle_status(self: *BridgeUnit) void {
        const status = [4]u8{
            if (self.spi_enabled) @as(u8, 1) else @as(u8, 0),
            if (self.mac_enabled) @as(u8, 1) else @as(u8, 0),
            0,
            0,
        };

        var i: usize = 0;
        while (i < 4) : (i += 1) {
            if (self.bridge_tx_space() > 0) {
                self.tx_buffer[self.tx_head] = status[i];
                self.tx_head = (self.tx_head + 1) % TX_BUFFER_SIZE;
            }
        }
    }

    /// Handle configuration packet.
    pub fn bridge_handle_config(self: *BridgeUnit) void {
        const read_result = buffer_read(&self.rx_buffer, RX_BUFFER_SIZE, self.rx_tail) orelse return;
        self.rx_tail = read_result.new_tail;

        const cfg_byte = read_result.data;
        self.spi_enabled = (cfg_byte & 0x01) != 0;
        self.mac_enabled = (cfg_byte & 0x02) != 0;
    }
};

// =====================================================================
// Tests
// =====================================================================

test "bridge_initially_idle" {
    const bridge = BridgeUnit.init();
    try std.testing.expectEqual(BRIDGE_IDLE, bridge.state);
}

test "bridge_rx_buffers_empty" {
    const bridge = BridgeUnit.init();
    try std.testing.expectEqual(@as(usize, 0), bridge.bridge_rx_available());
}

test "bridge_tx_buffer_full_space" {
    const bridge = BridgeUnit.init();
    try std.testing.expectEqual(@as(usize, TX_BUFFER_SIZE), bridge.bridge_tx_space());
}

test "bridge_rx_write_success" {
    var bridge = BridgeUnit.init();
    const result = BridgeUnit.buffer_write(&bridge.rx_buffer, RX_BUFFER_SIZE, bridge.rx_head, 0xAA);
    try std.testing.expect(result != null);
}

test "bridge_buffer_count_empty" {
    const count = BridgeUnit.buffer_count(0, 0, RX_BUFFER_SIZE);
    try std.testing.expectEqual(@as(usize, 0), count);
}

test "bridge_buffer_count_wrap" {
    const count = BridgeUnit.buffer_count(RX_BUFFER_SIZE - 1, 0, RX_BUFFER_SIZE);
    try std.testing.expectEqual(@as(usize, RX_BUFFER_SIZE - 1), count);
}

test "bridge_buffer_count_wrap2" {
    const count = BridgeUnit.buffer_count(0, RX_BUFFER_SIZE - 1, RX_BUFFER_SIZE);
    try std.testing.expectEqual(@as(usize, 1), count);
}

test "bridge_packet_types_defined" {
    try std.testing.expectEqual(@as(u8, 0x00), PKT_UART_DATA);
    try std.testing.expectEqual(@as(u8, 0x10), PKT_SPI_XFER);
    try std.testing.expectEqual(@as(u8, 0x20), PKT_MAC_OP);
}

test "bridge_max_packet_size" {
    try std.testing.expectEqual(@as(usize, 128), MAX_PACKET_SIZE);
}

test "bridge_timeout_defined" {
    try std.testing.expectEqual(@as(u32, 10_000), PACKET_TIMEOUT);
}

test "bridge_rx_tx_buffer_sizes" {
    try std.testing.expectEqual(@as(usize, 256), RX_BUFFER_SIZE);
    try std.testing.expectEqual(@as(usize, 256), TX_BUFFER_SIZE);
}

test "bridge_spi_enabled_by_default" {
    const bridge = BridgeUnit.init();
    try std.testing.expect(bridge.spi_enabled);
    try std.testing.expect(bridge.mac_enabled);
}

test "bridge_parse_header_requires_2_bytes" {
    var bridge = BridgeUnit.init();
    // Only 1 byte available (head=1, tail=0)
    bridge.rx_head = 1;
    const result = bridge.bridge_parse_header();
    try std.testing.expect(!result);
}

test "bridge_config_enables_spi" {
    var bridge = BridgeUnit.init();
    bridge.rx_buffer[0] = 0x01;
    bridge.rx_head = 1;
    bridge.rx_tail = 0;
    bridge.bridge_handle_config();
    try std.testing.expect(bridge.spi_enabled);
}

test "bridge_config_enables_mac" {
    var bridge = BridgeUnit.init();
    bridge.rx_buffer[0] = 0x02;
    bridge.rx_head = 1;
    bridge.rx_tail = 0;
    bridge.bridge_handle_config();
    try std.testing.expect(bridge.mac_enabled);
}

test "bridge_config_disables_spi" {
    var bridge = BridgeUnit.init();
    bridge.rx_buffer[0] = 0x00;
    bridge.rx_head = 1;
    bridge.rx_tail = 0;
    bridge.bridge_handle_config();
    try std.testing.expect(!bridge.spi_enabled);
}
