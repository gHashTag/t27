"""Does every JSON file this repository ships actually parse?

`schema-validation.yml` was named a required check in docs/BRANCH-PROTECTION.md
and its entire body was:

    echo "Validating JSON schemas..."

A required check that cannot fail reads as coverage and is worse than none. This
replaces it with the weakest question worth asking -- does the file parse at all
-- because that one is cheap, has no false theory behind it, and it already finds
something the echo was hiding:

  clara-bridge/audit-trail/experience-schema.json  contains a literal `...` on
  line 40 and cannot parse. clara-bridge/tests/run_tests.py:152 does
  `json.load()` on exactly that path, so load_experience_schema() raises every
  time it is called -- and no workflow runs clara-bridge, so nothing said so.

Empty files are reported separately from malformed ones: an empty artefact is
usually a build that died, not a syntax error, and the two want different fixes.

`external/` is excluded. It is vendored, and TypeScript's tsconfig.json is JSONC
by convention -- comments there are correct, and flagging them would be this
gate making the same mistake it exists to catch.

Usage:
  tools/check_json_parses.py                gate
  tools/check_json_parses.py --self-check   negative control
  tools/check_json_parses.py --update-baseline

Exits non-zero on any new unparseable file.
"""
import json
import os
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BASELINE = ROOT / "tools/json_parse_baseline.txt"
EXCLUDE = ("external/",)


def tracked_json(root=ROOT):
    try:
        out = subprocess.run(["git", "ls-files", "-z", "*.json"], cwd=root,
                             capture_output=True, check=True).stdout
        names = [f.decode() for f in out.split(b"\0") if f]
    except Exception:
        names = [str(p.relative_to(root)) for p in root.rglob("*.json")]
    return [n for n in names if not any(n.startswith(x) for x in EXCLUDE)]


def scan(root=ROOT):
    empty, bad = [], []
    for rel in tracked_json(root):
        p = root / rel
        try:
            raw = p.read_bytes()
        except OSError as e:
            bad.append((rel, f"unreadable: {e}"))
            continue
        if not raw.strip():
            empty.append(rel)
            continue
        try:
            json.loads(raw.decode("utf-8", "replace"))
        except Exception as e:
            bad.append((rel, str(e)[:90]))
    return empty, bad


def baseline():
    if not BASELINE.exists():
        return set()
    return {l.split("|")[0].strip() for l in BASELINE.read_text().splitlines()
            if l.strip() and not l.startswith("#")}


