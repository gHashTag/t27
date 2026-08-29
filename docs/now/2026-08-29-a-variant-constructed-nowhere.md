# NOW -- A variant constructed nowhere, and a comment that lied (2026-08-29)

## Three from the C-backend audit (Refs #2844, #2845)

- a condition matching `NodeKind::ExprRange` compiled, ran, and matched nothing: that variant is declared and CONSTRUCTED NOWHERE, and the parser builds an `ExprBinary` with `extra_op == ".."`
- the tell was the measurement, not the code -- bare blocks 374 before and 374 after; "it compiles and tests pass" would have shipped a fix that fixed nothing
- before matching on an enum variant, grep for where it is CONSTRUCTED, not where it is declared
- `gen_c_for_stmt` opened with "emit as a for loop with index" and emitted `{ body }`; 374 loops ran once, in C that cc accepts silently
- `compound_binop`'s docstring already described its own failure mode -- "a miscompilation rather than an error" -- and `/=` was missing anyway
- grepping the helper's name found three call sites; the Rust backend has two more that never call it, so it emitted ZERO compound assignments corpus-wide against Zig and C's 31
- enumerate call sites by BEHAVIOUR (every place that writes an assignment operator), not by the helper's name
