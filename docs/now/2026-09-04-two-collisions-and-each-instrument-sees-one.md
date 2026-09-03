# NOW -- Two collisions, and each instrument sees one (2026-09-04)

## A scratch key needs a per-call component AND a per-process one

- Four test binaries shared a scratch path with no per-process component.
  Measured with a control -- run alone, then 16 copies at once:
  `verilog_real_arithmetic` 0/8 alone and 41/64 concurrent, `backend_behaviour`
  30/32, `verilog_range_bound` 24/64, `generic_type_application` 10/192.
- The four-arm experiment on one file settles which half does what:
  neither 6/150 intra and 41/64 inter; `process::id` only 7/150 and 0/64;
  counter only 0/150 and 29/64; both 0/150 and 0/64. The counter separates the
  THREADS of one run, the pid separates concurrent RUNS, and neither alone is
  enough.
- `tri harness scratch` advised "an AtomicUsize counter, not the pid". The
  first half is right; the second is the 29/64 row. The advice now asks for
  both and carries the table, and two unit tests pin it -- one fails if the old
  sentence returns, one fails if an arm of the table is dropped.
- After the fix all four read 0/150 intra and 0/64 inter. That pair is the
  acceptance criterion, not a green ordinary run: none of them ever failed
  alone.
- Where the two instruments disagree is the finding. Reading proves the
  STRUCTURE of a race; running shows whether the window opens. `verilog_r_si_1`
  has the structure -- one literal path, two callers, a truncating write -- and
  0 of 64 concurrent runs plus 0 vacuous passes. `backend_behaviour` was
  refuted by reading, correctly, on the only axis reading was asked about, and
  is 30/32 on the other.
