#!/usr/bin/env python3
"""What the pinned tri-api keeps between runs: specs/api/tri_api_session.t27 held to the source, to the
compiled Zig of session_store.zig and memory.zig, and to the binary driven against a scripted provider.

WHY THIS EXISTS
---------------
S08 of gHashTag/trinity#988 (gHashTag/t27#4827, re-filed from #3570). tri-api writes a record of every
conversation, reads a memory file into its system prompt and stashes a file before writing it. Three
designs in this repository describe durable state -- specs/organism/dna.tri and mozg.tri, which do not
parse, specs/brain/unified_state.t27, which nothing persists, and specs/memory/tmem/session.t27, whose
TMSS record nothing writes -- and none of them was held to what tri-api does. This tool measures what it
does, and holds the session spec, and the tmem session spec's own conformance file, to the measurement.

THREE KINDS OF EVIDENCE, KEPT APART
-----------------------------------
  model    a Python reading of SessionStore.save/load/loadLatest and Memory.load, byte for byte
  zig      session_store.zig and memory.zig at the pin compiled with Zig 0.15.2 and run with generated
           `test` blocks appended to a copy of each: save/load round trips, every prefix of a saved
           record loaded back, the memory caps; Debug, ReleaseSafe, ReleaseFast, ReleaseSmall
  binary   the real tri-api built from the pinned main.zig, driven by tools/trinity_tri_api.py's scripted
           Messages server in a private HOME and working directory: twelve scenarios, real files
Nothing here talks to a real provider and no credential appears anywhere; the key is a fake.

Usage:
  python3 tools/trinity_tri_api_session.py zig  --trinity-root <clone> --zig <zig> [--sysroot <dir>]
  python3 tools/trinity_tri_api_session.py e2e  --trinity-root <clone> --zig <zig> [--sysroot <dir>]
  python3 tools/trinity_tri_api_session.py run
  python3 tools/trinity_tri_api_session.py check
  python3 tools/trinity_tri_api_session.py --self-check [--trinity-root <clone> --zig <zig>]

Exit codes: 0 no finding; 1 findings; 2 could not run.
"""
from __future__ import annotations

import argparse
import json
import os
import pathlib
import pty
import re
import resource
import select
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
import threading
import time

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import trinity_tri_api as base  # noqa: E402  (the S07 harness: model of the protocol, fake provider, builds)

ROOT = base.ROOT
SPEC = ROOT / "specs/api/tri_api_session.t27"
TMEM_SPEC = ROOT / "specs/memory/tmem/session.t27"
TMEM_CONF = ROOT / "conformance/tmem_session.json"
OUT_DIR = ROOT / "conformance/trinity"
EVIDENCE_ZIG = OUT_DIR / "tri_api_session_zig.json"
EVIDENCE_E2E = OUT_DIR / "tri_api_session_e2e.json"
PIN = base.DEFAULT_REV
ZIG_MODES = ("Debug", "ReleaseSafe", "ReleaseFast", "ReleaseSmall")


# ===========================================================================================
# MODEL: session_store.zig and memory.zig, byte for byte (escape/extract/unescape are S07's model)
# ===========================================================================================
PREVIEW_MAX = 80
MEMORY_MAX_LINES = 200
MEMORY_MAX_BYTES = 256 * 1024


def record_body(messages: bytes, sid: bytes, ts: int) -> bytes:
    """SessionStore.save: {"id":"<id>","ts":<ts>,"messages":"<escaped messages>"}"""
    return b'{"id":"' + sid + b'","ts":' + str(ts).encode() + b',"messages":"' + base.json_escape(messages) + b'"}'


def session_id(ts: int) -> bytes:
    return b"%08x" % (ts & 0xFFFFFFFF)


def index_entry(sid: bytes, ts: int, prompt: bytes) -> bytes:
    return b'{"id":"' + sid + b'","ts":' + str(ts).encode() + b',"preview":"' + base.json_escape(prompt[:PREVIEW_MAX]) + b'"}'


def load(content: bytes):
    """SessionStore.load after the file is read: extractField(content, "messages") then unescapeString."""
    v = base.extract_field(content, b"messages")
    return None if v is None else base.unescape_string(v)


def resume_inputs(content):
    """(value_found, ends_with_bracket) as main.zig sees a loaded record; content None = unreadable."""
    if content is None:
        return False, False
    v = load(content)
    if v is None:
        return False, False
    return True, len(v) > 1 and v[-1:] == b"]"


def load_class(content: bytes, original: bytes) -> str:
    v = load(content)
    if v is None:
        return "none"
    if v == original:
        return "equal"
    return "bracket" if (len(v) > 1 and v[-1:] == b"]") else "partial"


def memory_load(content: bytes):
    """Memory.load: None past 256 KiB, else the first 200 lines (the 200th newline included)."""
    if len(content) > MEMORY_MAX_BYTES:
        return None
    count, end = 0, 0
    while end < len(content):
        if content[end:end + 1] == b"\n":
            count += 1
            if count >= MEMORY_MAX_LINES:
                end += 1
                break
        end += 1
    return content[:end] if end < len(content) else content


def runs(classes: list) -> list:
    """[[class, first_k, last_k], ...] -- a prefix sweep compressed into runs."""
    out = []
    for k, c in enumerate(classes):
        if out and out[-1][0] == c and out[-1][2] == k - 1:
            out[-1][2] = k
        else:
            out.append([c, k, k])
    return out


# the messages strings saved and loaded back in Zig; each is what the loop could hold, or a deliberate edge
ROUNDTRIP = [
    ("rt-simple", b'[{"role":"user","content":"hello"}]'),
    ("rt-tool-cycle", b'[{"role":"user","content":"read it"},{"role":"assistant","content":[{"type":"tool_use","id":"toolu_1","name":"read_file","input":{"path":"a.txt"}}]},{"role":"user","content":[{"type":"tool_result","tool_use_id":"toolu_1","content":"hello\\n"}]}]'),
    ("rt-escapes", b'[{"role":"user","content":"q \\" bs \\\\ nl \\n tab \\t u \\u00e9 ctl \\u0001"}]'),
    ("rt-trailing-backslash", b'[{"role":"user","content":"C:\\\\"}]'),
    ("rt-utf8", '[{"role":"user","content":"\u043f\u0440\u0438\u0432\u0435\u0442 \u4e16\u754c"}]'.encode()),
    ("rt-raw-newline", b'[\n {"role":"user","content":"pretty"}\n]'),
    ("rt-raw-control", b'[{"role":"user","content":"a\x01b"}]'),
    ("rt-large", b'[{"role":"user","content":"' + b"x" * 1000000 + b'"}]'),
]
SWEEP_MESSAGES = ROUNDTRIP[1][1]
SWEEP_ID, SWEEP_TS = b"aaaaaaaa", 1790000000
MEMORY_CASES = [
    ("mem-small", b"remember: X\n"),
    ("mem-250-lines", b"".join(b"mem line %d\n" % i for i in range(250))),
    ("mem-no-final-newline", b"one\ntwo"),
    ("mem-at-cap", b"m" * (MEMORY_MAX_BYTES - 1) + b"\n"),
    ("mem-over-cap", b"m" * MEMORY_MAX_BYTES + b"\n"),
]


# ===========================================================================================
# INVENTORY: the constants the spec restates, read out of the pinned Zig
# ===========================================================================================
def source_constants(trinity: pathlib.Path, rev: str) -> dict:
    g = lambda f: base.git_show(trinity, rev, f"src/tri-api/{f}.zig").decode()
    ss, mem, ctx = g("session_store"), g("memory"), g("context")
    return {
        "sessions_subdir": re.search(r'const sessions_subdir = "([^"]+)";', ss).group(1),
        "index_filename": re.search(r'const index_filename = "([^"]+)";', ss).group(1),
        "session_max_bytes": eval(re.search(r"const max_file_size = ([\d\s\*]+);", ss).group(1)),
        "preview_max": int(re.search(r"@min\(prompt\.len, (\d+)\)", ss).group(1)),
        "id_hex_digits": int(re.search(r'\{x:0>(\d+)\}', ss).group(1)),
        "memory_max_lines": int(re.search(r"const max_lines = (\d+);", mem).group(1)),
        "memory_max_bytes": eval(re.search(r"const max_file_size = ([\d\s\*]+);", mem).group(1)),
        "truncate_keep": int(re.search(r"if \(original_len > (\d+)\)", ctx).group(1)),
        "memory_append_callers": sum(len(re.findall(r"\bmem\.append\(", g(f))) for f in base.SRC_FILES if f != "memory"),
    }


# ===========================================================================================
# BINARY: scenarios against the real tri-api
# ===========================================================================================
def drive(binary, args, replies, home, cwd, extra_env=None, timeout=90, preexec=None, stdin=None, sigterm_after=None):
    fp = base.FakeProvider(replies)
    env = {"PATH": os.environ["PATH"], "HOME": str(home), "ANTHROPIC_API_KEY": "fake-key-not-real",
           "ANTHROPIC_BASE_URL": f"http://127.0.0.1:{fp.port}"}
    env.update(extra_env or {})

    def pre():
        os.umask(0o022)
        if preexec:
            preexec()
    t0 = time.time()
    pr = subprocess.Popen([str(binary)] + list(args), cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                          stdin=subprocess.PIPE if stdin is not None else subprocess.DEVNULL, preexec_fn=pre)
    try:
        if sigterm_after is not None:
            time.sleep(sigterm_after)
            pr.send_signal(signal.SIGTERM)
        out, err = pr.communicate(input=stdin.encode() if stdin is not None else None, timeout=timeout)
        rc = pr.returncode
    except subprocess.TimeoutExpired:
        pr.kill()
        out, err = pr.communicate()
        rc = "timeout"
    fp.close()
    return {"rc": rc, "out": out.decode("utf-8", "replace"), "err": err.decode("utf-8", "replace"), "reqs": fp.requests,
            "secs": time.time() - t0}


def fresh():
    return pathlib.Path(tempfile.mkdtemp(prefix="s08h-")), pathlib.Path(tempfile.mkdtemp(prefix="s08c-"))


def sdir(home):
    return home / ".trinity/api/sessions"


def record_files(home):
    d = sdir(home)
    return sorted(f for f in os.listdir(d) if f != "index.json") if d.is_dir() else []


def read_index(home):
    p = sdir(home) / "index.json"
    if not p.exists():
        return None, None
    raw = p.read_bytes()
    try:
        return json.loads(raw.decode("utf-8")), True
    except (UnicodeDecodeError, ValueError):
        return None, False


def body(r, i=0):
    """The i-th request as JSON, the string INVALID-JSON when it does not parse, None when absent."""
    if len(r["reqs"]) <= i:
        return None
    try:
        return json.loads(r["reqs"][i]["body"])
    except ValueError:
        return "INVALID-JSON"


