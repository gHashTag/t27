# NOW -- A dereference that did not tokenise (2026-09-08)

## A dereference that did not tokenise (Closes #3422, Refs #3420)

- Zig spells a dereference postfix, `count.*`, and the corpus is written that way. gen-rust emitted it **verbatim** -- `error: unexpected token: '*'`, so the file did not even tokenise. 70 sites across 6 specs. And a bare `*T` parameter became `*mut T`, whose every dereference needs an `unsafe` block this emitter never writes: **81 files** carry such a parameter, and none of them could have compiled. Zig handles `.*` natively because it IS Zig syntax; C renders the parameter `size_t*`. Rust was the only column that could not compile.
- The repair is narrow and the narrowing was **measured, not assumed**: `*T` becomes `&mut T` in PARAMETER position only, because a struct field cannot take `&mut T` without a lifetime.

  | version | rustc accepts | coded diagnostics | introduced | revealed |
  |---|---:|---:|---:|---:|
  | master | 433 | 2937 | -- | -- |
  | every position | 435 | 2724 | **9** | 1 |
  | **parameter only** | **439** | **2713** | **0** | 1 |

  Third time this session that narrowing beat widening on every axis at once.
- **How this was found: not by reading the compiler.** Ten of seventeen `rings/*` crates name a spec that exists, and a new tool compares them. `ring-099` read DRIFTED on exactly two functions, and the only difference was `count: &mut usize` in the hand-written model against `count: *mut usize` from the spec. The hand-written code was right again -- the second time in two passes (#3420 was the first).
- `tools/check_ring_spec_drift.py` classifies each pair CONVERGED / DRIFTED / UNRELATED, and its self-check requires every verdict to be reachable. It carries a warning it earned: **a matching signature is not matching behaviour.** ring-090 read 16 of 16 identical signatures and still disagreed on 126 of 1190 differential cases. Today: 2 CONVERGED, 0 DRIFTED, 7 UNRELATED -- and UNRELATED mostly means the doc comment names a spec the crate was never generated from, `ring-097` sharing 7 names with `proof_trace.t27` and **zero** signatures.
- Both guards from earlier passes worked again without being asked: the seal checker named 135 stale seals across 68 specs, and `--save` refreshed every duplicate including the two whose filenames are quoted type strings.
- Self-criticism: my mutation harness reported MUTANT B as surviving. It had not applied -- a broken anchor made the helper run `cargo test` on an unmutated tree and report its pass. Re-run with the anchor asserted, the mutant dies with the exact `E0133`. A mutation run that does not verify the mutation landed is measuring the same thing twice.
