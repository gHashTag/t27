# NOW — `sys.exit` is a verdict (2026-08-23)

Giving `verify_multitarget.py` its first negative control turned into a finding about the operators, and then into a survivor inside a gate reported clean for the whole campaign.

- **The control**: the skip pair is this gate's most rottable branch — without `--require` a missing prerequisite is a SKIP and exit 0, with it the same state is a FAILURE. Three lines apart, same message, only the exit code differs, so each case names the other's marker as forbidden.
- **The plant was wrong on the first attempt, and said so.** I ran the gate from a temp working directory expecting the binary to be missing; it resolves against `ROOT`, not cwd, so the gate found the real one and ran the whole check. The control reported CONTROL FAILED rather than passing — which is what a control is for.
- **Then two columns of zeros.** `silent 0/0, loud 0/0` for a file that is nothing but verdicts. The operators understood `return N` and `raise SystemExit(N)` and **not `sys.exit(N)`**, the spelling half this repository uses. Two empty columns read as *nothing here to break* — the same sentence a clean gate prints.
- **The fix immediately found a survivor in a gate scored 3/3 for weeks.** `check_catalog_count.py` has a `sys.exit(2)` for the codegen subprocess *itself failing*, and every case in its control plants SSOT **content** — so the codegen always ran and always succeeded. Closed with a planted codegen that dies.

**The rule:** an operator that recognises one spelling of a verdict measures that spelling. When a column reads `0/0`, ask whether the gate has no such site or the scanner has no such pattern — those print identically and mean opposite things.

Left open and named: the cross-target MISMATCH verdict, which needs a fake compiler emitting deliberately wrong C and Rust. The tool's survivors match that written declaration exactly — the first time this campaign a declared gap and a measured one agreed line for line.