def roles(r, i=0):
    b = body(r, i)
    return b if not isinstance(b, dict) else [m["role"] for m in b["messages"]]


def first_mark(r):
    m = [x for x in base.marks(r["err"]) if x.startswith(("Resuming session", "No session found"))]
    return m[0] if m else None


def saved_line(r):
    return any(l.startswith("[tri-api] Session saved: ") for l in r["err"].splitlines())


def write_record(home, sid, messages: bytes, ts=SWEEP_TS, extra=b""):
    sdir(home).mkdir(parents=True, exist_ok=True)
    b = record_body(messages, sid.encode(), ts)
    if extra:
        b = b.replace(b',"messages":', extra + b',"messages":', 1)
    (sdir(home) / f"{sid}.json").write_bytes(b)
    return b


def write_index(home, ids):
    sdir(home).mkdir(parents=True, exist_ok=True)
    (sdir(home) / "index.json").write_bytes(b"[" + b",".join(index_entry(i.encode(), SWEEP_TS, b"p") for i in ids) + b"]")


def mode_of(p):
    return stat.S_IMODE(pathlib.Path(p).stat().st_mode)


SCENARIOS = []


def scenario(sid, finding, modes=("Debug",)):
    def deco(fn):
        SCENARIOS.append({"id": sid, "finding": finding, "modes": modes, "fn": fn})
        return fn
    return deco


ALLOW = '{"permissions":{"allow":["write_file(*)","bash(*)"]}}'
VOLATILE = {"id", "ts"}            # the clock: asserted against each other (c01), never against a rerun
TWO_TURNS = json.dumps([{"role": "user", "content": "first"}, {"role": "assistant", "content": [{"type": "text", "text": "hi"}]}],
                       separators=(",", ":")).encode()


@scenario("s01-record-shape", "f35")
def s01(b):
    home, cwd = fresh()
    prompt = "a" + "\u0436" * 45                      # 91 bytes: the 80-byte preview cuts the 40th character in half
    r = drive(b, [prompt], [base.text_reply("ok")], home, cwd)
    files = record_files(home)
    rec = json.loads((sdir(home) / files[0]).read_bytes()) if files else {}
    raw_index = (sdir(home) / "index.json").read_bytes() if (sdir(home) / "index.json").exists() else b""
    preview = raw_index.split(b'"preview":"', 1)[1].rsplit(b'"}', 1)[0] if b'"preview":"' in raw_index else b""
    try:
        raw_index.decode("utf-8")
        utf8 = True
    except UnicodeDecodeError:
        utf8 = False
    return {"rc": r["rc"], "records": len(files), "record_bytes": (sdir(home) / files[0]).stat().st_size if files else None,
            "record_keys": list(rec), "id_is_hex_of_ts": rec.get("id") == session_id(rec.get("ts", 0)).decode(),
            "ts": rec.get("ts"), "id": rec.get("id"), "saved_line": saved_line(r), "prompt_bytes": len(prompt.encode()),
            "preview_bytes": len(preview), "index_valid_utf8": utf8, "record_mode": mode_of(sdir(home) / files[0]) if files else None,
            "index_mode": mode_of(sdir(home) / "index.json") if raw_index else None, "dir_mode": mode_of(sdir(home)),
            "messages_roundtrip": load((sdir(home) / files[0]).read_bytes()) == json.dumps(
                [{"role": "user", "content": prompt}, {"role": "assistant", "content": [{"type": "text", "text": "ok"}]}],
                separators=(",", ":"), ensure_ascii=False).encode() if files else None}


@scenario("s02-resume", "f36-f37")
def s02(b):
    home, cwd = fresh()
    (cwd / "a.txt").write_text("hello\n")
    r1 = drive(b, ["--model", "claude-test-model", "read it"],
               [base.tool_reply([base.tu("toolu_1", "read_file", {"path": "a.txt"})]), base.text_reply("It says hello.")], home, cwd)
    first_id = record_files(home)[0][:-5] if record_files(home) else None
    time.sleep(1.1)
    r2 = drive(b, ["--continue", "and now?"], [base.text_reply("again")], home, cwd)
    time.sleep(1.1)
    r3 = drive(b, ["--resume", first_id or "x", "third"], [base.text_reply("ok")], home, cwd)
    r4 = drive(b, ["--resume", "deadbeef", "fourth"], [base.text_reply("fresh")], home, cwd)
    b1, b2 = body(r1), body(r2)
    pairs = lambda bb: [c.get("type") for m in bb["messages"] if isinstance(m["content"], list) for c in m["content"]]
    idx, _ = read_index(home)
    return {"run1_model": b1["model"], "run2_mark": first_mark(r2), "run2_model": b2["model"], "run2_roles": roles(r2), "run2_blocks": pairs(b2),
            "run2_last_user": b2["messages"][-1], "run3_mark": first_mark(r3), "run3_roles": roles(r3), "run4_mark": first_mark(r4),
            "run4_roles": roles(r4), "run4_rc": r4["rc"], "records_after_4_runs": len(record_files(home)),
            "index_entries_after_4_runs": len(idx) if idx else None, "distinct_ids": len({e["id"] for e in idx}) if idx else None}


@scenario("s03-same-second", "f30")
def s03(b, trials=10, n=8):
    out = []
    for _ in range(trials):
        home, cwd = fresh()
        while time.time() % 1.0 > 0.05:               # start the eight just after a second begins
            time.sleep(0.005)
        res = [None] * n

        def go(i):
            res[i] = drive(b, ["run %d" % i], [base.text_reply("ok %d" % i)], home, cwd)
        th = [threading.Thread(target=go, args=(i,)) for i in range(n)]
        [t.start() for t in th]
        [t.join() for t in th]
        idx, valid = read_index(home)
        ids = [e["id"] for e in idx] if idx else []
        out.append({"rc0": all(r["rc"] == 0 for r in res), "records": len(record_files(home)), "distinct_ids": len(set(ids)), "index_entries": len(ids),
                    "index_valid": valid, "every_id_has_a_record": all(f"{i}.json" in record_files(home) for i in set(ids))})
    return {"runs": n, "trials": trials, "all_rc0": all(t["rc0"] for t in out), "all_index_valid": all(t["index_valid"] for t in out),
            "records_equal_ids": all(t["records"] == t["distinct_ids"] and t["every_id_has_a_record"] for t in out),
            "single_id_trials": sum(1 for t in out if t["distinct_ids"] == 1), "fewer_records_than_runs": all(t["records"] < n for t in out),
            "entries_at_most_runs": all(t["index_entries"] <= n for t in out), "trials_losing_entries": sum(1 for t in out if t["index_entries"] < n),
            "~per_trial": out}


def fsize_limit(n):
    def f():
        signal.signal(signal.SIGXFSZ, signal.SIG_IGN)
        resource.setrlimit(resource.RLIMIT_FSIZE, (n, n))
    return f


@scenario("s04-interrupted-write", "f31")
def s04(b):
    home, cwd = fresh()
    limit = 1000
    r = drive(b, ["x" * 2500], [base.text_reply("ok")], home, cwd, preexec=fsize_limit(limit))
    files = record_files(home)
    size = (sdir(home) / files[0]).stat().st_size if files else None
    sid = files[0][:-5] if files else "none"
    index_exists = (sdir(home) / "index.json").exists()
    state = {f: (sdir(home) / f).read_bytes() for f in os.listdir(sdir(home))} if sdir(home).is_dir() else {}

    def after(args):                                    # each follow-up starts from its own copy of what the cut left
        h, c = fresh()
        sdir(h).mkdir(parents=True)
        for f, data in state.items():
            (sdir(h) / f).write_bytes(data)
        return drive(b, args, [base.text_reply("again")], h, c)
    r2 = after(["--continue", "next"])
    r3 = after(["--resume", sid, "next"])
    errs = [l for l in r["err"].splitlines() if l.startswith("[tri-api]") and ("session" in l.lower() or "save" in l.lower() or "error" in l.lower())]
    return {"limit": limit, "rc": r["rc"], "records": len(files), "record_bytes": size, "index_exists": index_exists,
            "saved_line": saved_line(r), "tri_api_lines_about_saving": errs, "continue_mark": first_mark(r2), "continue_roles": roles(r2),
            "resume_mark": first_mark(r3), "resume_roles": roles(r3)}


@scenario("s05-damaged-records", "f32-f33-f34")
def s05(b):
    out = {}

    def cont(home, cwd, args=("--continue", "next")):
        r = drive(b, list(args), [base.text_reply("again")], home, cwd)
        return {"mark": first_mark(r), "roles": roles(r), "rc": r["rc"]}
    cases = {
        "not-an-array": lambda h: (write_record(h, "aaaaaaaa", b'"hello"'), write_index(h, ["aaaaaaaa"])),
        "garbage-ending-in-bracket": lambda h: (write_record(h, "aaaaaaaa", b"garbage]"), write_index(h, ["aaaaaaaa"])),
        "last-entry-missing": lambda h: (write_record(h, "aaaaaaaa", TWO_TURNS), write_index(h, ["aaaaaaaa", "bbbbbbbb"])),
        "version-99": lambda h: (write_record(h, "aaaaaaaa", TWO_TURNS, extra=b',"version":99,"magic":"TMSS"'), write_index(h, ["aaaaaaaa"])),
        "tmss-header": lambda h: (sdir(h).mkdir(parents=True, exist_ok=True), (sdir(h) / "aaaaaaaa.json").write_bytes(b"TMSS" + bytes(172)), write_index(h, ["aaaaaaaa"])),
    }
    for name, make in cases.items():
        home, cwd = fresh()
        make(home)
        out[name] = cont(home, cwd)
    home, cwd = fresh()                                # a newer record that never reached the index
    write_record(home, "aaaaaaaa", TWO_TURNS)
    write_index(home, ["aaaaaaaa"])
    write_record(home, "bbbbbbbb", json.dumps([{"role": "user", "content": "NEWEST"}, {"role": "assistant", "content": [{"type": "text", "text": "x"}]}], separators=(",", ":")).encode())
    r = drive(b, ["--continue", "next"], [base.text_reply("again")], home, cwd)
    bb = body(r)
    out["newer-record-not-indexed"] = {"mark": first_mark(r), "resumed_first_content": bb["messages"][0]["content"] if isinstance(bb, dict) else bb}
    home, cwd = fresh()                                # the record of the Zig sweep, cut at both ends of every run
    rec = write_record(home, SWEEP_ID.decode(), SWEEP_MESSAGES)
    sweep = []
    bounds = runs([load_class(rec[:k], SWEEP_MESSAGES) for k in range(len(rec) + 1)])
    for c in sorted({k for _, lo, hi in bounds for k in (lo, hi)}):
        (sdir(home) / "aaaaaaaa.json").write_bytes(rec[:c])
        r = drive(b, ["--resume", SWEEP_ID.decode(), "next"], [base.text_reply("again")], home, cwd)
        sweep.append({"cut": c, "of": len(rec), "class": load_class(rec[:c], SWEEP_MESSAGES), "mark": first_mark(r), "roles": roles(r)})
    out["cuts"] = sweep
    return out


