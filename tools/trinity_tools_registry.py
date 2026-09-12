#!/usr/bin/env python3
"""The Trinity CLI and MCP catalog held to one verified registry: the cards under specs/tools/
against what gHashTag/trinity exports, dispatches and advertises, plus offline MCP fixtures.

WHY THIS EXISTS
---------------
S06 of gHashTag/trinity#988 (gHashTag/t27#3568): the consumer has five dispatch layers, a 187-entry
command table of which the binary exports 29, an MCP server that advertises 210 tools from a
literal and an exporter that writes 29 others, and documents that count 35, 47, 100 and 280.
specs/tools/catalog.t27 names the one registry the catalog derives from -- .trinity/registry.json,
exported by `zig build export-registry` and drift-checked in CI -- and states the schema, the
repository-qualified IDs and the legacy resolution. This tool measures the consumer and holds the
cards, the catalog spec and the protocol spec to the measurement, so that an addition or a removal
the cards do not explain fails.

WHAT IS MEASURED
----------------
  inventory  reads .trinity/registry.json and .trinity/mcp_schemas.json, counts the command table
             (src/registry/command_table.zig), the execute map (src/tri/tri_register.zig), the
             parseCommand tokens (src/tri/tri_utils.zig), the first-argument chain (src/tri/main.zig),
             the cell dispatch maps (src/tri/tri_cell_dispatch.zig), the MCP server's tool literal
             (tools/mcp/trinity_mcp/server.zig; each object parsed on its own), the needle server, the
             resources and prompts, and with --tri the help output of the t27 Rust tri against the 52
             t27 cards (witness help-output). Writes conformance/trinity/tools_inventory.json.
  check      the 29 Trinity cards equal the registry field by field and cover it exactly; every card
             carries schema 2 (REPO, QUALIFIED_ID = REPO:ID); the catalog spec's counts, collisions
             and legacy table equal the inventory; the protocol spec's counts equal the inventory.
  fixtures   writes conformance/trinity/mcp_fixtures.json from a Python model of processMessage and
             handleToolsCall; run replays it through the generated C of specs/tools/mcp_protocol.t27.
Nothing here runs the consumer's binaries: no workflow does either, and no Zig on this host builds
the tree. The registry is the binary's own export; the rest is source, read at the pin.

Usage:
  python3 tools/trinity_tools_registry.py inventory --trinity-root <clone> [--tri <t27 tri binary>]
  python3 tools/trinity_tools_registry.py check [--inventory conformance/trinity/tools_inventory.json]
  python3 tools/trinity_tools_registry.py fixtures
  python3 tools/trinity_tools_registry.py run
  python3 tools/trinity_tools_registry.py --self-check [--trinity-root <clone>]

Exit codes: 0 no finding; 1 findings; 2 could not run.
"""
from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import pathlib
import platform
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
TOOLS_DIR = ROOT / "specs/tools"
CATALOG = ROOT / "specs/tools/catalog.t27"
PROTOCOL = ROOT / "specs/tools/mcp_protocol.t27"
INVENTORY = ROOT / "conformance/trinity/tools_inventory.json"
FIXTURES = ROOT / "conformance/trinity/mcp_fixtures.json"
CONST_RE = re.compile(r"^pub const (\w+) : ([^=]+?) = (.*);\s*$")
INT_TYPES = {"i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "usize", "isize"}
T27_REPO, TRINITY_REPO = "gHashTag/t27", "gHashTag/trinity"


def sha256(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def run(cmd: list[str], cwd=None, timeout: int = 300) -> tuple[int, str, str]:
    try:
        p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout)
        return p.returncode, p.stdout, p.stderr
    except (OSError, subprocess.TimeoutExpired) as e:
        return 127, "", str(e)


def t27c_path() -> pathlib.Path | None:
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


def load_card(path: pathlib.Path) -> dict:
    text = path.read_text(encoding="utf-8")
    if re.search(r"[^\x00-\x7f]", text):
        raise ValueError(f"{path}: non-ASCII byte (L3)")
    f = {"_path": str(path)}
    for n, line in enumerate(text.split("\n"), 1):
        m = CONST_RE.match(line)
        if m:
            f[m.group(1)] = parse_value(m.group(3), m.group(2).strip())
        elif line.startswith("pub const"):
            raise ValueError(f"{path}:{n}: cannot read constant line")
    return f


def load_cards(tools_dir: pathlib.Path) -> dict[str, list[dict]]:
    out = {"t27_tri": [], "mcp": [], "trinity_tri": []}
    for p in sorted((tools_dir / "tri").glob("*.t27")):
        out["t27_tri"].append(load_card(p))
    for p in sorted((tools_dir / "mcp").glob("*.t27")):
        out["mcp"].append(load_card(p))
    for p in sorted((tools_dir / "trinity/tri").glob("*.t27")):
        out["trinity_tri"].append(load_card(p))
    return out


# ---------------------------------------------------------------------------
# inventory: the consumer, measured.
# ---------------------------------------------------------------------------
def read(trinity: pathlib.Path, rel: str) -> str:
    p = trinity / rel
    return p.read_text(encoding="utf-8", errors="replace") if p.exists() else ""


def table_entries(text: str) -> list[str]:
    """Top-level CommandDef names of all_commands: single-line `    .{ .name = "x", ...` entries and the
    `.name` line right after a multi-line `    .{` opener; nested Subcommand and InputParam names are not counted."""
    start, end = text.find("pub const all_commands"), text.find("fn countMcpTools")
    seg = text[start:end] if start >= 0 and end > start else text
    names, opened = [], False
    for line in seg.split("\n"):
        m = re.match(r'^    \.\{ \.name = "([^"]+)"', line)
        if m:
            names.append(m.group(1)); opened = False; continue
        if re.match(r"^    \.\{\s*$", line):
            opened = True; continue
        m = re.match(r'^        \.name = "([^"]+)"', line)
        if m and opened:
            names.append(m.group(1)); opened = False
    return names


def entry_names_at_indent(text: str, indent: int) -> list[str]:
    return re.findall(r'^' + " " * indent + r'\.name = "([^"]+)"', text, re.M)


def execute_map(text: str) -> tuple[list[str], list[str]]:
    live, dead = [], []
    body = text[text.index("execute_map"):] if "execute_map" in text else text
    for line in body.split("\n"):
        m = re.search(r'\.\{\s*\.name\s*=\s*"([^"]+)"', line)
        if not m:
            continue
        (dead if line.strip().startswith("//") else live).append(m.group(1))
    return live, dead


def parse_command_tokens(text: str) -> list[str]:
    start = text.find("fn parseCommand")
    if start < 0:
        return []
    ends = [i for i in (text.find("\nfn ", start + 1), text.find("\npub fn ", start + 1)) if i > 0]
    end = min(ends) if ends else len(text)
    return re.findall(r'eql\(u8,\s*\w+,\s*"([^"]+)"\)', text[start:end])


def main_chain_tokens(text: str) -> list[str]:
    start, end = text.find("fn main("), text.find("fn dispatchNamespacedCommand")
    seg = text[start:end] if start >= 0 and end > start else text
    return re.findall(r'eql\(u8,\s*[\w\[\]\.]+,\s*"([^"]+)"\)', seg)


def cell_map_keys(text: str) -> list[str]:
    return re.findall(r'\.\{\s*"([^"]+)",\s*', text)


def server_tools(text: str) -> tuple[list[str], list[str], int]:
    start = text.find("const tools_body")
    end = text.find("const footer", start) if start >= 0 else -1
    seg = text[start:end] if start >= 0 and end > start else ""
    names, malformed, prefixes = [], [], 0
    for line in seg.split("\n"):
        s = line.strip()
        if not s.startswith("\\\\"):
            continue
        obj = s[2:].strip().rstrip(",").strip()
        m = re.search(r'"name":"([^"]+)"', obj)
        if not m:
            continue
        names.append(m.group(1))
        try:
            json.loads(obj)
        except json.JSONDecodeError:
            malformed.append(m.group(1))
    hstart = text.find("fn handleToolsCall")
    hend = text.find("\n    fn ", hstart + 1) if hstart >= 0 else -1
    hseg = text[hstart:hend] if hstart >= 0 and hend > hstart else text[hstart:hstart + 20000] if hstart >= 0 else ""
    prefixes = len(set(re.findall(r'startsWith\(u8,\s*\w+,\s*"([^"]+)"\)', hseg)))
    return names, malformed, prefixes


def help_witness(tri: pathlib.Path, cards: list[dict]) -> dict:
    rc, out, err = run([str(tri), "--help"], timeout=60)
    top = re.findall(r"^  ([a-z][a-z0-9-]*)\s", out, re.M)
    names = {c["ID"].split("/", 1)[1] for c in cards}
    shown = str(tri.relative_to(ROOT)) if tri.is_relative_to(ROOT) else tri.name
    result = {"tri": shown, "version": (run([str(tri), "--version"], timeout=60)[1] or "").strip()[:80], "top_level": len(top),
              "in_help_not_carded": sorted(set(top) - names - {"help"}), "carded_not_in_help": sorted(names - set(top)), "actions_mismatch": [], "checked": 0}
    for c in cards:
        cmd = c["COMMAND"].split()[1]
        rc, out, err = run([str(tri), cmd, "--help"], timeout=60)
        text = out + err
        sub = []
        if "Commands:" in text:
            sub = [x for x in re.findall(r"^  ([a-z][a-z0-9-]*)\s", text.split("Commands:")[1].split("Options:")[0], re.M) if x != "help"]
        if sorted(sub) != sorted(c["ACTIONS"]):
            result["actions_mismatch"].append({"command": cmd, "card": sorted(c["ACTIONS"]), "help": sorted(sub)})
        result["checked"] += 1
    result["ok"] = not result["in_help_not_carded"] and not result["carded_not_in_help"] and not result["actions_mismatch"]
    return result


def build_inventory(trinity: pathlib.Path, cards: dict, tri: pathlib.Path | None) -> dict:
    rc, out, _ = run(["git", "-C", str(trinity), "rev-parse", "HEAD"], timeout=60)
    head = out.strip() if rc == 0 else ""
    registry = json.loads(read(trinity, ".trinity/registry.json") or "{}")
    schemas = json.loads(read(trinity, ".trinity/mcp_schemas.json") or "{}")
    table_text = read(trinity, "src/registry/command_table.zig")
    table_names = table_entries(table_text)
    table_mcp = len(re.findall(r"\.mcp_enabled\s*=\s*true", table_text[table_text.find("pub const all_commands"):table_text.find("fn countMcpTools")] if "pub const all_commands" in table_text else table_text))
    live, dead = execute_map(read(trinity, "src/tri/tri_register.zig"))
    tokens = parse_command_tokens(read(trinity, "src/tri/tri_utils.zig"))
    chain = main_chain_tokens(read(trinity, "src/tri/main.zig"))
    cells = cell_map_keys(read(trinity, "src/tri/tri_cell_dispatch.zig"))
    stools, malformed, prefixes = server_tools(read(trinity, "tools/mcp/trinity_mcp/server.zig"))
    needle = re.findall(r'"name":"(needle_[a-z_]+)"', read(trinity, "tools/mcp/needle_mcp/server.zig"))
    rtext = read(trinity, "tools/mcp/trinity_mcp/resources.zig")
    rseg = rtext[rtext.find("pub const resources"):] if "pub const resources" in rtext else rtext
    resources = re.findall(r'\.uri = "([^"]+)"', rseg[:rseg.find("};") + 2] if "};" in rseg else rseg)
    prompts = entry_names_at_indent(read(trinity, "tools/mcp/trinity_mcp/prompts.zig"), 8)
    reg_cmds = registry.get("commands", [])
    reg_names = [c["name"] for c in reg_cmds]
    routed_set = set(live) | set(tokens) | set(chain) | set(cells)
    unrouted = [n for n in reg_names if n not in routed_set]
    route_kind = {n: ("execute_map" if n in set(live) else "parse_command" if n in set(tokens) else "main_chain" if n in set(chain) else "cell_map" if n in set(cells) else "none") for n in reg_names}
    table_unrouted = [n for n in table_names if n not in routed_set]
    schema_names = [t["name"] for t in schemas.get("tools", [])]
    overlap = sorted(set(schema_names) & set(stools))
    t27_names = {c["ID"].split("/", 1)[1] for c in cards["t27_tri"]}
    trinity_names = set(reg_names) | {a for c in reg_cmds for a in c.get("aliases", []) or []}
    collisions = sorted(t27_names & trinity_names)
    inv = {
        "version": 1,
        "generated_by": "tools/trinity_tools_registry.py inventory",
        "at": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "consumer": {"repo": TRINITY_REPO, "revision": head},
        "method": {"registry": ".trinity/registry.json (the binary's export, drift-checked by ci.yml)", "execute_map": "src/tri/tri_register.zig: live and commented-out .name entries",
                   "parse_command": "src/tri/tri_utils.zig parseCommand: string literals compared with eql", "main_chain": "src/tri/main.zig main..dispatchNamespacedCommand: string literals compared with eql", "cell_maps": "src/tri/tri_cell_dispatch.zig: StaticStringMap keys",
                   "server_tools": "tools/mcp/trinity_mcp/server.zig tools_body: one object per line, each parsed alone", "routed": "a registry name is routed when the execute map, parseCommand, the main chain or a cell map names it; whether the arm it reaches executes or prints not-implemented is not measured",
                   "table": "src/registry/command_table.zig: top-level entries of all_commands (single-line entries and the .name line after a multi-line opener)"},
        "registry": {"version": registry.get("version"), "generated_at": registry.get("generated_at"), "count": len(reg_cmds), "names": reg_names, "commands": reg_cmds,
                     "sha256": sha256((trinity / ".trinity/registry.json").read_bytes()) if (trinity / ".trinity/registry.json").exists() else None},
        "mcp_schemas": {"count": len(schema_names), "names": schema_names, "equals_registry_mcp_names": schema_names == [c.get("mcp_name") for c in reg_cmds]},
        "command_table": {"count": len(table_names), "mcp_enabled": table_mcp, "unrouted": table_unrouted, "unrouted_count": len(table_unrouted)},
        "dispatch": {"execute_map_live": len(live), "execute_map_live_distinct": len(set(live)), "execute_map_disabled": dead, "parse_command_tokens": len(tokens), "parse_command_distinct": len(set(tokens)),
                     "main_chain_tokens": len(chain), "main_chain_distinct": len(set(chain)), "cell_map_keys": len(cells)},
        "registry_routing": {"routed": [n for n in reg_names if n in routed_set], "unrouted": unrouted, "route_kind": route_kind},
        "mcp_server": {"static_tools": len(stools), "names": stools, "malformed": malformed, "malformed_count": len(malformed), "prefix_handlers": prefixes,
                       "overlap_with_export": overlap, "overlap_count": len(overlap), "needle_tools": needle, "resources": resources, "prompts": prompts},
        "cards": {"t27_tri": len(cards["t27_tri"]), "mcp": len(cards["mcp"]), "trinity_tri": len(cards["trinity_tri"]), "trinity_tri_names": sorted(c["ID"].split(":tri/")[1] for c in cards["trinity_tri"])},
        "collisions": collisions,
        "help_witness": help_witness(tri, cards["t27_tri"]) if tri else None,
    }
    return inv


# ---------------------------------------------------------------------------
# check: cards, catalog and protocol against the inventory.
# ---------------------------------------------------------------------------
def expected_args(cmd: dict) -> list[str]:
    return ["<%s>: %s%s%s" % (p["name"], p["type"], ", required" if p.get("required") else ", optional", ("; " + p["description"]) if p.get("description") else "") for p in cmd.get("input_params", [])]


def check_cards(cards: dict, inv: dict) -> list[str]:
    f = []
    reg = {c["name"]: c for c in inv["registry"]["commands"]}
    by_name = {}
    for c in cards["trinity_tri"]:
        name = c["ID"].split(":tri/")[1] if ":tri/" in c["ID"] else c["ID"]
        by_name[name] = c
    for n in reg:
        if n not in by_name:
            f.append(f"registry command {n} has no card under specs/tools/trinity/tri/ (unexplained addition in the registry)")
    for n in by_name:
        if n not in reg:
            f.append(f"card {n} names no registry command (unexplained removal from the registry)")
    routed = set(inv["registry_routing"]["routed"])
    for n, c in by_name.items():
        if n not in reg:
            continue
        r = reg[n]
        want = {"ID": f"{TRINITY_REPO}:tri/{n}", "QUALIFIED_ID": f"{TRINITY_REPO}:tri/{n}", "REPO": TRINITY_REPO, "SCHEMA": 2, "FAMILY": "tri-cli", "KIND": "tool", "COMMAND": f"tri {n}",
                "ABOUT": r.get("description", ""), "ALIASES": r.get("aliases", []) or [], "NAMESPACE": r.get("cli_namespace", ""), "MODE": r.get("mode", ""), "STABILITY": r.get("stability", ""),
                "CATEGORY": r.get("category", ""), "JOB_TIMEOUT": int(r.get("job_timeout", 0)), "SIDE_EFFECTS": r.get("side_effects", []) or [], "MCP_ENABLED": bool(r.get("mcp_enabled")),
                "MCP_NAME": r.get("mcp_name", ""), "MCP_DISPLAY_NAME": r.get("mcp_display_name", ""), "EXAMPLES": r.get("examples", []) or [], "ARGS": expected_args(r), "ROUTED": n in routed,
                "WITNESS": "registry-export", "ENABLED": True, "ROUTE_KIND": inv["registry_routing"].get("route_kind", {}).get(n, "none")}
        for k, v in want.items():
            if c.get(k) != v:
                f.append(f"{n}: {k} is {c.get(k)!r}, the registry says {v!r}")
        if c.get("COLLIDES_WITH", "") != (f"{T27_REPO}:tri/{n}" if n in inv["collisions"] else ""):
            f.append(f"{n}: COLLIDES_WITH is {c.get('COLLIDES_WITH')!r}, the inventory says {'collision' if n in inv['collisions'] else 'none'}")
    for kind, repo in (("t27_tri", T27_REPO), ("mcp", None)):
        for c in cards[kind]:
            path = c["_path"].split("specs/")[-1]
            if c.get("SCHEMA") != 2:
                f.append(f"{path}: SCHEMA must be 2")
            if repo and c.get("REPO") != repo:
                f.append(f"{path}: REPO must be {repo}")
            if not c.get("REPO"):
                f.append(f"{path}: REPO missing")
            if c.get("QUALIFIED_ID") != f"{c.get('REPO')}:{c.get('ID')}":
                f.append(f"{path}: QUALIFIED_ID must be REPO:ID")
            if ":" in c.get("ID", ""):
                f.append(f"{path}: a legacy card keeps its short ID")
    return f


def check_catalog(catalog: dict, cards: dict, inv: dict) -> list[str]:
    f = []
    expect = {"SCHEMA_VERSION": 2, "T27_TRI_CARDS": inv["cards"]["t27_tri"], "MCP_CARDS": inv["cards"]["mcp"], "TRINITY_TRI_CARDS": inv["cards"]["trinity_tri"],
              "TRINITY_TABLE_COMMANDS": inv["command_table"]["count"], "TRINITY_TABLE_MCP_ENABLED": inv["command_table"]["mcp_enabled"], "TRINITY_EXPORTED": inv["registry"]["count"],
              "TRINITY_EXPORTED_UNROUTED": len(inv["registry_routing"]["unrouted"]), "TRINITY_TABLE_UNROUTED": inv["command_table"]["unrouted_count"],
              "TRINITY_MCP_SERVER_TOOLS": inv["mcp_server"]["static_tools"], "TRINITY_MCP_SERVER_MALFORMED": inv["mcp_server"]["malformed_count"], "TRINITY_MCP_OVERLAP": inv["mcp_server"]["overlap_count"],
              "TRINITY_EXECUTE_MAP_LIVE": inv["dispatch"]["execute_map_live"], "TRINITY_EXECUTE_MAP_DISABLED": len(inv["dispatch"]["execute_map_disabled"]), "TRINITY_PARSE_TOKENS": inv["dispatch"]["parse_command_tokens"],
              "COLLISIONS": inv["collisions"], "UNROUTED_EXPORTED": inv["registry_routing"]["unrouted"]}
    for k, v in expect.items():
        if catalog.get(k) != v:
            f.append(f"catalog {k} is {catalog.get(k)!r}, the inventory measures {v!r}")
    legacy = {c["ID"]: c["QUALIFIED_ID"] for c in cards["t27_tri"] + cards["mcp"]}
    table = dict(zip(catalog.get("LEGACY_IDS", []), catalog.get("LEGACY_TARGETS", [])))
    if table != legacy:
        missing = sorted(set(legacy) - set(table)); extra = sorted(set(table) - set(legacy)); wrong = sorted(k for k in table if k in legacy and table[k] != legacy[k])
        f.append(f"catalog LEGACY_IDS/LEGACY_TARGETS drift: missing {missing[:3]}, extra {extra[:3]}, wrong {wrong[:3]}")
    if inv.get("help_witness") and not inv["help_witness"]["ok"]:
        f.append("help witness: the t27 tri --help disagrees with the t27 cards: " + json.dumps({k: inv["help_witness"][k] for k in ("in_help_not_carded", "carded_not_in_help")}) + f", {len(inv['help_witness']['actions_mismatch'])} action mismatch(es)")
    return f


def check_protocol(proto: dict, inv: dict) -> list[str]:
    f = []
    ms = inv["mcp_server"]
    expect = {"TOOLS_STATIC": ms["static_tools"], "TOOLS_MALFORMED": ms["malformed_count"], "EXPORTED_TOOLS": inv["mcp_schemas"]["count"], "CALL_PREFIX_HANDLERS": ms["prefix_handlers"],
              "RESOURCE_COUNT": len(ms["resources"]), "PROMPT_COUNT": len(ms["prompts"]), "NEEDLE_TOOLS": ms["needle_tools"], "RESOURCES": ms["resources"], "PROMPTS": ms["prompts"]}
    for k, v in expect.items():
        if proto.get(k) != v:
            f.append(f"protocol {k} is {proto.get(k)!r}, the inventory measures {v!r}")
    if sorted(proto.get("CALL_PREFIXES", [])) != sorted(proto.get("CALL_PREFIXES", [])) or len(proto.get("CALL_PREFIXES", [])) != proto.get("CALL_PREFIX_HANDLERS"):
        f.append("protocol CALL_PREFIXES does not hold CALL_PREFIX_HANDLERS entries")
    return f


def load_spec_dict(path: pathlib.Path) -> dict:
    return load_card(path)


# ---------------------------------------------------------------------------
# fixtures: the server's answers, modelled; replayed through the generated C.
# ---------------------------------------------------------------------------
METHODS = ["initialize", "tools/list", "tools/call", "resources/list", "resources/read", "prompts/list", "prompts/get"]
EXACT_TOOLS = {"tri_execute", "tri_status", "tri_gen", "tri_spec_create", "tri_constants", "tri_phi", "tri_fib", "tri_lucas", "tri_bench", "tri_verdict", "tri_notify"}
PREFIXES = ["needle_", "swarm_", "chain_", "cloud_", "farm_evolve_", "fpga_", "doctor_", "job_", "issue_", "deploy_", "experience_", "patent_", "depin_", "research_", "experiment_", "chimera_", "ouroboros_", "self_", "context_", "faculty_", "mu_", "zenodo_", "farm_", "oracle_", "tri_train_"]


def classify(request: str, tool: str | None) -> dict:
    is_notification = "notifications/" in request
    is_known = any(m in request for m in METHODS)
    exact = tool in EXACT_TOOLS if tool else False
    prefix = any(tool.startswith(p) for p in PREFIXES) if tool else False
    return {"is_known_method": is_known, "is_notification": is_notification, "tool_exact": exact, "tool_prefix": prefix}


def build_fixtures() -> dict:
    F = []
    def fx(fid, request, note, tool=None, expect_error=False, permission=None, client=None, needle=None):
        c = classify(request, tool)
        reply = 1 if c["is_notification"] else (0 if c["is_known_method"] else 2)
        route = None if tool is None else (0 if c["tool_exact"] else 1 if c["tool_prefix"] else 2)
        e = {"reply": reply, "reply_name": ["answered", "silent notification", "silent unknown method"][reply], "route": route,
             "route_name": None if route is None else ["exact handler", "prefix handler", "generic tri executor"][route], "refused": False if tool else None,
             "error_as_result": expect_error, "jsonrpc_error_object": False}
        if permission is not None:
            e["client_permission"] = permission
        if needle is not None:
            e["needle_mcp"] = needle
        F.append({"id": fid, "request": request, "classification": c, "tool": tool, "expect": e, "note": note, "client": client})
    fx("initialize", '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"fixture","version":"0"}}}',
       "answered with the literal initialize result; the requested protocolVersion 2025-06-18 is ignored and 2024-11-05 comes back")
    fx("tools-list", '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}', "answered with the static literal of 210 objects; the body is not parseable JSON (32 malformed objects, one closing brace too many)")
    fx("tools-call-exact", '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"tri_status","arguments":{}}}', "an exact handler: spawns zig-out/bin/tri status", tool="tri_status")
    fx("tools-call-prefix", '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"needle_search","arguments":{"query":"phi"}}}', "a prefix handler (needle_)", tool="needle_search")
    fx("tools-call-missing-required", '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"needle_structural_replace","arguments":{}}}',
       "malformed parameters: the required file_path is absent; the answer is a result with isError true and the text Error: Missing file_path, not a -32602", tool="needle_structural_replace", expect_error=True)
    fx("tools-call-unknown-tool", '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"no_such_tool","arguments":{}}}',
       "an unknown tool is not refused: the name is stripped of tri_ and run as `tri no_such_tool` with no arguments through the generic executor", tool="no_such_tool", needle={"error_code": -32601, "message": "Unknown tool"})
    fx("tools-call-fallback-advertised", '{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"tri_pipeline","arguments":{"task":"x"}}}',
       "an advertised tool with no handler of its own: arguments are dropped and `tri pipeline` runs bare", tool="tri_pipeline")
    fx("cancellation", '{"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":3,"reason":"user"}}', "swallowed: no reply, and the child process of request 3 keeps running (cancellation.zig is not imported)")
    fx("initialized-notification", '{"jsonrpc":"2.0","method":"notifications/initialized"}', "swallowed, as every notification")
    fx("unknown-method", '{"jsonrpc":"2.0","id":8,"method":"ping"}', "no reply at all; method_not_found (-32601) exists in errors.zig and is never sent")
    fx("transport-garbage", 'not json at all', "no method substring matches: no reply; parse_error (-32700) is never sent")
    fx("denied-capability-at-the-client", '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"trinity.tri_notify","arguments":{"message":"hi"}}}',
       "the only permission gate in the tree is tri-api's: with no allow rule for tri_notify the default denies it, the executor returns Permission denied with is_error and the server never sees the call", tool="tri_notify",
       permission={"has_deny_rule": False, "has_allow_rule": False, "default_allowed": False, "decision": 1, "decision_name": "deny"}, client="src/tri-api/tool_executor.zig:145-171 with src/tri-api/permissions.zig")
    fx("allowed-capability-at-the-client", '{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"trinity.tri_status","arguments":{}}}',
       "an allow rule tool(tri_status) lets the call reach the server", tool="tri_status", permission={"has_deny_rule": False, "has_allow_rule": True, "default_allowed": False, "decision": 0, "decision_name": "allow"}, client="src/tri-api/permissions.zig")
    fx("deny-beats-allow-at-the-client", '{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"trinity.tri_status","arguments":{}}}',
       "a deny rule wins over an allow rule for the same tool", tool="tri_status", permission={"has_deny_rule": True, "has_allow_rule": True, "default_allowed": True, "decision": 1, "decision_name": "deny"}, client="src/tri-api/permissions.zig")
    return {"version": 1, "generated_by": "tools/trinity_tools_registry.py fixtures", "spec": "specs/tools/mcp_protocol.t27",
            "consumer": {"repo": TRINITY_REPO, "server": "tools/mcp/trinity_mcp/server.zig", "client": "src/tri-api/mcp_client.zig", "needle_server": "tools/mcp/needle_mcp/server.zig"},
            "description": "Offline protocol fixtures: what the pinned trinity-mcp does with each request, modelled from server.zig (substring method routing, exact/prefix/generic tool routing, failures as isError results, notifications and unknown methods answered with silence) and the client-side permission gate of tri-api. Not run against a server: no workflow runs one and no Zig on this host builds it.",
            "reply_codes": {"0": "answered", "1": "silent notification", "2": "silent unknown method"}, "route_codes": {"0": "exact handler", "1": "prefix handler", "2": "generic tri executor"},
            "exact_tools_modelled": sorted(EXACT_TOOLS), "prefixes": PREFIXES, "fixtures": F}


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


def prototypes(c_text: str) -> list[str]:
    return [l.strip() for l in c_text.split("\n") if re.match(r"^(?:u?int\d+_t|bool|double|float|void)\s+[a-z_]\w*\(.*\);\s*$", l)]


def driver_case(fx: dict) -> str:
    c, e = fx["classification"], fx["expect"]
    b = lambda v: "true" if v else "false"
    lines = ["    { const char *fail = NULL;"]
    lines.append(f'if (method_reply({b(c["is_known_method"])}, {b(c["is_notification"])}) != {e["reply"]}u) fail = "reply";')
    if fx["tool"] is not None:
        lines.append(f'if (!fail && call_route({b(c["tool_exact"])}, {b(c["tool_prefix"])}) != {e["route"]}u) fail = "route";')
        lines.append(f'if (!fail && call_is_refused({b(c["tool_exact"])}, {b(c["tool_prefix"])}) != {b(e["refused"])}) fail = "refused";')
    lines.append(f'if (!fail && failure_is_result({b(e["error_as_result"])}) != {b(e["error_as_result"])}) fail = "error channel";')
    lines.append('if (!fail && error_code_is_emitted_by_trinity_mcp(2) != false) fail = "trinity-mcp emits a JSON-RPC error";')
    if e.get("needle_mcp"):
        lines.append(f'if (!fail && (error_code(2) != {e["needle_mcp"]["error_code"]} || !error_code_is_emitted_by_needle_mcp(2))) fail = "needle-mcp unknown tool code";')
    if e.get("client_permission"):
        p = e["client_permission"]
        lines.append(f'if (!fail && permission({b(p["has_deny_rule"])}, {b(p["has_allow_rule"])}, {b(p["default_allowed"])}) != {p["decision"]}u) fail = "permission";')
    lines.append(f'verdict({json.dumps(fx["id"])}, fail); }}')
    return "\n    ".join(lines)


def replay(spec: pathlib.Path, fixtures: dict, t27c: pathlib.Path, work: pathlib.Path) -> dict:
    work.mkdir(parents=True, exist_ok=True)
    rc, out, err = run([str(t27c), "gen-c", str(spec)])
    if rc != 0 or not out.strip():
        return {"ok": False, "stage": "generate", "detail": (err or out).strip()[:300], "passed": 0, "failed": 0, "failures": []}
    (work / "spec.c").write_text(out, encoding="utf-8")
    driver = DRIVER.replace("@PROTOTYPES@", "\n".join(prototypes(out))).replace("@CASES@", "\n".join(driver_case(f) for f in fixtures["fixtures"]))
    (work / "driver.c").write_text(driver, encoding="utf-8")
    rc, out2, err = run(["cc", "-std=c11", "-w", "-c", "-x", "c", str(work / "spec.c"), "-o", str(work / "spec.o")])
    if rc != 0:
        return {"ok": False, "stage": "compile", "detail": err.strip()[:400], "passed": 0, "failed": 0, "failures": []}
    rc, out2, err = run(["cc", "-std=c11", "-w", "-c", "-x", "c", str(work / "driver.c"), "-o", str(work / "driver.o")])
    if rc != 0:
        return {"ok": False, "stage": "compile", "detail": "driver: " + err.strip()[:400], "passed": 0, "failed": 0, "failures": []}
    rc, out2, err = run(["cc", str(work / "driver.o"), str(work / "spec.o"), "-o", str(work / "driver")])
    if rc != 0:
        return {"ok": False, "stage": "link", "detail": err.strip()[:400], "passed": 0, "failed": 0, "failures": []}
    rc, out2, err = run([str(work / "driver")], timeout=60)
    results = re.findall(r"^\[FIX\] (.+?) : (PASSED|FAILED)(?: \((.*)\))?$", out2, re.M)
    failures = [{"id": i, "detail": d or ""} for i, v, d in results if v == "FAILED"]
    passed = sum(1 for _, v, _ in results if v == "PASSED")
    ok = rc == 0 and not failures and passed == len(fixtures["fixtures"])
    return {"ok": ok, "stage": "runtime", "detail": "" if results else (err or out2)[-200:], "passed": passed, "failed": len(failures), "failures": failures, "generated": sha256(out.encode())}


def replay_record(spec: pathlib.Path, fixtures: dict, t27c: pathlib.Path) -> dict:
    with tempfile.TemporaryDirectory() as tmp:
        r = replay(spec, fixtures, t27c, pathlib.Path(tmp))
    r["at"] = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    r["t27c"] = (run([str(t27c), "--version"], timeout=60)[1] or "t27c").strip().split("\n")[0][:80]
    r["host"] = f"{platform.system()} {platform.machine()}"
    r["spec_sha256"] = sha256(spec.read_bytes())
    r["fixtures"] = len(fixtures["fixtures"])
    return r


def check_fixtures(fixtures: dict) -> list[str]:
    f = []
    fresh = build_fixtures()
    if json.dumps(fixtures.get("fixtures"), sort_keys=True) != json.dumps(fresh["fixtures"], sort_keys=True):
        f.append("fixtures: the file drifts from what `fixtures` writes now")
    r = fixtures.get("replay")
    if not isinstance(r, dict):
        f.append("fixtures: no replay; run `run`")
        return f
    if r.get("spec_sha256") != sha256(PROTOCOL.read_bytes()):
        f.append("fixtures: the protocol spec changed since the replay; run `run`")
    if not r.get("ok") or r.get("failed", 1) != 0 or r.get("passed") != len(fixtures.get("fixtures", [])):
        f.append(f"fixtures: replay {r.get('passed')} passed, {r.get('failed')} failed at stage {r.get('stage')}: {r.get('detail', '')[:200]}")
    return f


# ---------------------------------------------------------------------------
# --self-check
# ---------------------------------------------------------------------------
def self_check(trinity: pathlib.Path | None) -> int:
    ok = True

    def expect(cond: bool, what: str):
        nonlocal ok
        print(f"  {'ok ' if cond else 'BAD'} {what}")
        ok = ok and cond

    inv = json.loads(INVENTORY.read_text()) if INVENTORY.exists() else None
    expect(inv is not None, "the committed inventory exists")
    cards = load_cards(TOOLS_DIR)
    catalog = load_spec_dict(CATALOG) if CATALOG.exists() else {}
    proto = load_spec_dict(PROTOCOL)
    if inv:
        f = check_cards(cards, inv) + check_catalog(catalog, cards, inv) + check_protocol(proto, inv)
        expect(not f, f"check: the cards, the catalog and the protocol match the committed inventory ({len(f)} finding(s){': ' + f[0] if f else ''})")
        planted = json.loads(json.dumps(inv))
        planted["registry"]["commands"].append({"name": "planted", "description": "x", "mcp_enabled": True, "mcp_name": "tri_planted"})
        planted["registry"]["names"].append("planted")
        expect(any("planted has no card" in x for x in check_cards(cards, planted)), "planted: a registry command without a card is an unexplained addition")
        planted = json.loads(json.dumps(inv))
        planted["registry"]["commands"] = [c for c in planted["registry"]["commands"] if c["name"] != "bio"]
        expect(any("card bio names no registry command" in x for x in check_cards(cards, planted)), "planted: a card without a registry command is an unexplained removal")
        planted = json.loads(json.dumps(inv))
        for c in planted["registry"]["commands"]:
            if c["name"] == "phi":
                c["description"] = "changed"
        expect(any(x.startswith("phi: ABOUT") for x in check_cards(cards, planted)), "planted: a changed registry description is reported on its card")
        planted = json.loads(json.dumps(inv))
        planted["mcp_server"]["static_tools"] = 211
        expect(any("TOOLS_STATIC" in x for x in check_protocol(proto, planted)), "planted: a changed tool count is reported against the protocol spec")
        planted = json.loads(json.dumps(inv))
        planted["collisions"] = ["fpga"]
        expect(any("COLLISIONS" in x for x in check_catalog(catalog, cards, planted)), "planted: a changed collision set is reported against the catalog")
        bad_cards = json.loads(json.dumps({k: v for k, v in cards.items()}))
        bad_cards["t27_tri"][0]["QUALIFIED_ID"] = "wrong"
        expect(any("QUALIFIED_ID must be REPO:ID" in x for x in check_cards(bad_cards, inv)), "planted: a legacy card whose QUALIFIED_ID is not REPO:ID is reported")
    t27c = t27c_path()
    if t27c and shutil.which("cc"):
        fixtures = build_fixtures()
        with tempfile.TemporaryDirectory() as tmp:
            r = replay(PROTOCOL, fixtures, t27c, pathlib.Path(tmp) / "good")
            expect(r["ok"], f"replay: the protocol spec passes every fixture ({r['passed']} passed, {r['failed']} failed{', ' + r['detail'][:120] if r['detail'] else ''})")
            bad = json.loads(json.dumps(fixtures))
            bad["fixtures"][2]["expect"]["route"] = 2
            r = replay(PROTOCOL, bad, t27c, pathlib.Path(tmp) / "bad")
            expect(r["failed"] == 1 and r["failures"][0]["id"] == "tools-call-exact", "planted: a wrong expected route fails exactly its fixture")
            text = PROTOCOL.read_text(encoding="utf-8")
            planted = text.replace("pub fn call_is_refused(exact: bool, prefix: bool) -> bool {\n    return false;\n}", "pub fn call_is_refused(exact: bool, prefix: bool) -> bool {\n    return exact == false and prefix == false;\n}")
            expect(planted != text, "planted: the refusal rule was found to replace")
            pp = pathlib.Path(tmp) / "mcp_protocol.t27"
            pp.write_text(planted, encoding="utf-8")
            r = replay(pp, fixtures, t27c, pathlib.Path(tmp) / "spec")
            ids = sorted(x["id"] for x in r["failures"])
            expect(r["failed"] >= 1 and all("unknown" in i or "fallback" in i for i in ids), f"planted: a server that refused unknown tools fails only the unknown-tool fixtures ({ids})")
    else:
        print("  skip replay checks: t27c or cc missing")
    if trinity:
        cards_now = load_cards(TOOLS_DIR)
        inv2 = build_inventory(trinity, cards_now, None)
        expect(inv2["registry"]["count"] == inv["registry"]["count"] if inv else True, "inventory: a fresh measurement of the clone agrees with the committed count")
    print("trinity_tools_registry --self-check:", "ok" if ok else "FAILED")
    return 0 if ok else 1


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("command", nargs="?", choices=["inventory", "check", "fixtures", "run"])
    ap.add_argument("--trinity-root")
    ap.add_argument("--tri", help="the t27 Rust tri binary for the help-output witness of the 52 t27 cards")
    ap.add_argument("--inventory", default=str(INVENTORY))
    ap.add_argument("--fixtures", default=str(FIXTURES))
    ap.add_argument("--self-check", action="store_true")
    a = ap.parse_args()
    if a.self_check:
        return self_check(pathlib.Path(a.trinity_root).resolve() if a.trinity_root else None)
    if a.command is None:
        ap.print_help()
        return 2
    cards = load_cards(TOOLS_DIR)
    if a.command == "inventory":
        if not a.trinity_root:
            print("inventory: --trinity-root is required", file=sys.stderr)
            return 2
        trinity = pathlib.Path(a.trinity_root).resolve()
        tri = pathlib.Path(a.tri).resolve() if a.tri else None
        inv = build_inventory(trinity, cards, tri)
        pathlib.Path(a.inventory).write_text(json.dumps(inv, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        ms = inv["mcp_server"]
        hw = inv["help_witness"]
        print(f"inventory: registry {inv['registry']['count']} (table {inv['command_table']['count']}, mcp_enabled {inv['command_table']['mcp_enabled']}), unrouted exported {len(inv['registry_routing']['unrouted'])}, table unrouted {inv['command_table']['unrouted_count']}; "
              f"server tools {ms['static_tools']} ({ms['malformed_count']} malformed, {ms['prefix_handlers']} prefixes, overlap {ms['overlap_count']}); cards {inv['cards']}; collisions {inv['collisions']}; help witness {'ok' if hw and hw['ok'] else ('not run' if not hw else 'MISMATCH')}")
        return 0
    if a.command == "fixtures":
        fx = build_fixtures()
        out = pathlib.Path(a.fixtures)
        if out.exists():
            old = json.loads(out.read_text())
            if "replay" in old:
                fx["replay"] = old["replay"]
        out.write_text(json.dumps(fx, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"wrote {out}: {len(fx['fixtures'])} fixtures")
        return 0
    if a.command == "run":
        t27c = t27c_path()
        if t27c is None or shutil.which("cc") is None:
            print("run: needs t27c and cc", file=sys.stderr)
            return 2
        fp = pathlib.Path(a.fixtures)
        if not fp.exists():
            print("run: no fixtures; run `fixtures` first", file=sys.stderr)
            return 2
        fx = json.loads(fp.read_text())
        fx["replay"] = replay_record(PROTOCOL, fx, t27c)
        fp.write_text(json.dumps(fx, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        r = fx["replay"]
        print(f"replay: {r['passed']} passed, {r['failed']} failed of {r['fixtures']} fixtures (stage {r['stage']}) with {r['t27c']}")
        for x in r["failures"][:10]:
            print(f"  FAILED {x['id']}: {x['detail']}")
        if r["detail"] and not r["ok"]:
            print(f"  {r['detail'][:600]}")
        return 0 if r["ok"] else 1
    ip = pathlib.Path(a.inventory)
    if not ip.exists():
        print("check: no inventory; run `inventory` first", file=sys.stderr)
        return 2
    inv = json.loads(ip.read_text())
    findings = []
    try:
        catalog = load_spec_dict(CATALOG)
        proto = load_spec_dict(PROTOCOL)
    except (ValueError, FileNotFoundError) as e:
        print(f"finding: {e}")
        return 1
    findings += check_cards(cards, inv) + check_catalog(catalog, cards, inv) + check_protocol(proto, inv)
    if pathlib.Path(a.fixtures).exists():
        findings += check_fixtures(json.loads(pathlib.Path(a.fixtures).read_text()))
    else:
        findings.append("fixtures: conformance/trinity/mcp_fixtures.json missing")
    for x in findings:
        print(f"finding: {x}")
    print(f"check: {inv['cards']['trinity_tri']} Trinity cards against a registry of {inv['registry']['count']}, {inv['cards']['t27_tri'] + inv['cards']['mcp']} legacy cards at schema 2, catalog and protocol counts; {len(findings)} finding(s)")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
