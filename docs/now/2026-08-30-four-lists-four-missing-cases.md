# NOW -- Four hardcoded lists, four missing cases (2026-08-30)

## The same defect four times, in four emitters (Refs #2875)

- Verilog test-block statements omitted `StmtIf`: a test that could not fail, reported PASSED
- Rust `has_body` omitted `StmtAssign`: 53 functions emitted as `{ unimplemented!() }`
- Rust `expr_is_bool` omitted `ExprFieldAccess`: `!x.flag` became `(x.flag) == 0`, E0308
- `compound_binop` omitted `/=`: `x /= 2` emitted as `x = 2`, in three backends
- none is a subtle algorithm; each is one identifier absent from a `matches!`
- a hardcoded list of node kinds is a CLAIM that the enumeration is complete, and the claim is checkable
- every one of the four was found by comparing two backends on the same spec line -- the disagreement IS the finding, no golden file needed
- this project generates four languages from one source, so it carries three oracles for every construct, at one command each
- where that fails: a defect all four share. `while (c) : (step)` was wrong everywhere, so no comparison could see it

## New: `tri kinds drift`

- reports a match arm whose own comment NAMES a case the pattern omits -- the shape that produced two of the four
- silent on master today, and the output says so: "Zero is a result here and not a silence: 57 arms were read"
- historical control: run against the commit before #2871 it finds exactly the `StmtIf` arm, which is the defect it was written after
- `assignment` unquoted matched prose and gave two false hits; requiring the backtick took them to zero without touching the control