@scenario("s06-memory", "f39")
def s06(b):
    out = {}
    for name, content in MEMORY_CASES[:2] + MEMORY_CASES[4:]:
        home, cwd = fresh()
        (home / ".tri-api").mkdir()
        (home / ".tri-api/MEMORY.md").write_bytes(content)
        r = drive(b, ["hi"], [base.text_reply("ok")], home, cwd)
        sys_ = (body(r) or {}).get("system") or ""
        out[name] = {"bytes": len(content), "lines": content.count(b"\n"), "memory_heading": "# Memory" in sys_,
                     "memory_lines_in_system": sys_.count("mem line") if "250" in name else (1 if "remember: X" in sys_ else 0),
                     "file_bytes_after": (home / ".tri-api/MEMORY.md").stat().st_size}
    home, cwd = fresh()
    r = drive(b, ["hi", "remember", "this"], [base.text_reply("noted")], home, cwd)
    out["no-file"] = {"created": (home / ".tri-api/MEMORY.md").exists(), "system": (body(r) or {}).get("system")}
    return out


@scenario("s07-system-provenance", "f37")
def s07(b):
    home, cwd = fresh()
    (home / ".claude").mkdir()
    (home / ".claude/CLAUDE.md").write_text("GLOBAL-RULES")
    (cwd / "CLAUDE.md").write_text("PROJECT-RULES")
    (cwd / ".claude").mkdir()
    (cwd / ".claude/CLAUDE.md").write_text("LOCAL-RULES")
    (home / ".tri-api").mkdir()
    (home / ".tri-api/MEMORY.md").write_text("MEMORY-A")
    r1 = drive(b, ["--model", "claude-test-model", "first"], [base.text_reply("ok")], home, cwd)
    (home / ".tri-api/MEMORY.md").write_text("MEMORY-B")
    (cwd / "CLAUDE.md").write_text("PROJECT-RULES-CHANGED")
    time.sleep(1.1)
    r2 = drive(b, ["--continue", "second"], [base.text_reply("ok")], home, cwd)
    raws = b"".join((sdir(home) / f).read_bytes() for f in record_files(home))
    return {"run1_system": body(r1)["system"], "run2_system": body(r2)["system"], "run1_model": body(r1)["model"], "run2_model": body(r2)["model"],
            "records_hold": {k: k.encode() in raws for k in ("GLOBAL-RULES", "PROJECT-RULES", "LOCAL-RULES", "MEMORY-A", "claude-test-model")}}


@scenario("s08-privacy", "f38")
def s08(b):
    home, cwd = fresh()
    tok = "sk-ant-" + "a" * 24
    (cwd / "cfg.txt").write_text("token=" + tok + "\n")
    (home / ".tri-api").mkdir()
    (home / ".tri-api/MEMORY.md").write_text("MEMORY-SECRET-NOTE")
    (cwd / "CLAUDE.md").write_text("PROJECT-RULES-NOTE")
    r = drive(b, ["read cfg"], [base.tool_reply([base.tu("t1", "read_file", {"path": "cfg.txt"})]), base.text_reply("ok")], home, cwd)
    raws = b"".join((sdir(home) / f).read_bytes() for f in record_files(home))
    audit = (home / ".tri-api/audit.jsonl").read_bytes() if (home / ".tri-api/audit.jsonl").exists() else b""
    sent = "".join(q["body"] for q in r["reqs"])
    first = json.loads(audit.splitlines()[0]) if audit else {}
    return {"token_in_record": tok.encode() in raws, "token_sent": tok in sent, "record_mode": mode_of(sdir(home) / record_files(home)[0]),
            "audit_entry": first, "token_in_audit": tok.encode() in audit, "audit_mode": mode_of(home / ".tri-api/audit.jsonl") if audit else None,
            "in_requests": {"record-content": "token=" in sent, "preview-content": "read cfg" in sent, "memory": "MEMORY-SECRET-NOTE" in sent,
                            "claude-md": "PROJECT-RULES-NOTE" in sent, "audit-verdict-line": json.dumps(first, separators=(",", ":")) in sent}}


def tr(tid, content, err=False):
    return {"type": "tool_result", "tool_use_id": tid, "content": content, **({"is_error": True} if err else {})}


def asst(tid, name="bash", inp=None):
    return {"role": "assistant", "content": [{"type": "tool_use", "id": tid, "name": name, "input": inp or {"command": "ls"}}]}


def long_history(first_result, n_big, big_len, recent_len) -> bytes:
    msgs = [{"role": "user", "content": "ORIGINAL-PROMPT do the task"}]
    msgs += [asst("t1"), {"role": "user", "content": [tr("t1", first_result)]}]
    msgs += [asst("t2", "write_file", {"path": "secret.txt", "content": "y"}),
             {"role": "user", "content": [tr("t2", "Permission denied: write_file(secret.txt)", True)]}]
    for k in range(n_big):
        msgs += [asst("b%d" % k), {"role": "user", "content": [tr("b%d" % k, "B" * big_len)]}]
    for k in range(3):
        msgs += [asst("r%d" % k), {"role": "user", "content": [tr("r%d" % k, "R" * recent_len)]}]
    return json.dumps(msgs, separators=(",", ":")).encode()


@scenario("s09-compaction", "f41-f42-f43")
def s09(b):
    out = {}
    cases = {"truncate-clean": (long_history("A" * 300, 12, 60000, 1000), [base.text_reply("done")]),
             "truncate-trailing-backslash": (long_history("A" * 300 + "\\", 12, 60000, 1000), [base.text_reply("done")]),
             "summary-claims-done": (long_history("A" * 300, 2, 1000, 200000),
                                     [base.text_reply("SUMMARY: write_file(secret.txt) completed successfully; all done."), base.text_reply("done")])}
    for name, (hist, replies) in cases.items():
        home, cwd = fresh()
        write_record(home, "aaaaaaaa", hist)
        write_index(home, ["aaaaaaaa"])
        r = drive(b, ["--continue", "next step"], replies, home, cwd, timeout=180)
        reqs = []
        for i in range(len(r["reqs"])):
            bb = body(r, i)
            q = r["reqs"][i]["body"]
            if not isinstance(bb, dict):
                reqs.append({"valid_json": False})
                continue
            ms = bb["messages"]
            uses = [c["id"] for m in ms if isinstance(m["content"], list) for c in m["content"] if c.get("type") == "tool_use"]
            res = [c["tool_use_id"] for m in ms if isinstance(m["content"], list) for c in m["content"] if c.get("type") == "tool_result"]
            reqs.append({"valid_json": True, "has_tools": "tools" in bb, "messages": len(ms),
                         "first_head": (ms[0]["content"] if isinstance(ms[0]["content"], str) else "blocks")[:26],
                         "denial_kept": "Permission denied: write_file(secret.txt)" in q, "denial_is_error_kept": '"content":"Permission denied: write_file(secret.txt)","is_error":true' in q,
                         "original_prompt_kept": "ORIGINAL-PROMPT" in q, "truncation_markers": q.count("[truncated "),
                         "orphan_results": [x for x in res if x not in uses], "claims_completed": "completed successfully" in q})
        saved = [f for f in record_files(home) if f != "aaaaaaaa.json"]
        rec = {}
        if saved:
            msg = load((sdir(home) / saved[0]).read_bytes()) or b""
            try:
                json.loads(msg)
                valid = True
            except ValueError:
                valid = False
            rec = {"valid_json": valid, "original_prompt": b"ORIGINAL-PROMPT" in msg, "denial": b"Permission denied: write_file(secret.txt)" in msg,
                   "claims_completed": b"completed successfully" in msg}
        time.sleep(1.1)
        r2 = drive(b, ["--continue", "again"], [base.text_reply("x"), base.text_reply("y")], home, cwd, timeout=180)
        b2 = body(r2)
        out[name] = {"history_bytes": len(hist), "requests": reqs, "marks": [m for m in base.marks(r["err"]) if "compacted" in m],
                     "saved_record": rec, "next_continue_request_valid": isinstance(b2, dict)}
    return out


def git(cwd, cmd):
    subprocess.run(cmd, shell=True, cwd=cwd, check=True, capture_output=True)


def stashes(cwd):
    return subprocess.run("git stash list", shell=True, cwd=cwd, capture_output=True, text=True).stdout.strip().splitlines()


@scenario("s10-checkpoint", "f40", modes=("ReleaseFast",))
def s10(b):
    out = {}
    init = "git init -q . && git config user.email a@b && git config user.name n && echo v1 > tracked.txt && git add -A && git commit -q -m init"

    def write(cwd, home, path, replies_extra=()):
        (cwd / ".trinity/api").mkdir(parents=True, exist_ok=True)
        (cwd / ".trinity/api/settings.json").write_text(ALLOW)
        reps = [base.tool_reply([base.tu("t1", "write_file", {"path": path, "content": "NEW"})])] + list(replies_extra) + [base.text_reply("ok")]
        return drive(b, ["w"], reps, home, cwd)
    home, cwd = fresh()
    git(cwd, init + " && echo PRECIOUS > untracked.txt")
    r = write(cwd, home, "untracked.txt")
    out["untracked"] = {"content": (cwd / "untracked.txt").read_text(), "stashes": len(stashes(cwd)),
                        "stash_failed_line": any("checkpoint: stash failed" in l for l in r["err"].splitlines())}
    home, cwd = fresh()
    (cwd / "f.txt").write_text("OLD")
    r = write(cwd, home, "f.txt")
    out["no-repo"] = {"content": (cwd / "f.txt").read_text(), "checkpoint_lines": [l for l in r["err"].splitlines() if "checkpoint" in l]}
    home, cwd = fresh()
    git(cwd, init)
    r = write(cwd, home, "tracked.txt")
    out["tracked-clean"] = {"content": (cwd / "tracked.txt").read_text(), "stashes": len(stashes(cwd))}
    home, cwd = fresh()
    git(cwd, init + " && echo local-edit > tracked.txt")
    r = write(cwd, home, "tracked.txt", [base.tool_reply([base.tu("t2", "write_file", {"path": "tracked.txt", "content": "TWO"})])])
    st = stashes(cwd)
    first = subprocess.run("git stash show -p stash@{1}", shell=True, cwd=cwd, capture_output=True, text=True).stdout if len(st) > 1 else ""
    out["tracked-modified-twice"] = {"content": (cwd / "tracked.txt").read_text(), "stashes": len(st), "oldest_stash_holds_local_edit": "+local-edit" in first,
                                      "local_edit_in_requests": any("local-edit" in q["body"] for q in r["reqs"])}
    home, cwd = fresh()
    r = drive(b, ["--undo"], [base.text_reply("ok")], home, cwd)
    bb = body(r)
    out["undo-flag"] = {"taken_as_prompt": bb["messages"][0]["content"] if isinstance(bb, dict) else None}
    return out


