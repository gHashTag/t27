# Audit: silent statement drops in function bodies

**Status:** finding, not yet fixed (the fix is coupled to a corpus reseal -- see below).
**Date:** 2026-07-09. **Compiler:** branch `codegen-clean` @ `8af24079`.

## Summary

`Parser::parse_fn_body` swallows any statement that fails to parse: on error it
calls `recover_to_stmt_boundary` and continues, **without recording anything**.
The parse then returns `Ok(module)` for a body that is silently missing
statements, `typecheck` reports `Typecheck OK (0 errors)`, and every backend
generates a function with its dropped `if` / `while` / assignment simply absent.
The saved seal is computed over that broken output, so `suite` reports the spec
as passing. The one diagnostic signal a spec author has -- "typecheck OK" -- lies.

Measured over `specs/` (504 specs):

- **131 specs (26%) drop at least one statement.**
- **755 statements dropped in total.**

## Root causes (ranked by frequency of the parse error that triggers the drop)

| count | parse error | real cause |
|------:|-------------|-----------|
| 270 | `Unexpected token in expression: KwInvariant` | **cascade** -- recovery consumes a function's closing `}` and runs into the next module-level `invariant`/`test` |
| 201 | `Expected LParen, got Ident` | **paren-free `if cond {}` / `while cond {}`** (Zig/Rust style); the parser requires `if (cond)` / `while (cond)` |
| 74 | `Unexpected token ... RBracket` | array-literal / indexing forms in expression position |
| 41 | `Expected RParen, got Comma` | multi-argument form the expression parser rejects |
| 27 | `Unexpected token ... Equals` | **compound assignment** `\|=`, `*=`, `/=`, `-=` (only `+=` is handled) |
| 21 | `Expected LParen, got Bang` | paren-free `if !cond {}` |

The **dominant, fixable** cause is paren-free `if`/`while`: the parser's
`parse_if_stmt` / `parse_while_stmt` `expect(LParen)` immediately after the
keyword, so a body like

```
fn total_domain_power(domains: [ClockDomainPower], count: u32) -> u32 {
    var total : u32 = 0;
    var i : u32 = 0;
    while i < count {                 // <-- dropped: parser wanted `while (i < count)`
        total = total + domains[i].power_mw;
        i = i + 1;
    }
    return total;                     // generated fn returns 0 -- the loop is gone
}
```

drops the entire `while` and its body, and the generated function is wrong.
`grep` counts ~164 specs writing paren-free `if` and ~49 writing paren-free
`while`, so most of the 131 affected specs are this one gap plus its cascade.

## Why the obvious fixes are not drop-in

Two experiments (both reverted):

1. **Make the drop fatal** (parse returns `Err` listing dropped statements).
   Correct in principle, but the corpus *depends* on the sealed-broken state:
   this turned 0 parse failures into **131**, and broke one compiler test
   (`test_roundtrip_bridge_spec`). Not viable without first fixing the causes.

2. **Support the missing syntax** (paren-free `if`/`while`, compound assigns).
   This is the right fix, but it *changes the generated output* of ~100 specs
   (from broken to correct), which invalidates their saved seals. So it is
   inseparable from a corpus-wide **regenerate + reseal**.

## Recommended path (must be done as one coupled change)

1. Extend `parse_if_stmt` / `parse_while_stmt` to accept a paren-free condition,
   and `parse_body_stmt` to accept `|= &= ^= -= *= /= <<= >>=`.
2. Regenerate all backends for every spec with the fixed `t27c`.
3. **Reseal on the corrected output** (`.trinity/seals/`) -- the current seals
   bless broken code and must not be carried forward.
4. Land on `master` together with the 9 pending `codegen-clean` fixes.

Steps 3-4 change committed seals and merge to `master`, so they need explicit
maintainer authorization; do not reseal ahead of the syntax fix or the broken
output gets locked in.

## Reproduce

Instrument the drop site and run `check` over the corpus:

```
// in Parser::parse_fn_body, the Err arm:
Err(e) => { eprintln!("DROP\t{}\t{}\t{}", decl.name, self.current.line, e);
            self.recover_to_stmt_boundary(); }
```

```
for s in $(find specs -name '*.t27'); do t27c check "$s" 2>&1 | grep '^DROP'; done
```

phi^2 + 1/phi^2 = 3 | TRINITY
