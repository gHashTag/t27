# NOW — the same trap in three places (2026-08-24)

A reviewer found that `tri gates mutate` never cleared Python's bytecode cache. It was fixed there two days ago. Today I hit the identical defect by hand, and then a grep found it in two more files that nobody had run into yet.

- **How Python loses an edit.** A `.pyc` is keyed on (source mtime in **whole seconds**, source size). A one-line edit that preserves the size, followed by a run inside the same second, is served the previous state's bytecode. `return 1` → `return 0` preserves the size. So does `else 2` → `else 0`.

- **It cost a round.** Five hand mutations of `wp18_conformance_gate.py` in quick succession left a `.pyc` that made its control report **five failures on a tree whose `git status` was empty and whose source sha matched HEAD**. Every "mutant killed" verdict taken before that discovery was suspect — not wrong, suspect, because a control can be red for the right reason or the wrong one and the output does not distinguish them. All were re-run after clearing the cache; they held.

- **The sweep.** A grep for files importing a sibling module found three that matter, none of them guarded:

  | file | imports | why it matters |
  |---|---|---|
  | `wp18_selftest_gate.py` | `wp18_conformance_gate` | the control reads a stale gate |
  | `wp18_gate_selfconsistent_selftest.py` | `wp18_conformance_gate` | same, second control |
  | `verify_exhaustive.py` | `ternary_model` | **the gate compares against a stale independent model** |

  The third is the worst of the three. `verify_exhaustive` is the gate that proves four backends agree with a model written separately; a stale model means the comparison is against a version nobody is looking at.

- **Proven on the worst one, both directions.** Breaking `ternary_model.py` with a same-size edit → exit 1. Reverting and running immediately → exit 0. Without the guard the second run reads the broken model.

- **Where the repair belongs.** Fixing a hazard inside one tool leaves it live wherever a person does the same thing by hand — and the manual path is the one used *while investigating*, which is when a false red costs most. The three lines go in the artefact both paths reach.

Refs #2472
