# NOW -- three specs whose tests did not test now test what they claim (2026-10-07)

## specs/ml/layers/layernorm_layer.t27, specs/automation/wrapup-auto.t27, specs/port/fpga/verilog/ternary_mac_synth.t27 (Closes #7404)

- layernorm_layer: `forward` assigned names it never declared, so no backend could run it. It now computes the mean and the biased variance over the input and writes `(x - mean) / sqrt(variance + 1e-5)`; five tests check known outputs for [1,2,3,4] and [12,14,16,18], an empty input, one element and a constant input. Reference 5/5 pass with 0 vacuous; 8 of 8 hand mutants of `forward` fail a test.
- wrapup-auto: the only test had no braces, so the reference ran nothing. `wrapup_run` is Python (contrib/backend/notebooklm/wrapup_auto.py) and stays NOT CHECKED in a comment; the test now builds a `WrapUpInput` and checks its fields at run time (5 runtime asserts).
- ternary_mac_synth: `[a,b]u8` in 13 places is now `[a, b]`; the typed form was printed as `.{ _ }` by the reference's Zig backend. gen-verilog, gen-c and gen-rust output is unchanged. Reference 10/10 pass, 0 vacuous.
- Resealed on the t27c lab: Layernorm, layers_Layernorm, automation::wrapup (both copies), and a first seal for verilog_ternary_mac_top. check_seal_coverage and check_seal_currency both exit 0.
- t27b ledger (targeted corpus run on the t27b lab): layernorm_layer blocked -> pass (17 asserts), ternary_mac_synth blocked -> pass (11 asserts), wrapup-auto stays blocked with a new first blocker, `ExprArrayLiteral(to slice field)` (a str array in a struct field, which needs global relocations in t27b). max_not_pass 24 -> 22 (recounted after the master merge).
