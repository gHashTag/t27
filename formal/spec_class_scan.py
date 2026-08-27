#!/usr/bin/env python3
"""Gate 28: a metric over a mixed population measures composition, not quality.

`spec_parse_gate` ratchets one number: recovery events across specs/. It has been
read every wave as a parser backlog -- 1741, then 384, then 161, then 154, each
drop credited to a parser repair. Emitting the recovery MESSAGES (Prop. 197) and
ranking them shows the number is not one population:

    48 events / 30 files   Unexpected top-level token: Ident   e.g. `## Specification`
    43 events / 17 files   Unexpected top-level token: Minus   e.g. `- protocol_version: ...`

Both are **Markdown**. Headings and bullet lists, in files named `.t27`.
Enumerated: **16 of 497 specs carry Markdown structure, and they account for 55 of
the 154 events -- 36%.**

Prop. 189 already met one of these. `c_api_contract.t27` was excused BY NAME as
"documentation wearing a .t27 extension", with the reasoning that renaming it is a
decision about the corpus rather than a parser defect. That reasoning was right and
the scope was wrong: it was recorded as a singleton exception when it was a sample
of a population, and the fifteen others went on inflating a number read as a parser
backlog.

WHAT THIS GATE DOES. It does not rename anything -- that remains a corpus decision.
It SPLITS the population, so the two numbers can move independently:

  * documents: >= 2 Markdown headings or >= 2 fenced blocks
  * specs: everything else

and ratchets the DOCUMENT COUNT, so a new .md-shaped file cannot enter specs/
unnoticed and silently raise the parser's backlog.

COVERAGE. Examines all 497 `.t27` files. Classification is structural and
therefore approximate in both directions: a spec with a long fenced example is
called a document, and a document with no headings is called a spec. The threshold
(2 headings or 2 fences) was chosen after inspecting the distribution and is
printed with the run so it can be argued with; it is not derived from anything. It
classifies FILES, never claims, and says nothing about whether any spec's content
is correct.

ARTIFACTS. Reads `specs/**/*.t27`. WRITES `formal/spec_class_baseline.txt`, and only when
`--init` is passed. Nothing else.

Prop. 197.
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SPECS = ROOT / "specs"
MIN_HEADINGS = 2
MIN_FENCES = 2


def is_document(text):
    headings = len([l for l in text.splitlines() if re.match(r"^#{1,6} \S", l)])
    fences = text.count("```") // 2
    return headings >= MIN_HEADINGS or fences >= MIN_FENCES, headings, fences


def main():
    if not SPECS.exists():
        print("::error::spec class scan: no such directory 'specs' under the "
              "repository root -- nothing was scanned")
        return 1
    files = sorted(SPECS.rglob("*.t27"))
    if not files:
        print("::error::spec class scan: found no .t27 files under specs/ -- "
              "nothing was scanned")
        return 1

    docs = []
    for f in files:
        doc, h, fc = is_document(f.read_text(errors="ignore"))
        if doc:
            docs.append(f"{f.relative_to(ROOT)}\t{h}h\t{fc}f")

    print(f"spec class scan: {len(files)} .t27 files, {len(docs)} are Markdown "
          f"documents by structure (>= {MIN_HEADINGS} headings or "
          f">= {MIN_FENCES} fenced blocks), {len(files) - len(docs)} are specs")

    baseline = ROOT / "formal" / "spec_class_baseline.txt"
    now = sorted(docs)
    if not baseline.exists():
        # Prop. 211c: writing a baseline is an explicit act, never a fallback.
        # `if not exists(): write(now); return 0` resets the ratchet on one
        # `rm`, and on a clone that never had the file it rubber-stamps the tree
        # it was handed and exits 0. Measured before f66561f33: 8 of the 13
        # baselines in this suite were on disk and in no commit, and 8 of the 13
        # gates owning them re-baseline a possibly-broken tree and pass.
        if "--init" not in sys.argv[1:]:
            print(f"::error::spec class scan: {baseline.name} does not exist and "
                  f"--init was not given. Writing one here would record "
                  f"whatever this tree contains as the accepted state -- on a "
                  f"fresh clone that is a green run which checked nothing. "
                  f"Genuine first run: `python3 formal/spec_class_scan.py --init`. "
                  f"Otherwise the baseline was lost and belongs in the commit "
                  f"that lost it (Prop. 211)")
            return 1
        baseline.write_text("\n".join(now) + ("\n" if now else ""))
        print(f"spec class scan: baseline written to {baseline.name} "
              f"({len(now)} documents)")
        return 0
    was = [l for l in baseline.read_text().splitlines() if l.strip()]
    new = [d for d in now if d not in was]
    if new:
        print(f"::error::spec class scan: {len(new)} new Markdown document(s) "
              f"under specs/. Prose in a .t27 file raises the parser's recovery "
              f"count without any parser or spec changing -- 36% of this "
              f"corpus's recovery events come from 16 such files, and the "
              f"number has been read as a parser backlog every wave (Prop. 197)")
        for d in new[:10]:
            print(f"  {d}")
        return 1
    fixed = [w for w in was if w not in now]
    if fixed:
        print(f"spec class scan: {len(fixed)} file(s) no longer document-shaped; "
              f"update {baseline.name} to lock it in")
    print(f"spec class scan: ratchet holds ({len(now)} <= {len(was)} documents)")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::spec class scan: could not scan specs/ "
              f"({type(exc).__name__}: {exc}) -- nothing was scanned")
        sys.exit(1)
