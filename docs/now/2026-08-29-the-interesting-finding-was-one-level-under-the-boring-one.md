# NOW -- The interesting finding was one level under the boring one (2026-08-29)

## The interesting finding was one level under the boring one (Refs #2804)

- a guard titled for a CLASS greps one member of it; the unguarded spelling is in 33 files against the guarded one's 5
- a guard that lands red gets ignored, so the debt is pinned per file rather than fixed -- new, growing and shrinking all fail
- found by opening all six FPGA rows the detector reported: all are build outputs, and two of them are fragments joined onto a hardcoded absolute checkout root
