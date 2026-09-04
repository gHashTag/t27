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

  not-evaluated            neither binary reached a verdict, or the caller held
                           the file out. Always carries a reason code. This is
                           NOT agreement: it is the absence of a measurement, and
                           collapsing it into `unchanged` is what made the old
                           aggregate report 52 % coverage as 100 %.

Usage:
    tri diffbin <base-binary> <candidate-binary> [corpus-dir] [--limit N]
                [--jsonl PATH] [--timeout SEC] [--include-scratch]
                [--exclude-status STATUS.json]

Exit status:
    0  no field-loss and no unknown (coverage is printed, not folded in)
    1  at least one field-loss or unknown file
    2  usage or setup error, or an internal partition/reason-code violation
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
    "not-evaluated",
)

# `not-evaluated` exists because `unchanged` was carrying two different
# statements. Measured 2026-08-15 over the 634-spec library: of 616 files called
# `unchanged`, only 330 were 'both parsed, field sets identical' -- the other 286
# were 'both error', i.e. neither binary produced a verdict. The historical
# headline '634 specs, 0 regressions' therefore rested on a measured base of 330
# files, 52 % of the corpus, with the remaining 48 % counted as agreement because
# both sides were silent.
#
# That is not a reporting blemish. It changes the meaning of every differential
# total ever quoted from this tool: an absence of evidence was printed as evidence
# of absence.
#
# Every not-evaluated file MUST carry one of these codes. A bare `not-evaluated`
# count is not admissible -- 'no verdict' for six different reasons is six
# different facts, and only some of them are about the compiler at all.
NOT_EVALUATED_REASONS = (
    "both-error",            # neither binary parsed the file; no verdict exists
    "base-timeout",          # baseline hit the wall clock; a property of the run
    "candidate-timeout",     # candidate hit the wall clock; likewise
    "environment-failure",   # the binary could not be spawned at all
    "excluded-source-loss",  # caller excluded it: the declaring text is gone
    "other",                 # must be accompanied by explanatory text
)

# A timeout is a property of the RUN, not of the file (LOOP-RULES R14). Measured
# 2026-08-15 on specs/scratch at a 12 s threshold: 26 files moved ok -> timeout
# between two runs and 0 moved back, and all 26 already sat between 9322 ms and
# 11952 ms against a 12000 ms wall. So the shift measured machine load at the
# boundary, not the compiler. Timeouts are kept out of every substantive category
# for that reason, and two runs at different thresholds must never be joined.
DEFAULT_TIMEOUT = 25


def path_key(p):
    """Normalise a path to its repo-relative tail from `specs/`.

    Exclusion lists come from `tri corpus-status`, which may have been produced
    in a different worktree or against a repaired tree. Joining on the raw string
    would silently fail to exclude anything, and a silent non-exclusion is the
    worst outcome here: the caller would believe files were held out when they
    were not.
    """
    p = p.replace("\\", "/")
    i = p.find("specs/")
    return p[i:] if i >= 0 else p


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


