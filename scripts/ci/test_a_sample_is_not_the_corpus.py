#!/usr/bin/env python3
"""A --limit sample must not be printed as the corpus.

`diffbin` truncated its file list with `files = files[:limit]` and then took its
denominator from the truncated list, printing "MEASURED COVERAGE: m/t = p% of the
corpus" and a header reading "corpus: t specs under <dir>".

Measured 2026-09-05 against the real tree: `--limit 10` over `specs` printed
`corpus: 10 specs under specs` while 650 .t27 files were present. A 2% sample
could print 100% coverage -- directly under a paragraph that reads "Coverage
below 100% bounds what the run can claim", so the one figure that bounds the
claim was the one the truncation had already destroyed.

No compiler is needed: the fake binary here never produces a verdict, which is
irrelevant to the question. The subject is the DENOMINATOR and its label.
"""
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
DIFFBIN = os.path.join(ROOT, "tri_loop", "diffbin.py")

CORPUS_N = 25
SAMPLE_N = 4


def build_tree(d):
    specs = os.path.join(d, "specs")
    os.makedirs(specs)
    for i in range(CORPUS_N):
        with open(os.path.join(specs, f"s{i:03d}.t27"), "w") as fh:
            fh.write("# fixture\n")
    fake = os.path.join(d, "fake-t27c")
    with open(fake, "w") as fh:
        fh.write("#!/bin/sh\nexit 1\n")
    os.chmod(fake, 0o755)
    return specs, fake


def run(specs, fake, *extra):
    p = subprocess.run(
        [sys.executable, DIFFBIN, fake, fake, specs, *extra],
        capture_output=True, text=True,
    )
    return p.stdout + p.stderr


def main():
    with tempfile.TemporaryDirectory() as d:
        specs, fake = build_tree(d)

        whole = run(specs, fake)
        if f"corpus: {CORPUS_N} specs" not in whole:
            print("FAIL: an untruncated run must name the corpus and its real size")
            print(whole)
            return 1
        if "THIS IS A SAMPLE" in whole:
            print("FAIL: nothing was sampled, so nothing may say it was")
            print(whole)
            return 1

        sample = run(specs, fake, "--limit", str(SAMPLE_N))
        if f"sample: {SAMPLE_N} of {CORPUS_N} specs" not in sample:
            print(f"FAIL: a truncated run must say it is a sample and name both numbers")
            print(sample)
            return 1
        if f"corpus: {SAMPLE_N} specs" in sample:
            print(f"FAIL: {SAMPLE_N} files were compared and {CORPUS_N} are present; "
                  f"calling the {SAMPLE_N} 'the corpus' is the whole defect")
            print(sample)
            return 1

        cov = [l for l in sample.splitlines() if "MEASURED COVERAGE" in l]
        if not cov:
            print("FAIL: no coverage line at all")
            print(sample)
            return 1
        if "of the corpus" in cov[0]:
            print("FAIL: the coverage denominator is the SAMPLE, so its label may not "
                  "say 'of the corpus'")
            print(cov[0])
            return 1
        if f"of the {SAMPLE_N} compared" not in cov[0]:
            print(f"FAIL: coverage must name what it is a fraction OF")
            print(cov[0])
            return 1
        if "THIS IS A SAMPLE" not in sample:
            print("FAIL: a sampled run must say so where the number is read, not only "
                  "in the header")
            print(sample)
            return 1

    print(f"ok       {CORPUS_N}-file corpus, --limit {SAMPLE_N}: sample named as a "
          f"sample, coverage denominator not called the corpus")
    return 0


if __name__ == "__main__":
    sys.exit(main())
