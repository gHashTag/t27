# Vetted Logic Blocks Catalog

This directory contains JSON catalogs of logic blocks extracted from `architecture/graph_v2.json`.

## Catalog Files

- `math-constants-sacred-chain.json` — The sacred physics subgraph:
  - math/constants (node 4)
  - physics/chern-simons (node 54)
  - math/sacred_physics (node 17)

## Schema

Each node entry contains:

| Field | Description |
|-------|-------------|
| `name` | Module name (e.g., "math/constants") |
| `node_id` | Node ID from graph_v2.json |
| `path` | Path to .t27 spec file |
| `tier` | Dependency tier level |
| `kind` | Spec kind (spec, compiler, codegen, etc.) |
| `exports` | Array of exported functions/types |
| `sacred_level` | foundation, phi-critical, sacred-core, or null |
| `test_invariant` | Mathematical guarantee provided |
| `toxic_regression` | Downstream impact if broken |
| `conformance_file` | Path to conformance JSON (if exists) |

## Using the Catalog

Before composing modules, check:
1. **Sacred level** — phi-critical blocks require careful review
2. **Test invariant** — Understand the mathematical guarantee
3. **Toxic regression** — Know what downstream modules are affected
4. **Conformance** — Run tests with `tri test <conformance_file>`

## Adding New Catalogs

Extract subgraphs from graph_v2.json following the pattern:
```bash
# 1. Read graph_v2.json to find related nodes
# 2. Trace edges to identify composition path
# 3. Create JSON file in this directory
# 4. Add entry to README.md
```
