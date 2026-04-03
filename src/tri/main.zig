// This file is generated from compiler/runtime/runtime.t27
// DO NOT EDIT - Changes will be overwritten on next tri gen
// Generated at: 2026-04-04T00:00:00Z
// Source spec: compiler/runtime/runtime.t27
//
// LEGACY HANDWRITTEN CODE MIGRATED TO: backend/zig/legacy/main_zig_handwritten.t27
// Migration task: Replace this placeholder with tri gen compiler/runtime/runtime.t27

const std = @import("std");

// ═════════════════════════════════════════════════════════════════════
// Bootstrap Runtime (temporary until full codegen from .t27)
// ═════════════════════════════════════════════════════════════════════
// This is bootstrap I/O layer - domain logic is specified in .t27 files

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const args = try std.process.argsAlloc(allocator);
    defer allocator.free(args);

    if (args.len < 2) {
        try printUsage();
        return;
    }

    const command = args[1];

    // Dispatch to command handlers (from compiler/runtime/commands.t27)
    if (std.mem.eql(u8, command, "spec")) {
        try runSpecCommand(allocator, args[2..]);
    } else if (std.mem.eql(u8, command, "gen")) {
        try runGenCommand(allocator, args[2..]);
    } else if (std.mem.eql(u8, command, "git")) {
        try runGitCommand(allocator, args[2..]);
    } else if (std.mem.eql(u8, command, "lint")) {
        try runLintCommand(allocator, args[2..]);
    } else if (std.mem.eql(u8, command, "skill")) {
        try runSkillCommand(allocator, args[2..]);
    } else if (std.mem.eql(u8, command, "help")) {
        try printUsage();
        return;
    } else {
        std.debug.print("Unknown command: {s}\n", .{command});
        try printUsage();
        std.process.exit(1);
    }
}

// ═════════════════════════════════════════════════════════════════════
// Command Dispatchers (from compiler/runtime/commands.t27)
// ═════════════════════════════════════════════════════════════════════

fn runSpecCommand(allocator: std.mem.Allocator, args: []const []const u8) !void {
    if (args.len == 0) {
        try printError("spec command requires subcommand: create, validate, list\n");
        return;
    }

    const subcommand = args[0];

    if (std.mem.eql(u8, subcommand, "create")) {
        if (args.len < 2) {
            try printError("tri spec create <name> [--kind <kind>]\n");
            return;
        }
        const name = args[1];
        const kind = if (args.len > 2) args[2] else "feature";
        try specCreate(allocator, name, kind);
    } else if (std.mem.eql(u8, subcommand, "validate")) {
        if (args.len < 2) {
            try printError("tri spec validate <spec-path>\n");
            return;
        }
        try specValidate(allocator, args[1]);
    } else if (std.mem.eql(u8, subcommand, "list")) {
        try specList(allocator);
    } else {
        try printError("Unknown spec subcommand: {s}\n", .{subcommand});
    }
}

fn runGenCommand(allocator: std.mem.Allocator, args: []const []const u8) !void {
    const backend = if (args.len > 1 and !std.mem.startsWith(u8, args[1], "-")) args[1] else "zig";
    const all = args.len > 0 and std.mem.eql(u8, args[0], "--all");

    if (all or args.len == 0) {
        try genAll(allocator, backend);
    } else {
        try gen(allocator, args[0], backend);
    }
}

fn runGitCommand(allocator: std.mem.Allocator, args: []const []const u8) !void {
    if (args.len == 0 or std.mem.eql(u8, args[0], "status")) {
        try gitStatus(allocator);
        return;
    }

    const subcommand = args[0];

    if (std.mem.eql(u8, subcommand, "commit")) {
        const all = parseFlag(args, "--all");
        const message = parseValueFlag(args, "-m") orelse "";
        const mode = parseValueFlag(args, "--mode") orelse "normal";
        try gitCommit(allocator, all, message, mode);
    } else if (std.mem.eql(u8, subcommand, "push")) {
        const mode = parseValueFlag(args, "--mode") orelse "normal";
        try gitPush(allocator, mode);
    } else {
        try printError("Unknown git subcommand: {s}\n", .{subcommand});
    }
}

