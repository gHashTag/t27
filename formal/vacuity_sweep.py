#!/usr/bin/env python3
"""Retroactive vacuity audit of every proof step in the formal workflows.

`vacuity_gate.py` checks the engine suite as it stands today. Every other proof
in this campaign -- 134 propositions' worth -- was run before that check existed,
and twelve of the fifteen property wrappers use `assume`. Prop. 133 showed what
that risks: an unsatisfiable assumption set makes yosys report "proof succeeded"
for every property in the run, including `assert (1'b0)`, with exit code 0 and no
diagnostic.

So this asks the question once for every step: parse each `sat -verify
-prove-asserts` invocation out of the workflows, re-run it verbatim with
`assert (1'b0)` injected into its `-top` module, and require a REFUTATION.
A step whose false assertion proves has been reporting vacuous passes for
however long it has existed.

The probe is injected as a separate always block appended inside the top
module, guarded by `T27_VACUITY_PROBE` -- so a run without the define is the
untouched original, which is what makes the pair a control rather than a claim.

ARTIFACTS. Reads `.github/workflows/*.yml` and the RTL and property files
those steps name (`build/rtl/*.sv`, `formal/*.sv`). Writes nothing outside a
temporary directory: every probed source is a COPY, and `.github`, `build/rtl`
and `formal/` are left exactly as found.

Prop. 135."""
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
WORKFLOWS = [
    ROOT / ".github" / "workflows" / "formal-yosys.yml",
    ROOT / ".github" / "workflows" / "fpga-formal.yml",
]

PROBE = """
`ifdef T27_VACUITY_PROBE
    always @(posedge {clk}) a_vacuity_probe: assert (1'b0);
`endif
"""


def steps(wf_text):
    """Every yosys invocation that proves assertions, with its step name."""
    out = []
    name = None
    for block in re.split(r"\n      - name: ", wf_text):
        first = block.split("\n", 1)[0].strip()
        if not first:
            continue
        name = first
        # Join the continuation lines of a single shell command.
        joined = re.sub(r"\\\s*\n\s*", " ", block)
        for m in re.finditer(r'yosys\s+-p\s+"([^"]+)"', joined):
            script = re.sub(r"\s+", " ", m.group(1)).strip()
            if "-prove-asserts" not in script:
                continue
            top = re.search(r"-top\s+(\S+)", script)
            files = re.findall(r"(\S+\.s?v)\b", script)
            if not top or not files:
                continue
            out.append((name, script, top.group(1), files))
    return out


def strip_comments(t):
    """Verilog comments are not Verilog.

    A `// always @(posedge fake_clk)` in a header banner would send the probe to
    a clock that does not exist, and a commented-out `module` line would send it
    to the wrong file. Five fixes across four files have gone to this one shape
    (Props. 95, 102c, 118), so it is stripped here before any regex runs.
    """
    t = re.sub(r"/\*.*?\*/", "", t, flags=re.S)
    return re.sub(r"//[^\n]*", "", t)


def clock_of(text):
    """The clock this module is posedge-triggered on, read from the module."""
    m = re.search(r"always\s*@\s*\(\s*posedge\s+(\w+)", strip_comments(text))
    return m.group(1) if m else None


def main():
    if not shutil.which("yosys"):
        print("::error::vacuity sweep: yosys not found on PATH -- "
              "no step was audited")
        return 1

    found = []
    for wf in WORKFLOWS:
        if wf.exists():
            found.extend(steps(wf.read_text()))
    if not found:
        print("::error::vacuity sweep: found no proof steps in "
              ".github/workflows -- nothing was audited")
        return 1

    vacuous, skipped, live = [], [], 0
    for name, script, top, files in found:
        srcs = [ROOT / f for f in files]
        if any(not s.exists() for s in srcs):
            missing = next(s for s in srcs if not s.exists())
            skipped.append(f"{name[:44]}: no such file '{missing.name}'")
            continue

        with tempfile.TemporaryDirectory() as td:
            work = pathlib.Path(td)
            # The workflow writes RELATIVE paths ('build/rtl/x.sv'); keying the
            # substitution on absolute ones silently left the script pointing at
            # the UNPROBED originals, so every step "proved" and the sweep read
            # that as vacuity. Twelve false positives, including a step
            # vacuity_gate.py had just measured live. Key on the literal text
            # the script contains. Prop. 135.
            local = {}
            for s, lit in zip(srcs, files):
                shutil.copy(s, work / s.name)
                local[lit] = str(work / s.name)

            # Inject the probe at the end of the -top module.
            target = None
            for s in srcs:
                t = (work / s.name).read_text()
                if re.search(rf"\bmodule\s+{re.escape(top)}\b", strip_comments(t)):
                    target = work / s.name
                    break
            if target is None:
                skipped.append(f"{name[:44]}: -top {top} in none of its files")
                continue

            text = target.read_text()
            clk = clock_of(text)
            if clk is None:
                skipped.append(f"{name[:44]}: {top} has no posedge clock; "
                               f"a combinational step cannot be probed this way")
                continue
            # `endmodule` closing the target module, not a later one.
            # Detection uses stripped text; the INSERTION OFFSET must come from the
            # original, because stripping shifts every index after the first
            # comment. Mixing the two put the probe outside the module and the
            # sweep reported a false vacuous step one command after the strip
            # was added. Prop. 136.
            mstart = re.search(rf"\bmodule\s+{re.escape(top)}\b", text).start()
            mend = text.find("endmodule", mstart)
            if mend < 0:
                skipped.append(f"{name[:44]}: no endmodule for {top}")
                continue
            probed = (text[:mend] + PROBE.format(clk=clk) + text[mend:])
            target.write_text(probed)

            probed_script = script
            for orig, new in local.items():
                probed_script = probed_script.replace(orig, new)
            probed_script = probed_script.replace(
                "read_verilog -sv -formal", "read_verilog -sv -formal -DT27_VACUITY_PROBE", 1)

            # A probe that does not land tests nothing -- and reports the
            # opposite of the truth, because an unprobed suite proves.
            if str(target) not in probed_script:
                skipped.append(f"{name[:44]}: probe landed in {target.name} but "
                               f"the script still reads the original; not audited")
                continue
            if "-DT27_VACUITY_PROBE" not in probed_script:
                skipped.append(f"{name[:44]}: probe define not applied")
                continue

            r = subprocess.run(["yosys", "-q", "-p", probed_script],
                               capture_output=True, text=True)
            if r.returncode == 0:
                vacuous.append(f"{name[:56]} (-top {top})")
            elif r.returncode == 1:
                live += 1
            else:
                skipped.append(f"{name[:44]}: solver returned {r.returncode}")

    for s in skipped:
        print(f"  skipped  {s}")
    print(f"vacuity sweep: {live} live, {len(vacuous)} vacuous, "
          f"{len(skipped)} not audited, of {len(found)} proof steps")

    if vacuous:
        print(f"::error::vacuity sweep: {len(vacuous)} proof step(s) prove "
              f"assert(1'b0) -- their assumption sets are unsatisfiable and "
              f"every property they report has been passing vacuously")
        for v in vacuous:
            print(f"  {v}")
        return 1
    if live == 0:
        print("::error::vacuity sweep: no step was actually audited -- "
              "a sweep that probes nothing reports success for having "
              "looked at nothing (Prop. 103)")
        return 1
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::vacuity sweep: could not audit .github/workflows "
              f"({type(exc).__name__}: {exc}) -- nothing was audited")
        sys.exit(1)
