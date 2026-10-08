---
id: self-host
name: self-host
description: Work on epic #5980, rewriting t27c from Rust into t27. Use when changing specs/compiler/core/t27core.t27, when fixing a gen-c defect the core refuses, or when filing the next self-host task for the Queen.
argument-hint: "[issue number]"
---

# /self-host -- the t27 compiler written in t27

<!-- only-t27-rule -->
> **Only t27: everything is written in `.t27`.** (Owner hard rule 2026-10-05; budget #7371; restated in every skill per #7588.)
> - New logic starts as a `.t27` spec with `test` / `invariant` blocks. It reaches Rust, C, Zig, JS or Python only through `t27c gen-*`, and generated files are never hand-edited (L2).
> - Hand-written code in any other language is denied by the hooks and by CI: `.rs .py .ts .js .sh .zig .c .go .v .yml .toml`, Dockerfile, Makefile; the full list is in `specs/policy/own_language.t27`.
> - Existing foreign code is debt that only shrinks. Delete it, or replace it with a spec and its generated output.
> - Thin hand-written glue is capped at 40 added lines per file and 80 per PR. No label lifts the cap. Never split a PR to slip under it.
> - What gen cannot express yet is a compiler defect. File it on #5980 with a minimal `.t27` repro instead of writing the code by hand.
> - This rule overrides any step below that says to write a script or code in another language. The full text is in `AGENTS.md`, "Only t27".

The goal of epic #5980 is a t27 compiler written in t27 that replaces the Rust
`t27c`. Step one (#5981) is `specs/compiler/core/t27core.t27`, a core that
compiles a subset of t27 to C. Its gate is `bootstrap/tests/core_selfhost.rs`.

## The one rule: byte identity, and refuse what gen-c gets wrong

Inside its subset, the core writes **exactly** the bytes `t27c gen-c` writes.
The proof is a fixpoint: `core(t27core.t27) == gen-c(t27core.t27)`. Outside its
subset, it refuses with an error code.

Some shapes are ones that gen-c lowers *with loss*. Examples: a dropped
initializer, `- -x` written as `--x`, a repeat filled with zeros. The core must
**refuse** those, never copy them. Agreeing with gen-c there would mean agreeing
with a defect. Every such shape is:

1. a row in the `REFUSALS` table of `core_selfhost.rs`, with its code;
2. a bee-ready issue `Part of #5980` that fixes gen-c in
   `bootstrap/src/compiler.rs`, first filed as #6046 to #6052;
3. after that issue lands, a candidate fixture: move the row from `REFUSALS`
   to `FIXTURES` and teach the core the corrected bytes.

## Before you widen the subset

- **Probe gen-c first.** Write the smallest file, then run
  `./target/release/t27c gen-c /tmp/x.t27`, and read the bytes before you
  write a single emitter line. Never assume what gen-c prints.
- Compile what it prints with `cc` and **run** it. Text that looks right has
  shipped wrong values before: a `{0}` repeat, or a shadowed global.
- Add the fixture, then make it pass. The corpus differential must not
  lose files: `CORPUS_FLOOR` only goes up.

## Traps that cost hours

- A line holding only `;` silently breaks the parse of what follows. Use a
  blank line.
- Prose inside a function body must be a `//` comment, not `;`.
- `module` must be the first declaration. Whatever follows `endmodule` is
  dropped today (#6047).
- `cargo build` in a shared target dir can take more than 15 minutes when other
  sessions are building. Run experiments on the Railway bees by filing an issue,
  not on a laptop.

## Filing the next task for the Queen

A task is dispatchable only with a `## Boundary` section of file paths. Check it
with `python3 tools/queen/task_shape.py --issue N` before calling it delegated.
Write acceptance criteria as numbered commands, each with today's output. Each
criterion must have a regression test that fails with the fix reverted. Tasks
that share `bootstrap/src/compiler.rs` run one at a time, so keep each one small.

## Who

Gamma (agent C, Compiler Core) is the domain lead; see
`.claude/agents/agent-c-compiler.md` and `specs/agents/c.t27`.
