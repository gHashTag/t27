# t27b: t27's own native backend (MVP)

`t27b` compiles a subset of t27 straight to AArch64 machine code. It does not
call LLVM, zig, clang or rustc, at build time or at run time. It produces two
outputs:

* `t27b test`: an in-process JIT test runner. The code is written into one
  executable region (`MAP_JIT` on macOS; `mmap` RW, then `mprotect` RX on
  Linux) and the module's `test` blocks run there.
* `t27b build -o out.o`: a Mach-O `MH_OBJECT` for arm64. It links with
  `cc driver.c out.o`.

The JIT runs on arm64 macOS and arm64 Linux. The Railway t27b lab
(`contrib/railway/t27b-lab`) runs the Linux build under qemu-user on x86_64.

The front-end is t27c's own, unmodified. `bootstrap/src/compiler.rs` and
`bootstrap/src/use_resolve.rs` are mounted with `#[path]`, so
`bootstrap/stage0/FROZEN_HASH` still holds. A program that t27c rejects is
rejected here with the same message. The crate's one dependency is `serde`,
and only because `compiler.rs` derives `Serialize`. The JIT declares its own
`extern "C"` for `mmap`, `munmap`, `pthread_jit_write_protect_np` and
`sys_icache_invalidate`.

Outside the subset, `t27b` stops with exit code 2 and names the construct and
the line:

```
t27b: unsupported construct StructDecl at line 2
t27b: unsupported construct ExprUnary(-) at line 6 (negation of u32)
```

It never guesses. Within the subset, every result is cross-checked by a
reference interpreter (see Verification).

## Architecture

```
  .t27 source
      |
      v
  +----------------------------------------------------------------+
  | t27c front-end, byte-for-byte (src/front.rs is the glue)       |
  |   use splicing -> lexer/parser -> AST -> typecheck_ast          |
  +----------------------------------------------------------------+
      | compiler::Node
      v
  lower.rs   name resolution, Zig-like integer typing, desugaring of
             compound assignment / unary minus / test bindings; every
             construct outside the subset is rejected here (exit 2)
      | ir.rs: typed tree; every operation that can trap carries a site
      +-------------------------------+
      v                               v
  eval.rs                         codegen.rs + a64.rs
  reference interpreter           one pass per function, straight to
  (i128 arithmetic; used by       32-bit instruction words; a64.rs is the
  --check, corpus, the            encoder plus a disassembler for `asm`
  differential test)                  |
                        +-------------+-------------+
                        v                           v
                    jit.rs                      macho.rs
        one MAP_JIT image:                  MH_OBJECT, one __TEXT,__text,
        [enter][trap_common][fns]           one external symbol `_name`
        pthread_jit_write_protect_np,       per function, no relocations
        sys_icache_invalidate               (bl offsets resolved at emit)
        -> t27b test                        -> t27b build -o out.o
```

| File | Role |
|---|---|
| `src/front.rs` | Calls t27c's parser and `typecheck_ast`; `use` splicing as in every t27c `gen-*` path |
| `src/lower.rs` | AST to IR: scopes, types, the subset check |
| `src/ir.rs` | The IR: `Ty`, `Expr`, `Stmt`, trap sites |
| `src/eval.rs` | Reference interpreter; the semantics oracle |
| `src/codegen.rs` | Instruction selection, register assignment, frames, trap stubs, linking |
| `src/a64.rs` | AArch64 encoders and the disassembler |
| `src/jit.rs` | `MAP_JIT` image, the `enter` trampoline and `trap_common` |
| `src/macho.rs` | Hand-written Mach-O 64 relocatable object |
| `src/main.rs` | Command line, `corpus` runner |
| `src/timing.rs` | `--time` phase clock |

### Code generation

* **Canonical register form.** A value of 32 bits or less lives in the low 32
  bits of its register:
  * u8, u16 and bool are zero-extended;
  * i8 and i16 are sign-extended;
  * the upper 32 bits are undefined.

  64-bit values use the whole X register. Narrow arithmetic is done in W
  registers, then re-normalised or overflow-checked.
