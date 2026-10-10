# NOW -- t27b: the ledger blessed from a full lab run on master (2026-10-07)

## batch bless (Closes #7432)

- `t27b-native-ratchet` was red on master. It showed 123 UNEXPECTED PASS (`pass_vacuous` specs under `specs/{crons,skills,tools,agents,functions,docs,i18n}` that gained counted asserts) and more than 200 UNLISTED (specs landed with no ledger row, such as `specs/verified/verdict.t27`, `specs/queen/*.t27` and `specs/isa/*.t27`).
- Lab run (t27b Railway lab, `t27b corpus specs --jobs 2 --blockers`, reference = the same head's `t27c`) at master ba161bba1: 1539 of 1539 files, crash 0, mismatch 0, JIT/interpreter mismatch 0, reference disagree 0 of 992 compared.
- Master kept moving while the PR was open. The directories of the 18 specs it added or changed up to 34897263f were run again, at 848490e59, 285694ea5 and 34897263f, with that head's binaries. The 286 other rows in those directories agree with the full run.
- A fourth tail ran at b94f48659 over the 17 specs master added or changed after 34897263f: the test-name fixes of #7386, plus `ed25519.t27`, `device_dna.t27` and `postfix_optional.t27`. Mismatch 0, reference disagree 0, crash 0. The 254 other rows in those directories agree. CI's first run on the merge (37611885532) had listed 11 of them as UNLISTED.
- Blessed with `tri t27b ratchet --bless --accept-new`. A row is recorded only where the reference passes. The blessed ledger is byte-identical to the lab's, and the ratchet of the run against it is green: UNEXPECTED FAILURE 0, UNEXPECTED PASS 0, UNLISTED 0, STALE 0.
- One row was added by hand: `specs/ml/recurrent/self_attention.t27`, blocked on `StmtAssign(undeclared)`. CI's native arm64 reference passes it. The lab's x86 reference fails its one test (`forward_basic_case`) with the same zig 0.16.0. t27b blocks it on both machines.
- Moves: 123 `pass_vacuous` -> `pass`, plus 239 new rows (180 pass, 11 pass_vacuous, 45 blocked, 3 codegen). No row leaves the ledger.
- Counts: pass 536 -> 839, pass_vacuous 262 -> 150, not_pass 24 -> 72. The cap goes from 24 to 72. The whole rise of 48 is new specs (45 blocked, 3 codegen over the 16 KiB frame); no row the ledger named regressed.
- Left red on master, outside this bless: `specs/policy/own_language.t27` changed in #7399 and now uses `@intCast`, which t27b blocks (`ExprCall(@intCast)`, lane 4, #7412). The bless keeps master's `pass` row for it, measured on the lab at cfda070b2 before the change.