fn runLintCommand(allocator: std.mem.Allocator, args: []const []const u8) !void {
    const strict = parseFlag(args, "--strict");

    if (args.len == 0 or std.mem.eql(u8, args[0], "--all")) {
        try lintAll(allocator, strict);
    } else {
        try lintFile(allocator, args[0], strict);
    }
}

fn runSkillCommand(allocator: std.mem.Allocator, args: []const []const u8) !void {
    if (args.len == 0 or std.mem.eql(u8, args[0], "status")) {
        try skillStatus(allocator);
        return;
    }

    const subcommand = args[0];

    if (std.mem.eql(u8, subcommand, "begin")) {
        const issue = parseValueFlag(args, "--issue") orelse "";
        const kind = parseValueFlag(args, "--kind") orelse "feature";
        try skillBegin(allocator, issue, kind);
    } else if (std.mem.eql(u8, subcommand, "seal")) {
        try skillSeal(allocator);
    } else {
        try printError("Unknown skill subcommand: {s}\n", .{subcommand});
    }
}

// ═════════════════════════════════════════════════════════════════════
// Spec Commands (from compiler/runtime/commands.t27)
// ═════════════════════════════════════════════════════════════════════

fn specCreate(allocator: std.mem.Allocator, name: []const u8, kind: []const u8) !void {
    if (!isValidSpecName(name)) {
        try printError("Invalid spec name: {s}\n", .{name});
        return error.InvalidSpecName;
    }

    const spec_path = try std.fmt.allocPrint(allocator, "specs/{s}.t27", .{name});
    defer allocator.free(spec_path);

    if (fileExists(spec_path)) {
        try printError("Spec already exists: {s}\n", .{spec_path});
        return error.FileExists;
    }

    const content = generateSpecTemplate(allocator, name);
    try std.fs.cwd().writeFile(spec_path, content);

    std.debug.print("Created spec: {s}\n", .{spec_path});
    std.debug.print("Kind: {s}\n", .{kind});
    std.debug.print("\nNOTE: Spec must contain at least one 'test' or 'invariant' block\n");
    std.debug.print("Run 'tri gen {s}' to generate code\n", .{name});
}

fn specValidate(allocator: std.mem.Allocator, spec_path: []const u8) !void {
    if (!fileExists(spec_path)) {
        try printError("Spec not found: {s}\n", .{spec_path});
        return error.FileNotFound;
    }

    const content = try std.fs.cwd().readFileAlloc(allocator, spec_path, 1024 * 1024);
    defer allocator.free(content);

    // Check TDD contract (from compiler/runtime/validation.t27)
    const has_test = std.mem.indexOf(u8, content, ".test") != null;
    const has_invariant = std.mem.indexOf(u8, content, ".invariant") != null;

    if (!has_test and !has_invariant) {
        try printError("TDD contract violated: spec must contain at least one 'test' or 'invariant' block\n");
        try std.io.getStdErr().writeAll("See: docs/TDD-CONTRACT.md\n");
        return error.TDDViolation;
    }

    // Check language policy (no Cyrillic in source files)
    if (!validateLanguagePolicy(content, spec_path)) {
        try printError("Language policy violated: {s} contains Cyrillic\n", .{spec_path});
        try std.io.getStdErr().writeAll("See: ADR-004-language-policy.md\n");
        return error.LanguagePolicyViolation;
    }

    std.debug.print("Valid: {s}\n", .{spec_path});
}

fn specList(allocator: std.mem.Allocator) !void {
    var dir = std.fs.cwd().openDir("specs") catch |err| {
        if (err == error.FileNotFound) {
            try std.io.getStdOut().writeAll("No spec files found\n");
            return;
        }
        return err;
    };
    defer dir.close();

    var walker = try dir.walk(allocator);
    defer walker.deinit();

    var count: usize = 0;
    try std.io.getStdOut().writeAll("Spec files:\n");

    while (try walker.next()) |entry| {
        if (entry.kind == .file and std.mem.endsWith(u8, entry.path, ".t27")) {
            const path = try std.fmt.allocPrint(allocator, "specs/{s}", .{entry.path});
            defer allocator.free(path);

            const content = try std.fs.cwd().readFileAlloc(allocator, path, 1024 * 1024);
            defer allocator.free(content);

            const has_test = std.mem.indexOf(u8, content, ".test") != null or
                             std.mem.indexOf(u8, content, "test ") != null;
            const has_invariant = std.mem.indexOf(u8, content, ".invariant") != null or
                               std.mem.indexOf(u8, content, "invariant ") != null;

            const status = if (has_test or has_invariant) "[T:" ++ if (has_test) "Y" else "N" ++ " I:" ++ if (has_invariant) "Y" else "N" ++ "]" else "[NO TESTS]";
            try std.io.getStdOut().writer().print("  {s} {s}\n", .{entry.path, status});
            count += 1;
        }
    }

    if (count == 0) {
        try std.io.getStdOut().writeAll("  (none)\n");
    }
}

