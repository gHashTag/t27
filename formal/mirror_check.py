#!/usr/bin/env python3
"""Fail if the abstract composition stops mirroring the concrete circuit.

Wave 636. Prop. 92 proves T5 from lemma F by replacing the full adder with an
abstraction constrained only by F and chaining three of them exactly as
`trit3_add` chains the real ones. That proof is worth exactly as much as the
word "exactly".

The abstraction DUPLICATES the wiring rather than sharing it -- there is no way
in this flow to instantiate the real module's structure with a different leaf.
So a future edit to `trit3_add` (reordering the trit slices, changing which
carry feeds which stage, passing a different first `cin`) leaves `add3_abstract`
behind, and the composition proof keeps passing while describing a circuit that
no longer exists. Nothing else in the suite would notice: both modules would
still prove their own assertions.

This compares the two instantiations structurally. For each stage it extracts
the port connections of the concrete `trit_full_adder` and the abstract
`fv_abstract_fa` and requires them to agree, modulo the instance type and the
concrete module's `sum`/`cout` naming.

It is a small check for a narrow claim, and it is here because the claim is
load-bearing: without it, "chained exactly as trit3_add chains them" is a
sentence in a comment rather than a fact about the tree.

Reads build/rtl/trit_stdlib.sv and formal/trit_algebra_props.sv. Writes
nothing in the repository: its self-test and its run() helper build temporary
trees under a tempdir and mutate only those.

Usage:  python3 formal/mirror_check.py [--self-test]
"""

import pathlib
import re
import sys

INST = re.compile(r"^\s*(\w+)\s+(\w+)\s*\((.*?)\);", re.M | re.S)
CONN = re.compile(r"\.(\w+)\s*\(\s*([^)]*?)\s*\)")
# Wave 636b: resolve localparams before comparing. The first version of this
# gate compared connection TEXT, and `TRIT_Z` is declared SEPARATELY in each of
# the two files -- build/rtl/trit_stdlib.sv for the concrete tree and
# formal/trit_algebra_props.sv for the abstraction. Two independent declarations
# sharing a name compare equal as strings no matter what they hold, so setting
# the concrete tree's TRIT_Z to 2'b10 while the abstraction kept 2'b01 left the
# two circuits genuinely different and this gate reported "0 disagreements".
# That is the campaign's own "read the declaration, not the use" rule (Wave 632),
# violated by the gate written to enforce a mirror.
PARAM = re.compile(r"^\s*localparam\s*(?:\[[^\]]*\]\s*)?(\w+)\s*=\s*([^;]+);", re.M)


def body(text, module):
    m = re.search(rf"^module\s+{module}\b.*?^endmodule", text, re.M | re.S)
    return m.group(0) if m else None


def params(text, module):
    """localparam name -> literal, from the module and the enclosing file."""
    b = body(text, module) or ""
    out = {}
    for src in (text, b):                    # module scope wins over file scope
        for name, val in PARAM.findall(re.sub(r"//[^\n]*", "", src)):
            out[name] = re.sub(r"\s+", "", val)
    # Resolve TRANSITIVELY. Wave 642: this resolved exactly one level, so
    # `localparam TRIT_Z = ZERO;` left the string "ZERO" where a value was
    # meant -- a name standing in for a value, the same shape the resolution
    # was added to fix one wave earlier. Fixed point, with a bound so a cyclic
    # definition terminates instead of hanging.
    for _ in range(8):
        changed = False
        for k, v in list(out.items()):
            if v in out and out[v] != v:
                out[k] = out[v]
                changed = True
        if not changed:
            break
    return out


def stages(text, module, leaf):
    """Ordered list of {port: net} for each `leaf` instance inside `module`.

    Nets that name a localparam are replaced by its VALUE, so two files that
    both write `.cin(TRIT_Z)` while defining TRIT_Z differently no longer look
    identical.
    """
    b = body(text, module)
    if b is None:
        return None
    consts = params(text, module)
    b = re.sub(r"//[^\n]*", "", b)
    out = []
    for m in INST.finditer(b):
        if m.group(1) != leaf:
            continue
        conns = {}
        for p, n in CONN.findall(m.group(3)):
            n = re.sub(r"\s+", "", n)
            conns[p] = f"{consts[n]}" if n in consts else n
        out.append((m.group(2), conns))
    return out


