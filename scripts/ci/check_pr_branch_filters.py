#!/usr/bin/env python3
"""Configuration test: merge-critical workflows must not filter `pull_request` by branch.

WHY THIS TEST EXISTS
--------------------
A workflow declared as

    on:
      pull_request:
        branches: [master]

does not run at all when the pull request's BASE is another branch. On a stacked
PR -- base = some other feature branch -- every such gate is simply absent, and
`gh pr checks` then prints a green list. That green list is not the green list a
master-based PR would get: it is the green of a gate that never ran.

Observed 2026-08-15 in this repository: three gates (`now-sync-gate`,
`issue-gate`, `seal-staleness-warn`) were invisible on a stacked PR for exactly
this reason, and the PR read as fully checked. The failure mode is silent by
construction -- nothing reports a gate that did not fire -- so it needs a
configuration test rather than vigilance.

WHAT IT CHECKS
--------------
1. Every workflow listed in MERGE_CRITICAL must exist.
2. None of them may carry a `branches` (or `branches-ignore`) filter under
   `pull_request` or `pull_request_target`. A `paths` filter is fine: it selects
   by what changed, not by where the change is headed. A `push` branch filter is
   also fine and is left alone -- restricting post-merge runs to master is a cost
   decision, not a gating hole.
3. Every workflow file must parse as YAML. A workflow GitHub cannot load is a
   gate that does not exist, which is the same hazard by a different route.

WHAT IT DELIBERATELY DOES NOT CHECK
-----------------------------------
It does not decide which workflows *ought* to be merge-critical. That list is a
human judgement about what must block a merge, so it is written out explicitly
below and reviewed as code. A test that inferred the list -- say, "everything
whose name contains gate" -- would quietly stop covering a gate the moment
someone renamed it.

Exit status: 0 clean, 1 violations found, 2 setup error.
"""
import glob
import os
import sys

try:
    import yaml
except ImportError:
    print("pyyaml is required: pip install pyyaml", file=sys.stderr)
    sys.exit(2)

# Merge-critical: failure of this workflow should be able to block a merge.
# Reviewed as code on purpose -- see the docstring.
MERGE_CRITICAL = (
    "build-paper.yml",
    "catalog-count-gate.yml",
    "check-now-freshness.yml",
    "coq-kernel.yml",
    "emit-bitexact-gate.yml",
    "fpga-build.yml",
    "issue-gate.yml",
    "notebook-gate.yml",
    "now-sync-gate.yml",
    "phi-loop-ci.yml",
    "schema-validation.yml",
    "seal-coverage.yml",
    "secret-scan.yml",
    "verilog-widths.yml",
    "damage-negatives.yml",
    # Reads README, .claude/skills and docs against `t27c --help`. Merge-
    # critical because the defect it catches is a document telling a reader
    # to run a command that does not exist, and a branch filter would skip
    # it on exactly the PR that renames one.
    "documented-commands.yml",
    # Added when the third bucket below was first printed. All three were in
    # NEITHER list, so this check reported CLEAN without ever reading them --
    # and two of them carried the exact defect it exists to detect.
    "corpus-ratchet.yml",
    "withdrawn-live-gate.yml",
    "harness-scratch.yml",
)

# The two lists above are a partition ONLY of the files they name. Everything
# else in .github/workflows/ was read by nothing here, and the summary printed
# the two counts beside the file count without ever subtracting them: 15 + 4
# against 49 present, so 30 files were never examined and the last line still
# said CLEAN.
#
# Two of those 30 carried `pull_request: branches: [master]` -- the very defect
# this check exists to detect -- and one of them was `corpus-ratchet.yml`, which
# does not run at all on a stacked pull request and shows a green check list
# instead.
#
# A ceiling rather than a refusal, because 27 files cannot be classified in the
# commit that discovers them and a gate that is red on the day it lands teaches
# everyone to ignore red. It moves DOWN only: classify a file and lower this in
# the same commit, so the next unclassified workflow cannot hide in the slack.
MAX_UNCLASSIFIED = 27

