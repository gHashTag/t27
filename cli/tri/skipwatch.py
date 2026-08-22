#!/usr/bin/env python3
"""Ratchet on the set of `cli/tri` tests that skip themselves at runtime.

Why this exists (#2370, and the narrow issue this landed under):

Three smoke-gate tests need a Lean toolchain, no workflow installs one, and they
do not use `#[ignore]`. They `println!` a reason and `return`, so libtest counts
them as PASSED. Two consequences, both measured on master:

  * `cargo test -p tri` reports `173 passed; 0 failed; 0 ignored`. The word
    "ignored" is 0. The aggregate cannot tell you a phase never ran.
  * `cargo test` captures stdout of passing tests, so the printed reason is not
    in the log at all. `--nocapture` is required to see it, and
    `--test-threads=1` is required to attribute it to a test name, because
    libtest emits the reason onto the `test NAME ... ` line it already opened.

So a green `cli-tri` run is consistent with those tests never having executed.
This script makes that visible and ratchets it: it fails when the skip set
GROWS, which is the regression nobody would otherwise see -- a fourth test
starting to skip silently.

It deliberately does NOT fail on the skips that exist today. Turning those red
is the owner's policy call, and a skip is not a failure. #2370 stays open for
the real fix (install the toolchain).

Usage:
    skipwatch.py <test-log> <baseline-file>            # ratchet (exit 1 on growth)
    skipwatch.py <test-log> <baseline-file> --emit-baseline   # write baseline to stdout

The baseline is produced by THIS script in --emit-baseline mode. It is never
hand-written: a hand-written "3" would only restate what we already know and
would not survive a rename.
"""

import os
import re
import sys

# libtest, under `--nocapture --test-threads=1`, opens a line as
# "test some::path::name ... " and then lets the test's own stdout land on it.
TEST_LINE = re.compile(r"^test ([A-Za-z0-9_:]+) \.\.\. ?(.*)$")
# The verdict libtest appends once the test returns.
VERDICT = re.compile(r"^(ok|FAILED|ignored)$")
# The in-repo convention for a self-skip, in either case, with : or -.
SKIP_MARK = re.compile(r"^\s*skip\b\s*[:\-]", re.IGNORECASE)

# Volatile substrings that must not enter the baseline: absolute paths differ
# between a runner and a laptop, and pids/sizes differ between runs.
ABS_PATH = re.compile(r"/[^\s,;)]+")
LONG_NUM = re.compile(r"\b\d{3,}\b")


def normalise(reason):
    """Reduce a skip reason to something stable across machines and runs."""
    text = ABS_PATH.sub("<path>", reason.strip())
    text = LONG_NUM.sub("<n>", text)
    return re.sub(r"\s+", " ", text).strip().lower()


def parse(log_lines):
    """Return (completed, {test_name: normalised_reason}).

    `completed` is False when the log has no `test result:` line, i.e. the run
    did not finish. That is NOT an empty skip set -- absence is not a value --
    and the caller must refuse to ratchet on it.
    """
    completed = any(line.startswith("test result:") for line in log_lines)
    skips = {}
    i = 0
    while i < len(log_lines):
        match = TEST_LINE.match(log_lines[i])
        if not match:
            i += 1
            continue
        name, tail = match.group(1), match.group(2)
        # The test's stdout: whatever shared the opening line, plus every line
        # up to libtest's verdict or the next test.
        block = []
        if tail and not VERDICT.match(tail.strip()):
            block.append(tail)
        j = i + 1
        while j < len(log_lines):
            line = log_lines[j]
            if (
                TEST_LINE.match(line)
                or VERDICT.match(line.strip())
                or line.startswith("test result:")
            ):
                break
            block.append(line)
            j += 1
        for line in block:
            if SKIP_MARK.match(line):
                skips[name] = normalise(line)
                break
        i = max(j, i + 1)
    return completed, skips


def render(skips):
    return sorted("%s\t%s" % (name, reason) for name, reason in skips.items())


