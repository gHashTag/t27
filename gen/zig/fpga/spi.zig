// Auto-generated from specs/fpga/spi.t27
// DO NOT EDIT -- regenerate with: tri gen specs/fpga/spi.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 43 | Module: SPI_Master

const std = @import("std");

// =====================================================================
// 1. SPI Configuration
// =====================================================================

pub const CLK_FREQ: u32 = 50_000_000; // 50 MHz

// SPI Mode 0: CPOL=0, CPHA=0
// CPOL (Clock Polarity): 0 = SCK idle low
// CPHA (Clock Phase): 0 = Sample on first (rising) edge
pub const SPI_CPOL: u8 = 0;
pub const SPI_CPHA: u8 = 0;

pub const MAX_DATA_WIDTH: u8 = 32; // Max bits per transfer
pub const CS_ASSERT_DELAY: u32 = 100; // CS to SCK delay (ns)
pub const CS_DEASSERT_DELAY: u32 = 100; // SCK to CS delay (ns)

// SPI prescaler values (divides system clock)
pub const PRESCALER_2: u8 = 0;
pub const PRESCALER_4: u8 = 1;
pub const PRESCALER_8: u8 = 2;
pub const PRESCALER_16: u8 = 3;
pub const PRESCALER_32: u8 = 4;
pub const PRESCALER_64: u8 = 5;
pub const PRESCALER_128: u8 = 6;
pub const PRESCALER_256: u8 = 7;

// =====================================================================
// 2. SPI State Machine
// =====================================================================

pub const SPI_IDLE: u8 = 0;
pub const SPI_CS_ASSERT: u8 = 1;
pub const SPI_TRANSFER: u8 = 2;
pub const SPI_CS_DEASSERT: u8 = 3;

// Transfer states
pub const TX_BIT: u8 = 0;
pub const RX_BIT: u8 = 1;
pub const WAIT_EDGE: u8 = 2;

// =====================================================================
// 3. SPI Master Unit
// =====================================================================

pub const SPIMasterUnit = struct {
    state: u8,
    tx_state: u8,
    cs_asserted: bool,
    busy: bool,

    // Transfer configuration
    prescaler: u8,
    data_width: u8,
    cs_mode: u8,

    // Data registers
    tx_data: u32,
    rx_data: u32,
    bit_count: u8,
    bit_counter: u32,

    // CS delay counters
    cs_assert_cnt: u32,
    cs_deassert_cnt: u32,

    pub fn init() SPIMasterUnit {
        return SPIMasterUnit{
            .state = SPI_IDLE,
            .tx_state = TX_BIT,
            .cs_asserted = false,
            .busy = false,
            .prescaler = PRESCALER_16,
            .data_width = 8,
            .cs_mode = 0,
            .tx_data = 0,
            .rx_data = 0,
            .bit_count = 0,
            .bit_counter = 0,
            .cs_assert_cnt = 0,
            .cs_deassert_cnt = 0,
        };
    }

    // =================================================================
    // 4. Prescaler and Configuration
    // =================================================================

    /// Set SPI clock prescaler. Returns true on success.
    pub fn spi_set_prescaler(self: *SPIMasterUnit, psc: u8) bool {
        if (psc > PRESCALER_256) {
            return false;
        }
        self.prescaler = psc;
        return true;
    }

    /// Get actual prescaler divider value.
    pub fn spi_get_prescaler_div(self: *const SPIMasterUnit) u32 {
        return switch (self.prescaler) {
            PRESCALER_2 => 2,
            PRESCALER_4 => 4,
            PRESCALER_8 => 8,
            PRESCALER_16 => 16,
            PRESCALER_32 => 32,
            PRESCALER_64 => 64,
            PRESCALER_128 => 128,
            PRESCALER_256 => 256,
            else => 16,
        };
    }

    /// Get SPI SCK frequency.
    pub fn spi_get_sck_freq(self: *const SPIMasterUnit) u32 {
        return CLK_FREQ / self.spi_get_prescaler_div();
    }

    /// Set data width (1-32 bits). Returns true on success.
    pub fn spi_set_data_width(self: *SPIMasterUnit, width: u8) bool {
        if (width == 0 or width > MAX_DATA_WIDTH) {
            return false;
        }
        self.data_width = width;
        return true;
    }

    // =================================================================
    // 5. Transfer Control
    // =================================================================

    /// Check if SPI is busy.
    pub fn spi_is_busy(self: *const SPIMasterUnit) bool {
        return self.busy;
    }

    /// Start SPI transfer. Returns true on success.
    pub fn spi_transfer(self: *SPIMasterUnit, data: u32) bool {
        if (self.busy) {
            return false;
        }
        self.tx_data = data;
        self.rx_data = 0;
        self.bit_count = 0;
        self.bit_counter = 0;
        self.state = SPI_CS_ASSERT;
        self.busy = true;
        return true;
    }

    /// Read received data (lower bits only).
    pub fn spi_read_rx(self: *const SPIMasterUnit) u32 {
        const mask = (@as(u32, 1) << @intCast(self.data_width)) - 1;
        return self.rx_data & mask;
    }

    // =================================================================
    // 6. Line State Queries
    // =================================================================

    /// Get CS line state.
    pub fn spi_get_cs(self: *const SPIMasterUnit) bool {
        return self.cs_asserted;
    }

    /// Get SCK line state (Mode 0: idle low).
    pub fn spi_get_sck(self: *const SPIMasterUnit) bool {
        return switch (self.tx_state) {
            TX_BIT => false, // SCK low (setup)
            RX_BIT => true, // SCK high (sample)
            else => SPI_CPOL == 0,
        };
    }

    /// Get MOSI line state.
    pub fn spi_get_mosi(self: *const SPIMasterUnit) bool {
        if (!self.busy or self.state != SPI_TRANSFER) {
            return false; // Idle: MOSI low
        }
        const shift: u5 = @intCast(self.data_width - self.bit_count - 1);
        return (self.tx_data >> shift) & 1 == 1;
    }

    // =================================================================
    // 7. Tick / State Machine
    // =================================================================

    /// Process one system clock cycle.
    pub fn spi_tick(self: *SPIMasterUnit) void {
        switch (self.state) {
            SPI_IDLE => {
                // Waiting for transfer
            },
            SPI_CS_ASSERT => {
                self.cs_assert_cnt += 1;
                const delay_cycles = CS_ASSERT_DELAY * CLK_FREQ / 1_000_000_000;
                if (self.cs_assert_cnt >= delay_cycles) {
                    self.cs_assert_cnt = 0;
                    self.cs_asserted = true;
                    self.state = SPI_TRANSFER;
                    self.tx_state = TX_BIT;
                }
            },
            SPI_TRANSFER => {
                self.spi_transfer_bit();
            },
            SPI_CS_DEASSERT => {
                self.cs_deassert_cnt += 1;
                const delay_cycles = CS_DEASSERT_DELAY * CLK_FREQ / 1_000_000_000;
                if (self.cs_deassert_cnt >= delay_cycles) {
                    self.cs_deassert_cnt = 0;
                    self.cs_asserted = false;
                    self.state = SPI_IDLE;
                    self.busy = false;
                }
            },
            else => {},
        }
    }

    /// Transfer single bit (internal).
    pub fn spi_transfer_bit(self: *SPIMasterUnit) void {
        const prescaler_div = self.spi_get_prescaler_div();
        self.bit_counter += 1;

        switch (self.tx_state) {
            TX_BIT => {
                if (self.bit_counter >= prescaler_div / 2) {
                    self.bit_counter = 0;
                    self.tx_state = RX_BIT;
                }
            },
            RX_BIT => {
                if (self.bit_counter >= prescaler_div / 2) {
                    // Sample MISO (simulated as 0 in software model)
                    const miso_bit: u32 = 0;
                    self.rx_data = (self.rx_data << 1) | miso_bit;
                    self.bit_count += 1;
                    self.bit_counter = 0;

                    if (self.bit_count >= self.data_width) {
                        self.tx_state = WAIT_EDGE;
                    } else {
                        self.tx_state = TX_BIT;
                    }
                }
            },
            WAIT_EDGE => {
                if (self.bit_counter >= prescaler_div / 2) {
                    self.bit_counter = 0;
                    self.state = SPI_CS_DEASSERT;
                }
            },
            else => {},
        }
    }
};

