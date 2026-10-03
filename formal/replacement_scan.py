#!/usr/bin/env python3
"""Gate 33: a population-level ratchet cannot see content that moved.

Gate 32 (Prop. 209/210) closed the case where a corpus SHRINKS: it holds one
file count and one summed line count per population, and a build that loses
files fails until the loss is acknowledged. It holds exactly five pairs of
scalars for the whole repository, and a sum is indifferent to where its terms
came from. Measured against that shape:

    move a declaration out of one spec and into another   -> both scalars hold
    delete file A, add file B of equal size, new name     -> both scalars hold
    rewrite a file's bodies without changing its length   -> both scalars hold

None of those is hypothetical arithmetic; the third is the commonest commit
shape in this repo's spec history. Of the last 30 commits touching `specs/`,
28 changed `.t27` content and **7 were exactly line-count-neutral** -- 90+/90-
across 57 files, 1679+/1679- across 112 files, 400+/400- across 27. Gate 32
reports `specs 497 / specs-lines 76092` before and after every one of them.

WHAT THIS GATE DOES. It takes a per-file DECLARATION CENSUS and ratchets it by
PATH: one count per file instead of one sum per population -- 540 scalars where
gate 32 holds 10. A file that loses declarations fails the build; a baselined
path that disappears fails the build, separately reported, because that is the
class gate 32 already covers and its acknowledgement cost should not be
confused with this gate's. Losing declarations is often correct. The gate does
not forbid it; it forbids doing it silently.

It is a CENSUS, not a content-replacement gate, and the distinction is the
point. The same 7 line-count-neutral commits changed the declaration name set
by +0/-0, so **this gate detects 0 of 7** of them. A per-declaration BODY hash
would catch 6, and would also fire on 46% of all spec-content commits -- a hash
of the corpus with extra steps, switched off within a wave. So the body-level
signal is PUBLISHED, never ratcheted: files whose declaration count is stable
while the declaration set changed are printed as a churn record.

COVERAGE. Three populations, each keyed by path:

  * specs      `specs/**/*.t27`, keywords fn const struct enum module spec
               algorithm type trait impl test invariant (test+invariant are 38%
               of spec declarations and are the verification surface; both were
               missing from the first draft of this list)
  * coq        `trios-coq/**/*.v`, Lemma Theorem Corollary Definition Inductive
               Record Fixpoint Instance
  * rtl        `build/rtl/*.sv`, module

Populations are enumerated from the GIT INDEX, not from `rglob` on disk. That
is not a stylistic preference: `formal/*.sv` is 15 files on disk and 0 in the
index, and `formal/*_baseline.txt` is 12 on disk and 5 in the index -- 22 files
that a disk-walking census counts and a clean CI checkout does not have.
`build/rtl` is the exception and is enumerated from disk BY NECESSITY: it is
gitignored and generated during the run, so the census digest is the only
record of that content that outlives the job.

What it does not see. Same-file body replacement under a preserved declaration
name -- the residue, stated rather than implied away. Anything inside a
declaration body. Any declaration whose keyword is not in the lists above; the
run prints how many code lines sit in files it finds no declaration in, and
that figure is UNRATCHETED. Whether any declaration is correct.

ARTIFACTS. Reads the git index, `specs/`, `trios-coq/`, `build/rtl/`, and
`.github/workflows/*.yml`. WRITES `formal/replacement_baseline.txt` only when
`--init` is passed AND git does not know a baseline already -- a baseline that git tracks and
disk lacks is a deleted ratchet, and re-writing it would silently re-baseline
to whatever the tree currently contains, which is how every
`if not BASELINE.exists(): write(now); return 0` gate in this suite can be
reset by one `rm`. Nothing else.

Prop. 211.
"""
import hashlib
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BASELINE = ROOT / "formal" / "replacement_baseline.txt"
SELF = pathlib.Path(__file__).resolve()

# name, directory, glob, source ("git" | "disk"), comment style, decl regex
POPULATIONS = [
    ("specs", "specs", "*.t27", "git", "t27",
     r"^[ \t]*(?:pub[ \t]+)?(fn|const|struct|enum|module|spec|algorithm|type|"
     r"trait|impl|test|invariant)[ \t]+([A-Za-z_][A-Za-z0-9_\-]*)"),
    ("coq", "trios-coq", "*.v", "git", "coq",
     r"^[ \t]*(Lemma|Theorem|Corollary|Definition|Inductive|Record|Fixpoint|"
     r"Instance)[ \t]+([A-Za-z_][A-Za-z0-9_']*)"),
    ("rtl", "build/rtl", "*.sv", "disk", "c",
     r"^[ \t]*(module)[ \t]+([A-Za-z_][A-Za-z0-9_]*)"),
]

