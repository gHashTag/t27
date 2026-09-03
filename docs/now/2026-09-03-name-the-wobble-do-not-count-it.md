# NOW -- Name the wobble, do not count it (2026-09-03)

## `tri emit determinism`: four specs, not three emitter defects (Refs #3006)

- new command generates the corpus N times with the SAME binary and prints the specs whose output differs, because #3006's own next step was "name the files rather than count them"
- measured over 650 specs, 2 runs: gen-c 4 differing, gen-rust 2, gen (Zig) 4, gen-verilog **0**
- the names change the finding: `specs/igla/coder/pipeline.t27` and `specs/physics/sacred_verification.t27` wobble in ALL THREE non-Verilog backends, and `specs/igla/coder/_tmp_pipeline_import.t27` and `specs/physics/zamolodchikov_4d_conjecture.t27` in two -- **four specs in total, largely the same ones**, which points at something shared rather than at three independent emitter bugs
- the COUNTS are not stable between measurements (an earlier harness read 1 / 3 / 2 where this reads 4 / 2 / 4) because which files wobble is itself a draw; only the union is meaningful, and it is these four
- refuses on `--runs 1` ("one run cannot disagree with itself") and on an unknown backend, naming what it refused; refuses with no compiler, because a determinism report of zero taken with no binary reads exactly like health
- reports and never gates: whether a wobbling file is a defect or an ordering nothing downstream reads is not a walker's question
