#!/usr/bin/env python3
"""tri diffbin -- compare two t27c binaries over a spec corpus, by category.

Why this exists, and why it is a rewrite rather than a restore.

An earlier version of this script was written, used to produce a "634 specs, 0
regressions" figure, and lost: it was never committed to any branch, and the
sandbox holding the working copy was re-cloned. Six recovery routes were checked
and all came back empty -- dangling git objects (only a `git stash` WIP holding
`triage.py`), the reflog (which shows the clone, i.e. the loss), shell history
(absent), CI artifacts (only FPGA build outputs), PR and issue comments (no
pasted source), and the session snapshot (prose about the script, not the
script). So this is a reimplementation from an explicit written contract, not a
reconstruction from memory of what the old one did.

That is the better outcome anyway, because the old aggregate was wrong.

The defect in the previous tool.

It reported "0 regressions" over 634 specs. It was also true that 17 files lost
struct fields: on input like `a : Map<K,` followed by `b : u8`, the candidate
absorbs the following `name : type` pairs into the first field's type text, so
three declared fields become one. Field loss was the tool's own stated regression
criterion. Both statements stood because a per-file judgement had relabelled the
loss as an acceptable tradeoff, and the aggregate then reported the judgement as
if it were a measurement.

The invariant this file exists to enforce:

    No differential result may be called "0 regressions" unless the metric
    actually checks the claimed class of loss.

So there is no single pass/fail number here. Every file lands in exactly one of
five categories, `field-loss` is never folded into another, and the exit status
is driven by `field-loss` and `unknown` counts rather than by a summary verdict.

The five categories, matched in this order:

  unchanged                identical field set, name for name and type for type
  field-loss               a field present in base is gone, and it was not a
                           phantom -- its base type text was non-empty
  strict-improvement       no loss, and either a phantom field disappeared or the
                           number of truncated type texts went down
  malformed-input-tradeoff no loss, the field set moved, and the input is
                           malformed, so no reading of it is correct
  unknown                  no loss, the field set moved, and the input is
                           well-formed -- this must not happen and needs a human

`field-loss` is tested before `strict-improvement` on purpose: a change that
removes a phantom and also drops a real field is a loss, not an improvement.

Phantom vs. real, which is the whole discrimination:

  A removed field is a PHANTOM if its type text in the base was EMPTY. That is
  the signature of an identifier lifted out of a type argument list and promoted
  to a struct field -- `a : Map<K, V;` made the old parser emit a field named `V`
  with no type. A removed field whose base type text was non-empty was declared
  by the author and its removal is a loss. The two look identical if you only
  count fields, which is how the old aggregate missed 17 files.

Usage:
    tri diffbin <base-binary> <candidate-binary> [corpus-dir] [--limit N]
                [--jsonl PATH] [--timeout SEC] [--include-scratch]

Exit status:
    0  no field-loss and no unknown
    1  at least one field-loss or unknown file
    2  usage or setup error
"""
import json
import os
import subprocess
import sys
from collections import Counter

CATEGORIES = (
    "unchanged",
    "field-loss",
    "strict-improvement",
    "malformed-input-tradeoff",
    "unknown",
)
DEFAULT_TIMEOUT = 25


def _unquote(line, key):
    t = line.strip()
    if not t.startswith(key):
        return None
    rest = t[len(key):].strip()
    if not rest.startswith('"'):
        return None
    rest = rest[1:]
    end = rest.rfind('"')
    if end < 0:
        return None
    return rest[:end]


