// AUTO-GENERATED from specs/ar/ternary_logic.t27 — DO NOT EDIT
// Ring: 18 | Module: TernaryLogic | φ² + 1/φ² = 3
// Generator: PHI LOOP manual codegen (bootstrap unavailable)

const std = @import("std");

// ═══════════════════════════════════════════════════════════════
// Kleene K3 Truth Values — Isomorphism: Trit ≅ Kleene K3
// ═══════════════════════════════════════════════════════════════

pub const Trit = enum(i8) {
    neg = -1,
    zero = 0,
    pos = 1,
};

pub const K_FALSE: Trit = .neg;
pub const K_UNKNOWN: Trit = .zero;
pub const K_TRUE: Trit = .pos;

// ═══════════════════════════════════════════════════════════════
// Base operations (from base::ops)
// ═══════════════════════════════════════════════════════════════

fn trit_min(a: Trit, b: Trit) Trit {
    const ai = @intFromEnum(a);
    const bi = @intFromEnum(b);
    return if (ai < bi) a else b;
}

fn trit_max(a: Trit, b: Trit) Trit {
    const ai = @intFromEnum(a);
    const bi = @intFromEnum(b);
    return if (ai > bi) a else b;
}

fn trit_not(a: Trit) Trit {
    return @enumFromInt(-@intFromEnum(a));
}

fn trit_compare(a: Trit, b: Trit) i8 {
    const ai = @intFromEnum(a);
    const bi = @intFromEnum(b);
    if (ai < bi) return -1;
    if (ai > bi) return 1;
    return 0;
}

// ═══════════════════════════════════════════════════════════════
// Kleene K3 Logical Operations
// ═══════════════════════════════════════════════════════════════

/// Kleene conjunction: minimum of truth values
pub fn k3_and(a: Trit, b: Trit) Trit {
    return trit_min(a, b);
}

/// Kleene disjunction: maximum of truth values
pub fn k3_or(a: Trit, b: Trit) Trit {
    return trit_max(a, b);
}

/// Kleene negation: truth value inversion
pub fn k3_not(a: Trit) Trit {
    return trit_not(a);
}

/// Kleene implication: ¬a ∨ b
pub fn k3_implies(a: Trit, b: Trit) Trit {
    return k3_or(k3_not(a), b);
}

/// Kleene equivalence: (a→b) ∧ (b→a)
pub fn k3_equiv(a: Trit, b: Trit) Trit {
    const ab = k3_implies(a, b);
    const ba = k3_implies(b, a);
    return k3_and(ab, ba);
}

// ═══════════════════════════════════════════════════════════════
// Automated Reasoning Primitives
// ═══════════════════════════════════════════════════════════════

pub const Rule = struct {
    antecedent: Trit,
    consequent: Trit,
};

/// Forward chaining: modus ponens for Kleene K3
pub fn forward_chain(rule: Rule, fact: Trit) Trit {
    const fact_matches = k3_equiv(fact, rule.antecedent);
    return k3_and(fact_matches, rule.consequent);
}

/// Backward chaining: search for rules that support the goal
pub fn backward_chain(goal: Trit, rules: []const Rule, count: usize) Trit {
    var result: Trit = K_UNKNOWN;
    var i: usize = 0;
    while (i < count) : (i += 1) {
        const rule = rules[i];
        const consequent_matches = k3_equiv(rule.consequent, goal);
        const support = k3_and(consequent_matches, rule.antecedent);
        result = k3_or(result, support);
    }
    return result;
}

/// Resolution principle for Kleene logic
pub fn resolve(clause_a: []const Trit, clause_b: []const Trit, len: usize) [64]Trit {
    var result: [64]Trit = undefined;
    var i: usize = 0;
    while (i < len) : (i += 1) {
        const a = clause_a[i];
        const b = clause_b[i];
        if (a == K_TRUE and b == K_FALSE) {
            result[i] = K_UNKNOWN;
        } else if (a == K_FALSE and b == K_TRUE) {
            result[i] = K_UNKNOWN;
        } else {
            result[i] = k3_or(a, b);
        }
    }
    return result;
}

// ═══════════════════════════════════════════════════════════════
// Restraint and Bounded Rationality
// ═══════════════════════════════════════════════════════════════

/// Check if a truth value represents restraint (bounded rationality)
pub fn is_restraint(t: Trit) bool {
    return t == K_UNKNOWN;
}

/// Apply restraint optimization: replace K_UNKNOWN with K_FALSE
pub fn apply_restraint(values: []const Trit, len: usize) [256]Trit {
    var result: [256]Trit = undefined;
    var i: usize = 0;
    while (i < len) : (i += 1) {
        const t = values[i];
        if (is_restraint(t)) {
            result[i] = K_FALSE;
        } else {
            result[i] = t;
        }
    }
    return result;
}

