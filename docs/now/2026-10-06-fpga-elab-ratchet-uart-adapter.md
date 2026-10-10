# NOW -- fpga-conformance: elaboration ratchet green again (2026-10-06)

## uart restored, adapter recorded, ternary_isa a named gap (Closes #5908)

- Master's `FPGA Build / fpga-conformance` failed at the elaboration ratchet: `178 (baseline 176)`, rows `NEW adapter 0`, `WORSE ternary_isa 6 -> 7`, `WORSE uart 0 -> 1`.
- `uart` 1 -> 0, fixed in the spec. 942a00132 (#4651) replaced `fn on_comb(data: u8) -> bool { return uart_tx_send(data); }` with an `on_clock` reproduction of compiler issue #2364, and the generated module failed with `Could not find variable on_clock in ZeroDSP_UART`. The original line is back; its ports `data`/`result` are what `contrib/formal/uart_formal_props.v` binds. The three seals naming `specs/fpga/uart.t27` are resealed by `t27c seal --save` on the Railway t27c lab.
- `adapter 0` is a new module with no errors (d4a71646f). Recording it hides nothing.
- `ternary_isa 7` is recorded as a KNOWN GAP, not fixed: gen-verilog W459 binds a test-local array to a module array (#6685). The compiler fix is PR #5948 (hand-written Rust, needs the owner's `owner-approved-foreign` label). When it lands, the row must return to 6.
- Measured on the lab with master's t27c: `elaboration errors: 177 (baseline 177)`, exit 0; `--self-check` passes. The baseline header now says iverilog 12.0, the version CI installs.
