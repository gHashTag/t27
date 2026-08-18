# [GOLD-RING] proposal: compound assignment (`-=`, `*=`, `/=`, `%=`)

**Not applied.** `bootstrap/src/compiler.rs` is FROZEN_HASH-enforced
(`build.rs:206`); FROZEN.md requires a [GOLD-RING] PR with Architect approval,
and W780's precedent is verify, document, do not apply. This directory is the
verification and the documentation. The patch includes the matching
`stage0/FROZEN_HASH` update, as the freeze process requires.

## What the grammar lacks, measured

`+=` has been the only compound assignment for the grammar's whole life —
`git log -S'"*="' -- bootstrap/src/compiler.rs` is empty. Minimal repros
(five-line specs, one construct each):

    t += x   PASS          t -= x   FAIL
                           t *= x   FAIL
                           t /= x   FAIL
                           t %= x   FAIL

## The patch, validated on a scratchpad build

Five sites, each mirroring the existing `+=` handling:

| site | change |
|---|---|
| `TokenKind` enum | `MinusEquals`, `StarEquals`, `SlashEquals`, `PercentEquals` |
| lexer (`two ==` chain) | four two-byte cases beside `[b'+', b'=']` |
| parser (`parse_stmt` assign arm) | the `if PlusEquals` becomes a five-way `match` filling `extra_op` |
| Zig + C codegen | `match extra_op` writes the operator through |
| Verilog codegen | every compound desugars to `lhs <= lhs <op> rhs`, exactly as `+=` already did |

Built cleanly; all four repros parse; existing repro suite unaffected.

## Honest blast radius — smaller than the wave that found it assumed

Of the 201 unique stale-seal specs, **165 already parse** with the unpatched
compiler: most reseal refusals are BACKEND coverage (compile to `none` for
parseable specs), not grammar. Against the 36 that fail parse:

    fixed outright by this patch     1    specs/numeric/tf3.t27
    advanced but not through         1    specs/numeric/gf16.t27 (L6 SSOT):
                                          824 -> 3507, where the next gap is a
                                          NESTED fn (fn phi_dist inside a fn)
                                          with an if-expression body
    untouched                       ~34   need nested fns / if-exprs / others

So this patch is necessary for the L6 SSOT and sufficient for one spec. The
nested-fn gap is the next [GOLD-RING] candidate and is deliberately not bundled:
one grammar change per proposal keeps the Architect's decision reviewable.

## Reproduction

    cp -R bootstrap <scratch>/ && cd <scratch>/bootstrap
    printf '\n[workspace]\n' >> Cargo.toml        # detach from the parent workspace
    git apply 0001-compound-assignment.patch
    cargo build --release
    ./target/release/t27c parse specs/numeric/tf3.t27   # rc=0, was rc=1
