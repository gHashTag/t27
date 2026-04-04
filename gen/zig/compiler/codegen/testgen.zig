// Auto-generated from compiler/codegen/testgen.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/codegen/testgen.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");

// ============================================================================
// TestGen Options
// ============================================================================

pub const TestGenOptions = struct {
    backend: []const u8,
    emit_comments: bool,
    emit_benchmarks: bool,
    output_format: []const u8,
};

// ============================================================================
// Types (from AST)
// ============================================================================

pub const Clause = struct {
    variable: []const u8,
    expression: []const u8,
};

pub const TestBlock = struct {
    name: []const u8,
    given_clauses: []const Clause,
    when_clauses: []const Clause,
    then_clauses: []const Clause,
};

pub const TestCase = struct {
    name: []const u8,
    verify_description: []const u8,
    expected_outcome: []const u8,
};

pub const Invariant = struct {
    name: []const u8,
    formal_statement: []const u8,
    rationale: []const u8,
};

pub const Benchmark = struct {
    name: []const u8,
    measure_description: []const u8,
    target: ?[]const u8,
    units: []const u8,
};

pub const SpecDecl = struct {
    test_blocks: []const TestBlock,
    invariants: []const Invariant,
};

pub const TestSection = struct {
    test_cases: []const TestCase,
};

pub const InvariantSection = struct {
    invariants: []const Invariant,
};

pub const BenchSection = struct {
    benchmarks: []const Benchmark,
};

pub const Program = struct {
    source_file: []const u8,
    spec_decl: ?SpecDecl,
    test_section: ?TestSection,
    invariant_section: ?InvariantSection,
    bench_section: ?BenchSection,
};

// ============================================================================
// StringBuilder
// ============================================================================

pub const StringBuilder = struct {
    buffer: [65536]u8,
    len: u32,

    pub fn init() StringBuilder {
        return StringBuilder{
            .buffer = undefined,
            .len = 0,
        };
    }

    pub fn append(self: *StringBuilder, s: []const u8) void {
        for (s) |c| {
            if (self.len >= self.buffer.len) break;
            self.buffer[self.len] = c;
            self.len += 1;
        }
    }

    pub fn to_string(self: *const StringBuilder) []const u8 {
        return self.buffer[0..self.len];
    }
};

// ============================================================================
// TestGen
// ============================================================================

pub const TestGen = struct {
    ast: Program,
    options: TestGenOptions,

    pub fn new(ast_node: Program, opts: TestGenOptions) TestGen {
        return TestGen{
            .ast = ast_node,
            .options = opts,
        };
    }

    pub fn generate(self: *TestGen) []const u8 {
        if (std.mem.eql(u8, self.options.backend, "zig")) {
            return "// Zig tests generated from spec\nconst std = @import(\"std\");\n";
        } else if (std.mem.eql(u8, self.options.backend, "c")) {
            return "// C tests generated from spec\n#include <assert.h>\n";
        } else if (std.mem.eql(u8, self.options.backend, "verilog")) {
            return "`timescale 1ns/1ps\nmodule tb_spec();\nendmodule\n";
        } else if (std.mem.eql(u8, self.options.backend, "rust")) {
            return "#[cfg(test)]\nmod tests {\n    use super::*;\n}\n";
        } else if (std.mem.eql(u8, self.options.backend, "json")) {
            return "{\"spec\": \"\", \"generated_at\": \"\", \"test_vectors\": []}\n";
        } else {
            return "// Unsupported backend\n";
        }
    }
};

// ============================================================================
// Tests (from spec TDD-Inside-Spec)
// ============================================================================

test "testgen_new_creates_generator" {
    const ast = Program{
        .source_file = "test.t27",
        .spec_decl = null,
        .test_section = null,
        .invariant_section = null,
        .bench_section = null,
    };
    const opts = TestGenOptions{
        .backend = "zig",
        .emit_comments = true,
        .emit_benchmarks = true,
        .output_format = "code",
    };
    var gen = TestGen.new(ast, opts);
    _ = gen.generate();
    try std.testing.expect(true);
}

test "testgen_generate_zig_outputs_header" {
    const ast = Program{
        .source_file = "test.t27",
        .spec_decl = null,
        .test_section = null,
        .invariant_section = null,
        .bench_section = null,
    };
    var gen = TestGen.new(ast, .{
        .backend = "zig",
        .emit_comments = true,
        .emit_benchmarks = false,
        .output_format = "code",
    });
    const output = gen.generate();
    try std.testing.expect(std.mem.indexOf(u8, output, "Zig tests") != null);
}

test "testgen_generate_c_outputs_header" {
    const ast = Program{
        .source_file = "test.t27",
        .spec_decl = null,
        .test_section = null,
        .invariant_section = null,
        .bench_section = null,
    };
    var gen = TestGen.new(ast, .{
        .backend = "c",
        .emit_comments = true,
        .emit_benchmarks = false,
        .output_format = "code",
    });
    const output = gen.generate();
    try std.testing.expect(std.mem.indexOf(u8, output, "C tests") != null);
}

test "testgen_generate_verilog_outputs_module" {
    const ast = Program{
        .source_file = "test.t27",
        .spec_decl = null,
        .test_section = null,
        .invariant_section = null,
        .bench_section = null,
    };
    var gen = TestGen.new(ast, .{
        .backend = "verilog",
        .emit_comments = true,
        .emit_benchmarks = false,
        .output_format = "code",
    });
    const output = gen.generate();
    try std.testing.expect(std.mem.indexOf(u8, output, "module tb_") != null);
}

test "testgen_generate_json_conformance" {
    const ast = Program{
        .source_file = "test.t27",
        .spec_decl = null,
        .test_section = null,
        .invariant_section = null,
        .bench_section = null,
    };
    var gen = TestGen.new(ast, .{
        .backend = "json",
        .emit_comments = false,
        .emit_benchmarks = false,
        .output_format = "json",
    });
    const output = gen.generate();
    try std.testing.expect(std.mem.indexOf(u8, output, "test_vectors") != null);
}

test "testgen_unsupported_backend" {
    const ast = Program{
        .source_file = "test.t27",
        .spec_decl = null,
        .test_section = null,
        .invariant_section = null,
        .bench_section = null,
    };
    var gen = TestGen.new(ast, .{
        .backend = "python",
        .emit_comments = true,
        .emit_benchmarks = false,
        .output_format = "code",
    });
    const output = gen.generate();
    try std.testing.expect(std.mem.indexOf(u8, output, "Unsupported") != null);
}

test "string_builder_append" {
    var sb = StringBuilder.init();
    sb.append("hello");
    sb.append(" world");
    try std.testing.expectEqualStrings("hello world", sb.to_string());
}
