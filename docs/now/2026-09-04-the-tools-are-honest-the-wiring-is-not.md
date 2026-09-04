# NOW -- The tools are honest; the wiring is not (2026-09-04)

## I went looking for gates reading a slice of their subject and found none

- Three times in two passes a gate's population turned out smaller than its subject: `lake build`
  red with its workflow run once in sixty (#3142), the `Admitted` gate scoped to two of 58 Coq
  files (#3153), a `paths:` filter matching nothing for months. That is a class, so I measured it.
- `tri gate-reads` runs each `tools/check_*.py` under an audit hook and records every file opened
  and every subprocess spawned. **Across all 20 gates, not one reads a slice of its own declared
  subject.** Every small number has an innocent explanation and each was checked:
  `check_duplicate_agreement` reads **64 of 64** `specs/ternary/*.t27`; `check_sync_repo_root` reads
  its one target by design; `check_specs_generate` opens 3 files and spawns **1040** subprocesses.
- **So the shape does not live in the tools.** It lives one layer up, in which gate runs and on
  what. The tools are honest about what they read.

## Two ways the instrument lied before it worked

- **A skipped gate reads nothing.** On the first run eleven gates opened two files and exited 0 --
  a fleet of apparently empty checks. They had skipped: `t27c` was not where `_prereq` looks, and
  without `--require` a skip exits **0**. CI passes `--require`. Every number was a measurement of
  my own environment.
- **A gate that shells out reads nothing visibly.** The audit hook sees `open` in this process;
  `grep` and `t27c` read in a child. Hence the spawn column.
- And the first version swallowed `SystemExit`, so every gate reported exit 0 -- a refusal and a
  clean pass were indistinguishable.

## What it deliberately does not claim

Coverage. Files-opened against files-in-the-tree is the wrong ratio and produced the only wrong
conclusions here: `check_assertionless_spec_tests` reads 650 of 745 `.t27` and is **complete**,
because its subject is `specs/`. The denominator has to come from the gate, and none declares one.

Refs #3153
