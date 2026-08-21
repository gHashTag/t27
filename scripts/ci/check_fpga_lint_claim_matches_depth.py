#!/usr/bin/env python3
"""Configuration test: what `fpga-lint` CLAIMS must name the depth it actually runs.

WHY THIS TEST EXISTS
--------------------
`fpga-lint` runs, can fail, and has caught real regressions. It is not vacuous in
the usual sense. It over-claims in a narrower way: its verdict is green on files
that do not elaborate, and no amount of failing-harder changes that, because the
defective code is never reached.

Yosys elaborates a function body only where the function is CALLED. The generated
Verilog almost never calls its own functions. Measured over the 32 files of the
`fpga-verilog` artifact of run 32464326319: 485 functions defined, 79 with a call
site, 406 with none. `hierarchy -top` validates the module header and the one or
two logic blocks and walks past the rest. On that one artifact:

    read_verilog -sv -DSIMULATION + hierarchy -top   ->  32 pass /  0 fail
    iverilog -g2012 -DSIMULATION                     ->   4 pass / 28 fail

Both re-measured on the artifact of run 32511534193 with the same result. In run
32464326319 itself `fpga-lint` concluded success and `fpga-conformance` concluded
failure against the same upload.

The depth is not the fixable axis. Reachability is. `synth -top` over the same 32
is 31 pass / 1 fail, and the single failure is `zerodsp_top` for a missing
submodule -- an artifact of compiling one file at a time, not a codegen defect.
Promoting the job to full synthesis would flag 1 of the 28 for the wrong reason
and still certify the other 27.

So the job's reach is left alone. What is corrected -- and what this test holds
corrected -- is what it says about itself. See issue #2326.

WHAT IT CHECKS
--------------
It reads the yosys passes the lint step actually runs out of the `-p` script, and
then requires the job's claims to name them:

1. The `fpga-lint` job exists and its lint step still invokes `yosys -p`. If the
   command disappears there is nothing to compare a claim against, and a claim
   gate that silently passes when it can no longer measure is the failure mode
   this file exists to prevent -- so that is a violation, not a skip.
2. The job carries an explicit `name:` (the display name is the only thing most
   readers see in a check list) and that name contains every yosys pass the step
   runs. Deepen the command to `synth` and the label must say `synth`.
3. The text the step writes to `$GITHUB_STEP_SUMMARY` names every yosys pass it
   runs.
4. That summary carries a `**Result:**` line, and that line names every one of
   those passes -- so the headline count is stated at its depth rather than as
   "passed Yosys lint".
5. That summary states its non-coverage: at least one summary line carries a
   negation together with "elaborat", i.e. it says out loud that function bodies
   are not elaborated here.
6. The `fpga-report` row that renders `needs.fpga-lint.result` states the same
   non-coverage. That row is the only fpga-lint status a reader of the run page
   sees, and it read `Lint (all 31 modules)` -- wrong depth and wrong count.

Only lines that are actually redirected to `$GITHUB_STEP_SUMMARY` are examined.
A shell comment inside the `run:` block does not satisfy anything here; neither
does a YAML comment, which never reaches the parsed document at all. A gate that
could be satisfied by a comment would be the same kind of decorative green.

WHAT IT DELIBERATELY DOES NOT CHECK
-----------------------------------
It does not check that the RTL elaborates, and it cannot: that is Icarus-class
work and it is red for 28 modules today. Making `fpga-lint` catch them is a
gate-policy decision for the repository owner, not a wording fix, and this file
does not smuggle one in.

It does not check `fpga-conformance`. That job's own gap -- vectors compiled but
never executed -- is #2241 and is out of scope here.

It does not verify that the summary is TRUE, only that its claim is stated at the
depth the command runs. A sentence can name `hierarchy` and still be nonsense.
This buys one specific thing: the label and the headline cannot drift back to
implying elaboration, and if someone changes the yosys command the claim has to
move with it.

The word it anchors on is "elaborat". That is deliberate and it is tight: a
rewording that drops the word fails, and the author then either restores an
equivalent word or edits this list on purpose. A looser anchor -- "not", say --
would pass on the very text this test was written to reject.

Exit status: 0 clean, 1 violations found, 2 setup error.
Run with --self-test to check that this test can fail.
"""
import os
import re
import sys

try:
    import yaml
except ImportError:
    print("pyyaml is required: pip install pyyaml", file=sys.stderr)
    sys.exit(2)

WORKFLOW = os.path.join(".github", "workflows", "fpga-build.yml")
LINT_JOB = "fpga-lint"
REPORT_JOB = "fpga-report"
SUMMARY_VAR = "GITHUB_STEP_SUMMARY"

