#!/usr/bin/env python3
"""A conformance vector case must carry data, or it is documentation (#2241).

MEASURED STATE, which is why this gate exists: of 512 cases across 34 files,
412 carry nothing but an id and a sentence -- no inputs, no expected values.
No runner can execute those as written. They accumulated because nothing ever
objected: a file landed in conformance/, the summary counted it, and "34 vector
files" read as coverage while the number actually applied to RTL was zero.

WHAT CHANGED (T67). The first version of this gate recorded only the NAMES of
the prose-only files, and asked one question: did a new name appear? An audit
put four data-loss modes to it and it caught one:

    strip every data field from a file   ->  caught, exit 1
    reduce 7 cases to 1                  ->  exit 0
    set cases to []                      ->  exit 0   (the `total > 0` guard
                                                       made an emptied file
                                                       invisible ENTIRELY)
    corrupt the JSON                     ->  exit 0   (a parse failure was
                                                       swallowed as (0, 0),
                                                       i.e. "nothing here")
    delete the file                      ->  "FIXED ... now carries data"

That last line is the shape worth naming: the FIXED branch asserted from set
arithmetic alone -- a name that left the bad set must have been repaired -- and
so it congratulated a deletion, a corruption, and a renamed top-level key. The
NEW branch re-read the file; the FIXED branch did not.

So the ledger is now a CENSUS: every vector file with its case counts. A number
that falls is a failure whatever the cause, and a file that disappears is
reported as DEPARTED rather than as a repair.

    tools/check_vector_data.py                    # verify
    tools/check_vector_data.py --update-baseline  # after a deliberate change
    tools/check_vector_data.py --self-check       # negative control

Prose fields are id/description/note/name/comment. Any other key makes a case
data-carrying -- deliberately generous: a single input field is enough, because
the question this gate asks is "could a runner ever apply this?", not "is this
a good vector".

The history above was written by an AUDIT, not by this gate: every one of those
five modes was found by hand, because nothing here could fail on demand. The
`--self-check` control closes that -- it plants each failing class in a temp
tree and runs THIS WHOLE FILE against it as a subprocess, so the census, the
baseline writer, the classification and main()'s own return value all run.
"""
import json
import os
import pathlib
import subprocess
import sys
import tempfile

# T87: overridable so the negative control can point the WHOLE program -- census,
# baseline reader AND baseline writer, all derived from ROOT at import -- at a
# planted tree. Nothing in the repository sets it. It exists because a control
# that runs this tool from a temp cwd would leave ROOT resolving to the real
# repository (or, if the tool were copied out, to `/`), find nothing, and pass
# for the wrong reason. The clean case asserts the planted file COUNT, so a
# child that ignored this variable is caught rather than believed.
ROOT = pathlib.Path(os.environ.get("T27_VECTOR_ROOT")
                    or pathlib.Path(__file__).resolve().parent.parent)
VECTORS = ROOT / "conformance"
BASELINE = ROOT / "tools/vector_data_baseline.txt"
PROSE = {"id", "description", "note", "name", "comment"}


def counts(path):
    """(total cases, data-carrying cases) for one file, or None if unreadable.

    None, not (0, 0). A file this gate cannot parse is an anomaly, and reading
    an anomaly as "nothing to check" is how a corrupted fpga_uart.json passed
    end to end. `check_seal_coverage.py` already records unreadable inputs as
    findings; this is the same choice.
    """
    try:
        doc = json.loads(path.read_text())
    except Exception:
        return None
    total = data = 0
    for group in (doc.get("vectors") or {}).values():
        if not isinstance(group, dict):
            continue
        for case in group.get("cases") or []:
            if not isinstance(case, dict):
                continue
            total += 1
            if any(k not in PROSE for k in case):
                data += 1
    return (total, data)


def census():
    """{name: (total, data)} for every vector file; unreadable ones as None."""
    return {p.name: counts(p) for p in sorted(VECTORS.glob("fpga_*.json"))}


def baseline():
    """{name: (total, data)} from the ledger."""
    if not BASELINE.exists():
        return {}
    out = {}
    for ln in BASELINE.read_text().splitlines():
        ln = ln.strip()
        if not ln or ln.startswith("#"):
            continue
        parts = [p.strip() for p in ln.split("|")]
        if len(parts) != 3:
            continue
        try:
            out[parts[0]] = (int(parts[1]), int(parts[2]))
        except ValueError:
            continue
    return out


# ---------------------------------------------------------------- control ----

# The exact prefixes the report prints, padded as in the report, so that a
# marker can only have come from its own branch and not from prose elsewhere.
MARKERS = ("UNREADABLE ", "DEPARTED   ", "EMPTIED    ",
           "LOST DATA  ", "NEW        ", "BETTER     ")
