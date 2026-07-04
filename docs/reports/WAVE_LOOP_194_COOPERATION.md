# Wave Loop 194 — Cooperation Variants for W195

**Date:** 2026-06-16
**Status:** Proposed

---

## Variant A — Graph + Encoding Functionalization Standard (tri/graph, tri/encoding)

**Goal:** Expand the W193 pilot functionalization into a systematic standard for the newly promoted octa specs in `tri/graph/` and `tri/encoding/`.

**Actions:**
- 1. Replace placeholder invariants in `specs/tri/graph/dijkstra.t27` with `invariant dijkstra_nonnegative: forall edge : Edge, edge.weight >= 0`.
- 2. Replace placeholder invariants in `specs/tri/encoding/json.t27` with `invariant json_roundtrip: parse(encode(v)) == v`.
- 3. Replace placeholder invariants in `specs/tri/graph/topological_sort.t27` with `invariant topo_acyclic: forall g : Graph, topo_sort(g).valid == (g.cycles == 0)`.
- 4. Publish the pattern in `docs/STANDARDS/FUNCTIONAL_INVARIANTS.md` as the canonical example set.

**Risk:** Very low. Internal spec enhancement.
**Benefit:** Creates a reproducible template for converting placeholder invariants into genuine behavioral checks across the octa layer.

---

## Variant B — Ternary Sort Benchmark Consortium (tri/sort Co-Design)

**Goal:** Leverage the newly promoted `tri/sort/` octa specs (quick_sort, radix_sort, sort) to establish a cross-project ternary sorting benchmark with Ternary Fabric and TernaryCore.

**Actions:**
- 1. Engage Ternary Fabric (github.com/t81dev/ternary-fabric) to adapt their PT-5 trit encoding for integer-key sorting workloads.
- 2. Define a shared `bench_ternary_sort` harness under `specs/benchmarks/bench_main.t27` that measures cycles per key for quick_sort and radix_sort on ternary-encoded data.
- 3. Propose a quarterly report comparing Trinity `t27c` simulation results against Ternary Fabric's Zynq-7000 cycle counts and TernaryCore's Artix-7 measurements.

**Risk:** Medium. Requires hardware access and agreed-upon encoding format.
**Benefit:** First cross-project ternary algorithmic benchmark; positions Trinity as the SSOT for ternary data-structure verification.

---

## Variant C — I/O + Graph Co-Verification with IGLA Race Specs

**Goal:** Bridge the newly promoted `tri/io/` and `tri/graph/` octa specs with the IGLA race verification pipeline to ensure hardware-software alignment.

**Actions:**
- 1. Add `invariant io_block_aligned: block_size % 16 == 0` to `specs/tri/io/fs.t27` to enforce GF16-friendly block boundaries.
- 2. Add `invariant graph_edge_bounds: forall e : Edge, e.src < n && e.dst < n` to `specs/tri/graph/graph.t27` to prevent out-of-bounds memory access in FPGA implementations.
- 3. Run a joint regression test: execute `t27c suite --repo-root .` on both the IGLA race specs and the newly functionalized tri specs in a single pass, confirming no cross-module seal drift.

**Risk:** Low. Infrastructure and alignment only.
**Benefit:** Prevents the historical IGLA seal drift pattern from recurring by hardening cross-module invariants.

---

## Decision Matrix

| Variant | Effort | Impact | Timeline | Recommended |
|---------|--------|--------|----------|-------------|
| A | Low | High (spec quality) | W195 | **Primary** |
| B | Medium | High (cross-project) | W195–W197 | **Stretch** |
| C | Low | Medium (infra) | W195 | **Parallel** |

---

**φ² + 1/φ² = 3 | TRINITY**
