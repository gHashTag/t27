#!/usr/bin/env python3
"""Permute the trit encoding and check that exactly the right theorems break.

Wave 635. Prop. 86 tested one claim by breaking it: `trit_compare` is correct
only because the two-bit encoding is monotone in trit value, so permuting the
encoding should refute that theorem and no other. It refuted a second --
`trit_full_adder` had the encoding baked in as literals where every sibling,
including its own sub-instances, routed through the named constants. A
renumbering would have moved the siblings and left it behind, silently.

That experiment found a real defect on its first run, so it becomes a gate.

The check is two-sided, and both sides matter:

  NO NEW BREAKS      a theorem that survives today must survive the permutation.
                     A new failure means some primitive has acquired a hidden
                     dependency on the literal encoding -- the Wave 634 defect,
                     recurring.
  NO LOST BREAKS     `cmp_props` must STILL refute. It is encoding-dependent by
                     design, and if it stops being so, either the comparison was
                     rewritten (fine, but the exemption is now stale) or the
                     permutation stopped permuting (not fine, and invisible:
                     the gate would pass by testing nothing).

The second is the one that would rot quietly. A gate that asserts only "nothing
broke" passes when its own perturbation has become a no-op, which is this
campaign's oldest failure shape -- an absence read as a pass (Props. 58-60).

The permutation swaps the codes for -1 and 0, so the encoding order 00 < 01 < 10
no longer agrees with the value order. It is applied to BOTH the RTL localparams
and the property file's value macro, because permuting only one would break
every theorem trivially and prove nothing about any of them.

Usage:  python3 formal/encoding_gate.py [--self-test]
"""

import pathlib
import re
import subprocess
import sys
import tempfile

# Theorem -> must it refute under the permutation?
EXPECTED = {
    "not_props": False,
    "and_props": False,
    "or_props": False,
    "lattice_props": False,
    "mul_props": False,
    "half_adder_props": False,
    "full_adder_props": False,
    "add3_props": False,
    # Encoding-dependent BY DESIGN: trit_compare compares raw codes with `<`.
    # Prop. 86b. If this ever stops refuting, this table is what must change.
    "cmp_props": True,
}

PERM_RTL = [
    (r"localparam \[1:0\] TRIT_N = 2'b00;", "localparam [1:0] TRIT_N = 2'b01;"),
    (r"localparam \[1:0\] TRIT_Z = 2'b01;", "localparam [1:0] TRIT_Z = 2'b00;"),
]
PERM_TV = (r"`define TV\(t\) \(\(\(t\) == 2'b00\) \? -7'sd1 : \(\(t\) == 2'b10\) \? 7'sd1 : 7'sd0\)",
           "`define TV(t) (((t) == 2'b01) ? -7'sd1 : ((t) == 2'b10) ? 7'sd1 : 7'sd0)")


def permute(rtl_text, props_text):
    """Apply the permutation to both sides, and report how much it changed."""
    r, n_rtl = rtl_text, 0
    for pat, rep in PERM_RTL:
        r, k = re.subn(pat, rep, r)
        n_rtl += k
    p, n_tv = re.subn(PERM_TV[0], PERM_TV[1], props_text)
    return r, p, n_rtl, n_tv


def prove(rtl, props, top):
    """True if the theorem proves; False if it refutes. Raises on tool error."""
    with tempfile.TemporaryDirectory() as td:
        d = pathlib.Path(td)
        (d / "rtl.sv").write_text(rtl)
        (d / "props.sv").write_text(props)
        r = subprocess.run(
            ["yosys", "-q", "-p",
             f"read_verilog -sv -formal {d}/rtl.sv {d}/props.sv; "
             f"prep -top {top} -flatten; async2sync; chformal -lower; "
             "sat -verify -prove-asserts -seq 1 -set-init-zero -set-assumes"],
            capture_output=True, text=True)
        err = (r.stderr or "") + (r.stdout or "")
        # A tool error is not a refutation. Prop. 39d, which this campaign has
        # now folded into a verdict three separate times.
        if re.search(r"ERROR: (?!Called with -verify)", err):
            raise RuntimeError(f"{top}: yosys error, not a verdict:\n"
                               f"{err.strip().splitlines()[-1:]}")
        return r.returncode == 0


