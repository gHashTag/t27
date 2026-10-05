# NOW -- t27b fixed-size arrays: [N]T literals, indexing, len and for (2026-10-05)

## t27b: fixed-size arrays [N]T (Closes #6245)

- `[N]T` (N a literal or an integer constant; int, bool, str or struct elements) is N elements back to back in memory, an aggregate like a struct: copied on assignment, passed by pointer, returned through sret.
- An array literal takes its result type, as t27c's Zig backend writes `.{...}`; the element count must be N, and a literal with no result type is refused.
- `a[i]` reads, writes and compound-assigns (also through `*[N]T` and struct fields); the index is u64, a constant one out of range is refused, a runtime one traps `index out of bounds` like Zig's panic. `.len` is the constant N; `for (a) |x|` walks the elements.
- Module array constants: scalar and struct elements go to read-only data; str elements stay a compile-time value, written into a frame temporary only where memory is needed.
- Corpus (`t27b corpus specs`, native macOS arm64): 84 -> 161 files pass, 778 -> 931 tests, 38 -> 278 pass with 0 runtime asserts, 0 JIT/interpreter mismatch; all 317 newly passing files also pass `t27c test-report`.
- Not yet: f32/f64 elements, `[]T` slices and slicing, `[_]T`, `**`, range and multi-iterable `for`, `s[i]` on str, str arrays inside struct constants, `usize`.
