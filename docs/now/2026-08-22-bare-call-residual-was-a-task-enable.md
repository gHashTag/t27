# NOW -- a hoisted bare call's residual was a task enable (2026-08-22)

## class (3) of #2413 is dead: 12 -> 5 self-test errors (Refs #2413)

- A bare call in statement position gets its value hoisted into a W557 temp;
  the statement then emitted the RESIDUAL temp name alone -- and a bare
  identifier statement is a task enable in Verilog ("Enable of unknown task
  _t27_call_tmp_..."). The materialized assignment had already performed the
  call, so the residual said nothing: it is now skipped, guarded to the
  test/bench temp mode.
- mac self-test: 12 -> 5 elaboration errors; every survivor belongs to the
  slice-dependent test (class 1, rides #2410). 32/32 yosys smoke. M5.
