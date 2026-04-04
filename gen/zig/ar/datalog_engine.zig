// AUTO-GENERATED from specs/ar/datalog_engine.t27 — DO NOT EDIT
// Ring: 18 | Module: DatalogEngine | phi^2 + 1/phi^2 = 3
// Generator: PHI LOOP manual codegen (bootstrap unavailable)

const std = @import("std");
const ternary_logic = @import("ternary_logic.zig");

pub const Trit = ternary_logic.Trit;
pub const K_FALSE = ternary_logic.K_FALSE;
pub const K_UNKNOWN = ternary_logic.K_UNKNOWN;
pub const K_TRUE = ternary_logic.K_TRUE;
pub const Rule = ternary_logic.Rule;
pub const forward_chain = ternary_logic.forward_chain;
pub const k3_equiv = ternary_logic.k3_equiv;

// ═══════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════

pub const MAX_CLAUSES: usize = 256;
pub const MAX_ARGS: usize = 8;

// ═══════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════

/// A Horn clause with a name identifier and up to MAX_ARGS ternary arguments.
pub const HornClause = struct {
    name: u16,
    args: [MAX_ARGS]Trit,
    arg_count: u8,
};

/// Datalog engine: stores facts and rules, performs forward-chaining inference.
pub const DatalogEngine = struct {
    facts: [MAX_CLAUSES]HornClause,
    fact_count: usize,
    rules: [MAX_CLAUSES]Rule,
    rule_count: usize,
    derived_facts: [MAX_CLAUSES]bool,
    solved: bool,
};

// ═══════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════

/// Compare two HornClauses for equality: same name and matching args up to arg_count.
pub fn horn_clause_eq(a: HornClause, b: HornClause) bool {
    if (a.name != b.name) return false;
    if (a.arg_count != b.arg_count) return false;
    var i: usize = 0;
    while (i < a.arg_count) : (i += 1) {
        if (a.args[i] != b.args[i]) return false;
    }
    return true;
}

// ═══════════════════════════════════════════════════════════════
// Core Functions
// ═══════════════════════════════════════════════════════════════

/// Initialize a new empty DatalogEngine with zero facts, zero rules, unsolved.
pub fn datalog_init() DatalogEngine {
    var engine: DatalogEngine = undefined;
    engine.fact_count = 0;
    engine.rule_count = 0;
    engine.solved = false;
    var i: usize = 0;
    while (i < MAX_CLAUSES) : (i += 1) {
        engine.derived_facts[i] = false;
    }
    return engine;
}

/// Add a fact to the engine. Returns true on success, false if full or duplicate.
pub fn add_fact(engine: *DatalogEngine, fact: HornClause) bool {
    // Check for duplicate
    var i: usize = 0;
    while (i < engine.fact_count) : (i += 1) {
        if (horn_clause_eq(engine.facts[i], fact)) return false;
    }
    // Check capacity
    if (engine.fact_count >= MAX_CLAUSES) return false;
    engine.facts[engine.fact_count] = fact;
    engine.fact_count += 1;
    engine.solved = false;
    return true;
}

/// Check whether a fact exists in the engine (including derived facts).
pub fn has_fact(engine: *DatalogEngine, fact: HornClause) bool {
    var i: usize = 0;
    while (i < engine.fact_count) : (i += 1) {
        if (horn_clause_eq(engine.facts[i], fact)) return true;
    }
    return false;
}

/// Add a rule to the engine. Returns true on success, false if full.
pub fn add_rule(engine: *DatalogEngine, rule: Rule) bool {
    if (engine.rule_count >= MAX_CLAUSES) return false;
    engine.rules[engine.rule_count] = rule;
    engine.rule_count += 1;
    engine.solved = false;
    return true;
}

/// Forward-chaining Datalog solver: iterate rules over facts until fixed point.
/// For each rule, forward_chain with each fact's first arg. If result is K_TRUE
/// and produces a new derived fact, add it. Repeat until no new facts are derived.
pub fn datalog_solve(engine: *DatalogEngine) void {
    var changed = true;
    while (changed) {
        changed = false;
        var ri: usize = 0;
        while (ri < engine.rule_count) : (ri += 1) {
            const rule = engine.rules[ri];
            // Snapshot current fact_count so we iterate only existing facts
            const current_count = engine.fact_count;
            var fi: usize = 0;
            while (fi < current_count) : (fi += 1) {
                const fact = engine.facts[fi];
                // Forward chain uses the first arg of the fact as the trit value
                const fact_trit = if (fact.arg_count > 0) fact.args[0] else K_UNKNOWN;
                const result = forward_chain(rule, fact_trit);
                if (result == K_TRUE) {
                    // Derive a new fact: same name, first arg = consequent value
                    var derived: HornClause = undefined;
                    derived.name = fact.name;
                    derived.arg_count = 1;
                    derived.args[0] = rule.consequent;
                    // Zero out remaining args
                    var ai: usize = 1;
                    while (ai < MAX_ARGS) : (ai += 1) {
                        derived.args[ai] = K_UNKNOWN;
                    }
                    if (!has_fact(engine, derived)) {
                        if (engine.fact_count < MAX_CLAUSES) {
                            engine.facts[engine.fact_count] = derived;
                            engine.derived_facts[engine.fact_count] = true;
                            engine.fact_count += 1;
                            changed = true;
                        }
                    }
                }
            }
        }
    }
    engine.solved = true;
}

