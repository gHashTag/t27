# NOW -- A struct holding an optional of itself is infinitely sized (2026-09-06)

## An optional of itself is infinitely sized (Refs #3375)

- `pub left: Option<KDNode>` inside `KDNode` is E0072. Zig writes `?KDNode` and stores
  it inline because its optional of a struct is a tagged union of known size; Rust needs
  the indirection spelled out, and rustc names the repair itself.
- Only `Option<ThisStruct>` is rewritten, on an exact name match. `Vec<ThisStruct>` is
  already indirect; a bare `ThisStruct` would still be infinite but does not occur, and
  guessing at a shape nothing exhibits is how a rule outgrows its evidence.
- Measured: **357 to 360**, zero regressions: `kd_tree`, `octree`, `quadtree`.
- `octree.t27` was recorded as a REGRESSION in #3247, where I wrote that `[8]?OctNode`
  is genuinely infinitely sized. That was true of the emitted Rust and not of the spec.
  It is no longer a regression.
- Boundary control in one probe: `?Node` inside `Node` boxes; `?Other` does not;
  `?NodeExtra` does not, so an exact match does not catch a prefix; `[]Node` does not.
