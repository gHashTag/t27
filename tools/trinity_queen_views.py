#!/usr/bin/env python3
"""The Queen's project views -- identity, evidence, snapshot and live -- held to specs/queen/views.t27,
by running the site's own code.

WHY THIS EXISTS
---------------
S10 of gHashTag/trinity#988 (gHashTag/t27#4829, re-filed from #3572). t27.ai (gHashTag/trinity
apps/website) shows specs, skills, crons, agents, tools, documents and a hive of issues; every number on
those views comes either from a dated file the build ships or from a public endpoint of the trios
supervisor. specs/queen/views.t27 states the contract between those sources and the views. This tool
measures the site at its pin and holds the spec to it.

SIX KINDS OF EVIDENCE, KEPT APART
---------------------------------
  ts       the site's pure TypeScript (catalog, shared core, atlas, HUD, live-cell and paint helpers)
           imported under bun 1.3.6 and called on fixtures
  data     the public files of the pinned tree, read whole: manifest, shared core, atlas, foundation,
           the vendored mirror against t27 at the revision the manifest names
  live     GitHub on the day: the states of the issues the atlas froze, the epic's sub-issues, the
           re-filed work packages, one answer of each public supervisor endpoint (a snapshot, labelled
           so; never asserted to stay true)
  gates    every check, audit and test script of the site: whether a workflow runs it, and what it
           says here at the pin
  browser  the built site in headless Chrome over CDP, desktop, phone and reduced motion, with every
           live endpoint answered from the live record and every other network host refused
  native   the Swift package apps/queen built and tested at the pin
No credential is used: GitHub is read through `gh` with the public API, the supervisor endpoints are
public, and the browser reaches nothing outside the machine.

Every decision the spec states is replayed through the C and the Zig the compiler generates from it,
over cases whose inputs come from the records; each case also carries what the site did, and a case
where the site and the contract part is a FINDING the spec names, or the check fails.

Usage:
  python3 tools/trinity_queen_views.py ts      --trinity-root <checkout at PIN> [--bun <bun>]
  python3 tools/trinity_queen_views.py data    --trinity-root <checkout>
  python3 tools/trinity_queen_views.py live    --trinity-root <checkout>
  python3 tools/trinity_queen_views.py gates   --trinity-root <checkout> [--chrome <path>]
  python3 tools/trinity_queen_views.py browser --trinity-root <checkout> [--chrome <path>]
  python3 tools/trinity_queen_views.py native  --trinity-root <checkout>
  python3 tools/trinity_queen_views.py run     [--zig <zig> --sysroot <dir>]
  python3 tools/trinity_queen_views.py check
  python3 tools/trinity_queen_views.py --self-check

The site needs node_modules (npm ci) and a build (vite build) for `gates` and `browser`; both are made
here if absent. Exit codes: 0 no finding; 1 findings; 2 could not run.
"""
from __future__ import annotations

import argparse
import collections
import datetime
import hashlib
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.request

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import trinity_tri_api as base  # noqa: E402  (shared helpers: run, sha256, load_spec, replay_spec, now, host)

ROOT = base.ROOT
SPEC = ROOT / "specs/queen/views.t27"
PROJECT_SPEC = ROOT / "specs/trinity/project.t27"
EVIDENCE_SPEC = ROOT / "specs/ui/queen_evidence.t27"
OUT = ROOT / "conformance/trinity"
KINDS = ("ts", "data", "live", "gates", "browser", "native")
REC = {k: OUT / f"queen_views_{k}.json" for k in KINDS}
TRINITY_PIN = "afc9d38435ad01c48af6c33c311da422c38f046e"
SITE = "apps/website"
LIVE_BASE = "https://trios-agent-server-production.up.railway.app"
LIVE_PATHS = ("status", "public-board", "public-activity", "public-research", "public-hardware", "public-foundation", "public-modules")
EPIC = ("gHashTag/trinity", 988)
REFILED = {"S08": ("gHashTag/t27", 4827), "S09": ("gHashTag/t27", 4828), "S10": ("gHashTag/t27", 4829), "S11": ("gHashTag/t27", 4830)}
CHROMES = ["/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", "/Applications/Chromium.app/Contents/MacOS/Chromium",
           "/usr/bin/google-chrome", "/usr/bin/chromium", "/usr/bin/chromium-browser"]
BUN_DEFAULT = pathlib.Path(os.environ.get("BUN", "bun"))


def wr(path, doc):
    pathlib.Path(path).write_text(json.dumps(doc, indent=1, sort_keys=True, ensure_ascii=True) + "\n", encoding="utf-8")


def load(key):
    return base.load_json(REC[key])


