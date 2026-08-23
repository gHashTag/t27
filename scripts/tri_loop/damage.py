#!/usr/bin/env python3
"""tri damage -- classify the mangled type annotations in the spec corpus (#2154).

What this is for.

A rewrite left behind lines whose type side is not a type: `public_key : [[]U8",`
instead of `public_key : U8`. Those lines are the floor under every field-set
measurement taken on this corpus -- 16 of the 18 files whose field sets moved
between two compiler binaries are in this set, which means the measurement was
reading the damage and not the parser.

This tool only reports. It rewrites nothing. The point of a classifier is that a
later repair can be an inspectable diff of a stated size per class, instead of
one sweep over 115 lines that nobody can review.

Class is decided by the shape of the type side after normalising every run of
identifier characters to `X` and every digit run to `9`, so `[[]U8",` and
`[[]Bool",` are the same class and get counted once.

Usage:
    tri damage [corpus-dir] [--json PATH] [--emit-fixtures DIR] [--class SHAPE]

Exit status:
    0  no damaged lines found
    1  damaged lines found
"""
import json
import os
import re
import sys

IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
DIGITS = re.compile(r"[0-9]+")
FIELD_LINE = re.compile(r"^\s*(?:pub\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(.*)$")
# `r#"` opens a raw string that legitimately closes on a later line, so its lone
# quote is not damage. Found by reading the 3 lines the odd-quote signal flagged
# in specs/pins/parser.t27 rather than by trusting the signal.
RAW_STRING_OPEN = re.compile(r'\br#"')


def shape(rhs):
    """Normalise a type side to its shape: identifiers -> X, digits -> 9."""
    s = IDENT.sub("X", rhs)
    s = DIGITS.sub("9", s)
    s = re.sub(r"\s+", " ", s).strip()
    return s


def is_damaged(rhs):
    """A type side that cannot be a type, with the reason.

    Two signals only, and the history of the third, fourth and fifth is the point
    of this comment.

    The first draft of this function also flagged an unbalanced `<` or `>` and an
    unbalanced `[`. Run over the corpus it reported 429 damaged lines. 230 of
    those were `target : < 5000ns` -- a legitimate constraint bound where `<` is
    the less-than operator, not a bracket. Another 17 were match arms
    (`ConstraintType : :TEMPORAL => {`), 10 were the opening line of a multi-line
    array literal, and 6 were function signatures that the field-line regex had
    no business matching. So the headline 429 measured this function's regex, not
    the corpus, and dropping the two bad signals is the fix -- not tuning a
    threshold until the number looks right.

    What survives are the two signals that cannot occur in any well-formed type
    annotation and cannot occur in a legitimate operator either:

      doubled-bracket  `[[]` -- an empty slice inside a slice, which the language
                       has no syntax for
      odd-quote        an odd number of `"` on the line, so a string opens and
                       never closes

    Both are textual and never consult the parser: asking the parser whether its
    own input was malformed is circular, since the malformed reading is exactly
    what is in question.
    """
    reasons = []
    if "[[]" in rhs or "[?[]" in rhs:
        reasons.append("doubled-bracket")
    if rhs.count('"') % 2 == 1 and not RAW_STRING_OPEN.search(rhs):
        reasons.append("odd-quote")
    return reasons


def scan(corpus):
    """Return (rows, files_scanned).

    The count is returned, not just the rows, because zero rows has two
    meanings and the output could not tell them apart: over the 650-spec
    corpus and over a directory that does not exist, this printed the SAME
    line -- `damaged lines: 0 in 0 files, 0 distinct shapes`. The "0 files"
    there counts files WITH damage, so a scan of nothing and a scan that found
    nothing read identically.

    That matters most where it is used as a negative control. A fixture set
    saying "tri damage must report zero here" passes vacuously if the path is
    wrong, misspelled, or never merged -- and the fixtures for this tool were
    in fact never merged (#2161), so the control has never run.
    """
    rows = []
    scanned = 0
    for root, _dirs, names in os.walk(corpus):
        for n in sorted(names):
            if not n.endswith(".t27"):
                continue
            path = os.path.join(root, n)
            try:
                with open(path, "r", errors="replace") as fh:
                    lines = fh.readlines()
            except OSError:
                continue
            scanned += 1
            for ln, line in enumerate(lines, 1):
                m = FIELD_LINE.match(line.rstrip("\n"))
                if not m:
                    continue
                field, rhs = m.group(1), m.group(2).strip()
                if not rhs:
                    continue
                reasons = is_damaged(rhs)
                if reasons:
                    rows.append({"file": path, "line": ln, "field": field,
                                 "rhs": rhs, "shape": shape(rhs),
                                 "reasons": reasons,
                                 "family": os.path.dirname(path)})
    return rows, scanned


