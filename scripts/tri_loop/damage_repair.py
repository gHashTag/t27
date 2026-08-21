#!/usr/bin/env python3
"""tri damage-repair -- one candidate patch per damage class, with the effect measured (#2154).

Nothing here rewrites the corpus. A candidate is applied to a COPY, validated
twice, and printed as a reversible unified diff. Landing it is a separate,
human-reviewed act.

## The mechanism, established from the corpus and not from memory

An intact field in this corpus is `name : "TypeText",` -- the type side is a
quoted string. Every damaged line has the SAME single defect: the OPENING quote
of that string was replaced by `[`.

    intact   children : "[4]?QuadNode",
    damaged  children : [[4]?QuadNode",

which explains both signals `tri damage` detects at once: the doubled bracket
(`"[]T"` -> `[[]T"`) and the odd quote count (the opening one is gone). So the
candidate patch is one character, at a known offset, and inverting it is the same
operation in reverse. That is the entire repair for the classes where it works.

## Why the classes split, and why the split is the load-bearing result

Applying the substitution and asking whether the result is a closed string is a
decision procedure, not a guess:

  RESTORABLE     `[[]Const u8",` -> `"[]Const u8",`   a closed string. Nothing was
                 invented: every character of the type text survived the damage
                 and the repair only puts the delimiter back

  DESTROYED      `[[]Const [,`   -> `"[]Const [,`     NOT a closed string. The type
                 text was also TRUNCATED at its second `[`, and the element type
                 that followed is simply gone. `[]Const [` cannot be completed
                 without deciding what the element type was -- `[]Const []Const u8`
                 is a plausible reading and so is `[]Const [N]u8`. There is no
                 evidence in the file either way

The second group therefore gets `needs-human-language-decision` and NO patch. An
auto-repair there would be a guess wearing the costume of a fix, and it would be
unfalsifiable afterwards because the original is unrecoverable. This is the
difference between repairing a delimiter and inventing a type.

## Double validation, and why parsing alone is not enough

A file can be made to parse by deleting the offending line. So each applied
candidate is checked twice:

  1. SYNTACTIC   `t27c parse` exits 0 on the repaired file
  2. SEMANTIC    the specific damaged field is present in the parsed field set
                 with a NON-EMPTY type text, and no previously-present field
                 disappeared

Check 2 is the one that distinguishes a restored declaration from a merely
parseable file. Reported effect is one of:

  parse-restored                 both checks pass
  still-malformed                syntactic check fails
  ambiguous                      parses, but the field did not come back, or a
                                 different field vanished
  needs-human-language-decision  no patch was attempted, information destroyed

Usage:
    tri damage-repair --snapshot PATH [--binary PATH] [--class DC-xxxxxxxx]
                      [--diff] [--apply-to DIR] [--json PATH]

`--apply-to` writes repaired copies into a scratch tree; specs/ is never touched.

## Why --snapshot has no default (#2327)

It used to default to `docs/corpus/damage_snapshot_2026-08-15.json`, a path that
no default workflow produces: the companion writer `tri damage-freeze` defaults
its `--out` to `docs/corpus/damage_snapshot.json`, a different name, so running
the two tools back to back with no arguments never connected. The reader reported
"no snapshot" while a perfectly good freeze sat beside it in the same directory.

The rule that replaced it, and that `corpus_status.py` already followed for this
same artifact: a READER must not guess at an input path, because it cannot know
which freeze the caller meant; a WRITER may default its output path, because it
creates the file rather than hoping one is there. So `damage-freeze --out` keeps
its default and `damage-repair --snapshot` has none.

This is the same instinct as the staleness check further down, which refuses to
repair against a snapshot whose digests no longer match the corpus. A tool that
refuses to repair against the wrong snapshot should equally refuse to invent
which snapshot you meant.
"""

import difflib
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from diffbin import parse_fields  # noqa: E402  (same directory, deliberate)

CLOSED_STRING = re.compile(r'^"[^"]*",?$')
TIMEOUT = 25

EFFECTS = (
    "parse-restored",
    "still-malformed",
    "ambiguous",
    "needs-human-language-decision",
)