def parse_fields(binary, path, timeout):
    """Return (status, [(qualified_name, type_text)]).

    status is one of ok / error / timeout. On error or timeout the field list is
    empty, and the categoriser treats a status change as its own outcome rather
    than reading an empty list as "all fields lost".

    Only an ExprIdentifier whose PARENT node is a StructDecl counts as a struct
    field. `t27c parse` prints the same node kind for every identifier in the
    tree, including every identifier in a function body, so matching on the kind
    alone would count expression operands as fields and make the corpus totals
    meaningless. Parenthood is read off the indentation of the `kind:` lines: the
    parent of a node is the nearest preceding `kind:` line with strictly smaller
    indentation. Names are qualified with the struct name, so that two structs in
    one file each declaring `value` stay distinguishable.
    """
    try:
        r = subprocess.run([binary, "parse", path], capture_output=True,
                           timeout=timeout, text=True, errors="replace")
    except subprocess.TimeoutExpired:
        return "timeout", []
    except OSError as e:
        return f"spawn-error: {e}", []
    if r.returncode != 0:
        return "error", []
    lines = (r.stdout + r.stderr).splitlines()
    out = []
    stack = []  # (indent, kind, name)
    for i, line in enumerate(lines):
        stripped = line.strip()
        if not stripped.startswith("kind: "):
            continue
        indent = len(line) - len(line.lstrip())
        kind = stripped[len("kind: "):].rstrip(",")
        name = _unquote(lines[i + 1], "name:") if i + 1 < len(lines) else None
        while stack and stack[-1][0] >= indent:
            stack.pop()
        parent = stack[-1] if stack else None
        if kind == "ExprIdentifier" and parent and parent[1] == "StructDecl":
            ty = _unquote(lines[i + 3], "extra_type:") if i + 3 < len(lines) else None
            if name is not None and ty is not None:
                out.append((f"{parent[2]}.{name}", ty))
        stack.append((indent, kind, name or ""))
    return "ok", out


def truncated(ty):
    """A type text with an unclosed bracket, angle or paren."""
    return (ty.count("<") > ty.count(">")
            or ty.count("(") > ty.count(")")
            or ty.count("[") > ty.count("]"))


def input_is_malformed(path):
    """Decide malformed-ness from the SOURCE, never from parser behaviour.

    Deciding it from the parser would make the categoriser circular: any change
    in output would justify itself as "the input must have been malformed".
    These are textual signals only, and each one is a thing that cannot appear in
    a well-formed field declaration:

      * an unbalanced `"` on a field line
      * `[[` immediately before `]`
      * a field line whose type side has more openers than closers
    """
    try:
        with open(path, "r", errors="replace") as fh:
            lines = fh.readlines()
    except OSError:
        return False, "unreadable"
    signals = []
    for n, line in enumerate(lines, 1):
        if ":" not in line:
            continue
        if line.count('"') % 2 == 1:
            signals.append(f"{n}:odd-quote")
            continue
        if "[[]" in line:
            signals.append(f"{n}:doubled-bracket")
            continue
        rhs = line.split(":", 1)[1]
        if truncated(rhs.rstrip().rstrip(",")):
            signals.append(f"{n}:unclosed-type")
    return bool(signals), ",".join(signals[:6])


def categorise(base, cand, path):
    """Return (category, reason, detail dict)."""
    bstat, bfields = base
    cstat, cfields = cand

    if bstat != cstat:
        if bstat == "ok" and cstat in ("error", "timeout"):
            return "field-loss", f"base parsed, candidate {cstat}", {}
        if bstat in ("error", "timeout") and cstat == "ok":
            return "strict-improvement", f"base {bstat}, candidate parsed", {}
        return "unknown", f"status {bstat} -> {cstat}", {}
    if bstat != "ok":
        return "unchanged", f"both {bstat}", {}

    if bfields == cfields:
        return "unchanged", "identical field set", {}

    bnames = [n for n, _ in bfields]
    cnames = [n for n, _ in cfields]
    btypes = dict(bfields)
    removed = [n for n in bnames if n not in cnames]
    added = [n for n in cnames if n not in bnames]

    phantoms = [n for n in removed if btypes.get(n, "") == ""]
    real_lost = [n for n in removed if btypes.get(n, "") != ""]

    detail = {"removed": removed, "added": added,
              "phantoms": phantoms, "real_lost": real_lost,
              "base_n": len(bfields), "cand_n": len(cfields),
              "base_trunc": sum(1 for _, t in bfields if truncated(t)),
              "cand_trunc": sum(1 for _, t in cfields if truncated(t))}

    if real_lost:
        return ("field-loss",
                f"declared field(s) gone with non-empty base type: {real_lost}",
                detail)

    if phantoms:
        return ("strict-improvement",
                f"phantom field(s) removed (empty base type): {phantoms}",
                detail)

    if detail["cand_trunc"] < detail["base_trunc"]:
        return ("strict-improvement",
                f"truncated type texts {detail['base_trunc']} -> {detail['cand_trunc']}",
                detail)

    malformed, signals = input_is_malformed(path)
    if malformed:
        return ("malformed-input-tradeoff",
                f"field set moved on malformed input ({signals})", detail)

    return ("unknown",
            "field set moved on well-formed input with no loss and no improvement",
            detail)