# The workflow step that produces the `rtl` population. This gate must run
# AFTER it or it measures a directory that does not exist yet -- the defect
# gate 32 currently has, where the committed baseline reads `generated-rtl 0`
# because it is invoked 24 workflow lines before the bundle is emitted.
RTL_STEP = "gen-bitnet-bundle"


def _blank(m):
    """Replace a comment with spaces, preserving newlines and line numbers."""
    return re.sub(r"[^\n]", " ", m.group(0))


def strip_comments(text, style):
    """Blank out comments before any regex touches the text.

    Five fixes across four files went to patterns matched against raw source
    (Props. 95, 102c, 118); `formal/comment_scan.py` exists to stop the sixth.
    `.t27` carries BOTH `//` and leading-`;` comment lines -- `gf16.t27` opens
    with five `;` lines -- and Coq's `(* *)` nests.

    # comment-scan: strips `//`, `/* */`, leading `;` (t27) and nested `(* *)`
    # (coq) before matching. String literals containing `//` lose their tail;
    # every decl pattern here is anchored at line start, so that cannot add or
    # remove a match.
    """
    if style == "coq":
        out, depth, i, n = [], 0, 0, len(text)
        while i < n:
            if text.startswith("(*", i):
                depth += 1
                out.append("  ")
                i += 2
                continue
            if depth and text.startswith("*)", i):
                depth -= 1
                out.append("  ")
                i += 2
                continue
            ch = text[i]
            out.append(ch if (depth == 0 or ch == "\n") else " ")
            i += 1
        return "".join(out)
    text = re.sub(r"/\*.*?\*/", _blank, text, flags=re.S)
    text = re.sub(r"//[^\n]*", _blank, text)
    if style == "t27":
        text = re.sub(r"(?m)^[ \t]*;[^\n]*", _blank, text)
    return text


def git_files(rel, suffix):
    """Index entries under `rel` with `suffix`. None if git cannot answer.

    `git ls-files` lists the INDEX. A tracked file deleted from the worktree is
    still listed -- deliberately: it must appear in the census with zero
    declarations rather than vanish from the denominator.
    """
    try:
        r = subprocess.run(["git", "-C", str(ROOT), "ls-files", "-z", "--", rel],
                           capture_output=True, timeout=60)
    except (OSError, subprocess.SubprocessError):
        return None
    if r.returncode != 0:
        return None
    return sorted(p for p in r.stdout.decode(errors="ignore").split("\0")
                  if p.endswith(suffix))


def census():
    """{pop\tpath: (count, digest)}, plus per-population reporting."""
    rows, absent, report = {}, [], {}
    for name, rel, pat, source, style, rx in POPULATIONS:
        rxc = re.compile(rx, re.M)
        d = ROOT / rel
        suffix = pat.lstrip("*")
        note = ""
        if source == "git":
            paths = git_files(rel, suffix)
            if paths is None:
                paths, note = [], "git could not enumerate the index"
            else:
                on_disk = {p.relative_to(ROOT).as_posix()
                           for p in d.rglob(pat)} if d.exists() else set()
                untracked = len(on_disk - set(paths))
                if untracked:
                    note = (f"{untracked} file(s) on disk are not in the index "
                            f"-- a clean checkout would not have them")
        else:
            paths = sorted(p.relative_to(ROOT).as_posix()
                           for p in d.rglob(pat)) if d.exists() else []
            note = "enumerated from disk: generated and gitignored"

        decls, uncovered_lines, files_no_decl = 0, 0, 0
        for rp in paths:
            f = ROOT / rp
            if f.resolve() == SELF:          # never measure the scanner itself
                continue
            try:
                body = strip_comments(f.read_text(errors="ignore"), style)
            except OSError:
                absent.append(f"{name}\t{rp}")
                rows[f"{name}\t{rp}"] = (0, "-")
                continue
            ds = sorted(f"{k} {n}" for k, n in rxc.findall(body))
            rows[f"{name}\t{rp}"] = (
                len(ds), hashlib.sha256("\n".join(ds).encode()).hexdigest()[:12])
            decls += len(ds)
            if not ds:
                files_no_decl += 1
                uncovered_lines += sum(1 for l in body.splitlines() if l.strip())
        report[name] = dict(files=len([p for p in paths]), decls=decls,
                            no_decl=files_no_decl, uncovered=uncovered_lines,
                            note=note)
    return rows, absent, report


