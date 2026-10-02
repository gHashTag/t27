#!/usr/bin/env python3
"""The Queen's task lifecycle -- what the running supervisor does with an issue, from candidate to
done -- held to specs/queen/dispatch.t27, by running the supervisor's own code.

WHY THIS EXISTS
---------------
S09 of gHashTag/trinity#988 (gHashTag/t27#4828, re-filed from #3571). This repository describes at
least four Queen "cycles": six phases in gHashTag/trinity AGENTS.md (Plan, Assign, Run, Test, Verdict,
Evolve), six in specs/queen/lotus.t27 (Observe, Recall, Evaluate, Plan, Act, Record), five stages in
src/tri/queen/lotus_cycle.zig, and the round of the trios supervisor that actually dispatches workers
("bees") onto GitHub issues (gHashTag/BrowserOS, trios/agent-server). Only the last one leases, retries,
reviews and finishes tasks. specs/queen/dispatch.t27 states that lifecycle as the canonical machine and
names the others as adapters; this tool measures the supervisor at its pin and holds the spec to it.

FIVE KINDS OF EVIDENCE, KEPT APART
----------------------------------
  ts       the supervisor's own TypeScript, imported and called under bun 1.3.6: the claim a dispatch
           row exerts (stateOfDispatch over a grid of 2112 rows), the public board's column for a row
           (composeCards), the single-flight round gate under concurrent requests
  queend   the supervisor's Swift policy binary, built from queen-core at the pin and asked the
           questions the tick asks: choose (every order of five candidates), review (a grid), retry
  pg       a throwaway PostgreSQL 16 with the supervisor's own migrations: the singleton lease under 32
           concurrent contenders, expiry and fencing; the dispatch writers (recordDispatch,
           finishDispatch, the reapers) on real rows; the board query the round actually runs
  lotus    gHashTag/trinity src/tri/queen/*.zig at the pin compiled with Zig 0.15.2 and its tests run
  live     one public snapshot of trios-agent-server-production: /queen/status and /queen/public-board
           (vocabulary and counts only; labelled a snapshot, never asserted to stay true)
No credential is used anywhere: queend needs none, the database is local and throwaway, and the two
live endpoints are public.

Usage:
  python3 tools/trinity_queen_dispatch.py ts     --browseros-root <checkout at PIN> --bun <bun> --queend <queend>
  python3 tools/trinity_queen_dispatch.py pg     --browseros-root <checkout> --bun <bun> --database-url <url>
  python3 tools/trinity_queen_dispatch.py lotus  --trinity-root <clone> --zig <zig> [--sysroot <dir>]
  python3 tools/trinity_queen_dispatch.py live
  python3 tools/trinity_queen_dispatch.py run
  python3 tools/trinity_queen_dispatch.py check
  python3 tools/trinity_queen_dispatch.py --self-check

Exit codes: 0 no finding; 1 findings; 2 could not run.
"""
from __future__ import annotations

import argparse
import itertools
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile
import urllib.request

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import trinity_tri_api as base  # noqa: E402  (shared helpers: run, sha256, load_spec, replay_spec, git_show)

ROOT = base.ROOT
SPEC = ROOT / "specs/queen/dispatch.t27"
TASK_SPEC = ROOT / "specs/queen/task_analysis.t27"
OUT = ROOT / "conformance/trinity"
REC = {k: OUT / f"queen_dispatch_{k}.json" for k in ("ts", "queend", "pg", "lotus", "live")}
BROWSEROS_REPO = "gHashTag/BrowserOS"
BROWSEROS_BRANCH = "feat/queen-supervisor"
BROWSEROS_PIN = "c25e1b0278a690745c8b6ab591a37d2885656573"
TRINITY_PIN = base.DEFAULT_REV
SERVER = "trios/agent-server/apps/server"
LIVE_BASE = "https://trios-agent-server-production.up.railway.app"

VERDICTS = [None, "", "wait", "empty", "sendBack", "escalate", "accept", "failed", "cancelled", "stale-contract", "unrecognised"]
MIN = 60 * 1000
IDLES = [0, 30 * MIN - 1, 30 * MIN, 60 * MIN - 1, 60 * MIN, 6 * 60 * MIN - 1, 6 * 60 * MIN, 48 * 60 * MIN]
SEND_BACKS = [0, 1, 2, 3]
RELEASES = [0, 1, 2]

# ===========================================================================================
# TS: the supervisor's own functions under bun
# ===========================================================================================
TS_FIXTURE = r"""
import { stateOfDispatch, dispatchRowState, boardTask, createRoundGate,
         SEND_BACK_IDLE_FLOOR_MS, WAIT_FROZEN_FLOOR_MS, EMPTY_ATTEMPT_FLOOR_MS, CEILING_RELEASE_MS, MAX_CEILING_RELEASES } from '@SERVER@/src/api/services/queen-tick.ts'
import { composeCards } from '@SERVER@/src/api/routes/queen-kanban.ts'
import { spawnSync } from 'node:child_process'
import { readFileSync } from 'node:fs'

const input = JSON.parse(readFileSync(process.argv[2], 'utf8'))
const out: Record<string, unknown> = {}
out.constants = { SEND_BACK_IDLE_FLOOR_MS, WAIT_FROZEN_FLOOR_MS, EMPTY_ATTEMPT_FLOOR_MS, CEILING_RELEASE_MS, MAX_CEILING_RELEASES }

// 1. the claim every dispatch row exerts, over the whole grid
const claims: unknown[] = []
for (const finished of [false, true])
  for (const verdict of input.verdicts)
    for (const idle of input.idles)
      for (const sb of input.sendBacks)
        for (const rel of input.releases)
          claims.push([finished, verdict, idle, sb, rel, stateOfDispatch(finished, verdict, { idleMs: idle, sendBacks: sb, releases: rel })])
out.claims = claims

// 2. the public board's column for a finished row, by issue openness
const now = Date.parse('2026-10-01T00:00:00Z')
const fin = new Date(now - 2 * 3600e3).toISOString()
const disp = new Date(now - 3 * 3600e3).toISOString()
function row(issue: number, finished: boolean, review: string | null, extra: Record<string, unknown> = {}) {
  return { issue, branch: 'queen-' + issue, started: true, detail: 'x', finished_at: finished ? fin : null, outcome: finished ? 'finished' : null,
           review_state: review, review_note: null, owned_paths: ['src/f' + issue + '.ts'], dispatched_at: disp, send_backs: 0, ...extra }
}
const boardCases: Record<string, unknown> = {}
const verdictsForBoard = ['accept', 'escalate', 'sendBack', 'wait', 'empty', null]
for (const v of verdictsForBoard) {
  for (const open of [true, false]) {
    const r = row(5, true, v)
    const cards = composeCards({ tasks: [], dispatches: [r], issues: [{ number: open ? 5 : 6, title: 't', owned_paths: [], criteria: [], criteria_source: 'none', missing: [] }], now })
    const card = (cards as Array<{ number: number; column: string }>).find((c) => c.number === 5)
    boardCases[(v ?? 'null') + (open ? '|open' : '|closed')] = card ? card.column : null
  }
}
{
  const r = row(5, false, null)
  const cards = composeCards({ tasks: [], dispatches: [r], issues: [{ number: 6, title: 't', owned_paths: [], criteria: [], criteria_source: 'none', missing: [] }], now })
  const card = (cards as Array<{ number: number; column: string }>).find((c) => c.number === 5)
  boardCases['running|closed'] = card ? card.column : null
}
for (const rel of [undefined, 1]) {
  const r = row(5, true, 'sendBack', { send_backs: 2, ...(rel === undefined ? {} : { ceiling_releases: rel }) })
  const cards = composeCards({ tasks: [], dispatches: [r], issues: [{ number: 5, title: 't', owned_paths: [], criteria: [], criteria_source: 'none', missing: [] }], now })
  const card = (cards as Array<{ number: number; column: string }>).find((c) => c.number === 5)
  boardCases['ceiling|' + (rel === undefined ? 'column-not-selected' : 'releases-1')] = { column: card ? card.column : null, state: dispatchRowState(r, now) }
}
out.board = boardCases

// 3. the round gate: requests that arrive while a round runs
{
  let rounds = 0
  let release: () => void = () => {}
  const gate = createRoundGate(async () => { rounds++; if (rounds === 1) await new Promise<void>((r) => { release = r }) })
  gate.request('first')
  await new Promise((r) => setTimeout(r, 20))
  for (let i = 0; i < 5; i++) gate.request('during ' + i)
  release()
  await new Promise((r) => setTimeout(r, 200))
  out.gate = { requests_during_round: 5, rounds_run: rounds }
}

// 4. queend, asked what the tick asks
const Q = input.queend
function ask(q: unknown, env: Record<string, string> = {}) {
  const r = spawnSync(Q, [], { input: JSON.stringify(q), env: { PATH: process.env.PATH ?? '', ...env } })
  try { return JSON.parse(r.stdout.toString()) } catch { return { error: 'unparsed', status: r.status, out: r.stdout.toString().slice(0, 200) } }
}
const spec = (paths: string[]) => '## Boundary\n' + paths.map((p) => '- `' + p + '`').join('\n') +
  '\n\n## User Scenarios & Testing\nGiven a, When b, Then c.\n\n## Requirements\n- It MUST hold.\n\n## Success Criteria\n- `tests/a.test.ts` passes\n'
const at = new Date(Date.now() - 60e3).toISOString()
const uuid = (n: number) => '00000000-0000-0000-0000-' + String(n).padStart(12, '0')
const tasks = [
  boardTask('o', 'r', { conversationId: uuid(1), issue: 103, ownedPaths: ['src/claimed.ts'], branch: 'queen-103', at, title: 't', state: 'running' }),
  boardTask('o', 'r', { conversationId: uuid(2), issue: 900, ownedPaths: ['src/shared'], branch: 'queen-900', at, title: 't', state: 'running' }),
]
const bodies: Record<string, string> = {
  '101': spec(['src/a.ts']), '102': spec(['src/b.ts']), '103': spec(['src/claimed.ts']),
  '104': spec(['src/shared/x.ts']), '105': 'a body with no sections',
}
const choose: unknown[] = []
for (const perm of input.permutations) {
  const a = ask({ kind: 'choose', candidates: perm, tasks, candidateBodies: bodies })
  choose.push([perm, a.chosen ?? null, a.allowed ?? null, a.skipped ?? null])
}
out.choose = choose
const capacity: unknown[] = []
for (const limit of [null, 2]) {
  for (let k = 0; k <= 5; k++) {
    const running = Array.from({ length: k }, (_, i) => boardTask('o', 'r', { conversationId: uuid(100 + i), issue: 500 + i, ownedPaths: ['src/r' + i + '.ts'], branch: 'b', at, title: 't', state: 'running' }))
    const a = ask({ kind: 'choose', candidates: [101], tasks: running, candidateBodies: bodies }, limit === null ? {} : { TRIOS_QUEEN_MAX_WORKERS: String(limit) })
    capacity.push([limit, k, a.allowed ?? null, a.chosen ?? null, a.refusal ?? null])
  }
}
out.capacity = capacity
const review: unknown[] = []
for (const total of [0, 1, 2]) {
  for (let judged = 0; judged <= total; judged++) {
    for (let met = 0; met <= judged; met++) {
      for (const committed of [null, 0, 1]) {
        for (const prior of [0, 1, 2, 3]) {
          const verdicts = Array.from({ length: judged }, (_, i) => ({ criterion: 'c' + i, met: i < met }))
          const q: Record<string, unknown> = { kind: 'review', verdicts, totalCriteria: total, priorSendBacks: prior }
          if (committed !== null) q.committedFiles = committed
          const a = ask(q)
          review.push([total, judged, met, committed, prior, a.verdict ?? a.error ?? null])
        }
      }
    }
  }
}
out.review = review
const retry: unknown[] = []
for (const seq of input.retrySequences) {
  const a = ask({ kind: 'retry', priorAttempts: seq })
  retry.push([seq, a.allowed ?? null])
}
out.retry = retry
console.log(JSON.stringify(out))
"""