# `yosys -p "read_verilog ...; hierarchy -top X"` -- either quote style.
YOSYS_P = re.compile(r"""yosys\s+-p\s+(["'])(.*?)\1""", re.S)
NEGATION = ("not", "no ", "never", "without")


def summary_lines(run_text):
    """Lines of a `run:` block that are actually written to the step summary.

    A shell comment is excluded even when it mentions the summary variable, so a
    claim can only be satisfied by text a reader will really see.
    """
    out = []
    for line in (run_text or "").splitlines():
        stripped = line.strip()
        if stripped.startswith("#"):
            continue
        if SUMMARY_VAR in stripped:
            out.append(stripped)
    return out


def yosys_passes(run_text):
    """Pass names from every `yosys -p` script in a run block, in order.

    `read_verilog -sv -DSIMULATION $v; hierarchy -top $name` -> [read_verilog,
    hierarchy]. Tokens that are shell interpolations are dropped: a pass name
    assembled at runtime is not something a static claim can be matched against.
    """
    names = []
    for _quote, script in YOSYS_P.findall(run_text or ""):
        for part in script.split(";"):
            part = part.strip()
            if not part:
                continue
            head = part.split()[0]
            if head.startswith("$") or not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", head):
                continue
            if head not in names:
                names.append(head)
    return names


def states_non_coverage(lines):
    """True when some line carries a negation AND the word 'elaborat'."""
    for line in lines:
        low = line.lower()
        if "elaborat" in low and any(n in low for n in NEGATION):
            return True
    return False


def check(doc):
    """Return a list of (code, message). Empty list means clean."""
    v = []
    jobs = doc.get("jobs") if isinstance(doc, dict) else None
    if not isinstance(jobs, dict):
        return [("no-jobs", "the workflow has no readable `jobs:` mapping")]

    lint = jobs.get(LINT_JOB)
    if not isinstance(lint, dict):
        return [("no-lint-job", f"job `{LINT_JOB}` is missing; nothing to hold to its claim")]

    steps = lint.get("steps") or []
    lint_step = None
    for st in steps:
        if not isinstance(st, dict):
            continue
        run = st.get("run") or ""
        if "yosys" in run and SUMMARY_VAR in run:
            lint_step = st
            break

    if lint_step is None:
        v.append(("no-lint-step",
                  f"no step in `{LINT_JOB}` both runs yosys and writes to ${SUMMARY_VAR}. "
                  "There is nothing to compare a claim against, and passing here would "
                  "be a gate that stopped measuring without saying so."))
        return v

    run = lint_step.get("run") or ""
    passes = yosys_passes(run)
    lines = summary_lines(run)

    if not passes:
        v.append(("no-yosys-passes",
                  "the lint step runs yosys but no pass names could be read from a "
                  "`yosys -p \"...\"` script; the depth is unknown, so no claim about "
                  "it can be verified"))
        return v

    depth = " + ".join(passes)

    # 2. the display name must carry the depth
    job_name = lint.get("name")
    if not isinstance(job_name, str) or not job_name.strip():
        v.append(("job-name-missing",
                  f"job `{LINT_JOB}` has no `name:`, so the check list shows the bare "
                  f"job id. `{LINT_JOB}` reads as a verdict on the RTL; it runs {depth}. "
                  f"Give it a display name that says so."))
    else:
        absent = [p for p in passes if p not in job_name]
        if absent:
            v.append(("job-name-omits-depth",
                      f"job display name {job_name!r} does not name {', '.join(absent)}, "
                      f"which the step runs. The label must state the depth it measures."))

    # 3. the summary text must carry the depth
    joined = "\n".join(lines)
    absent = [p for p in passes if p not in joined]
    if absent:
        v.append(("summary-omits-depth",
                  f"the step summary never names {', '.join(absent)}, which the step "
                  f"runs. A reader cannot tell how deep the green goes."))

    # 4. the headline Result line must be stated at that depth
    result_lines = [l for l in lines if "**Result:**" in l]
    if not result_lines:
        v.append(("summary-no-result-line",
                  "the step summary has no `**Result:**` line. Deleting the headline "
                  "instead of qualifying it leaves the count unstated, not honest."))
    else:
        # Every pass, not merely one of them. "at least one" was the first rule
        # here and the self-test's deepen-to-synth mutant walked straight through
        # it: the line still said `read_verilog + hierarchy`, `read_verilog` was
        # still a pass, and the assertion was satisfied by the stale half of a
        # claim that had gone wrong. A headline that names some of the depth is
        # how a headline drifts.
        joined_result = "\n".join(result_lines)
        absent = [p for p in passes if p not in joined_result]
        if absent:
            v.append(("result-line-omits-depth",
                      f"the `**Result:**` line states a count without naming "
                      f"{', '.join(absent)} (the step runs {depth}): "
                      + " | ".join(result_lines)
                      + ". A bare 'passed Yosys lint' is read as a clean bill for the file."))

    # 5. the summary must state what it does not reach
    if not states_non_coverage(lines):
        v.append(("summary-omits-non-coverage",
                  "no line of the step summary states that function bodies are not "
                  "elaborated here. Yosys enters a function only at a call site, and "
                  "406 of the 485 functions in these files have none (#2326), so the "
                  "summary must say what the green does not cover."))

    # 6. the pipeline report row must state the same
    report = jobs.get(REPORT_JOB)
    if isinstance(report, dict):
        rows = []
        for st in report.get("steps") or []:
            if isinstance(st, dict):
                rows += [l for l in summary_lines(st.get("run"))
                         if f"needs.{LINT_JOB}.result" in l]
        if not rows:
            v.append(("report-row-missing",
                      f"`{REPORT_JOB}` no longer renders needs.{LINT_JOB}.result; the "
                      f"row this test holds honest has gone, which needs a human look, "
                      f"not a silent pass."))
        elif not states_non_coverage(rows):
            v.append(("report-row-omits-non-coverage",
                      f"the `{REPORT_JOB}` row for {LINT_JOB} does not say function "
                      f"bodies are not elaborated: " + " | ".join(rows)
                      + ". That row is the only fpga-lint status most readers see."))

    return v


