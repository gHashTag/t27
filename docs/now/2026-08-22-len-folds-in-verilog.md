# NOW -- .len() folds to the declared length in Verilog (2026-08-22)

## class (2) of #2413 is dead; three parse shapes, one fold (Refs #2413)

- The Rust backend resolved `.len()` for ages; Verilog emitted the method call
  VERBATIM and iverilog rejected it (arrays have no methods). The fold now
  covers all three parse shapes the same source text produces: method-kind
  (assert path), call-over-field-access (given-binding path), and the
  qualified name. Receiver length comes from the type registries through
  parse_array_type -- symbolic dims already resolved by the #2402 pass.
- mac self-test: 13 -> 12 elaboration errors; the remainder is #2413's
  classes (1) slice-dependent test (rides #2410) and (3) _t27_call_tmp task
  enables. 32/32 yosys smoke unchanged. M5 performed.