def load_baseline(path):
    try:
        with open(path, encoding="utf-8") as handle:
            raw = handle.read().splitlines()
    except OSError:
        return None
    return sorted(
        line.rstrip()
        for line in raw
        if line.strip() and not line.lstrip().startswith("#")
    )


def summary(text):
    sys.stdout.write(text + "\n")
    path = os.environ.get("GITHUB_STEP_SUMMARY")
    if path:
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(text + "\n")


def main(argv):
    if len(argv) < 3:
        sys.stderr.write(__doc__)
        return 2
    log_path, baseline_path = argv[1], argv[2]
    emit = "--emit-baseline" in argv[3:]

    try:
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            lines = handle.read().splitlines()
    except OSError as exc:
        sys.stderr.write("cannot read test log %s: %s\n" % (log_path, exc))
        return 0 if not emit else 2

    completed, skips = parse(lines)
    observed = render(skips)

    if emit:
        sys.stdout.write("# Generated by cli/tri/skipwatch.py --emit-baseline.\n")
        sys.stdout.write("# Do not hand-edit: regenerate from a real test log.\n")
        sys.stdout.write("# Each line is a test that skipped ITSELF at runtime,\n")
        sys.stdout.write("# with its reason. libtest reports every one as passed.\n")
        for line in observed:
            sys.stdout.write(line + "\n")
        return 0

    if not completed:
        # The build or the run died. We have no measurement, so we make no claim.
        print("::notice title=skip ratchet::test run did not complete; skip set NOT evaluated")
        summary("### Runtime skip ratchet\n\nTest run did not complete — skip set not evaluated.")
        return 0

    baseline = load_baseline(baseline_path)
    if baseline is None:
        print("::error title=skip ratchet::baseline %s is missing" % baseline_path)
        return 1

    baseline_set, observed_set = set(baseline), set(observed)
    new = sorted(observed_set - baseline_set)
    gone = sorted(baseline_set - observed_set)

    lines_out = ["### Runtime skip ratchet", ""]
    lines_out.append(
        "`%d` test(s) skipped themselves at runtime. libtest counts every one of "
        "them as **passed** and reports `0 ignored`, so a green run here is *not* "
        "evidence these ran." % len(observed)
    )
    lines_out.append("")
    if observed:
        lines_out.append("| test | reason |")
        lines_out.append("| --- | --- |")
        for line in observed:
            name, _, reason = line.partition("\t")
            lines_out.append("| `%s` | %s |" % (name, reason))
        lines_out.append("")

    print(
        "::notice title=skip ratchet::%d test(s) skipped at runtime and were "
        "counted as passed (baseline %d)" % (len(observed), len(baseline))
    )

    if gone:
        lines_out.append(
            "%d baseline entr(y/ies) no longer skip — tighten the baseline by "
            "regenerating it:" % len(gone)
        )
        for line in gone:
            lines_out.append("- ~~`%s`~~" % line.replace("\t", "` — `"))
        lines_out.append("")
        print(
            "::notice title=skip ratchet::%d baseline entr(y/ies) no longer skip; "
            "regenerate the baseline to tighten it" % len(gone)
        )

    if new:
        lines_out.append("**FAILED — the skip set grew.** New runtime skip(s):")
        for line in new:
            lines_out.append("- `%s`" % line.replace("\t", "` — `"))
        lines_out.append("")
        lines_out.append(
            "A test that skips itself is reported as passed. Either make it run "
            "on this runner, or record it in `%s` by regenerating the baseline "
            "and say in the PR why the coverage loss is acceptable." % baseline_path
        )
        summary("\n".join(lines_out))
        for line in new:
            name, _, reason = line.partition("\t")
            print(
                "::error title=skip ratchet::new runtime skip: %s (%s)" % (name, reason)
            )
        return 1

    lines_out.append("No new runtime skips. Baseline holds.")
    summary("\n".join(lines_out))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
