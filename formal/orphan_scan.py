#!/usr/bin/env python3
"""Fail if a property file is not run by any CI step.

Wave 618. Wave 617 found eight properties that README counted as proved while no
job in the repository ran them: `zero_size_props.sv` appeared exactly once in all
of `.github/`, inside the *weekly* mutation harness, and two of its four wrappers
appeared nowhere at all. Nothing was broken -- all eight held -- which is exactly
why it sat unnoticed. **An ungated property that happens to hold is
indistinguishable from a gated one.**

That was found by accident, from one line of a bound audit that I nearly wrote
off as my own bug. This is the systematic version, and it is cheap: counting
properties tells you nothing, so count the STEPS THAT RUN THEM.

Two levels of finding:

  ORPHAN   no workflow references the file at all -- an error
  WEEKLY   referenced only by a schedule-triggered workflow, so a defect in it
           is invisible on a pull request. Reported, not failed: weekly is a
           deliberate choice for expensive harnesses. Silence is what is not
           allowed.

Usage:  python3 formal/orphan_scan.py [--self-test]
"""

import pathlib
import re
import sys

import yaml


def runnable_text(doc):
    """Concatenated `run:` scalars of every step that can actually execute.

    Wave 640. This gate asked "does this filename appear anywhere in the
    workflow file", and called the answer "referenced by a workflow". Those are
    different questions, and an adversarial audit demonstrated the gap four
    ways -- a filename in a `#` COMMENT, in a step carrying `if: false`, in a
    step that only `grep`s it, and in a workflow triggered `on: [release]` --
    every one of which the gate credited as run.

    The live hazard was not hypothetical. `formal-yosys.yml` already carries two
    retrospective comments naming `zero_size_props.sv`, the very file whose
    ungated properties Prop. 69 was about. Deleting its two EXECUTABLE
    references left the gate printing a byte-identical healthy summary: the
    comments narrating Wave 617's defect would have concealed its recurrence.

    So the text searched is now the `run:` bodies of reachable steps only, with
    `#` comments stripped from them. A step whose `if:` is literally false is
    skipped, as is a workflow that no push or pull_request can trigger.
    """
    out = []
    for job in (doc.get("jobs") or {}).values():
        if not isinstance(job, dict):
            continue
        if str(job.get("if", "")).strip().lower() in ("false", "${{ false }}"):
            continue
        for step in job.get("steps") or []:
            if not isinstance(step, dict):
                continue
            if str(step.get("if", "")).strip().lower() in ("false", "${{ false }}"):
                continue
            body = step.get("run")
            if not isinstance(body, str):
                continue
            body = re.sub(r"#[^\n]*", "", body)
            # A file must be an ARGUMENT TO A PROVER, not merely mentioned in a
            # runnable step. Wave 640: the first fix here searched every `run:`
            # body, which still credited a step whose only use of the file was
            # `grep -c assert formal/ghost_props.sv` -- a step that runs, reads
            # the file, and proves nothing. So the step's body must also invoke
            # something that could prove or load it. `open(` is included
            # because several steps hand a property file to a python harness
            # that mutates it before calling yosys.
            if not re.search(r"\byosys\b|read_verilog|\bopen\s*\(", body):
                continue
            out.append(body)
    return "\n".join(out)


def workflows(root):
    """[(name, is_scheduled, runnable_text)] for every workflow."""
    out = []
    for p in sorted((root / ".github" / "workflows").glob("*.yml")):
        text = p.read_text()
        try:
            doc = yaml.safe_load(text) or {}
        except yaml.YAMLError:
            # An unparseable workflow contributes NO runnable text. It must not
            # fall back to the raw file, which is what made a comment count.
            doc = {}
        # `on:` parses as the boolean True in YAML 1.1 -- check both spellings.
        trig = doc.get("on", doc.get(True, {})) or {}
        scheduled = isinstance(trig, dict) and set(trig) <= {"schedule", "workflow_dispatch"}
        out.append((p.name, scheduled, runnable_text(doc)))
    return out


# Emitted modules that legitimately have no properties, with the reason each is
# exempt. Prop. 59d's rule: an exemption added without argument is how a sweep
# comes to pass while checking less than it claims.
MODULE_EXEMPT = {
    "behavior_sva_v2":
        "concurrent SVA (`##N`, `s_eventually`) which this flow cannot check at "
        "all -- Props. 2/5/6. It is the artifact whose uncheckability is the "
        "reason gen-behavior-sva-yosys exists, so proving it is not merely "
        "undone but impossible here.",
}