def workflow_reference_set():
    """Scripts CI actually runs, and where this one sits relative to the RTL."""
    wf = ROOT / ".github" / "workflows"
    cited, order = set(), []
    if not wf.exists():
        return cited, order
    for y in sorted(wf.glob("*.yml")):
        try:
            lines = y.read_text(errors="ignore").splitlines()
        except OSError:
            continue
        me = rtl = None
        for i, line in enumerate(lines, 1):
            cited.update(re.findall(r"python3 formal/(\w+\.py)", line))
            if "python3 formal/replacement_scan.py" in line and me is None:
                me = i
            if RTL_STEP in line and rtl is None:
                rtl = i
        if me is not None:
            order.append((y.name, me, rtl))
    return cited, order


def load_baseline():
    was = {}
    for line in BASELINE.read_text().splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        pop, path, n, dig = line.split("\t")
        was[f"{pop}\t{path}"] = (int(n), dig)
    return was


def baseline_is_tracked():
    """True if git knows a baseline that disk does not have."""
    try:
        r = subprocess.run(
            ["git", "-C", str(ROOT), "ls-files", "--", "formal/replacement_baseline.txt"],
            capture_output=True, timeout=60)
    except (OSError, subprocess.SubprocessError):
        return False
    return r.returncode == 0 and bool(r.stdout.strip())


def show(key):
    return key.replace("\t", "  ")


