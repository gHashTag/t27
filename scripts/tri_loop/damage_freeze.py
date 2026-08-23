#!/usr/bin/env python3
"""tri damage-freeze -- freeze the corpus damage list before any repair touches it (#2154).

A repair that has no frozen "before" is not a repair, it is an edit. This writes an
archive that pins every damaged line to a stable class identifier, so a later patch
can cite the class ID and the snapshot digest instead of asserting that it changed
the right thing.

What is recorded per row, and why each field is needed:

  class_id     stable across runs and across changes to the class SET. Derived from
               sha256 of the normalised shape, NOT from frequency rank -- ranking
               by count means adding one damaged line silently renumbers every
               class, and every earlier citation of "class 04" then points at
               something else.
  file, line   location
  field        the field name whose type side is damaged
  rhs          the damaged type side, VERBATIM. This is the archive: no repair may
               delete a damaged line without this record existing first
  before/after two lines of context each side, so the struct the field belongs to
               is identifiable without reopening the file at a moved line number
  file_sha256  digest of the whole file at freeze time. A patch applied to a file
               whose digest no longer matches the snapshot is applied to a
               different file than the one that was surveyed, and must refuse

  corpus_sha256  digest over the sorted (file, sha256) pairs of every damaged file,
                 so the snapshot as a whole has one number to cite

Usage:
    tri damage-freeze [corpus-dir] [--out PATH]

This tool writes the snapshot only. It never edits a spec.
"""

import hashlib
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from damage import scan, shape  # noqa: E402  (same directory, deliberate)

CONTEXT = 2


def class_id(shape_text):
    """Stable ID for a damage class.

    Keyed on the shape text, so the ID of a class does not depend on how many
    lines happen to be in it, nor on which other classes exist.
    """
    return "DC-" + hashlib.sha256(shape_text.encode("utf-8")).hexdigest()[:8]


def file_digest(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def freeze(corpus):
    # `scan` returns the denominator too: zero rows over a path that does not
    # exist used to be indistinguishable from zero rows over a clean corpus.
    rows, scanned = scan(corpus)
    digests = {}
    cache = {}
    out = []
    for r in rows:
        path = r["file"]
        if path not in digests:
            digests[path] = file_digest(path)
            with open(path, "r", errors="replace") as fh:
                cache[path] = fh.read().splitlines()
        lines = cache[path]
        i = r["line"] - 1
        out.append({
            "class_id": class_id(r["shape"]),
            "shape": r["shape"],
            "file": path,
            "line": r["line"],
            "field": r["field"],
            "rhs": r["rhs"],
            "raw_line": lines[i] if 0 <= i < len(lines) else None,
            "context_before": lines[max(0, i - CONTEXT):i],
            "context_after": lines[i + 1:i + 1 + CONTEXT],
            "reasons": r["reasons"],
            "file_sha256": digests[path],
        })

    # One number for the snapshot as a whole. Sorted so it does not depend on
    # walk order.
    agg = hashlib.sha256()
    for path in sorted(digests):
        agg.update(f"{path}:{digests[path]}\n".encode("utf-8"))

    classes = {}
    for row in out:
        c = classes.setdefault(row["class_id"], {
            "class_id": row["class_id"], "shape": row["shape"],
            "count": 0, "files": set(), "reasons": set(),
        })
        c["count"] += 1
        c["files"].add(row["file"])
        c["reasons"].update(row["reasons"])
    for c in classes.values():
        c["files"] = sorted(c["files"])
        c["reasons"] = sorted(c["reasons"])

    return {
        "tool": "tri damage-freeze",
        "corpus": corpus,
        # The denominator, in the artifact and not only on the terminal.
        # `files` counts files WITH damage, so a snapshot of a clean corpus and
        # a snapshot of a path that does not exist had identical shapes and
        # identical zeros. Whoever reads this file later can now tell them
        # apart without re-running anything.
        "files_scanned": scanned,
        "lines": len(out),
        "files": len(digests),
        "classes": len(classes),
        "corpus_sha256": agg.hexdigest(),
        "file_sha256": {p: digests[p] for p in sorted(digests)},
        "class_index": [classes[k] for k in sorted(classes, key=lambda k: (-classes[k]["count"], k))],
        "rows": out,
    }


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
    out_path = None
    for i, a in enumerate(argv):
        if a == "--out" and i + 1 < len(argv):
            out_path = argv[i + 1]
    if out_path is None:
        out_path = "docs/corpus/damage_snapshot.json"

    snap = freeze(corpus)

    print(f"corpus:        {snap['corpus']}")
    print(f"damaged lines: {snap['lines']}")
    print(f"damaged files: {snap['files']}")
    print(f"classes:       {snap['classes']}")
    print(f"corpus_sha256: {snap['corpus_sha256']}\n")
    print(f"{'class_id':12} {'n':>4}  {'files':>5}  shape")
    print("-" * 74)
    for c in snap["class_index"]:
        print(f"{c['class_id']:12} {c['count']:4d}  {len(c['files']):5d}  {c['shape']!r}")

    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    with open(out_path, "w") as fh:
        json.dump(snap, fh, indent=1, sort_keys=True)
        fh.write("\n")
    print(f"\nwrote {out_path}")
    print("Class IDs are keyed on the shape text, so they survive a change to the")
    print("class set. Cite the class_id and corpus_sha256, never a rank.")
    # A snapshot of nothing is a snapshot, and it looks exactly like a
    # snapshot of a clean corpus: same shape, same zero counts. The file is
    # still written so the recorded `corpus` field says WHICH path was empty,
    # but the verdict is 2, matching cost, corpus-parse, diffbin and damage.
    if snap["files_scanned"] == 0:
        print()
        print(f"NOTHING WAS SCANNED under {corpus}.")
        print("This snapshot records the size of a corpus that was not there.")
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
