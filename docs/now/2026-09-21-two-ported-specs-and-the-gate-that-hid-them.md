# NOW -- Two more ported specs were not t27, and the gate that should have said so (2026-09-21)

## specs/port/tools/{run_conformance_vvp,check_fix_carries_source}.t27 (Fixes #4560)

- Both landed on 2026-09-21 -- #4553 and #4556 -- and neither has ever compiled on any backend. `run_conformance_vvp.t27` is transliterated Python (51 `let`, 4 `try`, first error at `tb_case_spi_prescaler` line 19); `check_fix_carries_source.t27` is 565 lines of Zig with `let` (52 `std.*`, 30 `catch`, 27 `x += 1`, 25 `'\n' as u8`) against a Python source of 284.
- The transliteration also changed what the tool decides. The emitted testbench declares and increments `fails`, and the single line that reports success reads `if (folds == 0)` -- a name declared nowhere, so the pass line is unreachable. And `vvp_path`, bound to the compiled output at line 76, is rebound to the simulator binary at line 94 and passed twice: `vvp vvp`. Neither is a parse error; a port that only fixed the syntax would have kept both.
- Rewritten as specs rather than transliterations; `t27c check` reports 0 errors and 0 warnings for each and all four backends exit 0. `check_fix_carries_source.t27` deliberately does not restate `SOURCE_SCOPES` or `PROSE_EXT` -- those have one definition in `cli/tri/src/hooks.rs` and the Python parses it rather than copying it, so a third copy would be the defect that tool exists to catch. Its predicates take the answers to lookups, not the tables.

## tools/check_specs_generate.py answered one question and left (Fixes #4560)

- Four `return 1`s in a row meant the first class found was the only class reported, and `fixed` -- the mildest -- was checked three lines before `new`, the most serious. `adamw.t27` was repaired by #4540 and its ledger line was left behind, so every run printed one stale word about adamw and never reached the scan for newly-broken specs. Three landed in that window: #4533, #4553, #4556.
- All four classes now report and the exit code is taken once at the end. The stale-ledger branch also had no self-check case at all -- the only `return 1` in `main()` that nothing ever executed, and the one that caused this. It has one now, plus a case that plants a stale line and a new break in the same tree.
- The ordering case was measured against the defect, not just asserted: restoring the `return 1` makes `a stale line does not hide a new break` report CONTROL FAILED while the two branch cases correctly survive.

## What this does NOT establish

- That the 41 specs classed *Working* which call builtins that do not exist are working. `@thisBuiltinDoesNotExist(a, "nonsense", 1, 2, 3)` typechecks with 0 errors, passes `gen`, and reaches the C verbatim; 35 of those 41 emit an invented name into their generated C. Filed as #4561, not fixed here.
- That the generated C compiles. `undefined` lowers to `{0};`, which is invalid C, and `read_user1.t27` -- classed *Working* -- fails `cc` for that reason. A backend defect across every spec that stubs a body.