@scenario("s11-interactive", "f30-f36-f49", modes=("Debug", "ReleaseFast"))
def s11(b):
    home, cwd = fresh()
    r = drive(b, [], [base.text_reply("one"), base.text_reply("two")], home, cwd, stdin="first prompt\nsecond prompt\n/quit\n")
    idx, _ = read_index(home)
    files = record_files(home)
    newest_roles = None
    if files:
        m = load((sdir(home) / files[-1]).read_bytes())
        newest_roles = [x["role"] for x in json.loads(m)] if m else None
    piped = {"rc": r["rc"], "requests": len(r["reqs"]), "records": len(files), "index_entries": len(idx) if idx else 0,
             "distinct_ids": len({e["id"] for e in idx}) if idx else 0, "newest_record_roles": newest_roles,
             "fsync_panic": "panic: reached unreachable code" in r["err"] and "fsync" in r["err"]}
    # the same two prompts through a pseudo-terminal
    home, cwd = fresh()
    fp = base.FakeProvider([base.text_reply("one"), base.text_reply("two")])
    env = {"PATH": os.environ["PATH"], "HOME": str(home), "ANTHROPIC_API_KEY": "fake-key-not-real", "ANTHROPIC_BASE_URL": f"http://127.0.0.1:{fp.port}", "TERM": "xterm"}
    pid, fd = pty.fork()
    if pid == 0:
        os.chdir(cwd)
        os.umask(0o022)
        os.execve(str(b), [str(b)], env)
    out, sent, t0 = b"", 0, time.time()
    script = [b"first prompt\n", b"second prompt\n", b"/quit\n"]
    while time.time() - t0 < 30:
        rl, _, _ = select.select([fd], [], [], 0.3)
        if rl:
            try:
                chunk = os.read(fd, 4096)
            except OSError:
                break
            if not chunk:
                break
            out += chunk
        if sent < len(script) and out.count(b"tri> ") > sent:
            os.write(fd, script[sent])
            sent += 1
    try:
        _, status = os.waitpid(pid, 0)
    except ChildProcessError:
        status = None
    fp.close()
    idx, _ = read_index(home)
    tty = {"exit_status": status, "requests": len(fp.requests), "index_entries": len(idx) if idx else 0,
           "distinct_ids": len({e["id"] for e in idx}) if idx else 0, "records": len(record_files(home))}
    return {"piped": piped, "terminal": tty}


@scenario("s12-sigterm-side-effect", "f44", modes=("ReleaseFast",))
def s12(b):
    home, cwd = fresh()
    (cwd / ".trinity/api").mkdir(parents=True)
    (cwd / ".trinity/api/settings.json").write_text(ALLOW)
    r = drive(b, ["append a line"], [base.tool_reply([base.tu("t1", "bash", {"command": "echo did >> log.txt"})]),
                                      (200, base.text_reply("late")[1], {"_delay": 8})], home, cwd, sigterm_after=3, timeout=60)
    log = (cwd / "log.txt").read_text() if (cwd / "log.txt").exists() else ""
    records = len(record_files(home))
    r2 = drive(b, ["--continue", "what did you do?"], [base.text_reply("?")], home, cwd)
    return {"rc": r["rc"], "side_effect": log, "records": records, "continue_mark": first_mark(r2),
            "action_in_any_request": any("echo did" in q["body"] for q in r2["reqs"])}


def run_e2e(trinity, rev, zig, sysroot, only=None) -> dict:
    work = pathlib.Path(tempfile.mkdtemp(prefix="s08-e2e-"))
    base.fetch_pinned(trinity, rev, work)
    bins = {m: base.build_binary(zig, sysroot, work, m) for m in ("Debug", "ReleaseFast")}
    out = {}
    for sc in SCENARIOS:
        if only and sc["id"] not in only:
            continue
        out[sc["id"]] = {"finding": sc["finding"], "modes": {m: sc["fn"](bins[m]) for m in sc["modes"]}}
    return out


# ===========================================================================================
# ZIG: session_store.zig and memory.zig at the pin, compiled with generated fixtures appended
# ===========================================================================================
PREVIEW_PROMPT = ("a" + "ж" * 45).encode()


def model_answers() -> dict:
    """What the model says each Zig fixture must print."""
    out = {}
    for vid, msg in ROUNDTRIP:
        got = load(record_body(msg, b"00000000", 0))
        out[vid] = "equal" if got == msg else {"len": len(got) if got is not None else None}
    rec = record_body(SWEEP_MESSAGES, SWEEP_ID, SWEEP_TS)
    out["sweep"] = runs([load_class(rec[:k], SWEEP_MESSAGES) for k in range(len(rec) + 1)])
    out["preview"] = base.L(index_entry(b"00000000", 0, PREVIEW_PROMPT).split(b'"preview":"', 1)[1][:-2])
    for vid, content in MEMORY_CASES:
        got = memory_load(content)
        out[vid] = None if got is None else {"len": len(got), "lines": got.count(b"\n")}
    return out


def session_fixture() -> str:
    z = base.zlit
    t = ['test "FIXTURE session_store" {',
         '    const a = std.heap.page_allocator;',
         '    var tmp = std.testing.tmpDir(.{});',
         '    defer tmp.cleanup();',
         '    const dir = try tmp.dir.realpathAlloc(a, ".");',
         '    var store = SessionStore{ .allocator = a, .base_dir = dir, .base_dir_owned = false };']
    for vid, msg in ROUNDTRIP:
        t += ['    {',
              '        const msg = %s;' % z(msg),
              '        store.save(msg, "p");',
              '        const got = store.loadLatest();',
              '        std.debug.print("\\nVEC {{\\"id\\":\\"%s\\",\\"r\\":", .{});' % vid,
              '        if (got) |g| {',
              '            if (std.mem.eql(u8, g, msg)) std.debug.print("\\"equal\\"", .{}) else std.debug.print("{{\\"len\\":{d}}}", .{g.len});',
              '        } else std.debug.print("{{\\"len\\":null}}", .{});',
              '        std.debug.print("}}\\n", .{});',
              '    }']
    rec = record_body(SWEEP_MESSAGES, SWEEP_ID, SWEEP_TS)
    t += ['    {',
          '        const rec = %s;' % z(rec),
          '        const original = %s;' % z(SWEEP_MESSAGES),
          '        var k: usize = 0;',
          '        std.debug.print("\\nVEC {{\\"id\\":\\"sweep\\",\\"r\\":[", .{});',
          '        while (k <= rec.len) : (k += 1) {',
          '            try tmp.dir.writeFile(.{ .sub_path = "%s.json", .data = rec[0..k] });' % SWEEP_ID.decode(),
          '            const got = store.load("%s");' % SWEEP_ID.decode(),
          '            const c: []const u8 = if (got) |g| (if (std.mem.eql(u8, g, original)) "equal" else if (g.len > 1 and g[g.len - 1] == \']\') "bracket" else "partial") else "none";',
          '            if (k > 0) std.debug.print(",", .{});',
          '            std.debug.print("\\"{s}\\"", .{c});',
          '        }',
          '        std.debug.print("]}}\\n", .{});',
          '    }',
          '    {',
          '        var tmp2 = std.testing.tmpDir(.{});',
          '        defer tmp2.cleanup();',
          '        const dir2 = try tmp2.dir.realpathAlloc(a, ".");',
          '        var store2 = SessionStore{ .allocator = a, .base_dir = dir2, .base_dir_owned = false };',
          '        store2.save("[]", %s);' % z(PREVIEW_PROMPT),
          '        const idx = try tmp2.dir.readFileAlloc(a, "index.json", 1 << 20);',
          '        const needle = "\\"preview\\":\\"";',
          '        const at = std.mem.indexOf(u8, idx, needle).? + needle.len;',
          '        std.debug.print("\\nVEC {{\\"id\\":\\"preview\\",\\"r\\":", .{});',
          '        fxJ(idx[at .. idx.len - 3]);',
          '        std.debug.print("}}\\n", .{});',
          '    }',
          '}']
    return "\n".join(t) + "\n"


def memory_fixture() -> str:
    z = base.zlit
    t = ['test "FIXTURE memory" {',
         '    const a = std.heap.page_allocator;',
         '    var tmp = std.testing.tmpDir(.{});',
         '    defer tmp.cleanup();',
         '    const dir = try tmp.dir.realpathAlloc(a, ".");',
         '    var mem = Memory{ .allocator = a };',
         '    @memcpy(mem.base_dir[0..dir.len], dir);',
         '    mem.base_dir_len = dir.len;']
    for vid, content in MEMORY_CASES:
        t += ['    {',
              '        try tmp.dir.writeFile(.{ .sub_path = "MEMORY.md", .data = %s });' % z(content),
              '        const got = mem.load();',
              '        std.debug.print("\\nVEC {{\\"id\\":\\"%s\\",\\"r\\":", .{});' % vid,
              '        if (got) |g| std.debug.print("{{\\"len\\":{d},\\"lines\\":{d}}}", .{ g.len, std.mem.count(u8, g, "\\n") }) else std.debug.print("null", .{});',
              '        std.debug.print("}}\\n", .{});',
              '    }']
    t += ['}']
    return "\n".join(t) + "\n"


FIXTURES = {"session_store": session_fixture, "memory": memory_fixture}


def run_fixture(zig, sysroot, src: pathlib.Path, module: str, mode: str) -> dict:
    work = src.parent / f"fx-{module}-{mode}"
    if work.exists():
        shutil.rmtree(work)
    work.mkdir(parents=True)
    for f in src.glob("*.zig"):
        shutil.copy(f, work / f.name)
    pinned = (src / f"{module}.zig").read_text(encoding="utf-8")
    (work / f"{module}.zig").write_text(pinned + base.ZIG_HELPERS + FIXTURES[module](), encoding="utf-8")
    cmd = [zig, "test"] + (["--sysroot", sysroot] if sysroot else []) + ["-O", mode, f"{module}.zig"]
    rc, out, err = base.run(cmd, cwd=work, timeout=1800)
    text = out + err
    got = {}
    for m in base.VEC_LINE.finditer(text):
        try:
            d = json.loads(m.group(1))
            got[d["id"]] = d["r"]
        except (ValueError, KeyError):
            pass
    if "sweep" in got:
        got["sweep"] = runs(got["sweep"])
    m = re.search(r"All (\d+) tests passed", text)
    return {"rc": rc, "tests": base.zig_tests_line(text), "upstream_tests_passed": int(m.group(1)) - 1 if m else None,
            "answers": got, "tail": "" if rc == 0 else text[-600:]}


