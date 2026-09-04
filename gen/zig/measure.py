#!/usr/bin/env python3
"""One command: rebuild t27c, regenerate the corpus, count failing files.

Run after every single patch to bootstrap/src/compiler.rs. Never apply two
patches between measurements — a flat total can hide one fix and one regression
cancelling out, which has happened on this corpus before.

    python3 gen/zig/measure.py            # measure, print delta vs last run
    python3 gen/zig/measure.py --baseline # measure and reset the stored baseline

The counting method is fixed here on purpose. Counted a looser way, the same run
reports 189 instead of 184, and a delta between two different methods is noise
wearing the costume of progress.
"""
import subprocess, glob, os, sys, json, collections, shutil, re

ROOT = "/Users/playom/t27"
GEN = os.path.join(ROOT, "gen/zig")
STATE = os.path.join(GEN, ".measure_state.json")

def sh(cmd, **kw):
    return subprocess.run(cmd, cwd=kw.pop("cwd", ROOT), capture_output=True, text=True, **kw)

def free_mb():
    st = os.statvfs("/")
    return st.f_bavail * st.f_frsize / 1e6

def build():
    # An unattended loop that fills the disk stops EVERYTHING, including its own
    # ability to report why. Twice today a build ran the volume to zero and no
    # command could write its output file afterwards. Refuse early instead.
    mb = free_mb()
    if mb < 800:
        print(f"  ABORT: {mb:.0f} MB free, need 800. Not building.")
        print("  Free space first; the corpus measurement needs ~400 MB transient.")
        sys.exit(3)
    r = sh(["cargo", "build", "--release", "-p", "t27c"], timeout=900)
    if r.returncode != 0:
        err = [l for l in r.stderr.splitlines() if "error" in l.lower()][:5]
        print("  BUILD FAILED — patch is not applicable as written")
        for l in err:
            print("   ", l[:130])
        sys.exit(2)

def regen():
    specs = sorted(glob.glob(os.path.join(ROOT, "specs", "**", "*.t27"), recursive=True))
    keep = {"all.zig", "build.zig", "measure.py"}
    for p in glob.glob(os.path.join(GEN, "**", "*.zig"), recursive=True):
        if os.path.basename(p) not in keep:
            os.remove(p)
    fails = 0
    for s in specs:
        rel = os.path.relpath(s, os.path.join(ROOT, "specs")).replace(".t27", ".zig")
        dst = os.path.join(GEN, rel)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        g = sh([os.path.join(ROOT, "target/release/t27c"), "gen", s], timeout=60)
        if g.returncode == 0:
            open(dst, "w").write(g.stdout)
        else:
            fails += 1
    return len(specs), fails

def parse_zig_test_output(output_text, gen_dir):
    """Pure parsing step of measure(), split out so it's testable without a
    real `zig test` subprocess and a full corpus. gen_dir is where a path
    ending .zig is checked for existence (see the phantom-import-target note
    below) -- pass measure.py's own GEN in production, a fixture dir in tests.
    """
    files, phantom, classes = set(), set(), collections.Counter()
    for line in output_text.splitlines():
        if ": error:" in line:
            path = line.split(":", 1)[0]
            if path.endswith(".zig"):
                rel = path.replace("gen/zig/", "")
                # A path that does not exist on disk is not a failing file: it is
                # an import target some spec declares and the corpus never
                # provides (`use base::benchmarking;` with no such spec). The
                # emitter is faithful and the defect is in the spec. Counting
                # them inflates the headline -- I attributed two of them to a
                # patch of mine before reading the error.
                (files if os.path.exists(os.path.join(gen_dir, rel)) else phantom).add(rel)
            msg = line.split(": error:", 1)[1].strip()
            classes[msg.split("'")[0].strip()[:52]] += 1
    return files, classes, phantom

def measure():
    r = sh(["zig", "test", "gen/zig/all.zig"], timeout=1800)
    return parse_zig_test_output((r.stderr or "") + "\n" + (r.stdout or ""), GEN)

