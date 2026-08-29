# NOW -- The division was dropped, by all three backends (2026-08-29)

## A compound operator missing from a table becomes a plain store (Refs #2834)

- `scaled /= 2.0;` was emitted as `scaled = 2.0;` -- the division gone, the variable overwritten with the divisor -- by Zig, C AND Rust
- `compound_binop` mapped `+= -= *= |= &= ^=` and nothing else; all call sites fall back to ` = ` when the lookup returns None
- the table's own docstring already described this failure mode -- "a miscompilation rather than an error" -- and `/=` was still missing
- five occurrences in three specs, every one of them valid output in the target language and none of them the stated program
- the Rust backend was worse: both of its StmtAssign arms hardcode `"{} = {};"` and never read `extra_op`, so EVERY compound assignment became a plain store
- measured across the corpus: compound assignments emitted, Zig 31 -> 33, C 31 -> 33, Rust **0 -> 27**
- four call sites, not three: I found three by grepping the helper's name, and the fourth pair never called it at all
- the fallback is now loud -- an unrecognised compound operator is passed through verbatim, so it either means what the spec said or the target compiler refuses it by name
