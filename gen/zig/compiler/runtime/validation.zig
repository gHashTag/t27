// Auto-generated from compiler/runtime/validation.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/runtime/validation.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");

// ============================================================================
// Validation Result
// ============================================================================

pub const ValidationResult = struct {
    valid: bool,
    error_msg: []const u8,
    hint: []const u8,
};

// ============================================================================
// Validation Rules
// ============================================================================

pub fn validate_tdd_contract(has_tests: bool, has_invariants: bool) ValidationResult {
    if (!has_tests and !has_invariants) {
        return ValidationResult{
            .valid = false,
            .error_msg = "TDD contract violated: spec must contain at least one test or invariant block",
            .hint = "See: docs/TDD-CONTRACT.md",
        };
    }
    return ValidationResult{ .valid = true, .error_msg = "", .hint = "" };
}

pub fn validate_language_policy(content: []const u8, is_docs: bool) ValidationResult {
    if (is_docs) {
        return ValidationResult{ .valid = true, .error_msg = "", .hint = "" };
    }
    // Check for non-ASCII characters
    for (content) |c| {
        if (c > 127) {
            return ValidationResult{
                .valid = false,
                .error_msg = "Language policy violated: source file contains non-ASCII characters",
                .hint = "Source files must be ASCII-only. See: ADR-004-language-policy.md",
            };
        }
    }
    return ValidationResult{ .valid = true, .error_msg = "", .hint = "" };
}

pub fn validate_naming_convention(name: []const u8) ValidationResult {
    if (name.len == 0) {
        return ValidationResult{
            .valid = false,
            .error_msg = "Name cannot be empty",
            .hint = "Use snake_case for identifiers",
        };
    }
    return ValidationResult{ .valid = true, .error_msg = "", .hint = "" };
}

// ============================================================================
// Tests (from spec TDD-Inside-Spec)
// ============================================================================

test "validate_tdd_with_tests" {
    const result = validate_tdd_contract(true, false);
    try std.testing.expect(result.valid);
}

test "validate_tdd_with_invariants" {
    const result = validate_tdd_contract(false, true);
    try std.testing.expect(result.valid);
}

test "validate_tdd_without_either" {
    const result = validate_tdd_contract(false, false);
    try std.testing.expect(!result.valid);
}

test "validate_language_ascii" {
    const result = validate_language_policy("hello world", false);
    try std.testing.expect(result.valid);
}

test "validate_language_docs_allowed" {
    const result = validate_language_policy("any content", true);
    try std.testing.expect(result.valid);
}

test "validate_naming_empty" {
    const result = validate_naming_convention("");
    try std.testing.expect(!result.valid);
}

test "validate_naming_valid" {
    const result = validate_naming_convention("valid_name");
    try std.testing.expect(result.valid);
}