// ═══════════════════════════════════════════════════════════════
// Tests — Conformance vectors from spec
// ═══════════════════════════════════════════════════════════════

fn make_clause(name: u16, args_slice: []const Trit) HornClause {
    var clause: HornClause = undefined;
    clause.name = name;
    clause.arg_count = @intCast(args_slice.len);
    var i: usize = 0;
    while (i < args_slice.len) : (i += 1) {
        clause.args[i] = args_slice[i];
    }
    while (i < MAX_ARGS) : (i += 1) {
        clause.args[i] = K_UNKNOWN;
    }
    return clause;
}

test "datalog_init_empty" {
    const engine = datalog_init();
    try std.testing.expectEqual(@as(usize, 0), engine.fact_count);
    try std.testing.expectEqual(@as(usize, 0), engine.rule_count);
    try std.testing.expectEqual(false, engine.solved);
    // All derived_facts should be false
    for (engine.derived_facts) |d| {
        try std.testing.expectEqual(false, d);
    }
}

test "add_fact_success" {
    var engine = datalog_init();
    const args = [_]Trit{ K_TRUE, K_FALSE };
    const fact = make_clause(1, &args);
    const ok = add_fact(&engine, fact);
    try std.testing.expect(ok);
    try std.testing.expectEqual(@as(usize, 1), engine.fact_count);
    // Adding same fact again should fail (duplicate)
    const dup = add_fact(&engine, fact);
    try std.testing.expect(!dup);
    try std.testing.expectEqual(@as(usize, 1), engine.fact_count);
}

test "has_fact_present_and_absent" {
    var engine = datalog_init();
    const args_a = [_]Trit{K_TRUE};
    const fact_a = make_clause(10, &args_a);
    _ = add_fact(&engine, fact_a);

    try std.testing.expect(has_fact(&engine, fact_a));

    const args_b = [_]Trit{K_FALSE};
    const fact_b = make_clause(20, &args_b);
    try std.testing.expect(!has_fact(&engine, fact_b));
}

test "add_rule_success" {
    var engine = datalog_init();
    const rule = Rule{ .antecedent = K_TRUE, .consequent = K_TRUE };
    const ok = add_rule(&engine, rule);
    try std.testing.expect(ok);
    try std.testing.expectEqual(@as(usize, 1), engine.rule_count);
}

test "solve_derives_facts" {
    var engine = datalog_init();

    // Add a fact with arg = K_TRUE
    const args = [_]Trit{K_TRUE};
    const fact = make_clause(42, &args);
    _ = add_fact(&engine, fact);

    // Add a rule: if K_TRUE then consequent K_FALSE
    // forward_chain(Rule{K_TRUE, K_FALSE}, K_TRUE) -> k3_and(k3_equiv(K_TRUE, K_TRUE), K_FALSE) = k3_and(K_TRUE, K_FALSE) = K_FALSE
    // That does NOT produce K_TRUE, so no derivation.

    // Add a rule: if K_TRUE then consequent K_TRUE
    // forward_chain(Rule{K_TRUE, K_TRUE}, K_TRUE) -> k3_and(k3_equiv(K_TRUE, K_TRUE), K_TRUE) = k3_and(K_TRUE, K_TRUE) = K_TRUE
    // This produces K_TRUE, so a derived fact with arg=K_TRUE should be added.
    // But that fact already exists (same name=42, args=[K_TRUE]), so no new fact.

    // We need a rule that derives something new. Use consequent=K_FALSE so the derived fact has arg=K_FALSE.
    // Wait: result must be K_TRUE for derivation, but the derived fact uses rule.consequent as its arg.
    // forward_chain(Rule{K_TRUE, K_TRUE}, K_TRUE) = K_TRUE -> derive fact(42, [K_TRUE]) which already exists.
    // Let's make a rule that maps K_TRUE -> derive with consequent K_FALSE, but result must be K_TRUE.
    // forward_chain checks: k3_and(k3_equiv(fact_trit, antecedent), consequent) == K_TRUE
    // So we need k3_equiv(fact_trit, antecedent) >= K_TRUE AND consequent >= K_TRUE
    // => antecedent must equal fact_trit, and consequent must be K_TRUE.
    // Derived fact arg = consequent = K_TRUE. Same as original. Not new.

    // To get a NEW derived fact we need: original fact arg != consequent but still result == K_TRUE.
    // That requires k3_equiv(fact_trit, antecedent) == K_TRUE AND consequent == K_TRUE.
    // => fact_trit == antecedent. Derived arg = K_TRUE.
    // If original fact arg is K_FALSE, antecedent is K_FALSE, consequent K_TRUE:
    // forward_chain = k3_and(k3_equiv(K_FALSE, K_FALSE), K_TRUE) = k3_and(K_TRUE, K_TRUE) = K_TRUE
    // Derived: name=42, args=[K_TRUE] -- different from original [K_FALSE]. New fact!

    // Reset engine
    engine = datalog_init();
    const args_neg = [_]Trit{K_FALSE};
    const fact_neg = make_clause(42, &args_neg);
    _ = add_fact(&engine, fact_neg);

    const rule = Rule{ .antecedent = K_FALSE, .consequent = K_TRUE };
    _ = add_rule(&engine, rule);

    try std.testing.expectEqual(@as(usize, 1), engine.fact_count);
    try std.testing.expect(!engine.solved);

    datalog_solve(&engine);

    try std.testing.expect(engine.solved);
    // Should now have the original fact plus the derived fact
    try std.testing.expect(engine.fact_count >= 2);

    const derived_args = [_]Trit{K_TRUE};
    const derived_fact = make_clause(42, &derived_args);
    try std.testing.expect(has_fact(&engine, derived_fact));
    // The derived fact should be marked as derived
    try std.testing.expect(engine.derived_facts[1]);
}

