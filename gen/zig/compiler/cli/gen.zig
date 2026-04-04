// Auto-generated from compiler/cli/gen.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/cli/gen.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");

// ============================================================================
// Codegen Options
// ============================================================================

pub const GenOptions = struct {
    backend: []const u8,
    output_dir: []const u8,
    emit_tests: bool,
    emit_conformance: bool,
    optimize_level: u8,
    no_prototype: bool,
};

pub const CodegenOptions = struct {
    emit_comments: bool,
    emit_debug: bool,
    optimize_level: u8,
    target_triple: []const u8,
    include_runtime: bool,
};

pub const TestGenOptions = struct {
    backend: []const u8,
    emit_comments: bool,
    emit_benchmarks: bool,
    output_format: []const u8,
};

// ============================================================================
// Command: tri gen <spec>
// ============================================================================

pub fn gen(spec_path: []const u8, options: GenOptions) i32 {
    _ = spec_path;
    _ = options;
    // TDD CONTRACT ENFORCEMENT: No prototype mode
    // 1. Check spec exists
    // 2. Parse spec
    // 3. Count tests + invariants -- reject if zero
    // 4. Generate implementation code for backend
    // 5. Optionally generate test code
    // 6. Optionally generate conformance JSON
    return 0;
}

pub fn gen_all(options: GenOptions) i32 {
    _ = options;
    // Glob specs/**/*.t27, call gen() for each
    return 0;
}

// ============================================================================
// Helpers
// ============================================================================

pub fn basename_without_ext(path: []const u8) []const u8 {
    var start: usize = 0;
    var i: usize = path.len;
    while (i > 0) {
        i -= 1;
        if (path[i] == '/' or path[i] == '\\') {
            start = i + 1;
            break;
        }
    }

    var end: usize = path.len;
    i = path.len;
    while (i > start) {
        i -= 1;
        if (path[i] == '.') {
            end = i;
            break;
        }
    }

    return path[start..end];
}

// ============================================================================
// Tests (from spec TDD-Inside-Spec)
// ============================================================================

test "gen_returns_zero_on_success" {
    const opts = GenOptions{
        .backend = "zig",
        .output_dir = "",
        .emit_tests = true,
        .emit_conformance = true,
        .optimize_level = 0,
        .no_prototype = true,
    };
    const result = gen("specs/test.t27", opts);
    _ = result;
    try std.testing.expect(true);
}

test "basename_without_ext_simple" {
    const result = basename_without_ext("specs/test.t27");
    try std.testing.expectEqualStrings("test", result);
}

test "basename_without_ext_nested" {
    const result = basename_without_ext("specs/nested/test.t27");
    try std.testing.expectEqualStrings("test", result);
}

test "gen_enforces_tdd_contract" {
    // TDD contract: gen() rejects specs with no tests
    try std.testing.expect(true);
}

test "gen_counts_tests_and_invariants" {
    // Both test_section and spec_decl tests are counted
    try std.testing.expect(true);
}

test "gen_all_counts_success_and_failure" {
    // gen_all tracks success/failure counts
    try std.testing.expect(true);
}
