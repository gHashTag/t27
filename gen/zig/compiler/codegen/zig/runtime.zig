// Auto-generated from compiler/codegen/zig/runtime.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/codegen/zig/runtime.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");

// ============================================================================
// Zig Runtime Generation
// ============================================================================

pub fn generate_main() []const u8 {
    return
        \\// Generated from compiler/runtime/runtime.t27
        \\const std = @import("std");
        \\const commands = @import("commands.zig");
        \\
        \\pub fn main() !u8 {
        \\    const allocator = std.heap.page_allocator;
        \\    const args = try std.process.argsAlloc(allocator);
        \\    defer std.process.argsFree(allocator, args);
        \\    if (args.len < 2) { try commands.help(""); return 0; }
        \\    return dispatch_command(args[1], args[2..]);
        \\}
    ;
}

pub fn generate_commands_module() []const u8 {
    return
        \\// Generated from compiler/runtime/commands.t27
        \\const std = @import("std");
        \\
        \\pub fn spec_dispatch(args: [][]const u8) !u8 { _ = args; return 0; }
        \\pub fn gen_dispatch(args: [][]const u8) !u8 { _ = args; return 0; }
        \\pub fn git_dispatch(args: [][]const u8) !u8 { _ = args; return 0; }
        \\pub fn lint_dispatch(args: [][]const u8) !u8 { _ = args; return 0; }
        \\pub fn skill_dispatch(args: [][]const u8) !u8 { _ = args; return 0; }
        \\pub fn help(topic: []const u8) !void { _ = topic; }
    ;
}

pub fn generate_validation_module() []const u8 {
    return
        \\// Generated from compiler/runtime/validation.t27
        \\const std = @import("std");
        \\
        \\pub fn validate_tdd(spec_path: []const u8) bool { _ = spec_path; return true; }
        \\pub fn validate_language(spec_path: []const u8) bool { _ = spec_path; return true; }
    ;
}

// ============================================================================
// Tests (from spec TDD-Inside-Spec)
// ============================================================================

test "generate_main_contains_import" {
    const main = generate_main();
    try std.testing.expect(std.mem.indexOf(u8, main, "@import") != null);
}

test "generate_commands_contains_dispatch" {
    const cmds = generate_commands_module();
    try std.testing.expect(std.mem.indexOf(u8, cmds, "spec_dispatch") != null);
    try std.testing.expect(std.mem.indexOf(u8, cmds, "gen_dispatch") != null);
}

test "generate_validation_contains_tdd" {
    const val = generate_validation_module();
    try std.testing.expect(std.mem.indexOf(u8, val, "validate_tdd") != null);
}