// ═══════════════════════════════════════════════════════════════
// Tests — Conformance vectors from spec
// ═══════════════════════════════════════════════════════════════

test "k3_and_truth_table" {
    const t_values = [_]Trit{ K_FALSE, K_UNKNOWN, K_TRUE };
    for (t_values) |a| {
        for (t_values) |b| {
            const result = k3_and(a, b);
            const expected = trit_min(a, b);
            try std.testing.expectEqual(expected, result);
        }
    }
}

test "k3_or_truth_table" {
    const t_values = [_]Trit{ K_FALSE, K_UNKNOWN, K_TRUE };
    for (t_values) |a| {
        for (t_values) |b| {
            const result = k3_or(a, b);
            const expected = trit_max(a, b);
            try std.testing.expectEqual(expected, result);
        }
    }
}

test "k3_not_truth_table" {
    try std.testing.expectEqual(K_TRUE, k3_not(K_FALSE));
    try std.testing.expectEqual(K_UNKNOWN, k3_not(K_UNKNOWN));
    try std.testing.expectEqual(K_FALSE, k3_not(K_TRUE));
}

test "k3_implication_ex_falso" {
    try std.testing.expectEqual(K_TRUE, k3_implies(K_FALSE, K_FALSE));
}

test "k3_implication_when_antecedent_true" {
    try std.testing.expectEqual(K_FALSE, k3_implies(K_TRUE, K_FALSE));
    try std.testing.expectEqual(K_UNKNOWN, k3_implies(K_TRUE, K_UNKNOWN));
    try std.testing.expectEqual(K_TRUE, k3_implies(K_TRUE, K_TRUE));
}

test "k3_implication_when_consequent_true" {
    try std.testing.expectEqual(K_TRUE, k3_implies(K_FALSE, K_TRUE));
}

test "k3_implication_with_unknown" {
    try std.testing.expectEqual(K_UNKNOWN, k3_implies(K_UNKNOWN, K_UNKNOWN));
}

test "k3_equiv_reflexive" {
    try std.testing.expectEqual(K_TRUE, k3_equiv(K_FALSE, K_FALSE));
    try std.testing.expectEqual(K_TRUE, k3_equiv(K_UNKNOWN, K_UNKNOWN));
    try std.testing.expectEqual(K_TRUE, k3_equiv(K_TRUE, K_TRUE));
}

test "k3_equiv_symmetric" {
    const t_values = [_]Trit{ K_FALSE, K_UNKNOWN, K_TRUE };
    for (t_values) |a| {
        for (t_values) |b| {
            try std.testing.expectEqual(k3_equiv(a, b), k3_equiv(b, a));
        }
    }
}

test "k3_equiv_when_both_true" {
    try std.testing.expectEqual(K_TRUE, k3_equiv(K_TRUE, K_TRUE));
}

test "k3_equiv_when_both_false" {
    try std.testing.expectEqual(K_TRUE, k3_equiv(K_FALSE, K_FALSE));
}

test "k3_equiv_when_opposite" {
    try std.testing.expectEqual(K_FALSE, k3_equiv(K_TRUE, K_FALSE));
}

test "forward_chain_modus_ponens_true" {
    const rule = Rule{ .antecedent = K_TRUE, .consequent = K_TRUE };
    try std.testing.expectEqual(K_TRUE, forward_chain(rule, K_TRUE));
}

test "forward_chain_modus_ponens_false_consequent" {
    const rule = Rule{ .antecedent = K_TRUE, .consequent = K_FALSE };
    try std.testing.expectEqual(K_FALSE, forward_chain(rule, K_TRUE));
}

test "forward_chain_with_unknown_fact" {
    const rule = Rule{ .antecedent = K_TRUE, .consequent = K_TRUE };
    try std.testing.expectEqual(K_UNKNOWN, forward_chain(rule, K_UNKNOWN));
}

test "forward_chain_no_match" {
    const rule = Rule{ .antecedent = K_TRUE, .consequent = K_TRUE };
    try std.testing.expectEqual(K_FALSE, forward_chain(rule, K_FALSE));
}

test "backward_chain_finds_support" {
    const rules = [_]Rule{
        .{ .antecedent = K_TRUE, .consequent = K_TRUE },
        .{ .antecedent = K_FALSE, .consequent = K_UNKNOWN },
    };
    try std.testing.expectEqual(K_TRUE, backward_chain(K_TRUE, &rules, 2));
}

test "backward_chain_no_support" {
    const rules = [_]Rule{
        .{ .antecedent = K_FALSE, .consequent = K_FALSE },
    };
    try std.testing.expectEqual(K_UNKNOWN, backward_chain(K_TRUE, &rules, 1));
}

