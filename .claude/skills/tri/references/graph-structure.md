# Canonical Dependency Graph Reference

This document details the canonical dependency graph rules for t27 module evolution.

## Source Documents

- **`architecture/graph.tri`** — Canonical source of truth for dependencies
- **`architecture/graphv2.json`** — Typed graph representation

## Graph Schema

```tri
node struct Node {
    id: string,
    tier: u2,  // 0-4
    module: string,
    phi_critical: bool,
    sacred_core: bool,
    deps: []string,
}

edge struct Edge {
    from: string,
    to: string,
    type: EdgeType,
}

enum EdgeType {
    dependency,
    phi_critical,
    sacred_core,
}
```

## Tiers

| Tier | Domain | Modules | Example Paths |
|------|---------|----------|---------------|
| 0 | Base Types | Trit, PackedTrit, TernaryWord | `specs/base/*.t27` |
| 1 | Core Arithmetic | TritypeNumeric, TriFormat | `specs/numeric/*.t27` |
| 2 | VSA Primitives | TriVSA, TriISA, TriAttention | `specs/vsa/*.t27` |
| 3 | Orchestrator | TriQuenn | `specs/orch/*.t27` |
| 4 | Language Tooling | TriCLI | `specs/cli/*.t27` |

## Dependency Rules

1. **No forward tier dependencies**: Lower tiers cannot depend on higher tiers
2. **Minimal dependencies**: Only declare deps actually used
3. **No circular dependencies**: Graph must remain acyclic
4. **Sacred-first**: sacred_core edges must be satisfied before others
5. **Phi-critical**: phi_critical nodes require special approval

## Special Edge Types

### phi_critical
Edges that require PHI LOOP verification before proceeding.

- Affects core ternary arithmetic
- Requires hash seal with sacred physics conformance
- Must pass `tri verdict --toxic`

### sacred_core
Edges involving sacred constants.

- TRINITY, G, ΩΛ, γ, tpresent
- Exact tolerance requirements
- Constitutional law compliance

## Graph Validation

```bash
tri graph check        # Validate graph structure
tri graph deps <node>   # Show dependencies for node
tri graph topo          # Show topological order
```

## Evolution Protocol

Before adding a new module:

1. **Check tier**: Confirm tier assignment is correct
2. **Validate deps**: Run `tri graph check`
3. **Add phi-critical**: If affecting core, mark appropriately
4. **Update graph**: Modify `graph.tri`
5. **Revalidate**: Run `tri graph check` again
6. **Proceed**: Only on clean validation

## Example: Adding New Module

```tri
// graph.tri
node my_new_module = Node {
    .id = "my_new_module",
    .tier = 1,  // Tier 1: numeric
    .module = "specs/numeric/my_new_module.t27",
    .phi_critical = false,
    .sacred_core = false,
    .deps = ["trit_type"],  // Only depends on Tier 0
};
```

## Common Violations

**Toxic verdict triggers:**

1. Forward tier dependency (Tier 1 depends on Tier 2)
2. Circular dependency
3. Missing phi_critical flag on core changes
4. Sacred constant without sacred_core flag
5. Orphaned node (no path to/from main)

## See Also

- `references/constitutional-laws.md` — Constitutional foundation
- `references/numeric-standards.md` — Tier 1 numeric formats
