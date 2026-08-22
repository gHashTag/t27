# NOW -- the ratchet records the iverilog it was calibrated with (2026-08-23)

## the first CI run matched 186 vs 186, which is luck rather than a property (Refs #2325)

- The elaboration ratchet compares counts produced by a SPECIFIC iverilog. The
  baseline was taken locally with 13.0; the runner installs its own from apt.
  The first master run matched exactly (186 vs 186) -- and a match once is not
  a guarantee: an apt upgrade can move every count without a line of the
  compiler changing, and the gate would then report a regression that is not one.
- The baseline now carries "# iverilog-version <v>", and a failure whose version
  differs says so FIRST, before the rows, so the reader diagnoses calibration
  rather than the compiler. This is the broken-ruler rule applied to my own
  instrument: an instrument should say when its calibration may have changed.
- Negative control: baseline version forced to 12.0 with one module lowered ->
  the NOTE about the version prints above the WORSE row; restoring returns 0.