def main(argv):
    args = [a for a in argv if not a.startswith("--")]
    if len(args) < 2:
        print(__doc__.split("Usage:")[1].strip(), file=sys.stderr)
        return 2
    base_bin, cand_bin = args[0], args[1]
    corpus = args[2] if len(args) > 2 else "specs"
    for b in (base_bin, cand_bin):
        if not os.path.isfile(b) or not os.access(b, os.X_OK):
            print(f"not an executable: {b}", file=sys.stderr)
            return 2

    limit = None
    jsonl = None
    timeout = DEFAULT_TIMEOUT
    include_scratch = "--include-scratch" in argv
    for i, a in enumerate(argv):
        if a == "--limit" and i + 1 < len(argv):
            limit = int(argv[i + 1])
        if a == "--jsonl" and i + 1 < len(argv):
            jsonl = argv[i + 1]
        if a == "--timeout" and i + 1 < len(argv):
            timeout = int(argv[i + 1])

    files = []
    for root, _dirs, names in os.walk(corpus):
        if not include_scratch and f"{os.sep}scratch" in root:
            continue
        for n in sorted(names):
            if n.endswith(".t27"):
                files.append(os.path.join(root, n))
    files.sort()
    if limit:
        files = files[:limit]
    if not files:
        print(f"no .t27 files under {corpus}", file=sys.stderr)
        return 2

    counts = Counter()
    rows = []
    fh = open(jsonl, "w") if jsonl else None
    for path in files:
        b = parse_fields(base_bin, path, timeout)
        c = parse_fields(cand_bin, path, timeout)
        cat, reason, detail = categorise(b, c, path)
        counts[cat] += 1
        row = {"file": path, "category": cat, "reason": reason, **detail}
        rows.append(row)
        if fh:
            fh.write(json.dumps(row) + "\n")
            fh.flush()
    if fh:
        fh.close()

    print(f"corpus: {len(files)} specs under {corpus}"
          f"{'' if include_scratch else ' (scratch excluded)'}")
    print(f"base:      {base_bin}")
    print(f"candidate: {cand_bin}\n")
    for cat in CATEGORIES:
        print(f"  {counts.get(cat, 0):5d}  {cat}")

    for cat in ("field-loss", "unknown", "strict-improvement"):
        sel = [r for r in rows if r["category"] == cat]
        if not sel:
            continue
        print(f"\n{cat} ({len(sel)}):")
        for r in sel[:40]:
            print(f'  {r["file"]}')
            print(f'      {r["reason"]}')
        if len(sel) > 40:
            print(f"  ... {len(sel) - 40} more (use --jsonl for the full list)")

    lost = counts.get("field-loss", 0)
    unknown = counts.get("unknown", 0)
    print()
    if lost or unknown:
        print(f"NOT CLEAN: {lost} field-loss, {unknown} unknown.")
        print("These are NOT to be aggregated away. field-loss means a field the")
        print("author declared is gone from the parse. It may still be the right")
        print("trade to make -- that is a language decision about what malformed")
        print("input should mean -- but it is a decision, not a measurement, and")
        print("it does not belong inside a count of zero.")
    else:
        print("CLEAN: no field-loss and no unknown.")
        print("This says nothing about categories the tool does not check:")
        print("generated code, type inference, diagnostics, or timing.")
    return 1 if (lost or unknown) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