def check(root):
    rtl = (root / "build" / "rtl" / "trit_stdlib.sv")
    props = (root / "formal" / "trit_algebra_props.sv")
    if not rtl.exists() or not props.exists():
        print(f"::error::mirror_check needs {rtl} and {props} -- emit the "
              "bundle before running this gate")
        return 1

    concrete = stages(rtl.read_text(), "trit3_add", "trit_full_adder")
    abstract = stages(props.read_text(), "add3_abstract", "fv_abstract_fa")

    if not concrete or not abstract:
        print(f"::error::mirror_check parsed {len(concrete or [])} concrete and "
              f"{len(abstract or [])} abstract stages -- it compared nothing, so "
              "its silence means nothing. A module or instance type was renamed.")
        return 1

    bad = []
    # A stage with NO named connections compares equal to any other such stage,
    # so the gate would report "0 disagreements" while comparing nothing.
    # Wave 642: positional instantiation -- `trit_full_adder fa0 (a, b, cin,
    # sum, cout);` -- yields zero extracted connections and is perfectly legal
    # Verilog. Shape 2, a decline that was not counted, inside the gate that
    # holds Prop. 92's proof to the real circuit.
    for label, st in (("trit3_add", concrete), ("add3_abstract", abstract)):
        empty = [n for n, c in st if not c]
        if empty:
            bad.append(f"{label}: {len(empty)} stage(s) {empty} have no NAMED "
                       "port connections -- positional instantiation, which this "
                       "gate cannot compare. It would otherwise report agreement "
                       "between two things it never read.")

    if len(concrete) != len(abstract):
        bad.append(f"stage count differs: trit3_add has {len(concrete)}, "
                   f"add3_abstract has {len(abstract)}")

    for i, ((cn, cc), (an, ac)) in enumerate(zip(concrete, abstract)):
        for port in ("a", "b", "cin", "sum", "cout"):
            cv, av = cc.get(port), ac.get(port)
            if cv != av:
                bad.append(f"stage {i} ({cn} vs {an}) port .{port}: "
                           f"trit3_add wires {cv!r}, add3_abstract wires {av!r}")

    print(f"mirror check: {len(concrete)} concrete stages vs {len(abstract)} "
          f"abstract, {len(bad)} disagreements")
    for b in bad:
        print(f"::error::{b} -- the composition proof (Prop. 92) chains the "
              "abstract adder differently from the way trit3_add chains the "
              "real one, so it is proving the composition of a circuit that is "
              "not in the bundle. Both modules will still pass their own "
              "assertions, which is why this is checked separately.")
    return 1 if bad else 0


