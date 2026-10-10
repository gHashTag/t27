# JTAG verdict spec literal repair (#8762)

Author: Dmitrii Fedorov (@dmitrii-f-t27). Parent corpus issue: #7641. This branch includes the dependency PR #8749.
Native tooling rebuilt after synchronizing with upstream cd569e3e0.

## Change and scope

The existing MVP classifier JTAG no-port specification used Rust-style `u32`
suffixes in number expressions. T27 already declares each required u32 context;
removing those unsupported suffixes preserves the values and 32-bit operations.
The original `StartupConfig.prog_usr: &str` type is retained. A literals-only
probe confirms it is supported; no shared configuration module is needed.

All original test cases and six invariants remain. The specification models
verdict-word packing, capture/shift gating, old-state TDO, initial register,
LED-derived flags and BSCAN/STARTUP configuration values. It does not instantiate
BSCANE2/STARTUPE2, synthesize a classifier, flash a bitstream or execute physical
JTAG. No complete model inference is claimed.

## Executed checks

- Complete parse: zero discarded tokens.
- Reference t27c/Zig: 18/18 tests and six compile-time invariants. Sixteen tests
  execute 28 assertions; the two constant-construction tests execute zero
  runtime assertions and are reported as vacuous by the reference counter.
- Native arm64 t27b `--check`: 18/18, six invariants held, 38 runtime assertions
  including constant checks and invariants; JIT/interpreter agree.
- A temporary copy expects A5A5A5A6 instead of A5A5A5A7 for beat=true, ok=true.
  Both runners fail exactly that test, exit 1, with 17 others passing.
- Normal, unforced `seal --save` records 18/18 and two reference vacuous tests.
  `seal --verify` matches spec and generated C/Rust/Verilog/Zig hashes. Only
  the Zig and native T27 paths are execution claims; other hashes prove generation.
- The native expectation ledger adds this measured pass without increasing
  its failure cap (10). Duplicate-body and type-conflict ratchets pass.

Evidence: `/Users/ssdm4/trinity-results/jtag-verdict-spec/`, including final
source, literals-only probe, both damaged-result reports and seal verification.

## Task-chain disposition

1. Request: repair eligible specs and close own work under @dmitrii-f-t27.
2. Existing source, original hardware boundary and corpus blocker checked.
3. Own issue #8762 specifies this exact literal-only scope.
4. Source change is entirely in the existing `.t27` specification.
5. No handwritten foreign code or author remapping.
6. Existing vectors, six invariants, negative control and normal seal exercised.
7. Own PR must link this report, close #8762 and reference #7641.
8. Self-review checks unchanged values/types and old-state TDO behavior.
9. Latest-head CI remains pending after publication.
10. Ordinary merge needs usable upstream write permission; no independent
    GitHub review is required under the owner's standing instruction.
11. Merge and closure remain pending until checks/access permit them.
12. Publication is the accepted spec; physical FPGA/service deployment is
    outside this literal restoration.
13. No physical inference, customer activity or reward claim follows.
14. This report retains completed tests and the remaining publication step.
