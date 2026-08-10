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

Usage:  python3 formal/mirror_check.py [--self-test]
"""

import pathlib
import re
import sys

INST = re.compile(r"^\s*(\w+)\s+(\w+)\s*\((.*?)\);", re.M | re.S)
CONN = re.compile(r"\.(\w+)\s*\(\s*([^)]*?)\s*\)")


def body(text, module):
    m = re.search(rf"^module\s+{module}\b.*?^endmodule", text, re.M | re.S)
    return m.group(0) if m else None


def stages(text, module, leaf):
    """Ordered list of {port: net} for each `leaf` instance inside `module`."""
    b = body(text, module)
    if b is None:
        return None
    b = re.sub(r"//[^\n]*", "", b)
    out = []
    for m in INST.finditer(b):
        if m.group(1) != leaf:
            continue
        conns = {p: re.sub(r"\s+", "", n) for p, n in CONN.findall(m.group(3))}
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

    for b in bad:
        print(f"::error::mirror_check self-test: {b}")
    return 1 if bad else 0


if __name__ == "__main__":
    r = pathlib.Path(__file__).resolve().parent.parent
    sys.exit(self_test() if "--self-test" in sys.argv else check(r))
