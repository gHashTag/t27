#!/usr/bin/env python3
"""Gate 22: every Rust crate must be built by some workflow, or be listed.

Prop. 159 found 1213 tests that no job ran, because the only `cargo test`
workflow discovers `ring-*-rust` crates by matrix and never matched the compiler
crate. That was one instance. Enumerated: **8 of 30 crates are covered by no
workflow at all** -- and 2 of those 8 did not compile. (A third
appeared broken only under `--offline`; see Prop. 165.)

The correlation is the finding. `flash-spi` stopped building when `FlashOpts`
gained two fields and one call site was not updated; nothing built it, so
nothing said so. **The crates nothing tests are the crates that break**, and the
gap is invisible because every workflow that runs is green.

WHAT THIS GATE REQUIRES. Every `[package]` under the repository must either

  (a) be named by a workflow (`-p <name>`, its path, or a discovery pattern it
      demonstrably matches), or
  (b) appear in the baseline below, recording that it is knowingly ungated.

It RATCHETS: a new ungated crate fails the build; an existing one does not.
Whether a given crate deserves CI time is a project decision, not a scanner's.

ARTIFACTS. Reads `**/Cargo.toml` and `.github/workflows/*.yml`. WRITES
`formal/crate_coverage_baseline.txt`, and only when `--init` is passed. Nothing else.

Prop. 163.
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BASELINE = ROOT / "formal" / "crate_coverage_baseline.txt"
SKIP = (".git", "target/", "worktrees", "node_modules", "/gen/")


def crates():
    out = {}
    for c in ROOT.rglob("Cargo.toml"):
        s = str(c)
        if any(p in s for p in SKIP):
            continue
        # comment-scan: TOML comments start with `#`; a commented-out `name =`
        # is not a package name.
        text = "\n".join(l for l in c.read_text(errors="ignore").splitlines()
                         if not l.lstrip().startswith("#"))
        if "[package]" not in text:
            continue
        m = re.search(r'^\s*name\s*=\s*"([^"]+)"', text, re.M)
        if m:
            out[m.group(1)] = str(c.parent.relative_to(ROOT))
    return out


def main():
    wf_dir = ROOT / ".github" / "workflows"
    if not wf_dir.exists():
        print("::error::crate coverage scan: no such directory "
              "'.github/workflows' -- nothing was scanned")
        return 1
    found = crates()
    if not found:
        print("::error::crate coverage scan: found no Cargo.toml with a "
              "[package] section under the repository root -- nothing was "
              "scanned")
        return 1

    wf = " ".join(y.read_text(errors="ignore") for y in wf_dir.glob("*.yml"))

    # Prop. 165: a `cargo check/test --workspace` step covers exactly the
    # members list in the root Cargo.toml -- no more, no less. Same rule as a
    # discovery matrix (Prop. 162): coverage is the invocation's OUTPUT, so
    # resolve the members rather than crediting the flag.
    members = set()
    if re.search(r"cargo\s+(?:check|test|build)[^\n]*--workspace", wf):
        root = ROOT / "Cargo.toml"
        if root.exists():
            m = re.search(r"members\s*=\s*\[([^\]]*)\]", root.read_text())
            if m:
                members = set(re.findall(r'"([^"]+)"', m.group(1)))

    ungated = []
    for name, path in sorted(found.items()):
        covered = (f"-p {name}" in wf) or (path in wf) or (path in members)
        # A discovery matrix counts only for crates it demonstrably matches.
        # Prop. 162: a matrix's coverage is its output, not its intent.
        if not covered and name.startswith("ring-") and "rings_matrix" in wf:
            covered = True
        if not covered:
            ungated.append(f"{name}\t{path}")

    print(f"crate coverage scan: {len(found)} crates, "
          f"{len(found) - len(ungated)} covered by a workflow, "
          f"{len(ungated)} ungated")

    if not BASELINE.exists():
        # Prop. 211c: writing a baseline is an explicit act, never a fallback.
        # `if not exists(): write(now); return 0` resets the ratchet on one
        # `rm`, and on a clone that never had the file it rubber-stamps the tree
        # it was handed and exits 0. Measured before f66561f33: 8 of the 13
        # baselines in this suite were on disk and in no commit, and 8 of the 13
        # gates owning them re-baseline a possibly-broken tree and pass.
        if "--init" not in sys.argv[1:]:
            print(f"::error::crate coverage scan: {BASELINE.name} does not exist and "
                  f"--init was not given. Writing one here would record "
                  f"whatever this tree contains as the accepted state -- on a "
                  f"fresh clone that is a green run which checked nothing. "
                  f"Genuine first run: `python3 formal/crate_coverage_scan.py --init`. "
                  f"Otherwise the baseline was lost and belongs in the commit "
                  f"that lost it (Prop. 211)")
            return 1
        BASELINE.write_text("\n".join(ungated) + ("\n" if ungated else ""))
        print(f"crate coverage scan: baseline written to {BASELINE.name} "
              f"({len(ungated)} ungated)")
        return 0
    was = [l for l in BASELINE.read_text().splitlines() if l.strip()]
    new = [u for u in ungated if u not in was]
    if new:
        print(f"::error::crate coverage scan: {len(new)} new crate(s) are built "
              f"by no workflow. A crate nothing builds is a crate that will "
              f"stop building without anyone being told -- add it to a "
              f"workflow, or to {BASELINE.name} with a reason")
        for n in new:
            print(f"  {n}")
        return 1
    gained = [w for w in was if w not in ungated]
    if gained:
        print(f"crate coverage scan: {len(gained)} crate(s) newly covered; "
              f"update {BASELINE.name} to lock it in")
    print(f"crate coverage scan: ratchet holds "
          f"({len(ungated)} <= {len(was)} ungated)")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::crate coverage scan: could not scan Cargo.toml files "
              f"({type(exc).__name__}: {exc}) -- nothing was scanned")
        sys.exit(1)