def silent():
    """Count call sites where the emitter DELETED a method call's receiver.

    measure() cannot see any of these. The emitter turns
    `remainder.abs().compare_abs(b)` into `compare_abs(b)`, and the result
    still COMPILES -- roughly 32 of the 36 known sites raise no error at all.
    Patch K moved this number 36 -> 28 while the error count moved 444 -> 443
    and the file count reported `delta: unchanged`. Judged on that alone the
    patch looks worthless, and reverting it -- which the loop's "revert
    anything that regresses" rule would have made feel disciplined -- would
    have thrown away a correctness repair.

    So: an error count is not evidence in EITHER direction for a defect that
    compiles. This number is printed next to it so the two are never confused.
    """
    tot, files = 0, {}
    specs_root = os.path.join(ROOT, "specs")
    for s in glob.glob(os.path.join(specs_root, "**", "*.t27"), recursive=True):
        g = os.path.join(GEN, os.path.relpath(s, specs_root).replace(".t27", ".zig"))
        if not os.path.exists(g):
            continue
        spec = open(s, errors="ignore").read()
        # DECLARATION lines look exactly like bare calls (`pub fn to_i64(`), so
        # they must be dropped or every method is excluded by construction --
        # that mistake once turned a true count of 14 into 1.
        gen = [l for l in open(g, errors="ignore").read().splitlines()
               if not re.search(r'\bfn\s+[a-z_]\w*\s*\(', l)]
        # The spec's OWN bare calls must be subtracted. `abs`, `round` and `pow`
        # are math shims the specs legitimately call as free functions -- 9, 15
        # and 1 times in three files -- and every one of those was being counted
        # as a deleted receiver. Uncorrected this reported 16 where the truth is
        # 5. The contamination is an additive constant per (file, name), so the
        # DELTAS this printed were always sound; only the level was inflated.
        body = "\n".join(l for l in spec.splitlines()
                         if not re.search(r'\bfn\s+[a-z_]\w*\s*\(', l))
        # `(?<!\.\s)` as well as `(?<![\w.])`: capture_to_semicolon (patch I)
        # walks TOKENS and joins them with single spaces, so an intact call
        # comes out as `( A - B ) . abs ( )`. The character before `abs` is a
        # space, so a plain "not preceded by a dot" test passes and a perfectly
        # correct call is counted as a deleted receiver. That is what turned a
        # true 2 into 5 -- all three `.abs()` sites in gi1_analysis are intact.
        n = 0
        for nm in set(re.findall(r'\)\.([a-z_]\w*)\(', spec)):
            pat = rf'(?<![\w.])(?<!\.\s){re.escape(nm)}\s*\('
            legit = len(re.findall(pat, body))
            found = sum(len(re.findall(pat, l)) for l in gen)
            n += max(0, found - legit)
        if n:
            files[s] = n
            tot += n
    return tot, files

