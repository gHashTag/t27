# NOW -- The census I shipped already existed, and the tool that should have said so was scoped to a directory (2026-09-05)

An hour after #3205 merged, `tri unparsed report --list` printed the same eight specs
it announced. Correcting the record here; #3204 is closed as superseded.

## What was already in the tree (Refs #3204)

- `cli/tri/src/unparsed.rs` ranks the constructs that stop the parser, each row backed by a live probe: `6  import ..` and `3  algorithm NAME {`
- the five `specs/` imports plus the three `algorithm` blocks are exactly the eight #3205 named -- same files, found earlier
- its method is stronger than mine: causality by REMOVAL, "a confirmed item is one whose removal MOVES the reported error", with 14 candidates refuted that way; I showed only that a conversion moves the error, which is the weaker half of the same test
- its module header already stated the lesson I presented as new, naming the same four examples: "`import x`, `algorithm y {`, `type T = T` and an English sentence all print 'unexpected token after expression statement'. The message names the state the parser recovered INTO, not what stopped it."

## The framing was inverted, not just duplicated (Refs #3204)

- that report prints these rows under `work queue -- every row proved unsupported by its own probe`
- it keeps a separate one-row list headed `refused ON PURPOSE -- a position, not a gap`, holding only `x as T`, a cast to a non-primitive type
- so the project's recorded position is that `import` and `algorithm` are compiler gaps to implement; #3204 asserted "no compiler change can retire this", which is the opposite
- whether an unsupported construct is a gap or a position is a question the project answers, and it had answered it; the lexer's keyword list alone does not settle it

## Why the anti-rediscovery tool did not stop it (Refs #3204)

- `t27c known --dir . --about "..."` exists for exactly this, and it answered "Nothing speaks to this. Measure -- and record the negative, it is a result."
- that answer is true of its population: it reads gates under `tools/`, baselines, and a paper -- it does not read `cli/tri/src/`
- the command prints its own scope on its first line, `gates read from .../tools`, and I did not treat that as the caveat it is
- an all-clear is scoped to what was searched; `git grep` for the most specific noun in the claim, across the whole tree, costs one command and prints `unparsed.rs` immediately
- three short, highly specific probes (`algorithm`, `suite_expectations`, `max_entries`) each returned zero gates, and the control that mattered was checking whether those words occur in `tools/` at all -- `max_entries` occurs 0 times there and 24 times in the repository

## What from #3205 still stands (Refs #3204)

- the corpus figures, read correctly only after the filter that stripped nine of fourteen report rows was re-anchored: 581 generate, 308 Zig accepts, 190 Zig analyses, 224 rustc, 290 cc, 380 iverilog, 74 with a data port, 258 Zig-and-Verilog, 184 all four, 76 dropping 23,831 tokens, 69 unparsed
- the ledger sits at 152 entries against a cap of 152, and its `parse` slice is exactly current -- 0 of 69 parse today
- the three controls that killed my own would-be findings before they were written down
- W643's non-empty floor, audited against every phase target list and found wired to all seven
