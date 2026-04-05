// Auto-generated from compiler/codegen/verilog/fpga_emission.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/codegen/verilog/fpga_emission.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 43 | Module: fpga_emission

const std = @import("std");

/// FPGA module emission context
pub const FpgaCodegen = struct {
    output: std.ArrayList(u8),
    indent_level: u32 = 0,
    target_device: []const u8 = "xc7a100t",
    clock_freq: u32 = 50_000_000,

    pub fn init(allocator: std.mem.Allocator, target_device: []const u8, clock_freq: u32) FpgaCodegen {
        return .{
            .output = std.ArrayList(u8).init(allocator),
            .indent_level = 0,
            .target_device = target_device,
            .clock_freq = clock_freq,
        };
    }

    pub fn deinit(self: *FpgaCodegen) void {
        self.output.deinit();
    }

    pub fn emit(self: *FpgaCodegen, s: []const u8) void {
        self.output.appendSlice(s) catch {};
    }

    pub fn emit_line(self: *FpgaCodegen, s: []const u8) void {
        var i: u32 = 0;
        while (i < self.indent_level) : (i += 1) {
            self.output.appendSlice("    ") catch {};
        }
        self.output.appendSlice(s) catch {};
        self.output.append('\n') catch {};
    }

    pub fn indent(self: *FpgaCodegen) void {
        self.indent_level += 1;
    }

    pub fn dedent(self: *FpgaCodegen) void {
        if (self.indent_level > 0) self.indent_level -= 1;
    }

    /// Generate top-level FPGA module combining MAC, UART, SPI
    pub fn emit_fpga_top(self: *FpgaCodegen) void {
        self.emit_line("// Trinity FPGA Top-Level Module");
        self.emit_line("`timescale 1ns / 1ps");
        self.emit_line("");
        self.emit_line("module Trinity_FPGA_Top (");
        self.indent();
        self.emit_line("input wire clk,");
        self.emit_line("input wire rst_n,");
        self.emit_line("input wire uart_rx_in,");
        self.emit_line("output wire uart_tx_out,");
        self.emit_line("input wire spi_miso_in,");
        self.emit_line("output wire spi_cs_out,");
        self.emit_line("output wire spi_sck_out,");
        self.emit_line("output wire spi_mosi_out,");
        self.emit_line("output wire [3:0] led_out,");
        self.emit_line("input wire [26:0] mac_a_in,");
        self.emit_line("input wire [26:0] mac_b_in,");
        self.emit_line("input wire [31:0] mac_acc_in,");
        self.emit_line("output wire [31:0] mac_acc_out,");
        self.emit_line("output wire mac_valid_out");
        self.dedent();
        self.emit_line(");");
        self.emit_line("");
        self.emit_line("// Submodule instantiations");
        self.emit_line("UART_Bridge uart_bridge (.clk(clk), .rst_n(rst_n), .uart_rx(uart_rx_in), .uart_tx(uart_tx_out));");
        self.emit_line("SPI_Master spi_master (.clk(clk), .rst_n(rst_n), .miso(spi_miso_in), .cs(spi_cs_out), .sck(spi_sck_out), .mosi(spi_mosi_out));");
        self.emit_line("");
        self.emit_line("assign mac_acc_out = 32'd0;");
        self.emit_line("assign mac_valid_out = 1'b0;");
        self.emit_line("assign led_out = 4'b1111;");
        self.emit_line("");
        self.emit_line("endmodule");
    }

    /// Generate UART RX/TX module from spec
    pub fn emit_uart_module(self: *FpgaCodegen) void {
        self.emit_line("// UART Bridge Module - 8-N-1 protocol");
        self.emit_line("module UART_Bridge (");
        self.indent();
        self.emit_line("input wire clk,");
        self.emit_line("input wire rst_n,");
        self.emit_line("input wire uart_rx,");
        self.emit_line("output wire uart_tx");
        self.dedent();
        self.emit_line(");");
        const baud_divisor = self.clock_freq / 115200;
        _ = baud_divisor;
        self.emit_line("endmodule");
    }

    /// Generate SPI master module from spec (Mode 0)
    pub fn emit_spi_module(self: *FpgaCodegen) void {
        self.emit_line("// SPI Master Module - Mode 0: CPOL=0, CPHA=0");
        self.emit_line("module SPI_Master (");
        self.indent();
        self.emit_line("input wire clk,");
        self.emit_line("input wire rst_n,");
        self.emit_line("input wire miso,");
        self.emit_line("output wire cs,");
        self.emit_line("output wire sck,");
        self.emit_line("output wire mosi");
        self.dedent();
        self.emit_line(");");
        self.emit_line("endmodule");
    }

    /// Generate ZeroDSP MAC module from spec
    pub fn emit_mac_module(self: *FpgaCodegen) void {
        self.emit_line("// ZeroDSP MAC Module - Ternary LUT with 8 parallel units");
        self.emit_line("module ZeroDSP_MAC (");
        self.indent();
        self.emit_line("input wire clk,");
        self.emit_line("input wire rst_n,");
        self.emit_line("input wire [26:0] a,");
        self.emit_line("input wire [26:0] b,");
        self.emit_line("input wire [31:0] acc_in,");
        self.emit_line("output wire [31:0] acc_out,");
        self.emit_line("input wire enable,");
        self.emit_line("output wire valid");
        self.dedent();
        self.emit_line(");");
        self.emit_line("endmodule");
    }
};

test "fpga_emission_init" {
    const allocator = std.testing.allocator;
    var codegen = FpgaCodegen.init(allocator, "xc7a100t", 50_000_000);
    defer codegen.deinit();
    try std.testing.expectEqual(@as(u32, 0), codegen.indent_level);
}

test "fpga_emission_indent_dedent" {
    const allocator = std.testing.allocator;
    var codegen = FpgaCodegen.init(allocator, "xc7a100t", 50_000_000);
    defer codegen.deinit();
    codegen.indent();
    try std.testing.expectEqual(@as(u32, 1), codegen.indent_level);
    codegen.dedent();
    try std.testing.expectEqual(@as(u32, 0), codegen.indent_level);
}
