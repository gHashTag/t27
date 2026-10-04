# NOW -- t27b memory primitives (2026-10-04)

## t27b: frame slots, loads and stores, read-only data, bounds checks (Refs #5977)

- New IR primitives that structs, arrays, slices and strings will be lowered onto. Frame slots, the slot and data addresses, `Load`/`Store` at an offset, `Offset` (base + index * scale), `Bounds` (a checked index that traps), and `Copy`. Also a `Ptr` type for aggregates passed by hidden pointer.
- AArch64 code generation:
  - sized loads and stores, sign-extending where needed;
  - slots below x29, including past 4 KiB;
  - ADRP + ADD for data;
  - unrolled or looped copies.
- Read-only data lands in the JIT image and in a Mach-O `__const` section, with PAGE21/PAGEOFF12 relocations. A new test links the object with clang and runs it against the interpreter.
- The interpreter is the oracle. It refuses with a fault, never a pass: reads of unwritten bytes, accesses outside a live slot or blob, stores into data, and a bool load of any byte other than 0 or 1.
- The random differential generator now covers every primitive. It found 0 mismatches over 2500 programs per overflow mode, about 6700 of 11000 functions using frame slots. Corpus unchanged at 47 passing, since no front-end construct lowers to these yet.