test "backward_chain_multiple_rules" {
    const rules = [_]Rule{
        .{ .antecedent = K_FALSE, .consequent = K_TRUE },
        .{ .antecedent = K_TRUE, .consequent = K_TRUE },
        .{ .antecedent = K_UNKNOWN, .consequent = K_TRUE },
    };
    try std.testing.expectEqual(K_TRUE, backward_chain(K_TRUE, &rules, 3));
}

test "resolve_complementary_literals" {
    const clause_a = [_]Trit{ K_TRUE, K_FALSE, K_UNKNOWN };
    const clause_b = [_]Trit{ K_FALSE, K_TRUE, K_UNKNOWN };
    const result = resolve(&clause_a, &clause_b, 3);
    try std.testing.expectEqual(K_UNKNOWN, result[0]);
    try std.testing.expectEqual(K_UNKNOWN, result[1]);
    try std.testing.expectEqual(K_UNKNOWN, result[2]);
}

test "resolve_non_complementary" {
    const clause_a = [_]Trit{ K_TRUE, K_UNKNOWN };
    const clause_b = [_]Trit{ K_TRUE, K_FALSE };
    const result = resolve(&clause_a, &clause_b, 2);
    try std.testing.expectEqual(K_TRUE, result[0]);
    try std.testing.expectEqual(K_UNKNOWN, result[1]);
}

test "is_restraint_true_for_unknown" {
    try std.testing.expect(is_restraint(K_UNKNOWN));
}

test "is_restraint_false_for_false" {
    try std.testing.expect(!is_restraint(K_FALSE));
}

test "is_restraint_false_for_true" {
    try std.testing.expect(!is_restraint(K_TRUE));
}

test "apply_restraint_replaces_unknown" {
    const values = [_]Trit{ K_TRUE, K_UNKNOWN, K_FALSE, K_UNKNOWN, K_TRUE };
    const result = apply_restraint(&values, 5);
    try std.testing.expectEqual(K_TRUE, result[0]);
    try std.testing.expectEqual(K_FALSE, result[1]);
    try std.testing.expectEqual(K_FALSE, result[2]);
    try std.testing.expectEqual(K_FALSE, result[3]);
    try std.testing.expectEqual(K_TRUE, result[4]);
}

test "apply_restraint_preserves_known" {
    const values = [_]Trit{ K_TRUE, K_FALSE };
    const result = apply_restraint(&values, 2);
    try std.testing.expectEqual(K_TRUE, result[0]);
    try std.testing.expectEqual(K_FALSE, result[1]);
}

// ═══════════════════════════════════════════════════════════════
// Invariant tests
// ═══════════════════════════════════════════════════════════════

test "invariant_k3_and_commutative" {
    const t_values = [_]Trit{ K_FALSE, K_UNKNOWN, K_TRUE };
    for (t_values) |a| {
        for (t_values) |b| {
            try std.testing.expectEqual(k3_and(a, b), k3_and(b, a));
        }
    }
}

test "invariant_k3_or_commutative" {
    const t_values = [_]Trit{ K_FALSE, K_UNKNOWN, K_TRUE };
    for (t_values) |a| {
        for (t_values) |b| {
            try std.testing.expectEqual(k3_or(a, b), k3_or(b, a));
        }
    }
}

test "invariant_k3_and_associative" {
    const t_values = [_]Trit{ K_FALSE, K_UNKNOWN, K_TRUE };
    for (t_values) |a| {
        for (t_values) |b| {
            for (t_values) |c| {
                try std.testing.expectEqual(k3_and(k3_and(a, b), c), k3_and(a, k3_and(b, c)));
            }
        }
    }
}

test "invariant_k3_or_associative" {
    const t_values = [_]Trit{ K_FALSE, K_UNKNOWN, K_TRUE };
    for (t_values) |a| {
        for (t_values) |b| {
            for (t_values) |c| {
                try std.testing.expectEqual(k3_or(k3_or(a, b), c), k3_or(a, k3_or(b, c)));
            }
        }
    }
}

test "invariant_k3_double_negation" {
    const t_values = [_]Trit{ K_FALSE, K_UNKNOWN, K_TRUE };
    for (t_values) |x| {
        try std.testing.expectEqual(x, k3_not(k3_not(x)));
    }
}

test "invariant_k3_equiv_reflexive" {
    const t_values = [_]Trit{ K_FALSE, K_UNKNOWN, K_TRUE };
    for (t_values) |x| {
        try std.testing.expectEqual(K_TRUE, k3_equiv(x, x));
    }
}

test "invariant_trit_k3_isomorphism_ordering" {
    try std.testing.expectEqual(@as(i8, -1), trit_compare(K_FALSE, K_UNKNOWN));
    try std.testing.expectEqual(@as(i8, -1), trit_compare(K_UNKNOWN, K_TRUE));
    try std.testing.expectEqual(@as(i8, -1), trit_compare(K_FALSE, K_TRUE));
}
