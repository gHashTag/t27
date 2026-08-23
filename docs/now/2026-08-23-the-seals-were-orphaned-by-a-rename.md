# NOW — the seals were orphaned by a rename (2026-08-23)

`check_seal_coverage` has been red for weeks with 136 stale seals. Most of them are not stale work; they are duplicates left behind when the seal naming convention changed.

- **`t27c seal --save` writes `<dir>_<Module>.json`; the old files are bare `<Module>.json`.** Re-sealing `specs/ml/layers/avgpool2d_layer.t27` updates `layers_Avgpool2d.json` and leaves `Avgpool2d.json` untouched and stale forever. Re-sealing 94 specs moved the count by five.

- **Measured over the 121 that remain:** 99 have a prefixed twin, 97 of those twins name the **same** `spec_path`, and **81 twins are current** — their `spec_hash` matches the file on disk today. Those 81 stale seals are superseded records whose replacement is present and correct. 18 twins are themselves stale. 22 have no twin at all.

- **Two twins are not twins.** `Bridge_Testbench.json` maps to `testbench_APB_Bridge_Testbench.json`, a different `spec_path`. Name-matching finds them; they are not the same record, and a bulk rule keyed on the name would have merged two different things.

- **23 seals genuinely re-sealed.** Their four generated hashes actually changed, because the specs behind them were repaired — the top causes are commits named *repair corrupted type annotations*, *repair 13 corrupted module declarations*, *drain the compile-failure queue*. The seals described the broken versions.

- **71 re-seals reverted as noise.** `t27c seal --save` rewrites `sealed_at` on every run, so 71 of the 94 files it touched differed **only** in a timestamp with every hash identical. Committing those would have put 71 meaningless diffs in front of a reviewer alongside 23 that matter.

- **Filed, not done:** removing the 81 superseded seals. Deleting a reproducibility record is a distinct failure class in this gate's own doctrine, and doing 81 of them at once is an owner's call rather than maintenance.

- **A tool defect found on the way.** `t27c seal <spec>` exits **0** on a spec all four backends reject, printing `gen_hash=none`; only `--save` refuses. I used the read-only form as a dry run and it reported 109 of 109 sealable. `--save` then accepted 94 and rejected 15. The dry run measured whether the command ran, not whether the thing it was asked about was true.

- **And a broken ruler of my own, twice in one investigation.** The dry run above, and then a freshness check that read `seal["spec"]` when the field is `spec_path` — so it reported 0 of 99 twins current when the real answer is 81. Both were caught by a result that did not fit, not by reading the code.

Refs #2474