def build_zig_record(trinity, rev, zig, sysroot) -> dict:
    work = pathlib.Path(tempfile.mkdtemp(prefix="s08-zig-"))
    base.fetch_pinned(trinity, rev, work)
    model = model_answers()
    modes = {}
    for mode in ZIG_MODES:
        modes[mode] = {}
        for module in FIXTURES:
            r = run_fixture(zig, sysroot, work / "tri-api", module, mode)
            ids = [k for k in model if (k.startswith("mem-") if module == "memory" else not k.startswith("mem-"))]
            r["disagree"] = [{"id": i, "model": model[i], "zig": r["answers"].get(i)} for i in ids if i in r["answers"] and r["answers"][i] != model[i]]
            r["missing"] = [i for i in ids if i not in r["answers"]]
            r["agree"] = len(ids) - len(r["disagree"]) - len(r["missing"])
            r["vectors"] = len(ids)
            if r["rc"] == 0:
                r.pop("tail")
            modes[mode][module] = r
    return {"pin": rev, "zig": base.zig_version(zig), "host": base.host(), "at": base.now(), "model": model, "modes": modes,
            "description": "session_store.zig and memory.zig at the pin, each compiled with a generated test block appended to a copy: eight messages arrays saved and loaded back, every prefix of one saved record loaded back with load() (the sweep, compressed into runs of one class), the index preview of a 91-byte prompt, and five memory files; each Zig answer against the Python model's, in four optimize modes."}


def build_e2e_record(trinity, rev, zig, sysroot) -> dict:
    return {"pin": rev, "zig": base.zig_version(zig), "host": base.host(), "at": base.now(), "scenarios": run_e2e(trinity, rev, zig, sysroot),
            "provider": "tools/trinity_tri_api.py FakeProvider on 127.0.0.1; key fake-key-not-real; HOME and the working directory are fresh temporary directories; umask 022",
            "description": "The real tri-api built from the pinned main.zig in Debug and ReleaseFast, driven through twelve scenarios about what it keeps between runs. Each observation is what the process did: exit codes, the requests the fake provider saw, the files left in HOME and in the working directory. Keys starting with ~ are recorded and not asserted."}


# ===========================================================================================
# CLAIMS: the e2e observations the spec rests on, written out by hand
# ===========================================================================================
DEFAULT_MODEL = "claude-sonnet-4-20250514"
NO_SESSION, RESUMING = "No session found to resume", "Resuming session"
SWEEP_ROLES = ["user", "assistant", "user", "user"]


def expected_cut(cls):
    return {"none": (NO_SESSION, ["user"]), "partial": (RESUMING, ["user"]), "bracket": (RESUMING, "INVALID-JSON"),
            "equal": (RESUMING, SWEEP_ROLES)}[cls]


def claims(E) -> list:
    o = lambda sid, mode="Debug": E["scenarios"][sid]["modes"][mode]
    C = []

    def add(cid, fnd, text, fn):
        C.append((cid, fnd, text, fn))
    add("c01", "f35", "a record is {id, ts, messages}; the id is the hex of the second; files 0644, directories 0755; the 91-byte prompt leaves an 80-byte preview and an index that is not UTF-8",
        lambda: (lambda x: x["rc"] == 0 and x["records"] == 1 and x["record_keys"] == ["id", "ts", "messages"] and x["id_is_hex_of_ts"] and x["saved_line"]
                 and x["prompt_bytes"] == 91 and x["preview_bytes"] == 80 and x["index_valid_utf8"] is False and x["record_mode"] == 0o644
                 and x["index_mode"] == 0o644 and x["dir_mode"] == 0o755 and x["messages_roundtrip"] is True)(o("s01-record-shape")))
    add("c02", "f36-f37", "--continue and --resume <id> prepend the old turns, tool pair included, under the default model; an unknown id starts fresh with exit 0; four runs, no record deleted",
        lambda: (lambda x: x["run1_model"] == "claude-test-model" and x["run2_model"] == DEFAULT_MODEL and x["run2_mark"] == RESUMING
                 and x["run2_roles"] == ["user", "assistant", "user", "assistant", "user"] and x["run2_blocks"] == ["tool_use", "tool_result", "text"]
                 and x["run3_mark"] == RESUMING and x["run3_roles"] == ["user", "assistant", "user", "assistant", "user"]
                 and x["run4_mark"] == NO_SESSION and x["run4_roles"] == ["user"] and x["run4_rc"] == 0
                 and x["index_entries_after_4_runs"] == 4 and x["records_after_4_runs"] == x["distinct_ids"])(o("s02-resume")))
    add("c03", "f30", "eight runs started in one second, ten times: fewer records than runs every time, one record per id, and the index lost entries in at least one trial",
        lambda: (lambda x: x["all_rc0"] and x["all_index_valid"] and x["records_equal_ids"] and x["fewer_records_than_runs"]
                 and x["single_id_trials"] >= 1 and x["entries_at_most_runs"] and x["trials_losing_entries"] >= 1)(o("s03-same-second")))
    add("c04", "f31", "a record write cut at 1000 bytes: the partial record stays, no index, no Session saved line, no message, exit 0; --continue finds nothing, --resume announces a resume and drops the history",
        lambda: (lambda x: x["rc"] == 0 and x["records"] == 1 and x["record_bytes"] == x["limit"] == 1000 and x["index_exists"] is False and x["saved_line"] is False
                 and x["tri_api_lines_about_saving"] == [] and x["continue_mark"] == NO_SESSION and x["continue_roles"] == ["user"]
                 and x["resume_mark"] == RESUMING and x["resume_roles"] == ["user"])(o("s04-interrupted-write")))

    def c05():
        x = o("s05-damaged-records")
        ok = x["not-an-array"] == {"mark": RESUMING, "roles": ["user"], "rc": 0}
        ok = ok and x["garbage-ending-in-bracket"] == {"mark": RESUMING, "roles": "INVALID-JSON", "rc": 0}
        ok = ok and x["last-entry-missing"] == {"mark": NO_SESSION, "roles": ["user"], "rc": 0}
        ok = ok and x["version-99"] == {"mark": RESUMING, "roles": ["user", "assistant", "user"], "rc": 0}
        ok = ok and x["tmss-header"] == {"mark": NO_SESSION, "roles": ["user"], "rc": 0}
        ok = ok and x["newer-record-not-indexed"] == {"mark": RESUMING, "resumed_first_content": "first"}
        ok = ok and len(x["cuts"]) >= 8 and {c["class"] for c in x["cuts"]} == {"none", "partial", "bracket", "equal"}
        return ok and all((c["mark"], c["roles"]) == expected_cut(c["class"]) for c in x["cuts"])
    add("c05", "f32-f33-f34", "damaged records: a non-array and a partial record announce a resume and drop it, a record ending in ] becomes an invalid request, version 99 resumes, a TMSS header is no record, a missing last entry has no fallback, a newer record outside the index is ignored", c05)
    add("c06", "f39", "200 of 250 memory lines reach the system prompt, a memory past 256 KiB reaches it not at all, and tri-api writes no memory",
        lambda: (lambda x: x["mem-250-lines"]["memory_lines_in_system"] == 200 and x["mem-250-lines"]["memory_heading"]
                 and x["mem-over-cap"]["memory_heading"] is False and x["mem-over-cap"]["memory_lines_in_system"] == 0
                 and x["mem-small"]["memory_lines_in_system"] == 1 and all(v["file_bytes_after"] == v["bytes"] for k, v in x.items() if k != "no-file")
                 and x["no-file"] == {"created": False, "system": None})(o("s06-memory")))
    add("c07", "f37", "the system prompt is the three CLAUDE.md files and the memory, read again on --continue, and no record keeps any of it or the model",
        lambda: (lambda x: x["run1_system"] == "GLOBAL-RULES\n---\nPROJECT-RULES\n---\nLOCAL-RULES\n---\n# Memory\nMEMORY-A"
                 and x["run2_system"] == "GLOBAL-RULES\n---\nPROJECT-RULES-CHANGED\n---\nLOCAL-RULES\n---\n# Memory\nMEMORY-B"
                 and x["run1_model"] == "claude-test-model" and x["run2_model"] == DEFAULT_MODEL and not any(x["records_hold"].values()))(o("s07-system-provenance")))
    add("c08", "f38", "a token read by read_file is in the 0644 record and was sent to the provider; the audit log holds tool, path and verdict",
        lambda: (lambda x: x["token_in_record"] and x["token_sent"] and x["record_mode"] == 0o644 and x["token_in_audit"] is False
                 and {k: v for k, v in x["audit_entry"].items() if k != "ts"} == {"arg": "cfg.txt", "result": "ok", "tool": "read_file"} and x["audit_mode"] == 0o644
                 and x["in_requests"] == {"record-content": True, "preview-content": True, "memory": True, "claude-md": True, "audit-verdict-line": False})(o("s08-privacy")))

    def c09():
        x = o("s09-compaction")
        a, bk, sm = x["truncate-clean"], x["truncate-trailing-backslash"], x["summary-claims-done"]
        ok = len(a["requests"]) == 1 and a["requests"][0]["valid_json"] and a["requests"][0]["denial_kept"] and a["requests"][0]["denial_is_error_kept"]
        ok = ok and a["requests"][0]["original_prompt_kept"] and a["requests"][0]["orphan_results"] == [] and a["requests"][0]["truncation_markers"] > 0
        ok = ok and a["saved_record"]["valid_json"] and a["next_continue_request_valid"]
        ok = ok and len(bk["requests"]) == 1 and bk["requests"][0] == {"valid_json": False} and bk["saved_record"]["valid_json"] is False and bk["next_continue_request_valid"] is False
        r0, r1 = sm["requests"][0], sm["requests"][1]
        ok = ok and len(sm["requests"]) == 2 and r0["has_tools"] is False and r1["first_head"].startswith("[Previous context summary]")
        ok = ok and r1["denial_kept"] is False and r1["original_prompt_kept"] is False and r1["claims_completed"] is True and r1["orphan_results"] == []
        return ok and sm["saved_record"] == {"valid_json": True, "original_prompt": False, "denial": False, "claims_completed": True}
    add("c09", "f41-f42-f43", "truncation keeps the prompt, the denial and its is_error and every pair; a 301-byte output ending in a backslash leaves an invalid request, record and next request; a summary claiming the denied write is sent and saved in place of the denial", c09)
    add("c10", "f40", "an untracked file is overwritten after stash failed, nothing is kept outside a repository or for a clean file, two writes leave two stash entries, and --undo is a prompt",
        lambda: (lambda x: x["untracked"] == {"content": "NEW", "stashes": 0, "stash_failed_line": True} and x["no-repo"] == {"content": "NEW", "checkpoint_lines": []}
                 and x["tracked-clean"] == {"content": "NEW", "stashes": 0} and x["tracked-modified-twice"]["content"] == "TWO"
                 and x["tracked-modified-twice"]["stashes"] == 2 and x["tracked-modified-twice"]["oldest_stash_holds_local_edit"]
                 and x["tracked-modified-twice"]["local_edit_in_requests"] is False and x["undo-flag"] == {"taken_as_prompt": "--undo"})(o("s10-checkpoint", "ReleaseFast")))

    def c11():
        d, f = o("s11-interactive", "Debug"), o("s11-interactive", "ReleaseFast")
        ok = d["piped"]["rc"] == -6 and d["piped"]["fsync_panic"] and d["piped"]["records"] == 0 and d["piped"]["requests"] == 0
        ok = ok and f["piped"]["rc"] == 0 and f["piped"]["fsync_panic"] is False and f["piped"]["requests"] == 2 and f["piped"]["index_entries"] == 2
        ok = ok and f["piped"]["records"] == f["piped"]["distinct_ids"] and f["piped"]["newest_record_roles"] == ["user", "assistant", "user", "assistant"]
        for t in (d["terminal"], f["terminal"]):
            ok = ok and t["exit_status"] == 0 and t["requests"] == 2 and t["index_entries"] == 2 and t["records"] == t["distinct_ids"]
        return ok
    add("c11", "f30-f36-f49", "interactive mode saves after each prompt; piped stdout panics in Debug before the first prompt and works in ReleaseFast; through a terminal both work", c11)
    add("c12", "f44", "SIGTERM after a bash append: the line is in log.txt, no record exists, and --continue finds nothing that mentions it",
        lambda: (lambda x: x["rc"] == -15 and x["side_effect"] == "did\n" and x["records"] == 0 and x["continue_mark"] == NO_SESSION
                 and x["action_in_any_request"] is False)(o("s12-sigterm-side-effect", "ReleaseFast")))
    return C