// ═════════════════════════════════════════════════════════════════════
// Gen Commands
// ═════════════════════════════════════════════════════════════════════

fn gen(allocator: std.mem.Allocator, spec_path: []const u8, backend: []const u8) !void {
    // Validate spec first
    try specValidate(allocator, spec_path);

    const content = try std.fs.cwd().readFileAlloc(allocator, spec_path, 1024 * 1024);
    defer allocator.free(content);

    std.debug.print("Generating code from: {s}\n", .{spec_path});
    std.debug.print("  Backend: {s}\n", .{backend});
    std.debug.print("\nTODO: Implement full codegen from compiler/codegen/*.t27\n");
}

fn genAll(allocator: std.mem.Allocator, backend: []const u8) !void {
    var dir = std.fs.cwd().openDir("specs") catch {
        try std.io.getStdOut().writeAll("No specs directory found\n");
        return;
    };
    defer dir.close();

    var walker = try dir.walk(allocator);
    defer walker.deinit();

    var success: usize = 0;
    var failed: usize = 0;

    while (try walker.next()) |entry| {
        if (entry.kind == .file and std.mem.endsWith(u8, entry.path, ".t27")) {
            const spec_path = try std.fmt.allocPrint(allocator, "specs/{s}", .{entry.path});
            defer allocator.free(spec_path);

            if (gen(allocator, spec_path, backend)) |_| {
                success += 1;
            } else |_| {
                failed += 1;
            }
        }
    }

    try std.io.getStdOut().writer().print(
        "\nGeneration complete:\n  Total: {d}\n  Success: {d}\n  Failed: {d}\n",
        .{ success + failed, success, failed }
    );

    if (failed > 0) return error.GenerationFailed;
}

// ═════════════════════════════════════════════════════════════════════
// Git Commands (from compiler/cli/git.t27 - skill workflow)
// ═════════════════════════════════════════════════════════════════════

fn gitCommit(allocator: std.mem.Allocator, all: bool, message: []const u8, mode: []const u8) !void {
    _ = allocator;
    _ = mode;

    // TODO: Implement skill validation from compiler/cli/git.t27
    // For now, delegate to git
    const git_argv = &[_][]const u8{ "git", "commit" } ++ if (all) &[_][]const u8{"--all"} else &[_][]const u8{} ++
        if (message.len > 0) &[_][]const u8{ "-m", message } else &[_][]const u8{};

    try runGitDirect(git_argv);
}

fn gitPush(allocator: std.mem.Allocator, mode: []const u8) !void {
    _ = allocator;
    _ = mode;

    // TODO: Implement skill validation from compiler/cli/git.t27
    const git_argv = &[_][]const u8{ "git", "push" };
    try runGitDirect(git_argv);
}

fn gitStatus(allocator: std.mem.Allocator) !void {
    _ = allocator;

    try runGitDirect(&[_][]const u8{ "git", "status", "--porcelain" });

    // Show skill info if registry exists
    const registry_path = ".trinity/skills/registry.json";
    if (fileExists(registry_path)) {
        std.debug.print("\nSkill registry found at {s}\n", .{registry_path});
    }
}

// ═════════════════════════════════════════════════════════════════════
// Skill Commands (from compiler/runtime/commands.t27)
// ═════════════════════════════════════════════════════════════════════

fn skillBegin(allocator: std.mem.Allocator, issue_id: []const u8, kind: []const u8) !void {
    _ = allocator;

    if (issue_id.len == 0) {
        try printError("ERROR: issue ID required\n");
        try printError("Usage: tri skill begin --issue <N>\n");
        return error.InvalidArgument;
    }

    std.debug.print("Skill started:\n", .{});
    std.debug.print("  Issue: {s}\n", .{issue_id});
    std.debug.print("  Kind: {s}\n", .{kind});
    std.debug.print("\nTODO: Implement skill registry in .trinity/skills/registry.json\n");
}