DEBT_EPILOGUE = "the measured set getting smaller"
GAIN_EPILOGUE = "Files gained data. Record it"

DATA_A = {"id": "a1", "inputs": [1, 1], "expect": 1}
DATA_B = {"id": "a2", "inputs": [1, -1], "expect": -1}
PROSE_A = {"id": "a3", "description": "reads well, runs never"}

# 3 cases / 2 data-carrying, and a prose-only neighbour so the census line has
# a non-zero prose count to get wrong.
FIXTURE = {
    "fpga_alpha.json": [DATA_A, DATA_B, PROSE_A],
    "fpga_beta.json": [{"id": "b1", "description": "prose"},
                       {"id": "b2", "note": "prose"}],
}


def _write_vectors(conf, name, cases):
    (conf / name).write_text(json.dumps({"vectors": {"g": {"cases": cases}}}))


def _run_gate(root, *args):
    """Run THIS FILE as a subprocess with ROOT pointed at a planted tree."""
    return subprocess.run(
        [sys.executable, str(pathlib.Path(__file__).resolve()), *args],
        capture_output=True, text=True,
        env={**os.environ, "T27_VECTOR_ROOT": str(root)})


def _control_case(label, mutate, want, marker, code):
    """Plant the fixture, let the gate baseline it, mutate, run the gate.

    The baseline is written by the gate's own --update-baseline against the
    planted root, so the control never hand-writes the ledger it is about to
    compare against -- and never touches the repository's.
    """
    forbid = [m for m in MARKERS if m != marker]
    forbid.append(DEBT_EPILOGUE if GAIN_EPILOGUE in want else GAIN_EPILOGUE)
    with tempfile.TemporaryDirectory() as td:
        t = pathlib.Path(td)
        (t / "conformance").mkdir()
        (t / "tools").mkdir()
        for name, cases in FIXTURE.items():
            _write_vectors(t / "conformance", name, cases)
        rec = _run_gate(t, "--update-baseline")
        if rec.returncode != 0 or "2 files, 1 prose-only" not in rec.stdout:
            print(f"  {label:<10} FIXTURE NOT BASELINED (exit {rec.returncode}): "
                  f"{rec.stdout.strip()[:120]}")
            return False
        mutate(t / "conformance")
        r = _run_gate(t)
    missing = [s for s in want if s not in r.stdout]
    leaked = [s for s in forbid if s in r.stdout]
    ok = not missing and not leaked and r.returncode == code
    print(f"  {label:<10} exit {r.returncode} (want {code}); said it = {not missing}; "
          f"no other branch fired = {not leaked}")
    for s in missing:
        print(f"             MISSING  {s!r}")
    for s in leaked:
        print(f"             LEAKED   {s!r}  (wrong branch)")
    return ok


def self_check():
    """Prove this gate can go RED, once per failing class, on a planted tree.

    Each case runs the REAL program end to end -- census(), counts(), the
    baseline writer, the classification loop, the report and main()'s return --
    against a temp root handed over in T27_VECTOR_ROOT. Nothing here re-states
    the comparison: a control that evaluates its own copy of the logic certifies
    the copy, and three mutants of the real logic walked through exactly such a
    control elsewhere in this repository (T73).

    Each case names the MESSAGE of its branch and requires every other branch's
    marker to be ABSENT. Exit codes alone would not separate them: all six
    failing classes exit 1, and the audit in the module docstring is a list of
    faults that reached the wrong branch (a corrupt file read as "nothing here",
    a deletion reported as a repair). UNREADABLE is the sharpest of these -- if
    the parse failure is ever swallowed as (0, 0) again, the run still exits 1,
    through EMPTIED, and only the absent-marker assertion notices.
    """
    cases = [
        ("clean", lambda c: None,
         ["vector files: 2 (1 prose-only, baseline 2)",
          "OK: no vector file lost data, emptied, departed or stopped parsing"],
         None, 0),
        ("LOST DATA",
         lambda c: _write_vectors(c, "fpga_alpha.json", [DATA_A, PROSE_A, PROSE_A]),
         ["LOST DATA  fpga_alpha.json: 2 -> 1 data-carrying cases", DEBT_EPILOGUE],
         "LOST DATA  ", 1),
        ("EMPTIED",
         lambda c: _write_vectors(c, "fpga_alpha.json", []),
         ["EMPTIED    fpga_alpha.json: had 3 cases, now has none", DEBT_EPILOGUE],
         "EMPTIED    ", 1),
        ("UNREADABLE",
         lambda c: (c / "fpga_alpha.json").write_text('{"vectors": {"g": '),
         ["UNREADABLE fpga_alpha.json: cannot be parsed as JSON", DEBT_EPILOGUE],
         "UNREADABLE ", 1),
        ("DEPARTED",
         lambda c: (c / "fpga_alpha.json").unlink(),
         ["DEPARTED   fpga_alpha.json: was 3 cases / 2 carrying data, now absent",
          DEBT_EPILOGUE],
         "DEPARTED   ", 1),
        ("NEW",
         lambda c: _write_vectors(c, "fpga_gamma.json",
                                  [{"id": "g1", "description": "prose"}]),
         ["NEW        fpga_gamma.json: 1 cases, none carrying data", DEBT_EPILOGUE],
         "NEW        ", 1),
        ("BETTER",
         lambda c: _write_vectors(c, "fpga_alpha.json",
                                  [DATA_A, DATA_B, {"id": "a3", "expect": 0}]),
         ["BETTER     fpga_alpha.json: 2 -> 3 data-carrying cases", GAIN_EPILOGUE],
         "BETTER     ", 1),
    ]
    ok = True
    for label, mutate, want, marker, code in cases:
        ok = _control_case(label, mutate, want, marker, code) and ok
    print(f"  self-check: {len(cases) - 1} failing classes each reported by name, "
          f"clean planted tree silent = {ok}")
    return 0 if ok else 1


