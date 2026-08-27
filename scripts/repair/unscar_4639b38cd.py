#!/usr/bin/env python3
"""Restore the source text commit 4639b38cd overwrote with running indices.

The commit replaced the i-th non-ASCII character of each file with the ASCII
digits of `i`. That transform is exactly reproducible from the pre-image, and
reproducing it is what makes the repair safe: a line is restored ONLY when the
line at HEAD is byte-identical to the transform of its pre-image line, so any
line edited since the commit is left alone.
"""
import subprocess, pathlib, sys

def sh(*a):
    return subprocess.run(a, capture_output=True, text=True).stdout

def transform(text):
    out, i = [], 0
    for ch in text:
        if ord(ch) > 127:
            out.append(str(i)); i += 1
        else:
            out.append(ch)
    return "".join(out)

def main(apply):
    parent = sh("git", "rev-parse", "4639b38cd^").strip()
    files = [l for l in sh("git", "show", "--name-only", "--format=", "4639b38cd").split("\n")
             if l.endswith(".t27")]
    touched = restored = 0
    for f in files:
        p = pathlib.Path(f)
        if not p.exists():
            continue
        pre = sh("git", "show", f"{parent}:{f}")
        if not pre:
            continue
        post = transform(pre)
        table = {b: a for a, b in zip(pre.splitlines(), post.splitlines()) if a != b}
        if not table:
            continue
        cur = p.read_text(errors="replace")
        lines = cur.splitlines(keepends=True)
        n = 0
        for k, ln in enumerate(lines):
            stripped = ln.rstrip("\n")
            if stripped in table:
                lines[k] = table[stripped] + ("\n" if ln.endswith("\n") else "")
                n += 1
        if n:
            touched += 1; restored += n
            if apply:
                p.write_text("".join(lines))
    print(f"files touched: {touched}")
    print(f"lines restored: {restored}")
    return 0

if __name__ == "__main__":
    sys.exit(main("--apply" in sys.argv))