def modules(root):
    """Every emitted MODULE, classified by where its properties live.

    Per module, not per file: `trit_stdlib.sv` defines eleven ternary
    primitives, so a file-stem classifier reports one "module" that does not
    exist and misses eleven that do. Prop. 76.

      DIRECT     a formal/ suite instantiates it, or it carries inline assertions
      INDIRECT   no properties of its own, but reachable from bitnet_engine_top
                 through instantiation, so the integration properties constrain
                 it at one remove
      UNREACHED  no properties and instantiated by nothing in the bundle
      EXEMPT     listed above with a reason
    """
    rtl = sorted((root / "build" / "rtl").glob("*.sv"))
    if not rtl:
        return None
    srcs = {p.name: p.read_text() for p in rtl}
    formal = [(p.name, p.read_text()) for p in sorted((root / "formal").glob("*.sv"))]

    defined = {}                       # module name -> file
    for name, text in srcs.items():
        for m in re.findall(r"^module (\w+)", text, re.M):
            defined[m] = name

    def instantiates(text, mod):
        return bool(re.search(rf"^\s*{mod}\s+\w+\s*\(", text, re.M))

    # Reachability from the engine, transitively.
    reach, frontier = set(), ["bitnet_engine_top"]
    while frontier:
        cur = frontier.pop()
        if cur in reach or cur not in defined:
            continue
        reach.add(cur)
        text = srcs[defined[cur]]
        for cand in defined:
            if cand != cur and instantiates(text, cand):
                frontier.append(cand)

    out = []
    for mod, fname in sorted(defined.items()):
        text = srcs[fname]
        i = text.index(f"module {mod}")
        j = text.find("\nendmodule", i)
        body = text[i:j if j > 0 else len(text)]
        # Comments stripped first. Wave 639b: this counted `a_x: assert`
        # inside `//` comments, so a comment DISCUSSING an assertion made a
        # module look DIRECT. The sibling gate claims_check had the identical
        # defect and was fixed one wave earlier (Prop. 95); nobody checked
        # whether the same regex elsewhere had the same problem. It did.
        inline = len(re.findall(r"\ba_[a-z0-9_]+\s*:\s*assert",
                                re.sub(r"//[^\n]*", "", body)))
        # A suite counts as covering a module only if it instantiates it as the
        # module UNDER TEST -- by convention the instance named `dut`. Wave 627
        # added a wrapper that instantiates `trit27_dot_product` a second time
        # as a SHADOW, to compute an expected value; without this distinction
        # that primitive would be reported DIRECT while having no property about
        # it at all. An auxiliary instance is not coverage.
        suites = [n for n, t in formal
                  if re.search(rf"^\s*{mod}\s+dut\s*\(", t, re.M)]
        if mod in MODULE_EXEMPT:
            kind = "EXEMPT"
        elif inline or suites:
            kind = "DIRECT"
        elif mod in reach:
            kind = "INDIRECT"
        else:
            kind = "UNREACHED"
        out.append((mod, fname, inline, suites, kind))
    return out


def scan(root):
    wfs = workflows(root)
    files = sorted((root / "formal").glob("*.sv"))
    if not files:
        print(f"::error::orphan_scan found no property files under {root}/formal")
        return 1
    if not wfs:
        print(f"::error::orphan_scan found no workflows under {root}/.github/workflows")
        return 1

    bad, weekly = [], []
    print(f"{'property file':34s} {'workflows referencing it':>26s}")
    print("-" * 64)
    for f in files:
        # A filename is a NAME, not an arbitrary substring. Wave 640: `f.name
        # in text` credited a hypothetical `formal/props.sv` to eight unrelated
        # suites, because every one of them ends in `_props.sv`. The convention
        # in formal/ is `<thing>_props.sv`, so containment pairs are one new
        # file away.
        # The lookbehind must ALLOW `/` -- references are written
        # `formal/<name>.sv`. A first attempt excluded it and reported all 15
        # files orphaned at once, which is the right way for a gate to fail.
        pat = re.compile(r"(?<![\w.-])" + re.escape(f.name) + r"(?![\w.])")
        hits = [(n, sched) for n, sched, text in wfs if pat.search(text)]
        names = ", ".join(n.replace(".yml", "") for n, _ in hits) or "-- NONE --"
        print(f"{f.name:34s} {names:>26s}")
        if not hits:
            bad.append(f.name)
        elif all(sched for _, sched in hits):
            weekly.append(f.name)

    for b in bad:
        print(f"::error::formal/{b} is referenced by no workflow -- its properties "
              "are counted but never run, and an ungated property that happens to "
              "hold looks exactly like a gated one")
    for w in weekly:
        print(f"::warning::formal/{w} is referenced only by schedule-triggered "
              "workflows, so a defect in it is invisible on a pull request")
    mods = modules(root)
    if mods is None:
        print("\n(no emitted RTL -- module coverage not checked)")
    else:
        print(f"\n{'emitted module':26s} {'file':26s} {'inline':>7s} {'coverage':>10s}")
        print("-" * 74)
        for mod, fname, inline, _s, kind in mods:
            print(f"{mod:26s} {fname:26s} {inline:7d} {kind:>10s}")
        kinds = {k: [m for m, _f, _i, _s, kk in mods if kk == k] for k in
                 ("DIRECT", "INDIRECT", "UNREACHED", "EXEMPT")}
        # Reported, not failed. An unexercised library module is not a defect,
        # and a permanently red gate is one everyone learns to ignore -- the
        # workflow's own comment on the scale ceiling says exactly that. What is
        # not allowed is silence.
        for m in kinds["UNREACHED"]:
            print(f"::warning::module {m} has no properties and is instantiated "
                  "by nothing in the emitted bundle -- it is read into every "
                  "proof as source and constrained by none of them")
        for m in kinds["INDIRECT"]:
            print(f"::warning::module {m} has no properties of its own; the "
                  "engine's integration properties constrain it only at one "
                  "remove, so a defect in it is caught only if it reaches an "
                  "engine-level observable")
        print(f"module coverage: {len(mods)} modules -- "
              f"{len(kinds['DIRECT'])} direct, {len(kinds['INDIRECT'])} indirect, "
              f"{len(kinds['UNREACHED'])} unreached, {len(kinds['EXEMPT'])} exempt")

    print(f"\norphan scan: {len(files)} property files, {len(wfs)} workflows, "
          f"{len(bad)} orphaned, {len(weekly)} weekly-only")
    return 1 if bad else 0


