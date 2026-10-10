# Two blocked specs can execute and seal (issue #8745)

Author: Dmitrii Fedorov (@dmitrii-f-t27). Validation base:
`1c72b2786` (master), compiler built with `cargo build --release -p t27c`,
`t27c 0.5.2`, Zig 0.16.0. Related audit: #7641.

## Changes

`specs/ui/viewport.t27` compares each capacity with the integer quotient of
its pixel budget and row size. This is equivalent to the old pair of
inequalities for the positive row sizes, without multiplying two `u8` values
into a pixel count above 255. All public constant types and values remain
unchanged; the website consumer's schema continues to accept the spec.

`specs/port/fpga/vivado/gf16_matmul_top.t27` compares the two Boolean LED
inputs instead of XORing a Boolean with a `u32`, and masks the counter to
23 bits after each tick, matching `reg [22:0] counter` in
`fpga/vivado/gf16_matmul_top.v`. Two tests cover bit-19/20 transitions and
wrap from 8388607 to zero. The existing `dot_ok = true` assumption remains:
this spec tests a counter/LED model, not a dot product or a board.

## Executed validation

| Check | Result |
|---|---|
| Unmodified master copies of both specs, `t27c test-report` | BLOCKED, exit 2 |
| Viewport, `t27c test-report --verbose` | 6/6 pass, exit 0 |
| Current website viewport generator, `buildViewport` with compiler WASM | Clean schema/verdict; 6 tests, 43 asserts, no failures |
| Counter/LED, `t27c test-report --verbose` | 7/7 pass, 21 runtime asserts, no vacuous passes, exit 0 |
| Temporary viewport row changed from 54 to 55 | Test failure, exit 1 |
| Temporary counter mask removed | Wrap regression fails, exit 1 |
| `t27c seal <each spec> --save`, then `--verify` | Both test seals pass; spec and C/Rust/Verilog/Zig hashes match |

Four viewport tests are compile-time constant checks and the native runtime
counter reports zero executed asserts for them. They are not claimed as
runtime coverage; the website evaluator explicitly checks all 43 asserts.
The compiler build reports 631 existing warnings; this PR does not change
compiler sources. Generated output is produced by `t27c`, not edited by hand.

There is no `make t27-test` target. The equivalent checks above execute the
specs' existing six-size viewport matrix and counter boundary vectors.
No external wire protocol or conformance format changes.

## Seven other specs in #7641 remain blocked

These are fresh measurements on the same compiler and Zig version. Every
row exited 2, so none has a passing executable test result.

| Spec | First generated Zig error |
|---|---|
| `compiler/rust_repeat_counts.t27` | Repeat count loses grouping: tuple and comptime integer are incompatible |
| `port/fpga/verilog/mvp_ternary_classifier_jtag_noport.t27` | Rust-style `u32` literal suffix is emitted unchanged into Zig |
| `port/tools/check_conflict_markers.t27` | Anonymous result retains `&str`, invalid in Zig |
| `port/tools/jtag/link_relay.t27` | A `u32` is assigned where `u8` is required |
| `port/tools/jtag/mpsse_jtag.t27` | Unused function parameter `self` |
| `port/tools/jtag/read_user1.t27` | Undefined host boundary result is discarded incorrectly |
| `port/tools/jtag/read_verdict.t27` | Missing `Mpsse` declaration |

The JTAG specs also contain host stubs and incomplete cleanup behavior.
Changing assertions or casting their way past the first diagnostic would not
establish a working port. #7641 stays open; this PR closes only #8745.

## Task-chain disposition

1. Request: restore the two specified executable tests, attributed to the owner.
2. Instructions, current base and existing work checked; isolated worktree used.
3. Own issue #8745 defines this scope; #7641 remains the parent audit.
4. Behavior and regressions are in the `.t27` specs.
5. Changes are spec-only; no foreign implementation or gate change.
6. Tests, negative controls and generated seals are recorded above.
7. Own PR links #8745 and the parent audit.
8. Review must verify the final diff and CI for the latest PR head.
9. Any remaining red check must be investigated; inherited red is not green.
10. Maintainer action is needed only if permissions prevent an ordinary merge.
11. Merge is pending until all applicable CI is green and conflicts are absent.
12. Publication is the accepted repository/spec version; no service release is
    required for this test restoration. No deployed website change is claimed.
13. No physical FPGA test, full inference, credit or reward is claimed.
14. This report and the PR record preserve the result and remaining work.