TS_PG = r"""
import pg from 'pg'
import { runPgMigrations } from '@SERVER@/src/lib/db/pg-migrate.ts'
import { acquireQueenLease, releaseQueenLease } from '@SERVER@/src/api/services/queen-lease.ts'
import { recordDispatch, finishDispatch, reapStalledDispatches, reapDispatchesFromPreviousBoot } from '@SERVER@/src/api/services/queen-dispatch.ts'
import { stateOfDispatch, dispatchRowState, releaseStaleContracts } from '@SERVER@/src/api/services/queen-tick.ts'
import { takeBackRefusedAcceptances } from '@SERVER@/src/api/services/queen-ci-verdict.ts'
import { readFileSync } from 'node:fs'

const input = JSON.parse(readFileSync(process.argv[2], 'utf8'))
const out: Record<string, unknown> = {}
const url = process.env.DATABASE_URL as string
const opts = { connectionString: url, options: '-c search_path=trios' }
await runPgMigrations()
const admin = new pg.Pool({ ...opts, max: 4 })
await admin.query(input.ensureQueenColumns)
out.server_version = (await admin.query('SHOW server_version')).rows[0].server_version
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms))

// 1. the singleton lease under contention: 32 processes' worth of pools, one statement each
{
  const N = 32, ROUNDS = 20
  const pools = Array.from({ length: N }, () => new pg.Pool({ ...opts, max: 1 }))
  await Promise.all(pools.map((p) => p.query('SELECT 1')))
  const rounds: unknown[] = []
  let lastFence = 0
  let fencesIncrease = true
  for (let r = 0; r < ROUNDS; r++) {
    const grants = await Promise.all(pools.map((p, i) => acquireQueenLease(p, 'contended', 'c' + i, 60)))
    const winners = grants.filter((g) => g.acquired)
    const fence = winners.length ? winners[0].fence : -1
    if (!(fence > lastFence)) fencesIncrease = false
    lastFence = fence
    const sameAnswer = grants.every((g) => g.holder === (winners[0]?.holder ?? g.holder))
    rounds.push({ winners: winners.length, fence, losers_name_the_winner: sameAnswer })
    if (winners.length) await releaseQueenLease(admin, 'contended', winners[0].holder)
  }
  out.contention = { contenders: N, rounds: ROUNDS, per_round: rounds, max_winners: Math.max(...rounds.map((x: any) => x.winners)),
                     min_winners: Math.min(...rounds.map((x: any) => x.winners)), fences_strictly_increase: fencesIncrease }
  await Promise.all(pools.map((p) => p.end()))
}

// 2. expiry, renewal and fencing between two holders
{
  const a1 = await acquireQueenLease(admin, 'expiry', 'A', 1)
  const b1 = await acquireQueenLease(admin, 'expiry', 'B', 1)
  const a2 = await acquireQueenLease(admin, 'expiry', 'A', 1)
  await sleep(1300)
  const b2 = await acquireQueenLease(admin, 'expiry', 'B', 60)
  const a3 = await acquireQueenLease(admin, 'expiry', 'A', 60)
  const relA = await releaseQueenLease(admin, 'expiry', 'A')
  const relB = await releaseQueenLease(admin, 'expiry', 'B')
  const c1 = await acquireQueenLease(admin, 'expiry', 'C', 60)
  out.expiry = {
    a_takes: a1.acquired, a_fence: a1.fence, b_refused_while_a_holds: !b1.acquired, b_told_holder: b1.holder,
    a_renews_own: a2.acquired, a_renew_fence: a2.fence, b_takes_after_expiry: b2.acquired, b_fence: b2.fence,
    a_refused_after_takeover: !a3.acquired, non_holder_release: relA, holder_release: relB,
    c_after_release: c1.acquired, c_fence: c1.fence,
  }
}

// 3. the dispatch writers on real rows
{
  await admin.query('DELETE FROM queen_dispatch; DELETE FROM queen_dispatch_history')
  const conv1 = '11111111-1111-1111-1111-111111111111', conv2 = '22222222-2222-2222-2222-222222222222'
  const read = async (issue: number) => (await admin.query('SELECT * FROM queen_dispatch WHERE issue = $1', [issue])).rows
  const state = (r: any) => (r ? dispatchRowState(r) : null)
  const t: Record<string, unknown> = {}
  await recordDispatch(admin, 41, 'queen-41', true, 'started', ['src/a.ts'], conv1, 0)
  let rows = await read(41)
  t.started = { rows: rows.length, state: state(rows[0]) }
  t.first_end = await finishDispatch(admin, 41, 'finished', undefined, conv1)
  t.second_end = await finishDispatch(admin, 41, 'finished', undefined, conv1)
  rows = await read(41)
  t.after_end = { state: state(rows[0]), outcome: rows[0].outcome }
  await recordDispatch(admin, 41, 'queen-41', true, 'started again', ['src/a.ts'], conv2, 1)
  rows = await read(41)
  const hist = (await admin.query('SELECT count(*)::int AS n FROM queen_dispatch_history WHERE issue = 41')).rows[0].n
  t.redispatch = { rows: rows.length, state: state(rows[0]), conversation: rows[0].conversation_id, archived: hist }
  t.late_end_of_old_turn = await finishDispatch(admin, 41, 'finished', undefined, conv1)
  rows = await read(41)
  t.after_late_end = { state: state(rows[0]) }
  await recordDispatch(admin, 42, 'queen-42', false, 'every key is busy', ['src/b.ts'])
  rows = await read(42)
  t.refused = { started: rows[0].started, outcome: rows[0].outcome, finished: rows[0].finished_at !== null, state: state(rows[0]) }
  // the stall reaper: only age since dispatch counts
  await recordDispatch(admin, 43, 'queen-43', true, 'started', ['src/c.ts'], conv1, 2)
  await recordDispatch(admin, 44, 'queen-44', true, 'started', ['src/d.ts'], conv2, 3)
  await admin.query("UPDATE queen_dispatch SET dispatched_at = now() - interval '121 minutes' WHERE issue = 43")
  await admin.query("UPDATE queen_dispatch SET dispatched_at = now() - interval '119 minutes' WHERE issue = 44")
  const noSalvage = { salvage: async () => ({ committed: false }) } as any
  t.stall_reaped = (await reapStalledDispatches(admin, 120, noSalvage)).sort()
  const r43 = (await read(43))[0], r44 = (await read(44))[0]
  t.stall_states = { '43': { state: state(r43), outcome: r43.outcome }, '44': { state: state(r44) } }
  t.boot_reaped = (await reapDispatchesFromPreviousBoot(admin, noSalvage)).sort()
  out.writers = t
}

// 4. the ceiling, through the very query each round runs
{
  await admin.query('DELETE FROM queen_dispatch')
  await admin.query(`INSERT INTO queen_dispatch (issue, branch, started, detail, owned_paths, dispatched_at, finished_at, outcome, review_state, send_backs, ceiling_releases)
                     VALUES (77, 'queen-77', true, 'x', '[]', now() - interval '3 hours', now() - interval '2 hours', 'finished', 'sendBack', 2, 1)`)
  const roundRows = (await admin.query(input.roundQuery)).rows
  const r = roundRows.find((x: any) => x.issue === 77)
  const roundState = stateOfDispatch(r.finished_at != null, r.review_state, {
    idleMs: Date.now() - Date.parse(String(r.finished_at)), sendBacks: Number(r.send_backs ?? 0), releases: Number(r.ceiling_releases ?? 0) })
  const full = (await admin.query('SELECT * FROM queen_dispatch WHERE issue = 77')).rows[0]
  const boardRows = (await admin.query(input.boardQuery)).rows
  const b = boardRows.find((x: any) => x.issue === 77)
  out.ceiling = { stored_releases: full.ceiling_releases, round_selects_releases: Object.prototype.hasOwnProperty.call(r, 'ceiling_releases'),
                  board_selects_releases: Object.prototype.hasOwnProperty.call(b, 'ceiling_releases'),
                  round_state: roundState, full_row_state: dispatchRowState(full), board_state: dispatchRowState(b) }
}
// 5. which finished rows a criteria rewrite may release
{
  await admin.query('DELETE FROM queen_dispatch; DELETE FROM queen_issues')
  const mk = (issue: number, finished: boolean, review: string | null, outcome: string | null) => admin.query(
    `INSERT INTO queen_dispatch (issue, branch, started, detail, owned_paths, dispatched_at, finished_at, outcome, review_state, criteria)
     VALUES ($1::int, 'queen-' || $1::text, true, 'x', '[]', now() - interval '3 hours', CASE WHEN $2::boolean THEN now() - interval '2 hours' ELSE NULL END, $3::text, $4::text, '["old"]')`,
    [issue, finished, outcome, review])
  await mk(51, true, 'escalate', 'finished'); await mk(52, true, 'accept', 'finished'); await mk(53, false, null, null)
  await mk(54, true, null, 'reaped: no completion within 120 minutes'); await mk(55, true, 'wait', 'finished'); await mk(56, true, 'sendBack', 'finished')
  for (const n of [51, 52, 53, 54, 55, 56])
    await admin.query(`INSERT INTO queen_issues (number, title, state, criteria) VALUES ($1, 't', 'open', '["new"]')`, [n])
  const released = (await releaseStaleContracts(admin)).sort()
  const again = await releaseStaleContracts(admin)
  out.stale = { released, second_pass: again }
}

// 6. a required check refusing an accepted row
{
  await admin.query('DELETE FROM queen_dispatch; DELETE FROM queen_issues')
  for (const [issue, review, sb] of [[61, 'accept', 0], [62, 'accept', 1], [63, 'escalate', 0]] as Array<[number, string, number]>) {
    await admin.query(`INSERT INTO queen_dispatch (issue, branch, started, detail, owned_paths, dispatched_at, finished_at, outcome, review_state, send_backs)
                       VALUES ($1::int, 'queen-' || $1::text, true, 'x', '[]', now() - interval '3 hours', now() - interval '2 hours', 'finished', $2::text, $3::int)`, [issue, review, sb])
    await admin.query(`INSERT INTO queen_issues (number, title, state) VALUES ($1, 't', 'open')`, [issue])
  }
  const deps = { requiredChecks: async () => ['ci'], pullsForBranch: async () => [{ number: 9, state: 'open', merged: false, headSha: 'abc' }],
                 checkRuns: async () => [{ id: 1, name: 'ci', status: 'completed', conclusion: 'failure', url: null }], jobLog: async () => '' }
  const first = (await takeBackRefusedAcceptances(admin, deps as any)).map((t) => [t.issue, t.state])
  const second = (await takeBackRefusedAcceptances(admin, deps as any)).map((t) => [t.issue, t.state])
  const rows = (await admin.query('SELECT issue, review_state, send_backs FROM queen_dispatch ORDER BY issue')).rows
  const noPr = { ...deps, pullsForBranch: async () => [] }
  await admin.query("UPDATE queen_dispatch SET review_state = 'accept' WHERE issue = 61")
  const withoutPr = (await takeBackRefusedAcceptances(admin, noPr as any)).length
  out.ci = { first, second, rows, accepts_taken_back_without_a_pull_request: withoutPr }
}
await admin.end()
console.log(JSON.stringify(out))
"""