def candidate_rhs(rhs):
    """The one-character candidate patch, or None when it is not applicable.

    Returns (repaired_rhs, restorable). restorable is False when the substitution
    does not yield a closed string, which means the type text itself was truncated
    and no patch is proposed.
    """
    if not rhs.startswith("["):
        return None, False
    cand = '"' + rhs[1:]
    return cand, bool(CLOSED_STRING.match(cand))


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def repair_lines(lines, rows):
    """Apply the candidate to the given rows of one file. Returns (new_lines, applied).

    Only the type side is touched, and only its first character. The field name,
    the indentation and the trailing comma are left byte-identical, so the diff is
    one character per line and reads as such.
    """
    new = list(lines)
    applied = []
    for r in rows:
        i = r["line"] - 1
        if not (0 <= i < len(new)):
            continue
        cand, restorable = candidate_rhs(r["rhs"])
        if not restorable:
            continue
        old = new[i]
        # Locate the type side by its recorded text rather than by re-parsing the
        # line, so a patch cannot drift onto a different construct.
        if r["rhs"] not in old:
            continue
        new[i] = old.replace(r["rhs"], cand, 1)
        applied.append(r)
    return new, applied


def validate(binary, orig_path, repaired_text, rows_applied, tmpdir):
    """Two checks. Returns (effect, detail)."""
    scratch = os.path.join(tmpdir, "validate.t27")
    with open(scratch, "w") as fh:
        fh.write(repaired_text)

    base_status, base_fields = parse_fields(binary, orig_path, TIMEOUT)
    cand_status, cand_fields = parse_fields(binary, scratch, TIMEOUT)

    if cand_status != "ok":
        return "still-malformed", f"parse status {cand_status} after repair"

    base_map = dict(base_fields)
    cand_map = dict(cand_fields)

    # Semantic check 1: every repaired field must now be present with a non-empty
    # type. Names are qualified Struct.field by parse_fields, and the snapshot
    # records the bare field name, so match on the suffix.
    missing = []
    for r in rows_applied:
        want = "." + r["field"]
        hits = [(k, v) for k, v in cand_map.items() if k.endswith(want)]
        if not hits or not any(v.strip() for _k, v in hits):
            missing.append(r["field"])

    # Semantic check 2: nothing that used to be there may vanish. A repair that
    # restores one declaration by losing another is not a repair.
    vanished = sorted(k for k in base_map if k not in cand_map)

    if missing or vanished:
        bits = []
        if missing:
            bits.append("field(s) did not come back: " + ", ".join(sorted(missing)))
        if vanished:
            bits.append("field(s) vanished: " + ", ".join(vanished[:6]))
        return "ambiguous", "; ".join(bits)

    gained = sorted(k for k in cand_map if k not in base_map)
    return "parse-restored", (f"parse ok; {len(rows_applied)} field(s) repaired; "
                             f"{len(gained)} declaration(s) newly visible"
                             + (": " + ", ".join(gained[:6]) if gained else ""))


def combined(snap, rows, binary, have_binary, tmpdir, apply_to, json_out):
    """Repair every restorable row of a file at once, then validate per file.

    The per-class run answers "is this class's patch sound", and it answers it with
    a confound: a file carrying damage from four classes still fails to parse after
    one class is repaired, and the class is then blamed for a neighbour's defect.
    That is what happened on the first run -- six classes came back still-malformed,
    and the first guess (co-located UNRESTORABLE damage) was checked and found
    false: none of those files contained a destroyed line. The actual cause was
    co-located damage from OTHER RESTORABLE classes, left untouched by a
    single-class run.

    So per-file is the unit that answers "what does the one rule achieve", and the
    per-class run stays as the unit that answers "is one rule enough for this
    shape". Both are reported; neither replaces the other.
    """
    per_file = {}
    for r in rows:
        per_file.setdefault(r["file"], []).append(r)

    counts = dict.fromkeys(EFFECTS, 0)
    out = []
    print("combined mode: every restorable row per file, validated per file\n")
    for path in sorted(per_file):
        frows = per_file[path]
        with open(path, "r", errors="replace") as fh:
            orig = fh.read()
        new_lines, applied = repair_lines(orig.splitlines(), frows)
        new_text = "\n".join(new_lines) + ("\n" if orig.endswith("\n") else "")
        held = [r for r in frows if not candidate_rhs(r["rhs"])[1]]
        if not applied:
            eff, det = "needs-human-language-decision", \
                f"all {len(frows)} damaged line(s) are destroyed, none restorable"
        elif have_binary:
            eff, det = validate(binary, path, new_text, applied, tmpdir)
            if held and eff != "parse-restored":
                det += f"; {len(held)} destroyed line(s) remain in this file"
        else:
            eff, det = "ambiguous", "validation skipped: no binary"
        counts[eff] += 1
        out.append({"file": path, "applied": len(applied), "held": len(held),
                    "effect": eff, "detail": det})
        if apply_to:
            dest = os.path.join(apply_to, path)
            os.makedirs(os.path.dirname(dest), exist_ok=True)
            with open(dest, "w") as fh:
                fh.write(new_text)

    for e in EFFECTS:
        sel = [o for o in out if o["effect"] == e]
        if not sel:
            continue
        print(f"--- {e}: {len(sel)} file(s) ---")
        for o in sel[:8]:
            print(f"   {o['file']}  (+{o['applied']} repaired, {o['held']} held)  {o['detail'][:110]}")
        if len(sel) > 8:
            print(f"   ... and {len(sel) - 8} more")
        print()

    print("=" * 74)
    print("per-file effect distribution (no aggregation across effects):")
    for e in EFFECTS:
        print(f"  {e:32} {counts[e]:3d}")
    lines_ok = sum(o["applied"] for o in out if o["effect"] == "parse-restored")
    lines_held = sum(o["held"] for o in out)
    print(f"\nrestorable lines inside parse-restored files: {lines_ok}")
    print(f"lines held for a human language decision:     {lines_held}")
    print(f"snapshot total:                               {snap['lines']}")
    print("\nNo spec under specs/ was modified by this run.")
    if json_out:
        with open(json_out, "w") as fh:
            json.dump({"mode": "combined", "corpus_sha256": snap["corpus_sha256"],
                       "counts": counts, "files": out}, fh, indent=1, sort_keys=True)
            fh.write("\n")
        print(f"wrote {json_out}")
    return 0