def self_check():
    """Plant a malformed file and prove the scan reports it."""
    import shutil
    import tempfile
    with tempfile.TemporaryDirectory() as td:
        t = pathlib.Path(td)
        (t / "good.json").write_text('{"a": 1}')
        (t / "bad.json").write_text('{"a": 1,,}')
        (t / "empty.json").write_text("")
        empty, bad = scan(t)
        ok = ([b[0] for b in bad] == ["bad.json"]) and (empty == ["empty.json"])
    print(f"  self-check: malformed caught = {[b[0] for b in bad] == ['bad.json']}, "
          f"empty caught = {empty == ['empty.json']}, good file silent = {len(bad) + len(empty) == 2}")

    # Everything above proves scan(). Nothing above proves that a non-empty
    # result becomes a non-zero EXIT CODE -- that wiring is main()'s `return 1`,
    # which no case above reaches. Measured on check_catalog_integrity.py: with
    # main()'s `return 1` rewritten to `return 0` the gate printed OK on a
    # broken catalog while its control still reported every branch red.
    #
    # So run the WHOLE program. The script is COPIED into the planted tree, so
    # its module-level ROOT resolves there by the ordinary parent.parent rule --
    # no --root flag and no environment override, either of which would add a
    # way to aim the LIVE gate at somewhere harmless.
    def spawned(label, want, expect, absent, files):
        """Run the whole program on a planted tree; demand the exit AND the text.

        `expect` is text that must appear and `absent` names the markers of the
        sibling branches. main() reaches its exit code from four places -- the
        OK line, the FAIL list, `--update-baseline`, and this control -- and the
        exit code alone cannot tell them apart, so a fault reaching the right
        code through the wrong branch would otherwise pass.
        """
        nonlocal ok
        me = pathlib.Path(__file__).name
        with tempfile.TemporaryDirectory() as td:
            root = pathlib.Path(td)
            (root / "tools").mkdir()
            shutil.copy(__file__, root / "tools" / me)
            for rel, body in files.items():
                (root / rel).write_text(body, encoding="utf-8")
            # tracked_json() asks git first and only falls back to rglob when
            # that raises, so the planted tree is made a real repository and the
            # control exercises the path the gate actually takes here. `-f`
            # because a global ignore rule must not silently empty the tree. If
            # git is missing both calls fail, `git ls-files` then raises inside
            # the gate, and the rglob fallback finds the same files.
            for argv in (["git", "init", "-q"], ["git", "add", "-A", "-f"]):
                subprocess.run(argv, cwd=root, capture_output=True)
            proc = subprocess.run([sys.executable, str(root / "tools" / me)],
                                  capture_output=True, text=True)
        missing = [e for e in expect if e not in proc.stdout]
        leaked = [a for a in absent if a in proc.stdout]
        good = proc.returncode == want and not missing and not leaked
        print("  %-26s %s" % (label, "exit %d, says it" % want if good else "CONTROL FAILED"))
        if not good:
            ok = False
            print("       exit %r (want %r)" % (proc.returncode, want))
            if missing:
                print("       expected text absent: %r" % (missing,))
            if leaked:
                print("       neighbouring marker leaked: %r" % (leaked,))
            print("       stdout %r" % (proc.stdout[:400],))

    # A clean planted tree must exit 0 and say OK, or the two red cases below
    # would pass for a gate that is red on everything.
    spawned("end-to-end clean tree", 0,
            ["OK: 1 tracked JSON files, none newly unparseable"],
            ("FAIL:", "baseline written:", "empty file"),
            {"good.json": '{"a": 1}'})

    # main()'s `return 1`, reached through the malformed list. The closing
    # advice is asserted too: it is what distinguishes this branch's text from
    # every other non-zero exit the file can produce.
    spawned("end-to-end malformed", 1,
            ["FAIL: 1 JSON file(s) do not parse",
             "  broken.json",
             "rename it (.jsonc, .md) rather than adding it to the baseline"],
            ("OK:", "baseline written:", "empty file"),
            {"good.json": '{"a": 1}', "broken.json": '{"a": 1,,}'})

    # The same `return 1` reached through the OTHER list. Both prints funnel to
    # one return, so each case names the other's marker as forbidden -- the
    # shared "FAIL:" prefix cannot tell them apart.
    spawned("end-to-end empty file", 1,
            ["FAIL: 1 JSON file(s) do not parse",
             "  hollow.json",
             "empty file — a build that died, or a placeholder never filled"],
            ("OK:", "baseline written:", "Expecting"),
            {"good.json": '{"a": 1}', "hollow.json": ""})

    # NOT covered here: that a file already listed in the baseline is filtered
    # OUT of new_bad. Breaking that filter turns baselined debt into a false
    # RED, not a false green, and it is not a return site -- so it is named
    # rather than tested, instead of being left for a reader to infer.
    return 0 if ok else 1


def main():
    if "--self-check" in sys.argv:
        return self_check()
    empty, bad = scan()
    total = len(tracked_json())

    if "--update-baseline" in sys.argv:
        BASELINE.write_text(
            "# JSON files that do not parse today. Each line is a debt, not a rule.\n"
            "# Remove the line when the file is fixed; the gate then holds it fixed.\n"
            + "".join(f"{rel} | {why}\n" for rel, why in sorted(bad))
            + "".join(f"{rel} | empty file\n" for rel in sorted(empty)))
        print(f"  baseline written: {len(bad) + len(empty)} entries")
        return 0

    known = baseline()
    new_bad = [(r, w) for r, w in bad if r not in known]
    new_empty = [r for r in empty if r not in known]
    if not new_bad and not new_empty:
        print(f"OK: {total} tracked JSON files, none newly unparseable "
              f"({len(bad) + len(empty)} known, listed in {BASELINE.name})")
        return 0
    print(f"FAIL: {len(new_bad) + len(new_empty)} JSON file(s) do not parse\n")
    for rel, why in new_bad:
        print(f"  {rel}\n      {why}")
    for rel in new_empty:
        print(f"  {rel}\n      empty file — a build that died, or a placeholder never filled")
    print("\n  Fix the file. If it is a documentation example that only looks like JSON,")
    print("  rename it (.jsonc, .md) rather than adding it to the baseline.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
