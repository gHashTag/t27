// Auto-generated from compiler/codegen/verilog/codegen.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/codegen/verilog/codegen.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");

// ============================================================================
// Verilog Codegen Options
// ============================================================================

pub const VerilogCodegenOptions = struct {
    target_device: []const u8,
    clock_freq_hz: u32,
    include_testbench: bool,
    include_toplevel: bool,
};

// ============================================================================
// Verilog Code Generator
// ============================================================================

pub const VerilogCodegen = struct {
    indent_level: u32,
    pc_width: u8,
    addr_width: u8,
    data_width: u8,
    options: VerilogCodegenOptions,

    pub fn new(opts: VerilogCodegenOptions) VerilogCodegen {
        return VerilogCodegen{
            .indent_level = 0,
            .pc_width = 8,
            .addr_width = 12,
            .data_width = 32,
            .options = opts,
        };
    }

    pub fn calculate_width(count: u32) u8 {
        if (count <= 1) return 1;
        var w: u8 = 0;
        var n = count - 1;
        while (n > 0) : (n >>= 1) {
            w += 1;
        }
        return w;
    }
};

// ============================================================================
// Tests (from spec TDD-Inside-Spec)
// ============================================================================

test "verilog_codegen_new" {
    const gen = VerilogCodegen.new(.{
        .target_device = "XC7A100T",
        .clock_freq_hz = 100_000_000,
        .include_testbench = true,
        .include_toplevel = true,
    });
    try std.testing.expect(gen.data_width == 32);
    try std.testing.expect(gen.addr_width == 12);
}

test "verilog_calculate_width" {
    try std.testing.expect(VerilogCodegen.calculate_width(1) == 1);
    try std.testing.expect(VerilogCodegen.calculate_width(2) == 1);
    try std.testing.expect(VerilogCodegen.calculate_width(4) == 2);
    try std.testing.expect(VerilogCodegen.calculate_width(256) == 8);
}

test "verilog_default_widths" {
    const gen = VerilogCodegen.new(.{
        .target_device = "",
        .clock_freq_hz = 0,
        .include_testbench = false,
        .include_toplevel = false,
    });
    try std.testing.expect(gen.pc_width == 8);
    try std.testing.expect(gen.addr_width == 12);
    try std.testing.expect(gen.data_width == 32);
}