# ---------------------------------------------------------------------------
# Self-test: this gate must be able to fail.
# ---------------------------------------------------------------------------

_LINT_RUN_DEFECTIVE = """
echo "## Yosys Lint Results" >> $GITHUB_STEP_SUMMARY
echo "| Module | read_verilog | hierarchy | Status |" >> $GITHUB_STEP_SUMMARY
if yosys -p "read_verilog -sv -DSIMULATION $v; hierarchy -top $name" -q; then
  echo "| $name | OK | OK | PASS |" >> $GITHUB_STEP_SUMMARY
fi
echo "**Result:** $pass/$total modules passed Yosys lint" >> $GITHUB_STEP_SUMMARY
"""

_LINT_RUN_HONEST = """
echo "## Yosys parse + hierarchy (all modules)" >> $GITHUB_STEP_SUMMARY
echo 'Depth: `read_verilog -sv -DSIMULATION` then `hierarchy -top <module>`.' >> $GITHUB_STEP_SUMMARY
echo 'DOES NOT CHECK: function bodies are not elaborated.' >> $GITHUB_STEP_SUMMARY
if yosys -p "read_verilog -sv -DSIMULATION $v; hierarchy -top $name" -q; then
  echo "| $name | OK | OK | PASS |" >> $GITHUB_STEP_SUMMARY
fi
echo "**Result:** $pass/$total modules parsed and resolved top-level hierarchy (read_verilog + hierarchy)." >> $GITHUB_STEP_SUMMARY
"""

_REPORT_RUN_DEFECTIVE = 'echo "| Lint (all 31 modules) | ${{ needs.fpga-lint.result }} |" >> $GITHUB_STEP_SUMMARY'
_REPORT_RUN_HONEST = 'echo "| Lint: parse + hierarchy (function bodies NOT elaborated) | ${{ needs.fpga-lint.result }} |" >> $GITHUB_STEP_SUMMARY'


def _doc(lint_name, lint_run, report_run):
    lint = {"steps": [{"name": "lint", "run": lint_run}]}
    if lint_name is not None:
        lint["name"] = lint_name
    return {"jobs": {
        LINT_JOB: lint,
        REPORT_JOB: {"steps": [{"name": "Summary", "run": report_run}]},
    }}


