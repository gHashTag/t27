# Architect decision: three dialects, one grammar, one seal store

One page, one decision. Measured in W882; verification numbers from W883.

## The situation

`specs/` holds at least three surface languages. The bootstrap `t27c` parses one
of them. The meta compiler sealed all of them, and its certificates dominate the
store.

| dialect | files (measured) | marker |
|---|---|---|
| t27 (Zig-flavoured) — **the bootstrap grammar** | ~1,040 parse today | `fn f(x: T) T {`, `for (xs) \|x\|` |
| Rust-flavoured | ~7 | `let mut x: [T] = []`, `impl X {`, `for i in 1..=10` |
| `algorithm` DSL | 3 | `algorithm phi_rope {` |
| generic-struct extension | 11 (`tri/collections`) | `pub const Map(K, V) = struct {` |
| single-construct gaps | 2 | nested `fn` (blocks **gf16, the L6 SSOT**); `while (..) : (i += 1)` |

## What verification says about the two compilers

- bootstrap-minted seals (165, labelled `sealed_by`): **165/165 reproduce fully**
  — spec hash and every gen hash.
- meta-minted seals, random 100 of 1,014 parseable: **11 reproduce, 89 mismatch**
  on gen hashes. The meta compiler's codegen is a different codegen; its
  certificates cannot be re-verified by the tool the repository builds today.

## The options, priced

**1. Canonicalise the bootstrap dialect.** Rewrite ~23 files (11 generics, 3 DSL,
7 Rust-forms, 2 modules) into t27; land the two single-construct GOLD-RING
patches (0001 compound assignment — ready; 0002 nested fn — one file needs it).
Cost: mechanical rewrites plus generics need a design (the collections library
depends on them). Benefit: one grammar, one verifier, the whole store
re-verifiable.

**2. Keep the meta compiler as co-verifier.** Zero rewrites; the store stays
split. Cost: every audit needs `sealed_by` awareness forever, and 89 % of the
larger stratum stays unverifiable by the in-repo tool.

**3. Freeze the dialects as historical.** Mark the 23 files' seals as
archive-only. Cheapest; but `tri/collections` is a LIVE library and gf16 is the
SSOT — neither is history.

## The recommendation this audit can defend

Option 1, staged: land 0001 (ready), prototype 0002 (nested fn), rewrite the 12
non-generic files, and design generics separately — they are the only genuinely
open language question in the set. Options 2 and 3 both leave the L6 SSOT
unverifiable by the repository's own compiler, which is the state that let a
`*=` gap sit unmeasured for the grammar's whole life.
