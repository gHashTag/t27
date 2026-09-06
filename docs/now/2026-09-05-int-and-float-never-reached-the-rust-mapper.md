# NOW -- int and float never reached the Rust mapper (2026-09-05)

## int and float never reached the Rust mapper (Refs #3333)

- Fourteenth instance of one class: the rule exists in a neighbouring emitter and did not
  travel. `t27_array_type_to_zig` carries `float`, `double`, `int`, `uint`, with a
  comment naming the same defect on the C side; the C emitter matches `float`/`double`
  in three places; the Rust mapper knew none of them and they reached rustc verbatim.
- Found by censusing the FIRST rustc error of every failing spec: `cannot find type` was
  the largest class at 31, and `int`/`float` were inside it.
- **The column does not move: 336 both sides, zero regressions.** Two pinned binaries of
  distinct hashes over 651 specs.
- What moves is the emission. 5 specs change their generated Rust -- `pub step_id: int`
  becomes `pub step_id: i32` -- and `cannot find type int/float` was the FIRST error on
  3 of them and is now the first error on none. Their new first errors are `K_TRUE` and
  `Trit`, which is how the fix is known to have landed rather than assumed.
- Deliberately narrow: capitalised `Int` and `Float` are NOT mapped. No neighbour
  answers those, so mapping them would be a decision rather than a transfer, and this
  pass has no measurement to support one.
- Method note: the first measurement was discarded. I rebuilt the compiler while the
  baseline run was still going -- the ruler moved under the measurement -- so both sides
  were re-run against binaries pinned before either started.
