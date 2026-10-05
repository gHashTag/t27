# NOW -- the fifteen non-generating port specs are redone (2026-10-02)

## What changed

- All fifteen specs listed in issue 5549 parse and generate again: check_json_parses, check_duplicate_declarations, check_duplicate_agreement, check_assertionless_spec_tests, verify_multitarget, ring_spec_differential, gft_generalize_demo, trinity_fpga_adapter, feed_untested, gen_w366, gen_w390, dsp_probe, vsa, queen-ci-verdict, gf_decode_param_fp64.
- tools/specs_generate_baseline.txt: fifteen lines removed; the gate can catch the next regression.

## Verification

- `t27c typecheck` passes on every one of the fifteen; `t27c gen-c` and `t27c gen` produce output for each.
- Pure arithmetic kept (GF-T sadd/smul/magmul family, FP64 decode); host actions (file I/O, subprocesses, GitHub API) are stubbed, matching the sibling ports.