def self_test():
    """The gate must catch a file nothing runs, and must not cry wolf."""
    import shutil
    import tempfile
    root = pathlib.Path(__file__).resolve().parent.parent
    with tempfile.TemporaryDirectory() as td:
        td = pathlib.Path(td)
        shutil.copytree(root / "formal", td / "formal")
        shutil.copytree(root / ".github" / "workflows", td / ".github" / "workflows")
        cases = [("the real tree", lambda: None, 0),
                 ("a property file nothing references",
                  lambda: (td / "formal" / "orphaned_props.sv").write_text(
                      "module x_props (input wire clk);\n"
                      "    always @(posedge clk) a_x: assert (1'b1);\n"
                      "endmodule\n"), 1)]

        # Wave 640 regressions. An adversarial audit showed four ways a file
        # could be credited as "run" while nothing proved it, and one of them
        # was live: formal-yosys.yml already carries retrospective comments
        # naming zero_size_props.sv, so deleting its executable references left
        # the summary byte-identical to a healthy tree.
        GHOST = ("module ghost_props (input wire clk);\n"
                 "    always @(posedge clk) a_ghost: assert (1'b0);\n"
                 "endmodule\n")

        def only(step_yaml):
            def go():
                (td / "formal" / "orphaned_props.sv").unlink(missing_ok=True)
                (td / "formal" / "ghost_props.sv").write_text(GHOST)
                (td / ".github" / "workflows" / "zz.yml").write_text(
                    "name: zz\non: [push]\njobs:\n  j:\n"
                    "    runs-on: ubuntu-latest\n    steps:\n" + step_yaml)
            return go

        cases += [
            ("a file named only in a # comment",
             only("      # someday prove formal/ghost_props.sv\n"
                  "      - name: unrelated\n        run: echo hi\n"), 1),
            ("a file named only in a step with if: false",
             only("      - name: off\n        if: false\n"
                  "        run: yosys -p 'read_verilog formal/ghost_props.sv'\n"), 1),
            ("a file only grepped, never proved",
             only("      - name: greps\n"
                  "        run: grep -c assert formal/ghost_props.sv\n"), 1),
            ("a file named only inside a run: block's own comment",
             only("      - name: commented out\n        run: |\n"
                  "          # yosys -p 'read_verilog formal/ghost_props.sv'\n"
                  "          echo hi\n"), 1),
        ]
        bad = []
        for name, setup, want in cases:
            setup()
            got = scan(td)
            print(f"  {'ok  ' if got == want else 'FAIL'} {name}  "
                  f"(exit {got}, want {want})\n")
            if got != want:
                bad.append(name)
        # And it must fail loudly rather than pass on an empty tree.
        shutil.rmtree(td / "formal")
        (td / "formal").mkdir()
        got = scan(td)
        print(f"  {'ok  ' if got == 1 else 'FAIL'} no property files at all "
              f"(exit {got}, want 1)")
        if got != 1:
            bad.append("empty tree")
    for b in bad:
        print(f"::error::orphan_scan self-test: '{b}' gave the wrong answer")
    return 1 if bad else 0


if __name__ == "__main__":
    r = pathlib.Path(__file__).resolve().parent.parent
    sys.exit(self_test() if "--self-test" in sys.argv else scan(r))