def truncated_bodies():
    """Functions reduced to parameter discards plus a single `return`.

    A parse error inside a function body propagates and the caller's recovery
    discards EVERYTHING, leaving that signature. Two triggers found so far, both
    a single token the parser had no arm for:

        while (i < n) : (i += 1) { ... }   the continue expression  (patch Q)
        for (const xs) |x| { ... }         the const marker         (patch S)

    Mostly SILENT: a body reduced to `return X` still compiles when X resolves,
    and the function simply does nothing. It surfaces as an undeclared identifier
    only when the return happens to reference something the body had declared.
    Counted here because the error columns cannot see the rest.
    """
    # The signature ALONE is not enough: a spec function that really is one
    # statement produces exactly the same shape. Counted without this check the
    # number came out 49; with it, 13 -- so 36 of the original count were honest
    # one-line functions and the headline was wrong by nearly 4x. Confirm against
    # the spec before calling anything truncated.
    def spec_stmts(path, fn):
        # Count STATEMENT TERMINATORS at function depth, not LINES. A single
        # `return LoadResult{ .a = …, .b = … };` spans six lines and a line-based
        # count calls it six statements -- which is how this detector reported 49
        # truncated bodies, then 13 after one correction, when the true number is
        # 0. Every survivor was a multi-line struct-literal return.
        lines = open(path, errors="ignore").read().splitlines()
        for i, l in enumerate(lines):
            if re.search(rf'\bfn {re.escape(fn)}\s*\(', l):
                depth = l.count('{') - l.count('}')
                j, n = i + 1, 0
                while j < len(lines) and depth > 0:
                    s = lines[j].split('//')[0]
                    before = depth
                    depth += s.count('{') - s.count('}')
                    if ';' in s and before == 1 and depth <= 1:
                        n += 1
                    j += 1
                return n
        return None

    hits = []
    for g in glob.glob(os.path.join(GEN, "**", "*.zig"), recursive=True):
        spec_path = os.path.join(ROOT, "specs",
                                 os.path.relpath(g, GEN).replace(".zig", ".t27"))
        lines = open(g, errors="ignore").read().splitlines()
        i = 0
        while i < len(lines):
            if re.match(r'(pub )?fn (\w+)\(.*\{\s*$', lines[i]):
                body, j = [], i + 1
                while j < len(lines) and not lines[j].startswith('}'):
                    if lines[j].strip():
                        body.append(lines[j].strip())
                    j += 1
                meat = [b for b in body
                        if not re.match(r'_ = &?\w+;$', b) and not b.startswith('//')]
                if len(meat) == 1 and meat[0].startswith('return') and len(body) > len(meat):
                    fn = re.match(r'(pub )?fn (\w+)\(', lines[i]).group(2)
                    n = spec_stmts(spec_path, fn) if os.path.exists(spec_path) else None
                    if n is not None and n > 1:
                        hits.append(os.path.relpath(g, GEN))
                i = j
            i += 1
    return hits

if __name__ == "__main__":
    build()
    total, genfail = regen()
    files, classes, phantom = measure()
    n = len(files)

    prev = json.load(open(STATE)) if os.path.exists(STATE) else None
    print(f"  corpus {total} specs   gen failures {genfail}")
    print(f"  real files with >=1 error: {n}   clean: {total - n}  ({100.0*(total-n)/total:.1f}%)")
    print(f"  phantom import targets (not files, spec defects): {len(phantom)}")
    if prev:
        d = n - prev["failing"]
        arrow = "unchanged" if d == 0 else (f"{d:+d}  WORSE" if d > 0 else f"{d:+d}  better")
        print(f"  previous: {prev['failing']}   delta: {arrow}")
        gone = set(prev["files"]) - files
        new = files - set(prev["files"])
        if gone: print(f"  fixed  ({len(gone)}): {sorted(gone)[:6]}")
        if new:  print(f"  BROKEN ({len(new)}): {sorted(new)[:6]}")
        if d == 0 and (gone or new):
            print("  NOTE: total unchanged but the SET moved — one fix and one regression cancelled.")
    print("  top classes:")
    for m, c in classes.most_common(5):
        print(f"    {c:>4}  {m}")

    # SILENT defects -- invisible to everything above. See silent().
    sil, sil_files = silent()
    line = f"  receiver deleted but still compiles: {sil}  (in {len(sil_files)} specs)"
    if prev and "silent" in prev:
        ds = sil - prev["silent"]
        line += "  unchanged" if ds == 0 else (f"  {ds:+d}  WORSE" if ds > 0 else f"  {ds:+d}  better")
    print(line)
    if sil:
        print("    NOTE: these raise no error. Do NOT judge a fix for them by the counts above.")

    trunc = truncated_bodies()
    line = f"  function bodies discarded by a parse error: {len(trunc)}"
    if prev and "truncated" in prev:
        dt = len(trunc) - prev["truncated"]
        line += "  unchanged" if dt == 0 else (f"  {dt:+d}  WORSE" if dt > 0 else f"  {dt:+d}  better")
    print(line)

    if prev is None or "--baseline" in sys.argv:
        json.dump({"failing": n, "files": sorted(files), "silent": sil, "truncated": len(trunc)}, open(STATE, "w"))
        print("  baseline stored")
    else:
        json.dump({"failing": n, "files": sorted(files), "silent": sil, "truncated": len(trunc)}, open(STATE, "w"))
