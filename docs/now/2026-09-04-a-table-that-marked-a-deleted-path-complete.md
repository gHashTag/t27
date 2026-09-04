# NOW -- A table that marked a deleted path COMPLETE (2026-09-04)

## Four CLARA deliverables, ✅ since April, gone since April

- `docs/META_DASHBOARD.md` marks four `docs/clara/` paths **✅ COMPLETE**. None
  is on disk. They were produced (`c1582008b`) and removed on **2026-04-19** by
  `91653d2b9` -- *"fix(bootstrap): restore working main.rs -- recovery from
  detached HEAD (#523)"*, a 1214-file recovery that took them as collateral.
- Corrected rather than deleted: the rows now read
  `DELIVERED, then removed in 91653d2b9`. A dashboard that quietly drops a
  deliverable is worse than one that says where it went.
- The checker is deliberately narrow. A loose matcher -- any backticked token on
  any line mentioning COMPLETE -- gives **232 tokens, 153 "missing" (66%)**, and
  is catching bare extensions (`.bit`, `.rs`), signal names
  (`BSCAN.JTAG_CHAIN_1`), URLs and markdown links. A markdown TABLE ROW naming a
  token with a slash, under a top-level directory that exists, gives **12 rows,
  8 missing, one file, four distinct paths** and zero false positives.
- Three mutants, and M2 took three attempts to kill. Neutering the
  missing-computation left the suite green, so a planted-row self-check went in
  -- and it STILL survived, because the self-check computed its own copy of the
  decision. **A control that does not run the code under test is a second
  implementation agreeing with itself.** One function, two callers, and it dies.
- Four more measured drifts from the same fan-out are filed rather than fixed
  here: `PIN_COVERAGE.md` names two Rust functions that no longer exist (0
  definitions in the tree), `CORPUS-RATCHET.md` justifies an exclusion of
  `specs/scratch/` that today holds **0 files**, `NUMERIC_FORMATS_83_METRICS.md`
  cites `gen/numeric/formats_catalog.json` which is absent, and
  `SYNTH_REPORT.md` declares **yosys 0.65** where the bench runs **0.68+post**.