def main():
    now, absent, report = census()

    # STARVE. A census of nothing satisfies every count it makes. Gate 32's own
    # first version exited 0 alone in an empty tree because `gate-scripts`
    # counted that file (Prop. 209b); this one excludes itself from every
    # population by resolved path, so the only defence left is requiring the
    # subject to exist. Each declared population must yield declarations --
    # "the rtl step has not run yet" and "the RTL is empty" are the same
    # measurement, and neither is a pass.
    empty = [n for n, _, _, _, _, _ in POPULATIONS if report[n]["decls"] == 0]
    if empty:
        print(f"::error::replacement scan: {', '.join(empty)} yielded zero "
              f"declarations -- an empty population is not a passing "
              f"population, it is a census that measured nothing (Prop. 211)")
        for n in empty:
            note = report[n]["note"] or "directory absent or contains no matching files"
            print(f"  {n}: {report[n]['files']} file(s); {note}")
        if "rtl" in empty:
            print(f"  rtl is generated by the `{RTL_STEP}` workflow step; a run "
                  f"placed before it measures a directory that does not exist")
        return 1

    # The reference set. Same repair as Prop. 200/209: resolve what CI runs and
    # require it present, so this gate cannot certify a tree that is not the
    # tree CI executes.
    cited, order = workflow_reference_set()
    if not cited:
        print("::error::replacement scan: no `python3 formal/*.py` step in any "
              "workflow -- there is no reference set, so this census is a "
              "census of whatever happens to be on disk")
        return 1
    present = {f.name for f in (ROOT / "formal").glob("*.py")}
    missing_scripts = sorted(cited - present)
    if missing_scripts:
        print(f"::error::replacement scan: {len(missing_scripts)} script(s) CI "
              f"runs are absent from formal/ -- the tree being measured is not "
              f"the tree CI executes")
        for a in missing_scripts[:8]:
            print(f"  formal/{a}")
        return 1

    # Placement self-check. This gate reads build/rtl, which does not exist
    # until the bundle is emitted. Gate 32 is wired 24 lines too early and its
    # committed baseline therefore records `generated-rtl 0`; the same mistake
    # here would baseline an empty RTL census and then pass forever.
    for wf_name, me_line, rtl_line in order:
        if rtl_line is None:
            print(f"replacement scan: note: {wf_name} runs this gate at line "
                  f"{me_line} and has no `{RTL_STEP}` step -- the rtl "
                  f"population there is whatever the runner left behind")
        elif me_line < rtl_line:
            print(f"::error::replacement scan: {wf_name} runs this gate at line "
                  f"{me_line}, before `{RTL_STEP}` at line {rtl_line}. The rtl "
                  f"census would be taken on a directory that does not exist "
                  f"yet -- exactly the defect that leaves gate 32's committed "
                  f"baseline reading `generated-rtl 0` (Prop. 211)")
            return 1
    if not order:
        print("replacement scan: note: no workflow runs this gate yet -- it is "
              f"measuring, not gating, until a step is added after `{RTL_STEP}`")

    print("replacement scan: " + ", ".join(
        f"{n}={report[n]['decls']} decls/{report[n]['files']} files"
        for n, _, _, _, _, _ in POPULATIONS))
    for n, _, _, _, _, _ in POPULATIONS:
        r = report[n]
        print(f"replacement scan: {n}: {r['uncovered']} code lines in "
              f"{r['no_decl']} file(s) with no declaration this gate names "
              f"-- unratcheted" + (f"; {r['note']}" if r["note"] else ""))
    if absent:
        print(f"replacement scan: {len(absent)} tracked file(s) could not be "
              f"read from disk and are counted as zero:")
        for k in absent[:8]:
            print("  UNREADABLE " + show(k))

    if not BASELINE.exists():
        # Prop. 211b: `if not exists(): write(now)` makes `rm baseline` a
        # sanctioned re-baseline. Distinguish "never had one" from "had one and
        # lost it" by asking git, which is the only party that remembers.
        if baseline_is_tracked():
            print(f"::error::replacement scan: {BASELINE.name} is tracked by git "
                  f"and missing from the worktree. Writing a fresh one would "
                  f"re-baseline the ratchet to whatever the tree contains right "
                  f"now, which is not a first run -- it is a reset with the "
                  f"evidence removed. Restore it with `git checkout -- "
                  f"formal/{BASELINE.name}` (Prop. 211)")
            return 1
        # Prop. 211c: writing a baseline is an explicit act, never a fallback.
        # `if not exists(): write(now); return 0` resets the ratchet on one
        # `rm`, and on a clone that never had the file it rubber-stamps the tree
        # it was handed and exits 0. Measured before f66561f33: 8 of the 13
        # baselines in this suite were on disk and in no commit, and 8 of the 13
        # gates owning them re-baseline a possibly-broken tree and pass.
        if "--init" not in sys.argv[1:]:
            print(f"::error::replacement scan: {BASELINE.name} does not exist and "
                  f"--init was not given. Writing one here would record "
                  f"whatever this tree contains as the accepted state -- on a "
                  f"fresh clone that is a green run which checked nothing. "
                  f"Genuine first run: `python3 formal/replacement_scan.py --init`. "
                  f"Otherwise the baseline was lost and belongs in the commit "
                  f"that lost it (Prop. 211)")
            return 1
        BASELINE.write_text("".join(f"{k}\t{v[0]}\t{v[1]}\n"
                                    for k, v in sorted(now.items())))
        print(f"replacement scan: baseline written to {BASELINE.name} "
              f"({len(now)} files keyed by path)")
        return 0

    was = load_baseline()
    gone = [k for k in sorted(was) if k not in now]
    dropped = [(k, was[k][0], now[k][0]) for k in sorted(was)
               if k in now and now[k][0] < was[k][0]]
    swapped = [k for k in sorted(was) if k in now
               and now[k][0] == was[k][0] and was[k][1] != "-"
               and now[k][1] != was[k][1]]
    grew = [k for k in sorted(was) if k in now and now[k][0] > was[k][0]]
    added = [k for k in sorted(now) if k not in was]

    rc = 0
    # Partitioned deliberately. `gone` is gate 32's class -- it is reported
    # here only because a path-keyed census cannot avoid noticing it, and
    # separating the two keeps the acknowledgement cost legible.
    if gone:
        print(f"::error::replacement scan: {len(gone)} baselined file(s) are no "
              f"longer in their population. This is gate 32's class of loss "
              f"(Prop. 209) seen per path: a delete-and-add of equal file count "
              f"and equal line count holds every scalar gate 32 keeps. If the "
              f"loss is intended, update {BASELINE.name} in the same commit")
        for k in gone[:10]:
            print(f"  GONE     {show(k)}  (was {was[k][0]} decls)")
        rc = 1
    if dropped:
        print(f"::error::replacement scan: {len(dropped)} file(s) are present "
              f"and lost declarations. Gate 32 ratchets one summed line count "
              f"per population, so declarations moved from one file into "
              f"another leave its ratchet holding. If the loss is intended, "
              f"update {BASELINE.name} in the same commit (Prop. 211)")
        for k, o, n in dropped[:10]:
            print(f"  DROPPED  {show(k)}: {o} -> {n}")
        rc = 1

    # Published, never ratcheted. A digest-equality ratchet fires on 46% of
    # spec-content commits; this line is the honest form of the same signal.
    print(f"replacement scan: {len(swapped)} file(s) kept their declaration "
          f"count and changed the set (published, not ratcheted -- a body-level "
          f"ratchet fires on 46% of spec commits), {len(grew)} grew, "
          f"{len(added)} new")
    for k in swapped[:10]:
        print(f"  swap     {show(k)}  ({was[k][1]} -> {now[k][1]})")
    if grew or added:
        print(f"replacement scan: update {BASELINE.name} to lock in the growth")
    if rc == 0:
        print(f"replacement scan: ratchet holds ({len(was)} files keyed by "
              f"path, none lost declarations)")
    return rc


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::replacement scan: could not take the declaration "
              f"census ({type(exc).__name__}: {exc}) -- nothing was checked")
        sys.exit(1)