def run(root):
    rtl_p = root / "build" / "rtl" / "trit_stdlib.sv"
    props_p = root / "formal" / "trit_algebra_props.sv"
    if not rtl_p.exists() or not props_p.exists():
        print(f"::error::encoding_gate needs {rtl_p} and {props_p} -- emit the "
              "bundle before running this gate")
        return 1

    rtl, props, n_rtl, n_tv = permute(rtl_p.read_text(), props_p.read_text())
    print(f"permutation applied: {n_rtl} localparam sites, {n_tv} value macro")
    # A permutation that changed nothing tests nothing, and would report a
    # clean sweep of "no new breaks".
    if n_rtl == 0 or n_tv == 0:
        print("::error::the permutation matched nothing -- the encoding "
              "constants or the value macro were renamed, so this gate is "
              "perturbing an unchanged design and its silence means nothing.")
        return 1

    bad = []
    for top, must_refute in sorted(EXPECTED.items()):
        try:
            proved = prove(rtl, props, top)
        except RuntimeError as e:
            print(f"::error::{e}")
            bad.append(top)
            continue
        refuted = not proved
        ok = refuted == must_refute
        verdict = "refutes" if refuted else "proves "
        print(f"  {'ok  ' if ok else 'FAIL'} {top:20s} {verdict}  "
              f"(expected {'refutes' if must_refute else 'proves'})")
        if not ok:
            bad.append(top)
            if refuted:
                print(f"::error::{top} broke under a permuted encoding but is "
                      "supposed to be encoding-independent. Some primitive it "
                      "covers has the encoding written in as a literal instead "
                      "of going through TRIT_N/TRIT_Z/TRIT_P -- the Wave 634 "
                      "defect in trit_full_adder, recurring. See Prop. 86d.")
            else:
                print(f"::error::{top} SURVIVED the permutation but is listed "
                      "as encoding-dependent. Either it was rewritten to be "
                      "encoding-independent, in which case this table is stale, "
                      "or the permutation is no longer perturbing what it "
                      "thinks it is -- and a gate whose perturbation has become "
                      "a no-op passes by testing nothing.")

    print(f"\nencoding gate: {len(EXPECTED)} theorems permuted, {len(bad)} "
          "disagreeing with the expected split")
    return 1 if bad else 0


def self_test():
    """Both directions of the check must fire."""
    root = pathlib.Path(__file__).resolve().parent.parent
    rtl_p = root / "build" / "rtl" / "trit_stdlib.sv"
    props_p = root / "formal" / "trit_algebra_props.sv"
    bad = []

    rtl, props, n_rtl, n_tv = permute(rtl_p.read_text(), props_p.read_text())
    print(f"  {'ok  ' if n_rtl and n_tv else 'FAIL'} the permutation matches "
          f"the tree ({n_rtl} localparams, {n_tv} macro)")
    if not (n_rtl and n_tv):
        bad.append("the permutation no longer matches the source")

    # A primitive that hardcodes the encoding must be caught. Re-inject the
    # exact Wave 634 defect: trit_full_adder comparing against literals.
    hard = rtl.replace("(carry1 == TRIT_P) ? 3'sd1 :", "(carry1 == 2'b10) ? 3'sd1 :") \
              .replace("(carry1 == TRIT_N) ? -3'sd1 : 3'sd0;", "(carry1 == 2'b00) ? -3'sd1 : 3'sd0;")
    injected = hard != rtl
    print(f"  {'ok  ' if injected else 'FAIL'} the Wave 634 defect can be "
          "re-injected")
    if not injected:
        bad.append("the defect injection changed nothing, so the next case is vacuous")
    else:
        caught = not prove(hard, props, "full_adder_props")
        print(f"  {'ok  ' if caught else 'FAIL'} a primitive hardcoding the "
              f"encoding is caught ({'refutes' if caught else 'proves'})")
        if not caught:
            bad.append("a hardcoded encoding survived the permutation")

    # And the lost-break direction: cmp_props must refute under permutation.
    lost = prove(rtl, props, "cmp_props")
    print(f"  {'ok  ' if not lost else 'FAIL'} the encoding-dependent theorem "
          f"still refutes ({'proves' if lost else 'refutes'})")
    if lost:
        bad.append("cmp_props no longer refutes -- the perturbation may be a no-op")

    for b in bad:
        print(f"::error::encoding_gate self-test: {b}")
    return 1 if bad else 0


if __name__ == "__main__":
    r = pathlib.Path(__file__).resolve().parent.parent
    sys.exit(self_test() if "--self-test" in sys.argv else run(r))
