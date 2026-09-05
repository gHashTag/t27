# NOW -- The Zig-shaped array discards its length, and I moved the ruler under my own audit (2026-09-05)

Eighth compiler fix of the pass, and the first whose provenance had to be thrown away
and re-derived.

## I invalidated my own fan-out (Refs #3246)

- launched a 12-agent audit of the 43 remaining E0308 specs, telling every agent the binary was already built and not to rebuild
- then rebuilt it myself, twice, while they ran -- switching branches for unrelated work on `tri misread`
- one agent reported it in as many words: "THE RULER MOVED UNDER ME", naming the byte size it started with and the size it later saw; a second reported that the control figure of 315 accepting specs did not reproduce
- the workflow was **stopped** and its counts discarded. A measurement against a moving ruler is not a measurement, and this pass has spent itself insisting on that
- the repair is a **pinned** binary: `/tmp/t27c-pinned`, copied once and never rebuilt, and every number below comes from it. Its baseline reproduces exactly: 315 accept, 266 fail, 69 do not generate, 43 first-error E0308

## The defect (Closes #3246)

- the mapper has both array branches and the Zig-shaped one throws the size away: `[4]u8` becomes `Vec<u8>` while `[u8; 4]` becomes `[u8; 4]`
- the right answer was already forty lines above in the same function, whose comment records the history: "Previously only the Zig-style `[N]T` form was handled"
- measured against the pinned binary: **315 -> 318, +4, -1**

## The trade, stated in the open (Closes #3246)

- the one regression is `specs/tri/trees/octree.t27`, whose field is `children : [8]?OctNode` -- a fixed array of the struct it lives in
- as `Vec<Option<OctNode>>` it compiled, because a Vec supplies indirection BY ACCIDENT while discarding the length; as `[Option<OctNode>; 8]` it is honestly infinitely sized and rustc says so
- neither lowering is right: the correct Rust is `[Option<Box<OctNode>>; 8]`, which keeps both the length and the indirection, and it needs the enclosing type's name inside a static function with 18 call sites, so it is not a mapper tweak
- the siblings do not settle it: the C output for that spec is already invalid, the optional having leaked as `?OctNode* children;`, and Zig's acceptance is the shallow `build-obj` reading

## A regression the measurement caught before it shipped (Refs #3246)

- the first version read everything inside the brackets as a size and produced `[T; * as usize]` from `[*]T`, the source language's many-item POINTER, where the `*` is not a length
- the guard: anything inside the brackets that is not a digit string or a plain identifier keeps the previous unsized lowering
- both regressions came from one omission -- I enumerated the BRANCHES of the bracket syntax and not the CONTENTS of the brackets
