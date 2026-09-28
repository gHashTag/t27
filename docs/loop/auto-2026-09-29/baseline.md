# baseline.md — the measured starting inventory (2026-09-29, ~01:00)

Tree: `loop/auto-2026-09-29` = `origin/master` 2925def9b, untouched (no loop
edits yet). Suite: `t27c suite --repo-root .` → exit 1, JSON kept at
`/tmp/loop_suite_i1.json`, raw log `/tmp/loop_suite_i1.log`.

## Phase table (from the suite's own summary)

| phase | fails | notes |
|---|---|---|
| parse | 23 | specs the parser refuses outright |
| **parse-no-discard** | **114** | **parser reached EOF having silently DISCARDED top-level tokens** |
| typecheck | 36 fail lines / 26 specs | includes the 9-spec "line 6/7" cluster |
| gen-zig / gen-rust / gen-c | 23 each | gated upstream by parse |
| gen-verilog | 24 | +1 own failure |
| gen-verilog-yosys-smoke | 14 | |
| **no-vacuous-invariant** | **65** | **invariant bodies discarded — declared, checked nothing** |
| seal-verify | 681 lines / ~573 specs | stale seals |
| verilog-no-keyword-decl | 1 | |
| corpus PRIMARY | 148 | the suite's honest headline red count |

DISTINCT FAILING SPECS: 721 (of them corpus: 148; the rest is dominated by
seal-verify across the tracked tree).

## The finding that matters: silent discard, not loud refusal

Three phases fail for ONE underlying reason — the parser's error recovery
**discards and continues**:

1. `parse-no-discard` (114 specs): tokens dropped at module level, no error.
   Worst offenders by discarded-token count in `baseline/discard_ranked.txt`:
   `specs/igla/race/ternary_inference.t27` **1813**, `ternary_gemm.t27` 1566,
   `systolic_ternary.t27` 1409, `specs/ml/rl/ppo_actor.t27` 463,
   `specs/queen/brain_summaries.t27` 560 — and the numeric-format family
   `gf8/12/20/24/32.t27` each lose 31–37 tokens (L6-adjacent territory).
2. `no-vacuous-invariant` (65 specs): "invariant(s) declared but not lowered:
   the clause body was discarded and nothing is checked" — a spec-first
   compiler whose invariants silently check nothing is the inverted promise of
   the whole approach. `specs/base/ternary_add.t27` declares 10 such.
3. `parse` (23 specs): outright refusal, ranked by construct in
   `tri unparsed report` (body-less `fn` prototypes ×3, Rust macro calls ×2 —
   including the truncated `verilog_bench_harness.t27`, map types, `import`,
   `type T = U` aliases; 6 specs still "not decided" = the Markdown-headed
   cluster: `tri_net_api`, `bench_nn`, `gf16_bfloat16_nmse`, `e2e_scenarios`,
   `e8_lqg_bridge`, `hslm_benchmark`).

The `.tri` codegen silent-garbage measured in
`specs/mcp/server_registry.t27`'s header is the same disease one stage later:
continue-and-emit instead of fail-loudly.

## Artifacts

- `baseline/loop_failing_specs.txt` — 721 distinct failing specs
- `baseline/loop_fails_detail.txt` — 1027 phase|spec|message rows
- `baseline/typecheck_specs.txt` — 26 specs failing typecheck
- `baseline/discard_ranked.txt` — the 114, ranked by discarded-token count

## What later iterations are judged against

Suite reds may only shrink. The comparison command after any change:

```
t27c suite --repo-root . > /tmp/loop_suite_N.log 2>&1
diff <(sort docs/loop/auto-2026-09-29/baseline/loop_failing_specs.txt) \
     <(grep '^FAIL' /tmp/loop_suite_N.log | sed -E 's/^FAIL [a-z-]+ \(([^)]+)\).*/\1/' | sort -u)
```

Lines only in the right-hand side (new reds) are a self-revert trigger.
