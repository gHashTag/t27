#!/usr/bin/env python3
"""Gate 30: ask Coq what each theorem actually rests on.

Prop. 203 closed the trios-coq build at 28 of 28 and immediately flagged the
limit: **files compiling is not proofs being meaningful.** This gate is that
limit made measurable, using Coq's own answer rather than a scan of the text --
`Print Assumptions <thm>` reports the axioms a theorem transitively depends on,
which no regex over `Axiom` lines can reconstruct.

The first run answered the question it was built for. `avs_efficiency_lower_bound`
depends on:

    ClassicalDedekindReals.sig_forall_dec      <- standard, comes with Reals
    isscc_pi_2024_measured_bound               <- DOMAIN: asserts the bound

The second is not a lemma about the design; it is the measured bound, assumed.
The "lower bound theorem" restates its own hypothesis. That is the Prop. 194
numerator fallacy inside a proof: `Qed` is a statement about the derivation, never
about the truth of what was assumed, and the two are indistinguishable in a count
of `Qed`.

WHAT IT REPORTS. Per theorem: the domain axioms it rests on, where "domain" means
"not in the standard-library allowlist below". A theorem with none is closed under
the standard axioms. A theorem with some is conditional, and this gate names the
conditions.

It RATCHETS the set of (theorem, axiom) pairs. Whether a given physical constant
SHOULD be an axiom is a modelling decision -- a measured efficiency legitimately
enters as one -- so the gate does not forbid them. It fails when a NEW dependency
appears, so that a proof cannot quietly acquire an assumption.

COVERAGE. Runs over every `Lemma|Theorem|Corollary` name found in the `.v` files
of the trios-coq `_CoqProject`, requiring the compiled modules: it needs the tree
BUILT, and reports how many names it resolved versus found. It does not cover
`coq/` or `proofs/`, which have their own projects and roots. `Admitted` proofs do
not appear as axioms of themselves but DO appear in the assumptions of anything
using them, which is the point -- a `Qed` downstream of an `Admitted` is not a
proof, and only this instrument shows it.

MEASURED, first run: 460 theorem names found in 30 modules, **340 resolved**, 12
resting on a domain axiom, 41 distinct dependencies. The 120 unresolved names --
**26%** -- are the real residue: a module whose probe fails to load contributes
nothing and is counted, not silently dropped. So "12 conditional theorems" is a
statement about the 340, and a lower bound on the tree.

ARTIFACTS. Reads `trios-coq/**/*.v`, runs `coqc` on a generated probe file in a
temporary directory. WRITES `formal/assumption_baseline.txt`, and only when
`--init` is passed. Nothing else.

Prop. 204.
"""
import pathlib
import re
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
TREE = ROOT / "trios-coq"
BASELINE = ROOT / "formal" / "assumption_baseline.txt"

# Axioms every development using Reals or classical logic inherits. Listing them
# by PREFIX, with the reason, rather than filtering "anything that looks
# standard" -- Prop. 197: an exemption by property, not by name.
STDLIB = (
    "ClassicalDedekindReals.",   # constructive reals: comes with Require Reals
    "Classical_Prop.", "Classical_Pred_Type.",
    "FunctionalExtensionality.", "functional_extensionality",
    "ProofIrrelevance.", "proof_irrelevance",
    "Eqdep.", "JMeq_eq", "eq_rect_eq",
    "Rdefinitions.", "RinvImpl.", "ClassicalUniqueChoice.",
)


def theorem_names():
    out = {}
    for f in sorted(TREE.rglob("*.v")):
        src = re.sub(r"\(\*.*?\*\)", "", f.read_text(errors="ignore"), flags=re.S)
        rel = f.relative_to(TREE).with_suffix("")
        mod = "TriosCoq." + str(rel).replace("/", ".")
        # Prop. 205: a `Module X.` inside a file qualifies every name under it,
        # so `Print Assumptions foo` fails with "reference not found" -- which
        # is indistinguishable in the summary from "the module did not load".
        # Three files here use one. Sections do NOT qualify (their variables are
        # discharged but the lemma stays at the enclosing level), so only Module
        # is tracked.
        stack = []
        for line in src.splitlines():
            mm = re.match(r"\s*Module\s+(?!Type\b)(\w+)\s*\.", line)
            if mm:
                stack.append(mm.group(1))
                continue
            if re.match(r"\s*End\s+(\w+)\s*\.", line) and stack:
                if re.match(r"\s*End\s+" + re.escape(stack[-1]) + r"\s*\.", line):
                    stack.pop()
                continue
            lm = re.match(r"\s*(?:Lemma|Theorem|Corollary)\s+(\w+)", line)
            if lm:
                out.setdefault(mod, []).append(".".join(stack + [lm.group(1)]))
    return out


