# The discard, and how other parsers avoid needing one

**Status:** design note. Written 2026-08-29 while closing W699 (#2754), after the
parser stopped throwing away 1 292 tokens it had been throwing away silently.

## What t27 does today

`parse_bdd_clauses` lowers a braceless clause body into ordinary statements. Any
shape it does not fully understand restores a checkpoint and falls back to
`skip_to_next_top_level()` — the block's tokens are **consumed and dropped**, and
the surrounding file parses as if the block were empty.

That contract is deliberate and it is written into the function:

> Safety contract: this may only ADD assertions, never break a file.

It has a cost the contract does not name. A dropped `invariant` body is a dropped
set of assertions, and every downstream phase then reports success over an empty
block. `t27c parse-complete` exists to count what is dropped:

```
specs scanned            650
parse and consume all    472
parse but DISCARD        87 (32 485 token(s))
do not parse             91
```

Eighty-seven specs are accepted while part of their text is thrown away.

## Three designs that do not have this failure mode

### 1. Lossless concrete syntax trees — rowan (rust-analyzer), Swift, C#, Kotlin

Rowan's stated property is that **the original source can always be perfectly
reconstructed from the parse tree, even if it has errors** — comments,
whitespace, and parse errors are all nodes in the tree. Nothing can be dropped,
because dropping something would break the reconstruction invariant.

The relevant difference is not "better recovery". It is that *discarding is not
representable*. A `parse-complete` command cannot be needed by a parser whose
tree is a bijection with its input.

Cost: two tree layers (green nodes holding text position-independently, a red
tree modelling the exact source structure on top), and every consumer must
tolerate error nodes rather than assuming a well-formed AST.

### 2. Explicit ERROR and MISSING nodes — tree-sitter

Tree-sitter keeps the unparseable region as an `ERROR` node and can also insert
zero-width `MISSING` nodes when insertion is the cheaper repair. The dropped
region is *in the tree*, addressable by a query.

Worth reading before copying it: `tree-sitter parse` does not report an error
even when an ERROR node is present (tree-sitter/tree-sitter#4049), and MISSING
nodes are not captured by ERROR queries. **Having the representation is not the
same as reporting it** — which is this repository's own recurring lesson, arrived
at from the other direction.

Tree-sitter's own documentation notes the recovery "costs" are opaque to an
outside observer, and that it currently errs toward skipping subtrees where
inserting would be better.

### 3. Error productions — yacc/bison, and every LR grammar that uses them

The grammar names the recovery points itself (`stmt: error ';'`), so what is
skipped is a decision written in the grammar rather than a runtime heuristic.
Predictable, and auditable by reading the grammar — but it must be designed in
per-construct, and it says nothing about how much was skipped at runtime.

## Where t27 actually sits

Closer to tree-sitter than to rowan: recovery happens, and the discarded region
is not in the AST. The difference is that t27 **counts** it (`parse-complete`)
and **ratchets** it (`parse-no-discard` is a suite phase), which is more than
tree-sitter's own CLI does today.

The gap that remains is the same one tree-sitter#4049 describes: the count is
available to someone who runs the command, and the `parse-no-discard` phase sits
in the suite's BLOCKED column for the specs that need it most — so a spec can
lose its assertions and every gate stays green.

## What this note is not

It is not a proposal to rewrite the parser as a lossless CST. That is a rewrite
of every backend's assumption about its input, for a repository whose corpus is
650 specs. The cheap half of rowan's property is already available and unused:

- the discard is counted per spec, so it can be **ratcheted down** (it is)
- a spec that discards can be made to **fail its own phase** rather than be
  gated behind an upstream one
- the count can be printed **beside the acceptance columns**, where a reader who
  sees "217 accepted" would also see "on 32 485 tokens fewer than were written"

The third of those was one line, and it is written now — `t27c corpus` ends with

```
  specs with tokens DROPPED     87   13.4%
    ... tokens dropped       32485
  Accepted is not the same as accepted ON THE WHOLE SPEC.
```

## Sources

- rowan, README and design notes — <https://github.com/rust-analyzer/rowan>
- tree-sitter, ERROR/MISSING node semantics — <https://tree-sitter.github.io/tree-sitter/>
- `tree-sitter parse` does not report ERROR nodes — <https://github.com/tree-sitter/tree-sitter/issues/4049>
- MISSING nodes are not matched by ERROR queries — <https://github.com/tree-sitter/tree-sitter/issues/1136>