def check_claims(E) -> list:
    f = []
    need, have = {sc["id"] for sc in SCENARIOS}, set(E.get("scenarios", {}))
    if need != have:
        return [f"e2e: scenarios {sorted(need ^ have)} are missing from or extra in the record"]
    for sc in SCENARIOS:
        if set(E["scenarios"][sc["id"]]["modes"]) != set(sc["modes"]):
            f.append(f"e2e: {sc['id']} holds modes {sorted(E['scenarios'][sc['id']]['modes'])}, the harness runs {list(sc['modes'])}")
    if f:
        return f
    E = base.strip_unasserted(E)
    for cid, fnd, text, fn in claims(E):
        try:
            ok = bool(fn())
        except (KeyError, TypeError, IndexError, AttributeError) as e:
            ok, text = False, f"{text} (the record has no such observation: {type(e).__name__} {e})"
        if not ok:
            f.append(f"e2e {cid} [{fnd}]: {text}")
    return f


# ===========================================================================================
# REPLAY: the decisions of both session specs through the C backend, against the records
# ===========================================================================================
def tmem_consts() -> dict:
    s = TMEM_SPEC.read_text(encoding="utf-8")
    out = {}
    for m in re.finditer(r"^\s*pub const (\w+) : (\w+) = ([^;]+);", s, re.M):
        v = m.group(3).strip()
        out[m.group(1)] = int(v, 16) if v.startswith("0x") else int(v)
    return out


def resume_class(mark, roles_):
    if mark == NO_SESSION:
        return 0
    if roles_ == ["user"]:
        return 1
    return 2


def s04_partial_record(limit) -> bytes:
    msgs = json.dumps([{"role": "user", "content": "x" * 2500}, {"role": "assistant", "content": [{"type": "text", "text": "ok"}]}],
                      separators=(",", ":")).encode()
    return record_body(msgs, b"00000000", 1790000000)[:limit]


def s05_contents() -> dict:
    tmp = pathlib.Path(tempfile.mkdtemp(prefix="s08-r-"))
    out = {"not-an-array": write_record(tmp, "aaaaaaaa", b'"hello"'), "garbage-ending-in-bracket": write_record(tmp, "aaaaaaaa", b"garbage]"),
           "version-99": write_record(tmp, "aaaaaaaa", TWO_TURNS, extra=b',"version":99,"magic":"TMSS"'), "tmss-header": b"TMSS" + bytes(172),
           "last-entry-missing": None}
    shutil.rmtree(tmp, ignore_errors=True)
    return out


def session_cases(Z, E) -> list:
    E = base.strip_unasserted(E)
    S = base.load_spec(SPEC)
    g = lambda sid, mode="Debug": E["scenarios"][sid]["modes"][mode]
    cb = base.cb
    cases = []
    x = g("s01-record-shape")
    cases.append(("ses-id", [(f"session_id({x['ts']}u) == {int(x['id'], 16)}u", "id")]))
    cases.append(("ses-preview", [(f"preview_bytes({x['prompt_bytes']}u) == {x['preview_bytes']}u", "binary preview"),
                                  (f"preview_bytes({len(PREVIEW_PROMPT)}u) == {len(base.B(Z['modes']['Debug']['session_store']['answers']['preview']))}u", "zig preview")]))
    x = g("s03-same-second")
    one = x["single_id_trials"] >= 1 and x["records_equal_ids"]           # a trial whose eight saves all fell in one second
    cases.append(("ses-same-second", [(f"records_after_same_second({x['runs']}u) == {1 if one else x['runs']}u", "one record for eight saves in one second")]))
    x = g("s04-interrupted-write")
    vf, eb = resume_inputs(s04_partial_record(x["limit"]))
    cases.append(("ses-interrupted", [(f"write_is_atomic() == {cb(not (x['record_bytes'] == x['limit']))}", "atomic"),
                                      (f"index_written_after_failed_record() == {cb(x['index_exists'])}", "index"),
                                      (f"resume_outcome({cb(vf)}, {cb(eb)}) == {resume_class(x['resume_mark'], x['resume_roles'])}u", "resume of the partial record")]))
    x = g("s05-damaged-records")
    for name, content in s05_contents().items():
        vf, eb = resume_inputs(content)
        cases.append((f"ses-{name}", [(f"resume_outcome({cb(vf)}, {cb(eb)}) == {resume_class(x[name]['mark'], x[name]['roles'])}u", "outcome"),
                                      (f"resume_announced(resume_outcome({cb(vf)}, {cb(eb)})) == {cb(x[name]['mark'] == RESUMING)}", "announced")]))
    cases.append(("ses-no-fallback", [(f"continue_tries_older(false) == {cb(x['last-entry-missing']['roles'] != ['user'])}", "fallback")]))
    rec = record_body(SWEEP_MESSAGES, SWEEP_ID, SWEEP_TS)
    for c in x["cuts"]:
        vf, eb = resume_inputs(rec[:c["cut"]])
        cases.append((f"ses-cut-{c['cut']}", [(f"resume_outcome({cb(vf)}, {cb(eb)}) == {resume_class(c['mark'], c['roles'])}u", "outcome")]))
    for k_cls in Z["modes"]["Debug"]["session_store"]["answers"]["sweep"]:
        cls, lo, hi = k_cls
        for k in (lo, hi):
            vf, eb = resume_inputs(rec[:k])
            want = {"none": 0, "partial": 1, "bracket": 2, "equal": 2}[cls]
            cases.append((f"ses-zig-sweep-{k}", [(f"resume_outcome({cb(vf)}, {cb(eb)}) == {want}u", "zig sweep class")]))
    x = g("s02-resume")
    cases.append(("ses-retention", [(f"records_deleted() == {x['distinct_ids'] - x['records_after_4_runs']}u", "deleted")]))
    x = g("s06-memory")
    for name, content in MEMORY_CASES:
        nl, nb = content.count(b"\n"), len(content)
        if name in x:
            seen = x[name]["memory_lines_in_system"]
            cases.append((f"ses-{name}", [(f"memory_lines_loaded({nl}u, {nb}u) == {seen}u", "binary memory lines")]))
        zr = Z["modes"]["Debug"]["memory"]["answers"][name]
        cases.append((f"ses-zig-{name}", [(f"memory_lines_loaded({nl}u, {nb}u) == {0 if zr is None else zr['lines']}u", "zig memory lines")]))
    cases.append(("ses-memory-written", [(f"memory_written() == {cb(x['no-file']['created'])}", "written")]))
    x = g("s10-checkpoint", "ReleaseFast")
    obs = lambda v: 2 if v.get("stash_failed_line") else (1 if v.get("stashes", 0) > 0 else 0)
    for name, args in (("untracked", "true, true, false, false"), ("no-repo", "true, false, false, false"),
                       ("tracked-clean", "true, true, true, false"), ("tracked-modified-twice", "true, true, true, true")):
        cases.append((f"ses-ck-{name}", [(f"checkpoint_outcome({args}) == {obs(x[name])}u", "checkpoint")]))
    cases.append(("ses-restore", [(f"restore_reachable() == {cb(x['undo-flag']['taken_as_prompt'] != '--undo')}", "restore")]))
    cases.append(("ses-stash-local", [(f"artifact_class(5u) == {0 if not x['tracked-modified-twice']['local_edit_in_requests'] else 1}u", "stash entry")]))
    x = g("s09-compaction")
    clean_ok = x["truncate-clean"]["requests"][0]["valid_json"] and x["truncate-clean"]["requests"][0]["truncation_markers"] > 0
    cases.append(("ses-trunc-clean", [(f"truncation(300u, false) == {1 if clean_ok else 2}u", "clean")]))
    cases.append(("ses-trunc-backslash", [(f"truncation(302u, true) == {2 if not x['truncate-trailing-backslash']['requests'][0]['valid_json'] else 1}u", "backslash")]))
    cases.append(("ses-trunc-denial", [(f"truncation(41u, false) == {0 if x['truncate-clean']['requests'][0]['denial_kept'] else 1}u", "denial")]))
    sm = x["summary-claims-done"]["requests"][1]
    cases.append(("ses-summary", [(f"summary_keeps_denial() == {cb(sm['denial_kept'])}", "denial"),
                                  (f"summary_checked() == {cb(not sm['claims_completed'])}", "checked")]))
    x = g("s07-system-provenance")
    cases.append(("ses-provenance", [(f"provenance_recorded() == {sum(1 for v in x['records_hold'].values() if v)}u", "recorded")]))
    x = g("s08-privacy")["in_requests"]
    for i, key in enumerate(("record-content", "preview-content", "memory", "claude-md", "audit-verdict-line")):
        cases.append((f"ses-artifact-{key}", [(f"artifact_class({i}u) == {1 if x[key] else 0}u", "class")]))
    size = g("s01-record-shape")["record_bytes"]
    cases.append(("ses-tmss-record", [(f"tmss_status_of_record({size}u) == {S['TMSS_ERR_MAGIC']}", "record"),
                                      (f"tmss_status_of_record(30u) == {S['TMSS_ERR_TRUNCATED']}", "short record")]))
    for mode in ("Debug", "ReleaseFast"):
        p = g("s11-interactive", mode)["piped"]
        cases.append((f"ses-pipe-{mode}", [(f"interactive_pipe_panics({cb(mode == 'Debug')}) == {cb(p['fsync_panic'])}", "pipe")]))
    return cases