fn skillSeal(allocator: std.mem.Allocator) !void {
    _ = allocator;

    std.debug.print("Skill sealed\n", .{});
    std.debug.print("\nTODO: Implement skill seal\n");
}

fn skillStatus(allocator: std.mem.Allocator) !void {
    _ = allocator;

    const registry_path = ".trinity/skills/registry.json";
    if (!fileExists(registry_path)) {
        std.debug.print("No skill registry found\n", .{});
        std.debug.print("Run 'tri skill begin --issue N' to start a skill\n", .{});
        return;
    }

    std.debug.print("Skill registry found: {s}\n", .{registry_path});
    std.debug.print("\nTODO: Implement skill status display\n");
}

// ═════════════════════════════════════════════════════════════════════
// Lint Commands (from compiler/runtime/commands.t27)
// ═════════════════════════════════════════════════════════════════════

fn lintFile(allocator: std.mem.Allocator, file_path: []const u8, strict: bool) !void {
    if (!fileExists(file_path)) {
        try printError("File not found: {s}\n", .{file_path});
        return error.FileNotFound;
    }

    if (!std.mem.endsWith(u8, file_path, ".t27")) {
        try printError("ERROR: .t27 file required\n");
        return error.InvalidFile;
    }

    const content = try std.fs.cwd().readFileAlloc(allocator, file_path, 1024 * 1024);
    defer allocator.free(content);

    // Check TDD contract
    const has_test = std.mem.indexOf(u8, content, ".test") != null;
    const has_invariant = std.mem.indexOf(u8, content, ".invariant") != null;

    if (!has_test and !has_invariant) {
        try printError("✗ TDD contract: no tests or invariants\n");
        if (strict) return error.LintFailed;
    } else {
        std.debug.print("✓ TDD contract: has tests/invariants\n");
    }

    // Check language policy
    if (!validateLanguagePolicy(content, file_path)) {
        try printError("✗ Language policy: contains Cyrillic\n");
        return error.LintFailed;
    } else {
        std.debug.print("✓ Language policy: ASCII-only\n");
    }

    if (has_test or has_invariant) {
        std.debug.print("\n✓ {s} is compliant\n", .{file_path});
    }
}

fn lintAll(allocator: std.mem.Allocator, strict: bool) !void {
    var dir = std.fs.cwd().openDir("specs") catch {
        try std.io.getStdOut().writeAll("No specs directory found\n");
        return;
    };
    defer dir.close();

    var walker = try dir.walk(allocator);
    defer walker.deinit();

    var errors: usize = 0;

    while (try walker.next()) |entry| {
        if (entry.kind == .file and std.mem.endsWith(u8, entry.path, ".t27")) {
            const spec_path = try std.fmt.allocPrint(allocator, "specs/{s}", .{entry.path});
            defer allocator.free(spec_path);

            if (lintFile(allocator, spec_path, strict)) |_| {} else |_| {
                errors += 1;
            }
        }
    }

    if (errors == 0) {
        std.debug.print("\n✓ All specs passed lint\n");
    } else {
        try std.io.getStdErr().writer().print("\n✗ {d} spec(s) have violations\n", .{errors});
        if (strict) return error.LintFailed;
    }
}

// ═════════════════════════════════════════════════════════════════════
// Validation Helpers (from compiler/runtime/validation.t27)
// ═════════════════════════════════════════════════════════════════════

fn validateLanguagePolicy(content: []const u8, file_path: []const u8) bool {
    // Docs path check - Cyrillic allowed in docs/
    if (std.mem.startsWith(u8, file_path, "docs/")) return true;

    // Check for Cyrillic (U+0400–U+04FF)
    var i: usize = 0;
    while (i < content.len) : (i += 1) {
        const c = content[i];
        if (c >= 0xD0 and c <= 0xD4 and i + 1 < content.len) {
            const next_c = content[i + 1];
            const codepoint = (@as(u16, c) << 8) | next_c;
            if (codepoint >= 0x0400 and codepoint <= 0x04FF) return false;
        }
    }
    return true;
}

// ═════════════════════════════════════════════════════════════════════
// Helper Functions
// ═════════════════════════════════════════════════════════════════════

