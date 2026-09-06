# NOW -- A field that names its own struct needs a Box (2026-09-06)

## A field that names its own struct needs a Box (Refs #3377)

- First fix in five passes to move the column, and the first whose target was PICKED:
  `tri one-away` reported this class as the sole error of 3 specs whose count is exact,
  and the repair moved exactly those 3.
- `pub left: Option<KDNode>` inside `pub struct KDNode` is infinitely sized. There is
  no version that compiles without indirection, so nothing is chosen about representation
  -- `Box` is the ownership-preserving one of the three names rustc itself offers.
- Two shapes. `Option<Name>` is kd_tree. `[Option<Name>; N]` is octree and quadtree,
  and an earlier pass of mine wrote octree off as "genuinely infinitely sized". **That was
  wrong**: an array is inline storage, only the element needs the box, and
  `[Option<Box<OctNode>>; 8]` is finite.
- Measured, two pinned binaries from the same commit in ONE pass, 650 specs:
  **357 -> 360, +3, zero regressions.**
- Nine structs in the corpus name themselves. Three were one-away and are now green; the
  rest carry other errors.
- Narrow on purpose: `Vec<Name>` already carries indirection and is untouched, and a
  shape this does not recognise keeps its current output rather than getting a guess.
