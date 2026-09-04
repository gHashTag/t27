# NOW -- The channel was already in the tree, with a comment counting the times it was closed (2026-09-04)

## Third hand-written copy of a shipped check in one pass (Refs #2994)

- the mutation harness classified a mutant as *did not compile* by grepping `^error:`, and `cargo test` prints `error: test failed, to rerun pass …` when a **test** fails -- eight kills scored as eight refusals
- **the repair is already in `scripts/ci/test_ratchet.py`**: `RESULT = re.compile(r"^test result: (?:ok|FAILED)\.")`, and `if not targets or results == 0` refuses with *"A run that did not happen is not a run that passed."*
- its comment names the class: *"reading that as a clean set is the fail-open this repo has closed **ten times**"*, plus a second guard -- one target means `--no-fail-fast` was missing, *"the exact condition that hid 72 targets"*
- **three instances this pass, and each dropped the enumeration of what can be true:** the poll loop dropped `tri pr ready`'s refusal to score a check with `conclusion: null` AND `state: null`; the verb census dropped three of the four surfaces `tri` resolves through; the harness dropped the channel that exists only after compilation
- reachable by the same command every time: `git grep -n "test result"` returns the rule, in a file whose whole subject is reading a cargo log
- **and the gate nearly took a reading from a stale ruler mid-write:** `tri census pin --gate` went red on a **docs-only** change of 56 lines, reporting `SURFACE … 4 bounded -> 7 bounded`. Following its own "re-bless in the same commit" advice would have written **7** into the ledger over a reading no source change could produce
- the control said so first -- red on **clean master too** -- and the cause was one command: `find target/debug/tri -newer cli/tri/src/gates.rs` is empty, so the binary predates the code. A neighbouring session had tightened that census from 7 to 4 and blessed 4; `cargo build` and the gate prints `PASS`
- **a re-bless is a statement that the new output is wanted**, so it is the wrong response to a number the instrument produced and the tree did not. Control first, then age the ruler, then the ledger
- **measured and clean:** **0** classifiers in `tools/`, `scripts/` or `.github/` decide compilation by `^error:`. The defect lives only in ephemeral shell -- population **0 on disk, once per pass in practice** -- so it belongs in the skill and not in a gate
