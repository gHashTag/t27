# NOW -- The proof was provable because the model was wrong (2026-08-30)

## An audit found from the other side what 341 declined to risk (Refs #2893)

- `specs/tri/utils/args.t27` declares `fn parse(allocator: std.mem.Allocator)` -- ONE parameter
- its Lean model declares THREE: the dotted type path was split on its dots into `allocator`, `mem`, `Allocator`
- and the env was given `("Std", [("value", .u32)])` to match -- a name in ZERO of the 650 specs and exactly once in the Lean file
- that fabrication is load-bearing: `Ty.isLowerableFuel` rejects a `.struct` with empty fields, so a faithful model makes the theorem FALSE
- the compiler agrees with the faithful reading: `icarus-lowerable` prints `not_lowerable`
- and it is not mirroring the AST: `t27c parse` shows one parameter with the dotted path intact
- the same split is in 16 models; 15 assert `= false` where a wrong model changes nothing, and the ONE asserting `= true` is where it decides the answer
- an unfaithful model is worse than an empty one: the 114 empty ones announce themselves, this one lists four correctly-named functions in spec order and looks checked
