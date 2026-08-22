# NOW -- escaped where it is declared, bare where it is used (2026-08-23)

## the defect (Refs #2325)

- A parameter named `cross` -- a SystemVerilog keyword -- is emitted as
  `\cross ` in its declaration and as `cross` in the part-select that reads
  its fields. iverilog answers with a bare `syntax error`. 4 in the corpus.
- Cause: one variable serves two jobs. `base_name` is the key that looks the
  type up in `local_types`/`param_types` (must stay raw) AND the text that
  gets printed (must be escaped). Five places printed the key.
- This is the SECOND time this exact shape was fixed here. W643/T53 fixed it
  for a keyword-named local ARRAY -- declaration and initialiser -- and the
  expression paths were already correct. Nobody swept the part-select paths.

## the count went UP, and that is the fix working

- clock_domain 4 -> 5. Its 4 entries were syntax errors, and a syntax error
  TRUNCATES the file: the 5 real elaboration errors behind them (`a_name`,
  `b_name` -- string reads, #2433) had never been reached, so never counted.
- Written next to the number in the baseline, because a reader who sees only
  "+1" will read it as slippage. Syntax errors are worth more than their
  count: each one hides an unknown amount behind it.

## controls

- New test asserts the escaped form at the USE site; removing the fix at one
  of the five sites makes it fail. Verified both directions.
- `local_array_named_after_a_verilog_keyword_is_escaped` fails on this branch
  AND on clean master (#2292) -- checked before assuming authorship.
- yosys 32/32; conformance 18 mac + 3 spi still PASS.
