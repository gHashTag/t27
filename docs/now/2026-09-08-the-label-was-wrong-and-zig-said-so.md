# NOW -- The label was wrong, and Zig said so (2026-09-08)

## The label was wrong, and Zig said so (Closes #3497)

- Last pass proposed a cheap audit: **re-read every "impossible" this campaign has written down, and open one member in the SOURCE before believing it.** Five labels, one grep each: **two refuted, one narrowed, two held.**
- **REFUTED:** `cast_i8` was filed as *"a name nothing in the tree declares"* and therefore blocked. The Zig backend has lowered it since W570 -- `@as(i8, @intCast(v))` -- and **its own comment records the very fact used as the blocker**: *"`cast_i8(` alone appears 1,100 times and is defined nowhere in the corpus"*. Declared nowhere was true; *therefore unfixable* was not. `cast_i8` **1079** uses, `cast_i16` 38, `cast_i32` 2 -- the fifth instance this campaign of *three backends agree and one has no answer*.
- **REFUTED:** the 20 `OP_*` lists *"name no single enum"* -- true, because they are `pub const OP_X : u8`, a typed constant. The element type is `u8` and the annotation is right there. Filed, not folded in.
- **NARROWED:** `[T]` was called a language-surface decision. All four backends already lower it in PARAMETER position (`int32_t*`, `Vec<i32>`, `[]i32`); only the local-declaration path is broken. Smaller question than the label claimed.
- **HELD:** `POS`/`NEG` appear 12 times in the specs and every one is inside a comment. **HELD:** the slice half of `len` -- Zig answers `xs.len` *because its slices carry a length*, and C's `T*` cannot. That one really is the representation decision.
- Shipped: `cast_iN(x)` -> `((intN_t)(x))`, mirroring Zig's arm exactly -- integers only, guarded by the declared functions, arity one. Errors **11 042 -> 10 831**, **9 files better and none worse**; `igla_race_ternary_inference` 123 -> 40, `igla_race_ternary_gemm` 296 -> 220.
- The width-suffix test moved from a method on the Zig codegen to a **free function** both backends call: a second copy is how one backend grows a spelling the other refuses, which is the defect this pass repairs.
- A mutant anchor matched twice -- the same line exists in the Zig arm -- and the harness printed `ANCHOR FAILED -- NOT A VERDICT` instead of a survivor. That refusal was built two passes ago after it reported a false one.
