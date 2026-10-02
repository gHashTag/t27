# NOW -- `assert (a & b) == c` is one condition, not `assert(a & b)` then `== c` (2026-10-02)

## bootstrap/src/compiler.rs -- the bare assert may open with a parenthesis

- The statement parser sent every `assert (` to the call path. So
  `assert (a & b) == c` parsed as `assert(a & b)` followed by `== c`. The
  parenthesised operand became the call's argument, and the comparison took the
  call as its operand. Typecheck said ok, and the Zig backend emitted
  `if (!(a & b)) @panic("assertion failed") == c;`, which `zig test` rejects
  ("unreachable code", "expected type 'bool'").
- Now a lookahead skips the balanced parentheses after `assert`. If a binary
  operator follows the matching `)` on the same line, the statement is parsed as
  the bare form, and its whole condition goes into the assert.
- `assert(x);`, `assert((a & b) == c);` and `assert(x, "msg")` take the old
  path, so their output is unchanged.
- Corpus, master vs this branch, over all 1146 specs:
  - `ast-dump`, `gen` and `gen-c` change for 3 specs: specs/cloud/railway_deploy.t27
    (4 asserts), specs/compiler/mod_structure.t27 (1) and
    specs/port/fpga/verilog/mvp_ternary_classifier_jtag_noport.t27 (3).
  - `gen-rust` and `gen-verilog` change for none.
- Under `zig test --test-no-exec`, every error those 8 asserts caused is gone:
  8 "unreachable code" and 5 "expected type 'bool'". Each of the 3 specs still
  has an unrelated error that master has too, and no new error appears.
- A text scan finds 5 more lines with this shape, but none of them was broken:
  - 4 sit in `invariant` blocks (specs/fpga/fifo.t27, specs/fpga/uart.t27,
    specs/numeric/e8m0.t27, specs/math/radix_economy.t27), which have their own
    parse path and were already right;
  - 1 is in a spec that fails to parse earlier, at an unrelated line
    (gf_decode_param_fp64.t27:354).
- Seals:
  - the FROZEN seal moves (FROZEN.md section 5);
  - railway_deploy.t27 and mod_structure.t27 are resealed with this branch's
    t27c (`t27c seal <spec> --save`, which also refreshes the second seal file
    of each spec). Only `gen_hash_c`, `gen_hash_zig` and `sealed_at` change;
    both specs verified MATCH on master before the change.
  - mvp_ternary_classifier_jtag_noport.t27 has no seal.
- Regression guard: bootstrap/tests/assert_paren_binary.rs. The bare form with
  `==`, `and` and `-` must emit exactly what the call form with the same
  condition emits, and the call forms keep their old output.
- Found by a t27 spec of the CLK_PERF / PHSR_PERFCLK segbits rows
  (openXC7/prjxray-db#30). It had to work around the bug with a helper
  function; with this fix its original `assert (x & y) == x` passes `zig test`
  7/7.
