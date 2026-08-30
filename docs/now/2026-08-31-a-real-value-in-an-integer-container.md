# A real value must not travel in an integer container

- `gen_verilog_expr` sent every `*` through `__mul_noop`, the integer shift-add
  ladder from #741, so `ewma_step(5.0, 0.3, 10.0)` returned **5.0** where C,
  Rust, Zig and hand arithmetic all say **6.5**. R-SI-1 governs synthesizable
  RTL; Verilog `real` is simulation-only and was never in that rule's subject.
- A `given` binding was emitted `reg [63:0] e;` whatever it held, so 0.75 was
  stored as 1. Its sibling passed the first fix alone only because its expected
  value, 4.0, survives rounding -- an accident, not a check.
- `specs/trinet/etx.t27` under `t27c icarus-simulate`: **5 PASSED / 6 FAILED,
  exit 1 -> 11 PASSED, exit 0**. 46 of 581 generated `.v` change; `iverilog`
  accepts 380 before and 380 after, because the ladder always produced valid
  Verilog -- it computed something else. Seals for the 46 re-sealed here.
- I predicted three tests would flip and that the three `alpha_*` would not.
  One flipped; all six moved once the binding was fixed too. Wrong in both
  directions, and the correction is the finding: one class, two sites.
- The `ExprLiteral` arm first tested `extra_kind == "float"`. Nothing in this
  compiler sets that string. The mutation that exposed it was itself broken
  first (`false && A || B` disables only the left disjunct), so a live arm read
  as dead while the instrument was.
- Filed, not fixed, from the same sweep: #2987 the Icarus gate has had zero
  targets since `specs/scratch` was untracked, #2988 `break` lowers to
  `disable fork;` with no `fork` anywhere in the corpus, #2989 an early return
  inside a loop leaves the loop running and `binary_search` never terminates,
  #2992 `implies` appears 82 times in the corpus and 0 times in the compiler.

Closes #2990