def self_test():
    import tempfile
    root = pathlib.Path(__file__).resolve().parent.parent
    rtl_src = (root / "build" / "rtl" / "trit_stdlib.sv").read_text()
    props_src = (root / "formal" / "trit_algebra_props.sv").read_text()
    bad = []

    def run(rtl_text, props_text):
        with tempfile.TemporaryDirectory() as td:
            d = pathlib.Path(td)
            (d / "build" / "rtl").mkdir(parents=True)
            (d / "formal").mkdir(parents=True)
            (d / "build" / "rtl" / "trit_stdlib.sv").write_text(rtl_text)
            (d / "formal" / "trit_algebra_props.sv").write_text(props_text)
            return check(d)

    rc = run(rtl_src, props_src)
    print(f"  {'ok  ' if rc == 0 else 'FAIL'} the shipped tree mirrors "
          f"(exit {rc})")
    if rc != 0:
        bad.append("the shipped tree was reported as not mirroring")

    # Rewire the abstraction: feed stage 1 the wrong carry. Semantically this is
    # the exact drift the gate exists to catch.
    drift = props_src.replace(
        "fv_abstract_fa fa1 (.a(a[3:2]), .b(b[3:2]), .cin(c0),",
        "fv_abstract_fa fa1 (.a(a[3:2]), .b(b[3:2]), .cin(c1),")
    print(f"  {'ok  ' if drift != props_src else 'FAIL'} the drift injection "
          "landed")
    if drift == props_src:
        bad.append("the injection changed nothing, so the next case is vacuous")
    else:
        rc = run(rtl_src, drift)
        print(f"  {'ok  ' if rc else 'FAIL'} a rewired abstraction is caught "
              f"(exit {rc})")
        if rc == 0:
            bad.append("a rewired abstraction passed as mirroring")

    # A renamed module must not read as a clean pass.
    rc = run(rtl_src.replace("module trit3_add", "module trit3_add_renamed"),
             props_src)
    print(f"  {'ok  ' if rc else 'FAIL'} a renamed concrete module fails rather "
          f"than passing silently (exit {rc})")
    if rc == 0:
        bad.append("a missing concrete module was reported as mirroring")

    # The case this gate got WRONG on its first version, kept as a permanent
    # regression test: the same identifier holding different values on the two
    # sides. Wave 636b.
    # Wave 642 regressions.
    pos = rtl_src.replace(
        "trit_full_adder fa0 (.a(a[1:0]), .b(b[1:0]), .cin(TRIT_Z), .sum(sum[1:0]), .cout(c0));",
        "trit_full_adder fa0 (a[1:0], b[1:0], TRIT_Z, sum[1:0], c0);")
    if pos == rtl_src:
        bad.append("the positional-instantiation injection changed nothing")
    else:
        rc = run(pos, props_src)
        print(f"  {'ok  ' if rc else 'FAIL'} a positionally-instantiated stage "
              f"is caught, not silently compared as empty (exit {rc})")
        if rc == 0:
            bad.append("a stage with no named connections compared as agreeing "
                       "-- the gate read nothing and reported agreement")

    # Transitive localparam resolution, checked directly on the resolver rather
    # than through a full injection: `TRIT_Z = ZZ` must resolve to ZZ's VALUE,
    # not to the string "ZZ". Wave 642 -- the resolver added one wave earlier
    # went exactly one level deep, leaving a name where a value was meant.
    chained = ("module m;\n"
               "    localparam [1:0] ZZ = 2'b10;\n"
               "    localparam [1:0] TRIT_Z = ZZ;\n"
               "endmodule\n")
    got = params(chained, "m").get("TRIT_Z")
    ok = got == "2'b10"
    print(f"  {'ok  ' if ok else 'FAIL'} a localparam defined via another "
          f"resolves to its VALUE: TRIT_Z -> {got!r} (want \"2'b10\")")
    if not ok:
        bad.append(f"a chained localparam resolved to {got!r}, not its value")

    shifted = rtl_src.replace("localparam [1:0] TRIT_Z = 2'b01;",
                              "localparam [1:0] TRIT_Z = 2'b10;")
    print(f"  {'ok  ' if shifted != rtl_src else 'FAIL'} the constant-shift "
          "injection landed")
    if shifted == rtl_src:
        bad.append("the constant-shift injection changed nothing")
    else:
        rc = run(shifted, props_src)
        print(f"  {'ok  ' if rc else 'FAIL'} the same name holding a different "
              f"value on each side is caught (exit {rc})")
        if rc == 0:
            bad.append("two circuits differing only in a shared constant's "
                       "VALUE were reported as mirroring -- the gate is "
                       "comparing uses, not declarations")

    for b in bad:
        print(f"::error::mirror_check self-test: {b}")
    return 1 if bad else 0


if __name__ == "__main__":
    r = pathlib.Path(__file__).resolve().parent.parent
    sys.exit(self_test() if "--self-test" in sys.argv else check(r))
