# NOW -- A tuple type was emitted verbatim (2026-09-06)

## A tuple type was emitted verbatim (Refs #3345)

- The Rust type mapper had no arm for a tuple, so `(A, B)` fell through to the default and
  an inner `[]f32` -- which every other position maps to `Vec<f32>` -- reached rustc
  as `[]f32`.
- Measured: 3 specs emitted `-> (...[]...)` before, 0 after. The class is closed.
- **+0 on the column.** All three still fail on other defects; two of the three changed
  their failure cause, which is how the fix was confirmed rather than assumed.
- My first count of the class was 6 and it was wrong. The matcher also caught
  `std.StringHashMap([]Const u8)` and `std.HashMap(T, []T)` -- Zig standard-library
  types leaking into the Rust output, not tuples. That is a separate class of 8 specs.
- The split is depth-aware, because a naive `split(',')` would cut `(Map<K, V>, T)`
  into `Map<K` and ` V>` and emit something worse than the input. Controls:
  `([]f32, []u8)` maps both elements; the one-element `(u32)` stays `(u32)` and does
  not become the one-tuple `(u32,)`, which would be a different type.
- Found by censusing the FIRST rustc error of all 245 failures together with the generated
  LINE, which is what separated four classes inside one error text.
