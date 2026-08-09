# Wave Loop 583 — nobody had ever compiled the C backend's output

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_582_REPORT.md`](WAVE_LOOP_582_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
generated C headers that COMPILE      36  ->  101   of 397
`unknown type name` first errors     187  ->   64
specs that parse 397 · ALL_PASS 28 · 683 tests · assertions emitted 9,267
lex-conform 29/29 · parse-conform 13/13 · truncation 0 · T1/T2/T3 re-proved
```

W582 measured "409 invalid C field declarations" with a **regex**, and said so: a proxy
for validity, not validity. W583 ran the real thing — `cc -fsyntax-only` over every
generated header — for the first time in the project's life.

**36 of 397 compiled.**

---

## 1. The falsification check

W582's condition was *"if the generated headers are not self-contained, the first run is
all 'unknown type name' and the real work is header structure."* They are
self-contained — `#include <stdint.h>`, `<stdbool.h>`, `<stddef.h>`, an include guard —
so the audit measures type mapping, which is what it was for.

## 2. What the compiler said

| Count | First error |
|---:|---|
| **187** | `unknown type name` |
| 79 | `call to undeclared function` |
| 32 | `type name requires a specifier or qualifier` |
| 15 | `type specifier missing, defaults to 'int'` |

Resolving the top class:

```
  46  f32        34  f64        29  str        17  std        9  string     4  gf16
```

**`f32` and `f64` were simply absent from `type_to_c`.** So was `str`. The mapper's
`_ => ty` arm passed them through verbatim, and C received `f32 x;`. This is the same
default that W582 found emitting `[]u8 field;` — the arm was never the problem in
isolation; the problem is that nothing ever compiled the result.

`call to undeclared function` resolved to `assert_eq` (59) — a helper the backend emits
calls to in every test body and **never defined**.

## 3. Fixed

| | |
|---|---|
| `f32`/`f64` → `float`/`double`; `str`/`string` → `const char*`; `gf16` → `uint16_t`; `isize`, `u128`, `i128` | the missing scalars |
| `std.mem.Allocator` → `void*` | a dotted foreign type has no C spelling; `void*` is what a hand-written binding uses |
| `#define assert_eq(a, b)` | emitted alongside `t27_assert` |
| `_Static_assert(assert(f(x) == 3), …)` | **invalid twice over** — `assert` is a runtime macro and a call is not a constant expression. The condition is unwrapped, and an invariant that calls anything becomes a comment naming why C cannot check it |

One structural fix mattered more than the mappings: `param_type_to_c` gated its scalar
lowering behind `is_primitive`, which lists only the integers. So even after `type_to_c`
learned `f32`, the gate suppressed it. **`type_to_c` already passes genuinely custom
types through, so the guard only ever prevented correct mappings** — removing it was the
change that moved the number.

## 4. What remains, and what is not mine

296 headers still fail. The largest classes:

| Count | Class | |
|---:|---|---|
| 75 | `call to undeclared function` | **47 `default_input`, 27 `valid_input`** — the template scaffold, pending since W561 |
| 64 | `unknown type name` | cross-module types (`TernaryWeight`, `Trit`, `PinAssignment`) — the C backend has no `use` resolution, which the Zig backend only gained in W569 |
| 38 | `type name requires a specifier` | malformed array typedefs: `typedef struct { [u8 v[16];16]; }` |
| 32 | `type specifier missing` | same family |

The first is a maintainer decision. The second is a real feature gap with a known
shape. The third is a genuine bug in the array-typedef hoisting, and is the largest
thing here that is nobody's decision but the compiler's.

---

## 5. Verification

| Gate | Result |
|---|---|
| Generated C headers compiling | **36 → 101** of 397 |
| Parse, 608 non-scratch specs, per-file vs W568 | `341 → 397`, **0 regressions** beyond the three W577 un-truncations |
| Harness | `ALL_PASS 28, TEST_FAIL 0`, 683 tests passing |
| Assertions emitted | 9,267 (unchanged — the Zig path was not touched) |
| `lex-conform` / `parse-conform` / `parse-complete` | 29/29 · 13/13 · truncation 0 |
| Generated Verilog vs W568 | 16 byte-identical, 2 cosmetic |
| T1 / T2 / T3 | re-proved |

Raw compiler output: [`data/W583-c-compile-errors.tsv`](data/W583-c-compile-errors.tsv).

---

## 6. Three cooperation variants for W584

### Variant A (recommended) — The array-typedef hoisting

```c
typedef struct { [u8 v[16];16]; } t27_arr__u8_16__16;
```

70 headers fail on this and its family (`type name requires a specifier`, 38; `type
specifier missing`, 32). It is a formatting bug in the hoisted `[T; N]` struct — the
element type and the length are being interpolated in the wrong order — and unlike the
other two classes it is neither a pending decision nor a missing feature.

**Metric:** headers compiling, 101 → target ~170.

### Variant B — `use` resolution for the C backend

64 headers fail on types declared in a module they import. W569 built exactly this for
Zig (`use_resolve.rs`, selective splicing with collision detection); it operates on
source text before the compiler, so it is backend-agnostic and may need only to be
called from `run_gen_c`.

**What would falsify it.** If `run_gen_c` already routes through the same resolver and
the failures are something else, the 64 are a different problem — check one before
building.

### Variant C — Flash the board

Unchanged. Bitstream at 150.63 MHz, `fpga-flash --dry-run` correctly reporting
`BLOCKED -- no programmer on USB`, three theorems re-proved.

---

## Recommendation

**Variant A.** It is the largest class that is unambiguously a compiler bug, it needs no
decision from anyone, and the gate to measure it now exists.

---

*φ² + φ⁻² = 3 | TRINITY*
