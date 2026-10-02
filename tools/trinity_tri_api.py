#!/usr/bin/env python3
"""The tri-api agent loop, its tool protocol and its permission boundary: specs/api/tri_api_*.t27
held to the pinned source, to the pinned source's own compiled tests, and to the compiled binary
driven by a scripted provider.

WHY THIS EXISTS
---------------
S07 of gHashTag/trinity#988 (gHashTag/t27#3569). tri-api is an agent loop: it posts a conversation
to a Messages endpoint, scans the reply for text and tool_use blocks, checks each tool against a
permission table, runs it, appends the result and posts again. Until today no document stated what it
does at the edges: what a denied tool costs, what a malformed reply does, what the limits are, what
happens to a file the agent is about to overwrite. This tool measures those edges and holds three
specs to the measurement (specs/api/tri_api_loop.t27, tri_api_permissions.t27, tri_api_context.t27).

THREE KINDS OF EVIDENCE, KEPT APART
-----------------------------------
  model    a Python reading of the source (extract_field, parse_response, rule matching, the path and
           bash predicates, context truncation), written independently of the Zig and compared with it
  zig      the pinned .zig files compiled with Zig 0.15.2 and run: their own unit tests, plus generated
           `test` blocks appended to a copy of each file so that private functions are reachable; every
           vector is run in Debug and ReleaseFast, and the permission loader in four optimize modes
  binary   the pinned tri-api built from src/tri-api/main.zig and run against a scripted fake
           Anthropic server and a scripted fake MCP server: real process, real files, real exit codes
Nothing here talks to a real provider. No credential appears anywhere; the binary gets a fake key.
A real provider, a real model and a real MCP server are NOT measured, and the record says so.

Usage:
  python3 tools/trinity_tri_api.py inventory --trinity-root <clone> [--rev <sha>]
  python3 tools/trinity_tri_api.py vectors
  python3 tools/trinity_tri_api.py zig  --trinity-root <clone> --zig <zig> [--sysroot <dir>]
  python3 tools/trinity_tri_api.py e2e  --trinity-root <clone> --zig <zig> [--sysroot <dir>]
  python3 tools/trinity_tri_api.py run
  python3 tools/trinity_tri_api.py check
  python3 tools/trinity_tri_api.py --self-check [--trinity-root <clone> --zig <zig>]

Exit codes: 0 no finding; 1 findings; 2 could not run.
"""
from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
import pathlib
import platform
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, HTTPServer

ROOT = pathlib.Path(__file__).resolve().parents[1]
API_DIR = ROOT / "specs/api"
SPEC_LOOP = API_DIR / "tri_api_loop.t27"
SPEC_PERM = API_DIR / "tri_api_permissions.t27"
SPEC_CTX = API_DIR / "tri_api_context.t27"
SPECS = {"loop": SPEC_LOOP, "permissions": SPEC_PERM, "context": SPEC_CTX}
OUT_DIR = ROOT / "conformance/trinity"
INVENTORY = OUT_DIR / "tri_api_inventory.json"
VECTORS = OUT_DIR / "tri_api_vectors.json"
EVIDENCE_ZIG = OUT_DIR / "tri_api_zig.json"
EVIDENCE_E2E = OUT_DIR / "tri_api_e2e.json"
TRINITY_REPO = "gHashTag/trinity"
DEFAULT_REV = "afc9d38435ad01c48af6c33c311da422c38f046e"
ISSUE_BASELINE = "03ae2f93f5af2fd4fa23e4c613a3c0e19ac51530"
SRC_FILES = ["main", "tool_protocol", "tool_executor", "permissions", "context", "checkpoint", "mcp_client",
             "session_store", "memory", "claude_md", "tui", "perplexity_bridge"]