def categorise(base, cand, path, excluded=None):
    """Return (category, reason, detail dict).

    The six categories are mutually exclusive and exhaust the corpus by
    construction: every file leaves this function with exactly one of them, and
    the caller asserts that the counts sum to the file total.

    Order of decision, and why it is this order:

      1. caller-declared exclusion  the declaring text is gone from the source, so
                                    nothing about the compiler can be read from
                                    the file in either direction
      2. spawn failure              the tool did not run; not a result
      3. any timeout                a property of the run, not of the file (R14)
      4. both failed to parse       neither side took a verdict (R13)
      5. one-sided parse change     a real behavioural difference
      6. field-set comparison       the substantive comparison

    Exclusion is tested first on purpose. Testing it later would allow a file to
    be reported as a compiler regression when the field it appears to have lost
    was never present in the source the compiler read.
    """
    bstat, bfields = base
    cstat, cfields = cand

    if excluded and path_key(path) in excluded:
        return ("not-evaluated",
                "excluded-source-loss: caller excluded this file, the declaring "
                "text is absent from the source", {})

    if bstat.startswith("spawn-error") or cstat.startswith("spawn-error"):
        return ("not-evaluated",
                f"environment-failure: base={bstat} candidate={cstat}", {})

    # Each side gets its own timeout code. Folding them together would hide which
    # binary hit the wall, and that is the only part of a timeout that could ever
    # point at a real slowdown.
    if bstat == "timeout" and cstat == "timeout":
        return ("not-evaluated",
                "base-timeout: both binaries hit the wall clock "
                "(candidate-timeout applies equally)", {})
    if bstat == "timeout":
        return ("not-evaluated",
                f"base-timeout: baseline hit the wall clock, candidate={cstat}",
                {})
    if cstat == "timeout":
        return ("not-evaluated",
                f"candidate-timeout: candidate hit the wall clock, base={bstat}. "
                "Not a field loss, and not evidence of a slowdown by itself: "
                "check the baseline time against the threshold first (R14)", {})

    if bstat == "error" and cstat == "error":
        return ("not-evaluated",
                "both-error: neither binary parsed the file, so no verdict on "
                "field behaviour exists. Repairing such a file moves it out of "
                "this bucket, and a pre-existing divergence then becomes visible "
                "for the first time -- that is not a new regression (R13)", {})

    if bstat != cstat:
        if bstat == "ok" and cstat == "error":
            # A refusal is an answer, not a missing verdict, and it makes every
            # declared field unavailable. So this stays a loss.
            return ("field-loss",
                    "base parsed, candidate refused the file: every declared "
                    "field is unavailable", {})
        if bstat == "error" and cstat == "ok":
            return "strict-improvement", "base error, candidate parsed", {}
        return "unknown", f"status {bstat} -> {cstat}", {}

    if bfields == cfields:
        return "unchanged", "identical field set (both parsed)", {}

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
    exclude_file = None
    include_scratch = "--include-scratch" in argv
    for i, a in enumerate(argv):
        if a == "--limit" and i + 1 < len(argv):
            limit = int(argv[i + 1])
        if a == "--jsonl" and i + 1 < len(argv):
            jsonl = argv[i + 1]
        if a == "--timeout" and i + 1 < len(argv):
            timeout = int(argv[i + 1])
        if a == "--exclude-status" and i + 1 < len(argv):
            exclude_file = argv[i + 1]

    # An exclusion list must be declared, never inferred. The file is a
    # `tri corpus-status --out` JSON; files whose status is
    # `unrecoverable-source-loss` are held out under the reason code
    # `excluded-source-loss`. They are still counted and still printed -- holding
    # a file out of the substantive comparison is not the same as dropping it.
    excluded = set()
    if exclude_file:
        try:
            with open(exclude_file) as fh:
                st = json.load(fh)
            for r in st.get("rows", []):
                if r.get("status") == "unrecoverable-source-loss":
                    excluded.add(path_key(r["path"]))
        except (OSError, ValueError, KeyError) as e:
            print(f"cannot read --exclude-status {exclude_file}: {e}",
                  file=sys.stderr)
            return 2
        if not excluded:
            # Refuse to proceed quietly. An exclusion list that excludes nothing
            # is almost always a path-join failure, and it would produce a report
            # claiming files were held out when none were.
            print(f"--exclude-status {exclude_file} selected 0 files; refusing to "
                  f"report an exclusion that excludes nothing", file=sys.stderr)
            return 2

    files = []
    for root, _dirs, names in os.walk(corpus):
        if not include_scratch and f"{os.sep}scratch" in root:
            continue
        for n in sorted(names):
            if n.endswith(".t27"):
                files.append(os.path.join(root, n))
    files.sort()
    # The corpus is what is THERE. `--limit` selects a sample from it, and every
    # number printed below is about the sample -- so the corpus size has to be
    # captured before the truncation, or the report has no way to say which of
    # the two it is talking about.
    #
    # It did not, and said "of the corpus" over the sample. Measured 2026-09-05:
    # `--limit 10` over `specs` printed "corpus: 10 specs under specs" while 650
    # .t27 files were there, and "MEASURED COVERAGE: m/10 = p% of the corpus".
    # A 2% sample could print 100% coverage, under a paragraph that reads
    # "Coverage below 100% bounds what the run can claim" -- so the one number
    # that bounds the claim was the one the truncation had already destroyed.
    corpus_total = len(files)
    if limit:
        files = files[:limit]
    sampled = len(files) < corpus_total
    if not files:
        print(f"no .t27 files under {corpus}", file=sys.stderr)
        return 2

    counts = Counter()
    ne_codes = Counter()
    rows = []
    fh = open(jsonl, "w") if jsonl else None
    for path in files:
        b = parse_fields(base_bin, path, timeout)
        c = parse_fields(cand_bin, path, timeout)
        cat, reason, detail = categorise(b, c, path, excluded)
        counts[cat] += 1
        if cat == "not-evaluated":
            code = reason.split(":", 1)[0]
            if code not in NOT_EVALUATED_REASONS:
                # Fail loudly rather than emit an uncoded not-evaluated row. An
                # uncoded row is exactly the ambiguity this category was created
                # to remove.
                print(f"internal: not-evaluated without a declared reason code "
                      f"({code!r}) on {path}", file=sys.stderr)
                return 2
            ne_codes[code] += 1
        row = {"file": path, "category": cat, "reason": reason,
               "reason_code": (reason.split(":", 1)[0]
                               if cat == "not-evaluated" else None),
               **detail}
        rows.append(row)
        if fh:
            fh.write(json.dumps(row) + "\n")
            fh.flush()
    if fh:
        fh.close()

    if sampled:
        print(f"sample: {len(files)} of {corpus_total} specs under {corpus}"
              f"{'' if include_scratch else ' (scratch excluded)'}"
              f"  [--limit {limit}]")
    else:
        print(f"corpus: {corpus_total} specs under {corpus}"
              f"{'' if include_scratch else ' (scratch excluded)'}")
    print(f"base:      {base_bin}")
    print(f"candidate: {cand_bin}\n")
    total = len(files)
    ne = counts.get("not-evaluated", 0)
    measured = total - ne

    # Partition assertion. The six categories are claimed to be mutually
    # exclusive and exhaustive; a claim of that shape must be checked rather than
    # asserted in a docstring.
    s = sum(counts.get(c, 0) for c in CATEGORIES)
    if s != total:
        print(f"internal: categories sum to {s} but {total} file(s) were compared",
              file=sys.stderr)
        return 2

    for cat in CATEGORIES:
        print(f"  {counts.get(cat, 0):5d}  {cat}")

    if ne:
        print("\n  not-evaluated by reason code (each is a different fact):")
        for code in NOT_EVALUATED_REASONS:
            if ne_codes.get(code):
                print(f"    {ne_codes[code]:5d}  {code}")

    pct = (100.0 * measured / total) if total else 0.0
    scope = "the corpus" if not sampled else f"the {total} compared"
    print(f"\nMEASURED COVERAGE: {measured}/{total} = {pct:.1f}% of {scope}")
    if sampled:
        frac = 100.0 * total / corpus_total if corpus_total else 0.0
        print(f"  THIS IS A SAMPLE: {total} of {corpus_total} specs "
              f"({frac:.1f}% of the corpus) were compared at all.")
        print("  Coverage above is of the sample. No sentence about the corpus is")
        print("  admissible from this run.")
    print(f"  {ne} file(s) yielded no verdict and are NOT counted as agreement.")
    print("  Any sentence of the form 'no regressions' is admissible only with")
    print("  this coverage figure attached, and only when field-loss = 0 and")
    print("  unknown = 0. Coverage below 100% bounds what the run can claim.")

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
    if unknown:
        # An unknown is a file whose field set moved on well-formed input with
        # neither loss nor improvement. No verdict can be issued over a corpus
        # containing one, whatever the other counts say.
        print(f"NO VERDICT: {unknown} unknown file(s). A PASS is not available "
              f"while any file is unexplained,")
        print("regardless of the field-loss count. Explain them or classify them.")
    if lost or unknown:
        print(f"NOT CLEAN: {lost} field-loss, {unknown} unknown.")
        print("These are NOT to be aggregated away. field-loss means a field the")
        print("author declared is gone from the parse. It may still be the right")
        print("trade to make -- that is a language decision about what malformed")
        print("input should mean -- but it is a decision, not a measurement, and")
        print("it does not belong inside a count of zero.")
    else:
        print(f"CLEAN on the measured {measured}/{total} ({pct:.1f}%): "
              f"no field-loss and no unknown.")
        print("Scope of that statement, stated so it travels with the number:")
        print(f"  * it covers the {measured} file(s) on which both binaries gave a")
        print("    verdict, and says nothing about the rest;")
        print("  * it says nothing about categories the tool does not check --")
        print("    generated code, type inference, diagnostics, or timing.")
    return 1 if (lost or unknown) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