test "solve_no_rules_is_noop" {
    var engine = datalog_init();
    const args = [_]Trit{K_TRUE};
    const fact = make_clause(1, &args);
    _ = add_fact(&engine, fact);

    datalog_solve(&engine);

    try std.testing.expect(engine.solved);
    try std.testing.expectEqual(@as(usize, 1), engine.fact_count);
}

test "solve_fixed_point_no_infinite_loop" {
    // A rule that re-derives the same fact should reach fixed point immediately
    var engine = datalog_init();
    const args = [_]Trit{K_TRUE};
    const fact = make_clause(7, &args);
    _ = add_fact(&engine, fact);

    // Rule: K_TRUE -> K_TRUE. Derives fact(7, [K_TRUE]) which already exists. No change.
    const rule = Rule{ .antecedent = K_TRUE, .consequent = K_TRUE };
    _ = add_rule(&engine, rule);

    datalog_solve(&engine);

    try std.testing.expect(engine.solved);
    try std.testing.expectEqual(@as(usize, 1), engine.fact_count);
}

test "horn_clause_eq_basic" {
    const args_a = [_]Trit{ K_TRUE, K_FALSE, K_UNKNOWN };
    const a = make_clause(5, &args_a);
    const b = make_clause(5, &args_a);
    try std.testing.expect(horn_clause_eq(a, b));

    // Different name
    const c = make_clause(6, &args_a);
    try std.testing.expect(!horn_clause_eq(a, c));

    // Different args
    const args_d = [_]Trit{ K_TRUE, K_TRUE, K_UNKNOWN };
    const d = make_clause(5, &args_d);
    try std.testing.expect(!horn_clause_eq(a, d));

    // Different arg_count
    const args_e = [_]Trit{ K_TRUE, K_FALSE };
    const e = make_clause(5, &args_e);
    try std.testing.expect(!horn_clause_eq(a, e));
}

test "add_fact_capacity" {
    var engine = datalog_init();
    const args = [_]Trit{K_TRUE};
    // Fill to capacity
    var i: u16 = 0;
    while (i < MAX_CLAUSES) : (i += 1) {
        const fact = make_clause(i, &args);
        try std.testing.expect(add_fact(&engine, fact));
    }
    try std.testing.expectEqual(MAX_CLAUSES, engine.fact_count);
    // One more should fail
    const overflow = make_clause(999, &args);
    try std.testing.expect(!add_fact(&engine, overflow));
}

// ═══════════════════════════════════════════════════════════════
// Invariant tests
// ═══════════════════════════════════════════════════════════════

test "invariant_solve_idempotent" {
    var engine = datalog_init();
    const args = [_]Trit{K_FALSE};
    _ = add_fact(&engine, make_clause(1, &args));
    _ = add_rule(&engine, Rule{ .antecedent = K_FALSE, .consequent = K_TRUE });

    datalog_solve(&engine);
    const count_after_first = engine.fact_count;

    datalog_solve(&engine);
    const count_after_second = engine.fact_count;

    try std.testing.expectEqual(count_after_first, count_after_second);
}

test "invariant_has_fact_for_all_added" {
    var engine = datalog_init();
    const args1 = [_]Trit{K_TRUE};
    const args2 = [_]Trit{K_FALSE};
    const args3 = [_]Trit{K_UNKNOWN};
    const f1 = make_clause(1, &args1);
    const f2 = make_clause(2, &args2);
    const f3 = make_clause(3, &args3);
    _ = add_fact(&engine, f1);
    _ = add_fact(&engine, f2);
    _ = add_fact(&engine, f3);

    try std.testing.expect(has_fact(&engine, f1));
    try std.testing.expect(has_fact(&engine, f2));
    try std.testing.expect(has_fact(&engine, f3));
}
