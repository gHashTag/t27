# NOW -- A real value must not travel in an integer container (2026-08-31)

## A real value must not travel in an integer container (Closes #2990)

- gen_verilog_expr sent every `*` through `__mul_noop`, the integer shift-add ladder from #741, so `ewma_step(5.0, 0.3, 10.0)` returned 5.0 where C, Rust, Zig and hand arithmetic all say 6.5; R-SI-1 governs SYNTHESIZABLE RTL and Verilog `real` is simulation-only, so a real multiply was never in that rule's subject
- second site, same class: a `given` binding was emitted `reg [63:0] e;` whatever it held, so 0.75 was stored as 1
- specs/trinet/etx.t27 under `t27c icarus-simulate`: 5 PASSED / 6 FAILED, exit 1 -> 11 PASSED, exit 0; 46 of 581 generated .v change; iverilog accepts 380 before and 380 after, because the ladder always produced valid Verilog -- it computed something else; seals for the 46 re-sealed in the same commit with that acceptance reading as the reason
- I predicted three tests would flip and that the three alpha_* would not; one flipped, then all six moved once the binding was fixed too -- wrong in both directions, and the correction is the finding
- the `ExprLiteral` arm first tested `extra_kind == "float"`, a string nothing in this compiler ever sets; the mutation that exposed it was itself broken first (`false && A || B` disables only the left disjunct), so a live arm read as dead while the instrument was
- mutation-checked 5 of 5; verilog_r_si_1.rs is the control that the integer path is unchanged; rustfmt --check reads 315 complaints on compiler.rs before and 315 after
