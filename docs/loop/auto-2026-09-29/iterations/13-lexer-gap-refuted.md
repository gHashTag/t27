# Iteration 13 — the "lexer gap" refuted: escapes are native; the converter goes green

Date: 2026-09-29 (loop auto-2026-09-29, iter 13). Branch `loop/auto-2026-09-29`, PR #5084.

Goal (state.md priority (a)): measure the "string escapes `\"`" door — iteration 12
had named it the only language-side fault left in the census — then fix or file it.

## The refutation, in three measurements

1. **Probe** (`/tmp/t27_hyg/iter13_probe2.t27`): `\"` and `\\` inside a string
   literal lex, parse and typecheck natively — `nothing discarded`,
   `Typecheck OK (0 errors, 0 warnings)`.
2. **Corpus grep**: 38 specs contain `\"`; many are green (json, t.t27,
   pins/parser.t27). A lexer without escapes could not parse them.
3. **The spec itself**: `tri_to_t27_converter.t27` already uses single-escape
   `\n` in its own constants (SPDX_HEADER, line 13) and in later tests
   (lines 540-567).

Conclusion: **door 5 of the wall was misnamed in iteration 12.** There is no
lexer gap. The wall is FOUR doors (pointer/allocator APIs, method-calls,
generics `::`, #2774).

## What the census row actually was

Two spec-side defects in `specs/tools/tri_to_t27_converter.t27`:

| Line | Defect | Fix |
|---|---|---|
| 403, 408 | **Double-escaped strings** (Zig-source style): in t27 `\\` is a literal backslash, so `\\"1.0\\"` TERMINATED the string mid-clause and a bare `1.0` reached clause position | single escapes: `"name: test_algo\nversion: \"1.0\""` — the file's own idiom |
| 481 | **Unquoted test name with a space**: `test route_file maps_algo_dense` — the parser dropped the whole block (14 tokens) | `test "route_file maps_algo_dense"` — quoted names parse natively (probe-proven iteration 12) |

## Measured

- `parse-complete`: converter 48 → **0 discarded** ("nothing discarded").
- Typecheck after fix: **0 errors, 0 warnings** (all helpers — including the
  invariant helpers any_tri_spec/valid_tri_spec/… — resolve).
- Gate: **0 unexpected failures / 1 unexpected pass** — the converter left
  the ledger ENTIRELY (not re-pinned at a later phase; parse_tri_file being a
  stub is by design per the spec's own Implementation Notes: the contract is
  the spec, the implementation is `tools/converter/` Rust).
- Corpus: **129 → 128**; re-blessed **128/128 RATCHET CLEAN**.
- Census: **5,667 → 5,620 tokens (−79.6% cumulative)**, 85 → 84 discarding specs.
- The whole converter test section (~30 blocks: parse_tri_file contract,
  sanitize, is_valid_t27_syntax, route_file, parse_type_line, write_output)
  now emits — it sat behind the parse fallback since the spec's creation.

## Full census snapshot after iteration 13 (parse-no-discard, 84 specs / 5,620 tokens)

Top mass, for the next decomposition (each is a wall-family or #2774 member):
brain_summaries 560, ppo_actor 463, ternary_mac_demo_top 435, uart_echo_top
422, notebooklm 409, async_stream 213, red_black_tree 212, sdk_contract 207,
reader 114, io 112, sgd 104, ternary_add 96, phi_split_optimality 131,
compress 138, gla 151, sdk 56, ternary_shift 66, feed_forward 62, rsa 61,
multi_lang_harness 60, top_level 57, read_verdict 55, ternary_memory 52,
segment_tree 85, pilot_pretraining 82, constants 80, phi_ratio 83,
phi_universal_attractor 73, spi 72, circular_buffer 71, jones_polynomial 32,
config 41, logger 45, kd_tree 41, filesystem 38, vcd_conformance_compare 38,
disjoint_set 36, … (full list in the gate log; ~20 specs at 1 token each —
single stray tokens worth a batch pass one day).

## Self-critique

- Iteration 12 named "one lexer gap" from the census event WITHOUT a probe —
  the exact failure mode the iteration-9 lesson warns about (the row names the
  death site, not the cause). This iteration's probe refuted it in one run.
  The correction is recorded in REPORT.md and state.md; the honest wall has
  four doors, not five.
- I nearly shipped the first probe broken (missing module brace) and could
  have blamed the lexer for MY typo; the error message (EOF expecting RBrace)
  made the real cause obvious on read. Prove the probe compiles before it
  proves anything.
- The converter's `\\n` siblings (lines 413/419/425) keep literal backslash-n
  content — they parse and typecheck, and the suite does not execute them
  against the stub (the gate proved: no runtime failure). Left verbatim;
  changing them would be semantics repair outside this loop's parse mandate.

Phase complete: iteration 13 (lexer gap refuted, converter green)
→ next: iteration 14 — decompose the new top mass (brain_summaries/ppo_actor/
  port tops) against the four-door wall