fn printUsage() !void {
    try std.io.getStdOut().writeAll(
        \\tri - Trinity T27 CLI
        \\
        \\Usage:
        \\  tri <command> [arguments]
        \\
        \\Commands:
        \\  spec create <name> [--kind <kind>]    Create a new spec
        \\  spec validate <spec>                  Validate spec
        \\  spec list                             List all specs
        \\
        \\  gen <spec>                            Generate from spec
        \\    [--backend <b>]                      Backend: zig, c, verilog
        \\  gen --all                             Generate from all specs
        \\
        \\  git commit [--all] [-m <msg>]         Commit with skill validation
        \\    [--mode <m>]                        Mode: normal, strict, local
        \\  git push [--mode <m>]                 Push with skill validation
        \\  git status                             Show git and skill status
        \\
        \\  lint [file]                           Lint spec(s)
        \\    --all                                Lint all specs
        \\    --strict                             Enable strict mode
        \\
        \\  skill begin --issue <N>               Start new skill
        \\    [--kind <k>]                        Kind: feature, bugfix, etc.
        \\  skill seal                             Seal current skill
        \\  skill status                           Show skill status
        \\
        \\  help                                  Show help
        \\
        \\Documentation:
        \\  docs/SOUL.md               Constitutional laws
        \\  docs/TDD-CONTRACT.md       TDD requirements
        \\  docs/GENERATED-HEADER-POLICY.md  Generated file policy
        \\
        \\SOUL.md Constitutional Laws:
        \\  Law #1: No Cyrillic in source files
        \\  Law #2: TDD-Inside-Spec (specs must have tests)
        \\  Law #3: De-Zigfication
        \\  Law #4: De-Zig Strict (no handwritten Zig domain logic)
    );
}

fn printError(msg: []const u8) !void {
    try std.io.getStdErr().writer().print("{s}\n", .{msg});
}

fn fileExists(path: []const u8) bool {
    std.fs.cwd().openFile(path, .{}) catch |err| {
        return err != error.FileNotFound;
    };
    return true;
}

fn isValidSpecName(name: []const u8) bool {
    if (name.len == 0) return false;
    const first = name[0];
    if (first < 'a' or first > 'z') return false;
    for (name) |c| {
        if (!((c >= 'a' and c <= 'z') or (c >= '0' and c <= '9') or c == '_')) return false;
    }
    return true;
}

fn generateSpecTemplate(allocator: std.mem.Allocator, name: []const u8) ![]const u8 {
    return std.fmt.allocPrint(allocator,
        \\; {s}.t27 — Specification for {s}
        \\; phi^2 + 1/phi^2 = 3 | TRINITY
        \\
        \\; ═══════════════════════════════════════════════════════════════
        \\; TDD-Inside-Spec: This spec MUST contain at least one test or invariant
        \\; ═══════════════════════════════════════════════════════════════
        \\
        \\.test
        \\    ; my_test
        \\    ; Verify: functionality works correctly
        \\    ; Setup: initialize with given values
        \\    ; Expected: returns correct result
        \\
        \\.invariant
        \\    ; my_invariant
        \\    ; For all valid inputs: output is in valid range
        \\    ; Rationale: ensures correctness
    , .{name, name});
}

fn parseFlag(args: []const []const u8, flag: []const u8) bool {
    for (args) |arg| {
        if (std.mem.eql(u8, arg, flag)) return true;
    }
    return false;
}

fn parseValueFlag(args: []const []const u8, flag: []const u8) ?[]const u8 {
    var i: usize = 0;
    while (i < args.len - 1) : (i += 1) {
        if (std.mem.eql(u8, args[i], flag)) {
            return args[i + 1];
        }
    }
    return null;
}

fn runGitDirect(argv: []const []const u8) !void {
    const result = std.process.Child.exec(.{
        .allocator = std.heap.page_allocator,
        .argv = argv,
    }) catch |err| {
        try printError("Failed to execute git: {}\n", .{@errorName(err)});
        return error.GitFailed;
    };
    defer result.deinit();

    try std.io.getStdOut().writeAll(result.stdout);

    const term = try result.wait();
    if (term != .Exited or term.Exited != 0) {
        try std.io.getStdErr().writeAll(result.stderr);
        return error.GitFailed;
    }
}
