# NOW -- the 8 specs the gen typecheck gate would refuse, fixed at the source (2026-10-05)

## Refs #6446: gft64..gft1024, ar/composition, git/diff, port/tools/check_conflict_markers

- On the Railway lab, the held gen typecheck gate (branch `claude/t27c-gen-typecheck-gate`) stopped 8 corpus specs from generating. typecheck refused each one, but `gen*` emitted code for it anyway. Each is now fixed in its spec, so it typechecks and still generates on every backend.
  - gft64, gft128, gft256, gft512, gft1024: `EXP_OFFSET` and `OFFSET_MAX` (and `is_finite`'s parameter) declared `u32` for values of 12 to 187 digits. The values stay as they are: each is (3^E_t - 1)/2 and its double. The type is now the storage word that `specs/numeric/formats_catalog.t27` already names for the rung: u64, u128, u256, u512 and u1024. The `golden_section` and `balanced_offsets` tests pass under zig.
  - ar/composition: `execute_ar_component` was a Rust `match` whose arms assigned a tuple. It is now a `switch` that returns each arm's (result, steps) pair.
  - git/diff: 21 Rust `match` blocks over `Ok`/`Err` of `Command::new("git")`. These are rewritten the way `specs/git/status.t27` is written: run the command, test the exit code, and hand the text to the pure parsers `parse_name_status` and `parse_numstat`. A new `summarize` function totals the stats. The tests now check those parsers on literal git output instead of on whatever repository they run in.
  - port/tools/check_conflict_markers: `staged_files` called the 4-parameter `_git` with 7 arguments. It now passes the git flags as one argument string.
- Ledger: the 7 `typecheck` entries for these specs are removed from `docs/reports/suite_expectations.json` (84 -> 77, and the cap follows). ar/composition keeps its `verilog-no-keyword-decl` entry, which is not a typecheck failure.
- 9 seals were re-saved on the lab with zig on PATH.
- The 9 port specs whose `gen-verilog` fails on a local string array (`[N][]const u8`) fail with or without the gate. They stay ledgered under #5549.
- Two more specs outside `specs/` were refused once the gate ran over the whole tree. Both are fixed here:
  - `compiler/codegen/verilog/codegen.t27`: called the method `mangle_verilog_name(self, name)` without `self.`. All 3 call sites are now `self.mangle_verilog_name(..)`.
  - `contrib/backend/zig/legacy/main_zig_handwritten.t27`: 14 calls passed a format and args to the 1-parameter `printError`. They now call the spec's own `printErrorFmt`.
  - Their 3 seals (`verilog_codegen`, `verilog_verilog_codegen`, `legacy_main_zig_handwritten`) were re-saved on the lab.
