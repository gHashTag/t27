# AGENTS — Trinity S³AI / t27

This file is the **repository entry point** for humans and coding agents. It summarizes **where law lives** and **how to work safely** in this tree.

---

## 1. Read first (constitutional stack)

| Order | File | Role |
|------:|------|------|
| 1 | [`SOUL.md`](SOUL.md) | **Canonical** constitution (language policy, TDD mandate, validation). |
| 2 | [`docs/nona-03-manifest/SOUL.md`](docs/nona-03-manifest/SOUL.md) | Expanded reference; if it conflicts with root `SOUL.md`, **root wins**. |
| 3 | [`docs/T27-CONSTITUTION.md`](docs/T27-CONSTITUTION.md) | **SSOT-MATH**, **LANG-EN**, **DOCS-TREE** (where `docs/` files may live). |
| 4 | [`TASK.md`](TASK.md) + [`docs/coordination/TASK_PROTOCOL.md`](docs/coordination/TASK_PROTOCOL.md) | Multi-agent coordination, locks, anchor issue. |
| 5 | [`OWNERS.md`](OWNERS.md) | Domain ownership; each major directory may have its own `OWNERS.md`. |

Supporting: [`CONTRIBUTING.md`](CONTRIBUTING.md), [`SECURITY.md`](SECURITY.md), [`architecture/ADR-004-language-policy.md`](architecture/ADR-004-language-policy.md).

---

## 2. Agent model (27-letter alphabet)

- **Documentation map:** [`docs/README.md`](docs/README.md) — where first-party docs live (`agents/`, `coordination/`, `nona-01..03/`, `clara/`).
- **Full alphabet and roles:** [`docs/agents/AGENTS_ALPHABET.md`](docs/agents/AGENTS_ALPHABET.md) — canon for T, N, P, C, B, etc.
- **Operational agent specs (e.g. watchdogs, schemas):** [`docs/agents/AGENTS.md`](docs/agents/AGENTS.md) — complements the alphabet; not a second constitution.

Use **domain directories**, not “one folder per agent.” Primary contact for a path is the **Primary** listed in the nearest `OWNERS.md`.

---

## 3. Non-negotiables for changes

1. **Specs are source of truth** — behavior belongs in `.t27` / `.tri`; generated `gen/` output is not hand-edited (except documented exceptions).
2. **TDD inside specs** — new or changed specs need `test`, `invariant`, and/or `bench` where SOUL requires it.
3. **English + ASCII** — first-party Markdown and source comments per **LANG-EN** and **ADR-004**; grandfathered paths only in [`docs/.legacy-non-english-docs`](docs/.legacy-non-english-docs).
4. **No new Python on the verification critical path** — see **SSOT-MATH** and [`docs/nona-02-organism/TZ-T27-001-NO-PYTHON-CRITICAL-PATH.md`](docs/nona-02-organism/TZ-T27-001-NO-PYTHON-CRITICAL-PATH.md).
5. **Issue gate** — PRs should link issues (`Closes #N`) where project policy requires it.
6. **Ring / gold work** — follow [`docs/nona-01-foundation/GOLDEN-RINGS-CANON.md`](docs/nona-01-foundation/GOLDEN-RINGS-CANON.md) for parser/compiler/spec changes; compiler seal path: `bootstrap/stage0/FROZEN_HASH`.

---

## 4. Layout reminders (after repo hygiene refactors)

- **Core:** `specs/`, `compiler/`, `bootstrap/`, `gen/`, `conformance/`, `tests/`.
- **Non-core services & tooling:** `contrib/backend/`, `contrib/portable-claude-setup/`.
- **Vendored / datasets / upstream:** `external/` (e.g. OpenCode submodule, `external/kaggle/`).

---

## 5. Law Reference