def _self_test():
    """Each case is a separate run of check(), so one case cannot mask another."""
    cases = []

    # The real pre-change shape of fpga-build.yml, verbatim in the parts that matter.
    cases.append((
        "master shape (no display name, bare 'passed Yosys lint', no disclaimer)",
        _doc(None, _LINT_RUN_DEFECTIVE, _REPORT_RUN_DEFECTIVE),
        {"job-name-missing", "result-line-omits-depth",
         "summary-omits-non-coverage", "report-row-omits-non-coverage"},
    ))

    # The corrected shape must be clean, or the gate is not TRUE.
    cases.append((
        "corrected shape",
        _doc("fpga-lint (read_verilog + hierarchy)", _LINT_RUN_HONEST, _REPORT_RUN_HONEST),
        set(),
    ))

    # Mutant: deepen the command without moving the claim. The depth assertions
    # must bite even though every disclaimer is still in place.
    deepened = _LINT_RUN_HONEST.replace("hierarchy -top $name", "synth -top $name")
    cases.append((
        "yosys deepened to synth, claims left behind",
        _doc("fpga-lint (read_verilog + hierarchy)", deepened, _REPORT_RUN_HONEST),
        {"job-name-omits-depth", "summary-omits-depth", "result-line-omits-depth"},
    ))

    # Mutant: the disclaimer moved into a shell comment. Comments are not claims.
    commented = _LINT_RUN_HONEST.replace(
        "echo 'DOES NOT CHECK: function bodies are not elaborated.' >> $GITHUB_STEP_SUMMARY",
        "# DOES NOT CHECK: function bodies are not elaborated. >> $GITHUB_STEP_SUMMARY")
    cases.append((
        "disclaimer demoted to a shell comment",
        _doc("fpga-lint (read_verilog + hierarchy)", commented, _REPORT_RUN_HONEST),
        {"summary-omits-non-coverage"},
    ))

    # Mutant: the headline deleted rather than qualified.
    headless = "\n".join(l for l in _LINT_RUN_HONEST.splitlines()
                         if "**Result:**" not in l)
    cases.append((
        "headline Result line deleted instead of qualified",
        _doc("fpga-lint (read_verilog + hierarchy)", headless, _REPORT_RUN_HONEST),
        {"summary-no-result-line"},
    ))

    # Mutant: the yosys command removed. A gate that can no longer measure must
    # not report clean.
    cases.append((
        "yosys command gone",
        _doc("fpga-lint (read_verilog + hierarchy)",
             _LINT_RUN_HONEST.replace("yosys -p", "true #"), _REPORT_RUN_HONEST),
        {"no-lint-step"},
    ))

    failures = 0
    for label, doc, expected in cases:
        got = {code for code, _ in check(doc)}
        ok = got == expected
        print(f"  [{'ok' if ok else 'FAIL'}] {label}")
        print(f"        expected: {sorted(expected) or '(clean)'}")
        print(f"        got:      {sorted(got) or '(clean)'}")
        if not ok:
            failures += 1

    if failures:
        print(f"\nSELF-TEST FAILED: {failures}/{len(cases)} cases")
        return 1
    print(f"\nSELF-TEST PASSED: {len(cases)}/{len(cases)} cases, including "
          f"{len(cases) - 1} that must be rejected.")
    return 0


def main(argv):
    if "--self-test" in argv:
        print("Self-test: the claim gate must reject the shapes it exists to reject.")
        return _self_test()

    if not os.path.isfile(WORKFLOW):
        print(f"no {WORKFLOW} -- run from the repository root", file=sys.stderr)
        return 2
    try:
        with open(WORKFLOW) as fh:
            doc = yaml.safe_load(fh)
    except Exception as e:
        print(f"{WORKFLOW}: {type(e).__name__}: {e}", file=sys.stderr)
        return 2

    violations = check(doc)
    print(f"workflow: {WORKFLOW}")
    lint = (doc.get("jobs") or {}).get(LINT_JOB) or {}
    steps = [s for s in (lint.get("steps") or []) if isinstance(s, dict)]
    for st in steps:
        if "yosys" in (st.get("run") or "") and SUMMARY_VAR in (st.get("run") or ""):
            print(f"lint step:  {st.get('name', '(unnamed)')}")
            print(f"yosys depth: {' + '.join(yosys_passes(st['run'])) or '(none read)'}")
            break
    print(f"display name: {lint.get('name', '(none -- job id is shown instead)')}")

    if violations:
        print(f"\nCLAIM DOES NOT MATCH DEPTH ({len(violations)}):")
        for code, msg in violations:
            print(f"  [{code}] {msg}")
        print("\nfpga-lint runs read_verilog + hierarchy. It does not elaborate")
        print("function bodies, and 406 of 485 functions in the generated Verilog")
        print("have no call site for yosys to enter them by (#2326). Its name, its")
        print("summary and the fpga-report row must all say so.")
        return 1

    print("\nCLEAN: fpga-lint's label, summary headline and report row all state the")
    print("depth they measure and the elaboration they do not reach.")
    print("Scope: this checks what the job SAYS, not what it catches. Making it")
    print("catch the 28 modules that fail iverilog is a gate-policy decision and is")
    print("deliberately not made here -- see #2326, #2325, #2241.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
