# NOW -- gen-zig keeps a discarded call instead of deleting it (2026-10-04)

## gen-zig keeps a discarded call instead of deleting it (Closes #5984)

- the shared optimizer's dead-store pass read `_` as a variable nobody reads and deleted `_ = call(..);` at the top level of a fn body, call and all; gen-zig now asks it to keep a discard whose right-hand side holds a call (OptConfig.keep_call_discards), so the Zig output carries `_ = call(..);` as gen-rust and gen-c already did
- a parameter read only by the restored call is no longer given a second `_ = p;`, which Zig rejects as a pointless discard; a parameter nothing reads still gets one, and a plain `_ = x;` or a call with no `_ =` comes out exactly as before
- measured over 1297 tracked .t27 files: gen-zig output changes for 5 specs, 9 calls come back, 2 spurious `_ = param;` lines and 1 `_ = json_str;` line go, and their 9 seals are re-sealed; gen-verilog is byte-identical for all 37 specs holding a discarded call, because the flag is off on that path
- zig ast-check, build-obj and test --test-no-exec on the 5 changed specs: 0/5 before and 0/5 after; each stops on its own earlier error, and the ast-check error lists are identical before and after