TMEM_SHAPES = {
    ("crc_ok", "kind", "length", "magic_ok", "version"): lambda i: f"tms_session_header_status({i['length']}u, {i['magic_ok']}u, {i['version']}u, {i['kind']}u, {i['crc_ok']}u)",
    ("compiler_rev_match", "spec_hash_match"): lambda i: f"tms_session_ancestry_status({i['spec_hash_match']}u, {i['compiler_rev_match']}u)",
    ("action_count", "parent_match", "prev_action_count", "prev_sequence", "sequence"): lambda i: f"tms_session_chain_status({i['prev_sequence']}u, {i['sequence']}u, {i['prev_action_count']}u, {i['action_count']}u, {i['parent_match']}u)",
    ("seq_a", "seq_b", "valid_a", "valid_b"): lambda i: f"tms_session_recover_pick({i['seq_a']}u, {i['valid_a']}u, {i['seq_b']}u, {i['valid_b']}u)",
    ("kind", "newest", "retain", "sequence"): lambda i: f"tms_session_retain_keep({i['kind']}u, {i['sequence']}u, {i['newest']}u, {i['retain']}u)",
    ("field",): lambda i: f"tms_session_field_class({i['field']}u)",
    ("state", "step_ok"): lambda i: f"tms_session_write_next({i['state']}u, {i['step_ok']}u)",
}


def tmem_cases(conf, E) -> list:
    cases = []
    for v in conf["vectors"]:
        call = TMEM_SHAPES[tuple(sorted(v["inputs"]))](v["inputs"])
        cases.append((f"tmem-{v['id']}", [(f"{call} == {v['expected']}", "expected")]))
    E = base.strip_unasserted(E)
    size = E["scenarios"]["s01-record-shape"]["modes"]["Debug"]["record_bytes"]
    S = base.load_spec(SPEC)
    cases.append(("tmem-tri-api-record", [(f"tms_session_header_status({size}u, 0u, 1u, 0u, 1u) == {S['TMSS_ERR_MAGIC']}", "a tri-api record under the TMSS header rule"),
                                          (f"tms_session_header_status(30u, 0u, 1u, 0u, 1u) == {S['TMSS_ERR_TRUNCATED']}", "a short one")]))
    return cases


def replay_all(Z, E, conf, t27c, spec=None, tmem_spec=None) -> dict:
    res = {}
    with tempfile.TemporaryDirectory() as tmp:
        for key, path, cases in (("session", spec or SPEC, session_cases(Z, E)), ("tmem", tmem_spec or TMEM_SPEC, tmem_cases(conf, E))):
            r = base.replay_spec(pathlib.Path(path), cases, t27c, pathlib.Path(tmp) / key)
            r["spec_sha256"] = base.sha256(pathlib.Path(path).read_bytes())
            res[key] = r
    return {"at": base.now(), "host": base.host(), "t27c": (base.run([str(t27c), "--version"], timeout=60)[1] or "t27c").strip().split("\n")[0][:80], "specs": res}


# ===========================================================================================
# CHECK
# ===========================================================================================
SPEC_FROM_SOURCE = [("PREVIEW_MAX_BYTES", "preview_max"), ("SESSION_MAX_BYTES", "session_max_bytes"), ("ID_HEX_DIGITS", "id_hex_digits"),
                    ("MEMORY_MAX_LINES", "memory_max_lines"), ("MEMORY_MAX_BYTES", "memory_max_bytes"), ("TRUNCATE_KEEP_BYTES", "truncate_keep")]
SPEC_FROM_TMEM = [("TMSS_HEADER_BYTES", "TMS_SESSION_HEADER_BYTES"), ("TMSS_OK", "TMS_SESSION_OK"), ("TMSS_ERR_TRUNCATED", "TMS_SESSION_ERR_TRUNCATED"),
                  ("TMSS_ERR_MAGIC", "TMS_SESSION_ERR_MAGIC")]


def check_constants(sp: dict, Z: dict) -> list:
    f = []
    src = Z.get("source_constants", {})
    for c, k in SPEC_FROM_SOURCE:
        if sp.get(c) != src.get(k):
            f.append(f"session: {c} is {sp.get(c)!r}, the pinned source says {src.get(k)!r}")
    if src.get("memory_append_callers") != 0:
        f.append(f"session: MEMORY_RULE says nothing calls Memory.append; the pinned source has {src.get('memory_append_callers')} caller(s)")
    if src.get("sessions_subdir") and src["sessions_subdir"] not in sp.get("SESSIONS_DIR", ""):
        f.append("session: SESSIONS_DIR does not name the directory the source writes")
    if sp.get("PINNED_REVISION") != PIN:
        f.append("session: PINNED_REVISION is not the pin")
    tm = tmem_consts()
    for c, k in SPEC_FROM_TMEM:
        if sp.get(c) != tm.get(k):
            f.append(f"session: {c} is {sp.get(c)!r}, specs/memory/tmem/session.t27 says {k} = {tm.get(k)!r}")
    if sp.get("MAPPING_COUNT") != len(sp.get("MAPPING", [])):
        f.append("session: MAPPING_COUNT does not count MAPPING")
    return f


def check_tmem_conformance(conf: dict) -> list:
    f = []
    tm = tmem_consts()
    for k, v in conf.get("constants", {}).items():
        if tm.get(k) != v:
            f.append(f"tmem conformance: {k} is {v}, the spec says {tm.get(k)}")
    tests = set(re.findall(r"^\s*test (\w+)", TMEM_SPEC.read_text(encoding="utf-8"), re.M))
    for v in conf.get("vectors", []):
        if v["id"] not in tests:
            f.append(f"tmem conformance: vector {v['id']} names no test of the spec")
        if tuple(sorted(v["inputs"])) not in TMEM_SHAPES:
            f.append(f"tmem conformance: vector {v['id']} has inputs no function of the spec takes")
    return f


def check_zig(Z: dict) -> list:
    f = []
    if Z.get("pin") != PIN:
        f.append("zig: the record is not at the pin")
    model = model_answers()
    if Z.get("model") != json.loads(json.dumps(model)):
        f.append("zig: the recorded model answers drift from what the model computes now")
    if set(Z.get("modes", {})) != set(ZIG_MODES):
        return f + [f"zig: modes {sorted(Z.get('modes', {}))}, expected {list(ZIG_MODES)}"]
    for mode in ZIG_MODES:
        for module in FIXTURES:
            r = Z["modes"][mode].get(module)
            if not r:
                f.append(f"zig {mode}/{module}: no record")
                continue
            if r["rc"] != 0:
                f.append(f"zig {mode}/{module}: did not pass (rc {r['rc']}): {r.get('tail', '')[:160]}")
            if r["disagree"] or r["missing"]:
                f.append(f"zig {mode}/{module}: the model disagrees with the compiled Zig on {[d['id'] for d in r['disagree']]}, missing {r['missing']}")
            if r["agree"] != r["vectors"]:
                f.append(f"zig {mode}/{module}: {r['agree']} of {r['vectors']} answers agree")
            for vid in r.get("answers", {}):
                if r["answers"][vid] != model.get(vid):
                    f.append(f"zig {mode}/{module}: {vid} answered {str(r['answers'][vid])[:80]}, the model {str(model.get(vid))[:80]}")
    return f


def check_findings(sp: dict, Z: dict) -> list:
    f = []
    ids = {sc["id"] for sc in SCENARIOS}
    seen = set()
    items = sp.get("FINDINGS", [])
    if sp.get("FINDINGS_COUNT") != len(items):
        f.append(f"session: FINDINGS_COUNT {sp.get('FINDINGS_COUNT')} but {len(items)} findings")
    for it in items:
        m = re.match(r"(f\d\d) ", it)
        if not m:
            f.append(f"session: a finding without an id: {it[:40]}")
            continue
        seen.add(m.group(1))
        tag = re.search(r"\[([^\]]*)\]\s*$", it)
        if not tag:
            f.append(f"session {m.group(1)}: no evidence tag")
            continue
        for part in [p.strip() for p in tag.group(1).split(";")]:
            if re.fullmatch(r"s\d\d", part):
                if not any(i.startswith(part + "-") for i in ids):
                    f.append(f"session {m.group(1)}: scenario {part} is not in the harness")
            elif part == "zig prefix sweep":
                if "sweep" not in Z.get("model", {}):
                    f.append(f"session {m.group(1)}: the zig record has no prefix sweep")
            elif part == "zig memory caps":
                if not any(k.startswith("mem-") for k in Z.get("model", {})):
                    f.append(f"session {m.group(1)}: the zig record has no memory cases")
            elif part not in ("tmem", "code search", "source"):
                f.append(f"session {m.group(1)}: unrecognised evidence '{part}'")
    tags = set()
    for sc in SCENARIOS:
        tags |= set(re.findall(r"f\d\d", sc["finding"]))
    for t in sorted(tags - seen):
        f.append(f"a scenario measures {t} and the spec does not state it")
    return f


def check_replay(Z: dict, E: dict, conf: dict) -> list:
    r = Z.get("replay")
    if not isinstance(r, dict):
        return ["replay: none recorded; run `run`"]
    f = []
    want = {"session": (SPEC, len(session_cases(Z, E))), "tmem": (TMEM_SPEC, len(tmem_cases(conf, E)))}
    for k, (path, n) in want.items():
        x = r.get("specs", {}).get(k)
        if not x:
            f.append(f"replay {k}: none")
            continue
        if x.get("spec_sha256") != base.sha256(path.read_bytes()):
            f.append(f"replay {k}: {path.name} changed since the replay; run `run`")
        if not x.get("ok") or x.get("failed", 1) != 0 or x.get("passed") != n:
            f.append(f"replay {k}: {x.get('passed')} passed, {x.get('failed')} failed of {n} at stage {x.get('stage')}: {x.get('detail', '')[:160]} {x.get('failures', [])[:3]}")
    return f


def check_all() -> list:
    for p in (SPEC, TMEM_SPEC, TMEM_CONF):
        if not p.exists():
            return [f"{p}: missing"]
    Z, E = base.load_json(EVIDENCE_ZIG), base.load_json(EVIDENCE_E2E)
    if Z is None or E is None:
        return [f"{n}: no record; run `{n}`" for n, d in (("zig", Z), ("e2e", E)) if d is None]
    sp = base.load_spec(SPEC)
    conf = json.loads(TMEM_CONF.read_text())
    f = check_constants(sp, Z) + check_tmem_conformance(conf) + check_zig(Z)
    if E.get("pin") != PIN:
        f.append("e2e: the record is not at the pin")
    f += check_claims(E) + check_findings(sp, Z)
    if not f:
        f += check_replay(Z, E, conf)
    return f