// =====================================================================
// Tests
// =====================================================================

test "spi_mode_0_configuration" {
    try std.testing.expectEqual(@as(u8, 0), SPI_CPOL);
    try std.testing.expectEqual(@as(u8, 0), SPI_CPHA);
}

test "spi_prescaler_16_default" {
    const spi = SPIMasterUnit.init();
    try std.testing.expectEqual(PRESCALER_16, spi.prescaler);
}

test "spi_set_prescaler_valid" {
    var spi = SPIMasterUnit.init();
    try std.testing.expect(spi.spi_set_prescaler(PRESCALER_64));
}

test "spi_set_prescaler_invalid" {
    var spi = SPIMasterUnit.init();
    try std.testing.expect(!spi.spi_set_prescaler(99));
}

test "spi_prescaler_div_16" {
    const spi = SPIMasterUnit.init();
    try std.testing.expectEqual(@as(u32, 16), spi.spi_get_prescaler_div());
}

test "spi_sck_freq_at_50MHz" {
    const spi = SPIMasterUnit.init();
    const div = spi.spi_get_prescaler_div();
    const freq = spi.spi_get_sck_freq();
    try std.testing.expectEqual(CLK_FREQ / div, freq);
}

test "spi_set_data_width_8" {
    var spi = SPIMasterUnit.init();
    try std.testing.expect(spi.spi_set_data_width(8));
}

test "spi_set_data_width_32" {
    var spi = SPIMasterUnit.init();
    try std.testing.expect(spi.spi_set_data_width(32));
}

test "spi_set_data_width_invalid" {
    var spi = SPIMasterUnit.init();
    try std.testing.expect(!spi.spi_set_data_width(0));
}

test "spi_initially_not_busy" {
    const spi = SPIMasterUnit.init();
    try std.testing.expect(!spi.spi_is_busy());
}

test "spi_transfer_when_ready" {
    var spi = SPIMasterUnit.init();
    try std.testing.expect(spi.spi_transfer(0xAA));
}

test "spi_transfer_when_busy" {
    var spi = SPIMasterUnit.init();
    _ = spi.spi_transfer(0x55);
    try std.testing.expect(!spi.spi_transfer(0xAA));
}

test "spi_cs_idle_high" {
    const spi = SPIMasterUnit.init();
    try std.testing.expect(!spi.spi_get_cs());
}

test "spi_sck_idle_low" {
    const spi = SPIMasterUnit.init();
    try std.testing.expect(!spi.spi_get_sck());
}

test "spi_max_data_width_32" {
    try std.testing.expectEqual(@as(u8, 32), MAX_DATA_WIDTH);
}

test "spi_prescaler_range" {
    try std.testing.expectEqual(@as(u8, 0), PRESCALER_2);
    try std.testing.expectEqual(@as(u8, 7), PRESCALER_256);
}

test "spi_cs_delays_defined" {
    try std.testing.expectEqual(@as(u32, 100), CS_ASSERT_DELAY);
    try std.testing.expectEqual(@as(u32, 100), CS_DEASSERT_DELAY);
}