def write_fixture(browseros: pathlib.Path, name: str, text: str) -> pathlib.Path:
    d = browseros / SERVER / ".s09-fixtures"
    d.mkdir(exist_ok=True)
    p = d / name
    p.write_text(text.replace("@SERVER@", str(browseros / SERVER)), encoding="utf-8")
    return p


def run_bun(bun, script: pathlib.Path, payload: dict, env=None, timeout=1800) -> dict:
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as fh:
        json.dump(payload, fh)
        arg = fh.name
    e = dict(os.environ)
    e.update(env or {})
    p = subprocess.run([bun, "run", str(script), arg], cwd=script.parent, capture_output=True, text=True, timeout=timeout, env=e)
    os.unlink(arg)
    last = [l for l in p.stdout.splitlines() if l.startswith("{")]
    if p.returncode != 0 or not last:
        raise RuntimeError(f"bun {script.name} failed rc={p.returncode}: {(p.stderr or p.stdout)[-1200:]}")
    return json.loads(last[-1])


def checkout_facts(browseros: pathlib.Path) -> dict:
    head = subprocess.run(["git", "-C", str(browseros), "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
    srv = browseros / SERVER
    files = {}
    for rel in ("src/api/services/queen-tick.ts", "src/api/services/queen-dispatch.ts", "src/api/services/queen-lease.ts",
                "src/api/routes/queen-kanban.ts", "src/lib/db/pg-migrate.ts"):
        files[rel] = base.sha256((srv / rel).read_bytes())
    for rel in ("queen-core/Sources/queend/main.swift", "queen-core/Sources/QueenCore/QueenReviewDecision.swift",
                "queen-core/Sources/QueenCore/QueenRetryPolicy.swift", "queen-core/Sources/QueenPolicy/QueenDelegation.swift"):
        files[rel] = base.sha256((browseros / "trios/agent-server" / rel).read_bytes())
    pgpkg = None
    for cand in (srv / "node_modules/pg/package.json", browseros / "trios/agent-server/node_modules/pg/package.json"):
        if cand.exists():
            pgpkg = json.loads(cand.read_text()).get("version")
            break
    return {"head": head, "files": files, "pg_driver": pgpkg}


def build_ts_record(browseros, bun, queend):
    facts = checkout_facts(browseros)
    if facts["head"] != BROWSEROS_PIN:
        raise RuntimeError(f"the BrowserOS checkout is at {facts['head']}, not the pin {BROWSEROS_PIN}")
    script = write_fixture(browseros, "fixture.ts", TS_FIXTURE)
    payload = {"verdicts": VERDICTS, "idles": IDLES, "sendBacks": SEND_BACKS, "releases": RELEASES, "queend": str(queend),
               "permutations": [list(p) for p in itertools.permutations([101, 102, 103, 104, 105])],
               "retrySequences": [list(s) for n in range(0, 4) for s in itertools.product(["interrupted", "producedNothing", "workedButFailed", "unmeasured"], repeat=n)]}
    out = run_bun(bun, script, payload)
    qv = subprocess.run(["swift", "--version"], capture_output=True, text=True).stdout.strip().splitlines()[0] if shutil.which("swift") else None
    bv = subprocess.run([bun, "--version"], capture_output=True, text=True).stdout.strip()
    common = {"browseros": {"repo": BROWSEROS_REPO, "branch": BROWSEROS_BRANCH, **facts}, "host": base.host(), "at": base.now()}
    ts = {**common, "bun": bv, "result": {k: out[k] for k in ("constants", "claims", "board", "gate")},
          "description": "The supervisor's own TypeScript called under bun 1.3.6: stateOfDispatch over 2112 rows (finished, eleven verdicts, eight idle times, four send-back counts, three release counts), composeCards for finished rows on open and closed issues, and createRoundGate under five requests during a round."}
    qd = {**common, "swift": qv, "queend_sha256": base.sha256(pathlib.Path(queend).read_bytes()),
          "result": {k: out[k] for k in ("choose", "capacity", "review", "retry")},
          "description": "The supervisor's Swift policy binary queend, built from queen-core at the pin with swift build -c release and asked over stdin what the tick asks: choose for all 120 orders of five candidates (two eligible, one claimed, one whose boundary is held, one without sections; tasks built with the supervisor's own boardTask), choose at every running count from 0 to 5 with the default and a lowered worker limit, review over every combination of 0..2 criteria, judged and met counts, three commit counts and four prior send-backs, and retry over every sequence of up to three failure kinds."}
    return ts, qd


def extract_sql(browseros: pathlib.Path) -> dict:
    tick = (browseros / SERVER / "src/api/services/queen-tick.ts").read_text()
    kanban = (browseros / SERVER / "src/api/routes/queen-kanban.ts").read_text()
    ensure = re.search(r"async function ensureQueenColumns\(pool: Pool\): Promise<void> \{\n  await pool\.query\(`(.*?)`\)", tick, re.S).group(1)
    round_q = re.search(r"const inFlight = await pool\.query\(\n(?:\s*//[^\n]*\n)*\s*`(SELECT issue, branch, owned_paths.*?)`,?\n", tick, re.S).group(1)
    board_q = re.search(r"`(SELECT issue, branch, started, detail, finished_at, outcome,.*?)`", kanban, re.S).group(1)
    strip = lambda q: "\n".join(l for l in q.splitlines() if not l.strip().startswith("--"))
    return {"ensureQueenColumns": ensure, "roundQuery": strip(round_q), "boardQuery": strip(board_q)}


def build_pg_record(browseros, bun, database_url) -> dict:
    facts = checkout_facts(browseros)
    if facts["head"] != BROWSEROS_PIN:
        raise RuntimeError(f"the BrowserOS checkout is at {facts['head']}, not the pin {BROWSEROS_PIN}")
    script = write_fixture(browseros, "pg.ts", TS_PG)
    sql = extract_sql(browseros)
    out = run_bun(bun, script, sql, env={"DATABASE_URL": database_url, "QUEEN_DB_SCHEMA": "trios", "LOG_LEVEL": "silent"})
    return {"browseros": {"repo": BROWSEROS_REPO, "branch": BROWSEROS_BRANCH, **facts}, "host": base.host(), "at": base.now(),
            "queries": {k: base.sha256(v.encode()) for k, v in sql.items()}, "result": out,
            "description": "A throwaway PostgreSQL migrated by the supervisor's own runPgMigrations and ensureQueenColumns (the SQL is read out of the pinned queen-tick.ts). Then: the singleton lease taken by 32 pools at once in 20 rounds; expiry, renewal, release and fencing between holders; recordDispatch, finishDispatch and both reapers on real rows; and the round's and the board's own SELECT text (read out of the pinned source) run against a row that has spent its retry ceiling."}


# ===========================================================================================
# LOTUS: the local Zig cycle at the trinity pin
# ===========================================================================================
def build_lotus_record(trinity, zig, sysroot) -> dict:
    work = pathlib.Path(tempfile.mkdtemp(prefix="s09-lotus-"))
    names = subprocess.run(["git", "-C", str(trinity), "ls-tree", "--name-only", TRINITY_PIN, "src/tri/queen/"],
                           capture_output=True, text=True).stdout.split()
    for n in names:
        (work / pathlib.Path(n).name).write_bytes(base.git_show(trinity, TRINITY_PIN, n))
    src = (work / "lotus_cycle.zig").read_text()
    full = re.search(r"pub fn runFullCycle.*?\n\}", src, re.S).group(0)
    aware = re.search(r"pub fn runEpisodeAwareCycle.*?\n\}", src, re.S).group(0)
    stages = lambda body: [m.strip() for m in re.findall(r"// Stage \d+: ([^\n]+)", body)]
    modes = {}
    for mode in ("Debug", "ReleaseFast"):
        home = work / f"home-{mode}"
        home.mkdir()
        env = dict(os.environ, HOME=str(home))
        cmd = [zig, "test"] + (["--sysroot", sysroot] if sysroot else []) + ["-O", mode, "lotus_cycle.zig"]
        p = subprocess.run(cmd, cwd=work, capture_output=True, text=True, timeout=1800, env=env)
        text = p.stdout + p.stderr
        m = re.search(r"All (\d+) tests passed", text)
        modes[mode] = {"rc": p.returncode, "passed": int(m.group(1)) if m else None, "tail": "" if p.returncode == 0 else text[-400:]}
    cli = base.git_show(trinity, TRINITY_PIN, "src/tri/queen/lotus_cli.zig").decode()
    agents = base.git_show(trinity, TRINITY_PIN, "AGENTS.md").decode()
    build = base.git_show(trinity, TRINITY_PIN, "build.zig").decode()
    cmds = re.findall(r'std\.mem\.eql\(u8, command, "([a-z-]+)"\)', cli)
    return {"pin": TRINITY_PIN, "zig": base.zig_version(zig), "host": base.host(), "at": base.now(), "modes": modes,
            "stages_full_cycle": stages(full), "stages_episode_aware": stages(aware),
            "record_stage_is_a_comment": "// Stage 2: Record Episode" in full and "recordEpisode(" not in full.split("// Stage 3")[0],
            "lotus_cli_commands": sorted(set(cmds)),
            "lotus_cli_has_phase_option": "--phase" in cli,
            "documented_phase_flags": sorted(set(re.findall(r"tri queen lotus --phase ([a-z]+)", agents))),
            "build_has_lotus_step": "lotus" in build,
            "files": {n: base.sha256(base.git_show(trinity, TRINITY_PIN, n)) for n in names},
            "description": "gHashTag/trinity src/tri/queen/*.zig at the pin, compiled with Zig 0.15.2 and lotus_cycle.zig's tests run (they import every stage); the stage lists are read from the // Stage comments of the two cycle functions; the documented `tri queen lotus --phase` flags are read from AGENTS.md and the commands lotus_cli.zig actually accepts from its source."}


# ===========================================================================================
# LIVE: one public snapshot
# ===========================================================================================
def fetch_public(path: str) -> dict:
    """GET a public endpoint with curl. The local resolver of the measuring host could not resolve
    *.railway.app on 2026-10-01, so when plain DNS fails the address is taken from 1.1.1.1 and passed
    with --resolve; nothing else about the request changes."""
    host = LIVE_BASE.split("//", 1)[1]
    cmd = ["curl", "-s", "-m", "60", "-H", "User-Agent: t27-s09-snapshot", LIVE_BASE + path]
    p = subprocess.run(cmd, capture_output=True, text=True)
    via = "dns"
    if p.returncode != 0 or not p.stdout.strip():
        ip = subprocess.run(["dig", "+short", "@1.1.1.1", host, "A"], capture_output=True, text=True).stdout.split()
        ip = [x for x in ip if re.fullmatch(r"[0-9.]+", x)]
        if not ip:
            raise RuntimeError(f"{host} does not resolve, locally or through 1.1.1.1")
        p = subprocess.run(cmd[:1] + ["--resolve", f"{host}:443:{ip[0]}"] + cmd[1:], capture_output=True, text=True)
        via = "1.1.1.1"
    if p.returncode != 0:
        raise RuntimeError(f"GET {path} failed: curl exit {p.returncode}")
    return {"body": json.loads(p.stdout), "resolved_via": via}


def build_live_record() -> dict:
    st = fetch_public("/queen/status")
    bd = fetch_public("/queen/public-board")
    s, b = st["body"], bd["body"]
    counts = {}
    for c in b.get("cards", []):
        counts[c.get("column")] = counts.get(c.get("column"), 0) + 1
    w = s.get("workers") or {}
    return {"at": base.now(), "base": LIVE_BASE, "snapshot": True, "resolved_via": st["resolved_via"],
            "status": {"swarmState": s.get("swarmState"), "queue_state": (s.get("queue") or {}).get("state"),
                       "workers": {k: w.get(k) for k in ("capacity", "active", "idle")}, "keys": sorted(s)},
            "board": {"repo": b.get("repo"), "columns": [c.get("key") for c in b.get("columns", [])], "cards": len(b.get("cards", [])),
                      "per_column": counts, "pulse_keys": sorted((b.get("pulse") or {}).keys())},
            "description": "One read of two public, unauthenticated endpoints of the deployed supervisor. Vocabulary and counts only. A snapshot: it says what was served at that minute, not what is served now."}


# ===========================================================================================
# REPLAY: the decisions of specs/queen/dispatch.t27 through the C backend, against the records
# ===========================================================================================
V_CODE = {None: 0, "": 1, "wait": 2, "empty": 3, "sendBack": 4, "escalate": 5, "accept": 6, "failed": 7, "cancelled": 8,
          "stale-contract": 9, "unrecognised": 10}
C_CODE = {"running": 0, "accepted": 1, "rejected": 2, "awaitingReview": 3, "failed": 4}
COL_CODE = {"backlog": 0, "blocked": 1, "running": 2, "review": 3, "done": 4, "dropped": 5}
R_CODE = {"accept": 0, "sendBack": 1, "escalate": 2, "wait": 3}
ELIGIBLE = {101: True, 102: True, 103: False, 104: False, 105: False}
S = {"NONE": 0, "RUNNING": 1, "REFUSED": 2, "ENDED": 3, "WAIT": 4, "ACCEPT": 5, "SENDBACK": 6, "ESCALATE": 7, "EMPTY": 8,
     "STALE": 9, "REAPED": 10, "RELEASED": 11, "ILLEGAL": 99}
E = {"START": 0, "REFUSE": 1, "END": 2, "END_OTHER_TURN": 3, "REAP_STALL": 4, "REAP_BOOT": 5, "CRITERIA_CHANGED": 11,
     "CI_RED_UNDER_CEILING": 12, "CI_RED_AT_CEILING": 13, "REDISPATCH": 15}
TWO_HOURS = 2 * 3600 * 1000


def cb(v):
    return "true" if v else "false"


def spec_cases(T, Q, P) -> list:
    """Every case is (id, [(C expression, label)]); expected values come from the records."""
    t, q, p = T["result"], Q["result"], P["result"]
    cases = []
    for f, v, idle, sb, rel, st in t["claims"]:
        cases.append((f"claim-{int(f)}-{v}-{idle}-{sb}-{rel}", [(f"claim_of({cb(f)}, {V_CODE[v]}u, {idle}u, {sb}u, {rel}u) == {C_CODE[st]}u", "claim")]))
    for key, col in t["board"].items():
        if key.startswith("ceiling|"):
            rel = "releases_read_by_round(1u)" if key.endswith("column-not-selected") else "1u"
            cases.append((f"board-{key}", [(f"column_of(claim_of(true, 4u, {TWO_HOURS}u, 2u, {rel}), true, true) == {COL_CODE[col['column']]}u", "column"),
                                           (f"claim_of(true, 4u, {TWO_HOURS}u, 2u, {rel}) == {C_CODE[col['state']]}u", "state")]))
            continue
        verdict, openness = key.split("|")
        if verdict == "running":
            cases.append((f"board-{key}", [(f"column_of(0u, false, true) == {COL_CODE[col]}u", "column")]))
            continue
        v = None if verdict == "null" else verdict
        cases.append((f"board-{key}", [(f"column_of(claim_of(true, {V_CODE[v]}u, {TWO_HOURS}u, 0u, 0u), {cb(openness == 'open')}, true) == {COL_CODE[col]}u", "column")]))
    g = t["gate"]
    cases.append(("gate", [(f"1u + followup_rounds({g['requests_during_round']}u) == {g['rounds_run']}u", "rounds")]))
    for perm, chosen, allowed, _ in q["choose"]:
        flags = ", ".join(cb(ELIGIBLE[n]) for n in perm)
        want = perm.index(chosen) if chosen is not None else 5
        cases.append((f"choose-{'-'.join(map(str, perm))}", [(f"chosen_of({flags}) == {want}u", "first eligible")]))
    for limit, k, allowed, chosen, _ in q["capacity"]:
        cases.append((f"capacity-{limit}-{k}", [(f"can_start({k}u, {limit if limit is not None else 4}u) == {cb(allowed)}", "capacity")]))
    for total, judged, met, committed, prior, verdict in q["review"]:
        known = committed is not None
        cases.append((f"review-{total}-{judged}-{met}-{committed}-{prior}",
                      [(f"review_of({total}u, {judged}u, {met}u, {cb(known)}, {committed or 0}u, {prior}u) == {R_CODE[verdict]}u", "verdict")]))
    for seq, allowed in q["retry"]:
        real = sum(1 for k in seq if k != "interrupted")
        cases.append((f"retry-{'-'.join(seq) or 'none'}", [(f"retry_allowed({real}u) == {cb(allowed)}", "retry")]))
    c = p["contention"]
    conds = [(f"owners_per_term() == {c['max_winners']}u", "max owners"), (f"owners_per_term() == {c['min_winners']}u", "min owners")]
    fences = [r["fence"] for r in c["per_round"]]
    conds += [(f"fence_after({a}u) <= {b}u", f"fence {a}->{b}") for a, b in zip(fences, fences[1:])]
    cases.append(("lease-contention", conds))
    x = p["expiry"]
    cases.append(("lease-expiry", [
        (f"lease_taken(false, false) == {cb(not x['b_refused_while_a_holds'])}", "held"),
        (f"lease_taken(false, true) == {cb(x['a_renews_own'])}", "renew"),
        (f"lease_taken(true, false) == {cb(x['b_takes_after_expiry'])}", "takeover"),
        (f"lease_taken(false, false) == {cb(not x['a_refused_after_takeover'])}", "stale holder"),
        (f"fence_after({x['a_fence']}u) == {x['a_renew_fence']}u", "renew fence"),
        (f"fence_after({x['a_renew_fence']}u) == {x['b_fence']}u", "takeover fence"),
        (f"fence_after({x['b_fence']}u) == {x['c_fence']}u", "fence survives release")]))
    w = p["writers"]
    ns = lambda a, e: f"next_state({S[a]}u, {E[e]}u)"
    cases.append(("writer-start", [(f"{ns('NONE', 'START')} == {S['RUNNING'] if w['started']['state'] == 'running' else 98}u", "start")]))
    cases.append(("writer-end", [(f"{ns('RUNNING', 'END')} == {S['ENDED'] if w['first_end'] == 1 and w['after_end']['outcome'] == 'finished' else 98}u", "end")]))
    cases.append(("writer-second-end", [(f"{ns('ENDED', 'END')} == {S['ILLEGAL'] if w['second_end'] == 0 else 98}u", "second end")]))
    cases.append(("writer-late-end", [(f"{ns('RUNNING', 'END_OTHER_TURN')} == {S['ILLEGAL'] if w['late_end_of_old_turn'] == 0 and w['after_late_end']['state'] == 'running' else 98}u", "late end")]))
    cases.append(("writer-refused", [(f"{ns('NONE', 'REFUSE')} == {S['REFUSED'] if not w['refused']['started'] and w['refused']['outcome'] == 'refused' else 98}u", "refused")]))
    cases.append(("writer-overwrite", [(f"writer_enforces_table() == {cb(not (w['redispatch']['rows'] == 1 and w['redispatch']['archived'] == 1))}", "overwrite"),
                                       (f"{ns('ENDED', 'REDISPATCH')} == {S['ILLEGAL']}u", "the table forbids what the writer did")]))
    cases.append(("writer-stall", [(f"stall_reapable(121u) == {cb(43 in w['stall_reaped'])}", "121"), (f"stall_reapable(119u) == {cb(44 in w['stall_reaped'])}", "119"),
                                   (f"{ns('RUNNING', 'REAP_STALL')} == {S['REAPED'] if w['stall_states']['43']['outcome'].startswith('reaped') else 98}u", "reaped")]))
    cases.append(("writer-boot", [(f"{ns('RUNNING', 'REAP_BOOT')} == {S['REAPED'] if 44 in w['boot_reaped'] else 98}u", "boot")]))
    st = p["stale"]
    for issue, state in ((51, "ESCALATE"), (52, "ACCEPT"), (53, "RUNNING"), (54, "REAPED"), (55, "WAIT"), (56, "SENDBACK")):
        want = S["STALE"] if issue in st["released"] else S["ILLEGAL"]
        cases.append((f"stale-{issue}", [(f"{ns(state, 'CRITERIA_CHANGED')} == {want}u", "criteria rewrite")]))
    cases.append(("stale-second-pass", [(f"{ns('STALE', 'CRITERIA_CHANGED')} == {S['ILLEGAL'] if st['second_pass'] == [] else S['STALE']}u", "second pass")]))
    ci = p["ci"]
    first = {i: s_ for i, s_ in ci["first"]}
    cases.append(("ci-61", [(f"{ns('ACCEPT', 'CI_RED_UNDER_CEILING')} == {S['SENDBACK'] if first.get(61) == 'sendBack' else 98}u", "under ceiling")]))
    cases.append(("ci-62", [(f"{ns('ACCEPT', 'CI_RED_AT_CEILING')} == {S['ESCALATE'] if first.get(62) == 'escalate' else 98}u", "at ceiling")]))
    cases.append(("ci-63", [(f"{ns('ESCALATE', 'CI_RED_UNDER_CEILING')} == {S['ILLEGAL'] if 63 not in first else 98}u", "not an accept")]))
    cases.append(("ci-second-pass", [(f"{ns('SENDBACK', 'CI_RED_UNDER_CEILING')} == {S['ILLEGAL'] if ci['second'] == [] else 98}u", "second pass")]))
    cases.append(("ci-no-pull-request", [(f"accept_requires_pull_request() == {cb(ci['accepts_taken_back_without_a_pull_request'] > 0)}", "no pull request")]))
    ce = p["ceiling"]
    cases.append(("ceiling-round", [(f"claim_of(true, 4u, {TWO_HOURS}u, 2u, releases_read_by_round({ce['stored_releases']}u)) == {C_CODE[ce['round_state']]}u", "round"),
                                    (f"claim_of(true, 4u, {TWO_HOURS}u, 2u, {ce['stored_releases']}u) == {C_CODE[ce['full_row_state']]}u", "full row"),
                                    (f"claim_of(true, 4u, {TWO_HOURS}u, 2u, releases_read_by_round({ce['stored_releases']}u)) == {C_CODE[ce['board_state']]}u", "board")]))
    return cases


def replay(T, Q, P, t27c, spec=None) -> dict:
    with tempfile.TemporaryDirectory() as tmp:
        r = base.replay_spec(pathlib.Path(spec or SPEC), spec_cases(T, Q, P), t27c, pathlib.Path(tmp) / "dispatch")
    r["spec_sha256"] = base.sha256(pathlib.Path(spec or SPEC).read_bytes())
    return {"at": base.now(), "host": base.host(), "result": r}


def run_spec_tests(t27c, path: pathlib.Path) -> dict:
    """gen-c, cc and the spec's own test main: does the file compile, and do its tests pass."""
    with tempfile.TemporaryDirectory() as tmp:
        c = pathlib.Path(tmp) / "s.c"
        rc, out, err = base.run([str(t27c), "gen-c", str(path)])
        if rc != 0 or not out.strip():
            return {"generates": False, "compiles": False, "tests": None}
        c.write_text(out)
        rc, o2, e2 = base.run(["cc", "-std=c11", "-w", "-DT27_TEST_MAIN", str(c), "-o", str(pathlib.Path(tmp) / "s")])
        if rc != 0:
            return {"generates": True, "compiles": False, "tests": None}
        rc, o3, e3 = base.run([str(pathlib.Path(tmp) / "s")], timeout=60)
        m = re.search(r"All (\d+) tests passed", o3)
        return {"generates": True, "compiles": True, "tests": int(m.group(1)) if m and rc == 0 else f"failed rc={rc}"}


NEIGHBOUR_PATHS = ["specs/queen/lotus.t27", "specs/queen/task_analysis.t27", "specs/queen/brain_summaries.t27",
                   "specs/brain/brain.t27", "specs/brain/bus.t27", "specs/brain/cognitive_loop.t27"]
EXPECT_COMPILES_NOW = {"specs/queen/lotus.t27": False, "specs/queen/task_analysis.t27": True, "specs/queen/brain_summaries.t27": False,
                       "specs/brain/brain.t27": False, "specs/brain/bus.t27": True, "specs/brain/cognitive_loop.t27": True}
EXPECT_COMPILES_AT_MASTER = {**EXPECT_COMPILES_NOW, "specs/queen/task_analysis.t27": False}
MASTER_REF = "c8afd3ab8ee4070eb0de0a2207063e20f5dacd59"   # t27 master when S09 started (S08 merged)


def compile_status(t27c, ref=None) -> dict:
    out = {}
    for rel in NEIGHBOUR_PATHS:
        if ref is None:
            out[rel] = run_spec_tests(t27c, ROOT / rel)
            continue
        blob = subprocess.run(["git", "-C", str(ROOT), "show", f"{ref}:{rel}"], capture_output=True)
        if blob.returncode != 0:
            out[rel] = {"generates": None, "compiles": None, "tests": None}
            continue
        with tempfile.NamedTemporaryFile("wb", suffix=".t27", delete=False) as fh:
            fh.write(blob.stdout)
            tmp = fh.name
        out[rel] = run_spec_tests(t27c, pathlib.Path(tmp))
        os.unlink(tmp)
    return out


# ===========================================================================================
# CHECK
# ===========================================================================================
def load(key):
    return base.load_json(REC[key])


def record_claims(T, Q, P, L, V, sp) -> list:
    """The facts the spec's words rest on, written out by hand. (claim id, finding, text, lambda)."""
    t, q, p = T["result"], Q["result"], P["result"]
    C = []

    def add(cid, fnd, text, fn):
        C.append((cid, fnd, text, fn))
    add("q01", "f50", "the Zig cycle passes its 29 tests in Debug and ReleaseFast, its Record Episode stage is a comment, nothing builds it, and the six documented --phase flags meet a CLI with no --phase option",
        lambda: L["modes"]["Debug"]["passed"] == 29 and L["modes"]["ReleaseFast"]["passed"] == 29 and L["record_stage_is_a_comment"] is True
        and L["build_has_lotus_step"] is False and sorted(L["documented_phase_flags"]) == ["assign", "evolve", "plan", "run", "test", "verdict"]
        and L["lotus_cli_has_phase_option"] is False)
    add("q02", "f52", "queend chose the first eligible candidate in every one of the 120 orders",
        lambda: len(q["choose"]) == 120 and all(ch == next(n for n in perm if ELIGIBLE[n]) for perm, ch, _, _ in q["choose"]))
    add("q03", "f53", "a stored ceiling_releases of 1 is not selected by the round or the board, so the row reads failed instead of rejected",
        lambda: p["ceiling"] == {"stored_releases": 1, "round_selects_releases": False, "board_selects_releases": False, "round_state": "failed",
                                 "full_row_state": "rejected", "board_state": "failed"})
    add("q04", "f54", "the board draws escalate, send-back, wait, empty and no verdict as done once the issue is closed",
        lambda: all(t["board"][f"{v}|closed"] == "done" for v in ("escalate", "sendBack", "wait", "empty", "null")) and t["board"]["running|closed"] == "running")
    add("q05", "f55", "without an open pull request a red check takes nothing back; with one, the accepts at 0 and 1 prior send-backs become sendBack and escalate",
        lambda: p["ci"]["accepts_taken_back_without_a_pull_request"] == 0 and p["ci"]["first"] == [[61, "sendBack"], [62, "escalate"]])
    add("q06", "f56", "32 contenders in 20 rounds: one owner every round, fences strictly rising, every loser told the winner's name",
        lambda: p["contention"]["contenders"] == 32 and p["contention"]["rounds"] == 20 and p["contention"]["max_winners"] == 1
        and p["contention"]["min_winners"] == 1 and p["contention"]["fences_strictly_increase"] and all(r["losers_name_the_winner"] for r in p["contention"]["per_round"]))
    add("q07", "f57", "recordDispatch replaced a row awaiting review: one row, one archive, the new conversation running",
        lambda: p["writers"]["redispatch"] == {"rows": 1, "state": "running", "conversation": "22222222-2222-2222-2222-222222222222", "archived": 1})
    add("q08", "f58", "the stall reaper took the row dispatched 121 minutes ago and left the one at 119; boot reaped every unfinished row",
        lambda: p["writers"]["stall_reaped"] == [43] and p["writers"]["stall_states"]["44"]["state"] == "running" and p["writers"]["boot_reaped"] == [41, 44])
    add("q09", "f61", "a duplicate and a late ending matched no row, and the second criteria release and CI take-back found nothing",
        lambda: p["writers"]["second_end"] == 0 and p["writers"]["late_end_of_old_turn"] == 0 and p["stale"]["second_pass"] == [] and p["ci"]["second"] == [])
    add("q10", "f60", "review escalates at two prior send-backs, and retry refuses at two real attempts",
        lambda: all(v == "escalate" for tt, j, m, c, pr, v in q["review"] if tt > 0 and j == tt and m < tt and pr >= 2)
        and all(a is False for seq, a in q["retry"] if sum(1 for k in seq if k != "interrupted") >= 2))
    add("q11", "f64", "the live snapshot shows the six columns of the spec and a swarm state from its vocabulary",
        lambda: V["board"]["columns"] == ["backlog", "blocked", "running", "review", "done", "dropped"]
        and V["status"]["swarmState"] in ("working", "waiting_for_review", "healthy_idle", "unavailable") and V["snapshot"] is True)
    add("q12", "constants", "the floors, ceilings and release bound the supervisor exports are the spec's",
        lambda: t["constants"] == {"SEND_BACK_IDLE_FLOOR_MS": sp["SENDBACK_FLOOR_MS"], "WAIT_FROZEN_FLOOR_MS": sp["WAIT_FLOOR_MS"],
                                   "EMPTY_ATTEMPT_FLOOR_MS": sp["EMPTY_FLOOR_MS"], "CEILING_RELEASE_MS": sp["CEILING_RELEASE_MS"],
                                   "MAX_CEILING_RELEASES": sp["MAX_CEILING_RELEASES"]})
    return C


def check_findings(sp: dict) -> list:
    f = []
    allowed = {"ts", "queend", "pg", "lotus", "live", "source", "compile", "spec"}
    items = sp.get("FINDINGS", [])
    if sp.get("FINDINGS_COUNT") != len(items):
        f.append(f"dispatch: FINDINGS_COUNT {sp.get('FINDINGS_COUNT')} but {len(items)} findings")
    seen = set()
    for it in items:
        m = re.match(r"(f\d\d) ", it)
        tag = re.search(r"\[([^\]]*)\]\s*$", it)
        if not m or not tag:
            f.append(f"dispatch: a finding without an id or evidence: {it[:50]}")
            continue
        if m.group(1) in seen:
            f.append(f"dispatch: {m.group(1)} twice")
        seen.add(m.group(1))
        for part in [x.strip() for x in tag.group(1).split(";")]:
            if part not in allowed:
                f.append(f"dispatch {m.group(1)}: unrecognised evidence '{part}'")
    return f


def check_all(t27c=None) -> list:
    if not SPEC.exists():
        return [f"{SPEC}: missing"]
    T, Q, P, L, V = (load(k) for k in ("ts", "queend", "pg", "lotus", "live"))
    missing = [k for k, d in zip(("ts", "queend", "pg", "lotus", "live"), (T, Q, P, L, V)) if d is None]
    if missing:
        return [f"{k}: no record; run `{k}`" for k in missing]
    f = []
    for name, d in (("ts", T), ("pg", P)):
        if d["browseros"]["head"] != BROWSEROS_PIN:
            f.append(f"{name}: the record is not at the supervisor pin")
    if L.get("pin") != TRINITY_PIN:
        f.append("lotus: the record is not at the trinity pin")
    sp = base.load_spec(SPEC)
    if sp.get("SUPERVISOR_PIN") != BROWSEROS_PIN or sp.get("PINNED_REVISION") != TRINITY_PIN:
        f.append("dispatch: the spec's pins differ from the tool's")
    t = T["result"]
    want = len(VERDICTS) * len(IDLES) * len(SEND_BACKS) * len(RELEASES) * 2
    if len(t["claims"]) != want:
        f.append(f"ts: {len(t['claims'])} claim rows, the grid has {want}")
    for cid, fnd, text, fn in record_claims(T, Q, P, L, V, sp):
        try:
            ok = bool(fn())
        except (KeyError, TypeError, IndexError, StopIteration) as e:
            ok, text = False, f"{text} ({type(e).__name__} {e})"
        if not ok:
            f.append(f"record {cid} [{fnd}]: {text}")
    f += check_findings(sp)
    t27c = t27c or base.t27c_path()
    if t27c and shutil.which("cc"):
        now = compile_status(t27c)
        for rel, ok in EXPECT_COMPILES_NOW.items():
            if now[rel]["compiles"] != ok:
                f.append(f"compile: {rel} compiles={now[rel]['compiles']}, the spec says {ok}")
        if now["specs/queen/task_analysis.t27"]["tests"] != 7:
            f.append(f"compile: task_analysis.t27 tests {now['specs/queen/task_analysis.t27']['tests']}, expected 7 passing")
        then = compile_status(t27c, MASTER_REF)
        for rel, ok in EXPECT_COMPILES_AT_MASTER.items():
            if then[rel]["compiles"] is not None and then[rel]["compiles"] != ok:
                f.append(f"compile at {MASTER_REF[:9]}: {rel} compiles={then[rel]['compiles']}, the finding says {ok}")
        r = (T.get("replay") or {}).get("result")
        if not r:
            f.append("replay: none recorded; run `run`")
        else:
            n = len(spec_cases(T, Q, P))
            if r.get("spec_sha256") != base.sha256(SPEC.read_bytes()):
                f.append("replay: the spec changed since the replay; run `run`")
            if not r.get("ok") or r.get("passed") != n or r.get("failed") != 0:
                f.append(f"replay: {r.get('passed')} passed, {r.get('failed')} failed of {n}: {r.get('detail', '')[:120]} {r.get('failures', [])[:3]}")
    else:
        f.append("check: t27c or cc missing, the compile and replay checks did not run")
    return f


# ===========================================================================================
# --self-check
# ===========================================================================================
def self_check() -> int:
    ok = True

    def expect(cond, what):
        nonlocal ok
        print(f"  {'ok ' if cond else 'BAD'} {what}")
        ok = ok and bool(cond)
    clone = lambda d: json.loads(json.dumps(d))
    T, Q, P, L, V = (load(k) for k in ("ts", "queend", "pg", "lotus", "live"))
    expect(all(d is not None for d in (T, Q, P, L, V)), "the five committed records exist")
    if not all(d is not None for d in (T, Q, P, L, V)):
        return 1
    sp = base.load_spec(SPEC)
    f = check_all()
    expect(not f, f"check: records, spec, compile status and replay agree ({len(f)} finding(s){': ' + f[0] if f else ''})")
    claims_of = lambda T_=T, Q_=Q, P_=P, L_=L, V_=V: [cid for cid, _, _, fn in record_claims(T_, Q_, P_, L_, V_, sp) if not fn()]
    p = clone(P)
    p["result"]["contention"]["max_winners"] = 2
    expect("q06" in claims_of(P_=p), "planted: two owners in one term breaks q06")
    p = clone(P)
    p["result"]["ceiling"]["round_state"] = "rejected"
    expect("q03" in claims_of(P_=p), "planted: a round that reads the stored release breaks q03 (the defect would be gone)")
    t = clone(T)
    t["result"]["board"]["escalate|closed"] = "review"
    expect("q04" in claims_of(T_=t), "planted: a board that keeps an escalated closed issue in review breaks q04")
    q = clone(Q)
    q["result"]["choose"][7][1] = 102
    expect("q02" in claims_of(Q_=q), "planted: one order answered by a later candidate breaks q02")
    l_ = clone(L)
    l_["lotus_cli_has_phase_option"] = True
    expect("q01" in claims_of(L_=l_), "planted: a lotus CLI with a --phase option breaks q01")
    v = clone(V)
    v["board"]["columns"] = ["backlog", "running", "done"]
    expect("q11" in claims_of(V_=v), "planted: a live board with other columns breaks q11")
    s2 = clone(sp)
    s2["FINDINGS"][0] = s2["FINDINGS"][0].replace("[lotus]", "[rumour]")
    expect(any("rumour" in x for x in check_findings(s2)), "planted: a finding with evidence nobody measured")
    t27c = base.t27c_path()
    if t27c and shutil.which("cc"):
        good = replay(T, Q, P, t27c)["result"]
        expect(good["ok"], f"replay: {good['passed']}/{good.get('fixtures')} cases")
        with tempfile.TemporaryDirectory() as tmp:
            tmp = pathlib.Path(tmp)

            def planted(old, new, path=SPEC):
                text = path.read_text(encoding="utf-8")
                assert text.count(old) == 1, old
                pp = tmp / path.name
                pp.write_text(text.replace(old, new), encoding="utf-8")
                return pp
            r = replay(T, Q, P, t27c, spec=planted("    if (verdict == V_FAILED or verdict == V_CANCELLED) { return C_FAILED; }\n", ""))["result"]
            expect(any(x["id"].startswith("claim-1-failed") for x in r["failures"]), "planted: a claim that forgets the failed verdict fails the grid")
            r = replay(T, Q, P, t27c, spec=planted("    if (claim != C_RUNNING and issue_list_known and issue_open == false) { return COL_DONE; }\n", ""))["result"]
            expect(any(x["id"] == "board-escalate|closed" for x in r["failures"]), "planted: a board without the closed-issue rule fails the board cases")
            r = replay(T, Q, P, t27c, spec=planted("    if (e0) { return 0; }\n", ""))["result"]
            expect(any(x["id"].startswith("choose-10") for x in r["failures"]), "planted: a choice that skips the first candidate fails the orders that start with it")
            r = replay(T, Q, P, t27c, spec=planted("        if (committed_known and committed > 0) { return R_ACCEPT; }\n        return R_ESCALATE;", "        return R_ACCEPT;"))["result"]
            expect(any(x["id"].startswith("review-1-1-1-0") for x in r["failures"]), "planted: a review that accepts without a commit fails the review grid")
            r = replay(T, Q, P, t27c, spec=planted("    return age_minutes > STALL_MINUTES;", "    return age_minutes >= 119;"))["result"]
            expect(any(x["id"] == "writer-stall" for x in r["failures"]), "planted: a reaper at 119 minutes fails the stall case")
            r = replay(T, Q, P, t27c, spec=planted("    if (state == S_ACCEPT) {\n", "    if (state == S_ACCEPT) {\n        if (event == E_CRITERIA_CHANGED) { return S_STALE; }\n"))["result"]
            expect(any(x["id"] == "stale-52" for x in r["failures"]), "planted: a table that lets a rewrite release an accept fails the stale case")
            r = replay(T, Q, P, t27c, spec=planted("pub fn releases_read_by_round(stored: u32) -> u32 {\n    return 0;", "pub fn releases_read_by_round(stored: u32) -> u32 {\n    return stored;"))["result"]
            expect(any(x["id"] == "ceiling-round" for x in r["failures"]), "planted: a round that reads the stored release fails the ceiling case")
            ta = planted("        if (priority_rank(scores, created, n, i) == position) {\n            return i;\n        }",
                         "        if (i == position) {\n            return i;\n        }", TASK_SPEC)
            res = run_spec_tests(t27c, ta)
            expect(res["compiles"] and res["tests"] != 7, f"planted: a priority order that returns its input fails task_analysis.t27's own tests ({res['tests']})")
    else:
        print("  skip replay checks: t27c or cc missing")
    print("trinity_queen_dispatch --self-check:", "ok" if ok else "FAILED")
    return 0 if ok else 1


# ===========================================================================================
# CLI
# ===========================================================================================
def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("command", nargs="?", choices=["ts", "pg", "lotus", "live", "run", "check"])
    ap.add_argument("--browseros-root")
    ap.add_argument("--bun", default="bun")
    ap.add_argument("--queend")
    ap.add_argument("--database-url")
    ap.add_argument("--trinity-root")
    ap.add_argument("--zig")
    ap.add_argument("--sysroot")
    ap.add_argument("--self-check", action="store_true")
    a = ap.parse_args()
    if a.self_check:
        return self_check()
    if a.command is None:
        ap.print_help()
        return 2
    wr = lambda path, doc: pathlib.Path(path).write_text(json.dumps(doc, indent=1, sort_keys=True, ensure_ascii=True) + "\n", encoding="utf-8")
    OUT.mkdir(parents=True, exist_ok=True)
    try:
        if a.command == "ts":
            if not (a.browseros_root and a.queend):
                print("ts: --browseros-root and --queend are required", file=sys.stderr)
                return 2
            ts, qd = build_ts_record(pathlib.Path(a.browseros_root).resolve(), a.bun, a.queend)
            old = load("ts") or {}
            if "replay" in old:
                ts["replay"] = old["replay"]
            wr(REC["ts"], ts)
            wr(REC["queend"], qd)
            print(f"ts: {len(ts['result']['claims'])} claim rows; queend: {len(qd['result']['choose'])} orders, {len(qd['result']['review'])} reviews, {len(qd['result']['retry'])} retry sequences")
            return 0
        if a.command == "pg":
            if not (a.browseros_root and a.database_url):
                print("pg: --browseros-root and --database-url are required", file=sys.stderr)
                return 2
            doc = build_pg_record(pathlib.Path(a.browseros_root).resolve(), a.bun, a.database_url)
            wr(REC["pg"], doc)
            c = doc["result"]["contention"]
            print(f"pg: lease {c['min_winners']}..{c['max_winners']} owners over {c['rounds']} rounds of {c['contenders']}; ceiling {doc['result']['ceiling']['round_state']} vs {doc['result']['ceiling']['full_row_state']}")
            return 0
        if a.command == "lotus":
            if not (a.trinity_root and a.zig):
                print("lotus: --trinity-root and --zig are required", file=sys.stderr)
                return 2
            doc = build_lotus_record(pathlib.Path(a.trinity_root).resolve(), a.zig, a.sysroot)
            wr(REC["lotus"], doc)
            print(f"lotus: {doc['modes']['Debug']['passed']} tests in Debug, {doc['modes']['ReleaseFast']['passed']} in ReleaseFast")
            return 0
        if a.command == "live":
            doc = build_live_record()
            wr(REC["live"], doc)
            print(f"live: {doc['status']['swarmState']}, {doc['board']['cards']} cards")
            return 0
    except (RuntimeError, OSError, subprocess.TimeoutExpired) as e:
        print(f"{a.command}: could not run: {e}", file=sys.stderr)
        return 2
    if a.command == "run":
        t27c = base.t27c_path()
        T, Q, P = load("ts"), load("queend"), load("pg")
        if not (t27c and shutil.which("cc") and T and Q and P):
            print("run: needs t27c, cc and the ts, queend and pg records", file=sys.stderr)
            return 2
        T["replay"] = replay(T, Q, P, t27c)
        wr(REC["ts"], T)
        r = T["replay"]["result"]
        print(f"run: {r['passed']} passed, {r['failed']} failed ({r['stage']}) {r['detail'][:120]} {r['failures'][:3]}")
        return 0 if r["ok"] else 1
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