def main():
    if "--self-check" in sys.argv:
        return self_check()

    now = census()

    if "--update-baseline" in sys.argv:
        unreadable = [n for n, v in now.items() if v is None]
        if unreadable:
            print("refusing to record a baseline over unreadable files:")
            for n in unreadable:
                print(f"  {n}")
            return 1
        prose = sum(1 for v in now.values() if v[0] > 0 and v[1] == 0)
        header = (
            "# A CENSUS of conformance/fpga_*.json: name | total cases | cases\n"
            "# carrying data. Debt is any row whose data count is 0 -- documentation\n"
            "# living in a vectors/ shape, which no runner can execute (#2241).\n"
            "#\n"
            "# This gate fails when a data count FALLS, when a file empties, when a\n"
            "# baselined file disappears, and when one cannot be parsed. It records\n"
            "# numbers rather than names because the name-only version congratulated\n"
            "# a deletion with \"now carries data\" (T67).\n"
            f"# {len(now)} files, {prose} of them prose-only.\n"
        )
        body = "\n".join(f"{k} | {v[0]} | {v[1]}" for k, v in sorted(now.items()))
        BASELINE.write_text(header + body + "\n")
        print(f"baseline updated: {len(now)} files, {prose} prose-only")
        return 0

    base = baseline()
    if not base:
        print("no baseline; run --update-baseline once")
        return 1

    unreadable, departed, emptied, lost, new_prose, better = [], [], [], [], [], []

    for name in sorted(set(now) | set(base)):
        cur = now.get(name)
        old = base.get(name)
        if name not in now:
            departed.append((name, old))
            continue
        if cur is None:
            unreadable.append(name)
            continue
        total, data = cur
        if old is None:
            if total > 0 and data == 0:
                new_prose.append((name, total))
            continue
        if total == 0 and old[0] > 0:
            emptied.append((name, old[0]))
        elif data < old[1]:
            lost.append((name, old[1], data))
        elif data > old[1]:
            better.append((name, old[1], data))

    prose_now = sum(1 for v in now.values() if v and v[0] > 0 and v[1] == 0)
    print(f"vector files: {len(now)} ({prose_now} prose-only, baseline {len(base)})")
    for n in unreadable:
        print(f"  UNREADABLE {n}: cannot be parsed as JSON")
    for n, old in departed:
        print(f"  DEPARTED   {n}: was {old[0]} cases / {old[1]} carrying data, now absent")
    for n, was in emptied:
        print(f"  EMPTIED    {n}: had {was} cases, now has none")
    for n, was, is_ in lost:
        print(f"  LOST DATA  {n}: {was} -> {is_} data-carrying cases")
    for n, total in new_prose:
        print(f"  NEW        {n}: {total} cases, none carrying data")
    for n, was, is_ in better:
        print(f"  BETTER     {n}: {was} -> {is_} data-carrying cases")

    if unreadable or departed or emptied or lost or new_prose:
        print()
        print("A conformance case with no inputs and no expected values cannot be")
        print("executed by any runner -- it is documentation. A file that empties,")
        print("loses data cases, disappears or stops parsing is not a repair: it is")
        print("the measured set getting smaller, which reads as progress and is not.")
        print("If this is deliberate: tools/check_vector_data.py --update-baseline")
        return 1
    if better:
        print()
        print("Files gained data. Record it: tools/check_vector_data.py --update-baseline")
        return 1
    print("OK: no vector file lost data, emptied, departed or stopped parsing")
    return 0


if __name__ == "__main__":
    sys.exit(main())
