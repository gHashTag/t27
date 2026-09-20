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

## Start here

- `t27c spec-status <spec>` - the compiler's one-word verdict: IMPLEMENTED, PARTIAL, UNWRITTEN, NOPARSE, NOFN. Exit code is 0 whatever it says, so read the word.
- `t27c symbols <spec>` - every name the file declares, with its kind. Answers "does this already exist here?" before you add it.
- `t27c outline <spec>` - per function: its locals, what it calls, what it returns. The contract you are implementing against.
- `t27c coverage <spec>` - which functions have a test and which do not. The issue asks for tests; this is how you check you wrote them.
- `t27c lint <spec>` - the warnings a reviewer will quote at you, including every function with no test or invariant.
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
