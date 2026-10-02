# NOW -- Every tool and skill bound to the Queen and a domain lead (2026-10-03)

## What this PR changes (Refs #3545)

- `docs/agents/AGENTS_ALPHABET.md` states two rules: the Queen holds every tool and skill, and every card also has a domain lead read off what the card says it does, never off its name.
- The bindings are written on both sides, in `specs/agents/*.t27` (`TOOLS`, `SKILLS`) and in the tool cards under `specs/tools/` (`AGENTS`).
- Six tool cards get no domain lead and say why: four external MCP servers whose cards record only registration, and two cards whose stated subject is too thin to place.

## How it was checked

- The PR description reports 0 problems from gHashTag/trinity `scripts/agents-from-specs.mjs` against a vendored copy, with tools bound to an agent going from 6 to 92.

## Re-dated

- This entry was added on 2026-10-03 for re-review. The PR was opened on 2026-09-15 without a docs/now entry; the text above restates its description and claims nothing new.
