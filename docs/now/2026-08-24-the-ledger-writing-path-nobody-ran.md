# NOW — the ledger-writing path nobody ran (2026-08-24)

`--loud` found eight sites where nothing required a gate to be silent. Five are closed; the three that remain are a different class and are named rather than counted.

- **Eight → three.** Four were the success return of `--update-baseline`, right after *"baseline written: N entries"* — `check_json_parses`, `check_seal_coverage`, `check_specs_generate`, `check_withdrawn_live`. **Nothing in the tree ran the ledger-writing path at all.** A `--update-baseline` that wrote the ledger and then reported failure was invisible to every control.

- **Each case asserts the exit AND the effect.** The exit alone would pass a run that returned 0 without writing; the marker alone would pass one that wrote and then reported failure — which is the mutation that found this. `check_specs_generate` already had an `--update-baseline` case, but it asserts the **refusal** to grow the ledger; the success path needed a tree where the ledger *shrinks*.

- **The fifth was mine, and the worst.** `check_gate_preconditions.py` — the file that enforces this discipline for six other gates — printed `OK: 9 precondition(s) across 6 gates fail loudly` and **exited 1** with nothing noticing. Its control now runs the whole program on a tree where every gate is healthy and asserts exit 0 *and* the count, because an exit 0 from a table that silently emptied would satisfy the code alone.

- **The three that remain are report modes, not verdicts.** Each sits behind a flag CI never passes:

  | site | guard | the file's own words |
  |---|---|---|
  | `check_specs_parse.py:302` | `--all` | *"a report, not a gate"* |
  | `check_elab_ratchet.py:440` | `--allow-missing-tools` | *"reporting nothing, deliberately"* |
  | `check_specs_generate.py:375` | a counts-table branch | — |

  Covering them means asserting that report modes exit 0. That is worth doing and it is not the same finding, so it is left with its reason rather than folded into the count.

- **Method note.** The four `--update-baseline` fixes came from one shape read once, not four investigations. `--loud` grouped them by putting the same line in four reports; the reading was what turned "eight survivors" into "one class and three exceptions".

Refs #2492