* **Registers.**

  | Registers | Use |
  |---|---|
  | x0-x7 | Arguments and result |
  | x8 | Scratch |
  | x9-x15 | Expression temp stack; deeper temps spill to the frame |
  | x16/x17 | Scratch for spills and constants |
  | x18 | Never touched (Apple's platform register) |
  | x19-x28 | Variable homes, chosen by loop-weighted use count |

  There is no general register allocator.
* **Frame.** `stp x29, x30, [sp, #-16]!`. Leaf functions that need no frame
  get none.
* **Checks.** Overflow uses `adds`/`subs` + `b.vs`/`b.hs`/`b.lo`, or, for
  multiplication, `smull`/`umull` with an extend-compare (`smulh`/`umulh` at
  64 bits). Narrow types use an extend-compare. Division gets
  `cbz` (divide by zero) and a MIN/-1 check. Shifts get an unsigned range
  compare on the amount. A checked cast compares the operand with its own
  low bits extended (`cmp x, w, sxtb` and the like) and, where only the
  sign can be wrong, with zero.

  Each check branches to an out-of-line stub at the end of the function:
  * in the JIT, the stub records the site and jumps to `trap_common`, which
    unwinds to `enter` (no signals);
  * in an object file, the stub is `brk #kind`.

  | `brk` code | Trap |
  |---|---|
  | 1 | overflow |
  | 2 | divide by zero |
  | 3 | shift range |
  | 4 | assert |
  | 5 | assert_eq |
  | 6 | missing return |
  | 7 | cast out of range |

Example: `t27b asm` on one benchmark function. Left is trap mode (the
default); right is `--overflow wrap`.

```
_k0:  ; line 3                          _k0:  ; line 3
   0: adds w2, w0, #0                      0: add w2, w0, #0
   4: b.hs 0x48                            4: movz w1, #0x0
   8: movz w1, #0x0                        8: b 0x24
   c: b 0x38                               c: movz w17, #0x3
  10: movz w17, #0x3                      10: mul w9, w2, w17
  14: umull x9, w2, w17                   14: add w9, w9, w1
  18: cmp x9, w9, uxtw                    18: lsr w10, w2, #2
  1c: b.ne 0x48                           1c: eor w2, w9, w10
  20: adds w9, w9, w1                     20: add w1, w1, #1
  24: b.hs 0x48                           24: cmp w1, #8
  28: lsr w10, w2, #2                     28: b.lo 0xc
  2c: eor w2, w9, w10                     2c: mov x0, x2
  30: adds w1, w1, #1                     30: ret
  34: b.hs 0x48
  38: cmp w1, #8
  3c: b.lo 0x10
  40: mov x0, x2
  44: ret
  48: brk #0x1
```

## Supported subset

* `module Name; ... endmodule`, or a file with no module (it is named after
  the file). Module-level `const` with a comptime integer or bool value.
  `use` is spliced exactly as t27c does it.
* `fn name(a: T, ...) -> T { ... }` with at most 8 parameters, or no return
  type. The types are:
  * `bool`;
  * `u8`, `u16`, `u32`, `u64`;
  * `i8`, `i16`, `i32`, `i64`;
  * `usize` (= `u64`) and `isize` (= `i64`).
* Statements:
  * `var` / `const` locals;
  * `=` and the compound assignments `+= -= *= /= %= &= |= ^=` (the t27c
    parser has no `<<=`, `>>=` or `+%=`);
  * `if` / `else`;
  * `while (c) { }` and `while (c) : (step) { }`;
  * `break`, `continue`, `return`;
  * calls to the module's own functions, including as statements.
* Expressions:
  * integer and `true`/`false` literals;
  * `+ - * / %`, the wrapping `+% -% *%`, and `& | ^ << >>`;
  * `== != < <= > >=`;
  * `and or && ||`, which short-circuit;
  * unary `!` (bool), `~` (integers) and `-` (signed integers).
* `test name { ... }` blocks with `assert(c)` and `assert_eq(a, b)`. As in
  t27c's Zig backend, the first plain assignment to a fresh name in a test
  block declares it.

Everything else is rejected by name. That includes structs, enums, floats,
strings, slices and arrays, casts to or from a float or to `bool`,
`invariant` and `bench` blocks, builtins
(`@...`) and labelled loops. The corpus section lists what that rejects in
practice.

## Integer semantics

All of these are implemented identically in `eval.rs` and in the generated
code, and the differential test checks them trap for trap.

* **Typing is Zig-like.**
  * An untyped literal is a comptime integer: it takes the other operand's
    type and must fit in it.
  * Mixed types are allowed only if one widens losslessly to the other: same
    signedness and not narrower, or unsigned into a strictly wider signed
    type. `u32 + i32` is rejected; `u32 + i64` is `i64`.
  * Conditions must be `bool`.
  * `bool` and integers never mix.
  * Ordering comparisons on `bool` are rejected.
* **Overflow traps by default.**
  * `+ - *` trap when the result does not fit the type.
  * `--overflow wrap` turns these into two's-complement wrapping for the whole
    module.
  * `+% -% *%` always wrap.
  * Unary `-x` is `0 - x` on signed types, so `-MIN` traps. On an unsigned type
    `-x` is rejected; write `0 -% x`.
* **Division.**
  * `/` truncates toward zero.
  * A zero divisor traps in both modes.
  * Signed `MIN / -1` traps in trap mode and gives `MIN` in wrap mode.
* **Remainder.**
  * `%` truncates: the result has the sign of the dividend, and `-7 % 2 == -1`.
  * `MIN % -1` is 0.
  * A zero divisor traps.
  * `%` is accepted on signed types. The Zig backend refuses that; see below.
* **Shifts.**
  * `x << n` and `x >> n` have the type of `x`. `>>` is arithmetic on signed
    types and logical on unsigned.
  * The amount may be any integer type. In trap mode, an amount outside
    `[0, bits)` traps, and bits shifted out of the value are discarded (no
    overflow check on `<<`). In wrap mode the amount is masked to `bits - 1`.
  * A constant amount out of range is rejected at compile time in trap mode and
    masked in wrap mode.
  * An untyped literal shifted by a runtime amount (`1 << n`) is rejected,
    because its width would be a guess.
* **Casts.** `x as T` converts an integer or `bool` to integer `T`.
  * A lossless one is a plain widening, and `bool` becomes 0 or 1.
  * Between two unsigned types, a narrowing keeps the low bits, like Zig's
    `@truncate`: `300 as u8` from a `u32` is 44.
  * Every other conversion is checked, like `@intCast`, and traps when the
    value is outside `T` (`brk #7`): `-1 as u8`, `200 as i8`,
    `0x8000_0000 as i32` from a `u32`.
  * In wrap mode every narrowing keeps the low bits, as a C cast does.
  * A literal must fit `T`. `x as bool` is rejected, as Zig rejects it.
  * The Zig backend decides between `@truncate` and `@intCast` by the shape of
    the operand: an expression it can prove unsigned, such as a typed
    variable, a typed literal, a cast, or an arithmetic or shift of these.
    t27b decides by the operand's type. They differ only for an unsigned
    operand of another shape, such as a call result, that is out of range:
    t27b truncates it and the Zig backend panics. It also emits `@intCast` on
    a `bool`, which does not compile.
* **Missing return.** A function with a result type that falls off its end
  traps (`brk #6`).

### The three backends do not agree today

A probe module, `sem.t27`:

```
module Sem;

fn inc(x: i32) -> i32 {
    return x + 1;
}

fn shr(x: i32, n: i32) -> i32 {
    return x >> n;
}

fn rem(a: i32, b: i32) -> i32 {
    return a % b;
}

test inc_max { assert_eq(inc(2147483647), 0 - 2147483647 - 1); }
test shr_big { assert_eq(shr(1024, 40), 0); }
test rem_neg { assert_eq(rem(0 - 7, 2), 0 - 1); }

endmodule
```

Each cell is the outcome of that test (`inc_max`, `shr_big`, `rem_neg`):

| | `inc(MAX)` | `shr(1024, 40)` | `rem(-7, 2)` |
|---|---|---|---|
| t27b (trap mode) | trap: integer overflow, line 4 | trap: shift amount out of range, line 8 | -1, pass |
| t27c `gen` + `zig test` (Debug) | panic: integer overflow | panic: integer does not fit in destination type (the `@intCast` of the amount) | does not compile: signed `%` must use `@rem` or `@mod` |
| t27c `gen-c` + clang -O0 | wraps to MIN, test **passes** | hardware masks the amount: 4, test **fails** | -1, pass |
| t27c `gen-c` + clang -O2 | test **fails** | test **passes** | -1, pass |

* **t27b agrees with the Zig backend** on overflow and on shift range, and it
  also has a defined `%` on signed types.
* **`gen-c` emits plain C.** It writes `x + 1`, `x >> n` and `a % b`, so two
  of the three results depend on undefined behaviour.
  * At -O2 clang assumes the overflow and the oversized shift cannot happen
    and folds the inlined test comparisons. `inc_max` and `shr_big` swap
    outcomes between -O0 and -O2.
  * Called from a separate object, both levels return MIN and 4.
  * A failing `gen-c` test is a bare `__builtin_trap()` (SIGTRAP, exit 133),
    with no test name or values.
* **A real bug this found:** `specs/port/tools/ternary_model.t27`
  (`dot27_test`) shifts an `i32` by `i*2` with `i` up to 26. t27b reports
  "shift amount out of range at line 40".

## Usage

```
t27b test   <file.t27> [--overflow trap|wrap] [--time] [--quiet] [--check] [--blockers]
t27b build  <file.t27> -o <out.o> [--overflow trap|wrap] [--time]
t27b asm    <file.t27> [--overflow trap|wrap]
t27b corpus <dir> [--timeout-ms N] [--jobs N] [--overflow trap|wrap] [--list]
                  [--json <path>] [--runner "<cmd> [args]"]
                  [--blockers [--reference <t27c> [--reference-cache <file>]
                               [--reference-timeout-ms N]]]
```

Options:

* `--time`: prints per-phase milliseconds to stderr: read, parse, typecheck,
  lower, codegen, jit-map / emit, run / write.
* `--check`: runs every test in the interpreter too, and fails with exit 4 if
  the two disagree.
* `corpus`: runs `test --quiet --check` on every `.t27` under a directory, one
  process per file, with a timeout. It prints:
  * supported / rejected / front-end-error / failed / crash counts;
  * the 15 most common rejecting constructs.
* `corpus --json <path>`: also writes the same totals, every rejecting
  construct, and one record per file (`file`, `reference`, `t27b`: pass /
  fail / blocked / frontend / mismatch / codegen / timeout / crash, `tests`,
  `invariants`, `blockers`, `detail`). With `--reference`, each record's
  `reference` is the reference path's verdict (`pass` / `blocked` / `fail` /
  `timeout`, reason in `reference_detail`) and `totals.reference.ran` is
  true; without it, `reference` is `skip`.
