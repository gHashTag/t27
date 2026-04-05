// Auto-generated from compiler/skill/registry.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/skill/registry.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");

// ============================================================================
// Constants
// ============================================================================

pub const REGISTRY_PATH: []const u8 = ".trinity/skills/registry.json";

// ============================================================================
// Enums
// ============================================================================

pub const SkillStatus = enum(u8) {
    active = 0,
    sealed = 1,
    paused = 2,
    blocked = 3,
    completed = 4,
};

pub const SkillKind = enum(u8) {
    feature = 0,
    bugfix = 1,
    hotfix = 2,
    recovery = 3,
    refactor = 4,
};

pub const SkillVerdict = enum(u8) {
    not_toxic = 0,
    toxic = 1,
};

// ============================================================================
// Structures
// ============================================================================

pub const SkillMetadata = struct {
    title: []const u8,
    author: []const u8,
    priority: []const u8,
};

pub const Skill = struct {
    id: []const u8,
    status: SkillStatus,
    kind: SkillKind,
    issue: ?[]const u8,
    branch: []const u8,
    created_at: []const u8,
    updated_at: []const u8,
    sealed_at: ?[]const u8,
    commit: ?[]const u8,
    pushed: bool,
    verdict: ?SkillVerdict,
    seal_hash: ?[]const u8,
    artifacts: []const u8,
    metadata: SkillMetadata,
};

pub const SkillRegistry = struct {
    version: []const u8,
    skills: []const Skill,
};

// ============================================================================
// Tests (from spec TDD-Inside-Spec)
// ============================================================================

test "skill_status_values" {
    try std.testing.expect(@intFromEnum(SkillStatus.active) == 0);
    try std.testing.expect(@intFromEnum(SkillStatus.sealed) == 1);
    try std.testing.expect(@intFromEnum(SkillStatus.completed) == 4);
}

test "skill_kind_values" {
    try std.testing.expect(@intFromEnum(SkillKind.feature) == 0);
    try std.testing.expect(@intFromEnum(SkillKind.hotfix) == 2);
    try std.testing.expect(@intFromEnum(SkillKind.recovery) == 3);
}

test "skill_verdict_values" {
    try std.testing.expect(@intFromEnum(SkillVerdict.not_toxic) == 0);
    try std.testing.expect(@intFromEnum(SkillVerdict.toxic) == 1);
}

test "registry_path" {
    try std.testing.expectEqualStrings(".trinity/skills/registry.json", REGISTRY_PATH);
}