# Not merge-critical, and each exclusion is stated with its reason so that a
# future reader can disagree with the reason rather than guess at the omission.
NOT_MERGE_CRITICAL = {
    "pr-dashboard.yml": "reporting only; a stale dashboard does not gate a merge",
    "notebook-sync.yml": "automation targeted at feature branches by design",
    "seal-staleness-warn.yml": "warn-only by name and by intent",
    "auto-merge-ready-prs.yml": "auto-merge is disabled by policy in this repo",
}

FILTER_KEYS = ("branches", "branches-ignore")
PR_EVENTS = ("pull_request", "pull_request_target")


def load(path):
    with open(path) as fh:
        return yaml.safe_load(fh)


def on_block(doc):
    """Return the `on:` mapping.

    YAML 1.1 resolves the bare word `on` to the boolean True, so a document
    written with `on:` may land under the key True depending on the loader. Both
    are accepted; a test that missed the whole trigger block because of a loader
    quirk would pass by vacuity.
    """
    if not isinstance(doc, dict):
        return None
    for key in ("on", True, "On", "ON"):
        if key in doc:
            v = doc[key]
            return v if isinstance(v, dict) else None
    return None


def main():
    wf_dir = os.path.join(".github", "workflows")
    if not os.path.isdir(wf_dir):
        print(f"no {wf_dir} -- run from the repository root", file=sys.stderr)
        return 2

    violations = []
    unparseable = []
    missing = []

    present = {os.path.basename(p) for p in glob.glob(os.path.join(wf_dir, "*.yml"))}
    for name in MERGE_CRITICAL:
        if name not in present:
            missing.append(name)

    for path in sorted(glob.glob(os.path.join(wf_dir, "*.yml"))):
        name = os.path.basename(path)
        try:
            doc = load(path)
        except Exception as e:
            unparseable.append((name, f"{type(e).__name__}: {e}".splitlines()[0]))
            continue
        if name not in MERGE_CRITICAL:
            continue
        on = on_block(doc)
        if on is None:
            violations.append((name, "no readable `on:` block"))
            continue
        for ev in PR_EVENTS:
            cfg = on.get(ev)
            if not isinstance(cfg, dict):
                continue
            for k in FILTER_KEYS:
                if k in cfg:
                    violations.append(
                        (name, f"{ev}.{k} = {cfg[k]!r} -- this gate does not run "
                               f"when a PR targets any other base"))

    unclassified = sorted(present - set(MERGE_CRITICAL) - set(NOT_MERGE_CRITICAL))
    both = sorted(set(MERGE_CRITICAL) & set(NOT_MERGE_CRITICAL))

    # The same read, over the files no list names. Reported, not failed: whether
    # one of these ought to block a merge is a human call. What is NOT a human
    # call is whether anybody looked.
    unclassified_filtered = []
    for name in unclassified:
        path = os.path.join(wf_dir, name)
        try:
            doc = load(path)
        except Exception:
            continue
        on = on_block(doc)
        if on is None:
            continue
        for ev in PR_EVENTS:
            cfg = on.get(ev)
            if not isinstance(cfg, dict):
                continue
            for k in FILTER_KEYS:
                if k in cfg:
                    unclassified_filtered.append((name, f"{ev}.{k} = {cfg[k]!r}"))

    print(f"merge-critical workflows checked: {len(MERGE_CRITICAL)}")
    print(f"explicitly not merge-critical:    {len(NOT_MERGE_CRITICAL)}")
    print(f"in NEITHER list, never read:      {len(unclassified)}")
    print(f"workflow files present:           {len(present)}")
    print(f"  {len(MERGE_CRITICAL)} + {len(NOT_MERGE_CRITICAL)} + {len(unclassified)}"
          f" = {len(MERGE_CRITICAL) + len(NOT_MERGE_CRITICAL) + len(unclassified)}"
          f"  (must equal {len(present)})")

    if unclassified:
        print(f"\nNOT CLASSIFIED ({len(unclassified)}, ceiling {MAX_UNCLASSIFIED}):")
        for name in unclassified:
            print(f"  {name}")
        print("  This check reads none of these. Put each in one of the two lists")
        print("  above -- NOT_MERGE_CRITICAL carries its reason, so a later reader")
        print("  can disagree with the reason rather than guess at the omission.")

    if unclassified_filtered:
        print(f"\nWORK LIST -- unclassified AND branch-filtered ({len(unclassified_filtered)}):")
        for name, why in unclassified_filtered:
            print(f"  {name}\n      {why}")
        print("  Each does not run when a PR targets any base but master. That is")
        print("  a gating hole if the workflow is merge-critical and a cost")
        print("  decision if it is not, and nothing here can tell which.")
    else:
        print("\nUnclassified workflows carrying a pull_request branch filter: 0")

    # An unparseable file is split by whether it is merge-critical, and the split
    # is a deliberate judgement rather than leniency. A merge-critical workflow
    # that does not parse is a gate that does not exist, so it fails. A file that
    # is not merge-critical is reported as a warning, because a gate that is red
    # on the day it lands, and stays red for a reason nobody is allowed to fix,
    # teaches everyone to ignore red -- which costs more than the file it flags.
    hard = [(n, e) for n, e in unparseable if n in MERGE_CRITICAL]
    soft = [(n, e) for n, e in unparseable if n not in MERGE_CRITICAL]

    if hard:
        print(f"\nUNPARSEABLE MERGE-CRITICAL WORKFLOWS ({len(hard)}):")
        for name, err in hard:
            print(f"  {name}: {err}")
        print("  A workflow that does not parse is a gate that does not exist.")

    if soft:
        print(f"\nWARNING -- unparseable, not merge-critical ({len(soft)}):")
        for name, err in soft:
            why = NOT_MERGE_CRITICAL.get(name, "not in the merge-critical list")
            print(f"  {name}: {err}")
            print(f"      treated as a warning because: {why}")
        print("  GitHub cannot load these files either, so they do not run. That")
        print("  may be harmless or may be a silently dead automation; deciding")
        print("  which is a human call, so this does not fail the build.")

    if missing:
        print(f"\nMISSING ({len(missing)}): {', '.join(missing)}")
        print("  Either the file was renamed and this list was not updated, or a")
        print("  gate was deleted. Both need a human decision, not a silent pass.")

    if violations:
        print(f"\nBRANCH-FILTERED MERGE-CRITICAL WORKFLOWS ({len(violations)}):")
        for name, why in violations:
            print(f"  {name}\n      {why}")
        print("\nRemove the `branches:` filter from the pull_request trigger. Keep")
        print("any `paths:` filter -- it selects by what changed, not by target.")
        return 1

    if both:
        print(f"\nIN BOTH LISTS ({len(both)}): {', '.join(both)}")
        print("  A file cannot be merge-critical and explicitly not. One list is")
        print("  wrong and this check cannot say which.")
        return 1

    if len(unclassified) > MAX_UNCLASSIFIED:
        print(f"\nUNCLASSIFIED ROSE {MAX_UNCLASSIFIED} -> {len(unclassified)}")
        print("  A workflow was added and named in neither list, so this check")
        print("  does not read it. Classify it and the ceiling holds; classify an")
        print("  old one too and lower the ceiling in the same commit.")
        return 1

    if missing or hard:
        return 1

    print(f"\nCLEAN: no merge-critical workflow filters pull_request by branch,"
          f" and\n{len(unclassified)} file(s) remain unread at a ceiling of {MAX_UNCLASSIFIED}.")
    print("Scope: this checks trigger configuration only. It does not verify that")
    print("the gates are registered as required checks in branch protection, which")
    print("is repository settings and cannot be read from the tree.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
