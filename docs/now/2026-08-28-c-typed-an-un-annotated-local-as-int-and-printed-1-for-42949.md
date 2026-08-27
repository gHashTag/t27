# NOW -- C typed an un-annotated local as int and printed 1 for 4294967297 (2026-08-28)

## C typed an un-annotated local as int and printed 1 for 4294967297 (Refs #2161)

- Refs #2161. gen-c fell back to C int for a local with no declared type. Two arms above already special-case integer literals, so the fallback caught everything else -- a CALL among them. `const v = big()` with `big() -> u64` became `int v = big()`: C prints 1 where Rust and Zig print 4294967297, and the C compiles WITHOUT a diagnostic
- Fixed with GNU __auto_type when there is an initialiser -- the same builtin the tuple-destructure paths a few hundred lines above already emit, so no new portability cost. Without an initialiser there is nothing to follow and int stays
- Measured over 746 specs: C output changes for 396, cc -fsyntax-only -std=gnu11 errors 6163 -> 5958, specs with clean C 19 -> 20, and specs that went clean -> broken ZERO. At a radius of 396 that last number is the one that matters, and counting only the sum would have hidden a swap