ROTATOR_FILE = "src/tri/token_rotator.zig"
CONST_RE = re.compile(r"^pub const (\w+) : ([^=]+?) = (.*);\s*$")
INT_TYPES = {"i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "usize", "isize"}


def sha256(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def run(cmd, cwd=None, timeout=300, env=None):
    try:
        p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout, env=env)
        return p.returncode, p.stdout, p.stderr
    except (OSError, subprocess.TimeoutExpired) as e:
        return 127, "", str(e)


def t27c_path():
    for p in ("target/release/t27c", "target/debug/t27c"):
        if (ROOT / p).exists():
            return ROOT / p
    w = shutil.which("t27c")
    return pathlib.Path(w) if w else None


def parse_value(raw: str, typ: str):
    raw = raw.strip()
    if typ == "bool":
        return raw == "true"
    if typ in INT_TYPES:
        return int(raw)
    if typ == "str":
        return json.loads(raw)
    if typ == "f64":
        return float(raw)
    m = re.fullmatch(r"\[(\d+)\](\w+)", typ)
    if m:
        inner = raw[1:-1].strip()
        if m.group(2) == "str":
            items = json.loads("[" + inner + "]") if inner else []
        elif m.group(2) == "bool":
            items = [x.strip() == "true" for x in inner.split(",") if x.strip()]
        else:
            items = [int(x) for x in inner.split(",") if x.strip()]
        if len(items) != int(m.group(1)):
            raise ValueError(f"annotated {typ}, holds {len(items)}")
        return items
    raise ValueError(f"unknown type {typ}")


def load_spec(path: pathlib.Path) -> dict:
    text = path.read_text(encoding="utf-8")
    if re.search(r"[^\x00-\x7f]", text):
        raise ValueError(f"{path}: non-ASCII byte (L3)")
    f = {"_path": str(path)}
    lines = text.split("\n")
    n = 0
    while n < len(lines):
        line = lines[n]
        n += 1
        if line.startswith("pub const") and not line.rstrip().endswith(";"):
            while n < len(lines):          # a constant written over several lines ends at the line that ends in ;
                line += "\n" + lines[n]
                n += 1
                if lines[n - 1].rstrip().endswith(";"):
                    break
            line = re.sub(r"\n\s*", " ", line.replace('",\n', '", '))
        m = CONST_RE.match(line)
        if m:
            f[m.group(1)] = parse_value(m.group(3), m.group(2).strip())
        elif line.startswith("pub const"):
            raise ValueError(f"{path}:{n}: cannot read constant line")
    return f


def git_show(trinity: pathlib.Path, rev: str, rel: str) -> bytes:
    rc = subprocess.run(["git", "-C", str(trinity), "show", f"{rev}:{rel}"], capture_output=True)
    if rc.returncode != 0:
        raise FileNotFoundError(f"{rev}:{rel}: {rc.stderr.decode()[:120]}")
    return rc.stdout


# ===========================================================================================
# MODEL: a reading of the pinned Zig, byte for byte. Bytes in, bytes out; Zig's slices are
# Python slices, Zig's `indexOfPos` is bytes.find. Each function names the source it reads.
# ===========================================================================================
WS = b" \t\n\r\x0b\x0c"


def extract_field(data: bytes, key: bytes):
    """tool_protocol.zig extractField: the value of "key":"..." -- a STRING value only, compact form only."""
    needle = b'"' + key + b'":"'
    if len(needle) > 128:
        return None
    idx = data.find(needle)
    if idx < 0:
        return None
    start = idx + len(needle)
    if start >= len(data):
        return None
    end = start
    while end < len(data):
        if data[end:end + 1] == b'"' and (end == start or data[end - 1:end] != b"\\"):
            break
        end += 1
    if end == start:
        return None
    return data[start:end]


def extract_field_after(data: bytes, key: bytes, after: bytes):
    m = data.find(after)
    if m < 0:
        return None
    return extract_field(data[m:], key)


def extract_object(data: bytes, search_start: int, key: bytes):
    needle = b'"' + key + b'":'
    if len(needle) > 128:
        return None
    idx = data.find(needle, search_start)
    if idx < 0:
        return None
    start = idx + len(needle)
    while start < len(data) and data[start:start + 1] in (b" ", b"\t", b"\n"):
        start += 1
    if start >= len(data) or data[start:start + 1] != b"{":
        return None
    depth = 0
    in_string = False
    end = start
    while end < len(data):
        c = data[end:end + 1]
        if in_string:
            if c == b'"' and (end == 0 or data[end - 1:end] != b"\\"):
                in_string = False
            end += 1
            continue
        if c == b'"':
            in_string = True
        elif c == b"{":
            depth += 1
        elif c == b"}":
            depth -= 1
            if depth == 0:
                return data[start:end + 1]
        end += 1
    return None


def find_block_end(data: bytes, start: int) -> int:
    search_start = start + 10
    if search_start >= len(data):
        return len(data)
    nxt = data.find(b'"type":', search_start)
    if nxt < 0:
        return len(data)
    pos = nxt
    while pos > start and data[pos:pos + 1] != b"{":
        pos -= 1
    return pos


def parse_int_u32(b: bytes) -> int:
    try:
        s = b.decode("ascii")
        if re.fullmatch(r"[0-9]+", s) and int(s) < 2 ** 32:
            return int(s)
    except UnicodeDecodeError:
        pass
    return 0


class Hang(Exception):
    """parse_response would never return: the Zig loop makes no progress."""


def parse_response(body: bytes, cap: int = 100000) -> dict:
    """tool_protocol.zig parseResponse. A step that makes no progress is reported as Hang, not looped."""
    stop = b"end_turn"
    sr = extract_field(body, b"stop_reason")
    if sr is not None:
        stop = sr
    it = extract_field(body, b"input_tokens")
    ot = extract_field(body, b"output_tokens")
    blocks = []
    pos = 0
    steps = 0
    while pos < len(body):
        steps += 1
        if steps > cap:
            raise Hang()
        idx = body.find(b'"type":"text"', pos)
        if idx >= 0:
            tool_idx = body.find(b'"type":"tool_use"', pos)
            if tool_idx < 0:
                tool_idx = len(body)
            if idx < tool_idx:
                bs = idx - 50 if idx >= 50 else 0
                be = min(idx + 8192, len(body))
                tv = extract_field_after(body[bs:be], b"text", b'"type":"text"')
                if tv is not None:
                    blocks.append(("text", tv))
                pos = idx + 12
                continue
        idx = body.find(b'"type":"tool_use"', pos)
        if idx >= 0:
            be = find_block_end(body, idx)
            block = body[idx:be]
            bid = extract_field(block, b"id") or b"unknown"
            name = extract_field(block, b"name") or b"unknown"
            inp = extract_object(body, idx, b"input") or b"{}"
            blocks.append(("tool_use", bid, name, inp))
            if be == pos:
                raise Hang()
            pos = be
            continue
        break
    return {"stop": stop, "in": parse_int_u32(it) if it is not None else 0,
            "out": parse_int_u32(ot) if ot is not None else 0, "blocks": blocks}


def json_escape(s: bytes) -> bytes:
    out = bytearray()
    for c in s:
        if c == 0x22:
            out += b'\\"'
        elif c == 0x5C:
            out += b"\\\\"
        elif c == 0x0A:
            out += b"\\n"
        elif c == 0x0D:
            out += b"\\r"
        elif c == 0x09:
            out += b"\\t"
        elif c <= 0x08 or c in (0x0B, 0x0C) or 0x0E <= c <= 0x1F:
            out += b"\\u%04x" % c
        else:
            out.append(c)
    return bytes(out)


def unescape_string(s: bytes) -> bytes:
    out = bytearray()
    i = 0
    m = {ord("n"): 0x0A, ord("t"): 0x09, ord("r"): 0x0D, ord("\\"): 0x5C, ord('"'): 0x22}
    while i < len(s):
        if s[i] == 0x5C and i + 1 < len(s) and s[i + 1] in m:
            out.append(m[s[i + 1]])
            i += 2
        else:
            out.append(s[i])
            i += 1
    return bytes(out)


# ---- permissions.zig ----------------------------------------------------------------------
MAX_RULES = 64
DEFAULT_ALLOW_TOOLS = (b"read_file", b"grep")


def parse_rule_string(s: bytes):
    p = s.find(b"(")
    if p < 0:
        return None
    if len(s) < 3 or s[-1:] != b")":
        return None
    tool, pattern = s[:p], s[p + 1:len(s) - 1]
    if len(tool) == 0:
        return None
    return (tool, pattern)


def rule_matches(rule, tool: bytes, arg: bytes) -> bool:
    if rule[0] != tool:
        return False
    pat = rule[1]
    if len(pat) == 0 or pat == b"*":
        return True
    if pat[-1:] == b"*":
        prefix = pat[:-1]
        return len(arg) >= len(prefix) and arg[:len(prefix)] == prefix
    return arg == pat


def check_permission(allow_rules, deny_rules, tool: bytes, arg: bytes) -> str:
    for r in deny_rules:
        if rule_matches(r, tool, arg):
            return "deny"
    for r in allow_rules:
        if rule_matches(r, tool, arg):
            return "allow"
    return "allow" if tool in DEFAULT_ALLOW_TOOLS else "deny"


def parse_rule_array(data: bytes, key: bytes, rules: list) -> list:
    needle = b'"' + key + b'":['
    if len(needle) > 64:
        return rules
    idx = data.find(needle)
    if idx < 0:
        return rules
    pos = idx + len(needle)
    while pos < len(data) and data[pos:pos + 1] != b"]":
        q = data.find(b'"', pos)
        if q < 0:
            break
        s0 = q + 1
        s1 = s0
        while s1 < len(data) and data[s1:s1 + 1] != b'"':
            s1 += 1
        if s1 > s0:
            r = parse_rule_string(data[s0:s1])
            if r is not None and len(rules) < MAX_RULES:
                rules.append(r)
        pos = s1 + 1
    return rules


# ---- tool_executor.zig ----------------------------------------------------------------------
BASH_PREFIXES = [b"git ", b"git\x00", b"zig ", b"zig\x00", b"cat ", b"ls ", b"grep ", b"find ", b"echo ", b"mkdir ",
                 b"rm ", b"tri ", b"docker ", b"gh ", b"head ", b"tail ", b"wc ", b"pwd", b"date", b"env", b"which ",
                 b"file ", b"diff ", b"sort ", b"test ", b"cd ", b"cp ", b"mv ", b"chmod ", b"touch ", b"sed ", b"awk "]
SHELL_META = b"|;`$(){}"
BARE_OK = [b"pwd", b"date", b"env", b"ls"]
SECRET_PREFIXES = [b"sk-ant-", b"ghp_", b"ghu_", b"ghs_", b"xoxb-", b"xoxp-", b"AKIA"]


def is_path_safe(path: bytes) -> bool:
    if b".." in path:
        return False
    if b"\x00" in path:
        return False
    return True


def is_bash_allowed(command: bytes) -> bool:
    t = command.lstrip(WS)
    if b"&&" in t or b"||" in t:
        return False
    if any(bytes([m]) in t for m in SHELL_META):
        return False
    if any(t.startswith(p) for p in BASH_PREFIXES):
        return True
    return t in BARE_OK


def redact_secrets(inp: bytes, buf_len: int = 256) -> bytes:
    if len(inp) == 0:
        return inp
    if len(inp) > buf_len:
        return b"[REDACTED:too_long]"
    out = bytearray(inp)
    for prefix in SECRET_PREFIXES:
        pos = 0
        while pos < len(out):
            idx = bytes(out).find(prefix, pos)
            if idx < 0:
                break
            end = idx + len(prefix)
            while end < len(out) and out[end:end + 1] not in (b" ", b'"', b",", b"\n", b"'"):
                end += 1
            red = b"[REDACTED]"
            if end - idx >= len(red):
                out[idx:idx + len(red)] = red
                for k in range(idx + len(red), end):
                    out[k] = 0x20
            pos = end
    return bytes(out)


# ---- context.zig ----------------------------------------------------------------------------
CTX_MAX, CTX_THRESHOLD, CTX_KEEP = 180000, 144000, 3
ASSIST = b'"role":"assistant"'
TOOL_MARK = b'"type":"tool_result"'
CONTENT_MARK = b'"content":"'
SUMMARY_PROMPT = (b"Summarize the following conversation concisely in 2-3 paragraphs. Preserve: all file paths mentioned, "
                  b"key decisions made, current task state, and any errors encountered. Conversation:\\n\\n")
EXCERPT_MAX = 400000


def estimate_tokens(n: int) -> int:
    return max(1, n // 4)


def is_near_limit(n: int, threshold: int = CTX_THRESHOLD) -> bool:
    return estimate_tokens(n) >= threshold


def truncate_old_tool_outputs(data: bytes, keep: int = CTX_KEEP):
    """context.zig truncateOldToolOutputs -> (modified, new_data)."""
    L = len(ASSIST)
    turn_count = 0
    cutoff = len(data)
    search_end = len(data)
    while search_end > L:
        pos = search_end - 1
        found = False
        while pos >= L:
            if data[pos - L + 1:pos + 1] == ASSIST:
                turn_count += 1
                if turn_count == keep:
                    cutoff = pos - L + 1
                    found = True
                    break
                search_end = pos - L + 1
                found = True
                break
            if pos == 0:
                break
            pos -= 1
        if not found:
            break
        if cutoff < len(data):
            break
    if cutoff >= len(data):
        return False, data
    out = bytearray()
    modified = False
    i = 0
    while i < len(data):
        if i < cutoff and data[i:i + len(TOOL_MARK)] == TOOL_MARK:
            out += TOOL_MARK
            j = i + len(TOOL_MARK)
            ci = data.find(CONTENT_MARK, j)
            if ci >= 0 and ci < cutoff and ci - j < 200:
                out += data[j:ci]
                out += CONTENT_MARK
                vs = ci + len(CONTENT_MARK)
                ve = vs
                while ve < len(data):
                    if data[ve:ve + 1] == b'"' and (ve == vs or data[ve - 1:ve] != b"\\"):
                        break
                    ve += 1
                olen = ve - vs
                if olen > 200:
                    out += b"[truncated %d bytes]" % olen
                    modified = True
                else:
                    out += data[vs:ve]
                i = ve
                continue
            i = j
            continue
        out.append(data[i])
        i += 1
    return (True, bytes(out)) if modified else (False, data)


def apply_summary(data: bytes, summary: bytes, keep: int = CTX_KEEP) -> bytes:
    """context.zig applySummary: the new messages buffer (no closing bracket, as in the source)."""
    turn_count = 0
    keep_from = len(data)
    search_pos = len(data)
    while search_pos > 0:
        pos = data[:search_pos].rfind(ASSIST)
        if pos < 0:
            break
        turn_count += 1
        if turn_count == keep:
            ms = pos
            while ms > 0 and data[ms:ms + 1] != b",":
                ms -= 1
            keep_from = ms
            break
        search_pos = pos
    new = b'[{"role":"user","content":"[Previous context summary]\\n' + json_escape(summary) + b'"}'
    if keep_from < len(data):
        new += data[keep_from:]
    return new


def build_compaction_request(messages: bytes, model: bytes):
    if not is_near_limit(len(messages)):
        return None
    body = b'{"model":"' + model + b'","max_tokens":2048,"messages":[{"role":"user","content":"' + SUMMARY_PROMPT
    body += json_escape(messages[:min(len(messages), EXCERPT_MAX)])
    body += b'"}]}'
    return body


# ===========================================================================================
# VECTORS: inputs as latin-1 strings (one char per byte), expected results from the MODEL.
# ===========================================================================================
def L(b: bytes) -> str:
    return b.decode("latin-1")


def B(s: str) -> bytes:
    return s.encode("latin-1")


def messages_json(assistant_turns: int, tool_len: int, tail: bytes = b"") -> bytes:
    """The conversation buffer as main.zig builds it: an open array, no closing bracket."""
    out = b'[{"role":"user","content":"do the thing"}'
    for i in range(assistant_turns):
        out += b',{"role":"assistant","content":[{"type":"tool_use","id":"t%d","name":"read_file","input":{"path":"a.txt"}}]}' % i
        out += b',{"role":"user","content":[{"type":"tool_result","tool_use_id":"t%d","content":"%s"}]}' % (i, b"x" * tool_len)
    return out + tail


def build_vectors() -> list:
    V = []

    def add(kind, vid, note, **inp):
        V.append({"kind": kind, "id": vid, "note": note, "in": inp})

    # --- tool_protocol.extractField -------------------------------------------------------
    for vid, data, key, note in [
        ("ef-basic", b'{"name":"hello","value":"world"}', b"name", "compact string value"),
        ("ef-missing", b'{"name":"hello"}', b"value", "absent key"),
        ("ef-empty-value", b'{"k":"","z":"after"}', b"k", "an empty string value reads as absent"),
        ("ef-escaped-quote", b'{"k":"a\\"b"}', b"k", "the raw escaped text is returned, not the decoded value"),
        ("ef-space-after-colon", b'{"k": "v"}', b"k", "one space after the colon and the key is not found"),
        ("ef-number", b'{"input_tokens":1234}', b"input_tokens", "a numeric value is never read: only quoted strings are"),
        ("ef-number-quoted", b'{"input_tokens":"1234"}', b"input_tokens", "the same number as a string is read"),
        ("ef-unterminated", b'{"k":"runs to the end', b"k", "no closing quote: the value runs to the end of the data"),
        ("ef-at-end", b'{"k":"', b"k", "the needle ends the data: nothing to read"),
        ("ef-first-wins", b'{"a":{"k":"inner"},"k":"outer"}', b"k", "the first textual match wins, nested or not"),
        ("ef-backslash-path", b'{"p":"C:\\\\x"}', b"p", "a doubled backslash before a quote is taken as an escaped quote"),
        ("ef-utf8", B("{\"k\":\"\u043f\u0440\u0438\"}".encode("utf-8").decode("latin-1")), b"k", "multi-byte text is handled byte for byte"),
    ]:
        add("extract_field", vid, note, data=L(data), key=L(key))

    # --- tool_protocol.parseResponse -------------------------------------------------------
    usage = b',"stop_reason":"end_turn","usage":{"input_tokens":10,"output_tokens":5}}'
    bodies = [
        ("pr-text", b'{"content":[{"type":"text","text":"hello"}]' + usage, "a single text block"),
        ("pr-tool-use", b'{"content":[{"type":"tool_use","id":"t1","name":"read_file","input":{"path":"a.txt"}}],"stop_reason":"tool_use"}', "a single tool_use block"),
        ("pr-text-then-tool", b'{"content":[{"type":"text","text":"reading"},{"type":"tool_use","id":"t1","name":"bash","input":{"command":"ls"}}],"stop_reason":"tool_use"}', "text then tool_use, in order"),
        ("pr-two-tools", b'{"content":[{"type":"tool_use","id":"a","name":"read_file","input":{"path":"1"}},{"type":"tool_use","id":"b","name":"read_file","input":{"path":"2"}}],"stop_reason":"tool_use"}', "two tool_use blocks"),
        ("pr-usage-numbers", b'{"content":[{"type":"text","text":"x"}],"stop_reason":"end_turn","usage":{"input_tokens":1234,"output_tokens":56}}', "provider usage as JSON numbers reads as 0 and 0"),
        ("pr-usage-strings", b'{"content":[{"type":"text","text":"x"}],"stop_reason":"end_turn","usage":{"input_tokens":"1234","output_tokens":"56"}}', "usage as strings is read"),
        ("pr-pretty", b'{\n "content": [\n  {"type": "text", "text": "hi"}\n ],\n "stop_reason": "end_turn"\n}', "pretty-printed JSON yields no blocks"),
        ("pr-stop-space", b'{"content":[{"type":"text","text":"hi"}],"stop_reason": "tool_use"}', "a space after the colon makes stop_reason default to end_turn"),
        ("pr-garbage", b"this is not json at all", "a body that is not JSON yields nothing and no error"),
        ("pr-empty", b"", "an empty body"),
        ("pr-error-body", b'{"type":"error","error":{"type":"overloaded_error","message":"busy"}}', "a provider error object yields no blocks"),
        ("pr-tool-no-id", b'{"content":[{"type":"tool_use","name":"bash","input":{"command":"ls"}}],"stop_reason":"tool_use"}', "a tool_use without an id is named unknown"),
        ("pr-tool-no-input", b'{"content":[{"type":"tool_use","id":"t1","name":"grep"}],"stop_reason":"tool_use"}', "a tool_use without input gets {}"),
        ("pr-nested-text", b'{"content":[{"type":"tool_use","id":"t1","name":"read_file","input":{"path":"a.txt","meta":{"type":"text","text":"INJECTED"}}}],"stop_reason":"tool_use"}', "a nested type:text inside a tool input is read as a text block"),
        ("pr-text-2byte", b'{"content":[{"type":"text","text":"' + b"".join(bytes([65 + (i % 26)]) for i in range(20000)) + b'"}],"stop_reason":"end_turn"}', "a text block longer than the 8192-byte window is cut"),
        ("pr-max-tokens", b'{"content":[{"type":"text","text":"cut"}],"stop_reason":"max_tokens"}', "max_tokens read as the stop reason"),
        ("pr-escaped-in-text", b'{"content":[{"type":"text","text":"line1\\nline2 \\"q\\""}],"stop_reason":"end_turn"}', "escapes are returned raw"),
    ]
    for vid, body, note in bodies:
        add("parse_response", vid, note, body=L(body))
    add("parse_response_hang", "pr-hang-duplicate-type", "a tool_use block whose own second `type` key precedes any brace makes the scan spin forever (measured on the binary, not run here)",
        body=L(b'{"content":[{"type":"tool_use","id":"a","name":"read_file","type":"x"}],"stop_reason":"tool_use"}'))

    # --- escape / unescape -----------------------------------------------------------------
    for vid, s, note in [
        ("esc-basic", b'line1\nline2\ttab"quote\\back', "the escaped forms"),
        ("esc-control", b"\x01\x02\x08\x0b\x0c\x0e\x1f", "control bytes as \\u00xx"),
        ("esc-del-and-high", b"\x7f\xc3\xa9", "DEL and high bytes pass through raw"),
    ]:
        add("json_escape", vid, note, s=L(s))
    for vid, s, note in [
        ("unesc-basic", b"hello\\nworld\\t!\\\\ \\\" \\r", "the five escapes"),
        ("unesc-unicode", b"\\u0041 and \\/ slash", "\\uXXXX and \\/ are not decoded"),
        ("unesc-trailing", b"abc\\", "a trailing backslash is kept"),
    ]:
        add("json_unescape", vid, note, s=L(s))
    for vid, data, start, key, note in [
        ("eo-basic", b'{"name":"read_file","input":{"path":"build.zig"}}', 0, b"input", "a flat object"),
        ("eo-nested", b'{"input":{"a":{"b":"}"},"c":"{"}}', 0, b"input", "braces inside strings do not count"),
        ("eo-not-object", b'{"input":"text"}', 0, b"input", "a string value is not an object"),
        ("eo-space", b'{"input": {"p":1}}', 0, b"input", "whitespace before the brace is skipped"),
        ("eo-unterminated", b'{"input":{"p":1', 0, b"input", "no closing brace"),
    ]:
        add("extract_object", vid, note, data=L(data), start=start, key=L(key))

    # --- permissions -----------------------------------------------------------------------
    for vid, s, note in [
        ("prs-basic", b"bash(git diff *)", "tool and pattern"),
        ("prs-empty-pattern", b"bash()", "an empty pattern"),
        ("prs-no-paren", b"invalid", "no parenthesis"),
        ("prs-empty", b"", "empty"),
        ("prs-empty-tool", b"(x)", "empty tool name"),
        ("prs-no-close", b"bash(ls", "no closing parenthesis"),
        ("prs-nested", b"bash(echo (x))", "the pattern is everything between the first ( and the last )"),
        ("prs-too-short", b"a(", "shorter than three bytes"),
        ("prs-mcp", b"github.create_issue(*)", "an MCP name carries a dot"),
    ]:
        add("parse_rule_string", vid, note, s=L(s))
    for vid, tool, pat, t, a, note in [
        ("rm-star", b"bash", b"*", b"bash", b"anything", "star matches every argument"),
        ("rm-star-tool", b"bash", b"*", b"grep", b"anything", "a different tool never matches"),
        ("rm-empty", b"bash", b"", b"bash", b"x", "an empty pattern matches every argument"),
        ("rm-prefix", b"bash", b"git diff *", b"bash", b"git diff HEAD", "prefix: the star is only honoured as the last byte"),
        ("rm-prefix-miss", b"bash", b"git diff *", b"bash", b"git  diff HEAD", "two spaces escape a prefix rule"),
        ("rm-prefix-short", b"bash", b"git diff *", b"bash", b"git diff", "the argument is shorter than the prefix"),
        ("rm-exact", b"write_file", b".env", b"write_file", b".env", "exact match"),
        ("rm-exact-miss", b"write_file", b".env", b"write_file", b".env.local", "exact is not a prefix"),
        ("rm-mid-star", b"bash", b"a*b", b"bash", b"aXb", "a star in the middle is a literal"),
        ("rm-mid-star-lit", b"bash", b"a*b", b"bash", b"a*b", "and matches only itself"),
        ("rm-mcp-arg", b"fake.echo", b"hello*", b"fake.echo", b"", "an MCP call is checked with an empty argument, so a prefix rule never matches"),
        ("rm-mcp-star", b"fake.echo", b"*", b"fake.echo", b"", "only * or empty can allow or deny an MCP tool"),
    ]:
        add("rule_matches", vid, note, tool=L(tool), pattern=L(pat), t=L(t), a=L(a))
    for vid, allow, deny, t, a, note in [
        ("ck-deny-wins", [(b"bash", b"*")], [(b"bash", b"rm -rf *")], b"bash", b"rm -rf /", "deny beats allow"),
        ("ck-allow", [(b"bash", b"*")], [(b"bash", b"rm -rf *")], b"bash", b"ls", "allow when no deny matches"),
        ("ck-default-read", [], [], b"read_file", b"any", "read_file defaults to allow"),
        ("ck-default-grep", [], [], b"grep", b"any", "grep defaults to allow"),
        ("ck-default-bash", [], [], b"bash", b"any", "bash defaults to deny"),
        ("ck-default-write", [], [], b"write_file", b"any", "write_file defaults to deny"),
        ("ck-default-unknown", [], [], b"nope", b"any", "an unknown tool defaults to deny"),
        ("ck-deny-default-allow", [], [(b"read_file", b".env")], b"read_file", b".env", "a deny rule can close a default-allow tool"),
        ("ck-mcp-default", [], [], b"fake.echo", b"", "an MCP tool defaults to deny"),
        ("ck-mcp-allow", [(b"fake.echo", b"*")], [], b"fake.echo", b"", "an MCP tool with an allow rule"),
        ("ck-mcp-prefix-allow", [(b"fake.echo", b"hello*")], [], b"fake.echo", b"", "an MCP allow with a restricting pattern never matches"),
        ("ck-mcp-prefix-deny", [(b"fake.echo", b"*")], [(b"fake.echo", b"hello*")], b"fake.echo", b"", "an MCP deny with a restricting pattern never matches either"),
    ]:
        add("check", vid, note, allow=[[L(x), L(y)] for x, y in allow], deny=[[L(x), L(y)] for x, y in deny], t=L(t), a=L(a))
    many = b'{"permissions":{"allow":[' + b",".join(b'"bash(c%d *)"' % i for i in range(70)) + b"]}}"
    for vid, data, key, note in [
        ("pra-compact", b'{"permissions":{"allow":["bash(echo *)","read_file(*)"],"deny":["bash(rm -rf *)"]}}', b"allow", "compact JSON"),
        ("pra-compact-deny", b'{"permissions":{"allow":["bash(echo *)"],"deny":["bash(rm -rf *)"]}}', b"deny", "the deny key"),
        ("pra-pretty", b'{\n "permissions": {\n  "allow": [\n   "bash(echo *)"\n  ]\n }\n}', b"allow", "a space after the colon finds no array and loads no rule"),
        ("pra-space-before-bracket", b'{"allow": ["bash(echo *)"]}', b"allow", "the needle is exactly \"allow\":[ with no space"),
        ("pra-empty", b'{"allow":[]}', b"allow", "an empty array"),
        ("pra-absent", b'{"deny":["bash(x)"]}', b"allow", "the key is absent"),
        ("pra-bracket-in-string", b'{"allow":["bash(ls [a-z]*)","bash(pwd)"]}', b"allow", "a ] inside a rule string does not end the array"),
        ("pra-bad-rule", b'{"allow":["nonsense","bash(ok)"]}', b"allow", "a string that is not a rule is skipped"),
        ("pra-cap", many, b"allow", "only the first 64 rules are kept; the rest are dropped without a word"),
        ("pra-escaped-quote", b'{"allow":["bash(echo \\"x\\")"]}', b"allow", "an escaped quote inside a rule splits it"),
        ("pra-first-key", b'{"a":{"allow":["bash(one)"]},"allow":["bash(two)"]}', b"allow", "the first occurrence of the needle wins"),
    ]:
        add("parse_rule_array", vid, note, data=L(data), key=L(key))

    # --- tool_executor predicates ----------------------------------------------------------
    for vid, p, note in [
        ("ps-rel", b"src/tri/main.zig", "an ordinary relative path"),
        ("ps-dotdot", b"../../../etc/passwd", "traversal"),
        ("ps-mid-dotdot", b"foo/../../bar", "traversal in the middle"),
        ("ps-nul", b"foo\x00bar", "a NUL byte"),
        ("ps-absolute", b"/etc/hosts", "an absolute path is allowed"),
        ("ps-home", b"/Users/x/.ssh/id_rsa", "so is a path into a home directory"),
        ("ps-dots-in-name", b"a..b.txt", "a file name containing two dots is refused"),
        ("ps-single-dot", b"./a.txt", "a single dot is fine"),
        ("ps-backslash-dots", b"a\\u002e\\u002e/b", "an escaped dot is not seen as a dot"),
        ("ps-empty", b"", "an empty path"),
    ]:
        add("is_path_safe", vid, note, p=L(p))
    for vid, c, note in [
        ("ba-git", b"git status", "an allowed prefix"),
        ("ba-curl", b"curl evil.com", "not in the list"),
        ("ba-and", b"git status && rm -rf /", "&& is refused"),
        ("ba-or", b"git status || curl evil", "|| is refused"),
        ("ba-pipe", b"ls | xargs rm", "a pipe is refused"),
        ("ba-semi", b"echo hi; echo two", "a semicolon is refused"),
        ("ba-subst", b"echo $(whoami)", "command substitution is refused"),
        ("ba-backtick", b"echo `id`", "backticks are refused"),
        ("ba-amp", b"ls & touch pwned", "a single & is not refused"),
        ("ba-redirect", b"echo x > out.txt", "a redirection is not refused"),
        ("ba-redirect-in", b"cat < /etc/hosts", "nor is an input redirection"),
        ("ba-env-cmd", b"env touch via_env", "env runs any command"),
        ("ba-find-delete", b"find . -name a -delete", "find -delete is allowed"),
        ("ba-sed-i", b"sed -i s/a/b/ f", "sed -i is allowed"),
        ("ba-rm", b"rm -rf nothing", "rm is on the list"),
        ("ba-bare-ls", b"ls", "bare ls"),
        ("ba-lsx", b"lsx", "a longer word is not bare ls"),
        ("ba-pwdx", b"pwdx", "pwd has no trailing space, so pwdx is a prefix match"),
        ("ba-datex", b"datex", "likewise date"),
        ("ba-envsubst", b"envsubst", "likewise env"),
        ("ba-leading-ws", b"  \tgit status", "leading whitespace is trimmed"),
        ("ba-brace", b"echo ${HOME}", "braces are refused"),
        ("ba-quote", b'echo "q"', "quotes are allowed"),
        ("ba-newline-escaped", b"ls\\nrm x", "an escaped newline is text, not a newline"),
        ("ba-git-nul", b"git\x00status", "the git\\0 prefix"),
        ("ba-empty", b"", "an empty command"),
    ]:
        add("is_bash_allowed", vid, note, c=L(c))
    for vid, s, note in [
        ("rd-anthropic", b"key=sk-ant-abc123xyz", "an Anthropic key"),
        ("rd-github", b"token ghp_1234567890abcdef done", "a GitHub token"),
        ("rd-clean", b"git status --short", "nothing to redact"),
        ("rd-short-aws", b"AKIA12345", "a short AKIA token of nine bytes is NOT redacted"),
        ("rd-aws-ok", b"AKIAAAAAAAAAAAAAAAAA x", "a long one is"),
        ("rd-quote-end", b'{"k":"sk-ant-secretsecretsecret","z":1}', "the token ends at the quote"),
        ("rd-too-long", b"a" * 300, "input over 256 bytes"),
        ("rd-empty", b"", "empty"),
        ("rd-two", b"sk-ant-aaaaaaaaaaaa ghp_bbbbbbbbbbbbb", "two secrets"),
    ]:
        add("redact", vid, note, s=L(s))

    # --- context ---------------------------------------------------------------------------
    for vid, n, note in [
        ("tok-eight", 8, "8 bytes are 2 tokens"),
        ("tok-two", 2, "a minimum of one token"),
        ("tok-zero", 0, "an empty text is one token"),
    ]:
        add("estimate_tokens", vid, note, n=n)
    for vid, n, note in [
        ("near-under", 575999, "just under the 144000-token threshold"),
        ("near-at", 576000, "exactly at it"),
        ("near-over", 576003, "over it"),
    ]:
        add("is_near_limit", vid, note, n=n)
    add("truncate", "tr-few-turns", "fewer than three assistant turns: nothing is truncated", data=L(messages_json(2, 500)), keep=3)
    add("truncate", "tr-five-turns", "five turns: results before the last three are cut", data=L(messages_json(5, 500)), keep=3)
    add("truncate", "tr-short-results", "short results are kept even before the cut", data=L(messages_json(5, 150)), keep=3)
    add("truncate", "tr-boundary", "a result of exactly 200 bytes is kept, 201 is cut", data=L(messages_json(4, 200)), keep=3)
    add("truncate", "tr-boundary-201", "201 bytes", data=L(messages_json(4, 201)), keep=3)
    add("truncate", "tr-keep-1", "keep one turn", data=L(messages_json(3, 300)), keep=1)
    add("summary", "sm-no-turns", "no assistant turn: the whole history, the first prompt included, becomes the summary", data=L(b'[{"role":"user","content":"do the thing"}'), summary=L(b"SUMMARY"), keep=3)
    add("summary", "sm-two-turns", "two turns: still everything is summarised", data=L(messages_json(2, 10)), summary=L(b"SUMMARY"), keep=3)
    add("summary", "sm-five-turns", "five turns: the last three are kept verbatim, the first prompt is not", data=L(messages_json(5, 10)), summary=L(b"SUMMARY"), keep=3)
    add("summary", "sm-escape", "the summary is JSON-escaped", data=L(messages_json(4, 10)), summary=L(b'a "quoted"\nline'), keep=3)
    add("compaction", "cr-under", "under the threshold: no request", messages=L(messages_json(1, 10)), model=L(b"m"), threshold=144000)
    add("compaction", "cr-small-threshold", "a lowered threshold makes the request", messages=L(messages_json(2, 10)), model=L(b"claude-x"), threshold=10)
    add("compaction", "cr-big", "the excerpt is capped at 400000 bytes", messages=L(messages_json(1, 500000)), model=L(b"m"), threshold=10)
    return V


def model_result(v: dict):
    """The model's answer for one vector, as a JSON-able value (bytes as latin-1 strings)."""
    k, i = v["kind"], v["in"]
    if k == "extract_field":
        r = extract_field(B(i["data"]), B(i["key"]))
        return None if r is None else L(r)
    if k == "parse_response":
        p = parse_response(B(i["body"]))
        blocks = []
        for b in p["blocks"]:
            if b[0] == "text":
                blocks.append({"t": "text", "v": L(b[1])})
            else:
                blocks.append({"t": "tool_use", "id": L(b[1]), "name": L(b[2]), "input": L(b[3])})
        return {"stop": L(p["stop"]), "in": p["in"], "out": p["out"], "blocks": blocks}
    if k == "parse_response_hang":
        try:
            parse_response(B(i["body"]))
            return "returns"
        except Hang:
            return "hang"
    if k == "json_escape":
        return L(json_escape(B(i["s"])))
    if k == "json_unescape":
        return L(unescape_string(B(i["s"])))
    if k == "extract_object":
        r = extract_object(B(i["data"]), i["start"], B(i["key"]))
        return None if r is None else L(r)
    if k == "parse_rule_string":
        r = parse_rule_string(B(i["s"]))
        return None if r is None else {"tool": L(r[0]), "pattern": L(r[1])}
    if k == "rule_matches":
        return rule_matches((B(i["tool"]), B(i["pattern"])), B(i["t"]), B(i["a"]))
    if k == "check":
        return check_permission([(B(x), B(y)) for x, y in i["allow"]], [(B(x), B(y)) for x, y in i["deny"]], B(i["t"]), B(i["a"]))
    if k == "parse_rule_array":
        rules = parse_rule_array(B(i["data"]), B(i["key"]), [])
        return [{"tool": L(a), "pattern": L(b)} for a, b in rules]
    if k == "is_path_safe":
        return is_path_safe(B(i["p"]))
    if k == "is_bash_allowed":
        return is_bash_allowed(B(i["c"]))
    if k == "redact":
        return L(redact_secrets(B(i["s"])))
    if k == "estimate_tokens":
        return estimate_tokens(i["n"])
    if k == "is_near_limit":
        return is_near_limit(i["n"])
    if k == "truncate":
        mod, out = truncate_old_tool_outputs(B(i["data"]), i["keep"])
        return {"mod": mod, "out": L(out)}
    if k == "summary":
        return L(apply_summary(B(i["data"]), B(i["summary"]), i["keep"]))
    if k == "compaction":
        r = None
        m = B(i["messages"])
        if is_near_limit(len(m), i["threshold"]):
            r = build_compaction_request(m, B(i["model"])) if i["threshold"] == CTX_THRESHOLD else None
            if i["threshold"] != CTX_THRESHOLD:
                body = b'{"model":"' + B(i["model"]) + b'","max_tokens":2048,"messages":[{"role":"user","content":"' + SUMMARY_PROMPT
                body += json_escape(m[:min(len(m), EXCERPT_MAX)]) + b'"}]}'
                r = body
        return None if r is None else L(r)
    raise ValueError(k)


def vectors_with_model() -> list:
    out = []
    for v in build_vectors():
        w = dict(v)
        w["model"] = model_result(v)
        out.append(w)
    return out


# ===========================================================================================
# ZIG: the pinned files, compiled and run. Generated `test` blocks are APPENDED to a copy of each
# file so private functions are reachable; the real source above them is byte-for-byte the pin.
# ===========================================================================================
ZIG_MODULES = {
    "tool_protocol": ["extract_field", "parse_response", "json_escape", "json_unescape", "extract_object"],
    "permissions": ["parse_rule_string", "rule_matches", "check", "parse_rule_array"],
    "tool_executor": ["is_path_safe", "is_bash_allowed", "redact"],
    "context": ["estimate_tokens", "is_near_limit", "truncate", "summary", "compaction"],
}
ZIG_HELPERS = r"""

// ===== GENERATED FIXTURE BLOCK (tools/trinity_tri_api.py) -- everything above is the pinned source =====
fn fxJ(s: []const u8) void {
    std.debug.print("\"", .{});
    var start: usize = 0;
    for (s, 0..) |c, i| {
        const safe = c >= 0x20 and c < 0x7f and c != '"' and c != '\\';
        if (!safe) {
            if (i > start) std.debug.print("{s}", .{s[start..i]});
            if (c == '"') std.debug.print("\\\"", .{}) else if (c == '\\') std.debug.print("\\\\", .{}) else std.debug.print("\\u{x:0>4}", .{c});
            start = i + 1;
        }
    }
    if (s.len > start) std.debug.print("{s}", .{s[start..]});
    std.debug.print("\"", .{});
}
fn fxB(b: bool) void {
    std.debug.print("{s}", .{if (b) "true" else "false"});
}
"""


def zlit(b: bytes) -> str:
    safe = set(b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 _-.:,/*=()[]{}")
    return '"' + "".join(chr(c) if c in safe else "\\x%02x" % c for c in b) + '"'


def zig_vector(v: dict) -> str:
    k, i, vid = v["kind"], v["in"], v["id"]
    head = '        std.debug.print("\\nVEC {{\\"id\\":\\"%s\\",\\"r\\":", .{});\n' % vid
    tail = '        std.debug.print("}}\\n", .{});\n'
    Z = lambda s: zlit(B(s))
    opt = "        if (r) |v| fxJ(v) else std.debug.print(\"null\", .{});\n"
    if k == "extract_field":
        body = "        const r = extractField(%s, %s);\n" % (Z(i["data"]), Z(i["key"])) + head + opt + tail
    elif k == "extract_object":
        body = "        const r = extractObject(%s, %d, %s);\n" % (Z(i["data"]), i["start"], Z(i["key"])) + head + opt + tail
    elif k == "json_unescape":
        body = ("        const r = try unescapeString(std.testing.allocator, %s);\n        defer std.testing.allocator.free(r);\n" % Z(i["s"])) + head + "        fxJ(r);\n" + tail
    elif k == "json_escape":
        body = ("        var buf: [8192]u8 = undefined;\n        var fbs = std.io.fixedBufferStream(&buf);\n        try writeJsonEscaped(fbs.writer(), %s);\n" % Z(i["s"])) + head + "        fxJ(fbs.getWritten());\n" + tail
    elif k == "parse_response":
        body = ("        var p = parseResponse(std.testing.allocator, %s);\n        defer p.deinit(std.testing.allocator);\n" % Z(i["body"])) + head
        body += ('        std.debug.print("{{\\"stop\\":", .{});\n        fxJ(p.stop_reason);\n'
                 '        std.debug.print(",\\"in\\":{d},\\"out\\":{d},\\"blocks\\":[", .{ p.input_tokens, p.output_tokens });\n'
                 "        for (p.blocks.items, 0..) |b, n| {\n            if (n > 0) std.debug.print(\",\", .{});\n            switch (b) {\n"
                 '                .text => |t| { std.debug.print("{{\\"t\\":\\"text\\",\\"v\\":", .{}); fxJ(t); std.debug.print("}}", .{}); },\n'
                 '                .tool_use => |u| { std.debug.print("{{\\"t\\":\\"tool_use\\",\\"id\\":", .{}); fxJ(u.id); std.debug.print(",\\"name\\":", .{}); fxJ(u.name); std.debug.print(",\\"input\\":", .{}); fxJ(u.input_json); std.debug.print("}}", .{}); },\n'
                 "            }\n        }\n"
                 '        std.debug.print("]}}", .{});\n') + tail
    elif k == "parse_rule_string":
        body = ("        const r = parseRuleString(%s);\n" % Z(i["s"])) + head
        body += ('        if (r) |x| { std.debug.print("{{\\"tool\\":", .{}); fxJ(x.tool); std.debug.print(",\\"pattern\\":", .{}); fxJ(x.pattern); std.debug.print("}}", .{}); } else std.debug.print("null", .{});\n') + tail
    elif k == "rule_matches":
        body = ("        const r = ruleMatches(Rule{ .tool = %s, .pattern = %s }, %s, %s);\n" % (Z(i["tool"]), Z(i["pattern"]), Z(i["t"]), Z(i["a"]))) + head + "        fxB(r);\n" + tail
    elif k == "check":
        body = "        var cfg = PermissionConfig{ .allow_rules = std.ArrayList(Rule).empty, .deny_rules = std.ArrayList(Rule).empty };\n        defer cfg.deinit(std.testing.allocator);\n"
        for x, y in i["allow"]:
            body += "        try cfg.allow_rules.append(std.testing.allocator, .{ .tool = %s, .pattern = %s });\n" % (Z(x), Z(y))
        for x, y in i["deny"]:
            body += "        try cfg.deny_rules.append(std.testing.allocator, .{ .tool = %s, .pattern = %s });\n" % (Z(x), Z(y))
        body += "        const r = cfg.check(%s, %s);\n" % (Z(i["t"]), Z(i["a"])) + head + '        std.debug.print("\\"{s}\\"", .{@tagName(r)});\n' + tail
    elif k == "parse_rule_array":
        body = ("        var rules = std.ArrayList(Rule).empty;\n        defer rules.deinit(std.testing.allocator);\n        parseRuleArray(std.testing.allocator, %s, %s, &rules);\n" % (Z(i["data"]), Z(i["key"]))) + head
        body += ('        std.debug.print("[", .{});\n        for (rules.items, 0..) |x, n| {\n            if (n > 0) std.debug.print(",", .{});\n'
                 '            std.debug.print("{{\\"tool\\":", .{}); fxJ(x.tool); std.debug.print(",\\"pattern\\":", .{}); fxJ(x.pattern); std.debug.print("}}", .{});\n        }\n'
                 '        std.debug.print("]", .{});\n') + tail
    elif k == "is_path_safe":
        body = ("        const r = ToolExecutor.isPathSafe(%s);\n" % Z(i["p"])) + head + "        fxB(r);\n" + tail
    elif k == "is_bash_allowed":
        body = ("        const r = ToolExecutor.isBashAllowed(%s);\n" % Z(i["c"])) + head + "        fxB(r);\n" + tail
    elif k == "redact":
        body = ("        var buf: [256]u8 = undefined;\n        const r = ToolExecutor.redactSecrets(&buf, %s);\n" % Z(i["s"])) + head + "        fxJ(r);\n" + tail
    elif k == "estimate_tokens":
        body = ("        const buf = try std.testing.allocator.alloc(u8, %d);\n        defer std.testing.allocator.free(buf);\n        @memset(buf, 'a');\n        const r = estimateTokens(buf);\n" % i["n"]) + head + '        std.debug.print("{d}", .{r});\n' + tail
    elif k == "is_near_limit":
        body = ("        var ctx = ContextManager.init(std.testing.allocator);\n        var m = std.ArrayList(u8).empty;\n        defer m.deinit(std.testing.allocator);\n        try m.appendNTimes(std.testing.allocator, 'a', %d);\n        const r = ctx.isNearLimit(&m);\n" % i["n"]) + head + "        fxB(r);\n" + tail
    elif k in ("truncate", "summary"):
        body = ("        var ctx = ContextManager.init(std.testing.allocator);\n        ctx.config.keep_turns = %d;\n        var m = std.ArrayList(u8).empty;\n        defer m.deinit(std.testing.allocator);\n        try m.appendSlice(std.testing.allocator, %s);\n" % (i["keep"], Z(i["data"])))
        if k == "truncate":
            body += "        const modified = ctx.truncateOldToolOutputs(&m);\n" + head + '        std.debug.print("{{\\"mod\\":", .{});\n        fxB(modified);\n        std.debug.print(",\\"out\\":", .{});\n        fxJ(m.items);\n        std.debug.print("}}", .{});\n' + tail
        else:
            body += "        ctx.applySummary(&m, %s);\n" % Z(i["summary"]) + head + "        fxJ(m.items);\n" + tail
    elif k == "compaction":
        body = ("        var ctx = ContextManager.init(std.testing.allocator);\n        ctx.config.compact_threshold = %d;\n        var m = std.ArrayList(u8).empty;\n        defer m.deinit(std.testing.allocator);\n        try m.appendSlice(std.testing.allocator, %s);\n        const r = ctx.buildCompactionRequest(&m, %s);\n        defer if (r) |x| std.testing.allocator.free(x);\n" % (i["threshold"], Z(i["messages"]), Z(i["model"]))) + head + opt + tail
    else:
        raise ValueError(k)
    return "    {\n" + body + "    }\n"


UAF_SETTINGS = '{"permissions":{"allow":["bash(echo *)"],"deny":["read_file(.env)"]}}'
UAF_BLOCK = r"""
test "FIXTURE permissions loadFromFile" {
    const allocator = std.testing.allocator;
    var tmp = std.testing.tmpDir(.{});
    defer tmp.cleanup();
    try tmp.dir.writeFile(.{ .sub_path = "settings.json", .data = %s });
    const path = try tmp.dir.realpathAlloc(allocator, "settings.json");
    defer allocator.free(path);
    var config = PermissionConfig{ .allow_rules = std.ArrayList(Rule).empty, .deny_rules = std.ArrayList(Rule).empty };
    defer config.deinit(allocator);
    loadFromFile(allocator, &config, path);
    std.debug.print("\nVEC {{\"id\":\"uaf-loadfromfile\",\"r\":{{\"allow\":{d},\"deny\":{d},\"allow0_tool\":[", .{ config.allow_rules.items.len, config.deny_rules.items.len });
    if (config.allow_rules.items.len > 0) {
        for (config.allow_rules.items[0].tool, 0..) |c, n| {
            if (n > 0) std.debug.print(",", .{});
            std.debug.print("{d}", .{c});
        }
    }
    std.debug.print("],\"check_allow_bash_echo\":\"{s}\",\"check_deny_read_env\":\"{s}\"}}}}\n", .{ @tagName(config.check("bash", "echo hello")), @tagName(config.check("read_file", ".env")) });
}
"""


def zig_fixture_source(pinned: str, module: str, vectors: list) -> str:
    kinds = ZIG_MODULES[module]
    out = pinned + ZIG_HELPERS
    out += 'test "FIXTURE %s vectors" {\n' % module
    for v in vectors:
        if v["kind"] in kinds:
            out += zig_vector(v)
    out += "}\n"
    if module == "permissions":
        out += UAF_BLOCK % ('"' + UAF_SETTINGS.replace('"', '\\"') + '"')
    return out


VEC_LINE = re.compile(r"^VEC (\{.*\})$", re.M)


def zig_tests_line(text: str) -> str:
    m = re.search(r"All (\d+) tests passed", text)
    if m:
        return m.group(0)
    m = re.search(r"(\d+) passed; (\d+) skipped; (\d+) failed", text)
    return m.group(0) if m else "no result line"


def run_zig_module(zig, sysroot, srcdir: pathlib.Path, module: str, mode: str, vectors: list, timeout=1200) -> dict:
    pinned = (srcdir / f"{module}.zig").read_text(encoding="utf-8")
    work = srcdir.parent / f"work-{module}-{mode}"
    if work.exists():
        shutil.rmtree(work)
    work.mkdir(parents=True)
    for f in srcdir.glob("*.zig"):
        shutil.copy(f, work / f.name)
    (work / f"{module}.zig").write_text(zig_fixture_source(pinned, module, vectors), encoding="utf-8")
    cmd = [zig, "test"] + (["--sysroot", sysroot] if sysroot else []) + ["-O", mode, f"{module}.zig"]
    rc, out, err = run(cmd, cwd=work, timeout=timeout)
    text = out + err
    got = {}
    for m in VEC_LINE.finditer(text):
        try:
            d = json.loads(m.group(1))
            got[d["id"]] = d["r"]
        except (ValueError, KeyError):
            pass
    mine = [v for v in vectors if v["kind"] in ZIG_MODULES[module]]
    agree, disagree, missing = 0, [], []
    for v in mine:
        if v["id"] not in got:
            missing.append(v["id"])
        elif got[v["id"]] == v["model"]:
            agree += 1
        else:
            disagree.append({"id": v["id"], "model": v["model"], "zig": got[v["id"]]})
    return {"rc": rc, "tests": zig_tests_line(text), "vectors": len(mine), "agree": agree, "disagree": disagree, "missing": missing,
            "uaf": got.get("uaf-loadfromfile"), "tail": "" if rc == 0 else text[-600:]}


# ===========================================================================================
# BINARY: the real tri-api, built from the pinned main.zig, driven by a scripted Messages server
# and a scripted MCP stdio server. Real process, real files, real exit codes, fake key.
# ===========================================================================================
class FakeProvider:
    """Scripted Anthropic Messages endpoint: pops one (status, body, headers) per POST, records every request."""

    def __init__(self, replies):
        self.replies = list(replies)
        self.requests = []
        self.lock = threading.Lock()
        outer = self

        class H(BaseHTTPRequestHandler):
            def log_message(self, *a):
                pass

            def do_POST(self):
                n = int(self.headers.get("Content-Length", 0))
                body = self.rfile.read(n)
                with outer.lock:
                    outer.requests.append({"path": self.path, "headers": {k.lower(): v for k, v in self.headers.items()},
                                           "body": body.decode("utf-8", "replace")})
                    rep = outer.replies.pop(0) if outer.replies else (500, '{"type":"error","error":{"type":"script_exhausted"}}', {})
                status, rbody, hdrs = rep
                hdrs = dict(hdrs or {})
                delay = float(hdrs.pop("_delay", 0))
                if delay:
                    time.sleep(delay)
                data = rbody.encode("utf-8") if isinstance(rbody, str) else rbody
                try:
                    self.send_response(status)
                    self.send_header("Content-Type", "application/json")
                    self.send_header("Content-Length", str(len(data)))
                    for k, v in hdrs.items():
                        self.send_header(k, v)
                    self.end_headers()
                    self.wfile.write(data)
                except (BrokenPipeError, ConnectionResetError):
                    pass

        self.srv = HTTPServer(("127.0.0.1", 0), H)
        self.port = self.srv.server_address[1]
        threading.Thread(target=self.srv.serve_forever, daemon=True).start()

    def close(self):
        self.srv.shutdown()
        self.srv.server_close()


def text_reply(text, stop="end_turn", it=10, ot=5):
    return (200, json.dumps({"id": "msg_1", "type": "message", "role": "assistant", "content": [{"type": "text", "text": text}],
                             "stop_reason": stop, "usage": {"input_tokens": it, "output_tokens": ot}}, separators=(",", ":")), {})


def tool_reply(blocks, stop="tool_use", it=20, ot=8):
    return (200, json.dumps({"id": "msg_2", "type": "message", "role": "assistant", "content": blocks, "stop_reason": stop,
                             "usage": {"input_tokens": it, "output_tokens": ot}}, separators=(",", ":")), {})


def tu(i, n, inp):
    return {"type": "tool_use", "id": i, "name": n, "input": inp}


FAKE_MCP_SRC = r'''#!/usr/bin/env python3
# Scripted MCP stdio server, newline-delimited JSON-RPC. FAKE_MCP_MODE: ok | noinit (never answers initialize).
import sys, json, os
MODE = os.environ.get("FAKE_MCP_MODE", "ok")
def out(o):
    print(json.dumps(o, separators=(",", ":")), flush=True)
for line in sys.stdin:
    try:
        m = json.loads(line)
    except Exception:
        continue
    mid, meth = m.get("id"), m.get("method")
    if os.environ.get("FAKE_MCP_LOG"):
        open(os.environ["FAKE_MCP_LOG"], "a").write(str(meth) + "\n")
    if meth == "initialize":
        if MODE == "noinit":
            continue
        out({"jsonrpc": "2.0", "id": mid, "result": {"protocolVersion": "2024-11-05", "capabilities": {"tools": {}}, "serverInfo": {"name": "fake", "version": "0"}}})
    elif meth == "tools/list":
        out({"jsonrpc": "2.0", "id": mid, "result": {"tools": [{"name": "echo", "description": "echo back", "inputSchema": {"type": "object", "properties": {"text": {"type": "string"}}}}]}})
    elif meth == "tools/call":
        a = m["params"].get("arguments", {})
        out({"jsonrpc": "2.0", "id": mid, "result": {"content": [{"type": "text", "text": "echoed:" + str(a.get("text"))}]}})
'''


def fetch_pinned(trinity: pathlib.Path, rev: str, dest: pathlib.Path) -> None:
    (dest / "tri-api").mkdir(parents=True, exist_ok=True)
    (dest / "tri").mkdir(parents=True, exist_ok=True)
    for f in SRC_FILES:
        (dest / "tri-api" / f"{f}.zig").write_bytes(git_show(trinity, rev, f"src/tri-api/{f}.zig"))
    (dest / "tri" / "token_rotator.zig").write_bytes(git_show(trinity, rev, ROTATOR_FILE))


def build_binary(zig, sysroot, pinned: pathlib.Path, mode: str) -> pathlib.Path:
    out = pinned / f"tri-api-{mode}"
    cmd = [zig, "build-exe"] + (["--sysroot", sysroot] if sysroot else []) + [
        "-O", mode, "--dep", "token_rotator", "-Mroot=main.zig", "-Mtoken_rotator=../tri/token_rotator.zig", f"-femit-bin={out}"]
    rc, o, e = run(cmd, cwd=pinned / "tri-api", timeout=1800)
    if rc != 0 or not out.exists():
        raise RuntimeError(f"build {mode} failed rc={rc}: {(o + e)[-800:]}")
    return out


class Drive:
    """One driven run of the binary in a fresh HOME and a fresh working directory."""

    def __init__(self, binary, mcp_script):
        self.binary, self.mcp_script = binary, mcp_script

    def __call__(self, args, replies, files=None, settings=None, user_settings=None, git=None, sub=None, extra_env=None,
                 timeout=60, sigterm_after=None, sample_rss=None, pre=None):
        home = pathlib.Path(tempfile.mkdtemp(prefix="s07h-"))
        cwd = pathlib.Path(tempfile.mkdtemp(prefix="s07c-"))
        for k, v in (files or {}).items():
            p = cwd / k
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes(v if isinstance(v, bytes) else v.encode())
        if git:
            subprocess.run(git, shell=True, cwd=cwd, check=True, capture_output=True)
        work = cwd / sub if sub else cwd
        work.mkdir(parents=True, exist_ok=True)
        if settings is not None:
            (work / ".trinity/api").mkdir(parents=True, exist_ok=True)
            (work / ".trinity/api/settings.json").write_text(settings)
        if user_settings is not None:
            (home / ".trinity/api").mkdir(parents=True, exist_ok=True)
            (home / ".trinity/api/settings.json").write_text(user_settings)
        if pre:
            pre(home, cwd)
        fp = FakeProvider(replies)
        env = {"PATH": os.environ["PATH"], "HOME": str(home), "ANTHROPIC_API_KEY": "fake-key-not-real",
               "ANTHROPIC_BASE_URL": f"http://127.0.0.1:{fp.port}"}
        env.update(extra_env or {})
        t0 = time.time()
        rss = None
        pr = subprocess.Popen([str(self.binary)] + args, cwd=work, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            if sigterm_after is not None:
                time.sleep(sigterm_after)
                pr.send_signal(signal.SIGTERM)
            elif sample_rss is not None:
                time.sleep(sample_rss)
                ps = subprocess.run(["ps", "-o", "rss=", "-p", str(pr.pid)], capture_output=True, text=True).stdout.strip()
                rss = int(ps) if ps.isdigit() else None
            out, err = pr.communicate(timeout=timeout)
            rc = pr.returncode
        except subprocess.TimeoutExpired:
            pr.kill()
            out, err = pr.communicate()
            rc = "timeout"
        fp.close()
        return {"rc": rc, "out": out.decode("utf-8", "replace"), "err": err.decode("utf-8", "replace"), "reqs": fp.requests,
                "cwd": cwd, "work": work, "home": home, "secs": time.time() - t0, "rss_kb": rss}


def msgs_of(req):
    return json.loads(req["body"])["messages"]


def tool_results(req):
    """The tool_result blocks the model was shown in this request: (id, content[:70], is_error), in order."""
    out = []
    for m in msgs_of(req):
        if m["role"] == "user" and isinstance(m["content"], list):
            for b in m["content"]:
                if b.get("type") == "tool_result":
                    out.append([b["tool_use_id"], b["content"][:70], bool(b.get("is_error", False))])
    return out


def marks(err: str) -> list:
    """stderr `[tri-api]` lines minus the ones that vary by run (byte counts, session id, rotator, model)."""
    keep = []
    for l in err.splitlines():
        if not l.startswith("[tri-api] "):
            continue
        b = l[10:]
        if b.startswith(("Token rotator", "model=", "Session saved")):
            continue
        b = re.sub(r"^turn (\d+): sending \d+ bytes\.\.\.$", r"turn \1", b)
        keep.append(b)
    return keep


def nmarks(err: str, prefix: str) -> int:
    return len([m for m in marks(err) if m.startswith(prefix)])


SCENARIOS = []


def scenario(sid, finding, modes=("Debug",)):
    def deco(fn):
        SCENARIOS.append({"id": sid, "finding": finding, "modes": modes, "fn": fn})
        return fn
    return deco


ALLOW_ALL = '{"permissions":{"allow":["write_file(*)","bash(*)"]}}'
PRETTY_SETTINGS = '{\n  "permissions": {\n    "allow": [\n      "bash(echo *)"\n    ],\n    "deny": [\n      "bash(echo secret*)"\n    ]\n  }\n}'


def first_result(r, i=1):
    tr = tool_results(r["reqs"][i]) if len(r["reqs"]) > i else []
    return tr[-1][1:] if tr else None


@scenario("e01-read-cycle", "cycle-f23")
def e01(d):
    r = d(["read the file"], [tool_reply([{"type": "text", "text": "I will read it."}, tu("toolu_1", "read_file", {"path": "a.txt"})]),
                              text_reply("It says hello.")], files={"a.txt": "hello\n"})
    b0 = json.loads(r["reqs"][0]["body"])
    return {"rc": r["rc"], "requests": len(r["reqs"]), "stdout": r["out"], "path": r["reqs"][0]["path"],
            "x_api_key": r["reqs"][0]["headers"].get("x-api-key"), "anthropic_version": r["reqs"][0]["headers"].get("anthropic-version"),
            "tools": [t["name"] for t in b0["tools"]], "max_tokens": b0["max_tokens"], "first_roles": [m["role"] for m in b0["messages"]],
            "tool_results": tool_results(r["reqs"][1]), "marks": marks(r["err"])}


@scenario("e02-two-tool-use", "f07-consecutive-user-messages")
def e02(d):
    r = d(["two tools"], [tool_reply([tu("toolu_a", "read_file", {"path": "a.txt"}), tu("toolu_b", "read_file", {"path": "b.txt"})]),
                          text_reply("done")], files={"a.txt": "A\n", "b.txt": "B\n"})
    return {"roles": [m["role"] for m in msgs_of(r["reqs"][1])], "tool_results": tool_results(r["reqs"][1])}


@scenario("e03-write-default-deny", "deny-by-default")
def e03(d):
    r = d(["w"], [tool_reply([tu("t1", "write_file", {"path": "new.txt", "content": "x"})]), text_reply("ok")])
    return {"result": first_result(r), "file_exists": (r["cwd"] / "new.txt").exists(), "marks": marks(r["err"])}


@scenario("e04-path-boundary", "f16-path-boundary")
def e04(d):
    obs = {}
    for name, p in (("dotdot", "../secret.txt"), ("absolute", "/etc/hosts"), ("dots-in-name", "a..b.txt")):
        r = d(["r"], [tool_reply([tu("t1", "read_file", {"path": p})]), text_reply("ok")], files={"a..b.txt": "ok"})
        res = first_result(r)
        obs[name] = {"content_head": res[0][:24] if res else None, "is_error": res[1] if res else None}
    return obs


@scenario("e05-error-replies", "f03-silent-failure")
def e05(d):
    obs = {}
    cases = {"garbage-200": (200, "this is not json at all", {}), "http-400": (400, '{"type":"error","error":{"type":"x","message":"boom"}}', {}),
             "http-429": (429, '{"type":"error","error":{"type":"x","message":"boom"}}', {}), "http-500": (500, '{"type":"error","error":{"type":"x","message":"boom"}}', {})}
    for name, rep in cases.items():
        r = d(["m"], [rep, text_reply("second")])
        obs[name] = {"rc": r["rc"], "requests": len(r["reqs"]), "stdout": r["out"], "marks": marks(r["err"])}
    return obs


@scenario("e06-turn-limit", "f04-turn-limit")
def e06(d):
    r = d(["loop"], [tool_reply([tu("t%d" % i, "read_file", {"path": "a.txt"})]) for i in range(30)], files={"a.txt": "a"})
    return {"rc": r["rc"], "requests": len(r["reqs"]), "stdout": r["out"], "turn_marks": nmarks(r["err"], "turn "),
            "last_mark": marks(r["err"])[-1]}


@scenario("e07-stop-reasons", "f02-parse")
def e07(d):
    pretty = json.dumps({"id": "m", "type": "message", "role": "assistant", "content": [{"type": "text", "text": "hi pretty"}],
                         "stop_reason": "end_turn", "usage": {"input_tokens": 3, "output_tokens": 2}}, indent=1)
    spaced = '{"id":"m","type":"message","role":"assistant","content":[{"type":"text","text":"hi"}],"stop_reason": "tool_use","usage":{"input_tokens":1,"output_tokens":1}}'
    obs = {}
    for name, reps in (("pretty", [(200, pretty, {})]), ("space-after-colon", [(200, spaced, {}), text_reply("second")]),
                       ("max_tokens", [text_reply("cut off mid", stop="max_tokens"), text_reply("never")])):
        r = d(["m"], reps)
        obs[name] = {"requests": len(r["reqs"]), "stdout": r["out"], "done": [m for m in marks(r["err"]) if m.startswith("done")]}
    return obs


@scenario("e08-usage", "f01-usage")
def e08(d):
    r = d(["t"], [tool_reply([{"type": "text", "text": "x"}], stop="end_turn", it=1234, ot=56)])
    return {"tokens_line": [m for m in marks(r["err"]) if "tokens" in m]}


@scenario("e09-settings-shapes", "f11-f12-permission-rules", modes=("Debug", "ReleaseFast"))
def e09(d):
    obs = {}
    compact = '{"permissions":{"allow":["bash(echo *)"],"deny":["bash(echo secret*)"]}}'
    mixed = '{"permissions":{"allow":["bash(echo *)"],\n "deny": ["bash(echo secret*)"]}}'
    for name, s in (("compact", compact), ("pretty", PRETTY_SETTINGS), ("allow-compact-deny-pretty", mixed)):
        for c in ("echo ok", "echo secret-token"):
            r = d(["b"], [tool_reply([tu("t1", "bash", {"command": c})]), text_reply("ok")], settings=s)
            obs[f"{name}|{c}"] = {"loaded": [m for m in marks(r["err"]) if m.startswith("permissions:")], "result": first_result(r)}
    r = d(["r"], [tool_reply([tu("t1", "read_file", {"path": ".env"})]), text_reply("ok")],
          settings='{"permissions":{"deny":["read_file(.env)"]}}', files={".env": "SECRET=1\n"})
    obs["deny-read-env"] = {"loaded": [m for m in marks(r["err"]) if m.startswith("permissions:")], "result": first_result(r)}
    return obs


@scenario("e10-settings-precedence", "f13-project-vs-user", modes=("ReleaseFast",))
def e10(d):
    r = d(["b"], [tool_reply([tu("t1", "bash", {"command": "echo hi"})]), text_reply("ok")],
          settings='{"permissions":{"allow":["bash(echo *)"]}}', user_settings='{"permissions":{"deny":["bash(echo *)"]}}')
    r2 = d(["b"], [tool_reply([tu("t1", "bash", {"command": "echo hi"})]), text_reply("ok")],
           settings='{"permissions":{"deny":["bash(echo *)"]}}', user_settings='{"permissions":{"allow":["bash(echo *)"]}}')
    return {"project-allow-user-deny": {"loaded": [m for m in marks(r["err"]) if m.startswith("permissions:")], "result": first_result(r)},
            "project-deny-user-allow": {"loaded": [m for m in marks(r2["err"]) if m.startswith("permissions:")], "result": first_result(r2)}}


@scenario("e11-rule-cap", "f14-f25-rule-cap", modes=("ReleaseFast",))
def e11(d):
    rules = ",".join('"bash(echo n%d*)"' % i for i in range(70))
    r = d(["b"], [text_reply("ok")], settings='{"permissions":{"allow":[%s]}}' % rules)
    # the first tool call afterwards died on a signal in this build, twice out of two (a dangling rule slice, finding 11);
    # that is a consequence of a use-after-free, so it is recorded but not asserted (keys starting with "~")
    r2 = d(["b"], [tool_reply([tu("t1", "bash", {"command": "echo n3x"})]), text_reply("ok")], settings='{"permissions":{"allow":[%s]}}' % rules)
    return {"written": 70, "loaded": [m for m in marks(r["err"]) if m.startswith("permissions:")], "~first_call_rc": r2["rc"]}


BASH_CMDS = ["ls & echo BG > bg.txt", "echo RED > red.txt", "cat a.txt > copy.txt", "env touch via_env.txt",
             "find . -name a.txt -delete", "echo hi; echo two", "ls | wc -l", "echo $(id)", "rm -rf nothing",
             "sed -i.bak s/a/b/ a.txt", "git status", "pwdx", "datex", "curl x", "echo a && echo b", "echo a || echo b", "ls ${HOME}", 'echo "q"']


@scenario("e12-bash-allowlist", "f15-bash-allowlist", modes=("ReleaseFast",))
def e12(d):
    obs = {}
    for c in BASH_CMDS:
        r = d(["b"], [tool_reply([tu("t1", "bash", {"command": c})]), text_reply("ok")], settings='{"permissions":{"allow":["bash(*)"]}}',
              files={"a.txt": "aaa\n"})
        res = first_result(r)
        made = sorted(p.name for p in r["cwd"].iterdir() if p.name not in (".trinity",))
        obs[c] = {"result_head": res[0][:40] if res else None, "files": made}
    return obs


@scenario("e13-checkpoint-order", "f17-checkpoint-before-boundary", modes=("ReleaseFast",))
def e13(d):
    git = "git init -q . && git config user.email a@b && git config user.name n && echo v1 > top.txt && mkdir sub && echo x > sub/keep && git add -A && git commit -q -m init && echo v2-local-edit > top.txt"
    obs = {}
    git2 = git + " && echo sub-edit > sub/keep"
    for name, p, g in (("unsafe-../top.txt", "../top.txt", git), ("safe-keep", "keep", git), ("safe-keep-with-local-edit", "keep", git2)):
        r = d(["w"], [tool_reply([tu("t1", "write_file", {"path": p, "content": "NEW"})]), text_reply("ok")], settings=ALLOW_ALL, git=g, sub="sub")
        top = (r["cwd"] / "top.txt").read_text()
        keep = (r["cwd"] / "sub/keep").read_text()
        stash = subprocess.run("git stash list", shell=True, cwd=r["cwd"], capture_output=True, text=True).stdout.strip()
        obs[name] = {"result": first_result(r), "top_txt": top, "keep_txt": keep, "stash_entries": len(stash.splitlines())}
    return obs


@scenario("e14-input-handling", "f09-f10-f20-f21")
def e14(d):
    obs = {}
    r = d(["n"], [tool_reply([tu("t1", "read_file", {"path": "a.txt", "meta": {"type": "text", "text": "INJECTED-BY-INPUT"}})]), text_reply("fin")], files={"a.txt": "a"})
    obs["nested-type-text"] = {"stdout": r["out"]}
    long_text = "".join(chr(65 + (i % 26)) for i in range(20000))
    r = d(["t"], [text_reply(long_text)])
    sd = r["home"] / ".trinity/api/sessions"
    raw = "".join((sd / f).read_text() for f in os.listdir(sd) if f != "index.json") if sd.is_dir() else ""
    obs["long-text"] = {"printed": len(r["out"].rstrip("\n")), "of": len(long_text), "history_keeps_full": long_text in raw}
    r = d(["t"], [tool_reply([tu("t1", "read_file", {"path": 'we"ird.txt'})]), text_reply("ok")], files={'we"ird.txt': "Q"})
    obs["escaped-quote-path"] = {"result": first_result(r)}
    r = d(["t"], [tool_reply([tu("t1", "read_file", {"path": "big.bin"})]), text_reply("ok")], files={"big.bin": "x" * 600000})
    obs["file-over-512k"] = {"result": first_result(r)}
    return obs


@scenario("e15-duplicate-id", "f08-duplicate-tool-use", modes=("ReleaseFast",))
def e15(d):
    r = d(["d"], [tool_reply([tu("same", "bash", {"command": "echo once >> log.txt"}), tu("same", "bash", {"command": "echo once >> log.txt"})]),
                  text_reply("ok")], settings=ALLOW_ALL)
    log = (r["cwd"] / "log.txt")
    return {"log_lines": len(log.read_text().splitlines()) if log.exists() else 0, "tool_results": tool_results(r["reqs"][1])}


@scenario("e16-unknown-tool", "unknown-tool")
def e16(d):
    r = d(["u"], [tool_reply([tu("t1", "nope", {})]), text_reply("ok")])
    return {"result": first_result(r), "marks": marks(r["err"])}


@scenario("e17-mcp", "f18-f24-mcp", modes=("Debug", "ReleaseFast"))
def e17(d):
    obs = {}
    ms = lambda perm: json.dumps({"mcp_servers": {"fake": {"command": ["python3", d.mcp_script]}}, "permissions": perm}, separators=(",", ":"))
    for mode in ("ok", "noinit"):
        for name, perm in (("no-rule", {}), ("allow-star", {"allow": ["fake.echo(*)"]}), ("allow-and-deny", {"allow": ["fake.echo(*)"], "deny": ["fake.echo(*)"]}),
                           ("allow-arg-pattern", {"allow": ["fake.echo(hello*)"]})):
            log = tempfile.mktemp(prefix="mcplog-")
            r = d(["m"], [tool_reply([tu("t1", "fake.echo", {"text": "hello"})]), text_reply("ok")], settings=ms(perm), timeout=20,
                  extra_env={"FAKE_MCP_MODE": mode, "FAKE_MCP_LOG": log})
            body0 = json.loads(r["reqs"][0]["body"]) if r["reqs"] else None
            tools = [t["name"] for t in body0["tools"]] if body0 else None
            obs[f"{mode}|{name}"] = {"rc": r["rc"], "requests": len(r["reqs"]), "mcp_mark": [m for m in marks(r["err"]) if m.startswith("MCP:")],
                                     "advertised_fake": tools.count("fake.echo") if tools is not None else None, "result": first_result(r),
                                     "server_saw": open(log).read().split() if os.path.exists(log) else []}
    return obs


@scenario("e18-sigterm", "f06-no-cancel")
def e18(d):
    r = d(["slow"], [(200, text_reply("late")[1], {"_delay": 6})], sigterm_after=2, timeout=30)
    sd = r["home"] / ".trinity/api/sessions"
    return {"rc": r["rc"], "stdout": r["out"], "session_files": sorted(os.listdir(sd)) if sd.is_dir() else []}


@scenario("e19-slow-provider", "f05-f06-no-timeout-no-retry")
def e19(d):
    r = d(["slow"], [(200, text_reply("late")[1], {"_delay": 4})], timeout=30)
    return {"rc": r["rc"], "stdout": r["out"], "requests": len(r["reqs"]), "waited_over_3s": r["secs"] > 3.5}


@scenario("e20-compaction", "f19-f29-compaction")
def e20(d):
    r = d(["A" * 600000], [text_reply("SUMMARY-TEXT-FROM-FAKE"), text_reply("final answer")], timeout=90)
    obs = {"requests": len(r["reqs"]), "stdout": r["out"]}
    for i, q in enumerate(r["reqs"]):
        b = json.loads(q["body"])
        obs[f"req{i}"] = {"has_tools": "tools" in b, "roles": [m["role"] for m in b["messages"]],
                          "first_content_head": (b["messages"][0]["content"] if isinstance(b["messages"][0]["content"], str) else "blocks")[:40],
                          "first_content_bytes": len(b["messages"][0]["content"]) if isinstance(b["messages"][0]["content"], str) else None}
    obs["original_prompt_in_second_request"] = "A" * 1000 in r["reqs"][1]["body"] if len(r["reqs"]) > 1 else None
    obs["compact_marks"] = [m for m in marks(r["err"]) if "ompact" in m]
    return obs


@scenario("e21-hang-duplicate-type", "f22-parse-hang")
def e21(d):
    body = '{"id":"m","type":"message","role":"assistant","content":[{"type":"tool_use","id":"a","name":"read_file","type":"x"}],"stop_reason":"tool_use"}'
    r = d(["h"], [(200, body, {}), text_reply("never")], timeout=3, sample_rss=2)
    return {"rc": r["rc"], "requests": len(r["reqs"]), "~rss_kb_at_2s": r["rss_kb"]}


def run_e2e(trinity, rev, zig, sysroot, only=None) -> dict:
    """Build the pinned tri-api in Debug and ReleaseFast, drive every scenario, return {scenario: {mode: observed}}."""
    work = pathlib.Path(tempfile.mkdtemp(prefix="s07-e2e-"))
    fetch_pinned(trinity, rev, work)
    mcp = work / "fake_mcp.py"
    mcp.write_text(FAKE_MCP_SRC)
    bins = {}
    for mode in ("Debug", "ReleaseFast"):
        bins[mode] = build_binary(zig, sysroot, work, mode)
    out = {}
    for sc in SCENARIOS:
        if only and sc["id"] not in only:
            continue
        out[sc["id"]] = {"finding": sc["finding"], "modes": {}}
        for mode in sc["modes"]:
            out[sc["id"]]["modes"][mode] = sc["fn"](Drive(bins[mode], str(mcp)))
    return out


@scenario("e22-end-turn-with-tool-use", "f26-end-turn-runs-tools", modes=("ReleaseFast",))
def e22(d):
    r = d(["t"], [tool_reply([tu("t1", "bash", {"command": "echo ran >> log.txt"})], stop="end_turn"), text_reply("never")], settings=ALLOW_ALL)
    log = r["cwd"] / "log.txt"
    return {"requests": len(r["reqs"]), "tool_ran": log.exists(), "done": [m for m in marks(r["err"]) if m.startswith("done")]}


@scenario("e23-mcp-config-paths", "f27-mcp-config-path", modes=("ReleaseFast",))
def e23(d):
    obs = {}
    cfg = lambda: json.dumps({"mcp_servers": {"fake": {"command": ["python3", d.mcp_script]}}}, separators=(",", ":"))
    for name, kw, rel in (("home-.trinity/api", "user_settings", ".trinity/api"), ("home-.tri-api", None, ".tri-api"), ("project-.trinity/api", "settings", None)):
        log = tempfile.mktemp(prefix="mcplog-")
        pre = None
        if name == "home-.tri-api":
            def put(home, cwd, rel=rel):
                (home / rel).mkdir(parents=True, exist_ok=True)
                (home / rel / "settings.json").write_text(cfg())
            kwargs = {"pre": put}
        elif kw == "user_settings":
            kwargs = {"user_settings": cfg()}
        else:
            kwargs = {"settings": cfg()}
        r = d(["m"], [text_reply("ok")], extra_env={"FAKE_MCP_LOG": log}, timeout=20, **kwargs)
        obs[name] = {"server_spawned": os.path.exists(log)}
    return obs


@scenario("e24-grep", "f28-grep-needs-timeout-binary")
def e24(d):
    r = d(["g"], [tool_reply([tu("t1", "grep", {"pattern": "needle", "path": "."})]), text_reply("ok")], files={"a.txt": "a needle here\n"})
    return {"has_timeout_binary": shutil.which("timeout") is not None, "result_head": (first_result(r) or [None])[0]}


# ===========================================================================================
# INVENTORY: what the pin is, byte for byte, and the constants the specs restate
# ===========================================================================================
def _num(s: str) -> int:
    return int(s.replace("_", ""))


def source_constants(blobs: dict) -> dict:
    """The numbers and strings the specs restate, read out of the pinned Zig by pattern."""
    m = blobs["main"].decode()
    c = blobs["context"].decode()
    p = blobs["permissions"].decode()
    t = blobs["tool_executor"].decode()
    out = {
        "max_turns": _num(re.search(r"^const max_turns = (\d+);", m, re.M).group(1)),
        "api_version": re.search(r'^const api_version = "([^"]+)";', m, re.M).group(1),
        "default_model": re.search(r'^const default_model = "([^"]+)";', m, re.M).group(1),
        "max_tokens_request": _num(re.search(r'\\"max_tokens\\":(\d+)', m).group(1)),
        "max_rules": _num(re.search(r"^const max_rules = (\d+);", p, re.M).group(1)),
        "compact_threshold": _num(re.search(r"compact_threshold: u32 = ([\d_]+)", c).group(1)),
        "keep_turns": _num(re.search(r"keep_turns: u32 = (\d+)", c).group(1)),
        "context_max_tokens": _num(re.search(r"max_tokens: u32 = ([\d_]+)", c).group(1)),
        "excerpt_max": _num(re.search(r"@min\(messages\.items\.len, ([\d_]+)\)", c).group(1)),
        "read_limit": _num(re.search(r"readToEndAlloc\(self\.allocator, (\d+) \* 1024\)", t).group(1)) * 1024,
        "text_window": _num(re.search(r"@min\(idx \+ (\d+), body\.len\)", blobs["tool_protocol"].decode()).group(1)),
    }
    arr = re.search(r"const allowed_bash_cmds = \[_\]\[\]const u8\{(.*?)\};", t, re.S).group(1)
    out["bash_prefix_count"] = len(re.findall(r'"[^"]*"', arr))
    out["shell_meta"] = "".join(chr(int(x, 16)) if x.startswith("x") else x for x in
                                [re.sub(r"^\\", "", y) for y in re.findall(r"'((?:\\x?[0-9a-f]+|[^'\\]))'", re.search(r"const shell_meta = \[_\]u8\{(.*?)\};", t, re.S).group(1))])
    return out


def build_inventory(trinity: pathlib.Path, rev: str) -> dict:
    blobs = {f: git_show(trinity, rev, f"src/tri-api/{f}.zig") for f in SRC_FILES}
    rot = git_show(trinity, rev, ROTATOR_FILE)
    files = {f"src/tri-api/{f}.zig": {"sha256": sha256(b), "bytes": len(b), "lines": b.count(b"\n")} for f, b in blobs.items()}
    files[ROTATOR_FILE] = {"sha256": sha256(rot), "bytes": len(rot), "lines": rot.count(b"\n")}
    rc = subprocess.run(["git", "-C", str(trinity), "diff", "--name-only", ISSUE_BASELINE, rev, "--", "src/tri-api"], capture_output=True, text=True)
    changed = None if rc.returncode != 0 else [x for x in rc.stdout.split("\n") if x]
    return {"repo": TRINITY_REPO, "pin": rev, "issue_baseline": ISSUE_BASELINE, "src_tri_api_changed_since_baseline": changed,
            "files": files, "constants": source_constants(blobs),
            "builtin_tools": ["read_file", "write_file", "bash", "grep"],
            "description": "The tri-api tree at the pin: per-file digests and the constants that specs/api/tri_api_*.t27 restate, read out of the source by pattern."}


# ===========================================================================================
# RECORDS
# ===========================================================================================
def now() -> str:
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def host() -> str:
    return f"{platform.system()} {platform.machine()}"


def zig_version(zig: str) -> str:
    rc, o, e = run([zig, "version"], timeout=60)
    return o.strip() or e.strip()


ZIG_MODES = ("Debug", "ReleaseSafe", "ReleaseFast", "ReleaseSmall")


def build_zig_record(trinity, rev, zig, sysroot, vectors) -> dict:
    work = pathlib.Path(tempfile.mkdtemp(prefix="s07-zig-"))
    fetch_pinned(trinity, rev, work)
    src = work / "tri-api"
    modes = {}
    for mode in ZIG_MODES:
        modes[mode] = {}
        for module in ZIG_MODULES:
            r = run_zig_module(zig, sysroot, src, module, mode, vectors)
            m = re.search(r"All (\d+) tests passed", r["tests"])
            r["upstream_tests_passed"] = int(m.group(1)) - 1 if m else None     # minus the one generated fixture test
            modes[mode][module] = r
    uaf = {mode: modes[mode]["permissions"].pop("uaf") for mode in ZIG_MODES}
    for mode in ZIG_MODES:
        for module in ZIG_MODULES:
            modes[mode][module].pop("uaf", None)
            modes[mode][module].pop("tail", None) if modes[mode][module].get("rc") == 0 else None
    return {"pin": rev, "zig": zig_version(zig), "host": host(), "at": now(), "modes": modes, "uaf_loadfromfile": uaf,
            "excluded_from_zig": [v["id"] for v in vectors if v["kind"] == "parse_response_hang"],
            "description": "The pinned source files compiled and run: their own unit tests plus generated vectors appended to copies of each file; each vector's Zig answer against the Python model's, in four optimize modes. The hang vector is not run in Zig (it never returns); the binary scenario e21 covers it."}


def build_e2e_record(trinity, rev, zig, sysroot) -> dict:
    res = run_e2e(trinity, rev, zig, sysroot)
    return {"pin": rev, "zig": zig_version(zig), "host": host(), "at": now(), "scenarios": res,
            "provider": "scripted fake Messages server on 127.0.0.1; key fake-key-not-real; no network, no credential",
            "description": "The real tri-api built from the pinned main.zig in Debug and ReleaseFast and driven through 24 scripted scenarios. Each observation is what the process did: exit code, stdout, the requests the fake provider saw, the files left behind. Keys starting with ~ are recorded and not asserted."}


# ===========================================================================================
# CLAIMS: what the e2e record must show for the specs to be true. One entry per claim.
# ===========================================================================================
def _o(E, sid, mode="Debug"):
    return E["scenarios"][sid]["modes"][mode]


def claims(E) -> list:
    """(claim id, finding, statement, lambda) over the e2e record. Expected values are written here, by hand."""
    tr = lambda r, i=0: r["tool_results"][i]
    C = []

    def add(cid, fnd, text, fn):
        C.append((cid, fnd, text, fn))

    add("c01", "cycle", "a read_file cycle: two requests, the text printed, the fake key sent as x-api-key to /v1/messages with version 2023-06-01, four tools, max_tokens 8192",
        lambda: (lambda o: o["rc"] == 0 and o["requests"] == 2 and o["stdout"] == "I will read it.\nIt says hello.\n" and o["path"] == "/v1/messages"
                 and o["x_api_key"] == "fake-key-not-real" and o["anthropic_version"] == "2023-06-01" and o["tools"] == ["read_file", "write_file", "bash", "grep"]
                 and o["max_tokens"] == 8192 and o["first_roles"] == ["user"] and o["tool_results"] == [["toolu_1", "hello\n", False]])(_o(E, "e01-read-cycle")))
    add("c02", "f07", "two tool_use blocks come back as two consecutive user messages",
        lambda: _o(E, "e02-two-tool-use")["roles"] == ["user", "assistant", "user", "user"] and len(_o(E, "e02-two-tool-use")["tool_results"]) == 2)
    add("c03", "deny-by-default", "write_file with no rule is denied, reaches no file, and the model is told so",
        lambda: (lambda o: o["file_exists"] is False and o["result"] == ["Permission denied: write_file(new.txt)", True] and "DENIED: write_file(new.txt)" in o["marks"])(_o(E, "e03-write-default-deny")))
    add("c04", "f16", "../x and a..b.txt are refused, an absolute path is read",
        lambda: (lambda o: o["dotdot"]["is_error"] and o["dotdot"]["content_head"].startswith("error: path traversal") and o["dots-in-name"]["is_error"]
                 and o["dots-in-name"]["content_head"].startswith("error: path traversal") and o["absolute"]["is_error"] is False)(_o(E, "e04-path-boundary")))

    def c05():
        o = _o(E, "e05-error-replies")
        ok = all(v["rc"] == 0 and v["requests"] == 1 and v["stdout"] == "" and "done: end_turn" in v["marks"] for v in o.values())
        ok = ok and all(f"API status: {n}" in o[f"http-{n}"]["marks"] for n in (400, 429, 500))
        return ok and not any(m.startswith("API status") for m in o["garbage-200"]["marks"])
    add("c05", "f03", "a 400, 429, 500 or a garbage 200: one request, empty stdout, done end_turn, exit 0, no retry", c05)
    add("c06", "f04", "a model that calls a tool every turn is stopped after 20 requests, silently, exit 0",
        lambda: (lambda o: o["rc"] == 0 and o["requests"] == 20 and o["stdout"] == "" and o["turn_marks"] == 20 and o["last_mark"] == "0 input + 0 output tokens")(_o(E, "e06-turn-limit")))
    add("c07", "f02", "a pretty-printed reply prints nothing; stop_reason with a space is end_turn; max_tokens is reported",
        lambda: (lambda o: o["pretty"]["stdout"] == "" and o["pretty"]["requests"] == 1 and o["space-after-colon"]["done"] == ["done: end_turn"]
                 and o["max_tokens"]["done"] == ["done: max_tokens"] and o["max_tokens"]["stdout"] == "cut off mid\n")(_o(E, "e07-stop-reasons")))
    add("c08", "f01", "the provider reports 1234 input and 56 output tokens and the loop counts 0 and 0",
        lambda: _o(E, "e08-usage")["tokens_line"] == ["0 input + 0 output tokens"])

    def c09():
        d, f = _o(E, "e09-settings-shapes", "Debug"), _o(E, "e09-settings-shapes", "ReleaseFast")
        ok = d["compact|echo ok"]["result"] == ["Permission denied: bash(echo ok)", True]          # Debug: an allow rule never allows
        ok = ok and d["deny-read-env"]["result"] == ["SECRET=1\n", False]                               # Debug: a deny rule never denies
        ok = ok and f["compact|echo ok"]["result"] == ["ok\n", False] and f["compact|echo secret-token"]["result"] == ["Permission denied: bash(echo secret-token)", True]
        ok = ok and f["deny-read-env"]["result"] == ["Permission denied: read_file(.env)", True]
        ok = ok and f["pretty|echo ok"]["loaded"] == ["permissions: 0 allow, 0 deny rules"]
        ok = ok and f["allow-compact-deny-pretty|echo secret-token"]["loaded"] == ["permissions: 1 allow, 0 deny rules"]
        return ok and f["allow-compact-deny-pretty|echo secret-token"]["result"] == ["secret-token\n", False]
    add("c09", "f11-f12", "Debug: allow never allows and deny read_file(.env) still reads .env; ReleaseFast: the rules work, a pretty file loads 0 rules, a mixed file drops its deny rule", c09)
    add("c10", "f13", "user and project settings are added, deny wins from either side",
        lambda: (lambda o: all(v["result"] == ["Permission denied: bash(echo hi)", True] and v["loaded"] == ["permissions: 1 allow, 1 deny rules"] for v in o.values()))(_o(E, "e10-settings-precedence", "ReleaseFast")))
    add("c11", "f14", "70 rules written, 64 loaded",
        lambda: (lambda o: o["written"] == 70 and o["loaded"] == ["permissions: 64 allow, 0 deny rules"])(_o(E, "e11-rule-cap", "ReleaseFast")))

    def c12():
        o = _o(E, "e12-bash-allowlist", "ReleaseFast")
        refused = ["curl x", "echo $(id)", "echo a && echo b", "echo a || echo b", "echo hi; echo two", "ls ${HOME}", "ls | wc -l"]
        ok = all(o[c]["result_head"] == "error: command not in allowed list" for c in refused)
        ok = ok and "bg.txt" in o["ls & echo BG > bg.txt"]["files"] and "red.txt" in o["echo RED > red.txt"]["files"] and "copy.txt" in o["cat a.txt > copy.txt"]["files"]
        ok = ok and "via_env.txt" in o["env touch via_env.txt"]["files"] and o["find . -name a.txt -delete"]["files"] == [] and "a.txt.bak" in o["sed -i.bak s/a/b/ a.txt"]["files"]
        ok = ok and o["pwdx"]["result_head"].startswith("exit code 127") and o["datex"]["result_head"].startswith("exit code 127")
        return ok and o["rm -rf nothing"]["result_head"] == "" and o["git status"]["result_head"].startswith("exit code 128")
    add("c12", "f15", "the allowlist refuses | ; $( && || and lets through a single &, > redirects, env <cmd>, find -delete, sed -i, pwdx and datex", c12)

    def c13():
        o = _o(E, "e13-checkpoint-order", "ReleaseFast")
        u, s1, s2 = o["unsafe-../top.txt"], o["safe-keep"], o["safe-keep-with-local-edit"]
        return (u["result"] == ["error: path traversal blocked", True] and u["stash_entries"] == 1 and u["top_txt"] == "v1\n"
                and s1["stash_entries"] == 0 and s2["stash_entries"] == 1 and s2["keep_txt"] == "NEW")
    add("c13", "f17", "a refused write of ../top.txt still stashed top.txt's local edit; a permitted write stashes only a modified file", c13)

    def c14():
        o = _o(E, "e14-input-handling")
        return (o["nested-type-text"]["stdout"] == "INJECTED-BY-INPUT\nfin\n" and o["long-text"] == {"printed": 8170, "of": 20000, "history_keeps_full": True}
                and o["escaped-quote-path"]["result"] == ["read_file: open failed: FileNotFound", True] and o["file-over-512k"]["result"] == ["read_file: read failed: FileTooBig", True])
    add("c14", "f09-f10-f20-f21", "a nested type:text is printed, a text block is cut at 8170 of 20000, an escaped quote is not unescaped, a file over 512 KiB is refused", c14)
    add("c15", "f08", "two tool_use blocks with the same id run twice and answer twice",
        lambda: (lambda o: o["log_lines"] == 2 and [t[0] for t in o["tool_results"]] == ["same", "same"])(_o(E, "e15-duplicate-id", "ReleaseFast")))
    add("c16", "unknown-tool", "an unknown tool name is answered, not executed",
        lambda: _o(E, "e16-unknown-tool")["result"] == ["unknown tool: nope", True])

    def c17():
        for mode, rc_ok in (("Debug", lambda rc: rc == -6), ("ReleaseFast", lambda rc: rc == -11)):
            o = _o(E, "e17-mcp", mode)
            for name in ("no-rule", "allow-star", "allow-and-deny", "allow-arg-pattern"):
                ok_ = o[f"ok|{name}"]
                if not (ok_["rc"] == 0 and ok_["advertised_fake"] == 0 and ok_["result"] == ["unknown tool: fake.echo", True] and ok_["mcp_mark"] == []
                        and ok_["server_saw"] == ["initialize", "notifications/initialized", "tools/list"]):
                    return False
                ni = o[f"noinit|{name}"]
                if not (rc_ok(ni["rc"]) and ni["requests"] == 0 and ni["mcp_mark"] == ["MCP: fake (2 tools)"]):
                    return False
        return True
    add("c17", "f18-f24", "a compliant MCP server yields 0 tools and every call is unknown; a server that skips initialize registers tools and the process dies before its first request (Debug abort, ReleaseFast SIGSEGV)", c17)
    add("c18", "f06", "SIGTERM mid-request: exit by signal 15, no session file",
        lambda: (lambda o: o["rc"] == -15 and o["session_files"] == [] and o["stdout"] == "")(_o(E, "e18-sigterm")))
    add("c19", "f05-f06", "a reply delayed 4 s is waited for with no timeout, one request",
        lambda: (lambda o: o["rc"] == 0 and o["stdout"] == "late\n" and o["requests"] == 1 and o["waited_over_3s"] is True)(_o(E, "e19-slow-provider")))

    def c20():
        o = _o(E, "e20-compaction")
        return (o["requests"] == 2 and o["stdout"] == "final answer\n" and o["req0"]["has_tools"] is False and o["req1"]["has_tools"] is True
                and o["req1"]["first_content_head"].startswith("[Previous context summary]") and o["original_prompt_in_second_request"] is False
                and o["compact_marks"] == ["compacted: summarized conversation"] and o["req0"]["roles"] == ["user"] and o["req1"]["roles"] == ["user"])
    add("c20", "f19-f29", "a 600 KB prompt: a summary request with no tools, then a request that holds only the summary", c20)
    add("c21", "f22", "a duplicate type key inside a tool_use block: the process does not exit within 3 s",
        lambda: (lambda o: o["rc"] == "timeout" and o["requests"] == 1)(_o(E, "e21-hang-duplicate-type")))
    add("c22", "f26", "end_turn with a tool_use block: the tool ran, one request only",
        lambda: (lambda o: o["requests"] == 1 and o["tool_ran"] is True and o["done"] == ["done: end_turn"])(_o(E, "e22-end-turn-with-tool-use", "ReleaseFast")))
    add("c23", "f27", "MCP servers are read from the project file and from $HOME/.tri-api, not from $HOME/.trinity/api",
        lambda: _o(E, "e23-mcp-config-paths", "ReleaseFast") == {"home-.tri-api": {"server_spawned": True}, "home-.trinity/api": {"server_spawned": False}, "project-.trinity/api": {"server_spawned": True}})
    add("c24", "f28", "grep answers spawn failed when the host has no timeout binary, and works when it has one",
        lambda: (lambda o: o["result_head"] == ("grep: spawn failed: FileNotFound" if not o["has_timeout_binary"] else o["result_head"]) and (o["has_timeout_binary"] is False or o["result_head"].startswith("a.txt:")))(_o(E, "e24-grep")))
    return C


def strip_unasserted(o):
    """Observations under keys that start with ~ are recorded, never asserted."""
    if isinstance(o, dict):
        return {k: strip_unasserted(v) for k, v in o.items() if not k.startswith("~")}
    if isinstance(o, list):
        return [strip_unasserted(v) for v in o]
    return o


def check_claims(E) -> list:
    f = []
    need = {sc["id"] for sc in SCENARIOS}
    have = set(E.get("scenarios", {}))
    if need != have:
        f.append(f"e2e: scenarios {sorted(need ^ have)} are missing from or extra in the record")
        return f
    for sc in SCENARIOS:
        modes = set(E["scenarios"][sc["id"]]["modes"])
        if modes != set(sc["modes"]):
            f.append(f"e2e: {sc['id']} holds modes {sorted(modes)}, the harness runs {sorted(sc['modes'])}")
    if f:
        return f
    E = strip_unasserted(E)
    for cid, fnd, text, fn in claims(E):
        try:
            ok = bool(fn())
        except (KeyError, TypeError, IndexError, AttributeError) as e:
            ok, text = False, f"{text} (the record has no such observation: {type(e).__name__} {e})"
        if not ok:
            f.append(f"e2e {cid} [{fnd}]: {text}")
    return f


# ===========================================================================================
# REPLAY: the decision functions of the three specs, generated to C and executed against cases
# whose expected values come from the records, not from the specs
# ===========================================================================================
DRIVER = r"""
#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>
@PROTOTYPES@
static int passes = 0, failures = 0;
static void verdict(const char *fid, const char *fail) { if (fail) { printf("[FIX] %s : FAILED (%s)\n", fid, fail); failures++; } else { printf("[FIX] %s : PASSED\n", fid); passes++; } }
int main(void) {
@CASES@
    printf("fixtures: %d passed, %d failed\n", passes, failures);
    return failures ? 1 : 0;
}
"""


def prototypes(c_text: str) -> list:
    return [l.strip() for l in c_text.split("\n") if re.match(r"^(?:u?int\d+_t|bool|double|float|void)\s+[a-z_]\w*\(.*\);\s*$", l)]


def cb(v) -> str:
    return "true" if v else "false"


def case_c(cid: str, conds: list) -> str:
    """conds: (C expression that must be true, label)."""
    lines = ["    { const char *fail = NULL;"]
    for expr, label in conds:
        lines.append(f'if (!fail && !({expr})) fail = {json.dumps(label)};')
    lines.append(f"verdict({json.dumps(cid)}, fail); }}")
    return "\n    ".join(lines)


def loop_cases(E) -> list:
    E = strip_unasserted(E)
    S = load_spec(SPEC_LOOP)
    T_OK, T_STATUS, T_FAILED = S["TRANSPORT_OK"], S["TRANSPORT_STATUS"], S["TRANSPORT_FAILED"]
    K_DONE, K_LIMIT, K_TRANSPORT = S["TERMINATION_DONE"], S["TERMINATION_TURN_LIMIT"], S["TERMINATION_TRANSPORT"]
    g = lambda sid, mode="Debug": E["scenarios"][sid]["modes"][mode]
    cases = []
    o = g("e22-end-turn-with-tool-use", "ReleaseFast")
    cases.append(("loop-end-turn-with-tool-use", [(f"turn_continues(true, true) == {cb(o['requests'] > 1)}", "continues"),
                                                 (f"results_sent_after_end_turn(true) == {cb(o['requests'] > 1)}", "results sent")]))
    o = g("e06-turn-limit")
    n = o["requests"]
    cases.append(("loop-turn-limit", [(f"requests_sent(true) == {n}u", "requests"), (f"turn_limit_reached({n}u) == true", "at the limit"), (f"turn_limit_reached({n - 1}u) == false", "below the limit"),
                                      (f"termination({T_OK}u, true, {n - 1}u) == {K_LIMIT}u", "termination"), (f"exit_code_for({K_LIMIT}u) == {o['rc']}u", "exit")]))
    o = g("e05-error-replies")
    for name, v in o.items():
        status = int(name.split("-")[1]) if name.startswith("http-") else 200
        cls = 1 if any(m.startswith("API status") for m in v["marks"]) else 0
        done = any(m.startswith("done") for m in v["marks"])
        cases.append((f"loop-{name}", [(f"transport_class({status}u, true) == {cls}u", "class"),
                                       (f"termination(transport_class({status}u, true), false, 0u) == {(K_DONE if done else K_TRANSPORT)}u", "termination"),
                                       (f"exit_code_for({K_DONE}u) == {v['rc']}u", "exit"), (f"retries_after({status}u) == {v['requests'] - 1}u", "retries")]))
    n_in = int(g("e08-usage")["tokens_line"][0].split()[0])
    cases.append(("loop-usage", [(f"usage_recorded(1234u) == {n_in}u", "usage")]))
    roles = g("e02-two-tool-use")["roles"]
    trailing = len(roles) - 1 - max(i for i, r in enumerate(roles) if r == "assistant")
    cases.append(("loop-consecutive-user-messages", [(f"user_messages_for_results(2u) == {trailing}u", "messages")]))
    cases.append(("loop-duplicate-id", [(f"executions(2u, true) == {g('e15-duplicate-id', 'ReleaseFast')['log_lines']}u", "executions")]))
    lt = g("e14-input-handling")["long-text"]
    cases.append(("loop-text-window", [(f"text_printed({lt['of']}u) == {lt['printed']}u", "printed")]))
    cases.append(("loop-cancel", [(f"session_saved_on_cancel() == {cb(bool(g('e18-sigterm')['session_files']))}", "session")]))
    return cases


DOTDOT, NUL = b"..", b"\x00"


def perm_cases(vectors, Z, E) -> list:
    E = strip_unasserted(E)
    cases = []
    for v in vectors:
        i = v["in"]
        if v["kind"] == "check":
            rules = lambda key: [(B(x), B(y)) for x, y in i[key]]
            t, a = B(i["t"]), B(i["a"])
            has_deny = any(rule_matches(r, t, a) for r in rules("deny"))
            has_allow = any(rule_matches(r, t, a) for r in rules("allow"))
            dflt = t in DEFAULT_ALLOW_TOOLS
            want = 0 if v["model"] == "allow" else 1
            cases.append((f"perm-{v['id']}", [(f"permission({cb(has_deny)}, {cb(has_allow)}, {cb(dflt)}) == {want}u", "decision")]))
        elif v["kind"] == "parse_rule_array":
            global MAX_RULES
            keep, MAX_RULES = MAX_RULES, 10 ** 9
            try:
                written = len(parse_rule_array(B(i["data"]), B(i["key"]), []))
            finally:
                MAX_RULES = keep
            cases.append((f"perm-{v['id']}", [(f"rules_admitted({written}u) == {len(v['model'])}u", "admitted")]))
        elif v["kind"] == "is_path_safe":
            p = B(i["p"])
            has_dd, has_nul = DOTDOT in p, NUL in p
            cases.append((f"perm-{v['id']}", [(f"path_safe({cb(has_dd)}, {cb(has_nul)}) == {cb(v['model'])}", "path")]))
        elif v["kind"] == "is_bash_allowed":
            t = B(i["c"]).lstrip(WS)
            listed = any(t.startswith(p) for p in BASH_PREFIXES) or t in BARE_OK
            meta = any(bytes([m]) in t for m in SHELL_META)
            pair = b"&&" in t or b"||" in t
            cases.append((f"perm-{v['id']}", [(f"bash_allowed({cb(listed)}, {cb(meta)}, {cb(pair)}) == {cb(v['model'])}", "bash")]))
    for mode, u in Z["uaf_loadfromfile"].items():
        eff = u["check_allow_bash_echo"] == "allow" and u["check_deny_read_env"] == "deny"
        cases.append((f"perm-uaf-{mode}", [(f"rules_effective({cb(mode in ('Debug', 'ReleaseSafe'))}) == {cb(eff)}", "effective")]))
    o = E["scenarios"]["e11-rule-cap"]["modes"]["ReleaseFast"]
    loaded = int(re.match(r"permissions: (\d+) allow", o["loaded"][0]).group(1))
    cases.append(("perm-rule-cap", [(f"rules_admitted({o['written']}u) == {loaded}u", "cap")]))
    o = E["scenarios"]["e13-checkpoint-order"]["modes"]["ReleaseFast"]["unsafe-../top.txt"]
    cases.append(("perm-checkpoint-order", [(f"checkpoint_runs_before_path_check() == {cb(o['stash_entries'] > 0)}", "order")]))
    o = E["scenarios"]["e17-mcp"]["modes"]["ReleaseFast"]
    cases.append(("perm-mcp-discovery", [(f"mcp_lists_tools(true) == {cb(o['ok|allow-star']['advertised_fake'] > 0)}", "compliant"),
                                         (f"mcp_lists_tools(false) == {cb(bool(o['noinit|allow-star']['mcp_mark']))}", "silent server")]))
    return cases


def ctx_cases(vectors, E) -> list:
    E = strip_unasserted(E)
    cases = []
    for v in vectors:
        i = v["in"]
        if v["kind"] == "estimate_tokens":
            cases.append((f"ctx-{v['id']}", [(f"estimate_tokens({i['n']}u) == {v['model']}u", "estimate")]))
        elif v["kind"] == "is_near_limit":
            cases.append((f"ctx-{v['id']}", [(f"near_limit({i['n']}u) == {cb(v['model'])}", "near")]))
    o = E["scenarios"]["e20-compaction"]["modes"]["Debug"]
    hist = b'[{"role":"user","content":"' + b"A" * 600000 + b'"}'      # the history after the 600000-byte prompt of scenario e20; the request holds its first 400000 bytes plus the decoded instruction
    cases.append(("ctx-compaction-prompt", [(f"summary_keeps_first_prompt() == {cb(o['original_prompt_in_second_request'])}", "prompt"),
                                            (f"summary_request_has_tools() == {cb(o['req0']['has_tools'])}", "tools"),
                                            (f"excerpt_bytes({len(hist)}u) + {len(unescape_string(SUMMARY_PROMPT))}u == {o['req0']['first_content_bytes']}u", "excerpt")]))
    return cases


def replay_spec(spec: pathlib.Path, cases: list, t27c: pathlib.Path, work: pathlib.Path) -> dict:
    work.mkdir(parents=True, exist_ok=True)
    rc, out, err = run([str(t27c), "gen-c", str(spec)])
    if rc != 0 or not out.strip():
        return {"ok": False, "stage": "generate", "detail": (err or out).strip()[:300], "passed": 0, "failed": 0, "failures": []}
    (work / "spec.c").write_text(out, encoding="utf-8")
    driver = DRIVER.replace("@PROTOTYPES@", "\n".join(prototypes(out))).replace("@CASES@", "\n".join(case_c(cid, conds) for cid, conds in cases))
    (work / "driver.c").write_text(driver, encoding="utf-8")
    for src, obj in (("spec.c", "spec.o"), ("driver.c", "driver.o")):
        rc, o2, e2 = run(["cc", "-std=c11", "-w", "-c", "-x", "c", str(work / src), "-o", str(work / obj)])
        if rc != 0:
            return {"ok": False, "stage": "compile", "detail": f"{src}: {e2.strip()[:400]}", "passed": 0, "failed": 0, "failures": []}
    rc, o2, e2 = run(["cc", str(work / "driver.o"), str(work / "spec.o"), "-o", str(work / "driver")])
    if rc != 0:
        return {"ok": False, "stage": "link", "detail": e2.strip()[:400], "passed": 0, "failed": 0, "failures": []}
    rc, o2, e2 = run([str(work / "driver")], timeout=60)
    results = re.findall(r"^\[FIX\] (.+?) : (PASSED|FAILED)(?: \((.*)\))?$", o2, re.M)
    failures = [{"id": i, "detail": d or ""} for i, v, d in results if v == "FAILED"]
    passed = sum(1 for _, v, _ in results if v == "PASSED")
    ok = rc == 0 and not failures and passed == len(cases)
    return {"ok": ok, "stage": "runtime", "detail": "" if results else (e2 or o2)[-200:], "passed": passed, "failed": len(failures),
            "failures": failures, "fixtures": len(cases), "generated": sha256(out.encode())}


def replay_all(vectors, Z, E, t27c, specs=None, cases_override=None) -> dict:
    specs = specs or SPECS
    cs = cases_override or {"loop": loop_cases(E), "permissions": perm_cases(vectors, Z, E), "context": ctx_cases(vectors, E)}
    res = {}
    with tempfile.TemporaryDirectory() as tmp:
        for k in ("loop", "permissions", "context"):
            r = replay_spec(specs[k], cs[k], t27c, pathlib.Path(tmp) / k)
            r["spec_sha256"] = sha256(pathlib.Path(specs[k]).read_bytes())
            res[k] = r
    return {"at": now(), "host": host(), "t27c": (run([str(t27c), "--version"], timeout=60)[1] or "t27c").strip().split("\n")[0][:80], "specs": res}


# ===========================================================================================
# CHECK
# ===========================================================================================
SPEC_CONSTANTS = {   # spec key -> [(spec constant, inventory constant or literal-path)]
    "loop": [("MAX_TURNS", "max_turns"), ("API_VERSION", "api_version"), ("DEFAULT_MODEL", "default_model"),
             ("MAX_TOKENS", "max_tokens_request"), ("TEXT_WINDOW_BYTES", "text_window")],
    "permissions": [("MAX_RULES", "max_rules"), ("BASH_PREFIX_COUNT", "bash_prefix_count"), ("READ_LIMIT_BYTES", "read_limit"), ("BASH_META", "shell_meta")],
    "context": [("COMPACT_THRESHOLD", "compact_threshold"), ("KEEP_TURNS", "keep_turns"), ("MAX_TOKENS", "context_max_tokens"), ("EXCERPT_MAX_BYTES", "excerpt_max")],
}
ZIG_TEST_COUNTS = {"tool_protocol": 5, "permissions": 7, "tool_executor": 12, "context": 8}   # the pinned files' own tests, read off the run


def check_inventory(inv, specs) -> list:
    f = []
    if inv.get("pin") != DEFAULT_REV:
        f.append(f"inventory: pin {inv.get('pin')} is not {DEFAULT_REV}")
    if inv.get("src_tri_api_changed_since_baseline") != []:
        f.append(f"inventory: src/tri-api changed since the issue baseline: {inv.get('src_tri_api_changed_since_baseline')}")
    for key, pairs in SPEC_CONSTANTS.items():
        sp = specs[key]
        if sp.get("PINNED_REVISION") != inv["pin"] or sp.get("ISSUE_BASELINE") != inv["issue_baseline"]:
            f.append(f"{key}: PINNED_REVISION/ISSUE_BASELINE differ from the inventory")
        for c, ic in pairs:
            if sp.get(c) != inv["constants"].get(ic):
                f.append(f"{key}: {c} is {sp.get(c)!r}, the source says {inv['constants'].get(ic)!r}")
    if specs["loop"].get("BUILTIN_TOOLS") != inv.get("builtin_tools"):
        f.append("loop: BUILTIN_TOOLS differ from the inventory")
    if len(BASH_PREFIXES) != inv["constants"]["bash_prefix_count"]:
        f.append(f"model: {len(BASH_PREFIXES)} bash prefixes, the source has {inv['constants']['bash_prefix_count']}")
    return f


def check_vectors(doc, live) -> list:
    f = []
    if json.dumps(doc.get("vectors"), sort_keys=True) != json.dumps(live, sort_keys=True):
        a = {v["id"]: v for v in doc.get("vectors", [])}
        bad = [v["id"] for v in live if a.get(v["id"]) != v]
        f.append(f"vectors: the file drifts from what `vectors` writes now ({len(bad)} differ{': ' + ', '.join(bad[:6]) if bad else ''})")
    return f


def check_zig(Z, vectors) -> list:
    f = []
    if Z.get("pin") != DEFAULT_REV:
        f.append("zig: the record is not at the pin")
    if set(Z.get("modes", {})) != set(ZIG_MODES):
        f.append(f"zig: modes {sorted(Z.get('modes', {}))}, expected {list(ZIG_MODES)}")
        return f
    for mode in ZIG_MODES:
        for module, kinds in ZIG_MODULES.items():
            r = Z["modes"][mode].get(module)
            n = len([v for v in vectors if v["kind"] in kinds])
            if not r:
                f.append(f"zig {mode}/{module}: no record")
                continue
            if r["rc"] != 0:
                f.append(f"zig {mode}/{module}: did not pass (rc {r['rc']}): {r.get('tail', '')[:160]}")
            if r["disagree"] or r["missing"]:
                f.append(f"zig {mode}/{module}: the model disagrees with the compiled Zig on {[d['id'] for d in r['disagree']][:5]}, missing {r['missing'][:5]}")
            if r["vectors"] != n or r["agree"] != n:
                f.append(f"zig {mode}/{module}: {r['agree']} of {r['vectors']} vectors agree, the corpus holds {n}")
            if r.get("upstream_tests_passed") != ZIG_TEST_COUNTS[module]:
                f.append(f"zig {mode}/{module}: {r.get('upstream_tests_passed')} upstream tests passed, expected {ZIG_TEST_COUNTS[module]}")
    uaf = Z.get("uaf_loadfromfile", {})
    perm = load_spec(SPEC_PERM)
    for mode in ZIG_MODES:
        u = uaf.get(mode)
        if not u:
            f.append(f"zig uaf {mode}: no record")
            continue
        eff = u["check_allow_bash_echo"] == "allow" and u["check_deny_read_env"] == "deny"
        want = mode in perm["FAST_MODES"]
        if mode not in perm["FAST_MODES"] and mode not in perm["SAFE_MODES"]:
            f.append(f"zig uaf {mode}: the spec lists the mode in neither SAFE_MODES nor FAST_MODES")
        if eff != want:
            f.append(f"zig uaf {mode}: rules effective={eff}, the spec says {want}")
        if not want and u["allow0_tool"] != [170, 170, 170, 170]:
            f.append(f"zig uaf {mode}: the freed bytes are {u['allow0_tool']}, not 0xAA")
    return f


def scenario_tags() -> set:
    tags = set()
    for sc in SCENARIOS:
        tags |= set(re.findall(r"f\d\d", sc["finding"]))
    return tags


def check_findings(specs, e2e_ids, vector_ids) -> list:
    f = []
    seen = {}
    for key, sp in specs.items():
        items = sp.get("FINDINGS", [])
        if sp.get("FINDINGS_COUNT") != len(items):
            f.append(f"{key}: FINDINGS_COUNT {sp.get('FINDINGS_COUNT')} but {len(items)} findings")
        for it in items:
            m = re.match(r"(f\d\d) ", it)
            if not m:
                f.append(f"{key}: a finding without an id: {it[:40]}")
                continue
            fid = m.group(1)
            if fid in seen:
                f.append(f"{key}: {fid} is also in {seen[fid]}")
            seen[fid] = key
            tag = re.search(r"\[([^\]]*)\]\s*$", it)
            if not tag:
                f.append(f"{key} {fid}: no evidence tag")
                continue
            for part in re.split(r"[;,]", tag.group(1)):
                part = part.strip()
                if re.fullmatch(r"e\d\d", part):
                    if not any(i.startswith(part + "-") for i in e2e_ids):
                        f.append(f"{key} {fid}: scenario {part} is not in the harness")
                elif part.startswith("vector"):
                    for vid in part.split()[1:]:
                        if vid not in vector_ids:
                            f.append(f"{key} {fid}: vector {vid} is not in the corpus")
                elif part.startswith("zig uaf") or part in ("model", "recorded, not asserted") or part.startswith("the failure path"):
                    pass
                else:
                    f.append(f"{key} {fid}: unrecognised evidence '{part}'")
    for t in sorted(scenario_tags() - set(seen)):
        f.append(f"a scenario measures {t} and no spec states it")
    for t in sorted(set(seen) - scenario_tags()):
        f.append(f"{t} is stated by a spec and measured by no scenario")
    return f


def check_replay(doc, specs_paths, vectors, Z, E) -> list:
    f = []
    r = doc.get("replay")
    if not isinstance(r, dict):
        return ["replay: none recorded; run `run`"]
    want = {"loop": len(loop_cases(E)), "permissions": len(perm_cases(vectors, Z, E)), "context": len(ctx_cases(vectors, E))}
    for k, p in specs_paths.items():
        x = r.get("specs", {}).get(k)
        if not x:
            f.append(f"replay {k}: none")
            continue
        if x.get("spec_sha256") != sha256(pathlib.Path(p).read_bytes()):
            f.append(f"replay {k}: the spec changed since the replay; run `run`")
        if not x.get("ok") or x.get("failed", 1) != 0 or x.get("passed") != want[k]:
            f.append(f"replay {k}: {x.get('passed')} passed, {x.get('failed')} failed of {want[k]} at stage {x.get('stage')}: {x.get('detail', '')[:160]}")
    return f


def load_json(p):
    return json.loads(pathlib.Path(p).read_text()) if pathlib.Path(p).exists() else None


def check_all() -> list:
    f = []
    specs = {}
    for k, p in SPECS.items():
        if not p.exists():
            return [f"{p}: missing"]
        try:
            specs[k] = load_spec(p)
        except ValueError as e:
            return [str(e)]
    inv, vec, Z, E = (load_json(p) for p in (INVENTORY, VECTORS, EVIDENCE_ZIG, EVIDENCE_E2E))
    for name, d in (("inventory", inv), ("vectors", vec), ("zig", Z), ("e2e", E)):
        if d is None:
            f.append(f"{name}: no record; run `{name}`")
    if f:
        return f
    f += check_inventory(inv, specs)
    live = vectors_with_model()
    f += check_vectors(vec, live)
    f += check_zig(Z, live)
    if E.get("pin") != DEFAULT_REV:
        f.append("e2e: the record is not at the pin")
    f += check_claims(E)
    f += check_findings(specs, {sc["id"] for sc in SCENARIOS}, {v["id"] for v in live})
    if not f:
        f += check_replay(vec, SPECS, live, Z, E)
    return f


# ===========================================================================================
# --self-check: every gate here is shown to fail when its fault is planted
# ===========================================================================================
def self_check(trinity, zig, sysroot) -> int:
    ok = True

    def expect(cond: bool, what: str):
        nonlocal ok
        print(f"  {'ok ' if cond else 'BAD'} {what}")
        ok = ok and cond

    specs = {k: load_spec(p) for k, p in SPECS.items()}
    inv, vec, Z, E = (load_json(p) for p in (INVENTORY, VECTORS, EVIDENCE_ZIG, EVIDENCE_E2E))
    expect(all(d is not None for d in (inv, vec, Z, E)), "the four committed records exist")
    if not all(d is not None for d in (inv, vec, Z, E)):
        return 1
    live = vectors_with_model()
    clone = lambda d: json.loads(json.dumps(d))
    f = check_all()
    expect(not f, f"check: the committed records and specs agree ({len(f)} finding(s){': ' + f[0] if f else ''})")
    expect(len(SCENARIOS) == 24 and len(live) == 149, f"the harness holds 24 scenarios and the corpus 149 vectors ({len(SCENARIOS)}, {len(live)})")
    # e2e claims
    p = clone(E)
    p["scenarios"]["e06-turn-limit"]["modes"]["Debug"]["requests"] = 19
    expect(any(x.startswith("e2e c06") for x in check_claims(p)), "planted: a turn limit observed at 19 requests breaks claim c06")
    p = clone(E)
    del p["scenarios"]["e22-end-turn-with-tool-use"]
    expect(any("missing from or extra" in x for x in check_claims(p)), "planted: a missing scenario is reported")
    p = clone(E)
    p["scenarios"]["e09-settings-shapes"]["modes"]["Debug"]["deny-read-env"]["result"] = ["Permission denied: read_file(.env)", True]
    expect(any(x.startswith("e2e c09") for x in check_claims(p)), "planted: a Debug build whose deny rule works breaks claim c09 (the use-after-free would be gone)")
    p = clone(E)
    p["scenarios"]["e11-rule-cap"]["modes"]["ReleaseFast"]["~first_call_rc"] = 0
    expect(not check_claims(p), "an unasserted observation (~first_call_rc) may change without a finding")
    # vectors
    p = clone(vec)
    for v in p["vectors"]:
        if v["id"] == "ps-absolute":
            v["model"] = False
    expect(any("drifts" in x for x in check_vectors(p, live)), "planted: a changed model answer is a drift")
    # zig
    p = clone(Z)
    p["modes"]["Debug"]["permissions"]["disagree"] = [{"id": "ps-x", "model": True, "zig": False}]
    expect(any("disagrees" in x for x in check_zig(p, live)), "planted: a model/Zig disagreement is reported")
    p = clone(Z)
    p["uaf_loadfromfile"]["Debug"]["check_deny_read_env"] = "deny"
    p["uaf_loadfromfile"]["Debug"]["check_allow_bash_echo"] = "allow"
    expect(any("uaf Debug" in x for x in check_zig(p, live)), "planted: an intact rule table in Debug contradicts the spec's SAFE_MODES")
    p = clone(Z)
    p["modes"]["ReleaseSmall"]["context"]["upstream_tests_passed"] = 7
    expect(any("upstream tests" in x for x in check_zig(p, live)), "planted: a lost upstream test is reported")
    # inventory and constants
    p = clone(specs)
    p["loop"]["MAX_TURNS"] = 21
    expect(any("MAX_TURNS" in x for x in check_inventory(inv, p)), "planted: MAX_TURNS 21 against a source that says 20")
    p = clone(inv)
    p["src_tri_api_changed_since_baseline"] = ["src/tri-api/main.zig"]
    expect(any("changed since the issue baseline" in x for x in check_inventory(p, specs)), "planted: a moved baseline is reported")
    # findings
    p = clone(specs)
    p["loop"]["FINDINGS"][0] = p["loop"]["FINDINGS"][0].replace("[e08,", "[e99,")
    expect(any("e99" in x for x in check_findings(p, {sc["id"] for sc in SCENARIOS}, {v["id"] for v in live})), "planted: a finding that cites a scenario that does not exist")
    p = clone(specs)
    p["permissions"]["FINDINGS"] = [x for x in p["permissions"]["FINDINGS"] if not x.startswith("f17")]
    p["permissions"]["FINDINGS_COUNT"] = len(p["permissions"]["FINDINGS"])
    expect(any("f17" in x for x in check_findings(p, {sc["id"] for sc in SCENARIOS}, {v["id"] for v in live})), "planted: a measured defect the specs no longer state")
    # replay
    t27c = t27c_path()
    if t27c and shutil.which("cc"):
        good = replay_all(live, Z, E, t27c)
        expect(all(x["ok"] for x in good["specs"].values()), "replay: the three specs pass every case (" + ", ".join(f"{k} {x['passed']}/{x.get('fixtures')}" for k, x in good["specs"].items()) + ")")
        expect(not check_replay({"replay": good}, SPECS, live, Z, E), "check_replay accepts a fresh replay")
        with tempfile.TemporaryDirectory() as tmp:
            tmp = pathlib.Path(tmp)

            def planted(key, old, new):
                text = SPECS[key].read_text(encoding="utf-8")
                assert old in text, old
                pp = tmp / f"{key}.t27"
                pp.write_text(text.replace(old, new), encoding="utf-8")
                return replay_all(live, Z, E, t27c, specs={**SPECS, key: pp})["specs"][key]

            r = planted("loop", "    if (stop_is_end_turn) { return false; }\n    return has_tool_use;", "    return has_tool_use;")
            expect(any(x["id"] == "loop-end-turn-with-tool-use" for x in r["failures"]), "planted: a loop that continues after end_turn fails the e22 case")
            r = planted("loop", "pub const MAX_TURNS : u32 = 20;", "pub const MAX_TURNS : u32 = 25;")
            expect(r["stage"] == "compile" and "the_turn_budget_is_twenty" in r["detail"], "planted: a turn budget of 25 stops at the spec's own invariant, before any case runs")
            r = planted("loop", "    return turn >= MAX_TURNS;", "    return turn > MAX_TURNS;")
            expect(any(x["id"] == "loop-turn-limit" for x in r["failures"]), "planted: a limit that allows a 21st request fails the e06 case")
            r = planted("loop", "pub fn usage_recorded(provider_reported: u32) -> u32 {\n    return 0;", "pub fn usage_recorded(provider_reported: u32) -> u32 {\n    return provider_reported;")
            expect(any(x["id"] == "loop-usage" for x in r["failures"]), "planted: a loop that counts usage fails the e08 case")
            r = planted("permissions", "    if (has_dotdot) { return false; }\n", "")
            ids = sorted(x["id"] for x in r["failures"])
            expect("perm-ps-dotdot" in ids and "perm-ps-dots-in-name" in ids, f"planted: a path predicate that ignores .. fails the dotdot vectors ({ids[:4]})")
            r = planted("permissions", "    if (written > MAX_RULES) { return MAX_RULES; }\n", "")
            expect(any(x["id"] == "perm-pra-cap" for x in r["failures"]) and any(x["id"] == "perm-rule-cap" for x in r["failures"]), "planted: no rule cap fails the pra-cap vector and the e11 case")
            r = planted("permissions", "    return safe_build == false;", "    return true;")
            expect(any(x["id"] == "perm-uaf-Debug" for x in r["failures"]), "planted: rules that survive in Debug fail the use-after-free record")
            r = planted("permissions", "    if (has_meta) { return false; }\n", "")
            expect(any(x["id"].startswith("perm-ba-") for x in r["failures"]), "planted: a bash allowlist without the metacharacter rule fails the bash vectors")
            r = planted("context", "pub fn summary_keeps_first_prompt() -> bool {\n    return false;", "pub fn summary_keeps_first_prompt() -> bool {\n    return true;")
            expect(any(x["id"] == "ctx-compaction-prompt" for x in r["failures"]), "planted: a summary that keeps the prompt fails the e20 case")
            r = planted("context", "pub const COMPACT_THRESHOLD : u32 = 144000;", "pub const COMPACT_THRESHOLD : u32 = 140000;")
            expect(r["stage"] == "compile" and "the_threshold_is_four_fifths_of_the_window" in r["detail"], "planted: a threshold of 140000 stops at the spec's own invariant")
            r = planted("context", "    return estimate_tokens(bytes) >= COMPACT_THRESHOLD;", "    return estimate_tokens(bytes) > COMPACT_THRESHOLD;")
            expect(any(x["id"] == "ctx-near-at" for x in r["failures"]), "planted: a strict comparison fails the at-threshold vector")
    else:
        print("  skip replay checks: t27c or cc missing")
    # model against the compiled Zig is carried by the zig record; plant a wrong model and show the vectors file notices
    keep = globals()["is_path_safe"]
    globals()["is_path_safe"] = lambda path: b".." not in path
    try:
        expect(any("drifts" in x for x in check_vectors(vec, vectors_with_model())), "planted: a model that forgets the NUL rule no longer matches the committed answers")
    finally:
        globals()["is_path_safe"] = keep
    # live rerun
    if trinity and zig:
        ids = ["e16-unknown-tool", "e22-end-turn-with-tool-use", "e03-write-default-deny"]
        live_e = run_e2e(trinity, DEFAULT_REV, zig, sysroot, only=ids)
        same = all(strip_unasserted(live_e[i]) == strip_unasserted(E["scenarios"][i]) for i in ids)
        expect(same, f"rerun: {len(ids)} scenarios driven again against a fresh build give the recorded observations")
        inv2 = build_inventory(trinity, DEFAULT_REV)
        expect(inv2["files"] == inv["files"] and inv2["constants"] == inv["constants"], "inventory: a fresh read of the clone matches the committed digests and constants")
    print("trinity_tri_api --self-check:", "ok" if ok else "FAILED")
    return 0 if ok else 1


# ===========================================================================================
# CLI
# ===========================================================================================
def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("command", nargs="?", choices=["inventory", "vectors", "zig", "e2e", "run", "check"])
    ap.add_argument("--trinity-root")
    ap.add_argument("--rev", default=DEFAULT_REV)
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
    if a.command == "vectors":
        vs = vectors_with_model()
        old = load_json(VECTORS) or {}
        doc = {"description": "The tri-api vector corpus and the Python model's answer for each. Bytes are latin-1 strings. The compiled Zig's answers are recorded in tri_api_zig.json; the specs' decision functions are replayed against cases derived from these records (`replay`).",
               "count": len(vs), "kinds": sorted({v["kind"] for v in vs}), "vectors": vs}
        if "replay" in old:
            doc["replay"] = old["replay"]
        wr(VECTORS, doc)
        print(f"vectors: {len(vs)} vectors, {len(doc['kinds'])} kinds")
        return 0
    if a.command in ("inventory", "zig", "e2e"):
        if not trinity:
            print(f"{a.command}: --trinity-root is required", file=sys.stderr)
            return 2
    if a.command == "inventory":
        inv = build_inventory(trinity, a.rev)
        wr(INVENTORY, inv)
        print(f"inventory: {len(inv['files'])} files at {a.rev[:8]}, changed since the baseline: {inv['src_tri_api_changed_since_baseline']}")
        return 0
    if a.command in ("zig", "e2e"):
        if not a.zig:
            print(f"{a.command}: --zig is required", file=sys.stderr)
            return 2
        try:
            if a.command == "zig":
                doc = build_zig_record(trinity, a.rev, a.zig, a.sysroot, vectors_with_model())
                wr(EVIDENCE_ZIG, doc)
                bad = sum(len(r["disagree"]) + len(r["missing"]) for m in doc["modes"].values() for r in m.values())
                print(f"zig: {len(ZIG_MODES)} modes x {len(ZIG_MODULES)} modules, {bad} disagreement(s); uaf: " + ", ".join(f"{m}={'works' if u['check_deny_read_env'] == 'deny' else 'broken'}" for m, u in doc["uaf_loadfromfile"].items()))
                return 0 if bad == 0 else 1
            doc = build_e2e_record(trinity, a.rev, a.zig, a.sysroot)
            wr(EVIDENCE_E2E, doc)
            f = check_claims(doc)
            print(f"e2e: {len(doc['scenarios'])} scenarios, {len(f)} claim(s) failed")
            for x in f:
                print("  " + x)
            return 0 if not f else 1
        except (RuntimeError, OSError, FileNotFoundError) as e:
            print(f"{a.command}: could not run: {e}", file=sys.stderr)
            return 2
    if a.command == "run":
        t27c = t27c_path()
        Z, E, vec = load_json(EVIDENCE_ZIG), load_json(EVIDENCE_E2E), load_json(VECTORS)
        if not (t27c and shutil.which("cc") and Z and E and vec):
            print("run: needs t27c, cc and the zig, e2e and vectors records", file=sys.stderr)
            return 2
        vec["replay"] = replay_all(vectors_with_model(), Z, E, t27c)
        wr(VECTORS, vec)
        for k, x in vec["replay"]["specs"].items():
            print(f"run {k}: {x['passed']} passed, {x['failed']} failed ({x['stage']}) {x['detail'][:120]}")
        return 0 if all(x["ok"] for x in vec["replay"]["specs"].values()) else 1
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