def hexsha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def checkout_facts(trinity: pathlib.Path) -> dict:
    head = subprocess.run(["git", "-C", str(trinity), "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
    dirty = subprocess.run(["git", "-C", str(trinity), "status", "--porcelain", "--", SITE, "apps/queen", ".github"], capture_output=True, text=True).stdout
    tracked_dirty = [line for line in dirty.splitlines() if not line.startswith("??")]
    return {"head": head, "tracked_changes": tracked_dirty[:20]}


def need_pin(trinity: pathlib.Path) -> dict:
    facts = checkout_facts(trinity)
    if facts["head"] != TRINITY_PIN:
        raise RuntimeError(f"the checkout is at {facts['head'][:12]}, the pin is {TRINITY_PIN[:12]}")
    if facts["tracked_changes"]:
        raise RuntimeError(f"tracked changes in the checkout: {facts['tracked_changes'][:3]}")
    return facts


def find_chrome(arg=None):
    for c in [arg, os.environ.get("CHROME_PATH")] + CHROMES:
        if c and pathlib.Path(c).exists():
            return c
    return None


def tool_version(cmd) -> str:
    rc, o, e = base.run(cmd, timeout=60)
    return (o or e).strip().split("\n")[0][:120]


# ===========================================================================================
# TS: the site's pure functions under bun
# ===========================================================================================
PROBE_LIBS = r"""
import {createHash} from 'node:crypto';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
const SITE = process.argv[2];
const lib = async (p) => import(pathToFileURL(join(SITE, p)).href);
const {specExplorerHash, resolveManifestSpec} = await lib('src/lib/specCatalog.ts');
const {buildCore, matchIssue} = await lib('src/lib/sharedSpecCore.ts');
const {buildAtlas, validateAtlas} = await lib('src/lib/queenUniverseAtlas.ts');
const hud = await lib('src/components/queenHud.ts');
globalThis.__T27_WASM_TAG__ = '0'.repeat(16);
const {loadSpecSource} = await lib('src/lib/t27Compiler.ts');
const {readBoardCell, readActivityRows} = await lib('src/lib/queenIssueLive.ts');
const {hiveTaskPaint} = await lib('src/components/queenHiveDisplay.ts');
const sha = (s) => createHash('sha256').update(s).digest('hex');
const out = [];
const outcome = (fn) => { try { const v = fn(); return {ok: true, value: v === undefined ? null : v}; } catch (e) { return {ok: false, error: String(e.message ?? e)}; } };
const outcomeAsync = async (fn) => { try { const v = await fn(); return {ok: true, value: v === undefined ? null : v}; } catch (e) { return {ok: false, error: String(e.message ?? e)}; } };
const rec = (id, group, input, result) => out.push({id, group, input, ...result});

// An explicit address. Manifest entries carry path and repo and no sha256, as the shipped manifest does.
const entry = (path, repo = 't27', featured = false) => ({path, repo, featured, name: path.split('/').pop().replace(/\.t27$/, ''), category: path.split('/').slice(0, -1).join('/')});
const manifest = {specs: [entry('specs/demos/hello_world.t27', 't27', true), entry('specs/queen/dispatch.t27'), entry('trinity/specs/queen/dispatch.t27', 'trinity'), entry('specs/x.t27')]};
for (const [id, path] of [['r01', 'specs/queen/dispatch.t27'], ['r02', 'specs/queen/missing.t27'], ['r03', 'dispatch.t27'], ['r04', 'queen/dispatch.t27'],
    ['r05', 'specs/queen/Dispatch.t27'], ['r06', null], ['r07', '../specs/x.t27'], ['r08', 'specs//x.t27'], ['r09', 'specs/x.t27#y'], ['r10', 'trinity/specs/queen/dispatch.t27']]) {
  rec(id, 'resolve', {path}, outcome(() => resolveManifestSpec(manifest, path).path));
}
rec('r11', 'resolve', {path: 'specs/x.t27', entries_with_that_path: 2}, outcome(() => resolveManifestSpec({specs: [entry('specs/x.t27'), entry('specs/x.t27')]}, 'specs/x.t27').path));
rec('r12', 'resolve', {path: null, catalog: 'empty'}, outcome(() => resolveManifestSpec({specs: []}, null).path));
rec('r13', 'resolve', {path: null, featured: 'none'}, outcome(() => resolveManifestSpec({specs: [entry('specs/b.t27'), entry('specs/a.t27')]}, null).path));
for (const [id, sha256] of [['h01', 'a'.repeat(64)], ['h02', 'A'.repeat(64)], ['h03', 'a'.repeat(63)], ['h04', '']]) {
  rec(id, 'address', {sha256}, outcome(() => specExplorerHash('specs/x.t27', {sha256})));
}
// The bytes against the address's sha256, fetch stubbed.
const bytes = Buffer.from('module x;\n');
globalThis.fetch = async (url) => {
  if (url === 't27/manifest.json') return new Response(JSON.stringify(manifest));
  if (String(url).startsWith('t27/files/')) return new Response(bytes);
  return new Response('', {status: 404});
};
rec('b01', 'bytes', {sha256: 'of these bytes'}, await outcomeAsync(() => loadSpecSource('specs/x.t27', sha(bytes))));
rec('b02', 'bytes', {sha256: 'of other bytes'}, await outcomeAsync(() => loadSpecSource('specs/x.t27', sha('other'))));
rec('b03', 'bytes', {sha256: null}, await outcomeAsync(() => loadSpecSource('specs/x.t27')));
rec('b04', 'bytes', {path: 'x.t27', sha256: null}, await outcomeAsync(() => loadSpecSource('x.t27')));
// Identity in the shared core: the hash of the bytes; sources are repository plus full path.
const src = (repo, path, text) => ({repo, path, text, hash: sha(text)});
const core1 = outcome(() => buildCore([src('gHashTag/t27', 'specs/x.t27', 'module x;'), src('gHashTag/trinity', 'trinity/specs/x.t27', 'module x;')], {}));
rec('c01', 'core', {same_bytes_in_two_repositories: true}, core1.ok ? {ok: true, value: {specs: core1.value.specs.length, sources: core1.value.specs.map(s => s.sources.length)}} : core1);
rec('c02', 'core', {one_source_twice: true}, outcome(() => buildCore([src('gHashTag/t27', 'specs/x.t27', 'a'), src('gHashTag/t27', 'specs/x.t27', 'b')], {}).specs.length));
const core3 = buildCore([src('gHashTag/t27', 'specs/x.t27', 'module x; // t27'), src('gHashTag/trinity', 'trinity/specs/x.t27', 'module x; // trinity'), src('gHashTag/t27', 'specs/queen/dispatch.t27', 'pub fn claim_of() {}')], {});
const idOf = (repo, path) => core3.specs.find(s => s.sources.some(x => x.repo === repo.toLowerCase() && x.path === path)).id;
const relOf = (hits, repo, path) => (hits.find(h => h.specId === idOf(repo, path)) ?? {relation: 'none'}).relation;
for (const [id, body] of [['m01', 'See `x.t27` for the shape.'], ['m02', 'See `specs/x.t27` for the shape.'], ['m03', 'See `trinity/specs/x.t27` for the shape.'], ['m04', 'Nothing about any file here.']]) {
  const hits = matchIssue(core3, {repo: 'ghashtag/t27', number: 1, title: 'Fix the parser', body});
  rec(id, 'match', {issue: 'ghashtag/t27#1', body}, {ok: true, value: {t27: relOf(hits, 'gHashTag/t27', 'specs/x.t27'), trinity: relOf(hits, 'gHashTag/trinity', 'trinity/specs/x.t27'), coverage: [...new Set(hits.map(h => h.coverage))]}});
}
// The atlas: open issues only, per-world counts.
const atlasCore = {...core1.value, provenance: {manifestSha256: 'a'.repeat(64), indexerSha256: 'b'.repeat(64)}};
const world = (repo) => ({repo, owner: repo.split('/')[0], description: '', archived: false, issuesEnabled: true, strand: 'bridge', evidence: [{kind: 'seed', url: `https://github.com/${repo}`, detail: 'x'}]});
const snap = (repo, issues) => ({repo, observedAt: '2026-09-24T17:24:32.420Z', total: issues.filter(i => i.issue.repo === repo).length, loaded: issues.filter(i => i.issue.repo === repo).length, complete: true, error: null});
const report = (issues) => ({version: 1, at: '2026-09-24T17:24:32.420Z', provenance: atlasCore.provenance, discovery: {}, worlds: [world('ghashtag/t27'), world('ghashtag/trinity')], snapshots: [snap('ghashtag/t27', issues), snap('ghashtag/trinity', issues)], analyses: issues});
const issueA = (state) => ({issue: {repo: 'ghashtag/t27', number: 7, title: 't', state}, hits: []});
const a1 = outcome(() => buildAtlas(atlasCore, report([issueA('open')])));
rec('a01', 'atlas', {one_spec_in_two_worlds: true}, a1.ok ? {ok: true, value: {distinct: a1.value.specs.length, perWorld: a1.value.worlds.map(w => w.specCount), sumOfWorlds: a1.value.worlds.reduce((n, w) => n + w.specCount, 0)}} : a1);
rec('a02', 'atlas', {issue_state: 'closed'}, outcome(() => buildAtlas(atlasCore, report([issueA('closed')])).issues.length));
rec('a03', 'atlas', {at: 'not a date'}, outcome(() => validateAtlas({...a1.value, at: 'not a date'}).at));
rec('a04', 'atlas', {at: 'a year before the snapshot'}, outcome(() => validateAtlas({...a1.value, at: '2025-09-24T17:24:32.420Z'}).at));
// The HUD's pure functions.
const t0 = Date.parse('2026-09-04T09:00:00Z');
for (const [id, now, synced, error] of [['s01', t0 + 42000, t0, 'Failed to fetch'], ['s02', t0 + 42000, t0, null], ['s03', t0, null, 'Failed to fetch'], ['s04', t0 + 7200000, t0, 'Failed to fetch']]) {
  rec(id, 'stale', {now_s: (now - t0) / 1000, last_success_s: 0, has_success: synced !== null, failed: error !== null}, outcome(() => hud.staleAge(now, synced === null ? null : new Date(synced), error)));
}
const child = (number, state, reason) => ({number, title: 'c', state, closedAt: state === 'closed' ? '2026-09-26T00:00:00Z' : null, ...(reason ? {stateReason: reason} : {})});
const epic = {number: 988, title: 'EPIC', state: 'open', closedAt: null, labels: [], children: [child(3563, 'closed', 'completed'), child(3570, 'closed', 'not_planned'), child(3571, 'closed', 'not_planned'), child(989, 'open')]};
rec('e01', 'epic', {children: [['closed', 'completed'], ['closed', 'not_planned'], ['closed', 'not_planned'], ['open', null]]}, outcome(() => hud.epicProgress(epic)));
rec('e02', 'epic', {epic_closed: true, children: [['closed', 'completed'], ['closed', 'not_planned']]}, outcome(() => hud.towerStage({...epic, state: 'closed', children: [child(1, 'closed', 'completed'), child(2, 'closed', 'not_planned')]})));
rec('e03', 'epic', {asked: 'issue 989 of a repository the call cannot name'}, outcome(() => (hud.epicOfIssue(989, [epic]) ?? {number: null}).number));
rec('e04', 'module', {module: 'agent-server/apps/server/src'}, outcome(() => hud.moduleCard({path: 'agent-server/apps/server/src', files: 3, lastTouched: '2026-09-01T00:00:00Z', openIssues: []}, Date.parse('2026-09-04T00:00:00Z'), new Set()).number));
// Live rows against a cell: the board of one repository, a cell of another with the same number.
const board = {repo: 'gHashTag/t27', columns: [{key: 'done', title: 'Done', blurb: ''}], cards: [{number: 1153, title: '#1153', column: 'done'}], pulse: {}};
rec('j01', 'join', {cell: 'ghashtag/trinity#1153', board: 'gHashTag/t27'}, outcome(() => { const r = readBoardCell(board, 1153); return {joined: r.card !== null, column: r.card?.column ?? null}; }));
rec('j02', 'join', {cell: 'ghashtag/t27#1153', board: 'gHashTag/t27'}, outcome(() => { const r = readBoardCell(board, 1153); return {joined: r.card !== null, column: r.card?.column ?? null}; }));
rec('j03', 'join', {cell: 'ghashtag/trinity#1153', activity_of: 'gHashTag/t27'}, outcome(() => readActivityRows({events: [{id: 'e1', kind: 'dispatch', issue: 1153, at: '2026-10-01T00:00:00Z'}]}, 1153).length));
// What paints a cell honey.
for (const [id, state, coverage] of [['p01', 'closed', 'unknown'], ['p02', 'done', 'unknown'], ['p03', 'closed', 't27'], ['p04', 'open', 't27'], ['p05', 'running', 'unknown']]) {
  rec(id, 'paint', {state, coverage}, outcome(() => hiveTaskPaint({state, coverage})));
}
process.stdout.write(JSON.stringify(out) + '\n');
"""


def site_of(trinity: pathlib.Path) -> pathlib.Path:
    return trinity / SITE


def ensure_node_modules(site: pathlib.Path):
    if (site / "node_modules" / "vite").exists():
        return
    rc, o, e = base.run(["npm", "ci", "--prefer-offline", "--no-audit", "--no-fund", "--ignore-scripts"], cwd=str(site), timeout=1800)
    if rc != 0:
        raise RuntimeError(f"npm ci failed: {(e or o)[-300:]}")


def ensure_dist(site: pathlib.Path):
    if (site / "dist" / "index.html").exists():
        return
    ensure_node_modules(site)
    rc, o, e = base.run(["node", "node_modules/vite/bin/vite.js", "build"], cwd=str(site), timeout=1800)
    if rc != 0:
        raise RuntimeError(f"vite build failed: {(e or o)[-300:]}")


def build_ts_record(trinity: pathlib.Path, bun: str) -> dict:
    facts = need_pin(trinity)
    site = site_of(trinity)
    ensure_node_modules(site)
    with tempfile.TemporaryDirectory() as tmp:
        probe = pathlib.Path(tmp) / "probe_libs.mjs"
        probe.write_text(PROBE_LIBS, encoding="utf-8")
        rc, o, e = base.run([bun, str(probe), str(site)], cwd=str(site), timeout=600)
    if rc != 0 or not o.strip():
        raise RuntimeError(f"probe failed rc={rc}: {(e or o)[-400:]}")
    cases = json.loads(o)
    return {"at": base.now(), "host": base.host(), "trinity": facts, "bun": tool_version([bun, "--version"]),
            "probe_sha256": hexsha(PROBE_LIBS.encode()), "result": {"cases": cases}}


# ===========================================================================================
# DATA: the public files of the pinned tree
# ===========================================================================================
def build_data_record(trinity: pathlib.Path) -> dict:
    facts = need_pin(trinity)
    pub = site_of(trinity) / "public"
    t27dir = pub / "t27"
    m = json.loads((t27dir / "manifest.json").read_text())
    core = json.loads((t27dir / "shared-core.json").read_text())
    atlas = json.loads((t27dir / "universe-atlas.json").read_text())
    found = json.loads((pub / "queen" / "foundation.json").read_text())
    specs = m["specs"]
    bare = [s for s in specs if "/" not in s["repo"]]
    manifest = {
        "entries": len(specs), "entries_with_sha256": sum(1 for s in specs if s.get("sha256")),
        "entries_with_revision": sum(1 for s in specs if s.get("revision") or s.get("commit")),
        "entries_bare_repo": len(bare), "bare_repos": sorted({s["repo"] for s in bare}),
        "entry_fields": sorted(specs[0].keys()), "duplicates_listed": len(m.get("duplicates") or []),
        "featured": m.get("featured"), "generated_from": {"repo": m["generatedFrom"]["repo"], "commit": m["generatedFrom"]["commit"]},
        "paths_unique": len({s["path"] for s in specs}) == len(specs),
        "has_observed_time": any(k in m for k in ("at", "observedAt", "generatedAt")),
    }
    key_worlds = [w for w in atlas["worlds"] if w["specCount"] > 0]
    atlas_doc = {
        "at": atlas["at"], "worlds": len(atlas["worlds"]), "key_worlds": len(key_worlds), "issues": len(atlas["issues"]),
        "specs": len(atlas["specs"]), "sum_spec_count": sum(w["specCount"] for w in atlas["worlds"]),
        "specs_in_several_repositories": sum(1 for s in atlas["specs"] if len({x["repo"] for x in s["sources"]}) > 1),
        "issue_fields": sorted(atlas["issues"][0].keys()) if atlas["issues"] else [],
        "issues_carry_state": any("state" in i for i in atlas["issues"]),
    }
    core_doc = {"specs": len(core["specs"]), "specs_with_several_sources": sum(1 for s in core["specs"] if len(s["sources"]) > 1),
                "commit": core["provenance"].get("commit")}
    foundation = {"generatedAt": found.get("generatedAt"), "repo": found.get("repo"), "closed_issues": len(found.get("closedIssues", [])),
                  "epics": len(found.get("epics", [])), "children": sum(len(e.get("children", [])) for e in found.get("epics", [])),
                  "rule": found.get("rule"), "child_fields": sorted({k for e in found.get("epics", []) for c in e.get("children", []) for k in c})}
    # The vendored mirror: every gHashTag/t27 entry against t27 at the revision the manifest names.
    rev = m["generatedFrom"]["commit"]
    t27_entries = [s["path"] for s in specs if s["repo"] == "t27"]
    blobs = cat_blobs(rev, t27_entries)
    same = differ = absent = 0
    for p in t27_entries:
        mine = (t27dir / "files" / p).read_bytes()
        if blobs.get(p) is None:
            absent += 1
        elif blobs[p] == mine:
            same += 1
        else:
            differ += 1
    on_disk = sorted(str(p.relative_to(t27dir / "files")) for p in (t27dir / "files").rglob("*.t27"))
    listed = {s["path"] for s in specs}
    mirror = {"t27_entries": len(t27_entries), "identical_at_manifest_revision": same, "differ_at_manifest_revision": differ,
              "absent_at_manifest_revision": absent, "unreachable_listed": len(m["generatedFrom"].get("unreachableFromDefaultRef") or []),
              "files_on_disk": len(on_disk), "files_not_in_manifest": [p for p in on_disk if p not in listed]}
    evidence_rel = "specs/ui/queen_evidence.t27"
    ev = (t27dir / "files" / evidence_rel)
    mirror["queen_evidence_sha256"] = hexsha(ev.read_bytes()) if ev.exists() else None
    mirror["queen_evidence_in_t27_at_manifest_revision"] = cat_blobs(rev, [evidence_rel]).get(evidence_rel) is not None
    # The docs generator's schema is closed: it names every constant it accepts.
    agents_js = (site_of(trinity) / "scripts" / "agents-from-specs.mjs").read_text()
    closed = [i + 1 for i, line in enumerate(agents_js.split("\n")) if "unknown constant" in line]
    return {"at": base.now(), "trinity": facts,
            "result": {"manifest": manifest, "atlas": atlas_doc, "shared_core": core_doc, "foundation": foundation, "mirror": mirror,
                       "docs_schema_closed_at_lines": closed, "source": source_facts(site_of(trinity))}}


# What the pinned source says, by pattern: each is a fact a finding rests on, counted where it lives.
SOURCE_FACTS = [
    ("no_online_handling", "src", r"navigator\.onLine"),
    ("no_framer_motion_config", "src", r"MotionConfig"),
    ("queen_uses_framer_motion", "src/pages/Queen.tsx", r"from ['\"]framer-motion['\"]"),
    ("queen_polls_abort_controllers", "src/pages/Queen.tsx", r"AbortController"),
    ("display_coverage_is_unknown_by_type", "src/components/queenHiveDisplay.ts", r"^\s*coverage: 'unknown';"),
    ("honey_needs_t27_coverage", "src/components/queenHiveDisplay.ts", r"row\.coverage === 't27' \? \{ tone: 'honey'"),
    ("catalog_rows_merge_observed", "src/components/queenCatalogData.ts", r"\[\.\.\.rows,\.\.\.updates\]"),
    ("catalog_snapshot_label", "src/components/QueenCatalogHive.tsx", r"Public snapshot, not live"),
    ("board_card_by_number", "src/lib/queenIssueLive.ts", r"\.find\(\(row\) => row\.number === number\)"),
    ("activity_by_number", "src/lib/queenIssueLive.ts", r"\.filter\(\(row\) => row\.issue === number\)"),
    ("cell_live_by_number", "src/components/QueenCellStage.tsx", r"useCellLive\(number\)"),
    ("factory_live_claim", "src/pages/Queen.tsx", r"backed by the live Queen ledger"),
    ("worker_capacity_zero", "src/pages/Queen.tsx", r"capacity: 0,"),
    ("module_card_number_is_hash", "src/components/queenHud.ts", r"number: moduleId\(m\.path\)"),
    ("picked_issue_url_from_card_number", "src/pages/Queen.tsx", r"issues/\$\{pickedCard\.number\}"),
    ("embedded_unknown_keeps_open_spec", "src/pages/SpecExplorer.tsx", r"keep the spec that is open"),
    ("address_sha256_read_once", "src/pages/SpecExplorer.tsx", r"sha256:p\.get\('sha256'\)"),
    ("epic_child_without_repository", "src/components/queenHud.ts", r"export interface EpicChild \{ number: number; title: string; state: string; closedAt: string \| null \}"),
    ("progress_counts_every_closed", "src/components/queenHud.ts", r'c\.state === "closed"'),
    ("explorer_error_in_english_only", "src/lib/specCatalog.ts", r"Spec catalog path is missing or ambiguous"),
    ("foundation_wire_then_file", "src/pages/Queen.tsx", r'readFrom\("\./queen/foundation\.json", "file"\)'),
    ("explorer_alerts", "src/pages/SpecExplorer.tsx", r'role="alert"'),
    ("explorer_status_on_success", "src/pages/SpecExplorer.tsx", r'verifiedHash&&<p role="status"'),
    ("queen_aria_live", "src/pages/Queen.tsx", r"aria-live"),
]


def source_facts(site: pathlib.Path) -> dict:
    out = {}
    for name, rel, pattern in SOURCE_FACTS:
        target = site / rel
        files = sorted(target.rglob("*.ts*")) if target.is_dir() else [target]
        hits = []
        for f in files:
            for i, line in enumerate(f.read_text(errors="replace").split("\n")):
                if re.search(pattern, line):
                    hits.append(f"{f.relative_to(site)}:{i + 1}")
        out[name] = {"pattern": pattern, "where": rel, "count": len(hits), "lines": hits[:6]}
    return out


def cat_blobs(rev: str, paths: list) -> dict:
    """t27 blobs at a revision of this repository, None where the path is absent."""
    if not paths:
        return {}
    p = subprocess.run(["git", "-C", str(ROOT), "cat-file", "--batch"], input="".join(f"{rev}:{x}\n" for x in paths).encode(), capture_output=True)
    if p.returncode != 0:
        raise RuntimeError(f"git cat-file failed: {p.stderr.decode()[:200]}")
    out, i, data = {}, 0, p.stdout
    for x in paths:
        nl = data.index(b"\n", i)
        hdr = data[i:nl].decode()
        i = nl + 1
        if hdr.endswith("missing"):
            out[x] = None
            continue
        size = int(hdr.split()[2])
        out[x] = data[i:i + size]
        i += size + 1
    return out


# ===========================================================================================
# LIVE: GitHub and the supervisor's public endpoints, once
# ===========================================================================================
def gh_json(args: list) -> object:
    rc, o, e = base.run(["gh"] + args, timeout=300)
    if rc != 0:
        raise RuntimeError(f"gh {' '.join(args[:3])} failed: {(e or o)[:200]}")
    return json.loads(o) if o.strip() else None


def issue_states(issues: list) -> dict:
    by = collections.defaultdict(list)
    for i in issues:
        by[i["repo"]].append(i["number"])
    out = {}
    for repo, nums in sorted(by.items()):
        owner, name = repo.split("/")
        for k in range(0, len(nums), 80):
            chunk = nums[k:k + 80]
            q = 'query{repository(owner:"%s",name:"%s"){%s}}' % (owner, name, " ".join(
                "i%d:issueOrPullRequest(number:%d){__typename ... on Issue{state stateReason}}" % (n, n) for n in chunk))
            d = gh_json(["api", "graphql", "-f", "query=" + q])
            r = ((d or {}).get("data") or {}).get("repository") or {}
            for n in chunk:
                v = r.get("i%d" % n) or {}
                out[f"{repo}#{n}"] = f"{v.get('state', 'MISSING')}/{v.get('stateReason') or '-'}"
    return out


def fetch_public(path: str):
    url = f"{LIVE_BASE}/queen/{path}"
    req = urllib.request.Request(url, headers={"Accept": "application/json", "User-Agent": "trinity-queen-views/1"})
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            body = r.read()
            return r.status, body
    except urllib.error.HTTPError as e:
        return e.code, e.read()


def build_live_record(trinity: pathlib.Path) -> dict:
    facts = need_pin(trinity)
    atlas = json.loads((site_of(trinity) / "public/t27/universe-atlas.json").read_text())
    states = issue_states(atlas["issues"])
    counts = collections.Counter(states.values())
    closed = sorted(k for k, v in states.items() if v.startswith("CLOSED"))
    subs = gh_json(["api", f"repos/{EPIC[0]}/issues/{EPIC[1]}/sub_issues?per_page=100"]) or []
    epic = [{"repo": s["repository_url"].split("/repos/")[1], "number": s["number"], "state": s["state"], "reason": s.get("state_reason"),
             "title": s["title"][:60]} for s in subs]
    refiled = {}
    for wp, (repo, n) in REFILED.items():
        d = gh_json(["api", f"repos/{repo}/issues/{n}"])
        m = re.match(r"\*Re-filed from #(\d+)\.\*", d.get("body") or "")
        refiled[wp] = {"repo": repo, "number": n, "state": d["state"], "reason": d.get("state_reason"), "author": d["user"]["login"],
                       "refiled_from": int(m.group(1)) if m else None,
                       "sub_issue_of_epic": any(e["repo"].lower() == repo.lower() and e["number"] == n for e in epic)}
    endpoints, answers = live_answers()
    board = answers.get("public-board") or {}
    cards = {c["number"]: c for c in board.get("cards", [])}
    keyw = {w["repo"] for w in atlas["worlds"] if w["specCount"] > 0}
    board_repo = (board.get("repo") or "").lower()
    collisions = sorted(([i["key"], cards[i["number"]]["column"]] for i in atlas["issues"]
                         if i["repo"] in keyw and i["repo"] != board_repo and i["number"] in cards), key=lambda x: x[0])
    status = answers.get("status") or {}
    return {"at": base.now(), "trinity": facts, "snapshot": True,
            "result": {"atlas_at": atlas["at"], "issues": len(states), "states": dict(sorted(counts.items())), "closed": closed,
                       "epic": {"repo": EPIC[0], "number": EPIC[1], "sub_issues": epic}, "refiled": refiled,
                       "endpoints": endpoints, "board": {"repo": board.get("repo"), "cards": len(cards), "columns": [c.get("key") for c in board.get("columns", [])]},
                       "collisions": collisions, "status": {"swarmState": status.get("swarmState"), "has_workers": "workers" in status}}}


def live_answers():
    """One answer of each public endpoint: (status, size and hash of each; the parsed bodies that answered 200)."""
    endpoints, answers = {}, {}
    for p in LIVE_PATHS:
        status, body = fetch_public(p)
        endpoints[p] = {"status": status, "bytes": len(body), "sha256": hexsha(body)}
        if status == 200:
            answers[p] = json.loads(body)
    return endpoints, answers


# ===========================================================================================
# GATES: every check, audit and test script of the site, here, and whether a workflow runs it
# ===========================================================================================
GATE_PREFIXES = ("check:", "audit:", "test:", "typecheck")
GATE_WRITERS = {"typecheck:update"}          # changes the tree; not a gate
AUDIT_URLS = {"audit:en": "http://127.0.0.1:4173/index.html?lang=en", "audit:ru": "http://127.0.0.1:4173/index.html?lang=ru",
              "audit:mobile": "http://127.0.0.1:4173/index.html", "audit:mobile ru": "http://127.0.0.1:4173/index.html?lang=ru"}


def workflow_runs(trinity: pathlib.Path) -> dict:
    """Script name -> the workflow files that run `npm run <name>`."""
    out = collections.defaultdict(set)
    for f in sorted((trinity / ".github" / "workflows").glob("*.y*ml")):
        for m in re.finditer(r"npm run (?:-s )?([\w:.-]+)", f.read_text(errors="replace")):
            out[m.group(1)].add(f.name)
    return out


def run_gate(cmd, cwd, env=None, timeout=600) -> dict:
    t0 = time.time()
    p = subprocess.Popen(cmd, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, start_new_session=True)
    try:
        out, _ = p.communicate(timeout=timeout)
        rc = p.returncode
    except subprocess.TimeoutExpired:
        os.killpg(p.pid, 9)
        out, _ = p.communicate()
        rc = None
    lines = [x for x in (out or "").split("\n") if x.strip()]
    tail = lines[-1][:200] if lines else ""
    errs = [x.strip()[:200] for x in lines if re.search(r"\b(FAIL|Error|AssertionError|could not run)\b", x)][:3]
    verdict = "timeout" if rc is None else ("skipped" if rc == 0 and re.search(r"skipping|no Chrome found", out or "") else ("pass" if rc == 0 else "fail"))
    return {"rc": rc, "seconds": round(time.time() - t0), "verdict": verdict, "tail": tail, "errors": errs}


def twice(run) -> dict:
    """A gate that fails is run once more: failing both times is 'fail', passing once is 'flaky' (several
    browser contracts read the live supervisor, whose answer time is not the site's)."""
    first = run()
    if first["verdict"] not in ("fail", "timeout"):
        return first
    second = run()
    if second["verdict"] in ("fail", "timeout"):
        return {**first, "second": {k: second[k] for k in ("rc", "seconds", "verdict", "tail")}}
    return {**second, "verdict": "flaky", "first": {k: first[k] for k in ("rc", "seconds", "verdict", "tail")}}


def build_gates_record(trinity: pathlib.Path, chrome: str | None) -> dict:
    facts = need_pin(trinity)
    site = site_of(trinity)
    ensure_dist(site)
    scripts = json.loads((site / "package.json").read_text())["scripts"]
    names = sorted(k for k in scripts if k.startswith(GATE_PREFIXES) and k not in GATE_WRITERS)
    runs = workflow_runs(trinity)
    env = dict(os.environ)
    if chrome:
        env["CHROME_PATH"] = chrome
    results = {}
    preview = None
    try:
        for name in names:
            if name.startswith("audit:"):
                continue
            # --no-build reuses the one build: the browser contracts read it, plain node scripts ignore it, and
            # a script that is not a node script of qa/ or scripts/ (tsc -b, npx vite build && ...) gets nothing
            cmd = scripts[name]
            extra = ["--", "--no-build"] if cmd.startswith("node ") and re.search(r"\b(qa|scripts)/", cmd) else []
            results[name] = twice(lambda: run_gate(["npm", "run", "-s", name] + extra, str(site), env))
        preview = subprocess.Popen(["node", "node_modules/vite/bin/vite.js", "preview", "--port", "4173", "--strictPort"], cwd=str(site),
                                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
        time.sleep(4)
        for label, url in AUDIT_URLS.items():
            name = label.split(" ")[0]
            results[label] = twice(lambda: run_gate(["npm", "run", "-s", name], str(site), {**env, "AUDIT_BASE_URL": url}))
    finally:
        if preview:
            os.killpg(preview.pid, 9)
    gates = {k: {**v, "ci": sorted(runs.get(k.split(" ")[0], []))} for k, v in results.items()}
    ci_named = sorted(n for n in names if runs.get(n))
    after = checkout_facts(trinity)["tracked_changes"]
    if after:
        raise RuntimeError(f"the gates changed tracked files: {after[:5]}")
    return {"at": base.now(), "host": base.host(), "trinity": facts, "node": tool_version(["node", "--version"]),
            "chrome": tool_version([chrome, "--version"]) if chrome else None,
            "result": {"scripts": len(names), "run_by_ci": len(ci_named), "never_run_by_ci": sorted(n for n in names if not runs.get(n)), "gates": gates}}


# ===========================================================================================
# BROWSER: the built site in headless Chrome over CDP
# ===========================================================================================
PROBE_BROWSER = r"""
// S10 browser probe: the built site (dist/) in headless Chrome over CDP. Prints one JSON document.
// node probe_browser.mjs <site-root> <chrome> <fixtures.json>
import {spawn} from 'node:child_process';
import {createServer} from 'node:http';
import {existsSync, mkdtempSync, readFileSync, statSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join, extname} from 'node:path';

const [SITE, CHROME, FIXTURES] = process.argv.slice(2);
const DIST = join(SITE, 'dist');
const FX = JSON.parse(readFileSync(FIXTURES, 'utf8'));
const MIME = {'.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.png': 'image/png', '.svg': 'image/svg+xml', '.json': 'application/json', '.wasm': 'application/wasm', '.woff2': 'font/woff2', '.t27': 'text/plain', '.txt': 'text/plain'};
const server = createServer((req, res) => {
  let f = join(DIST, decodeURIComponent(new URL(req.url, 'http://x').pathname));
  if (!existsSync(f) || statSync(f).isDirectory()) f = join(DIST, 'index.html');
  res.writeHead(200, {'Content-Type': MIME[extname(f)] ?? 'application/octet-stream', 'Cache-Control': 'no-store'});
  res.end(readFileSync(f));
}).listen(0, '127.0.0.1');
await new Promise(r => server.once('listening', r));
const ORIGIN = `http://127.0.0.1:${server.address().port}`;
const profile = mkdtempSync(join(tmpdir(), 's10-'));
const chrome = spawn(CHROME, ['--headless=new', '--remote-debugging-port=0', `--user-data-dir=${profile}`, '--no-first-run', '--disable-extensions', '--mute-audio', '--window-size=1440,900', '--use-gl=swiftshader', '--enable-unsafe-swiftshader', 'about:blank'], {stdio: ['ignore', 'ignore', 'pipe']});
const browserWs = await new Promise((resolve, reject) => {
  const t = setTimeout(() => reject(new Error('no DevTools port')), 30000); let buf = '';
  chrome.stderr.on('data', d => { buf += d; const m = buf.match(/DevTools listening on (ws:\/\/\S+)/); if (m) { clearTimeout(t); resolve(m[1]); } });
});
const ws = new WebSocket(browserWs); await new Promise(r => ws.addEventListener('open', r));
let nextId = 0; const pending = new Map(); const handlers = [];
ws.addEventListener('message', ev => {
  const msg = JSON.parse(ev.data);
  if (msg.id && pending.has(msg.id)) { const {resolve, reject} = pending.get(msg.id); pending.delete(msg.id); msg.error ? reject(new Error(msg.error.message)) : resolve(msg.result); return; }
  for (const h of handlers) h(msg);
});
const send = (method, params = {}, sessionId) => new Promise((resolve, reject) => { const id = ++nextId; pending.set(id, {resolve, reject}); ws.send(JSON.stringify({id, method, params, ...(sessionId ? {sessionId} : {})})); });
const wait = ms => new Promise(r => setTimeout(r, ms));

// One page per scenario. `routes` answers requests by URL substring: {status, body, contentType} or 'fail'.
async function page({width = 1440, height = 900, mobile = false, reducedMotion = false, routes = []}) {
  const {targetId} = await send('Target.createTarget', {url: 'about:blank'});
  const {sessionId} = await send('Target.attachToTarget', {targetId, flatten: true});
  const call = (m, p) => send(m, p, sessionId);
  const log = [];
  const counts = new Map();
  handlers.push(async msg => {
    if (msg.sessionId !== sessionId) return;
    if (msg.method === 'Runtime.exceptionThrown') log.push('exception: ' + (msg.params.exceptionDetails?.exception?.description ?? msg.params.exceptionDetails?.text ?? '').slice(0, 200));
    if (msg.method === 'Fetch.requestPaused') {
      const {requestId, request} = msg.params;
      const route = routes.find(r => request.url.includes(r.match));
      if (!route) { await call('Fetch.continueRequest', {requestId}).catch(() => {}); return; }
      const n = (counts.get(route.match) ?? 0) + 1; counts.set(route.match, n);
      const answer = typeof route.answer === 'function' ? route.answer(n) : route.answer;
      if (answer === 'fail') { await call('Fetch.failRequest', {requestId, errorReason: 'ConnectionRefused'}).catch(() => {}); return; }
      if (answer.delay) await wait(answer.delay);
      await call('Fetch.fulfillRequest', {requestId, responseCode: answer.status ?? 200, responseHeaders: [{name: 'Content-Type', value: answer.contentType ?? 'application/json'}, {name: 'Access-Control-Allow-Origin', value: '*'}], body: Buffer.from(answer.body ?? '').toString('base64')}).catch(() => {});
    }
  });
  await call('Page.enable'); await call('Runtime.enable');
  if (routes.length) await call('Fetch.enable', {patterns: [{urlPattern: '*', requestStage: 'Request'}]});
  await call('Emulation.setDeviceMetricsOverride', {width, height, deviceScaleFactor: 1, mobile});
  if (mobile) await call('Emulation.setTouchEmulationEnabled', {enabled: true, maxTouchPoints: 5});
  await call('Emulation.setEmulatedMedia', {features: [{name: 'prefers-reduced-motion', value: reducedMotion ? 'reduce' : 'no-preference'}]});
  const evaluate = async expr => { const r = await call('Runtime.evaluate', {expression: expr, awaitPromise: true, returnByValue: true}); if (r.exceptionDetails) throw new Error('probe: ' + (r.exceptionDetails.exception?.description ?? r.exceptionDetails.text)); return r.result.value; };
  const until = async (expr, ms = 20000) => { const t0 = Date.now(); while (Date.now() - t0 < ms) { try { if (await evaluate(expr)) return true; } catch {} await wait(200); } return false; };
  const goto = async (path) => { await call('Page.navigate', {url: ORIGIN + (path.startsWith('/?') ? path : '/?lang=en' + path.slice(1))}); };
  const key = async (k) => { await call('Input.dispatchKeyEvent', {type: 'keyDown', key: k, text: k.length === 1 ? k : undefined}); await call('Input.dispatchKeyEvent', {type: 'keyUp', key: k}); };
  const close = async () => { await send('Target.closeTarget', {targetId}).catch(() => {}); };
  return {call, evaluate, until, goto, key, close, log, counts};
}

const results = [];
const scenario = async (id, fn) => {
  const t0 = Date.now();
  try { const r = await fn(); results.push({id, ok: true, ms: Date.now() - t0, ...r}); }
  catch (e) { results.push({id, ok: false, ms: Date.now() - t0, error: String(e.message ?? e).slice(0, 300)}); }
};
const text = `document.body.innerText`;
const mainText = `(document.querySelector('main')?.innerText ?? '')`;
const HELLO = 'specs/demos/hello_world.t27';

// ---- the spec explorer: an explicit address, a basename, a wrong hash, a later hash in a frame
await scenario('x01-exact', async () => {
  const p = await page({});
  await p.goto(`/#/specs?spec=${encodeURIComponent(HELLO)}`);
  const shown = await p.until(`${text}.includes(${JSON.stringify(HELLO)}) && !!document.querySelector('textarea, .cm-content, pre')`);
  const r = {shown, hash: await p.evaluate('location.hash'), error: await p.evaluate(`[...document.querySelectorAll('main div')].some(d => /missing or ambiguous|SHA-256 mismatch|Invalid catalog/.test(d.textContent))`)};
  await p.close(); return r;
});
for (const [id, address] of [['x02-missing', 'specs/queen/missing.t27'], ['x03-basename', 'hello_world.t27'], ['x04-suffix', 'demos/hello_world.t27']]) {
  await scenario(id, async () => {
    const p = await page({});
    await p.goto(`/#/specs?spec=${encodeURIComponent(address)}`);
    const error = await p.until(`/missing or ambiguous|Invalid catalog/.test(${text})`, 15000);
    const r = {error, otherSpecShown: await p.evaluate(`${mainText}.includes(${JSON.stringify(HELLO)})`), selected: await p.evaluate(`[...document.querySelectorAll('main span')].map(s => s.textContent).find(t => /\.t27$/.test(t ?? '')) ?? null`), hash: await p.evaluate('location.hash')};
    await p.close(); return r;
  });
}
await scenario('x05-wrong-sha', async () => {
  const p = await page({});
  await p.goto(`/#/specs?spec=${encodeURIComponent(HELLO)}&sha256=${'0'.repeat(64)}`);
  const error = await p.until(`/SHA-256 mismatch/.test(${text})`, 15000);
  const r = {error, verifiedBadge: await p.evaluate(`/SHA-256 verified/.test(${text})`), sourceShown: await p.evaluate(`/module HelloWorld/.test(${text})`)};
  await p.close(); return r;
});
await scenario('x06-right-sha', async () => {
  const p = await page({});
  await p.goto(`/#/specs?spec=${encodeURIComponent(HELLO)}&sha256=${FX.helloSha256}`);
  const verified = await p.until(`/SHA-256 verified/.test(${text})`, 15000);
  const r = {verified, hash: await p.evaluate('location.hash')};
  await p.close(); return r;
});
await scenario('x07-embed-later-missing', async () => {
  const p = await page({});
  await p.goto(`/#/specs?spec=${encodeURIComponent(HELLO)}&embed=1`);
  const first = await p.until(`${mainText}.includes(${JSON.stringify(HELLO)})`);
  await p.evaluate(`location.hash = '#/specs?spec=' + encodeURIComponent('specs/queen/missing.t27') + '&embed=1'`);
  await wait(2500);
  const r = {first, hash: await p.evaluate('location.hash'), stillShowsOld: await p.evaluate(`${mainText}.includes(${JSON.stringify(HELLO)})`), selected: await p.evaluate(`[...document.querySelectorAll('main span')].map(s => s.textContent).find(t => /\.t27$/.test(t ?? '')) ?? null`), error: await p.evaluate(`/missing or ambiguous|Invalid catalog/.test(${text})`)};
  await p.close(); return r;
});
await scenario('x08-embed-later-wrong-sha', async () => {
  const p = await page({});
  await p.goto(`/#/specs?spec=${encodeURIComponent(HELLO)}&embed=1`);
  const first = await p.until(`${mainText}.includes(${JSON.stringify(HELLO)})`);
  await p.evaluate(`location.hash = '#/specs?spec=' + encodeURIComponent(${JSON.stringify(FX.otherSpec)}) + '&sha256=${'0'.repeat(64)}&embed=1'`);
  const opened = await p.until(`${mainText}.includes(${JSON.stringify(FX.otherSpec)})`, 15000);
  await wait(1500);
  const r = {first, opened, mismatchShown: await p.evaluate(`/SHA-256 mismatch/.test(${text})`), hash: await p.evaluate('location.hash')};
  await p.close(); return r;
});
await scenario('x09-missing-ru', async () => {
  const p = await page({});
  await p.goto(`/?lang=ru#/specs?spec=${encodeURIComponent('specs/queen/missing.t27')}`);
  const error = await p.until(`/missing or ambiguous|Invalid catalog|\u043d\u0435 \u043d\u0430\u0439\u0434\u0435\u043d|\u043d\u0435\u043e\u0434\u043d\u043e\u0437\u043d\u0430\u0447\u043d/.test(${text})`, 15000);
  const r = {error, english: await p.evaluate(`/missing or ambiguous/.test(${text})`), lang: await p.evaluate('document.documentElement.lang')};
  await p.close(); return r;
});

// ---- the Queen: every live endpoint answered from the recorded public snapshot, nothing else reaches out
const API = 'trios-agent-server-production.up.railway.app';
const json = (v) => ({body: JSON.stringify(v)});
const offline = [{match: 'api.github.com', answer: 'fail'}, {match: 'vibee-render', answer: 'fail'}, {match: 't27-github-collab', answer: 'fail'},
  {match: 'queen-proxy', answer: 'fail'}, {match: 'app.t27.ai', answer: 'fail'}, {match: '127.0.0.1:8899', answer: 'fail'}];
const queenRoutes = (over = {}) => [
  ...(over.first ?? []),
  {match: `${API}/queen/status`, answer: json(over.status ?? FX.live.status)},
  {match: `${API}/queen/public-board`, answer: over.board ?? json(FX.live.board)},
  {match: `${API}/queen/public-activity`, answer: json(FX.live.activity)},
  {match: `${API}/queen/public-research`, answer: over.research ?? json(FX.live.research)},
  {match: `${API}/queen/public-hardware`, answer: json(FX.live.hardware)},
  {match: `${API}/queen/public-foundation`, answer: {status: 404, body: '{"error":"not found"}'}},
  {match: `${API}/queen/public-modules`, answer: {status: 404, body: '{"error":"not found"}'}},
  {match: `${API}/queen/public-leaderboard`, answer: {status: 404, body: '{}'}},
  ...offline,
];
const shell = `!!document.querySelector('main.queen27-page')`;
const viewNow = `({view: document.querySelector('main.queen27-page')?.dataset.view ?? null, tab: new URLSearchParams(location.hash.split('?')[1] ?? '').get('tab')})`;
const navigate = async (p, steps, how) => {
  const seen = [];
  for (const [step, want] of steps) {
    if (how === 'key') await p.key(step);
    else {
      const box = await p.evaluate(`(() => { const b = [...document.querySelectorAll('button.queen27-hud-cmd[data-view="${step}"]')].find(x => x.getBoundingClientRect().width > 0); if (!b) return null; const r = b.getBoundingClientRect(); return {x: r.x + r.width / 2, y: r.y + r.height / 2, w: r.width, h: r.height}; })()`);
      if (!box) { seen.push({step, want, error: 'no button'}); continue; }
      await p.call('Input.dispatchTouchEvent', {type: 'touchStart', touchPoints: [{x: box.x, y: box.y}]});
      await p.call('Input.dispatchTouchEvent', {type: 'touchEnd', touchPoints: []});
    }
    const reached = await p.until(`(${viewNow}).view === ${JSON.stringify(want)}`, 8000);
    seen.push({step, want, reached, ...(await p.evaluate(viewNow))});
  }
  return seen;
};
for (const [id, opts, how] of [['q01-nav-desktop', {}, 'key'], ['q02-nav-mobile', {width: 390, height: 844, mobile: true}, 'tap'], ['q03-nav-reduced-motion', {reducedMotion: true}, 'key']]) {
  await scenario(id, async () => {
    const p = await page({...opts, routes: queenRoutes()});
    await p.goto('/#/queen');
    const mounted = await p.until(shell, 30000);
    await wait(1500);
    const steps = how === 'key' ? [['3', 'kanban'], ['p', 'project'], ['2', 'specs'], ['1', 'comb']] : [['kanban', 'kanban'], ['project', 'project'], ['comb', 'comb']];
    const seen = await navigate(p, steps, how);
    const layout = await p.evaluate(`({scrollW: document.documentElement.scrollWidth, clientW: document.documentElement.clientWidth, reduce: matchMedia('(prefers-reduced-motion: reduce)').matches,
      small: [...document.querySelectorAll('button.queen27-hud-cmd')].filter(b => { const r = b.getBoundingClientRect(); return r.width > 0 && Math.min(r.width, r.height) < 44; }).length,
      rail: [...document.querySelectorAll('button.queen27-hud-cmd')].filter(b => b.getBoundingClientRect().width > 0).length})`);
    const r = {mounted, seen, layout, exceptions: p.log.length};
    await p.close(); return r;
  });
}
for (const [id, reducedMotion] of [['q04-motion-kanban', false], ['q05-reduced-motion-kanban', true]]) {
  await scenario(id, async () => {
    const p = await page({reducedMotion, routes: queenRoutes()});
    await p.goto('/#/queen');
    await p.until(shell, 30000); await wait(1500);
    await p.evaluate(`window.__seen = new Set(); window.__anims = []; (function sample(){ for (const a of document.getAnimations()) { if (window.__seen.has(a)) continue; window.__seen.add(a); const t = a.effect?.getTiming?.(); const d = typeof t?.duration === 'number' ? t.duration : 0; window.__anims.push({kind: a.constructor.name, d, name: a.animationName ?? a.id ?? ''}); } if (window.__anims.length < 20000) requestAnimationFrame(sample); })();`);
    await p.key('3');
    await p.until(`(${viewNow}).view === 'kanban'`, 8000);
    await wait(2000);
    const r = await p.evaluate(`(() => { const long = window.__anims.filter(a => a.d > 1); const kinds = {}; for (const a of long) kinds[a.kind] = (kinds[a.kind] ?? 0) + 1; return {reduce: matchMedia('(prefers-reduced-motion: reduce)').matches, animations: window.__anims.length, longer_than_1ms: long.length, kinds, maxMs: Math.max(0, ...long.map(a => a.d))}; })()`);
    await p.close(); return r;
  });
}
await scenario('q06-catalog-snapshot-label', async () => {
  const p = await page({routes: queenRoutes()});
  await p.goto('/#/queen');
  await p.until(`!!document.querySelector('.queen-catalog-law')`, 30000);
  await wait(1000);
  const r = await p.evaluate(`(() => { const t = document.body.innerText; const law = document.querySelector('.queen-catalog-law')?.innerText ?? ''; return {law, saysSnapshot: /Public snapshot, not live/.test(law),
    dateVisible: /2026-09-24|24\\/09\\/2026|24\\.09\\.2026|Sep(tember)? 24|24 Sep/.test(t), liveBadge: /LIVE/.test(document.querySelector('header')?.innerText ?? t)}; })()`);
  await p.close(); return r;
});
await scenario('q07-cell-joins-board-by-number', async () => {
  const issue = {number: FX.collision.number, title: FX.collision.title, state: 'open', html_url: `https://github.com/${FX.collision.repo}/issues/${FX.collision.number}`, body: 'fixture', assignees: [], labels: [], user: {login: 'x'}, created_at: '2026-09-20T00:00:00Z', updated_at: '2026-09-20T00:00:00Z', closed_at: null, state_reason: null};
  const p = await page({routes: queenRoutes({first: [{match: `api.github.com/repos/${FX.collision.repo}/issues/${FX.collision.number}`, answer: json(issue)}]})});
  await p.goto(`/#/queen?world=${encodeURIComponent(FX.collision.repo)}&task=${FX.collision.number}`);
  const opened = await p.until(`!!document.querySelector('.queen-cell-stage')`, 30000);
  await p.until(`!/\u2026/.test(document.querySelector('.queen-cell-column')?.textContent ?? '\u2026')`, 15000);
  const r = await p.evaluate(`({opened: ${opened}, cell: document.querySelector('.queen-cell-id')?.textContent ?? null, column: document.querySelector('.queen-cell-column')?.textContent ?? null,
    pulse: !!document.querySelector('.queen-cell-hex-pulse'), notOnBoard: /${FX.notOnBoardEn}/.test(document.querySelector('.queen-cell-stage')?.innerText ?? ''), boardRepo: ${JSON.stringify(FX.live.board.repo)}, boardCardColumn: ${JSON.stringify(FX.collision.boardColumn)}})`);
  await p.close(); return r;
});
await scenario('q08-missing-capacity', async () => {
  const status = JSON.parse(JSON.stringify(FX.live.status)); delete status.workers;
  const p = await page({routes: queenRoutes({status, research: 'fail'})});
  await p.goto('/#/queen');
  await p.until(`/\\d/.test(document.getElementById('stat-bees')?.textContent ?? '')`, 30000);
  await wait(1500);
  const r = await p.evaluate(`({bees: document.getElementById('stat-bees')?.textContent ?? null, running: ${status.dispatches?.running ?? null}})`);
  await p.close(); return r;
});
await scenario('q09-factory-live-claim', async () => {
  const p = await page({routes: queenRoutes({research: 'fail'})});
  await p.goto('/#/queen?tab=factory');
  await p.until(`(${viewNow}).view === 'factory'`, 30000); await wait(1500);
  const r = await p.evaluate(`({claimsLive: /backed by the live Queen ledger/.test(document.body.innerText), view: (${viewNow}).view})`);
  await p.close(); return r;
});
for (const [id, boardRepo] of [['q10-foundation-other-repository', null], ['q10-foundation-file-folded-in', 'gHashTag/trios']]) {
  await scenario(id, async () => {
    const board = boardRepo ? json({...FX.live.board, repo: boardRepo}) : json(FX.live.board);
    const p = await page({routes: queenRoutes({board, first: [{match: '/t27/universe-atlas.json', answer: {status: 500, body: '{}'}},
      {match: '/queen/foundation.json', answer: (n) => json(n === 1 ? FX.foundationA : FX.foundationB)}]})});
    await p.goto('/#/queen');
    const shellUp = await p.until(shell, 30000);
    const mounted = await p.until(`!!document.querySelector('[data-foundation]')`, 20000);
    const before = await p.evaluate(`document.querySelector('[data-foundation]')?.dataset.foundation ?? null`);
    const changed = mounted ? await p.until(`(document.querySelector('[data-foundation]')?.dataset.foundation ?? null) !== ${JSON.stringify(before)}`, 75000) : false;
    const r = await p.evaluate(`(() => { const t = document.body.innerText; return {shellUp: ${shellUp}, mounted: ${mounted}, before: ${JSON.stringify(before)}, after: document.querySelector('[data-foundation]')?.dataset.foundation ?? null, changed: ${changed},
      dateVisible: /${FX.foundationDatePattern}/.test(t), badge: (t.match(/LIVE[^\\n]{0,24}/) ?? [null])[0]}; })()`);
    r.boardRepo = boardRepo ?? FX.live.board.repo;
    r.fileRequests = p.counts.get('/queen/foundation.json') ?? 0;
    await p.close(); return r;
  });
}

await scenario('x10-explorer-late-answer', async () => {
  // The first spec's bytes are held for 2.5 s; the second is picked meanwhile and answers at once.
  const slow = FX.raceSlow, fast = FX.raceFast;
  const p = await page({routes: [{match: `/t27/files/${slow}`, answer: {delay: 2500, contentType: 'text/plain', body: FX.raceSlowText}}]});
  await p.goto(`/#/specs?spec=${encodeURIComponent(slow)}&embed=1`);
  await p.until(`${mainText}.includes(${JSON.stringify(slow)})`, 15000);
  await p.evaluate(`location.hash = '#/specs?spec=' + encodeURIComponent(${JSON.stringify(fast)}) + '&embed=1'`);
  await wait(300);
  const r0 = await p.evaluate(`(document.querySelector('main span')?.textContent ?? '')`);
  await wait(4000);
  const r = {selectedAfter: await p.evaluate(`[...document.querySelectorAll('main span')].map(s => s.textContent).find(t => /\\.t27$/.test(t ?? '')) ?? null`),
    sourceIsFast: await p.evaluate(`(document.querySelector('main textarea')?.value ?? document.querySelector('main')?.innerText ?? '').includes(${JSON.stringify(FX.raceFastMarker)})`),
    sourceIsSlow: await p.evaluate(`(document.querySelector('main textarea')?.value ?? document.querySelector('main')?.innerText ?? '').includes(${JSON.stringify(FX.raceSlowMarker)})`), first: r0};
  await p.close(); return r;
});
await scenario('q11-status-late-answer', async () => {
  const older = {...FX.live.status, swarmState: 'fixture-older'}, newer = {...FX.live.status, swarmState: 'fixture-newer'};
  let firstAt = 0;
  const p = await page({routes: queenRoutes({first: [{match: `${API}/queen/status`, answer: (n) => { if (n === 1) { firstAt = Date.now(); return {delay: 7000, body: JSON.stringify(older)}; } return json(newer); }}]})});
  await p.goto('/#/queen');
  await p.until(`/FIXTURE-NEWER/.test(document.getElementById('stat-status')?.textContent ?? '')`, 20000);
  const afterNewer = await p.evaluate(`document.getElementById('stat-status')?.textContent ?? null`);
  const waitFor = Math.max(0, firstAt + 7800 - Date.now());
  await wait(waitFor);
  const afterOlderLanded = await p.evaluate(`document.getElementById('stat-status')?.textContent ?? null`);
  const r = {afterNewer, afterOlderLanded, overwritten: /FIXTURE-OLDER/.test(afterOlderLanded ?? ''), statusRequests: p.counts.get(`${API}/queen/status`) ?? 0};
  await p.close(); return r;
});

process.stdout.write(JSON.stringify({origin: ORIGIN, chrome: CHROME, results}, null, 1) + '\n');
ws.close(); chrome.kill(); server.close(); try { rmSync(profile, {recursive: true, force: true}); } catch {}
process.exit(0);
"""


def browser_fixtures(trinity: pathlib.Path, answers: dict) -> dict:
    site = site_of(trinity)
    files = site / "public/t27/files"
    hello = "specs/demos/hello_world.t27"
    atlas = json.loads((site / "public/t27/universe-atlas.json").read_text())
    board = answers["public-board"]
    cards = {c["number"]: c for c in board.get("cards", [])}
    board_repo = (board.get("repo") or "").lower()
    keyw = {w["repo"] for w in atlas["worlds"] if w["specCount"] > 0}
    coll = sorted((i for i in atlas["issues"] if i["repo"] in keyw and i["repo"] != board_repo and i["number"] in cards), key=lambda i: i["key"])
    if not coll:
        raise RuntimeError("no catalog cell shares a number with a card of the live board; the collision scenario has nothing to show")
    c = coll[0]
    found_a = json.loads((site / "public/queen/foundation.json").read_text())
    found_b = json.loads(json.dumps(found_a))
    found_b["closedIssues"].append({"closedAt": "2026-09-06T10:00:00Z", "epicRefs": [], "labels": [], "number": 999999, "stateReason": "COMPLETED",
                                    "title": "fixture: one more closed issue"})
    slow, fast = "specs/queen/lotus.t27", hello
    return {"helloSha256": hexsha((files / hello).read_bytes()), "otherSpec": slow,
            "live": {"status": answers["status"], "board": board, "activity": answers["public-activity"], "research": answers["public-research"],
                     "hardware": answers["public-hardware"]},
            "collision": {"repo": c["repo"], "number": c["number"], "title": c["title"], "boardColumn": cards[c["number"]]["column"]},
            "notOnBoardEn": "The supervisor board does not carry this number",
            "foundationA": found_a, "foundationB": found_b, "foundationDatePattern": "2026-09-05|05\\/09\\/2026|05\\.09\\.2026|Sep(tember)? 5|5 Sep",
            "raceSlow": slow, "raceFast": fast, "raceSlowText": (files / slow).read_text(), "raceSlowMarker": "module QueenLotus",
            "raceFastMarker": "module HelloWorld"}


def build_browser_record(trinity: pathlib.Path, chrome: str) -> dict:
    facts = need_pin(trinity)
    site = site_of(trinity)
    ensure_dist(site)
    endpoints, answers = live_answers()
    missing = [p for p in ("status", "public-board", "public-activity", "public-research", "public-hardware") if p not in answers]
    if missing:
        raise RuntimeError(f"live endpoints did not answer: {missing}")
    fx = browser_fixtures(trinity, answers)
    with tempfile.TemporaryDirectory() as tmp:
        fpath = pathlib.Path(tmp) / "fixtures.json"
        fpath.write_text(json.dumps(fx), encoding="utf-8")
        probe = pathlib.Path(tmp) / "probe_browser.mjs"
        probe.write_text(PROBE_BROWSER, encoding="utf-8")
        rc, o, e = base.run(["node", str(probe), str(site), chrome, str(fpath)], cwd=str(site), timeout=1800)
    if rc != 0 or not o.strip():
        raise RuntimeError(f"browser probe failed rc={rc}: {(e or o)[-400:]}")
    doc = json.loads(o)
    return {"at": base.now(), "host": base.host(), "trinity": facts, "chrome": tool_version([chrome, "--version"]), "node": tool_version(["node", "--version"]),
            "probe_sha256": hexsha(PROBE_BROWSER.encode()), "dist_index_sha256": hexsha((site / "dist/index.html").read_bytes()),
            "fixtures": {"endpoints": endpoints, "collision": fx["collision"], "board_repo": fx["live"]["board"].get("repo")},
            "result": {r["id"]: {k: v for k, v in r.items() if k not in ("id", "ms")} for r in doc["results"]}}


# ===========================================================================================
# NATIVE: apps/queen built and tested
# ===========================================================================================
NATIVE_PATTERN = r"manifest\.json|universe-atlas|shared-core|api\.github\.com|t27\.ai|/queen/public-"


def build_native_record(trinity: pathlib.Path) -> dict:
    facts = need_pin(trinity)
    app = trinity / "apps/queen"
    with tempfile.TemporaryDirectory() as tmp:
        rc, o, e = base.run(["swift", "build", "--build-path", tmp], cwd=str(app), timeout=3600)
        warnings = len(re.findall(r"warning:", o + e))
        errors = len(re.findall(r"error:", o + e))
        b = {"verdict": "pass" if rc == 0 else "fail"}
        rc2, o2, e2 = base.run(["swift", "test", "--build-path", tmp], cwd=str(app), timeout=3600)
        t = {"verdict": "pass" if rc2 == 0 else "fail"}
    m = re.search(r"Test run with (\d+) tests? in (\d+) suites? passed", o2 + e2)
    refs = 0
    for f in list((app / "QueenUI").rglob("*.swift")) + list((app / "Tests").rglob("*.swift")):
        refs += len(re.findall(NATIVE_PATTERN, f.read_text(errors="replace")))
    swift_files = len(list((app / "QueenUI").rglob("*.swift"))) + len(list((app / "Tests").rglob("*.swift")))
    return {"at": base.now(), "host": base.host(), "trinity": facts, "swift": tool_version(["swift", "--version"]),
            "result": {"build": {"verdict": b["verdict"], "errors": errors, "warnings": warnings}, "test": {"verdict": t["verdict"],
                       "tests": int(m.group(1)) if m else None, "suites": int(m.group(2)) if m else None},
                       "swift_files": swift_files, "catalog_or_live_references": refs, "pattern": NATIVE_PATTERN}}


# ===========================================================================================
# THE CONTRACT, stated again in Python: the replay holds the spec's generated code to it, and the
# records hold the site to it. A case where the site parts from the contract names its finding.
# ===========================================================================================
def k_resolve(sp, grammar_ok, matches, hash_given, hash_equal):
    if not grammar_ok:
        return sp["R_INVALID"]
    if matches == 0:
        return sp["R_MISSING"]
    if matches > 1:
        return sp["R_AMBIGUOUS"]
    if hash_given and not hash_equal:
        return sp["R_HASH_MISMATCH"]
    return sp["R_RESOLVED"]


def k_view(sp, named, resolution, current):
    if not current:
        return sp["V_UNCHANGED"]
    if resolution != sp["R_RESOLVED"]:
        return sp["V_UNAVAILABLE"]
    return sp["V_SPEC"] if named else sp["V_FEATURED"]


def k_presentation(sp, source, has_data, loading, failed, online):
    if not has_data:
        return sp["P_LOADING"] if loading else (sp["P_OFFLINE"] if not online else sp["P_UNAVAILABLE"])
    if source == sp["SRC_SNAPSHOT"]:
        return sp["P_SNAPSHOT"]
    if not online:
        return sp["P_OFFLINE"]
    return sp["P_STALE"] if failed else sp["P_LIVE"]


CONTRACT = {
    "resolve_address": k_resolve,
    "view_of_address": k_view,
    "presentation": k_presentation,
    "accept_answer": lambda sp, request, latest: request == latest,
    "relation_of": lambda sp, kind: sp["REL_SAME"] if kind in (sp["ID_EXACT"], sp["ID_HASH"]) else sp["REL_CANDIDATE"],
    "row_joins_cell": lambda sp, same_repository, same_number: same_repository and same_number,
    "may_bind": lambda sp, kind: kind in (sp["ID_EXACT"], sp["ID_HASH"]),
    "counts_toward_progress": lambda sp, closed, completed: closed and completed,
    "epic_complete": lambda sp, closed, children, completed: closed and children > 0 and completed == children,
    "marks_done": lambda sp, source: source == sp["DONE_FROM_ACCEPTED"],
    "live_badge_allowed": lambda sp, source: source == sp["SRC_LIVE"],
    "stale_age_s": lambda sp, now, last, has_success, failed: (now - last) if (has_success and failed) else -1,
    "shows_number": lambda sp, read: read,
    "distinct_count": lambda sp, ids, n: len(set(ids[:n])),
    "motion_allowed": lambda sp, reduced: not reduced,
}
SIGS = {   # parameter types and return type of every replayed function, as the spec declares them
    "resolve_address": (["bool", "u32", "bool", "bool"], "u32"), "view_of_address": (["bool", "u32", "bool"], "u32"),
    "presentation": (["u32", "bool", "bool", "bool", "bool"], "u32"), "accept_answer": (["u32", "u32"], "bool"),
    "relation_of": (["u32"], "u32"), "row_joins_cell": (["bool", "bool"], "bool"), "may_bind": (["u32"], "bool"),
    "counts_toward_progress": (["bool", "bool"], "bool"), "epic_complete": (["bool", "u32", "u32"], "bool"),
    "marks_done": (["u32"], "bool"), "live_badge_allowed": (["u32"], "bool"), "stale_age_s": (["i64", "i64", "bool", "bool"], "i64"),
    "shows_number": (["bool"], "bool"), "distinct_count": (["u32x8", "u32"], "u32"), "motion_allowed": (["bool"], "bool"),
}

# The cases where the site does not do what the contract says, and the finding that names each.
KNOWN = {
    "x07-embed-later-missing": "f66", "x08-embed-later-wrong-sha": "f67",
    "j01-board-row-of-another-repository": "f70", "j03-activity-of-another-repository": "f70", "q07-cell-joins-board-by-number": "f70",
    "e01-child-3570-not-planned": "f71", "e01-child-3571-not-planned": "f71", "e02-epic-complete-with-not-planned": "f71", "e03-epic-of-a-bare-number": "f71",
    "q10-foundation-file-as-live": "f75", "q08-missing-capacity": "f76", "q09-factory-live-claim": "f77", "q11-status-late-answer": "f78",
    "q05-reduced-motion-kanban": "f80",
}


def ts_resolution(sp, c, matches):
    if c["ok"]:
        return sp["R_RESOLVED"]
    e = c["error"]
    if "Invalid catalog" in e:
        return sp["R_INVALID"]
    if "SHA-256 mismatch" in e:
        return sp["R_HASH_MISMATCH"]
    if "missing or ambiguous" in e:
        return sp["R_MISSING"] if matches == 0 else sp["R_AMBIGUOUS"]
    if "catalog is empty" in e:
        return sp["R_MISSING"]
    return None


def build_cases(sp, T, B) -> list:
    """(id, function, args, contract value, measured value) from the ts and browser records."""
    t = {c["id"]: c for c in T["result"]["cases"]}
    b = B["result"]
    cases = []

    def add(cid, fn, args, measured):
        cases.append({"id": cid, "fn": fn, "args": list(args), "contract": CONTRACT[fn](sp, *args), "measured": measured})
    R, V = sp["R_RESOLVED"], sp
    # An explicit address, under node's pure functions: grammar, exactly one entry, the bytes.
    for cid, args, matches in (("r01", (True, 1, False, False), 1), ("r02", (True, 0, False, False), 0), ("r03", (True, 0, False, False), 0),
                               ("r04", (True, 0, False, False), 0), ("r05", (True, 0, False, False), 0), ("r07", (False, 0, False, False), 0),
                               ("r08", (False, 0, False, False), 0), ("r09", (False, 0, False, False), 0), ("r10", (True, 1, False, False), 1),
                               ("r11", (True, 2, False, False), 2), ("h01", (True, 1, False, False), 1), ("h02", (False, 1, True, False), 1),
                               ("h03", (False, 1, True, False), 1), ("h04", (False, 1, True, False), 1), ("b01", (True, 1, True, True), 1),
                               ("b02", (True, 1, True, False), 1), ("b03", (True, 1, False, False), 1), ("b04", (True, 0, False, False), 0)):
        add(f"{cid}-{t[cid]['group']}", "resolve_address", args, ts_resolution(sp, t[cid], matches))
    featured = lambda c: V["V_FEATURED"] if c["ok"] else V["V_UNAVAILABLE"]
    add("r06-no-address", "view_of_address", (False, R, True), featured(t["r06"]))
    add("r12-no-address-empty-catalog", "view_of_address", (False, sp["R_MISSING"], True), featured(t["r12"]))
    add("r13-no-address-no-featured", "view_of_address", (False, R, True), featured(t["r13"]))
    # The same addresses in the built page.
    shown = lambda r: V["V_SPEC"] if r.get("shown") or r.get("verified") else None
    unavailable = lambda r: V["V_UNAVAILABLE"] if r.get("error") and not r.get("selected") and not r.get("otherSpecShown") else V["V_OTHER_SPEC"]
    add("x01-exact", "view_of_address", (True, R, True), shown(b["x01-exact"]) if not b["x01-exact"].get("error") else V["V_UNAVAILABLE"])
    for cid in ("x02-missing", "x03-basename", "x04-suffix"):
        add(cid, "view_of_address", (True, sp["R_MISSING"], True), unavailable(b[cid]))
    x5 = b["x05-wrong-sha"]
    add("x05-wrong-sha", "view_of_address", (True, sp["R_HASH_MISMATCH"], True), V["V_UNAVAILABLE"] if x5.get("error") and not x5.get("sourceShown") and not x5.get("verifiedBadge") else V["V_SPEC"])
    add("x06-right-sha", "view_of_address", (True, R, True), shown(b["x06-right-sha"]))
    x7 = b["x07-embed-later-missing"]
    add("x07-embed-later-missing", "view_of_address", (True, sp["R_MISSING"], True), V["V_OTHER_SPEC"] if x7.get("stillShowsOld") and not x7.get("error") else V["V_UNAVAILABLE"])
    x8 = b["x08-embed-later-wrong-sha"]
    add("x08-embed-later-wrong-sha", "view_of_address", (True, sp["R_HASH_MISMATCH"], True), V["V_SPEC"] if x8.get("opened") and not x8.get("mismatchShown") else V["V_UNAVAILABLE"])
    x10 = b["x10-explorer-late-answer"]
    add("x10-explorer-late-answer", "accept_answer", (1, 2), bool(x10.get("sourceIsSlow")) and not x10.get("sourceIsFast"))
    q11 = b["q11-status-late-answer"]
    add("q11-status-late-answer", "accept_answer", (1, 2), bool(q11.get("overwritten")))
    # Identity and joins.
    # A basename or a suffix found in an issue's text: the site lists the hit and labels it unverified
    # (validateAtlas holds coverage to 'unverified'); it binds nothing with it. Exact mentions are left
    # out: the contract lets them bind, it does not make them.
    for cid, repo, kind in (("m01", "t27", "ID_BASENAME"), ("m01", "trinity", "ID_BASENAME"), ("m02", "trinity", "ID_SUFFIX"), ("m03", "t27", "ID_SUFFIX")):
        hit = t[cid]["value"]
        binds = hit[repo] != "none" and hit["coverage"] != ["unverified"]
        add(f"{cid}-{repo}-{kind[3:].lower()}-binds", "may_bind", (sp[kind],), binds)
    j1, j2, j3 = t["j01"]["value"], t["j02"]["value"], t["j03"]["value"]
    add("j01-board-row-of-another-repository", "row_joins_cell", (False, True), bool(j1["joined"]))
    add("j02-board-row-of-the-cell", "row_joins_cell", (True, True), bool(j2["joined"]))
    add("j03-activity-of-another-repository", "row_joins_cell", (False, True), t["j03"]["value"] > 0)
    q7 = b["q07-cell-joins-board-by-number"]
    add("q07-cell-joins-board-by-number", "row_joins_cell", (False, True), bool(q7.get("column")) and q7.get("column") not in ("\u2014", "\u2026") and not q7.get("notOnBoard"))
    add("e03-epic-of-a-bare-number", "may_bind", (sp["ID_NUMBER_ONLY"],), t["e03"]["value"] is not None)
    # Progress and done.
    e1 = t["e01"]["value"]
    for number, closed, completed in ((3563, True, True), (3570, True, False), (3571, True, False), (989, False, False)):
        label = "completed" if completed else ("not-planned" if closed else "open")
        # the site's rule, held by the e01 total: a child counts when its state is closed
        add(f"e01-child-{number}-{label}", "counts_toward_progress", (closed, completed), closed if e1["closed"] == 3 else None)
    add("e02-epic-complete-with-not-planned", "epic_complete", (True, 2, 1), t["e02"]["value"] == "wizardTower")
    for cid, src in (("p01", "DONE_FROM_COMPLETED"), ("p02", "DONE_FROM_COMPLETED"), ("p03", "DONE_FROM_ACCEPTED"), ("p04", "DONE_FROM_MATCH")):
        add(f"{cid}-paint-{t[cid]['input']['state']}-{t[cid]['input']['coverage']}", "marks_done", (sp[src],), t[cid]["value"]["tone"] == "honey")
    q6 = b["q06-catalog-snapshot-label"]
    add("q06-gold-cell-is-a-spec", "marks_done", (sp["DONE_FROM_GOLD_CELL"],), not ("gold = spec, not a resolved issue" in (q6.get("law") or "")))
    # Stale, snapshot and live.
    for cid in ("s01", "s02", "s03", "s04"):
        i = t[cid]["input"]
        v = t[cid]["value"]
        add(f"{cid}-stale", "stale_age_s", (int(i["now_s"]), int(i["last_success_s"]), i["has_success"], i["failed"]), -1 if v is None else int(v))
    add("q06-catalog-is-a-snapshot", "presentation", (sp["SRC_SNAPSHOT"], True, False, False, True), sp["P_SNAPSHOT"] if q6.get("saysSnapshot") else sp["P_LIVE"])
    q10o = b["q10-foundation-other-repository"]
    add("q10-foundation-of-another-repository", "row_joins_cell", (False, True), bool(q10o.get("mounted")))
    q10 = b["q10-foundation-file-folded-in"]
    add("q10-foundation-file-as-live", "presentation", (sp["SRC_SNAPSHOT"], True, False, False, True),
        sp["P_LIVE"] if q10.get("mounted") and q10.get("changed") and q10.get("badge") and not q10.get("dateVisible") else sp["P_SNAPSHOT"])
    q8 = b["q08-missing-capacity"]
    add("q08-missing-capacity", "shows_number", (False,), bool(re.search(r"/\s*0\b", q8.get("bees") or "")))
    add("q09-factory-live-claim", "live_badge_allowed", (sp["SRC_SNAPSHOT"],), bool(b["q09-factory-live-claim"].get("claimsLive")))
    # Counting once.
    add("c01-same-bytes-two-repositories", "distinct_count", ([1, 1, 0, 0, 0, 0, 0, 0], 2), t["c01"]["value"]["specs"])
    add("a01-one-spec-two-worlds", "distinct_count", ([1, 1, 0, 0, 0, 0, 0, 0], 2), t["a01"]["value"]["distinct"])
    # Motion.
    for cid, reduced in (("q04-motion-kanban", False), ("q05-reduced-motion-kanban", True)):
        add(cid, "motion_allowed", (reduced,), (b[cid].get("longer_than_1ms") or 0) > 0)
    return cases


def render(value, typ, lang):
    if typ == "bool":
        return "true" if value else "false"
    if typ == "u32":
        return f"{int(value)}u" if lang == "c" else str(int(value))
    if typ == "i64":
        return f"((int64_t){int(value)})" if lang == "c" else f"@as(i64, {int(value)})"
    if typ == "u32x8":
        items = ", ".join(str(int(x)) for x in value)
        return f"(uint32_t[8]){{{items}}}" if lang == "c" else f"[8]u32{{ {items} }}"
    raise ValueError(typ)


def expr_of(case, lang):
    params, ret = SIGS[case["fn"]]
    args = ", ".join(render(v, ty, lang) for v, ty in zip(case["args"], params))
    return f"{case['fn']}({args}) == {render(case['contract'], ret, lang)}"


def replay_c(cases, t27c, spec=None) -> dict:
    with tempfile.TemporaryDirectory() as tmp:
        r = base.replay_spec(pathlib.Path(spec or SPEC), [(c["id"], [(expr_of(c, "c"), c["fn"])]) for c in cases], t27c, pathlib.Path(tmp) / "views")
    r["spec_sha256"] = base.sha256(pathlib.Path(spec or SPEC).read_bytes())
    return r


def replay_zig(cases, t27c, zig, sysroot, spec=None) -> dict:
    """The same cases through the Zig backend. `str` is aliased to []const u8 first: the backend emits
    `[N]str` for a string array and declares no `str` (the known gap every [N]str spec hits)."""
    spec = pathlib.Path(spec or SPEC)
    rc, out, err = base.run([str(t27c), "gen", str(spec)])
    if rc != 0 or not out.strip():
        return {"ok": False, "stage": "generate", "detail": (err or out)[:300], "passed": 0, "failed": 0, "failures": []}
    zsrc = out.replace('const std = @import("std");', 'const std = @import("std");\nconst str = []const u8;', 1)
    tests = "\n".join(f'test "{c["id"]}" {{\n    try std.testing.expect({expr_of(c, "zig")});\n}}' for c in cases)
    with tempfile.TemporaryDirectory() as tmp:
        f = pathlib.Path(tmp) / "views.zig"
        f.write_text(zsrc + "\n" + tests + "\n", encoding="utf-8")
        cmd = [zig, "test"] + (["--sysroot", sysroot] if sysroot else []) + [str(f)]
        rc, o, e = base.run(cmd, cwd=tmp, timeout=1800)
    text = o + e
    passed = re.search(r"All (\d+) tests passed", text)
    # One line per test: "3/70 views.test.<name>...OK", "...FAIL (...)", or "..." and nothing more when an
    # assert of the spec's own tests panics and takes the runner down with it.
    fails = []
    for m in re.finditer(r"^\d+/\d+ \S+?\.test\.(.+?)\.\.\.(.*)$", text, re.M):
        if m.group(2).strip() != "OK":
            fails.append(m.group(1))
    total = len(cases) + len(re.findall(r"^test \w+ \{", spec.read_text(encoding="utf-8"), re.M))   # the spec's own tests run with them
    ok = rc == 0 and passed is not None and int(passed.group(1)) == total
    return {"ok": ok, "stage": "runtime" if passed or fails else "compile", "passed": int(passed.group(1)) if passed else 0,
            "failed": len(fails), "failures": [{"id": x} for x in fails][:20], "detail": "" if ok else text[-400:], "fixtures": total,
            "spec_sha256": base.sha256(spec.read_bytes()), "generated": base.sha256(zsrc.encode())}


def replay(cases, t27c, zig=None, sysroot=None, spec=None) -> dict:
    out = {"at": base.now(), "host": base.host(), "cases": len(cases), "c": replay_c(cases, t27c, spec)}
    if zig:
        out["zig"] = replay_zig(cases, t27c, zig, sysroot, spec)
        out["zig_version"] = base.zig_version(zig)
    return out


# ===========================================================================================
# CHECK
# ===========================================================================================
EVIDENCE_TAGS = {"ts", "data", "live", "gates", "browser", "native", "source", "spec"}


def project_binding_ok(l) -> bool:
    p = base.load_spec(PROJECT_SPEC)
    cur = p.get("WORK_PACKAGE_CURRENT_ISSUES") or []
    if len(cur) != len(p.get("WORK_PACKAGES", [])):
        return False
    for i, wp in enumerate(p["WORK_PACKAGES"]):
        if wp in l["refiled"]:
            r = l["refiled"][wp]
            if cur[i] != f"https://github.com/{r['repo']}/issues/{r['number']}":
                return False
        elif cur[i] != p["WORK_PACKAGE_ISSUES"][i]:
            return False
    return True


def gates_ok(g, sp) -> bool:
    """Every gate that fails here is one the spec names, and of those exactly the named ones are run by a
    workflow (check:docs runs only in the nightly scan, after the scan has regenerated what it checks)."""
    failing = sorted(k for k, v in g["gates"].items() if v["verdict"] in ("fail", "timeout"))
    in_a_workflow = sorted(k for k in failing if g["gates"][k]["ci"])
    named = sp["GATES_FAILING_RUN_BY_A_WORKFLOW"]
    named = [x.strip() for x in named.split(",") if x.strip()] if isinstance(named, str) else list(named)
    return failing == sorted(sp["GATES_FAILING_AT_PIN"]) and in_a_workflow == sorted(named)


def record_claims(sp, T, D, L, G, B, N) -> list:
    """The facts the spec's findings rest on, written out by hand. (claim id, finding, text, lambda)."""
    t = {c["id"]: c for c in T["result"]["cases"]}
    d, l, g, b, n = D["result"], L["result"], G["result"], B["result"], N["result"]
    src = d["source"]
    C = []

    def add(cid, fnd, text, fn):
        C.append((cid, fnd, text, fn))
    add("v01", "f65", "the explorer resolves an exact path only: a basename, a suffix, a case variant, a duplicate and a malformed hash refuse, a wrong hash refuses the bytes, a right one is shown verified, and the late bytes of an earlier pick are dropped",
        lambda: all(not t[i]["ok"] for i in ("r02", "r03", "r04", "r05", "r07", "r08", "r09", "r11", "h02", "h03", "h04", "b02", "b04")) and t["r01"]["ok"] and t["b01"]["ok"]
        and b["x06-right-sha"]["verified"] and b["x10-explorer-late-answer"]["sourceIsFast"] and not b["x10-explorer-late-answer"]["sourceIsSlow"])
    add("v02", "f66", "in a frame a later address naming an unknown spec keeps the open spec under the new address, with no error",
        lambda: b["x07-embed-later-missing"]["selected"] == "specs/demos/hello_world.t27" and "missing.t27" in b["x07-embed-later-missing"]["hash"]
        and not b["x07-embed-later-missing"]["error"] and src["embedded_unknown_keeps_open_spec"]["count"] == 1)
    add("v03", "f67", "in a frame a later address with a wrong sha256 opens the spec unverified and the address loses its hash",
        lambda: b["x08-embed-later-wrong-sha"]["opened"] and not b["x08-embed-later-wrong-sha"]["mismatchShown"]
        and "sha256" not in b["x08-embed-later-wrong-sha"]["hash"] and src["address_sha256_read_once"]["count"] == 1)
    add("v04", "f68", "no manifest entry carries a sha256 or a revision, most name their repository by a bare name, and a plain address loads its bytes unverified",
        lambda: d["manifest"]["entries"] == sp["MANIFEST_ENTRIES"] and d["manifest"]["entries_with_sha256"] == sp["MANIFEST_ENTRIES_WITH_SHA256"] == 0
        and d["manifest"]["entries_with_revision"] == 0 and d["manifest"]["entries_bare_repo"] == sp["MANIFEST_ENTRIES_BARE_REPO"] and t["b03"]["ok"])
    add("v05", "f69", "a basename or a repository-stripped path in a gHashTag/t27 issue is a 'reference' hit of the gHashTag/trinity spec, and the reverse; every hit stays unverified",
        lambda: t["m01"]["value"] == {"t27": "reference", "trinity": "reference", "coverage": ["unverified"]} and t["m02"]["value"]["trinity"] == "reference"
        and t["m03"]["value"]["t27"] == "reference" and t["m04"]["value"]["coverage"] == [])
    add("v06", "f70", "the live board answered for one repository; catalog cells of other repositories share a number with its cards, and the close-up of one shows that card's column",
        lambda: bool(l["board"]["repo"]) and len(l["collisions"]) == sp["CELLS_JOINED_TO_ANOTHER_REPOSITORY"] and src["board_card_by_number"]["count"] == 1
        and src["activity_by_number"]["count"] == 1 and src["cell_live_by_number"]["count"] == 1 and b["q07-cell-joins-board-by-number"]["column"] not in (None, "\u2014", "\u2026"))
    add("v07", "f71", "an epic's progress counts every closed child, not-planned ones too; a child carries no repository; the castle's children are body references, not sub-issues",
        lambda: t["e01"]["value"]["closed"] == 3 and t["e02"]["value"] == "wizardTower" and t["e03"]["value"] == 988 and src["epic_child_without_repository"]["count"] == 1
        and src["progress_counts_every_closed"]["count"] >= 1 and d["foundation"]["rule"]["children"] == "body references in either direction"
        and "repo" not in d["foundation"]["child_fields"])
    add("v08", "f72", "the epic's sub-issues name S08-S11 by issues closed not planned; the re-filed issues are not sub-issues of it; project.t27 now binds the re-filed ones",
        lambda: [(e["number"], e["reason"]) for e in l["epic"]["sub_issues"] if e["number"] in (3570, 3571, 3572, 3573)] == [(3570, "not_planned"), (3571, "not_planned"), (3572, "not_planned"), (3573, "not_planned")]
        and all(not r["sub_issue_of_epic"] and r["refiled_from"] == 3570 + i for i, r in enumerate(l["refiled"][k] for k in ("S08", "S09", "S10", "S11")))
        and project_binding_ok(l))
    add("v09", "f73", "the atlas froze its open issues at 2026-09-24T17:24:32Z and some are closed on the day; it has no state field, accepts any age, and the map shows no time until a cell is selected",
        lambda: d["atlas"]["at"] == "2026-09-24T17:24:32.420Z" and d["atlas"]["issues"] == sp["ATLAS_ISSUES"] == l["issues"]
        and sum(v for k, v in l["states"].items() if k.startswith("CLOSED")) == sp["ATLAS_ISSUES_CLOSED_SINCE"] == len(l["closed"])
        and not d["atlas"]["issues_carry_state"] and t["a04"]["ok"] and t["a02"]["ok"] is False
        and b["q06-catalog-snapshot-label"]["saysSnapshot"] and not b["q06-catalog-snapshot-label"]["dateVisible"])
    add("v10", "f74", "the catalog map merges issues read live from GitHub into the snapshot rows by key, under the one label 'Public snapshot, not live'",
        lambda: src["catalog_rows_merge_observed"]["count"] == 1 and src["catalog_snapshot_label"]["count"] >= 1)
    add("v11", "f75", "the foundation endpoint answers 404 and the page reads the 2026-09-05 file of gHashTag/trios every minute; beside the live board of another repository it is dropped, beside a board of its own a changed file is taken on the next poll with no visible date",
        lambda: l["endpoints"]["public-foundation"]["status"] == 404 and d["foundation"]["generatedAt"] == "2026-09-05T14:18:58Z" and d["foundation"]["repo"] == "gHashTag/trios"
        and b["q10-foundation-other-repository"]["shellUp"] and not b["q10-foundation-other-repository"]["mounted"]
        and b["q10-foundation-file-folded-in"]["mounted"] and b["q10-foundation-file-folded-in"]["changed"] and not b["q10-foundation-file-folded-in"]["dateVisible"]
        and b["q10-foundation-file-folded-in"]["fileRequests"] >= 2 and src["foundation_wire_then_file"]["count"] == 1)
    add("v12", "f76", "with no workers in the status and research unreachable, BEES prints the running count over 0",
        lambda: re.search(r"/\s*0\b", b["q08-missing-capacity"]["bees"] or "") is not None and src["worker_capacity_zero"]["count"] >= 1)
    add("v13", "f77", "the factory prints that every laboratory is backed by the live Queen ledger",
        lambda: b["q09-factory-live-claim"]["claimsLive"] and src["factory_live_claim"]["count"] >= 1)
    add("v14", "f78", "an older status answer that lands after a newer one replaces it; no poll of Queen.tsx is aborted or ordered; nothing reads navigator.onLine",
        lambda: b["q11-status-late-answer"]["overwritten"] and "FIXTURE-NEWER" in (b["q11-status-late-answer"]["afterNewer"] or "")
        and src["queen_polls_abort_controllers"]["count"] == 0 and src["no_online_handling"]["count"] == 0)
    add("v15", "f79", "a module card's number is the hash of its path, and the OPEN ISSUE link is built from the picked card's number",
        lambda: t["e04"]["ok"] and t["e04"]["value"] > 100000 and src["module_card_number_is_hash"]["count"] == 1 and src["picked_issue_url_from_card_number"]["count"] == 1)
    add("v16", "f80", "keys on a desktop and with reduced motion, and taps on a phone, open kanban, project and comb, the address follows, nothing scrolls sideways; with reduced motion every CSS animation stops and one script animation still plays",
        lambda: all(s.get("reached") for k in ("q01-nav-desktop", "q02-nav-mobile", "q03-nav-reduced-motion") for s in b[k]["seen"])
        and all(b[k]["layout"]["scrollW"] <= b[k]["layout"]["clientW"] for k in ("q01-nav-desktop", "q02-nav-mobile", "q03-nav-reduced-motion"))
        and b["q03-nav-reduced-motion"]["layout"]["reduce"] and src["no_framer_motion_config"]["count"] == 0 and src["queen_uses_framer_motion"]["count"] == 1
        and b["q04-motion-kanban"]["kinds"].get("CSSAnimation", 0) > 0 and b["q05-reduced-motion-kanban"]["kinds"].get("CSSAnimation", 0) == 0
        and b["q05-reduced-motion-kanban"]["kinds"].get("Animation", 0) >= 1)
    add("v17", "f81", "the Russian page shows the explorer's refusal in English; the refusal has no role while the verified line is a status; Queen.tsx has no aria-live",
        lambda: b["x09-missing-ru"]["error"] and b["x09-missing-ru"]["english"] and src["explorer_error_in_english_only"]["count"] == 1
        and src["explorer_alerts"]["count"] == 0 and src["explorer_status_on_success"]["count"] == 1 and src["queen_aria_live"]["count"] == 0)
    add("v18", "f82", "the gates that fail here are the ones the spec names, and only the named ones among them are run by a workflow",
        lambda: g["scripts"] == sp["SITE_GATES"] and g["run_by_ci"] == sp["SITE_GATES_RUN_BY_CI"] and gates_ok(g, sp))
    add("v19", "f83", "apps/queen builds with no error and its tests pass; none of its sources names the catalog, the atlas, GitHub or a public Queen endpoint",
        lambda: n["build"]["verdict"] == "pass" and n["build"]["errors"] == 0 and n["test"]["tests"] == sp["NATIVE_TESTS"] and n["catalog_or_live_references"] == 0)
    add("v20", "f84", "the site's docs generator refuses an unknown constant; specs/ui/queen_evidence.t27 was in the mirror only and is now here byte for byte",
        lambda: len(d["docs_schema_closed_at_lines"]) >= 1 and d["mirror"]["queen_evidence_in_t27_at_manifest_revision"] is False
        and "specs/ui/queen_evidence.t27" in d["mirror"]["files_not_in_manifest"] and EVIDENCE_SPEC.exists()
        and hexsha(EVIDENCE_SPEC.read_bytes()) == d["mirror"]["queen_evidence_sha256"])
    add("v21", "f85", "the mirror is t27 at the revision the manifest names, file for file; the same bytes in two places are one spec; displayed totals count distinct ids",
        lambda: d["mirror"]["identical_at_manifest_revision"] + d["mirror"]["absent_at_manifest_revision"] == d["mirror"]["t27_entries"]
        and d["mirror"]["differ_at_manifest_revision"] == 0 and d["mirror"]["absent_at_manifest_revision"] == d["mirror"]["unreachable_listed"]
        and t["c01"]["value"] == {"specs": 1, "sources": [2]} and t["a01"]["value"]["sumOfWorlds"] == 2 and t["a01"]["value"]["distinct"] == 1
        and d["atlas"]["specs"] == sp["ATLAS_SPECS"] and d["atlas"]["worlds"] == sp["ATLAS_WORLDS"])
    add("v22", "f86", "no issue cell can be honey: every row is built with coverage 'unknown' and honey needs 't27'; a gold cell is a spec and the map says so",
        lambda: src["display_coverage_is_unknown_by_type"]["count"] == 1 and src["honey_needs_t27_coverage"]["count"] == 1 and t["p01"]["value"]["tone"] != "honey"
        and t["p03"]["value"]["tone"] == "honey" and "gold = spec, not a resolved issue" in (b["q06-catalog-snapshot-label"]["law"] or ""))
    return C


def check_findings(sp: dict) -> list:
    f = []
    items = sp.get("FINDINGS", [])
    if sp.get("FINDINGS_COUNT") != len(items):
        f.append(f"views: FINDINGS_COUNT {sp.get('FINDINGS_COUNT')} but {len(items)} findings")
    seen = set()
    for it in items:
        m = re.match(r"(f\d\d) ", it)
        tag = re.search(r"\[([^\]]*)\]\s*$", it)
        if not m or not tag:
            f.append(f"views: a finding without an id or evidence: {it[:50]}")
            continue
        if m.group(1) in seen:
            f.append(f"views: {m.group(1)} twice")
        seen.add(m.group(1))
        for part in [x.strip() for x in tag.group(1).split(";")]:
            if part not in EVIDENCE_TAGS:
                f.append(f"views {m.group(1)}: unrecognised evidence '{part}'")
    return f


def finding_ids(sp) -> set:
    return {m.group(1) for it in sp.get("FINDINGS", []) for m in [re.match(r"(f\d\d) ", it)] if m}


def judge_cases(cases, sp) -> list:
    f = []
    ids = finding_ids(sp)
    seen = {c["id"] for c in cases}
    for c in cases:
        if c["measured"] is None:
            f.append(f"case {c['id']}: the record does not say what the site did")
            continue
        parts = c["measured"] != c["contract"]
        known = KNOWN.get(c["id"])
        if parts and not known:
            f.append(f"case {c['id']}: the site did {c['measured']!r}, the contract says {c['contract']!r}, and no finding names it")
        elif known and not parts:
            f.append(f"case {c['id']}: listed under {known}, but the site now does what the contract says; update the spec")
        elif known and known not in ids:
            f.append(f"case {c['id']}: names {known}, which the spec's FINDINGS does not hold")
    for cid in KNOWN:
        if cid not in seen:
            f.append(f"KNOWN names {cid}, which no case has")
    return f


def probe_findings(T, B) -> list:
    """A record speaks for the probe that made it, not for the probe in this file."""
    return [f"{name}: the record was made by another version of the probe; run `{name}`"
            for name, rec, probe in (("ts", T, PROBE_LIBS), ("browser", B, PROBE_BROWSER)) if rec.get("probe_sha256") != hexsha(probe.encode())]


def check_all(t27c=None) -> list:
    if not SPEC.exists():
        return [f"{SPEC}: missing"]
    recs = {k: load(k) for k in KINDS}
    missing = [k for k, v in recs.items() if v is None]
    if missing:
        return [f"{k}: no record; run `{k}`" for k in missing]
    T, D, L, G, B, N = (recs[k] for k in KINDS)
    f = []
    for k, v in recs.items():
        if (v.get("trinity") or {}).get("head") != TRINITY_PIN:
            f.append(f"{k}: the record is not at the trinity pin")
    sp = base.load_spec(SPEC)
    if sp.get("PINNED_REVISION") != TRINITY_PIN:
        f.append("views: the spec's pin differs from the tool's")
    f += probe_findings(T, B)
    cases = build_cases(sp, T, B)
    f += judge_cases(cases, sp)
    for cid, fnd, text, fn in record_claims(sp, T, D, L, G, B, N):
        try:
            ok = bool(fn())
        except (KeyError, TypeError, IndexError, StopIteration, AttributeError) as e:
            ok, text = False, f"{text} ({type(e).__name__} {e})"
        if not ok:
            f.append(f"record {cid} [{fnd}]: {text}")
        elif fnd not in finding_ids(sp):
            f.append(f"record {cid}: names {fnd}, which the spec's FINDINGS does not hold")
    f += check_findings(sp)
    t27c = t27c or base.t27c_path()
    if t27c and shutil.which("cc"):
        own = run_spec_tests(t27c, SPEC)
        if own["tests"] != len(re.findall(r"^test \w+ \{", SPEC.read_text(encoding="utf-8"), re.M)):
            f.append(f"views.t27: its own tests in C: {own}")
        r = T.get("replay")
        if not r:
            f.append("replay: none recorded; run `run`")
        else:
            if r["c"].get("spec_sha256") != base.sha256(SPEC.read_bytes()):
                f.append("replay: the spec changed since the replay; run `run`")
            if r["cases"] != len(cases):
                f.append(f"replay: {r['cases']} cases replayed, the records make {len(cases)}; run `run`")
            for lang in ("c", "zig"):
                x = r.get(lang)
                if not x:
                    f.append(f"replay: no {lang} replay recorded")
                elif not x.get("ok"):
                    f.append(f"replay {lang}: {x.get('passed')} passed, {x.get('failed')} failed: {str(x.get('failures', []))[:160]} {x.get('detail', '')[:120]}")
    else:
        f.append("check: t27c or cc missing, the spec's tests and the replay were not checked")
    return f


def run_spec_tests(t27c, path: pathlib.Path) -> dict:
    with tempfile.TemporaryDirectory() as tmp:
        c = pathlib.Path(tmp) / "s.c"
        rc, out, err = base.run([str(t27c), "gen-c", str(path)])
        if rc != 0 or not out.strip():
            return {"generates": False, "compiles": False, "tests": None}
        c.write_text(out)
        rc, o2, e2 = base.run(["cc", "-std=c11", "-w", "-DT27_TEST_MAIN", str(c), "-o", str(pathlib.Path(tmp) / "s")])
        if rc != 0:
            return {"generates": True, "compiles": False, "tests": None, "detail": e2[:200]}
        rc, o3, e3 = base.run([str(pathlib.Path(tmp) / "s")], timeout=60)
        m = re.search(r"All (\d+) tests passed", o3)
        return {"generates": True, "compiles": True, "tests": int(m.group(1)) if m and rc == 0 else f"failed rc={rc}"}


# ===========================================================================================
# --self-check: every gate here has been seen to fail
# ===========================================================================================
def self_check(zig=None, sysroot=None) -> int:
    ok = True

    def expect(cond, what):
        nonlocal ok
        print(f"  {'ok ' if cond else 'BAD'} {what}")
        ok = ok and bool(cond)
    clone = lambda d: json.loads(json.dumps(d))
    recs = {k: load(k) for k in KINDS}
    expect(all(v is not None for v in recs.values()), "the six committed records exist")
    if not all(v is not None for v in recs.values()):
        return 1
    T, D, L, G, B, N = (recs[k] for k in KINDS)
    sp = base.load_spec(SPEC)
    f = check_all()
    expect(not f, f"check: records, spec, cases, claims and replay agree ({len(f)} finding(s){': ' + f[0] if f else ''})")
    judged = lambda T_=T, B_=B: judge_cases(build_cases(sp, T_, B_), sp)
    claims_of = lambda **kw: [cid for cid, _, _, fn in record_claims(sp, kw.get("T", T), kw.get("D", D), kw.get("L", L), kw.get("G", G), kw.get("B", B), kw.get("N", N)) if not _safe(fn)]
    b = clone(B)
    b["result"]["x07-embed-later-missing"].update({"stillShowsOld": False, "error": True})
    expect(any("x07" in x and "now does what the contract says" in x for x in judged(B_=b)), "planted: a frame that refuses the unknown address is reported as a stale finding")
    t = clone(T)
    for c in t["result"]["cases"]:
        if c["id"] == "r03":
            c.update({"ok": True, "value": "specs/demos/hello_world.t27"})
    expect(any("r03" in x and "no finding names it" in x for x in judged(T_=t)), "planted: a resolver that answers a basename is an unlisted divergence")
    t = clone(T)
    for c in t["result"]["cases"]:
        if c["id"] == "j01":
            c["value"] = {"joined": False, "column": None}
    expect(any("j01" in x for x in judged(T_=t)), "planted: a live join that checks the repository is reported (the finding would be gone)")
    l_ = clone(L)
    l_["result"]["closed"] = l_["result"]["closed"][:-1]
    expect("v09" in claims_of(L=l_), "planted: one closed issue fewer breaks v09")
    g = clone(G)
    name = next(k for k, v in g["result"]["gates"].items() if v["ci"])
    g["result"]["gates"][name]["verdict"] = "fail"
    expect("v18" in claims_of(G=g), f"planted: a CI gate failing here ({name}) breaks v18")
    n = clone(N)
    n["result"]["catalog_or_live_references"] = 1
    expect("v19" in claims_of(N=n), "planted: a native source that reads the catalog breaks v19")
    b = clone(B)
    b["probe_sha256"] = "0" * 64
    expect(any("another version of the probe" in x for x in probe_findings(T, b)), "planted: a browser record made by another probe is refused")
    s2 = clone(sp)
    s2["FINDINGS"][0] = s2["FINDINGS"][0].replace("[", "[rumour; ", 1)
    expect(any("rumour" in x for x in check_findings(s2)), "planted: a finding with evidence nobody measured")
    t27c = base.t27c_path()
    if t27c and shutil.which("cc"):
        cases = build_cases(sp, T, B)
        good = replay_c(cases, t27c)
        expect(good["ok"], f"replay C: {good['passed']}/{good.get('fixtures')} cases")
        with tempfile.TemporaryDirectory() as tmp:
            tmp = pathlib.Path(tmp)

            def planted(old, new):
                text = SPEC.read_text(encoding="utf-8")
                assert text.count(old) == 1, old
                pp = tmp / SPEC.name
                pp.write_text(text.replace(old, new), encoding="utf-8")
                return pp
            for old, new, want, what in (
                    ("    if (hash_given and hash_equal == false) { return R_HASH_MISMATCH; }\n", "", "b02-bytes", "a resolver that ignores the hash"),
                    ("    return same_repository and same_number;", "    return same_number;", "j01-board-row-of-another-repository", "a join by number alone"),
                    ("    return closed and completed;", "    return closed;", "e01-child-3570-not-planned", "progress that counts every closed child"),
                    ("    if (source == SRC_SNAPSHOT) { return P_SNAPSHOT; }\n", "", "q06-catalog-is-a-snapshot", "a snapshot presented as live"),
                    ("            if (ids[j] == ids[i]) { seen = true; }\n", "", "c01-same-bytes-two-repositories", "a count of placements, not of ids"),
                    ("    if (has_success == false or failed == false) { return -1; }\n", "    if (has_success == false) { return -1; }\n", "s02-stale", "a stale badge while requests succeed"),
                    ("    if (resolution != R_RESOLVED) { return V_UNAVAILABLE; }\n", "", "x07-embed-later-missing", "an unresolved address that shows a spec")):
                r = replay_c(cases, t27c, spec=planted(old, new))
                expect(any(x["id"] == want for x in r["failures"]), f"planted: {what} fails {want} in C")
            if zig:
                z = replay_zig(cases, t27c, zig, sysroot)
                expect(z["ok"], f"replay Zig: {z['passed']}/{z.get('fixtures')}")
                z = replay_zig(cases, t27c, zig, sysroot, spec=planted("    return same_repository and same_number;", "    return same_number;"))
                caught = [x["id"] for x in z["failures"]]
                expect(not z["ok"] and any("j01" in x or "never_joins" in x for x in caught), f"planted: a join by number alone fails in Zig ({', '.join(caught)[:80]})")
            else:
                print("  skip Zig replay checks: no --zig")
    else:
        print("  skip replay checks: t27c or cc missing")
    print("trinity_queen_views --self-check:", "ok" if ok else "FAILED")
    return 0 if ok else 1


def _safe(fn):
    try:
        return bool(fn())
    except (KeyError, TypeError, IndexError, StopIteration, AttributeError):
        return False


# ===========================================================================================
# CLI
# ===========================================================================================
def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("command", nargs="?", choices=["ts", "data", "live", "gates", "browser", "native", "run", "check"])
    ap.add_argument("--trinity-root")
    ap.add_argument("--bun", default=str(BUN_DEFAULT))
    ap.add_argument("--chrome")
    ap.add_argument("--zig")
    ap.add_argument("--sysroot")
    ap.add_argument("--self-check", action="store_true")
    a = ap.parse_args()
    if a.self_check:
        return self_check(a.zig, a.sysroot)
    if a.command is None:
        ap.print_help()
        return 2
    OUT.mkdir(parents=True, exist_ok=True)
    builders = {"ts": lambda tr: build_ts_record(tr, a.bun), "data": build_data_record, "live": build_live_record,
                "gates": lambda tr: build_gates_record(tr, find_chrome(a.chrome)), "native": build_native_record,
                "browser": lambda tr: build_browser_record(tr, find_chrome(a.chrome) or (_ for _ in ()).throw(RuntimeError("no Chrome found; pass --chrome")))}
    if a.command in builders:
        if not a.trinity_root:
            print(f"{a.command}: --trinity-root is required", file=sys.stderr)
            return 2
        try:
            doc = builders[a.command](pathlib.Path(a.trinity_root).resolve())
        except (RuntimeError, OSError, subprocess.TimeoutExpired, KeyError, ValueError) as e:
            print(f"{a.command}: could not run: {e}", file=sys.stderr)
            return 2
        if a.command == "ts":
            old = load("ts") or {}
            if "replay" in old:
                doc["replay"] = old["replay"]
        wr(REC[a.command], doc)
        print(f"{a.command}: written {REC[a.command].relative_to(ROOT)}")
        return 0
    if a.command == "run":
        t27c = base.t27c_path()
        T, B = load("ts"), load("browser")
        if not (t27c and shutil.which("cc") and T and B):
            print("run: needs t27c, cc and the ts and browser records", file=sys.stderr)
            return 2
        sp = base.load_spec(SPEC)
        cases = build_cases(sp, T, B)
        T["replay"] = replay(cases, t27c, a.zig, a.sysroot)
        wr(REC["ts"], T)
        r = T["replay"]
        for lang in ("c", "zig"):
            if lang in r:
                x = r[lang]
                print(f"run {lang}: {x['passed']} passed, {x['failed']} failed of {x.get('fixtures')} {str(x.get('failures', []))[:200]}")
        return 0 if r["c"]["ok"] and r.get("zig", {"ok": True})["ok"] else 1
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
