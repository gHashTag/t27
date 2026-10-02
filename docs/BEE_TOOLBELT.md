# The toolbelt

`t27c --help` lists 155 subcommands. The brief the swarm is given has named four
of them -- `spec-status`, `gen`, `parse`, `typecheck` -- and so has every
acceptance criterion filed against this repository. Everything else an agent
needed, it rebuilt out of `grep`.

That is not a style complaint. A grep is satisfied by a file that never builds:
388 of the 837 specs the oracle can reach generate Zig that does not compile,
and every one of them was written by an agent whose criteria were greps and who
had no cheap way to find out. Meanwhile `t27c lint` has been printing
`WARN: fn 'diff' has no test or invariant` -- the review's complaint, before the
review -- for longer than the swarm has existed.

This page is the list. `tri toolbelt` runs every command on it against a real
spec and prints what each one answered, so a command that stops working stops
being advertised. `tri toolbelt --brief` prints the *Start here* block exactly as
`tools/queen/feed_empty_bodies.py` embeds it into an issue, which is why that
block is short: it is read by every worker on every task.

Every command here READS. Nothing on this page writes to the tree.

## What the language accepts

`.t27` is not Rust and not Zig, and a bee fluent in either writes one by
accident. Measured 2026-09-20: a bee filled eight bodies in
`specs/file/watcher.t27` with `return Ok(());`, `Err(FileError::WatcherNotFound)`
and `for i in 0..watchers.length`. The parse ratchet refused the whole file --
`Unexpected token in expression: RParen`, line 123 -- and a spec that does not
parse generates nothing, so every test it already carried stopped running.

At the top level the parser accepts exactly these eight forms, with an optional
`pub`, and no others:

```
const   var   fn   enum   struct   test   invariant   bench
```

There is no `trait`, no `impl`, no `type X = ...`, no generics, no macro. What
a bee reaches for from another language, and what happens:

- Rust's result sugar (`Ok(())`, `Err(E::V)`, `?`) is not a construct here.
- Rust macros (`println!`, `format!`, `vec!`) are not constructs here.
- A range loop (`for i in 0..n`) is Rust and Zig, not this language.
- Zig builtins (`catch unreachable`, `@intCast`, `@constCast`) are not constructs here.
- Cross-module reuse (`use other::fn;`) parses and then generates a comment and an unqualified call, so the Zig fails with `use of undeclared identifier` (#4298). It does not work yet.

`t27c parse <spec>` answers in one line whether what you wrote is the language.
Run it before you report, every time.

## Start here

- `t27c spec-status <spec>` - the compiler's one-word verdict: IMPLEMENTED, PARTIAL, UNWRITTEN, NOPARSE, NOFN. Exit code is 0 whatever it says, so read the word.
- `t27c symbols <spec>` - every name the file declares, with its kind. Answers "does this already exist here?" before you add it.
- `t27c outline <spec>` - per function: its locals, what it calls, what it returns. The contract you are implementing against.
- `t27c coverage <spec>` - which functions have a test and which do not. The issue asks for tests; this is how you check you wrote them.
- `t27c lint <spec>` - style and shape warnings. It OVER-REPORTS `has no test or invariant`: measured 2026-09-20 over 60 specs it printed 677 of those where `coverage` found 190 untested functions, disagreeing on 59 of the 60 - it warns about `bit_to_trit_pair` in `specs/base/ternary_encoding.t27`, which `test bit_to_trit_pair_zero` calls on line 249. Read it as a hint; `coverage` is the answer.
- `t27c typecheck <spec>` - types, before generation. Prints `Typecheck OK (0 errors, 0 warnings)` or the errors.
- `t27c test-report <spec>` - builds this spec and runs its own tests, which is what the oracle does. `BLOCKED` means the generated Zig does not compile, with the error beside it.
- `python3 tools/dupe_scan.py --name <function>` - where that function already lives, if it does. 576 of 4021 bodies here are byte-identical copies.

## Reading a spec you did not write

- `t27c inspect <spec>` - the public surface: `pub` functions, structs, enums, consts.
- `t27c exports <spec>` - every exportable symbol, one per line.
- `t27c depends <spec>` - what this module says it depends on.
- `t27c tree <spec>` - the AST, indented. The parser's own reading of the file.
- `t27c strings <spec>` - every string literal, which is usually where the domain vocabulary is.
- `t27c test <spec>` - the `test` and `invariant` blocks the file declares, by name.

## Measuring before you claim

- `t27c count <spec>` - declarations by node type.
- `t27c loc <spec>` - lines of code per function, from the source.
- `t27c size <spec>` - size metrics for the file.
- `t27c metrics <spec>` - per function: complexity, lines, parameters.
- `t27c depth <spec>` - call depth per function.
- `t27c stack <spec>` - struct field layout with estimated byte sizes.
- `t27c hash <spec>` - SHA256 of the source, for saying WHICH version you measured.

## Finding what is wrong

- `t27c deadcode <spec>` - functions nothing calls, in this spec.
- `t27c orphans <spec>` - entry-point analysis: which functions are never called.
- `t27c spellcheck <spec>` - identifiers within edit distance 2 of each other, which is how `magmul` and `magmuI` both came to exist.
- `t27c validate-vacuity --specs-dir specs` - tests that assert nothing (`assert true`) and tautological invariants. A test that cannot fail is not coverage.
- `t27c check-calls --specs-dir specs` - call sites against the signatures they call, across the tree: arity, and aggregate-vs-scalar.
- `t27c impl-status` - every spec separated into UNWRITTEN, BROKEN and the rest.
- `t27c dupes --name <function>` - the compiler's own version of the lookup above. It parses every spec, so it takes about a minute where `dupe_scan.py` takes ten seconds.
- `t27c classify` - is this file source at all? A `.t27` extension is a filename, not a type declaration.

## What the generators actually emit

- `t27c gen <spec>` - the Zig. This is what the oracle compiles.
- `t27c gen-c <spec>` - the C.
- `t27c gen-rust <spec>` - the Rust.
- `t27c gen-verilog <spec>` - the Verilog.
