# NOW -- Four justifications for subjects that are gone (2026-09-04)

## Each names something the tree no longer has

- `docs/fpga/PIN_COVERAGE.md` lists `xdc_qmtech_minimal()` and
  `xdc_qmtech_full()` as "Rust pin definitions". **0 definitions in the tree** --
  added `1911edc80` (2026-04-09) with the `gen-xdc` command, removed
  `692ba5263` (2026-04-14). The subcommand went with them.
- `docs/CORPUS-RATCHET.md` justifies excluding `specs/scratch/` as "606,113,688
  of 612,924,235 bytes (98.89%)". That directory has been **out of the tree
  since `2255e4c32`** -- 0 files in the index, absent from disk. The exclusion
  excludes nothing, and the saving it claims was banked by the untracking. The
  untracking commit puts the same exclusion at **64.5%** where the doc says
  **98.89%**: two numbers for one subject, neither re-takable.
- `docs/metrics/NUMERIC_FORMATS_83_METRICS.md` cites
  `gen/numeric/formats_catalog.json` (**77 formats**) as a data source. That file
  was deleted by `aa01dd4f1` -- *"untrack stale gen/numeric catalog artifacts
  (**drift 77 vs SSOT 83**)"*. **It was removed for being 77 against 83, and the
  doc still publishes 77 and points at it.** The re-takable source is the catalog
  itself: `grep -c 'CATALOG:'` = **109**.
- `docs/SYNTH_REPORT.md` declares every figure measured with **yosys 0.65**; the
  bench reports **0.68+post**. Not re-measured here -- that needs a run, not an
  edit -- but the instrument is named beside the numbers now.
- Measured and DECLINED: extending the status-table check from paths to code
  symbols. 8 table rows name a `fn()`, 5 "missing" -- 62% -- and the population
  is contaminated: `uart_tx_ready` is a `.t27` function that exists,
  `quantize_groups` is an RFC proposal. Eight heterogeneous rows do not support
  a detector. The finding survives; the tool does not.
- Measured and CONFIRMED: excluding `docs/reports/**` from the status check.
  **1566 of 1569** name a wave, ring or date; the three that do not are a
  reported-upstream note, an open question, and a PR body -- none a status claim.
  My first measurement of my own exclusion said 63 undated, and was wrong:
  `W\d{3}` does not match `WAVE_LOOP_170`.
