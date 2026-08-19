# Wave Loop 569 Report — every IGLA CODER and IGLA RACE spec was silently truncated

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_568_REPORT.md`](WAVE_LOOP_568_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W569 set out to implement W568's Variant A — resolve `use` across specs. It did, and
that turned out to be the smaller half of the wave. Following the first resolved
import into `systolic_ternary.t27` exposed something nobody had measured:

> **29 specs carry a stray closing brace, and the parser silently discards everything
> after it — 16,792 lines and 2,080 assertion clauses. Nine of the nine IGLA RACE
> kernels and all nine IGLA CODER specs are among them.**

```
non-scratch parse OK        341  ->  351      (+10, 0 regressions)
truncated content recovered              5,661 lines
                                           918 test blocks
                                           720 assertion clauses
w582 benchmark parse time   313 s  ->  228 s
```

---

## 1. Cross-module `use`, implemented

`use a::b::c` was parsed and then ignored. `specs/igla/race/systolic_ternary.t27`
declares `use igla::race::ternary_mac;`, calls `ternary_mul(a_in, w)`, and generated
Zig that failed on `TernaryWeight` — a type declared in exactly the module it had just
imported.

[`bootstrap/src/use_resolve.rs`](../../bootstrap/src/use_resolve.rs) resolves each
`use` to its spec file and splices in the declarations the importer actually needs.
**Selective** splicing, because W568 measured 38 colliding top-level names in a
15-spec closure — `PHI` is declared in four of them. Pasting whole modules would pick
a winner silently; a name found in two dependencies is left unresolved with a comment
naming both.

Three guards, each added because it was needed, not anticipated:

1. **Only declarations at the file's own top-level indentation.** The first version
   treated `const a: PackedTrit = 0xFF;` *inside a test body* as a module declaration
   and spliced statement fragments into the importer.
2. **Only from dependencies that parse on their own.** `specs/base/types.t27` is
   imported by most of the corpus and does not parse; splicing from it broke the
   importer. Its own `use` targets are still followed — an unparsable file can still
   name a parsable one.
3. **Compile-or-fall-back**, the same contract the W559 lowering carries: if the
   spliced source stops compiling, the original is used, so this can only add.

It works: `systolic_ternary` no longer fails on `TernaryWeight`, and `ternary_gemm`
no longer fails on it either. Both now fail on their *own* missing helpers.

## 2. The finding: a brace that hid a fifth of nine specs

Chasing the next error in `systolic_ternary.t27` bisected to a bare `}` at line 2026
of 2,655 — with no `{` anywhere to match it, in a spec that opens `module …;` and has
no module brace at all. The parser stops there and reports success. **629 lines were
never seen.**

Scanning every non-scratch spec:

| | |
|---|---:|
| Specs with a stray closing brace | **29** |
| Lines silently discarded | **16,792** |
| Assertion clauses silently discarded | **2,080** |

Twenty-eight of the 29 are one bare `}` and a final depth of exactly −1, so deleting
that single line balances the file. The distribution is not random: **all nine IGLA
CODER specs and all seventeen IGLA RACE specs**, each losing exactly 629 lines and 80
assertions — one templated wave-loop append that carried a stray brace, repeated
across a family.

The 29th (`specs/sandbox/https_enforce.t27`, depth −2, `} else {`) is a different
defect and was left alone.

### Removing the brace made things worse before it made them better

All 28 stopped parsing. The brace had been masking a **real** parse error in the tail:

```
test igla_race_systolic_ternary_w353_batch_depth_invariant_1 {
    assert true          <- "unexpected token after expression statement: KwTrue"
}
```

`assert <expr>` without parentheses works as a *clause* (`then x == 1` has lowered
since W559) but was a parse error as a *statement* inside a brace body. It appears
**3,682 times** in the corpus. Adding the statement form — checkpointed, falling back
to the original path on anything it cannot model — brought 9 of the 28 back, fully
parsed:

`adder_tree`, `cordic`, `cordic_fixed`, `cordic_top`, `opcodes`, `systolic_ternary`,
`ternary_gemm`, `ternary_inference`, `ternary_mac` — **every IGLA RACE kernel**, each
gaining 629 lines, 102 test blocks and 80 assertion clauses.

The other 19 already failed to parse before this wave and still do; the brace was
hiding their first error, which is now visible.

## 3. A performance regression, found and paid off

The W568 scratch sweep finished during this wave and reported three benchmark specs
changing state — all three to exit 142, **SIGALRM**. Not a parse-outcome change: a
timeout. W568 had made parsing of the corpus's largest specs dramatically slower.

Cause: `Parser::save_state` clones the lexer, and `Lexer::source` was a `Vec<u8>` —
**a full copy of the file per checkpoint**. Checkpoints were rare enough for that not
to matter until W568 added one per bracketed expression. On specs like
`w593_bench_module_5x2p15_aos_var_call_write.t27`, which nests array literals fifteen
deep, every level copied the whole source.

Two fixes:

- `Lexer::source` is now `Rc<[u8]>`, so a checkpoint is a refcount bump. This speeds
  up every existing checkpoint user, including the W559 and W567 lowerings.
- `parse_bare_array_literal` rejects `[` *token* `]` before scanning anything — a bare
  list needs two elements, so that shape is a dimension and cannot be one.

| Spec | base | W568 | W569 |
|---|---:|---:|---:|
| `w582_bench_16d_aos_call_dedup` (589,845 lines) | 313 s | >600 s | **228 s** |
| `w593_bench_module_5x2p15_…` | 1.07 s | >600 s | **0.74 s** |
| `w594_bench_module_7x2p14_…` | 0.44 s | >600 s | **0.51 s** |

Faster than the baseline it regressed.

---

## 4. Verification

| Gate | Result |
|---|---|
| Parse, 608 non-scratch specs, per-file vs W568 | `341 -> 351`, **0 regressions**, 10 improvements |
| Harness, 201 BDD specs | `ALL_PASS 22, COMPILE_FAIL 179, TEST_FAIL 0` |
| Verilog backend, FPGA + board specs | **18 of 18 byte-identical** to W568 |
| `use_resolve` unit tests | 3 passed |
| Specs still carrying a stray brace | 1 (the `} else {` case, deliberately) |
| Freeze ceremony | performed on every `compiler.rs` edit |

`ALL_PASS` is unchanged at 22 because the nine recovered specs now reach their *own*
missing helpers rather than a truncation. That is the expected shape and it is the
whole content of Variant A below.

---

## 5. Three cooperation variants for W570

### Variant A (recommended) — The IGLA RACE helper functions

Every recovered kernel now fails on one undefined name, and none of them exists
anywhere in the corpus:

| Spec | Missing | Substantive assertions |
|---|---|---:|
| `cordic.t27` | `abs_f32` | 271 |
| `cordic_fixed.t27` | `abs_i16` | 279 |
| `ternary_mac.t27` | `cast_i8` | 274 |
| `adder_tree.t27` | `adder_tree_2` | 270 |
| `ternary_gemm.t27` | `len` | 271 |
| `systolic_ternary.t27` | `systolic_ternary_array` | 304 |
| `opcodes.t27` | `string` | — |
| `ternary_inference.t27` | `pointless discard of function parameter` | 188 |

`abs_f32`, `abs_i16` and `cast_i8` are one-line functions. `len` is a builtin the
backend should lower. `adder_tree_2` and `systolic_ternary_array` are functions the
specs test but nobody wrote — those are spec-authoring decisions, and they should be
written from the tests that already describe them.

**Payoff if all resolve: ~1,857 substantive assertions**, in the kernels this project
exists to prove. Each is individually decidable and independently verifiable.

### Variant B — The 19 specs the brace was hiding

All nine IGLA CODER specs plus ten others now show their real first error instead of
parsing-while-truncated. The dominant class is `Unexpected token in expression:
LBrace` — block expressions, the class W549 measured at ~40 specs and nobody has
touched since. One compiler feature, nineteen specs, and it makes the CODER family
parse for the first time.

### Variant C — Flash the board

Unchanged: bitstream at 150.63 MHz, `fpga-flash` preflight clean and correctly
reporting `BLOCKED -- no programmer on USB`, T1/T2/T3 re-proved in W568. Needs the
QMTech Wukong V1 and a Digilent HS2 cable.

---

## Recommendation

**Variant A.** Nine kernels are one named function each from compiling, the names are
enumerated above, and three of them are one-liners. Nothing else in the queue converts
this little work into this many live assertions.

---

*φ² + φ⁻² = 3 | TRINITY*
