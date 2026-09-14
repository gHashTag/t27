// SPDX-License-Identifier: Apache-2.0
# OWNERS — specs/memory/tmem/

## Primary

**Trinity Memory** (`dmitrii-f-t27/trinity-memory`, shared project of Dmitrii Fedorov and
Dmitrii Vasilev) — contract layer for the executable memory stack: trit lane codes, codec
geometry, TMEM/TTPK framing, Bridge protocol, stream ports, conformance schema and Edge
Demo scoring. Roadmap: [epic #3](https://github.com/dmitrii-f-t27/trinity-memory/issues/3).

## Dependencies

- The compiler pinned by `native/compiler.lock` (`gHashTag/t27`).
- The executable implementation in `t27/` and `t27/rtl/`. Specs restate contracts and are
  tied to the implementation by differential harnesses; they never replace it.

## Generates

- `build/t27/specs/<module>.h` — C with `_Static_assert` invariants and an executed test
  runner; `build/t27/specs/<module>.v` — Verilog syntax gate (`tools/check-specs.sh`).
- Seals in `.trinity/seals/memory_<Module>.json` (`t27c seal --save specs/memory/<file>.t27`
  from the repository root); vectors in `conformance/memory_<name>.json`.

## Conventions

- Header: SPDX line, `specs/memory/<file>.t27 -- <purpose>`, a short English description,
  then `phi^2 + 1/phi^2 = 3 | TRINITY`. Comments are English-first.
- Names: `pub const TMS_*`, functions `tms_*`, one brace-form `module` per file. The `TMS_`
  prefix keeps generated macros distinct from the implementation's `TM_*` names so both
  headers can be included by one harness.
- `invariant` blocks must be constant expressions over `const` values; the pinned compiler
  turns them into `_Static_assert` and silently comments out anything else. Computed checks
  belong in `test "..." { assert ...; }` blocks, which the generated C runner executes.
- An `assert` expression must not begin with `(`: the pinned parser folds it into a call
  (`assert((a)) == b`). Bind the value to a `var` first. `tools/check-specs.sh` rejects
  both failure modes.
- Every change to a spec: rerun `tools/check-specs.sh`, refresh the seal, regenerate the
  vectors with `tools/generate-spec-vectors.py`, and keep `tests/native_spec_types.c` and
  `tests/test_spec_types.py` passing.