* `corpus --runner "<cmd> [args]"`: starts each per-file `t27b test` as
  `<cmd> [args] <t27b> test ...`. Under qemu-user without binfmt_misc the
  driver cannot exec its own aarch64 binary; the lab passes
  `--runner "qemu-aarch64 -L /usr/aarch64-linux-gnu"`.
* `--blockers`: lists every unsupported construct of a file, not only the
  first. With `corpus`, it also prints the greedy order in which supporting
  constructs unlocks the most whole files. See "Blockers" below.
* `--reference <t27c>` (with `corpus`): also runs the reference path,
  `t27c test-report <spec>` (t27c's Zig backend, one test per process), on
  every file. A spec the reference path cannot compile or pass is listed and
  left out of the denominator, because it does not count against t27b.
  `--reference-cache <file>` keeps the verdicts, keyed by spec path, spec
  source and the t27c binary, so a second sweep is instant.

```
$ cargo build --release -p t27b
$ ./target/release/t27b test sem.t27
FAIL inc_max: integer overflow at line 4 (+ on i32)
FAIL shr_big: shift amount out of range at line 8 (>> on i32)
PASS rem_neg
Sem: 1 passed, 2 failed, 3 total
$ ./target/release/t27b build bench.t27 -o bench.o && cc driver.c bench.o -o driver
```

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | A test failed |
| 2 | Unsupported construct |
| 3 | Front-end (parse or typecheck) error |
| 4 | JIT and interpreter disagree (`--check`) |
| 5 | Code generation limit (frame larger than 32 KB, branch out of range) |
| 64 | Usage |
| 74 | I/O error |

## Verification

```
cargo test --release -p t27b
```

### `tests/encoder.rs`

* 97 instruction words, each assembled by Apple clang and read back with
  `otool`/`objdump`, compared word for word with the encoders.
* The disassembler reads back the whole subset.
* `mov_imm` output is executed by a small emulator over more than 80,000
  values (64- and 32-bit) and must materialise each one in at most 4 words.
* The bitmask-immediate encoder finds exactly the 5334 64-bit and 1302 32-bit
  encodable values. Each one round-trips through the decoder.

### `tests/differential.rs`

Seeded random IR programs, run JIT against interpreter. Each comparison checks:

* the value, and that it is in canonical register form; or
* for a trap, the exact trap site; for `assert_eq`, both operands.

The generator aims at the hard shapes:

* temp stacks deep enough to spill;
* calls with live temps;
* eight-argument calls;
* garbage in the upper halves of narrow argument registers;
* every division and shift corner;
* loops with `break`/`continue`.

Knobs:

| Variable | Default | Effect |
|---|---|---|
| `T27B_DIFF_CASES` | 2500 | Programs per overflow mode |
| `T27B_DIFF_SEED` | 0x7427 | Base seed |
| `T27B_DIFF_TRACE` | unset | Print each seed before running it |
| `T27B_DIFF_ONE=<seed>` | unset | Replay one case |

Result on the final tree: 7 tests, all pass. The differential file takes
7.3 s in release mode.

| Test | Programs | Calls compared | Returned a value | Trapped |
|---|---|---|---|---|
| `random_programs_jit_matches_interpreter`, trap mode | 2500 (11037 functions) | 49622 | 16143 | 33479: overflow 17595, divide by zero 2398, shift 6092, assert 2154, assert_eq 4285, missing return 955 |
| `random_programs_jit_matches_interpreter`, wrap mode | 2500 (11231 functions) | 50816 | 29601 | 21215: divide by zero 7546, assert 4638, assert_eq 7056, missing return 1975 |
| `every_operator_at_edge_values` | 256 | 114594 | 94664 | 19930 |
| `compare_unary_widen_at_edge_values` | 77 | 44611 | | |

There were 0 mismatches, and 0 programs were skipped for running out of fuel.

To check that the test can fail, three bugs were planted in `codegen.rs` one
at a time. Each was caught, and `codegen.rs` was then restored:

* a call that forgets to save one live temp;
* two spilled temps that share a frame slot;
* signed `<=` emitted as `<`.

### `bench/bench.py diff`

The same synthetic program in t27 and in C (see Benchmarks), linked into one
driver. Each function is called on 1005 inputs: 0, 1, 7, `0x7FFFFFFF`,
`0xFFFFFFFF` and 1000 xorshift values. Two comparisons are made:

* `t27b build --overflow wrap` against `clang -O2`: the values must be equal.
* `t27b build` (trap mode) against `clang -O0
  -fsanitize=unsigned-integer-overflow,signed-integer-overflow,shift
  -fsanitize-trap=all`: the results must be the same value, or both must trap.

Result (`bench.py diff 100 1000 5000`, exit 0):

| N | Comparison | Calls | Both trapped | Mismatches |
|---|---|---|---|---|
| 100 | wrap vs clang -O2 | 100500 | 0 | 0 |
| 100 | trap vs clang -O0 + UBSan trap | 100500 | 100200 | 0 |
| 1000 | wrap vs clang -O2 | 1005000 | 0 | 0 |
| 1000 | trap vs clang -O0 + UBSan trap | 1005000 | 1002000 | 0 |
| 5000 | wrap vs clang -O2 | 5025000 | 0 | 0 |
| 5000 | trap vs clang -O0 + UBSan trap | 5025000 | 5010000 | 0 |

The trap-mode half is weaker than its call count suggests. The loop
multiplies by 3 eight times, so every input except 0, 1 and 7 overflows
`u32`. That half therefore checks that the overflow is detected, and checks
values for only 3 inputs per function. The values themselves are covered by
the wrap-mode half and by the differential test above.

### Corpus

```
./target/release/t27b corpus specs --jobs 6
```

Result on master at 57c2cfaf2: 1185 files, about 40 s with `--jobs 3`, exit 0.

| Outcome | Files |
|---|---|
| Supported, all tests pass | 36 (36 tests in total; 26 of the files have no `test` block) |
| Supported, a test fails | 0 (`specs/port/tools/ternary_model.t27`, which failed with "shift amount out of range", was fixed in the spec) |
| Rejected (unsupported construct, exit 2) | 1129 |
| Front-end error (t27c's own parser or typechecker rejects the file) | 20 |
| JIT / interpreter mismatch | 0 |
| Codegen limit | 0 |
| Timeout (10 s) | 0 |
| Crash | 0 |

So t27b handles 36 of the 1185 specs (3.0%). The table below lists the first
construct that stopped each file. "Files" counts those first rejections.
"Items" counts every top-level item (function, test, declaration) whose
lowering stopped at that construct.

| Construct | Files | Items |
|---|---|---|
| `StructDecl` | 393 | 1569 |
| `ExprLiteral(string literal)` | 288 | 6344 |
| `EnumDecl` | 89 | 272 |
| `InvariantBlock` | 78 | 6141 |
| `type []T` | 60 | 1230 |
| `type str` | 31 | 532 |
| `type f64` | 28 | 326 |
| `ExprBinary(<< >>)` | 24 | 35 |
| `ExprCast` | 24 | 252 |
| `ExprCall(assert with message)` | 18 | 70 |
| `type [N]T` | 17 | 361 |
| `FnDecl` | 13 | 22 |
| `StmtExpr` | 12 | 27 |

What the smaller rows refuse:

| Construct | What was refused |
|---|---|
| `ExprBinary(<< >>)` | An untyped literal shifted by a runtime amount (`1 << n`) |
| `ExprCall(assert with message)` | `assert(c, "message")`, which has 2 arguments |
| `FnDecl` | A function with more than 8 parameters |
| `StmtExpr` | Not in the source. t27c's parser reads a dotted `module a.b;` or `use std.testing;` as `a` followed by a stray top-level expression `.b`, and records no line for it, so t27b reports line 0 |

Construct names carry their shape where one kind covers several things: a
type is `type []T`, `type [N]T`, `type (struct)` or `type (alias)` rather than
each element type, and a call t27b cannot resolve is `ExprCall(std.*)`,
`ExprCall(method)` or `ExprCall(undeclared fn)`.

### Blockers: what each file needs, and in what order

The first rejection says what stops a file, not what it would take to pass
it. `--blockers` keeps lowering after a rejection and names every construct
in the file, once per kind. The value of a rejected expression is "poison":
anything built from it is dropped without a second report, and so is a use of
a rejected declaration. One missing construct is counted once, not once per
use.

```
./target/release/t27b corpus specs --blockers --jobs 3
```

On the same tree: 1129 files are blocked by 169 distinct constructs. A file
needs between 1 and 35 of them (median 5). A rejected `struct` or `enum`
declaration counts once; its members are not inspected.

The greedy order picks, at each step, the construct that lets the most whole
files pass once it is supported. When no construct completes a file on its
own, it picks the one that removes the most outstanding work (the sum of
1 / remaining constructs over the files that need it). "Need" is the number
of files that use the construct at all.

| Step | Construct | +files | Passing | Need |
|---|---|---|---|---|
| 1 | `ExprCast` | 18 | 54 | 187 |
| 2 | `ExprBinary(<< >>)` | 27 | 81 | 47 |
| 3 | `ExprCall(assert with message)` | 18 | 99 | 27 |
| 4 | `StructDecl` | 17 | 116 | 512 |
| 5 | `InvariantBlock` | 14 | 130 | 367 |
| 6 | `FnDecl` | 12 | 142 | 24 |
| 7 | `BenchBlock` | 5 | 147 | 247 |
| 8 | `StmtAssign(undeclared)` | 22 | 169 | 74 |
| 9 | `ExprLiteral(negative literal)` | 3 | 172 | 38 |
| 10 | `ExprUnary(try) statement` | 3 | 175 | 51 |
| 13 | `EnumDecl` | 2 | 183 | 136 |
| 18 | `ExprStructLit` | 8 | 197 | 361 |
| 21 | `type []T` | 7 | 211 | 313 |
| 22 | `ExprLiteral(string literal)` | 10 | 221 | 625 |
| 23 | `type str` | 52 | 273 | 463 |
| 25 | `type [N]T` | 18 | 301 | 506 |
| 26 | `ExprArrayLiteral` | 333 | 634 | 558 |
| 41 | `type f64` | 59 | 826 | 135 |
| 42 | `type f32` | 30 | 856 | 66 |

The scalar constructs come first because they are cheap and complete files.
The memory constructs (structs, strings, slices, arrays) are needed by more
files, but each of those files also needs several others. `ExprArrayLiteral`
completes 333 files at step 26, once the 25 constructs before it are in.

`--reference <t27c>` adds the reference path to the same run: `t27c
test-report` on each file, which runs `t27c gen` and then `zig test`. Files
the reference itself fails are counted apart, and a second greedy order
counts only the files the reference passes. Without that, a spec that does
not even compile through Zig would count as a t27b gap. `--reference-cache
<file>` keeps those results between runs, keyed by the file's content and the
t27c binary. A full reference run takes about 2.5 hours with `--jobs 5`.

## Benchmarks

```
WORK=/tmp/t27b-bench T27C=./target/release/t27c python3 cli/t27b/bench/bench.py metrics 5 100 1000 5000
WORK=/tmp/t27b-bench T27C=./target/release/t27c python3 cli/t27b/bench/bench.py runtime 5 100 1000 5000
WORK=/tmp/t27b-bench python3 cli/t27b/bench/bench.py phases 15 100 1000 5000
WORK=/tmp/t27b-bench python3 cli/t27b/bench/bench.py diff 100 1000 5000
WORK=/tmp/t27b-bench python3 cli/t27b/bench/bench.py build 3 [t27b|t27c]
```

The synthetic program has N functions `k_i(x: u32) -> u32`. Each runs an
8-iteration loop of `u32` multiply, add, xor and shift, and each has one
`assert_eq` test. The same program is written in t27 (`bench.t27`) and in C
(`bench.c`), so the C file is a hand-written reference that does not depend on
t27c.

How the numbers were measured:

* **Wall time:** the median of 5 measured runs, after 1 warm-up. The cases
  are interleaved within each run.
* **CPU:** the median of user + sys of the process and its children, from
  `getrusage`.
* **RSS:** from `/usr/bin/time -l`.

**Machine and load.** All numbers below come from one Apple M1 Pro (8
cores: 6 performance, 2 efficiency) with 16 GB of RAM. The toolchain was
macOS 26.5.2, Apple clang 21.0.0, Zig 0.16.0 and rustc 1.98.1.

The machine was shared with several other build agents the whole time. The
1-minute load average was:

| Benchmark | Load average |
|---|---|
| `metrics` | 83-165 |
| `phases` | 131-137 |
| `runtime` | 144-180 |
| clean builds | 244-336 |

Wall medians are therefore inflated, and the longer a step runs the noisier
it gets. The minimum and the CPU time are the more robust numbers, so the
tables give all three.

The `./ctest` wall time includes 0.5-1 s that is not the program: it uses
under 8 ms of CPU. This is most likely macOS assessing a freshly linked
executable on its first launch.

### Test a module: `t27b test` vs `t27c gen` + `zig test`

`t27b test` JITs the module and runs all N tests. It is compared with two
paths through t27c:

* **The Zig path:** `t27c gen`, then `zig test` in Debug mode. Each run gets
  a fresh local cache; the global cache stays warm.
* **The gen-c path:** `t27c gen-c`, then `clang -O0 -DT27_TEST_MAIN`
  (compile and link), then running the binary.

| N | Step | Wall median ms | Wall min ms | CPU ms | Peak RSS MB |
|---|---|---|---|---|---|
| 100 | `t27b test` | 17.8 | 15.8 | 11.4 | 5.7 |
| 100 | `t27c gen` | 26.5 | 25.3 | 16.8 | 13.0 |
| 100 | `zig test g.zig` | 6771.3 | 5476.2 | 2777.3 | 247.1 |
| 100 | **t27c gen + zig test** | **6797.9** | **5501.6** | **2794.1** | 247.1 |
| 100 | `t27c gen-c` | 22.9 | 21.9 | 15.7 | 12.6 |
| 100 | `clang -O0 -DT27_TEST_MAIN g.c` (compile + link) | 165.1 | 154.4 | 141.5 | 44.4 |
| 100 | `./ctest` | 970.3 | 753.7 | 6.7 | 1.4 |
| 100 | **t27c gen-c + clang -O0 + run** | **1158.3** | **930.0** | **163.8** | 44.4 |
| 1000 | `t27b test` | 59.6 | 51.1 | 50.4 | 32.1 |
| 1000 | `t27c gen` | 62.6 | 60.4 | 55.0 | 36.8 |
| 1000 | `zig test g.zig` | 6061.1 | 3102.9 | 3339.6 | 316.8 |
| 1000 | **t27c gen + zig test** | **6123.7** | **3163.3** | **3394.6** | 316.8 |
| 1000 | `t27c gen-c` | 68.0 | 39.1 | 53.5 | 35.6 |
| 1000 | `clang -O0 -DT27_TEST_MAIN g.c` (compile + link) | 394.1 | 280.2 | 373.4 | 84.1 |
| 1000 | `./ctest` | 726.8 | 488.1 | 6.3 | 1.5 |
| 1000 | **t27c gen-c + clang -O0 + run** | **1188.9** | **807.4** | **433.3** | 84.1 |
| 5000 | `t27b test` | 916.7 | 250.1 | 308.2 | 147.2 |
| 5000 | `t27c gen` | 570.3 | 239.4 | 285.4 | 144.1 |
| 5000 | `zig test g.zig` | 20488.2 | 4765.8 | 7403.6 | 604.3 |
| 5000 | **t27c gen + zig test** | **21058.5** | **5005.2** | **7689.0** | 604.3 |
| 5000 | `t27c gen-c` | 364.3 | 225.9 | 252.1 | 138.1 |
| 5000 | `clang -O0 -DT27_TEST_MAIN g.c` (compile + link) | 3579.5 | 1374.2 | 1700.2 | 255.0 |
| 5000 | `./ctest` | 585.4 | 472.6 | 7.8 | 2.3 |
| 5000 | **t27c gen-c + clang -O0 + run** | **4529.2** | **2072.7** | **1960.1** | 255.0 |

| N | zig path / t27b, wall median | wall min | CPU | gen-c path / t27b, wall median | CPU |
|---|---|---|---|---|---|
| 100 | 381x | 349x | 246x | 65x | 14.4x |
| 1000 | 103x | 62x | 67x | 20x | 8.6x |
| 5000 | 23x | 20x | 25x | 5x | 6.4x |

* **Zig is mostly fixed cost and noise.** Its N = 100 runs were slower than
  its N = 1000 runs (min 5.5 s against 3.1 s), so the 381x at N = 100 is
  mostly the fixed cost of a `zig test` build under load. The CPU column is
  the steadier one.
* **At N = 5000 most of the cost is the front-end,** which t27b shares with
  t27c. `t27b test` (min 250 ms) costs about the same as `t27c gen` alone
  (min 239 ms). See the phase table below.

### Build an object: `t27b build` vs `t27c gen-c` + clang

`t27b build` writes a Mach-O object. It is compared with `t27c gen-c`
followed by `clang -c`, and with clang compiling the hand-written `bench.c`.

| N | Step | Wall median ms | Wall min ms | CPU ms | Peak RSS MB |
|---|---|---|---|---|---|
| 100 | `t27b build` (trap) | 21.4 | 17.7 | 11.3 | 5.7 |
| 100 | `t27b build --overflow wrap` | 17.3 | 16.6 | 11.7 | 5.6 |
| 100 | `t27c gen-c` | 22.9 | 21.9 | 15.7 | 12.6 |
| 100 | `clang -O0 -c g.c` | 97.4 | 73.3 | 70.5 | 41.6 |
| 100 | `clang -O2 -c g.c` | 278.3 | 189.7 | 178.0 | 55.5 |
| 100 | **t27c gen-c + clang -O0 -c** | **120.3** | **95.2** | **86.2** | 41.6 |
| 100 | **t27c gen-c + clang -O2 -c** | **301.2** | **211.7** | **193.7** | 55.5 |
| 100 | `clang -O0 -c bench.c` (hand-written C) | 74.4 | 68.9 | 53.6 | 39.8 |
| 100 | `clang -O2 -c bench.c` (hand-written C) | 192.6 | 166.8 | 143.8 | 50.2 |
| 1000 | `t27b build` (trap) | 62.9 | 50.4 | 48.9 | 31.6 |
| 1000 | `t27b build --overflow wrap` | 63.5 | 51.2 | 50.0 | 31.3 |
| 1000 | `t27c gen-c` | 68.0 | 39.1 | 53.5 | 35.6 |
| 1000 | `clang -O0 -c g.c` | 502.5 | 285.6 | 269.8 | 80.6 |
| 1000 | `clang -O2 -c g.c` | 1712.1 | 1142.0 | 1287.3 | 143.1 |
| 1000 | **t27c gen-c + clang -O0 -c** | **570.5** | **324.7** | **323.3** | 80.6 |
| 1000 | **t27c gen-c + clang -O2 -c** | **1780.1** | **1181.1** | **1340.8** | 143.1 |
| 1000 | `clang -O0 -c bench.c` (hand-written C) | 190.2 | 163.3 | 166.0 | 73.5 |
| 1000 | `clang -O2 -c bench.c` (hand-written C) | 1423.3 | 797.1 | 1081.9 | 100.1 |
| 5000 | `t27b build` (trap) | 822.7 | 212.9 | 274.9 | 143.8 |
| 5000 | `t27b build --overflow wrap` | 860.1 | 214.4 | 274.9 | 144.2 |
| 5000 | `t27c gen-c` | 364.3 | 225.9 | 252.1 | 138.1 |
| 5000 | `clang -O0 -c g.c` | 3483.5 | 1016.8 | 1603.5 | 246.8 |
| 5000 | `clang -O2 -c g.c` | 20024.6 | 4717.6 | 8167.9 | 447.6 |
| 5000 | **t27c gen-c + clang -O0 -c** | **3847.7** | **1242.7** | **1855.6** | 246.8 |
| 5000 | **t27c gen-c + clang -O2 -c** | **20388.9** | **4943.5** | **8420.0** | 447.6 |
| 5000 | `clang -O0 -c bench.c` (hand-written C) | 1496.0 | 642.5 | 933.3 | 219.8 |
| 5000 | `clang -O2 -c bench.c` (hand-written C) | 10880.1 | 5317.8 | 6652.6 | 244.3 |

| N | gen-c + clang -O0 / t27b build, wall median | CPU | gen-c + clang -O2 / t27b build, wall median | CPU |
|---|---|---|---|---|
| 100 | 5.6x | 7.6x | 14.1x | 17.1x |
| 1000 | 9.1x | 6.6x | 28.3x | 27.4x |
| 5000 | 4.7x | 6.7x | 24.8x | 30.6x |

### Where t27b spends its time

These are the `--time` phases inside one process, from
`bench.py phases 15 100 1000 5000`. "parse" includes `use` splicing, and
"parse" and "typecheck" are t27c's own code.

Minimum of 15 runs, ms (median in parentheses).

| N | Command | read | parse | typecheck | lower | codegen | jit-map / emit | run / write | total | parse + typecheck share |
|---|---|---|---|---|---|---|---|---|---|---|
| 100 | `t27b test` | 0.05 (0.06) | 2.31 (2.58) | 0.54 (0.62) | 0.43 (0.50) | 0.30 (0.34) | 0.04 (0.04) | 0.01 (0.01) | 3.79 (4.22) | 75% |
| 100 | `t27b build` | 0.05 (0.06) | 2.35 (2.58) | 0.55 (0.60) | 0.45 (0.49) | 0.15 (0.16) | 0.03 (0.03) | 0.20 (0.23) | 3.86 (4.74) | 75% |
| 1000 | `t27b test` | 0.10 (0.11) | 22.54 (25.45) | 10.78 (12.26) | 5.50 (6.07) | 2.64 (3.04) | 0.18 (0.20) | 0.11 (0.12) | 42.96 (48.14) | 78% |
| 1000 | `t27b build` | 0.10 (0.12) | 23.94 (29.68) | 10.30 (12.69) | 5.69 (6.78) | 1.28 (1.49) | 0.19 (0.21) | 0.26 (0.31) | 43.16 (56.44) | 79% |
| 5000 | `t27b test` | 0.30 (0.34) | 121.56 (157.18) | 73.94 (90.75) | 29.34 (37.80) | 14.12 (17.11) | 0.87 (0.99) | 0.57 (0.61) | 262.45 (325.75) | 74% |
| 5000 | `t27b build` | 0.31 (0.38) | 129.48 (139.81) | 73.38 (91.84) | 31.40 (37.35) | 6.62 (7.82) | 0.94 (1.02) | 0.33 (0.49) | 256.40 (294.21) | 79% |

At every size, 74-79% of the time goes to parse and typecheck, which are
t27c's front-end. The t27b-specific phases cost 0.8 ms at N = 100 and 44 ms at
N = 5000 (39 ms for `build`). Those phases are lower, codegen and
jit-map / emit; jit-map / emit also covers the `mmap`, the W^X toggle and the
icache flush.

* `build` codegen is about half of `test` codegen, because `build` does not
  emit the N test functions.
* Parse, lower and codegen grow roughly linearly with N.
* Typecheck, which is t27c's, grows faster: 20x from N = 100 to 1000, then
  6.9x from 1000 to 5000.

### Generated code: `__text` size and ns/call

All six objects are linked into one timing driver, which calls each `k_i`
through a function-pointer table so that nothing is inlined across objects.
Each `k_i` is called on the inputs 0-1023.

The time is thread CPU time, at QoS user-interactive, with 15 samples in each
of 5 processes. "min" is the fastest sample; "median" is the median of the
per-process medians.

Every object produced the same checksum: 2484369019 at N = 100, 813456388 at
N = 1000 and 3592169015 at N = 5000. The `.o` sizes include the symbol table.

| N | Object | `__text` bytes | `.o` bytes | ns/call min | ns/call median | vs clang -O2 (bench.c), min |
|---|---|---|---|---|---|---|
| 100 | t27b trap | 7996 | 10416 | 8.01 | 9.40 | 2.65x |
| 100 | t27b wrap | 6388 | 8808 | 5.54 | 7.36 | 1.83x |
| 100 | t27c gen-c + clang -O0 | 19200 | 33920 | 7.81 | 11.15 | 2.58x |
| 100 | t27c gen-c + clang -O2 | 10396 | 23520 | 3.93 | 3.96 | 1.30x |
| 100 | bench.c clang -O0 | 10400 | 16928 | 9.94 | 11.18 | 3.29x |
| 100 | bench.c clang -O2 | 9996 | 16528 | 3.02 | 3.17 | 1.00x |
| 1000 | t27b trap | 79996 | 102216 | 9.18 | 10.36 | 2.10x |
| 1000 | t27b wrap | 63988 | 86208 | 7.46 | 8.47 | 1.71x |
| 1000 | t27c gen-c + clang -O0 | 192000 | 337224 | 9.94 | 12.46 | 2.28x |
| 1000 | t27c gen-c + clang -O2 | 103996 | 233224 | 4.38 | 4.93 | 1.00x |
| 1000 | bench.c clang -O0 | 104000 | 166328 | 11.14 | 12.45 | 2.55x |
| 1000 | bench.c clang -O2 | 99996 | 162328 | 4.37 | 4.70 | 1.00x |
| 5000 | t27b trap | 400000 | 514216 | 10.29 | 10.95 | 2.10x |
| 5000 | t27b wrap | 319992 | 434208 | 7.60 | 8.50 | 1.55x |
| 5000 | t27c gen-c + clang -O0 | 963612 | 1696840 | 12.75 | 13.65 | 2.61x |
| 5000 | t27c gen-c + clang -O2 | 523608 | 1176832 | 4.89 | 5.30 | 1.00x |
| 5000 | bench.c clang -O0 | 523612 | 837944 | 12.51 | 14.55 | 2.56x |
| 5000 | bench.c clang -O2 | 503608 | 817936 | 4.90 | 5.26 | 1.00x |

* **Speed.** t27b's code is 1.55-1.83x slower than clang -O2 with
  `--overflow wrap`, and 2.1-2.65x slower in trap mode. That is roughly
  clang -O0 speed: within the noise of `gen-c` -O0 and faster than
  `bench.c` -O0.
* **Size.** t27b's `__text` is 64 bytes per function in wrap mode and 80 in
  trap mode. clang -O2 on `bench.c` takes 100 bytes and clang -O0 takes 104.
  `gen-c` -O0 takes 192, because the generated C is more verbose.
* **The cost of the checks.** Trap mode costs 25% in code size and 23-45%
  in time on this loop, which has three checked operations
  per iteration.

### Building the compiler

`bench.py build RUNS PKG` runs `cargo build --release -p PKG --timings` into
an empty target directory and measures the wall time. Both packages were
built on the same overloaded machine.

| Package | Runs | Wall median s | Wall min s | Load average | Units | Crates compiled | Binary bytes |
|---|---|---|---|---|---|---|---|
| t27b | 3 (127.1, 111.6, 67.2 s) | 111.6 | 67.2 | 244-336 | 17 | 8 | 1368288 |
| t27c | 1 | 1745.9 | 1745.9 | about 300 | 386 | 295 | 15626064 |
| t27c, the 142.7 s reference (lighter load, not re-measured here) | | 142.7 | | | 386 | | |

* **Units.** t27b builds 17 units against t27c's 386, 23x fewer. Its only
  dependencies are serde and the proc-macro chain behind `serde_derive`
  (proc-macro2, quote, syn, unicode-ident).
* **Time.** At comparable load, the median was 16x shorter than the single
  t27c build. Neither absolute time means much at a load average of 300 on 8
  cores. t27b's clean build was not timed on an idle machine.
* **Where t27b's build time goes.** Most of the source that the t27b crate
  compiles is not its own. The crate mounts t27c's `compiler.rs` (44,636
  lines) and `use_resolve.rs` (1,010 lines) next to about 5,100 lines of t27b.
  That is why an incremental rebuild after editing `lower.rs` still took
  49-58 s at this load. This split was not profiled. If the front-end were
  its own crate, editing t27b would no longer recompile it.

## Known limitations

* **No stack guard.** Runaway recursion in `t27b test` crashes the process
  instead of reporting a trap. `corpus` isolates files in subprocesses for
  this reason.
* **No loop bound.** An infinite loop hangs `t27b test`. `corpus` has a
  timeout (`--timeout-ms`, default 10 s).
* **Approximate line numbers.** A diagnostic carries the line of the last AST
  node visited that has one. Some t27c nodes carry no line (string
  literals, `invariant` blocks), so a message can point at the enclosing or
  preceding statement. A node with no line before it is reported at line 0.
* **Object-file traps lose detail.** A trap in an object file is
  `brk #kind`, with no site or operands. Only the JIT reports the line and the
  `assert_eq` values.
* **Simple code generation.**
  * Constants are re-materialised inside loops (`movz w17, #3` above).
  * `x + 0` is not folded.
  * There is no register allocator beyond the variable homes.
* **`step` blocks.** The `step` block of `while (c) : (step)` may contain
  only assignments.
* **Mach-O output.**
  * No `MH_SUBSECTIONS_VIA_SYMBOLS`, so the linker cannot dead-strip single
    functions.
  * No unwind info and no debug info.
* **W^X.** On macOS, `MAP_JIT` plus `pthread_jit_write_protect_np` is the
  whole protocol and `mprotect` is unused. On Linux the region is mapped RW,
  written, flushed (`dc cvau` / `ic ivau`) and switched to RX with
  `mprotect`; it is never writable and executable at once.
* **Untyped shift.** `1 << n` with a runtime `n` is rejected until literals
  can be given a type with a cast.
* **The front-end is mounted by path.** `src/lib.rs` uses
  `#[path = "../../../bootstrap/src/compiler.rs"]`. If `compiler.rs` is split
  into modules, those paths must follow it.

## Roadmap

1. **Coverage, in the greedy order of `--blockers`** (see the corpus
   section): casts, then typed `1 << n`, `assert(c, "message")`, `struct`,
   `invariant` blocks, more than 8 parameters through the stack, and so on.
   Twelve files need a t27c parser fix rather than a t27b one: dotted
   `module a.b;` and `use a.b;` (see the corpus section).
2. **Faster code:**
   * a linear-scan register allocator;
   * hoisting loop-invariant constants;
   * folding identities;
   * `MH_SUBSECTIONS_VIA_SYMBOLS`.
3. **More targets:** x86-64 (an encoder next to `a64.rs`), ELF64 `ET_REL`
   next to `macho.rs`, and a Linux JIT path (`mmap` + `mprotect`, which is
   why `mprotect` is already declared).
4. **Diagnostics:** a stack guard page and a loop fuel counter in the JIT;
   site tables in object files, so a `brk` can be mapped back to its line.
5. **Default test runner:** make `t27b test` the default runner for the
   specs it supports, with the Zig path kept as a cross-check, so that `tri
   test` stops paying for a `zig test` build per spec.
