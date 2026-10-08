---
id: providers-steward
name: providers-steward
description: Standing profile card for the Providers rung (specs/providers/**), run repeatedly by the Queen's scheduler. Use when keeping the rung true -- the compiler gates over every provider spec, the seals, and the issues that follow a defect.
---

# /providers-steward -- keep the Providers rung true

<!-- only-t27-rule -->
> **Only t27: everything is written in `.t27`.** (Owner hard rule 2026-10-05; budget #7371; restated in every skill per #7588.)
> - New logic starts as a `.t27` spec with `test` / `invariant` blocks. It reaches Rust, C, Zig, JS or Python only through `t27c gen-*`, and generated files are never hand-edited (L2).
> - Hand-written code in any other language is denied by the hooks and by CI: `.rs .py .ts .js .sh .zig .c .go .v .yml .toml`, Dockerfile, Makefile; the full list is in `specs/policy/own_language.t27`.
> - Existing foreign code is debt that only shrinks. Delete it, or replace it with a spec and its generated output.
> - Thin hand-written glue is capped at 40 added lines per file and 80 per PR. No label lifts the cap. Never split a PR to slip under it.
> - What gen cannot express yet is a compiler defect. File it on #5980 with a minimal `.t27` repro instead of writing the code by hand.
> - This rule overrides any step below that says to write a script or code in another language. The full text is in `AGENTS.md`, "Only t27".

The Queen's scheduler fires `skill/t27/providers-steward.run` via the Inngest app
`t27-queen`, so this card runs again and again rather than once. The skill card
`specs/skills/t27-providers-steward.t27` is the source of truth for the file
list; epic #6081 is the umbrella this card reports on.

## Map

- The rung is the files the card's SPECS names: `specs/providers/catalog.t27`, `specs/providers/tri_gnk_pair.t27`, the five files under `specs/providers/gonka/`, and the two under `specs/providers/trinet/`.
- The rung's records are the seals `.trinity/seals/providers_*.json`.
- The umbrella is epic #6081. Point at these files and records, never restate them here.

## One run

- Run `t27c parse` and `t27c typecheck` over every file in `specs/providers/`, then `t27c test-report` over the same set. Read the counts; never assume them.
- A seal is wrong when it records `blocked` or any absolute home path. On a true rung `grep -l 'blocked\|/Users/\|/home/' .trinity/seals/providers_*.json` prints nothing.
- Every GAPS line stays until a run shows it is no longer true; a run that cannot show that changes nothing.

## Report

- One comment on the epic issue per run: the commands run, the commit they ran on, and the count each command produced.
- One new Queen-shaped issue per defect found (`## Boundary`, `## User Scenarios`, `## Requirements`, `## Success Criteria`), with `Refs #6081`.
- A clean run is one comment, not a new issue; no comment names more than one defect.

## Never

- Never merge, approve, or auto-merge anything, including the epic itself.
- Never set a Railway variable.
- Never print a secret.
- Never claim a GPU worker or a $TRI mainnet without the run that shows it.
- Never run builds on the owner's machine.