def main():
    if not TREE.exists():
        print("::error::assumption scan: no such directory 'trios-coq' -- "
              "nothing was scanned")
        return 1
    mods = theorem_names()
    if not mods:
        print("::error::assumption scan: found no Lemma/Theorem/Corollary in "
              "trios-coq/**/*.v -- nothing was scanned")
        return 1
    if not list(TREE.rglob("*.vo")):
        print("::error::assumption scan: trios-coq has no .vo files -- the tree "
              "must be BUILT before its assumptions can be asked for. Run "
              "coq_makefile + make first; nothing was scanned")
        return 1

    # Prop. 205: the first version hardcoded `-R . TriosCoq`, and 120 of 460
    # names (26%) failed with "Cannot load T27.IGLA.RMarker: no physical path".
    # The project declares TWO roots -- `-R . TriosCoq` and `-R ../coq T27` --
    # so half the tree could not load and the gate reported an unresolved
    # fraction rather than a wrong load path. Resolve the flags from
    # _CoqProject; do not restate them (Props. 162, 165, 168: read the
    # construct, do not match it).
    proj = TREE / "_CoqProject"
    loadpath = []
    if proj.exists():
        for line in proj.read_text().splitlines():
            parts = line.split()
            if parts and parts[0] in ("-R", "-Q") and len(parts) >= 3:
                loadpath += parts[:3]
            elif parts and parts[0] == "-I" and len(parts) >= 2:
                loadpath += parts[:2]
    if not loadpath:
        print("::error::assumption scan: trios-coq/_CoqProject declares no -R/-Q "
              "load path -- coqc would resolve no module, so every count would "
              "be about an empty set")
        return 1

    total = sum(len(v) for v in mods.values())
    pairs, resolved = [], 0
    with tempfile.TemporaryDirectory() as td:
        for mod, names in sorted(mods.items()):
            probe = pathlib.Path(td) / "probe.v"
            probe.write_text(f"Require Import {mod}.\n" +
                             "".join(f"Print Assumptions {n}.\n" for n in names))
            r = subprocess.run(["coqc"] + loadpath + [str(probe)],
                               cwd=str(TREE), capture_output=True, text=True)
            out = r.stdout + r.stderr
            if "Error" in out and "Assumptions" not in out:
                continue          # module did not load; counted by `resolved`
            # Each report is "Axioms:\n<name> : ..." blocks, one per Print.
            for blk, name in zip(re.split(r"(?:Axioms|Closed under the global "
                                          r"context)", out)[1:], names):
                resolved += 1
                for ax in re.findall(r"^([A-Za-z_][\w.']*)\s*:", blk, re.M):
                    if not ax.startswith(STDLIB):
                        pairs.append(f"{mod}.{name}\t{ax}")

    now = sorted(set(pairs))
    conditional = len({p.split("\t")[0] for p in now})
    print(f"assumption scan: {total} theorems in {len(mods)} modules, "
          f"{resolved} resolved, {conditional} rest on a domain axiom "
          f"({len(now)} distinct dependencies)")
    if resolved == 0:
        print("::error::assumption scan: resolved 0 theorems -- coqc loaded no "
              "module, so every count above is about an empty set and this gate "
              "checked nothing")
        return 1

    if not BASELINE.exists():
        # Prop. 211c: writing a baseline is an explicit act, never a fallback.
        # `if not exists(): write(now); return 0` resets the ratchet on one
        # `rm`, and on a clone that never had the file it rubber-stamps the tree
        # it was handed and exits 0. Measured before f66561f33: 8 of the 13
        # baselines in this suite were on disk and in no commit, and 8 of the 13
        # gates owning them re-baseline a possibly-broken tree and pass.
        if "--init" not in sys.argv[1:]:
            print(f"::error::assumption scan: {BASELINE.name} does not exist and "
                  f"--init was not given. Writing one here would record "
                  f"whatever this tree contains as the accepted state -- on a "
                  f"fresh clone that is a green run which checked nothing. "
                  f"Genuine first run: `python3 formal/assumption_scan.py --init`. "
                  f"Otherwise the baseline was lost and belongs in the commit "
                  f"that lost it (Prop. 211)")
            return 1
        BASELINE.write_text("\n".join(now) + ("\n" if now else ""))
        print(f"assumption scan: baseline written to {BASELINE.name} "
              f"({len(now)} dependencies)")
        return 0
    was = [l for l in BASELINE.read_text().splitlines()
           if l.strip() and not l.startswith("#")]
    new = [p for p in now if p not in was]
    if new:
        print(f"::error::assumption scan: {len(new)} new theorem->axiom "
              f"dependenc(y/ies). A `Qed` is a statement about a derivation, "
              f"never about the truth of what it assumed -- a proof must not "
              f"acquire an assumption silently (Prop. 204)")
        for p in new[:10]:
            t, a = p.split("\t")
            print(f"  {t}  rests on  {a}")
        return 1
    gone = [w for w in was if w not in now]
    if gone:
        print(f"assumption scan: {len(gone)} dependenc(y/ies) discharged; "
              f"update {BASELINE.name} to lock it in")
    print(f"assumption scan: ratchet holds ({len(now)} <= {len(was)})")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::assumption scan: could not query Coq "
              f"({type(exc).__name__}: {exc}) -- nothing was scanned")
        sys.exit(1)