# ===========================================================================================
# --self-check
# ===========================================================================================
def self_check(trinity, zig, sysroot) -> int:
    ok = True

    def expect(cond, what):
        nonlocal ok
        print(f"  {'ok ' if cond else 'BAD'} {what}")
        ok = ok and bool(cond)
    clone = lambda d: json.loads(json.dumps(d))
    Z, E = base.load_json(EVIDENCE_ZIG), base.load_json(EVIDENCE_E2E)
    expect(Z is not None and E is not None, "the two committed records exist")
    if Z is None or E is None:
        return 1
    sp = base.load_spec(SPEC)
    conf = json.loads(TMEM_CONF.read_text())
    f = check_all()
    expect(not f, f"check: records, specs and source agree ({len(f)} finding(s){': ' + f[0] if f else ''})")
    expect(len(SCENARIOS) == 12, f"the harness holds twelve scenarios ({len(SCENARIOS)})")
    p = clone(E)
    p["scenarios"]["s03-same-second"]["modes"]["Debug"]["fewer_records_than_runs"] = False
    expect(any(x.startswith("e2e c03") for x in check_claims(p)), "planted: a record per run breaks c03 (the collision would be gone)")
    p = clone(E)
    p["scenarios"]["s04-interrupted-write"]["modes"]["Debug"]["index_exists"] = True
    expect(any(x.startswith("e2e c04") for x in check_claims(p)), "planted: an index after a failed record write breaks c04")
    p = clone(E)
    p["scenarios"]["s09-compaction"]["modes"]["Debug"]["truncate-trailing-backslash"]["saved_record"]["valid_json"] = True
    expect(any(x.startswith("e2e c09") for x in check_claims(p)), "planted: a valid record after the backslash truncation breaks c09")
    p = clone(E)
    p["scenarios"]["s03-same-second"]["modes"]["Debug"]["~per_trial"] = []
    expect(not check_claims(p), "an unasserted observation (~per_trial) may change without a finding")
    p = clone(Z)
    p["modes"]["ReleaseSmall"]["session_store"]["disagree"] = [{"id": "sweep", "model": [], "zig": []}]
    expect(any("disagrees" in x for x in check_zig(p)), "planted: a model/Zig disagreement in one mode is reported")
    p = clone(Z)
    p["modes"]["Debug"]["memory"]["answers"]["mem-over-cap"] = {"len": 262145, "lines": 1}
    expect(any("mem-over-cap" in x for x in check_zig(p)), "planted: a memory cap that no longer drops the file is reported")
    p = clone(Z)
    p["source_constants"]["preview_max"] = 120
    expect(any("PREVIEW_MAX_BYTES" in x for x in check_constants(sp, p)), "planted: a preview cap the source no longer has")
    p = clone(Z)
    p["source_constants"]["memory_append_callers"] = 1
    expect(any("Memory.append" in x for x in check_constants(sp, p)), "planted: a caller of Memory.append contradicts MEMORY_RULE")
    p = clone(conf)
    p["constants"]["TMS_SESSION_MAGIC"] = 1397523396
    expect(any("TMS_SESSION_MAGIC" in x for x in check_tmem_conformance(p)), "planted: the magic the conformance file carried until today is reported")
    p = clone(sp)
    p["FINDINGS"][0] = p["FINDINGS"][0].replace("[s03;", "[s99;")
    expect(any("s99" in x for x in check_findings(p, Z)), "planted: a finding that cites a scenario that does not exist")
    p = clone(sp)
    p["FINDINGS"] = [x for x in p["FINDINGS"] if not x.startswith("f44")]
    p["FINDINGS_COUNT"] = len(p["FINDINGS"])
    expect(any("f44" in x for x in check_findings(p, Z)), "planted: a measured finding the spec stops stating")
    t27c = base.t27c_path()
    if t27c and shutil.which("cc"):
        good = replay_all(Z, E, conf, t27c)
        expect(all(x["ok"] for x in good["specs"].values()), "replay: " + ", ".join(f"{k} {x['passed']}/{x.get('fixtures')}" for k, x in good["specs"].items()))
        with tempfile.TemporaryDirectory() as tmp:
            tmp = pathlib.Path(tmp)

            def planted(which, old, new):
                path = SPEC if which == "session" else TMEM_SPEC
                text = path.read_text(encoding="utf-8")
                assert text.count(old) == 1, old
                pp = tmp / f"{which}.t27"
                pp.write_text(text.replace(old, new), encoding="utf-8")
                r = replay_all(Z, E, conf, t27c, spec=pp if which == "session" else None, tmem_spec=pp if which == "tmem" else None)
                return r["specs"][which]
            r = planted("session", "    if (ends_with_bracket) { return RESUME_HISTORY; }\n    return RESUME_ANNOUNCED_DROPPED;", "    return RESUME_HISTORY;")
            expect(any(x["id"].startswith("ses-cut-") for x in r["failures"]), "planted: a resume that keeps a partial history fails the cut cases")
            r = planted("session", "    if (ends_with_backslash) { return TRUNCATE_CORRUPT; }\n", "")
            expect(any(x["id"] == "ses-trunc-backslash" for x in r["failures"]), "planted: a truncation that never corrupts fails the backslash case")
            r = planted("session", "    if (tracked == false) { return CHECKPOINT_FAILED_THEN_WRITTEN; }\n", "")
            expect(any(x["id"] == "ses-ck-untracked" for x in r["failures"]), "planted: a checkpoint that covers untracked files fails s10")
            r = planted("session", "    if (bytes > MEMORY_MAX_BYTES) { return 0; }\n", "")
            expect(any(x["id"] in ("ses-mem-over-cap", "ses-zig-mem-over-cap") for x in r["failures"]), "planted: a memory without the 256 KiB cliff fails the over-cap cases")
            r = planted("session", "    if (saves == 0) { return 0; }\n    return 1;", "    return saves;")
            expect(any(x["id"] == "ses-same-second" for x in r["failures"]), "planted: one record per save fails the same-second case")
            r = planted("tmem", "        if (magic_ok == 0) {\n            return TMS_SESSION_ERR_MAGIC;\n        }\n", "")
            ids = sorted(x["id"] for x in r["failures"])
            expect("tmem-bad_magic" in ids and "tmem-tri-api-record" in ids, f"planted: a TMSS reader without the magic check fails bad_magic and the tri-api record ({ids[:4]})")
            p = clone(conf)
            p["vectors"][0]["expected"] = -5
            rr = replay_all(Z, E, p, t27c)["specs"]["tmem"]
            expect(any(x["id"] == "tmem-valid_header" for x in rr["failures"]), "planted: a wrong expected value in the conformance file fails its vector")
    else:
        print("  skip replay checks: t27c or cc missing")
    if trinity and zig:
        live = run_e2e(trinity, PIN, zig, sysroot, only=["s01-record-shape", "s04-interrupted-write", "s10-checkpoint"])
        drop = lambda d: {**d, "modes": {m: {k: v for k, v in o.items() if k not in VOLATILE} for m, o in d["modes"].items()}}
        same = all(drop(base.strip_unasserted(live[i])) == drop(base.strip_unasserted(E["scenarios"][i])) for i in live)
        expect(same, "rerun: three scenarios driven again against a fresh build give the recorded observations")
        expect(source_constants(trinity, PIN) == Z.get("source_constants"), "inventory: the constants read from the clone match the zig record")
    print("trinity_tri_api_session --self-check:", "ok" if ok else "FAILED")
    return 0 if ok else 1


# ===========================================================================================
# CLI
# ===========================================================================================
def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("command", nargs="?", choices=["zig", "e2e", "run", "check"])
    ap.add_argument("--trinity-root")
    ap.add_argument("--rev", default=PIN)
    ap.add_argument("--zig")
    ap.add_argument("--sysroot")
    ap.add_argument("--self-check", action="store_true")
    a = ap.parse_args()
    trinity = pathlib.Path(a.trinity_root).resolve() if a.trinity_root else None
    if a.self_check:
        return self_check(trinity, a.zig, a.sysroot)
    if a.command is None:
        ap.print_help()
        return 2
    wr = lambda path, doc: pathlib.Path(path).write_text(json.dumps(doc, indent=1, sort_keys=True, ensure_ascii=True) + "\n", encoding="utf-8")
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    if a.command in ("zig", "e2e"):
        if not (trinity and a.zig):
            print(f"{a.command}: --trinity-root and --zig are required", file=sys.stderr)
            return 2
        try:
            if a.command == "zig":
                doc = build_zig_record(trinity, a.rev, a.zig, a.sysroot)
                doc["source_constants"] = source_constants(trinity, a.rev)
                old = base.load_json(EVIDENCE_ZIG) or {}
                if "replay" in old:
                    doc["replay"] = old["replay"]
                wr(EVIDENCE_ZIG, doc)
                f = check_zig(doc)
                print(f"zig: {len(ZIG_MODES)} modes x {len(FIXTURES)} modules, {len(f)} finding(s)")
                for x in f:
                    print("  " + x)
                return 0 if not f else 1
            doc = build_e2e_record(trinity, a.rev, a.zig, a.sysroot)
            wr(EVIDENCE_E2E, doc)
            f = check_claims(doc)
            print(f"e2e: {len(doc['scenarios'])} scenarios, {len(f)} claim(s) failed")
            for x in f:
                print("  " + x)
            return 0 if not f else 1
        except (RuntimeError, OSError) as e:
            print(f"{a.command}: could not run: {e}", file=sys.stderr)
            return 2
    if a.command == "run":
        t27c = base.t27c_path()
        Z, E = base.load_json(EVIDENCE_ZIG), base.load_json(EVIDENCE_E2E)
        if not (t27c and shutil.which("cc") and Z and E):
            print("run: needs t27c, cc and both records", file=sys.stderr)
            return 2
        Z["replay"] = replay_all(Z, E, json.loads(TMEM_CONF.read_text()), t27c)
        wr(EVIDENCE_ZIG, Z)
        for k, x in Z["replay"]["specs"].items():
            print(f"run {k}: {x['passed']} passed, {x['failed']} failed ({x['stage']}) {x['detail'][:120]} {x['failures'][:3]}")
        return 0 if all(x["ok"] for x in Z["replay"]["specs"].values()) else 1
    if a.command == "check":
        f = check_all()
        for x in f:
            print("FINDING:", x)
        if not f:
            print("check: ok")
        return 1 if f else 0
    return 2


if __name__ == "__main__":
    sys.exit(main())
