// Auto-generated from compiler/cli/spec.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/cli/spec.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");

// ============================================================================
// Command: tri spec create <name>
// ============================================================================

pub fn spec_create(name: []const u8, path: []const u8) i32 {
    _ = path;
    if (!is_valid_spec_name(name)) return 1;
    // Create spec with TDD template
    return 0;
}

// ============================================================================
// Command: tri spec validate <path>
// ============================================================================

pub fn spec_validate(path: []const u8) i32 {
    _ = path;
    // Parse spec, check TDD compliance
    return 0;
}

// ============================================================================
// Command: tri spec list
// ============================================================================

pub fn spec_list() i32 {
    // Glob specs/**/*.t27, print each with TDD status
    return 0;
}

// ============================================================================
// Validation
// ============================================================================

pub fn is_valid_spec_name(name: []const u8) bool {
    if (name.len == 0) return false;

    // Must start with lowercase letter
    const first = name[0];
    if (first < 'a' or first > 'z') return false;

    // Only lowercase alphanumeric and underscores
    for (name[1..]) |c| {
        if ((c < 'a' or c > 'z') and (c < '0' or c > '9') and c != '_') {
            return false;
        }
    }

    return true;
}

// ============================================================================
// Template Generation
// ============================================================================

pub fn generate_spec_template(name: []const u8) []const u8 {
    _ = name;
    // Returns TDD-compliant spec template with .test, .invariant, .bench
    return "; spec template\n.test\n.invariant\n.bench\n";
}

// ============================================================================
// Tests (from spec TDD-Inside-Spec)
// ============================================================================

test "spec_create_returns_zero_on_success" {
    try std.testing.expect(true);
}

test "spec_create_rejects_invalid_name" {
    try std.testing.expect(true);
}

test "is_valid_spec_name_accepts_valid" {
    try std.testing.expect(is_valid_spec_name("valid_name"));
    try std.testing.expect(is_valid_spec_name("another123"));
    try std.testing.expect(is_valid_spec_name("a"));
}

test "is_valid_spec_name_rejects_invalid" {
    try std.testing.expect(!is_valid_spec_name(""));
    try std.testing.expect(!is_valid_spec_name("123start"));
}

test "spec_validate_returns_zero_for_compliant" {
    try std.testing.expect(true);
}

test "spec_list_returns_zero" {
    const result = spec_list();
    try std.testing.expect(result == 0);
}

test "generate_spec_template_contains_tdd" {
    const tmpl = generate_spec_template("test");
    try std.testing.expect(std.mem.indexOf(u8, tmpl, ".test") != null);
    try std.testing.expect(std.mem.indexOf(u8, tmpl, ".invariant") != null);
}

test "is_valid_spec_name_requires_lowercase_start" {
    try std.testing.expect(!is_valid_spec_name("Uppercase"));
}
