# NOW -- Two of five ledgers announced a stale entry and exited zero (2026-08-30)

## Two of five ledgers announced a stale entry and exited zero (Refs #2864)

- Yesterday a stale ledger entry was found by ACCIDENT -- the corpus ratchet went red and named it. Asked deliberately this time: plant a line naming a spec that PASSES, run each gate, see who notices.
- seal_baseline, conflict_markers and the corpus ratchet FAIL. specs_generate_baseline and verilog_width_baseline printed a note and exited 0, so a line outliving its debt could live there forever -- gate green, note scrolling past.
- Both now fail, with the repair named in the message. Controls both ways: planted entry -> exit 1; no planted entry -> exit 0; both --self-check still pass.
- New: tri ledgers audit. It plants the false line itself and demands each gate fail -- a meta-gate over the ledgers. It refuses to run on a dirty ledger, because it restores what it rewrites. On the pre-fix gates it reports 2 MISSED and exits 1; after, 4 caught and exits 0.
- The sweep also corrected my own note: I had FOUR ledgers written down in memory and there are FIVE naming specs by path, plus one keyed by line hash.