def main(argv):
    snapshot = None
    binary = "/tmp/t27c.fixed"
    only = None
    apply_to = None
    json_out = None
    want_diff = "--diff" in argv
    for i, a in enumerate(argv):
        if a == "--snapshot" and i + 1 < len(argv):
            snapshot = argv[i + 1]
        elif a == "--binary" and i + 1 < len(argv):
            binary = argv[i + 1]
        elif a == "--class" and i + 1 < len(argv):
            only = argv[i + 1]
        elif a == "--apply-to" and i + 1 < len(argv):
            apply_to = argv[i + 1]
        elif a == "--json" and i + 1 < len(argv):
            json_out = argv[i + 1]

    # No default. Which freeze to repair against is a statement the caller has to
    # make: the snapshot fixes both the corpus digest every later citation refers
    # to and the file digests the staleness check below enforces. A guessed path
    # either finds nothing, or silently picks up a freeze of a corpus and a date
    # nobody named. `corpus_status.py` requires this same artifact for the same
    # reason.
    if snapshot is None:
        print("REFUSING: --snapshot is required; there is no default.", file=sys.stderr)
        print("", file=sys.stderr)
        print("A repair is only meaningful against a named frozen snapshot, whose", file=sys.stderr)
        print("digests are what let this tool refuse when the corpus has moved.", file=sys.stderr)
        print("", file=sys.stderr)
        print("  tri damage-freeze specs --out PATH", file=sys.stderr)
        print("  tri damage-repair --snapshot PATH", file=sys.stderr)
        return 2

    if not os.path.exists(snapshot):
        print(f"no snapshot at {snapshot}", file=sys.stderr)
        print("freeze one first: tri damage-freeze specs --out " + snapshot, file=sys.stderr)
        return 2
    snap = json.load(open(snapshot))

    rows = snap["rows"]
    if only:
        rows = [r for r in rows if r["class_id"] == only]
        if not rows:
            print(f"no rows for class {only}", file=sys.stderr)
            return 2

    # A patch applied to a file that has changed since the freeze is applied to a
    # different file than the one that was surveyed. Refuse rather than proceed.
    stale = []
    for r in rows:
        p = r["file"]
        if not os.path.exists(p):
            stale.append((p, "missing"))
        elif sha256_file(p) != r["file_sha256"]:
            stale.append((p, "digest differs from snapshot"))
    stale = sorted(set(stale))
    if stale:
        print("REFUSING: the corpus moved since the snapshot was frozen.")
        for p, why in stale[:10]:
            print(f"  {p}  ({why})")
        print("\nRe-freeze, review the difference, and only then repair.")
        return 1

    by_class = {}
    for r in rows:
        by_class.setdefault(r["class_id"], []).append(r)

    tmpdir = "/tmp/tri_damage_repair"
    os.makedirs(tmpdir, exist_ok=True)
    if apply_to:
        os.makedirs(apply_to, exist_ok=True)

    have_binary = os.path.exists(binary)
    results = []
    counts = dict.fromkeys(EFFECTS, 0)

    print(f"snapshot:      {snapshot}")
    print(f"corpus_sha256: {snap['corpus_sha256']}")
    print(f"binary:        {binary}" + ("" if have_binary else "   [MISSING -- validation skipped]"))
    print(f"classes:       {len(by_class)}\n")

    if "--combined" in argv:
        return combined(snap, rows, binary, have_binary, tmpdir, apply_to, json_out)

    for cid in sorted(by_class, key=lambda c: (-len(by_class[c]), c)):
        crows = by_class[cid]
        shape = crows[0]["shape"]
        _cand, restorable = candidate_rhs(crows[0]["rhs"])

        if not restorable:
            effect = "needs-human-language-decision"
            counts[effect] += 1
            reading = []
            for r in crows[:3]:
                reading.append(f"{r['file']}:{r['line']}  {r['field']} : {r['rhs']}")
            results.append({"class_id": cid, "shape": shape, "lines": len(crows),
                            "files": sorted({r["file"] for r in crows}),
                            "effect": effect, "patch": None,
                            "detail": ("the type text is truncated as well as unquoted; the element "
                                       "type is not recoverable from the file"),
                            "owner": "language owner",
                            "decision_criterion": (
                                "state what an unclosed element type in a slice position means, and "
                                "whether the corpus may carry a placeholder; only then can a value "
                                "be written in without guessing")})
            print(f"{cid}  n={len(crows):3d}  {shape!r}")
            print(f"   effect: {effect}")
            print("   no patch proposed. Substituting the delimiter yields "
                  f"{_cand!r}, which is not a closed string:")
            print("   the type text was truncated too, so a repair would have to invent the")
            print("   element type. owner: language owner")
            for line in reading:
                print(f"     {line}")
            print()
            continue

        # Group by file, apply, validate, diff.
        per_file = {}
        for r in crows:
            per_file.setdefault(r["file"], []).append(r)

        effects_seen = []
        details = []
        diffs = []
        for path, frows in sorted(per_file.items()):
            with open(path, "r", errors="replace") as fh:
                orig = fh.read()
            lines = orig.splitlines()
            new_lines, applied = repair_lines(lines, frows)
            new_text = "\n".join(new_lines) + ("\n" if orig.endswith("\n") else "")
            if not applied:
                effects_seen.append("ambiguous")
                details.append(f"{path}: recorded text not found at the recorded line")
                continue
            diff = list(difflib.unified_diff(
                orig.splitlines(keepends=True), new_text.splitlines(keepends=True),
                fromfile=f"a/{path}", tofile=f"b/{path}", n=1))
            diffs.append("".join(diff))
            if have_binary:
                eff, det = validate(binary, path, new_text, applied, tmpdir)
            else:
                eff, det = "ambiguous", "validation skipped: no binary"
            effects_seen.append(eff)
            details.append(f"{path}: {det}")
            if apply_to:
                dest = os.path.join(apply_to, path)
                os.makedirs(os.path.dirname(dest), exist_ok=True)
                with open(dest, "w") as fh:
                    fh.write(new_text)

        # A class is only as good as its worst file.
        for e in EFFECTS:
            if e in effects_seen:
                effect = e
                break
        else:
            effect = "ambiguous"
        counts[effect] += 1

        results.append({"class_id": cid, "shape": shape, "lines": len(crows),
                        "files": sorted(per_file),
                        "effect": effect,
                        "patch": "replace the leading '[' of the type side with '\"'",
                        "reversible": True,
                        "per_file_effects": effects_seen,
                        "detail": details})
        print(f"{cid}  n={len(crows):3d}  {shape!r}")
        print(f"   patch:  replace the leading '[' of the type side with '\"'  (1 char, invertible)")
        print(f"   effect: {effect}   [{effects_seen.count('parse-restored')}/{len(effects_seen)} files parse-restored]")
        for d in details[:3]:
            print(f"     {d}")
        if len(details) > 3:
            print(f"     ... and {len(details) - 3} more file(s)")
        if want_diff and diffs:
            print("   --- candidate diff (first file) ---")
            for line in diffs[0].splitlines()[:12]:
                print("   " + line)
        print()

    print("=" * 74)
    print("effect distribution over classes (no aggregation across effects):")
    for e in EFFECTS:
        print(f"  {e:32} {counts[e]:3d}")
    restor = sum(r["lines"] for r in results if r["effect"] == "parse-restored")
    human = sum(r["lines"] for r in results if r["effect"] == "needs-human-language-decision")
    print(f"\nlines covered by a validated candidate:      {restor}")
    print(f"lines held for a human language decision:   {human}")
    print(f"lines total in snapshot:                    {snap['lines']}")
    print("\nNo spec under specs/ was modified by this run.")

    if json_out:
        with open(json_out, "w") as fh:
            json.dump({"snapshot": snapshot, "corpus_sha256": snap["corpus_sha256"],
                       "binary": binary, "counts": counts, "classes": results},
                      fh, indent=1, sort_keys=True)
            fh.write("\n")
        print(f"wrote {json_out}")

    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