# Flags that take a value. `[a for a in argv if not a.startswith("--")]` strips
# the FLAG and leaves its VALUE, so `tri damage --json /tmp/x.json` read
# /tmp/x.json as the corpus directory and scanned nothing. With the file count
# absent from the output, that printed exactly what a clean 650-spec corpus
# prints. Two defects, and each one hid the other.
VALUE_FLAGS = ("--json", "--emit-fixtures", "--class", "--out", "--snapshot")


def positionals(argv):
    """argv minus flags and minus the values those flags consume."""
    out, skip = [], False
    for a in argv:
        if skip:
            skip = False
            continue
        if a in VALUE_FLAGS:
            skip = True
            continue
        if a.startswith("--"):
            continue
        out.append(a)
    return out


def main(argv):
    args = positionals(argv)
    corpus = args[0] if args else "specs"
    out_json = None
    emit = None
    only = None
    for i, a in enumerate(argv):
        if a == "--json" and i + 1 < len(argv):
            out_json = argv[i + 1]
        if a == "--emit-fixtures" and i + 1 < len(argv):
            emit = argv[i + 1]
        if a == "--class" and i + 1 < len(argv):
            only = argv[i + 1]

    rows, scanned = scan(corpus)
    if only:
        rows = [r for r in rows if r["shape"] == only]

    files = sorted({r["file"] for r in rows})
    shapes = {}
    for r in rows:
        shapes.setdefault(r["shape"], []).append(r)
    fams = {}
    for r in rows:
        fams[r["family"]] = fams.get(r["family"], 0) + 1

    print(f"corpus: {corpus}")
    print(f"files scanned: {scanned}")
    empty = scanned == 0
    if empty:
        print("NOTHING WAS SCANNED. A zero below is the absence of a corpus,")
        print("not the absence of damage. Check the path.")
    print(f"damaged lines: {len(rows)} in {len(files)} files, "
          f"{len(shapes)} distinct shapes\n")
    print("by shape (this is the repair unit -- one reviewable diff per row):")
    for sh, rs in sorted(shapes.items(), key=lambda kv: -len(kv[1])):
        reasons = sorted({x for r in rs for x in r["reasons"]})
        print(f"  {len(rs):5d}  {sh!r:28s} {','.join(reasons)}")
        print(f"         e.g. {rs[0]['file']}:{rs[0]['line']}  "
              f"{rs[0]['field']} : {rs[0]['rhs']}")
    print("\nby directory (top 12):")
    for fam, n in sorted(fams.items(), key=lambda kv: -kv[1])[:12]:
        print(f"  {n:5d}  {fam}")

    if out_json:
        with open(out_json, "w") as fh:
            json.dump({"corpus": corpus, "lines": len(rows),
                       "files": files, "shapes": {k: len(v) for k, v in shapes.items()},
                       "rows": rows}, fh, indent=2)
        print(f"\nwrote {out_json}")

    if emit:
        os.makedirs(emit, exist_ok=True)
        made = []
        for idx, (sh, rs) in enumerate(
                sorted(shapes.items(), key=lambda kv: -len(kv[1])), 1):
            r = rs[0]
            name = f"damage_class_{idx:02d}"
            body = (f"module {name}\n\n"
                    f"// shape: {sh}\n"
                    f"// {len(rs)} line(s) in the corpus share this shape\n"
                    f"// first seen: {r['file']}:{r['line']}\n"
                    f"// signals: {','.join(r['reasons'])}\n"
                    f"pub struct Damaged {{\n"
                    f"    {r['field']} : {r['rhs']}\n"
                    f"}}\n")
            p = os.path.join(emit, f"{name}.t27")
            with open(p, "w") as fh:
                fh.write(body)
            made.append((p, sh, len(rs)))
        print(f"\nemitted {len(made)} class fixtures into {emit}:")
        for p, sh, n in made:
            print(f"  {p}  ({n} lines, shape {sh!r})")

    print("\nNot claimed: that these are all the damage. This checks two textual")
    print("signals on lines that look like field declarations. Damage that")
    print("produces a balanced, quote-even, plausible-looking wrong type is")
    print("invisible here, and nothing in this output bounds how much of that")
    print("there is.")
    # An empty scan is neither "damage found" nor "corpus clean". Code 2 matches
    # `cost`, `corpus-parse` and `diffbin`, which all refuse an empty corpus.
    # Printing NOTHING WAS SCANNED and then exiting 0 leaves the CI step green
    # with the warning on the screen, which is where warnings go to be ignored.
    if empty:
        return 2
    return 1 if rows else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