The seven **Invariant Laws (L1–L8)** are defined in [`docs/T27-CONSTITUTION.md`](docs/T27-CONSTITUTION.md#2--invariant-laws-never-change-without-constitutional-amendment):

| Law | Name | Legacy alias | Summary |
|-----|------|-------------|---------|
| **L1** | TRACEABILITY | ISSUE-GATE | No code merged without `Closes #N` |
| **L2** | GENERATION | NO-HAND-EDIT-GEN | Files under `gen/` are generated; edit specs instead |
| **L3** | PURITY | SOUL-ASCII | Source files must be ASCII-only with English identifiers |
| **L4** | TESTABILITY | TDD-MANDATE | Every `.t27` spec must contain `test`/`invariant`/`bench` |
| **L5** | IDENTITY | PHI-IDENTITY | φ² = φ + 1; φ² + φ⁻² = 3; IEEE f64 checks use tolerance |
| **L6** | CEILING | TRINITY-SACRED | `FORMAT-SPEC-001.json` + `gf16.t27` are numeric SSOT |
| **L7** | UNITY | NO-NEW-SHELL | No new `*.sh` on critical path; use `tri`/`t27c` |

**Law Priority:** L1 > L2 > L3 > L4 > L5 > L6 > L7 > L8 (Asimov-style hierarchy)

---

## 6. Cursor / automation

- Rule file: [`.cursor/rules/t27-ssot-math.mdc`](.cursor/rules/t27-ssot-math.mdc) — keep in sync with **SSOT-MATH** and this entry point.

---

## 7. Crons and skills are specs

A scheduled job is a `.t27` card before it is anything else. Cron cards live in
[`specs/crons/`](specs/crons/README.md) (the README holds the schema and the
rules the generator enforces), skill cards in [`specs/skills/`](specs/skills/),
and the scheduler contract is
[`specs/automation/inngest-queen-scheduler.t27`](specs/automation/inngest-queen-scheduler.t27).
This repository is the only place to edit them.

Before you add, move, list or "migrate" a cron -- a workflow `schedule:`, an
Inngest function, a Railway cron service, a `setInterval`, a launchd plist or a
crontab line on the owner's Mac -- find its card here, and write one if it is
missing. Do not start a second registry (a database table, a JSON list, a
profile tab with its own store). A page that shows crons reads the cards.

Who reads the cards, and what each reader can and cannot tell you:

| Reader | Reads | Tells you | Does not tell you |
|---|---|---|---|
| `app.t27.ai/game/crons` (gHashTag/trinity `CronExplorer`) | a vendored copy, compiled at site build time into `/queen/crons/spec-crons.json` | the card typechecks and the code it names exists | whether the job ran: `health: ok` describes the card, and the page loads no run history |
| The Queen: Inngest app `t27-queen` in gHashTag/BrowserOS `trios/agent-server` | its own vendored copy, pinned to a t27 commit (`GET /queen/scheduler`, field `pin`) | how it serves each card: `HOST github-actions` gets `workflow-dispatch`, every other host gets `tick-only` | anything about a `tick-only` card, whose tick event has no consumer |
| The host (GitHub Actions, the 999 Inngest app, Railway, the process) | its own schedule | whether the job ran and how it ended | -- |

So "is this cron working?" is answered by the host, never by the card or the
page: `gh run list -R <repo> --workflow <file> --event schedule` for
`github-actions`, the 999 Inngest dashboard for `inngest`, the service logs for
`timer` and `railway-cron`. Put what you measured into the card's `NOTE` with
its date, and re-measure a `NOTE` before you repeat it.

Gaps that are open until a PR closes them. Do not assume otherwise:

- **The owner's Mac has no `HOST`.** The vocabulary is `github-actions`,
  `inngest`, `railway-cron`, `timer`. launchd jobs, crontab lines and Claude
  Code scheduled tasks therefore have no card, and neither the page nor the
  Queen sees them. A new host is a schema change in `specs/crons/README.md`
  first, then in the trinity generator (`apps/website/scripts/agents-from-specs.mjs`),
  then in the Queen's `src/inngest/plan.ts`.
- **Dispatched workflows fire twice.** A `github-actions` card the Queen
  dispatches still has its `schedule:` block until a PR removes it, in the
  order fixed by
  [`docs/now/2026-09-13-the-queen-holds-the-scheduler-crons-and-skills-as-one-inngest-app.md`](docs/now/2026-09-13-the-queen-holds-the-scheduler-crons-and-skills-as-one-inngest-app.md).
- **There are three copies.** The trinity site and the Queen each vendor the
  cards. Edit them here, then refresh the copies: trinity
  `apps/website/scripts/sync-t27-specs.mjs`, the Queen
  `trios/agent-server/scripts/sync-t27-specs.sh`. Check the Queen's `pin` after
  a squash merge: a pin to a deleted branch cannot be reproduced.

---

**φ² + 1/φ² = 3 | TRINITY**

---

## Only t27 (owner hard rule, 2026-10-05)

Owner's rule, 2026-10-05: no new commit or pull request may contain code in
another language; the hooks must stop it. It binds people, bees and agents alike.

- **Allowed:** `.t27` specs; files `t27c` generated under a generated root
  (`gen/`, `bootstrap/gen/`, `bootstrap/src/memory/generated/`); prose and data
  (`.md`, `.txt`, `.json`); and **deletions**.
- **Denied:** adding or modifying hand-written code in any other language --
  `.rs .py .ts .js .sh .zig .c .go .v .lean .yml .toml`, Dockerfile, Makefile and
  the rest of the list in the spec. A change to existing Rust
  (`bootstrap/src/compiler.rs`, `cli/t27b`) or Python is denied too.
- **The rule is** `specs/policy/own_language.t27`; its `t27c gen-c` output
  `gen/c/policy/own_language.c` runs in `lefthook.yml` (pre-commit, pre-push)
  and in `.github/workflows/own-language.yml` on every pull request. Read the
  spec for the exact list; do not copy it here.
- **The only override** is the label `owner-approved-foreign`. It is applied
  only on the owner's explicit approval, and the pull request body quotes that
  approval (in English, with its date and where it was given). Changing the gate
  itself needs the label as well.
- **Owner-approved exceptions** live in `tools/policy/foreign-exceptions.txt`:
  data, one path prefix per line, each under a comment naming the approval. On
  an approved branch, add the entry in the same branch; the local hooks read the
  working-tree list, so the commit goes through. CI reads the list from the pull
  request's base, and any change to the list file needs the owner's label, so
  every entry on master was approved by the owner. There is no env-var bypass.
- **Existing foreign code is debt** that only shrinks: by deletion, or by
  replacing it with a spec and its generated output.
- **`--no-verify` is forbidden**, and so is `LEFTHOOK=0`. Install the hooks
  once per clone with `lefthook install`.
- What gen cannot express yet is a compiler defect: file it on the self-host
  epic (#5980) instead of writing the code by hand.

## t27b is written in t27

Owner's rule, 2026-10-05: t27b -- the native backend -- and the tools that
measure it are written in **t27**, not in Rust or Python by hand.

- New t27b logic starts as a `.t27` spec with `test` blocks and reaches Rust
  or Python only through `t27c gen-rust` / `gen-c` / `gen-js`. The generated
  file is never hand-edited (L2).
- What gen cannot express yet is a defect of the self-host work (#5980): file
  it there. Do not work around it in a hand-written file.
- A hand-written addition or modification is denied by the gate in "Only t27"
  above unless the owner labels the pull request `owner-approved-foreign`; such
  a PR still names the spec that will replace it and links the port epic #6198.
- The debt only shrinks. On 2026-10-05 (master c532fcae5, `wc -l`) it is
  `cli/t27b/src/*.rs` 8247 lines plus `cli/t27b/tests/*.rs` 2865 (Rust, and
  it mounts `bootstrap/src/compiler.rs`; 14338 plus 7070 after #6864's
  brace-invariant predicates, +54 and +53; 14327 plus 7072 after #6911's
  module-var-in-test fix, -11 and +2; 15453 plus 7719 after #7368's odd-width
  integers, +111 and +42 on master cd6708d32's 15342 plus 7677; 14952 plus 7757
  after #7531 moved the A64 encoders to `specs/tri/t27b/a64.t27`, -520 on master
  ec5c3cf78's 15472 plus 7757; 14847 plus 7775 after #7526's Mach-O port to
  `specs/tri/t27b/macho.t27`, -132 and 0 on master 27493414d's 14979 plus
  7775; 14745 plus 7775 after #7549 moved `bitmask_imm`, `logic_imm` and
  `mov_imm` to `a64.t27` too, -102 on master 499487306's 14847 plus 7775;
  14812 plus 7814 after #7394's glue for `@abs`, `@max`, `@min` and
  `std.math.pi` / `e`, whose plan is `specs/tri/t27b/builtin_plan.t27`, +43
  and 0 on master 95182e95b's 14769 plus 7814; 14848 plus 7833 after #7412's
  glue for `@intCast` with an integer result type, whose plan is
  `specs/tri/t27b/int_cast_plan.t27`, +36 and +19 on master 1120d30ae's 14812
  plus 7814; 14881 plus 7848 after #7391's glue for `@exp`, whose plan is
  `specs/tri/t27b/libm_plan.t27` and whose routines are
  `specs/tri/t27b/libm.t27`, +33 and +15 on master 6fba037d1's 14848 plus
  7833; 14881 plus 7858 after #7217's `@log` from the same plan, 0 and +10
  on master 8c7b2ccff's 14881 plus 7848; 14917 plus 7877 after #7423's glue
  for integer constants wider than 64 bits, whose plan is
  `specs/tri/t27b/wide_plan.t27`, +36 and +19 on master 8415029ea's 14881
  plus 7858; 14945 plus 7877 after #7550's refusal of a return that hands
  out the address of the fn's own frame, +28 and 0 on master 5f3087125's
  14917 plus 7877; 14919 plus 7874 after #7422 removed the stale
  `StmtAssign(reference redeclares)` scan, -26 and -3 on master 2eabc3edd's
  14945 plus 7877; 14876 plus 7874 after #7673 moved the greedy blockers
  order, its replay and FNV-1a to `specs/tri/t27b/blockers.t27`, -43 and 0
  on master ef26684a1's 14919 plus 7874; 14842 plus 7866 after #7680's glue
  for array literals the reference prints as `@constCast(&[_]E{ ... })`,
  whose plan is `specs/tri/t27b/slice_lit_plan.t27`, -34 and -8 on master
  3d7130691's 14876 plus 7874; 14873 plus 7905 after #7459's glue for
  `@setEvalBranchQuota`, +31 and +39 on master 6157f8d07's 14842 plus 7866;
  14885 plus 7924 after #7740's glue for the
  `type mismatch` family, whose plan is `specs/tri/t27b/coerce_plan.t27`,
  +12 and +19 on master b8cb93f34's 14873 plus 7905; 14905 plus 7942 after
  #7690's glue for a local bound to a void fn's result, whose plan is
  `specs/tri/t27b/void_bind_plan.t27`, +20 and +18 on master 1d9c5960d's
  14885 plus 7924; 14944 plus 7964 after #7691's glue for a local bound to
  the W585 scaffold the reference never calls, whose plan is
  `specs/tri/t27b/scaffold_plan.t27`, +39 and +22 on master d42df2b06's
  14905 plus 7942; 14967 plus 7987 after #7448's glue for module vars that
  hold a string or start undefined, +23 and +23 on master 579bc55aa's 14944
  plus 7964; `cli/t27b/src/*/*.rs`, which the glob above never counted, is
  1222 more on master 1f2448e98, 16189 in all; 14995 plus 1166, 16161 in
  all, after #7524's `.len` call port, +28 glue and -56; 15026 plus 1177,
  16203 in all, and `cli/t27b/tests/*.rs` 8005, after #7735's glue for `.{}`
  by its result type and the refusal of a store of the fn's own frame address,
  whose plans are `specs/tri/t27b/empty_lit_plan.t27` and
  `specs/tri/t27b/frame_store_plan.t27`, +31, +11 and +18 on master
  4d7d2d3bd's 14995 plus 1166 and 7987; 15043 plus 1202, 16245 in all, and
  tests 8027, after #7765's refusal of a write through an array literal the
  reference prints as `@constCast(&[_]E{ ... })`, whose plan is
  `specs/tri/t27b/slice_lit_plan.t27`, +17, +25 and +22 on master 0c729d928;
  15064 plus 1202, 16266 in all, and tests 8043, after #7796's glue for
  `undefined` as an argument the callee never reads, whose plan is
  `specs/tri/t27b/undefined_arg_plan.t27`, +21 and +16 in the tests on master
  7f7bf2cb6's 15043 plus 1202 and 8027; 15100 plus 1202, 16302 in all, and tests
  8062, after #7791's glue for `@bitCast` and `@intFromBool`, whose plan is
  `specs/tri/t27b/bit_cast_plan.t27`, and for `std.math.nan`, `isNan`,
  `isPositiveInf` and `isNegativeInf` from `libm_plan.t27`, +36 and +19 on
  master f53a734b7's 15064 plus 1202 and 8043; 15145 plus 1202, 16347 in all,
  and `cli/t27b/tests/*.rs` 8094, after #7790's glue for `@divTrunc` and a
  `@panic` statement (`specs/tri/t27b/builtin_plan.t27`), +16 and +19 on
  master 14e1c7be9's 15129 plus 1202 and 8075; 15183 plus 1202, 16385 in all,
  and tests 8105, after #7737's glue for `?*anyopaque` and `@ptrFromInt` to an
  optional pointer, whose plan is `specs/tri/t27b/opaque_plan.t27`, +38, 0 and
  +11 on master 81da89413's 15145 plus 1202 and 8094; 15191 plus 1202, 16393
  in all, and tests 8119, after #7819's glue for `@floor`, `@ceil`, `@round`,
  `@trunc` and `@rem` of an f32 from `libm_plan.t27`, +8 and +14 on master
  bbb77f0c3's 15183 plus 1202 and 8105; 15195 plus 1202, 16397 in all, and
  `cli/t27b/tests/*.rs` 8134, after #7803's glue for `@floatFromInt` of a
  comptime_int (`specs/tri/t27b/coerce_plan.t27`), +4 and +15 on master
  10831352c's 15191 plus 1202 and 8119; 15224 plus 1202, 16426 in all, and
  tests 8149, after #7810's glue for `_ = e;` with e not a bare name, whose
  plan is `specs/tri/t27b/discard_plan.t27`, +29 and +15 on master 3d16be1f2's
  15195 plus 1202 and 8134; 15227 plus 1202, 16429 in all, and tests 8164,
  after #7811's glue for an optional enum compared with a variant, whose plan
  is `specs/tri/t27b/optional_compare_plan.t27`, +6 -3 and +15 on master
  824c82d78's 15224 plus 1202 and 8149; 15232 plus 1233, 16465 in all, and tests
  8183, after #7857's glue for `anytype` and `[*]T` in the signature of a fn
  nothing analyzed reaches (`specs/tri/t27b/lazy_sig_plan.t27`), +5, +31 and +19
  on master 408d740d9's 15227 plus 1202 and 8164; 15271 plus 1233, 16504 in
  all, and tests 8191, after #7742's glue for `==` and `!=` between pointers
  and optional pointers, whose plan is `specs/tri/t27b/ptr_eq_plan.t27`, +39,
  0 and +8 on master 42707603a's 15232 plus 1233 and 8183; 15271 plus 1239,
  16510 in all, and `cli/t27b/tests/*.rs` 8205, after #7805's glue for a call
  of a fn returning a float in `x as T` (`specs/tri/t27b/float_as_plan.t27`),
  +6 and +14 on master 2dfe9786f's 15271 plus 1233 and 8191; 15271 plus 1239,
  16510 in all, and tests 8215, after #7822's `@bitCast` result type from an
  assignment, for the f32 `@log`, `@log2` and `@log10` ports, +0 and +10 on
  master 09ffc2fc5's 15271 plus 1239 and 8205),
  `scripts/tri_loop/t27b.py` 1829
  (Python, `tri t27b`; 894 at c532fcae5, 1481 after #6317's `next`, 1568 after #6334's master look-back, 1829 after #6445's `reduce` wiring), `scripts/tri_loop/t27b_reduce.py` 632 (Python, `tri t27b reduce`, #6445; its decisions are `specs/tri/t27b/reduce.t27`), and `contrib/railway/t27b-lab/lab.py` 608 (Python, the
  Railway lab; 956 on master 0fbb0a033, 986 after #7672's lane requests). Update these numbers in the PR that moves them.

This is the "Own language first" rule below, applied to code: a project whose
claim is "here is a language worth writing" writes its own backend in it.

**A lane's evidence (#7672).** A t27b coverage PR proves "nothing regressed"
with two signed corpus receipts, not a pasted table of lab numbers.

- Base: `git merge-base origin/master HEAD`. Head: the PR's head, pushed to a
  branch on origin. Use `https://t27b-lab-production.up.railway.app/runs/<sha>.receipt.json`
  when it exists; otherwise request it with a fresh challenge you keep:
  `c=$(openssl rand -hex 32); ssh t27b-lab "mkdir -p /srv/requests && printf %s $c > /srv/requests/<sha>"`.
  The lab runs one request between master polls (about 10 minutes each, at most
  4 waiting, dropped after 6 hours) and publishes `/runs/<sha>.json` and the receipt.
- From a master checkout's root: `t27c corpus-receipt compare BASE.json HEAD.json
  --challenge <base's> --challenge-head <head's>` (omit a challenge you did not
  write). Paste its output and exit code: `lane` lines name each changed file.
- Exit 3 IMPROVED_ONLY, 0 EQUIVALENT or 4 NEUTRAL may merge. Exit 1 REGRESSED
  blocks the merge; exit 2 REFUSED means the evidence did not authenticate, so
  it proves nothing. The rules are `specs/verified/corpus_receipt.t27`.

## Actors, not threads (owner rule, 2026-10-08)

Owner's rule, 2026-10-08: the Queen and the swarm move to the actor model.
Tracking: #7851.

- **Everything that runs at the same time is an actor.** That covers a round,
  a reviewer, a poller and a runner lane. Each one has a pid and a bounded
  mailbox, and a supervisor restarts it. The rules are one spec,
  `specs/queen/actors.t27`, built on BEAM/OTP: links, monitors, a control lane,
  one_for_one / one_for_all / rest_for_one, restart intensity and backoff. A
  runtime reaches those rules only through the spec's generated card.
- **No new loops.** Queen or swarm work gets no new `setInterval` timer, thread
  pool, worker counter or ad-hoc promise fan-out. A new concurrent piece is a
  new actor under a supervisor.
- **Replace in this order, never another:**
  1. An MVP of the actor runtime runs the same workload.
  2. Its tests pass.
  3. A benchmark runs the current loop and the actor runtime on the same input,
     measuring throughput, latency, recovery from a crashed worker, and work
     lost.
  4. The numbers are posted on #7851.
  5. Only then does the swap happen, and only if the actors are at least as good
     on every measure.

  Never swap first and measure later.
- **Where the code lives.** The Queen server lives in `gHashTag/trios`, on
  branches `queen` and `queen-runners`. `gHashTag/BrowserOS` is a fork of
  someone else's project. It holds the browser extension and nothing else: no
  Queen code, no server code, no specs, no plans.

This section is the rule's one full home. The AGENTS.md of our other
repositories point here; they do not copy it.

## Own language first

When this project publishes something about itself, it publishes in **this
project's own language and format** -- not translated into somebody else's.

Owner's rule, 2026-09-20: stop writing in other people's languages, we have our
own.

This bites on any file whose only reason to exist is that an outside tool
expects that shape: `llms.txt`, `agents.json`, `ai.txt`, `.well-known/*.json`,
A2A agent cards, `ai-plugin` manifests, OpenAPI stubs, JSON-LD blocks, a README
that restates a spec. The reflex is to write four of them in four foreign
formats, and the reflex is wrong: a project whose claim is "here is a language
worth writing" and which then describes itself in three of other people's
formats has published three documents that are not true of it.

**The move:** find the address the outside world already fetches, then serve our
own language at it. `/llms.txt` at t27.ai **is** a t27 module -- `llms.txt`
requires nothing but text, and every prose line of a `.t27` file is a `;`
comment, so it stays readable to anything that cannot compile it.

**Three qualifications, so the rule stays honest:**

- A format a resolver genuinely parses -- a sitemap, `package.json`, a lockfile
  -- is machinery, not a description. **Generate** it from our own source; never
  hand-write it into a second home for the truth.
- Code against someone else's API uses their types. Prose for a human who has
  never heard of the project uses that human's language.
- If a format demands a claim we cannot back, **publish nothing**. An A2A card
  with no A2A server behind it is a false claim, and a missing file is more
  honest than a lying one.

The test: *is this file the project speaking about itself?* If yes, it speaks
our language. If it is plumbing, it speaks the plumbing's.

**Worked example, compiler-checked rather than asserted:** in `gHashTag/trinity`,
`apps/website/public/t27/files/specs/catalog/onboarding.t27` generates
`/llms.txt` and `/agents.t27` byte-identically, gated in CI as
`check:onboarding`. The generator evaluates the spec's own `test` blocks --
`typecheck.ok` stays true for `assert 1 > 2`, so a compiler saying "this parses"
is not a compiler saying "this is true" -- and re-compiles the rendered document
before writing it.

**The full rule lives in exactly one place: the `own-language-first` skill**
(`~/.claude/skills/own-language-first/SKILL.md`). It carries the consent gate for
documents addressed to other people's agents, the six negative controls, and the
`;`-alone-on-a-line trap that silently discards a `module` declaration. This
section is a pointer, not a copy -- the recorded defect in this codebase family
is the hand-copied rule that only two of its three homes knew about.
