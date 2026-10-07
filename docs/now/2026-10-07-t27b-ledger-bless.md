# NOW -- t27b: the ledger blessed from a full lab run on master (2026-10-07)

## batch bless (Closes #7432)

- `t27b-native-ratchet` was red on master. It showed 123 UNEXPECTED PASS (`pass_vacuous` specs under `specs/{crons,skills,tools,agents,functions,docs,i18n}` that gained counted asserts) and more than 200 UNLISTED (specs landed with no ledger row, such as `specs/verified/verdict.t27`, `specs/queen/*.t27` and `specs/isa/*.t27`).
- Lab run (t27b Railway lab, `t27b corpus specs --jobs 2 --blockers`, reference = the same head's `t27c`) at master ba161bba1: 1539 of 1539 files, crash 0, mismatch 0, JIT/interpreter mismatch 0, reference disagree 0 of 992 compared.
- Master then moved to 848490e59, adding or changing four specs (`specs/automation/reddit-desk.t27`, `specs/port/trinity/src/tri/gen_{canvas,perceptron}.t27`, `specs/queen/review_log.t27`). Their directories were run again with the same binaries at 848490e59. The 102 other rows in those directories agree with the full run.
- Blessed with `tri t27b ratchet --bless --accept-new`. A row is recorded only where the reference passes. The blessed ledger is byte-identical to the lab's, and the ratchet of the run against it is green: UNEXPECTED FAILURE 0, UNEXPECTED PASS 0, UNLISTED 0, STALE 0.
- Moves: 123 `pass_vacuous` -> `pass`, plus 224 new rows (170 pass, 12 pass_vacuous, 39 blocked, 3 codegen). No row leaves the ledger.
- Counts: pass 535 -> 828, pass_vacuous 262 -> 151, not_pass 24 -> 66. The cap goes from 24 to 66. The whole rise of 42 is new specs (39 blocked, 3 codegen over the 16 KiB frame); no row the ledger named regressed.
- Left red on master, outside this bless: `specs/policy/own_language.t27` changed on master after #7399 and now uses `@intCast`, which t27b blocks (`ExprCall(@intCast)`, lane 4, #7412). The bless keeps master's `pass` row for it, measured on the lab at cfda070b2 before the change. Every other row comes from the run above.
