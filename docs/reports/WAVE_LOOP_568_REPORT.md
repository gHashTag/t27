# Wave Loop 568 Report — the backend queue drained; the wall behind it is not the one I named

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_567_REPORT.md`](WAVE_LOOP_567_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W568 took its own Variant A — keep draining the compile-failure queue — and drained
most of it. Nine backend defects and one corpus typo, each with a reproduction and a
before/after count.

```
ALL_PASS         16  ->  22        (+6 specs)
tests passing   209  -> 280        (+71)
COMPILE_FAIL    183  -> 177
TEST_FAIL         0  ->   0
REGRESSIONS                0
```

And then the measurement that matters more than any of it: **the helper I have
recommended unblocking for six waves is not where the assertions are.**

---

## 1. What was fixed

Every entry below was diagnosed from the first Zig error of all 183 failing specs,
not from a sample. Raw before/after: [`data/W568-first-errors-before.tsv`](data/W568-first-errors-before.tsv),
[`data/W568-first-errors.tsv`](data/W568-first-errors.tsv).

| # | Defect | First-error count |
|---:|---|---:|
| 1 | Dotted type paths (`std.mem.Allocator`) parsed as one segment | 28 |
| 2 | Struct field types joined lexeme-by-lexeme (`[]const u8` -> `constu8`) | 11 |
| 3 | A bare `[1, 2, 3]` const value consumed the rest of the file | 11 |
| 4 | Field defaults swallowed into the type string | (in 3) |
| 5 | Duplicate test names rejected the whole file | 4 |
| 6 | `::` reached Zig verbatim (`Severity::Error`) | 5 |
| 7 | Explicitly-valued enum with no backing type | 3 |
| 8 | `<` on enum variants / `==` on string-typed fields | 4 |
| 9 | Quoted type annotations (`head : "usize"`) | 24 |
| 10 | Corpus: `Std.mem.Allocator` — a name that resolves nowhere | 81 occurrences, 32 specs |

### The one that was destroying whole specs

```t27
const COPTIC_ALPHABET : [27]u32 = [0x03B1, 0x03B2, ...]
```

`parse_array_literal` only understands the Zig-ish `[N]T{ ... }` spelling, so this
produced a childless node and the caller fell through to a text collector that runs
**to the next semicolon**. t27 declarations are newline-terminated, so one such const
swallowed every declaration after it — `specs/isa/registers.t27` collapsed into a
single unparsable string:

```zig
const A: [3]u32 = [1,2,3]constB:[3]u32=[1,2,3,]fng()->u32{returnA[0];
```

Fixed by parsing the bare list as expressions, and — separately — by bounding the
text collector at a declaration keyword that opens its own line. The second half
matters on its own: it converts "an unrecognised value destroys the file" into "an
unrecognised value stays one declaration wide."

### The one that needed a second attempt

Routing struct-field types through the real type grammar broke **9 specs** on the
first try. Three specs contain a malformed field whose type opens a string literal:

```t27
tag : [[]Const u8",
```

The bracket loop consumed that string across the rest of the file and the struct
never found its closing brace. The old lexeme-join stopped at the first comma, so the
damage stayed inside one field. Requiring the type to **end on its own line**
restores that containment. Zero regressions after.

### Duplicate test names: fixed without deleting anything

Four IGLA RACE specs accumulated colliding test names across waves —
`cordic_fixed_sin_zero_angle` appears **seven times**. Zig rejects the file outright.
The backend now suffixes repeats (`__dup2`, `__dup3`), so every test runs.

Suffixing silently would hide a real corpus defect, so `suite.rs` Phase 6 now counts
it: `duplicate test names: N name(s) across M spec(s)`. Reporting only, per the
standing rule that turning these into gates is a maintainer's decision.

---

## 2. The measurement that overturns my own recommendation

Since W561 I have carried Variant B: decide the fate of the 571 `default_input()`
template tests, "the largest remaining block", 169 specs. After this wave's fixes,
`default_input` is the first error in **110 of 177** remaining compile failures —
62%. By spec count it looks more dominant than ever.

By **assertions**, it is not close:

| Blocked by | Specs | Substantive assertion clauses |
|---|---:|---:|
| `default_input()` | 110 | **169** |
| everything else | 67 | **3,197** |

Nineteen times as many assertions sit behind the 67 as behind the 110. The 110 are
mostly template scaffolding — a `given input = default_input()` / `then result !=
undefined` pair with little else in the file. The 67 are dense specs.

**Counting specs pointed one way and counting assertions pointed the other.** This is
the third time in this chain that re-measuring reversed my own prioritisation
(W560→W561 was the same shape, and the same error: ranking by first-error frequency
instead of by what the fix releases). Skill rule 26 already said so. I applied it to
the queue and not to my own standing recommendation.

### Where the assertions actually are

The eight heaviest blocked specs are all **IGLA RACE** kernels:

| Spec | Substantive assertions | First error |
|---|---:|---|
| `systolic_ternary.t27` | 304 | undeclared `TernaryWeight` |
| `cordic_fixed.t27` | 279 | undeclared `abs_i16` |
| `cordic_top.t27` | 277 | `expected type expression` |
| `ternary_mac.t27` | 274 | undeclared `cast_i8` |
| `ternary_gemm.t27` | 271 | undeclared `TernaryWeight` |
| `cordic.t27` | 271 | undeclared `abs_f32` |
| `adder_tree.t27` | 270 | undeclared `adder_tree_2` |
| `ternary_inference.t27` | 188 | undeclared `TernaryWeight` |

2,134 substantive assertions in eight files — 67% of everything still blocked.

---

## 3. The new #1 blocker, named precisely

`specs/igla/race/systolic_ternary.t27` opens with:

```t27
use base::types;
use igla::race::ternary_mac;
```

and fails on `TernaryWeight` — which **is** defined, in `ternary_mac.t27`, the module
it just imported. The Zig backend emits one file per spec and resolves nothing across
`use`.

Measured across every remaining undeclared-identifier failure that is not
`default_input`. The first pass asked only "is this name defined *somewhere* in the
corpus", which is not the same question — re-measured strictly, against the modules
each file actually imports:

| | Specs | Substantive assertions |
|---|---:|---:|
| Name is declared in a module the spec `use`s | **7** | **993** |
| Name exists in the corpus but in no imported module | 9 | 36 |
| Name exists nowhere | 15 | — |

The strict number is smaller in specs and almost identical in assertions: seven files
carry 993 of them. The nine that merely *have* the name somewhere are worth 36 — they
are missing-import defects in the specs, not a compiler gap.

### The falsification condition, checked

Variant A below carries the condition "if the `use` graph is cyclic, single-file
emission cannot work." That was checked rather than assumed:

```
transitive closure from the 7 specs : 15 specs
use targets that do not exist       : 0
cycles                              : 1  (nn/attention.t27 imports itself)
```

Acyclic apart from one degenerate self-import, and every import resolves to a file
that exists. The queue has drained down to a **feature gap, not a defect list**: `use`
is parsed and then ignored.

---

## 4. Verification

| Gate | Result |
|---|---|
| Harness, 199 BDD specs | `ALL_PASS 22, COMPILE_FAIL 177, TEST_FAIL 0` |
| Regressions vs. W567 baseline | **0** |
| Tests executing and passing | **280** (was 209) |
| Parse census, 608 non-scratch specs | `OK 349` before and after, **0 differences** |
| Parse census, 455 `specs/scratch/` specs | in flight — 0 differences so far |
| Verilog backend, 87 FPGA/board/RACE specs | 26 generate before and after; **24 byte-identical, 2 strictly larger** |
| Icarus on the two that changed | fails identically to baseline (pre-existing duplicate declarations) |
| T1 / T2 / T3 theorems | re-proved |
| Freeze ceremony | performed on every `compiler.rs` edit via `t27c frozen-digest` |

The census was run as a **per-file comparison** this time — the baseline binary and the
W568 binary parse each spec and their exit codes are compared — rather than comparing
two aggregate counts. It answers the question the gate actually asks ("did any file
that parsed stop parsing?") instead of a proxy for it, and it catches a regression that
an equal-and-opposite improvement would hide.

Two generated Verilog files grew: `boards/xc7a100t_minimal` and
`fpga/testbench/top_tb` now emit the array-literal assignments that the backend
previously could not see (`all_pins[(0) * 107 +: 107] = PIN_CLK;`, `led[0] = 1'b1;`).
An earlier version of the array-literal change also *shrank* a comment in
`ternary_gemm` — the Verilog backend reads the literal's element text while the Zig
backend reads its children, and only children were being populated. Both are now
populated, and that file is byte-identical again.

Newly passing: `boards/xc7a100t_minimal` (23 tests), `compiler/pipeline` (13),
`fpga/clock_domain` (12), `fpga/stdlib` (17), `igla/training/low_bit_ternary` (3),
`igla/training/scale_up` (3).

Harness and raw results are committed in [`data/`](data/).

### A note on the census

The full census (1,063 specs) is slow enough to look like a hang, and it is worth
recording why so the next wave does not rediscover it. `specs/scratch/` holds 455
generated benchmark files totalling **22,994,697 lines** — 49 of them over 100,000
lines and one at 1,179,669. A single large file takes ~40 s to parse, and the corpus's
608 real specs are a rounding error beside it.

Practical consequences: use a per-file timeout so a slow file is distinguishable from
a hang (a bare `for` loop that appears stuck is almost certainly parsing
`w584_bench_17d_aos_call_dedup.t27`), split the run so the 608 real specs report in
about a minute, and treat the scratch sweep as a separate long-running check. The
scratch comparison for this wave was still running when the wave was committed; the
608 real specs showed zero differences and the scratch result is appended below when
it lands.

---

## 5. Three cooperation variants for W569

### Variant A (recommended) — Resolve `use` across specs

**Measured payoff: 7 specs, 993 substantive assertions**, including three of the
heaviest IGLA RACE kernels (`systolic_ternary` 304, `ternary_gemm` 271,
`ternary_inference` 188).

**Deliverables.**
1. Resolve `use a::b::c` to `specs/a/b/c.t27` and make its declarations visible to the
   backend — either by emitting the dependency's declarations into the generated file,
   or by emitting a real Zig `@import` and a file per module.
2. Skip self-imports, detect real cycles, and give a clear error when a `use` names a
   spec that does not exist.
3. Re-run the harness; report `ALL_PASS` / tests executing, and the new taxonomy.

**Its falsification condition is already checked** (§3): closure of 15 specs, zero
dangling imports, one self-import and no real cycle. The collision question was
checked too, and it **fails the naive design**:

```
closure               : 15 specs
distinct top-level names : 652
COLLIDING names       : 38     -- PHI is declared in four of them
```

So "paste the dependency's declarations into the generated file" is wrong. Two designs
survive: emit one Zig file per module and reference imports through it
(`constants.PHI`), or inline **selectively** — only names the importer references and
does not declare itself — and make an ambiguous name a hard error rather than a silent
pick. Start there; the 38 collisions are the design input, not an obstacle discovered
halfway.

### Variant B — The names defined nowhere

`abs_i16`, `cast_i8`, `abs_f32`, `adder_tree_2`, `SHA256`, `Instant`, plus the nine
missing-import cases. These are things the specs call and nobody wrote — the same
class as `default_input`, but each one small and individually decidable. Some
(`abs_i16`, `cast_i8`, `abs_f32`) are one-line prelude additions and unblock
`cordic`, `cordic_fixed` and `ternary_mac` — **824 more substantive assertions** in
the IGLA RACE family. Others are genuine gaps in what the spec claims to describe.

Together, A and B are what stands between this project and its RACE kernels' tests
actually running.

### Variant C — Flash the board

Unchanged and still the only item in the whole project waiting on something other
than work: a routed bitstream at 150.63 MHz since W553, `fpga-flash` pre-flighting
clean, T3 giving a falsifiable prediction. Needs the QMTech Wukong V1 and a Digilent
HS2 cable.

**`default_input()` is deliberately not a variant this time.** It has been Variant B
for six waves on the strength of a spec count. The assertion count says it is worth
169 assertions — real, but a twentieth of what Variant A releases. It should be done
eventually and it should not be done next.

---

## Recommendation

**Variant A.** The compile-failure queue that has driven eight waves is empty of
mechanical defects; what remains is one missing language feature with a measured
1,029-assertion payoff, most of it in the IGLA RACE kernels this project exists to
prove.

---

*φ² + φ⁻² = 3 | TRINITY*
