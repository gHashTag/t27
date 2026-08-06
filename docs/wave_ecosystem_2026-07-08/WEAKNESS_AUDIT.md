# Weakness Audit — Trinity Ecosystem → t27 Rewrite

**Date:** 2026-07-08 (baseline: t27 master `4832ec6a`)
**Scope:** 7 репо → рерайт в `.tri` спеки под корнем `gHashTag/t27`
**Author:** Wave loop agent (session 6734fbbe)
**Purpose:** Локализовать реальные слабости ДО написания эпика, чтобы план не был косметическим.

---

## Executive Verdict (токсично-честный)

**Экосистема НЕ едина.** 7 репозиториев говорят на 4 разных языках (Zig, Rust, TypeScript, `.t27`/`.tri`), три из них приватные, `trios-t27` до сих пор пустой, а сам t27 имеет **20+ конфликтующих Wave-Loop PR-ов** (#1362-#1437) — это не Wave, это очередь. Численный SSOT дрейфует по трём числам одновременно (84 в arXiv:2606.09686 / 83 live / 77 в stale gen JSON). IGLA CODER/RACE как обучающий корпус **сегодня не существует** — есть 601 `.t27` файлов, но нет ни пайплайна `(spec, gen)` пар, ни decontam-guard-а, ни detokenizer-а, ни манифеста с SHA-256.

Полный переезд 7 репо в один corpus за один Wave — нереально. **Честный масштаб:** пилотная переупаковка 6-8 модулей (по одному из каждого репо), генератор пар `(spec, gen)`, каркас датасета IGLA CODER v0.1, отчёт с 3 вариантами cooperation. Всё остальное — Wave-2/3/4.

---

## W-A: Language Fragmentation (Zig × Rust × TypeScript × .t27/.tri)

| Repo | Primary lang | LOC-класс | Rewrite cost to `.tri` |
|---|---|---|---|
| t27 | Zig + `.t27` + `.tri` | large | LOW — уже корень, добавить недостающие `.tri` фронты |
| trinity | Zig | medium | MED — Mozg/DNA → `specs/organism/mozg.tri`, `specs/organism/dna.tri` |
| trios | Zig | large | HIGH — 380+ open issues, git-orchestrator surface большая |
| trios-mcp | Rust | small | LOW — тонкий MCP wrapper, легко в `specs/mcp/wrapper.tri` |
| trios-t27 | EMPTY | 0 | N/A — deprecated по решению юзера, t27 = корень |
| 999-multibots-rust | Rust | medium | MED — но приватный, PII-риск |
| 999-multibots-telegraf | TypeScript | large | HIGH — приватный, платёжка, PII-риск, EPIC #358 в полёте |

**Слабость:** одного `.tri`-компилятора недостаточно — нужны **три кодогена**: `zig`, `rust`, `typescript`. Zig и Rust уже есть в `compiler/codegen/`, TypeScript **отсутствует** (`compiler/codegen/typescript/` нет).

**Митигация в этом Wave:** пилоты — только для тех модулей, где кодоген уже существует (zig, rust, c). TypeScript кодоген → Wave-3 (см. `epic/EPIC.md`).

**Residual risk:** приватные репо (999-multibots-*) не могут стать частью публичного IGLA CODER — их spec-и должны либо анонимизироваться, либо жить в отдельном приватном `dataset-igla-race-private`. Не решается за один Wave.

---

## W-B: Spec ↔ Code Drift (нет CI-инварианта что gen === spec)

**Наблюдение:** сегодня в t27 есть 601 `.t27` файлов и большая директория `gen/`. Нет CI-гейта, который бы:
1. Регенерил `gen/**` из `specs/**` на каждом PR
2. Падал, если `git diff gen/` не пуст (значит либо spec поменяли без regen, либо gen руками правили — оба нарушают **L2**)

**Прямое доказательство drift-а:** issue #1120 — `gen/numeric/formats_catalog.json` показывал 77, live SSOT = 83; PR #1128 сейчас **untracks** stale gen artifacts — то есть решение "не коммитить gen" вместо "прогонять CI". Это лечит симптом, не причину.

**Митигация в этом Wave:**
- В эпике зафиксировать **HR-drift**: каждый ring-NNN PR обязан приложить `gen-diff-report.txt` (`git diff --stat gen/`), а CI — падать если diff непустой (см. `epic/EPIC.md` §5).
- Пилотные `.tri` идут через полный цикл `edit → seal hash → gen → test → verdict → commit` — это доказывает, что PHI LOOP механически проверяем.

**Residual risk:** ретро-фикс 601 существующих `.t27` не в скоупе — только новые модули этого Wave.

---

## W-C: 20+ конфликтующих Wave-Loop PR-ов (technical debt на master)

**Список:** #1362, #1364, #1369, #1372, #1373, #1375, #1378, #1382, #1384, #1387, #1390, #1392, #1394, #1396, #1400, #1403, #1406, #1426, #1430, #1432, #1434 — все на своих `wave-loop-XYZ` ветках, все про IGLA/PVT/OSCFSEL. Многие ссылаются друг на друга (#1394 closes #1393, и т.д.).

**Слабость:** merge queue не работает, автор пишет "Wave Loop N" каждый день, конфликты между ними растут кубически. Ни один из них не в скоупе этого Wave — но они замусоривают issue-space.

**Митигация:** этот Wave НЕ трогает ни один Wave-Loop PR. Мы открываем свою ветку `ring-105-ecosystem-tri-rewrite` и делаем **новую** ring-серию, не сталкиваясь с существующими wave-loop-*.

**Residual risk:** если master сдвинется на массивный merge пока Wave идёт — придётся rebase. Оценка: 1-2 часа доп.работы.

---

## W-D: NO End-to-End PHI LOOP proof

**Наблюдение:** в t27 есть отдельные куски цикла:
- `compiler/parser/parser.t27` — есть
- `compiler/codegen/{zig,c,verilog,python}/codegen.t27` — есть
- `bootstrap/target/release/t27c` — бинарь есть (упомянут в правилах юзера)
- `experience/` директории — **нет** в клонированном срезе
- `.trinity` папки — **нет**

**Прямое доказательство:** `find . -maxdepth 3 -type d -name 'experience' -o -name '.trinity'` в клоне выдаёт пусто.

**Значит:** PHI LOOP шаг "save experience → skill commit" сегодня механически не проверяется. Это устный контракт, а не CI.

**Митигация в этом Wave:**
- Пилот включает создание `experience/ring-105-{module}/verdict.json` для каждого модуля с полями `{spec_hash, gen_hash, tests_passed, invariants_verified, bench_delta_pct, timestamp, agent_id}`.
- Отдельный `.tri` спек `specs/organism/experience.tri` описывает формат этой директории (types + invariants + tests).

**Residual risk:** без CI-гейта на существование `experience/**` для каждого PR — можно снова забыть. В этом Wave добавляем **hint-check**, не hard-fail.

---

## W-E: IGLA CODER/RACE не существует как dataset artefact

**Наблюдение:** упоминания IGLA CODER/RACE есть в BENCHMARKS.md и в правилах — но:
- **Нет** директории `dataset/igla-coder/` в t27
- **Нет** манифеста SHA-256 для (spec, gen) пар
- **Нет** decontam guard-а (Lee 2022 k=50) против утечки в eval
- **Нет** detokenizer / tokenizer spec для .tri/.t27 семьи
- Live corpus для IGLA-Coder (WP-8 diagnostic) = **1957 train tokens** — на 3+ порядка ниже phi-1-small floor (350M param, ~1B tokens)

Скилл `tt-lang-integration-weakness-map` подтверждает W-18 "data-starvation" **HIGH**: даже после WP-9/WP-10 augmentation deficit ~262× относительно Chinchilla-optimal.

**Митигация в этом Wave:**
- Создать `dataset/igla-coder/v0.1/` со структурой: `pairs/spec_XXXX.tri + gen_XXXX.zig`, `MANIFEST.json` (SHA-256 per pair), `DECONTAM_REPORT.md`, `README.md`.
- Из 8-10 пилотных .tri модулей + их gen выходов собрать первую волну ~20-40 pair-ов. Это **не решает** data-starvation (нужно 100-1000×) — но кладёт **честный якорь**: v0.1 существует, MANIFEST валидный, decontam CLEAN.
- В отчёте explicitly: "v0.1 = seed, W-18 остаётся HIGH".

**Residual risk:** реальный тренинг IGLA-Coder на этом seed **не будет работать** (n=40 pair-ов). Это **не претензия**, а **честный baseline** для Wave-2 (augment) и Wave-3 (scale).

---

## W-F: Numeric catalog count drift (paper #3 v4 = 84, live SSOT = 83)

**Наблюдение:** aрXiv:2606.09686 v4 Table 1 = 84 formats/13 clusters; live `specs/numeric/formats_catalog.t27` HEAD `4832ec6a` = **83** (проверено `grep -c '// CATALOG:' ...`); `paper3-methodology` repo README всё ещё headlined "84-Format". Это = **W-1** (single-author optics: не поймано ревьюером) и **W-9** (vector packs) из `tt-lang-integration-weakness-map`.

**Митигация в этом Wave:**
- Wave не трогает paper #3 (не в скоупе).
- Wave добавит в `epic/EPIC.md` явную HR-count: любой `.tri`/`.t27`/README, ссылающийся на количество форматов, **обязан** live-check-ить через `grep -c '// CATALOG:' specs/numeric/formats_catalog.t27` и включать этот вывод в PR body.

**Residual risk:** paper #3 errata (или SSOT растяжка до 84) — отдельная задача, не этот Wave.

---

## W-G: `trios` и `trinity` частично дублируют MCP/git surface

**Наблюдение:**
- `trios` (Zig, 8 open issues) — Git orchestrator + dual-MCP + Vision
- `trios-mcp` (Rust, 3 open issues) — Rust MCP wrapper
- `trinity` (Zig, 18 open issues) — Mozg/DNA
- `trios-mcp-rag` (не в этой семёрке, но упомянут в scientific-works-canon) — Rust MCP server, 13 tools, Railway

**Слабость:** MCP-surface растянут по 3-4 репо, идентификаторы tools разные. Нет единого `specs/mcp/*.tri` каталога.

**Митигация в этом Wave:**
- Один пилотный `.tri`: `specs/mcp/tool_registry.tri` (types + invariants + tests) как **предложение** единого реестра. Кодогенится в `trios-mcp` (Rust) и в `trios` (Zig, если существует surface).
- Не рефакторим внутренности `trios-mcp-rag` (другой репо, скилл-владелец `render-pipeline-mcp`).

**Residual risk:** дедуп MCP-tool имён между 4 репо — Wave-3 (см. EPIC.md §7 "roadmap after this Wave").

---

## W-H: приватные репо (999-multibots-rust, 999-multibots-telegraf) вне публичного корпуса

**Наблюдение:** оба **PRIVATE** — контент под NDA-класс (Telegram bots, платёжка, tariffs, PII в конверсиях). Нельзя включать в публичный IGLA CODER.

**Митигация в этом Wave:**
- Для этих двух репо в эпике зарезервировано **место** (ring-105-nnn-* branches), но пилот НЕ пишется в этом Wave.
- В отчёте — явная строка: "999-multibots-* rewrite gated on decision: public/private split for dataset".

**Residual risk:** архитектурное решение (публиковать анонимизированный слой? хранить приватно?) — вопрос юзера, не агента.

---

## W-I: `trinity` issue #601 = API credential leak (не наш скоуп, но не игнорировать)

**Наблюдение:** issue #601 в gHashTag/trinity — по названию похоже на утечку API токена (не открывал полный body, чтобы не тиражировать).

**Митигация в этом Wave:**
- НЕ трогать issue #601 в этом Wave (это security incident response, не rewrite).
- В отчёте — строчка "trinity#601 flagged as security-critical, out of scope for this ecosystem Wave".

**Residual risk:** если утечка живая — юзер обязан ротировать ключ независимо от Wave. Это упоминается в отчёте `report/FINAL_REPORT.md`.

---

## W-J: SSOT-MATH нарушается везде где live Python на critical path

**AGENTS.md §3.4:** "No new Python on the verification critical path — see SSOT-MATH and docs/nona-02-organism/TZ-T27-001-NO-PYTHON-CRITICAL-PATH.md"
**Прямое доказательство обхода:** `bootstrap/t27c.py` существует (найден в разведке).

**Слабость:** правило L7 UNITY / SSOT-MATH формально запрещает Python на критическом пути, но stage-0 бутстрап именно на Python. Это либо документированное исключение, либо реальное нарушение.

**Митигация в этом Wave:**
- НЕ трогаем `bootstrap/t27c.py` (риск сломать всё).
- В отчёте — flag: "L7 UNITY имеет живое исключение через `bootstrap/t27c.py`; либо документировать явно (ADR), либо мигрировать в Rust (Wave-3+)".

**Residual risk:** конфликт формулировок в конституции vs код — Wave-3.

---

## W-K: 10 существующих `.tri` в specs/ — 4 из них выглядят как заглушки (03-*)

**Файлы:** `03-bootstrap-lexer.tri`, `03-simple-parser.tri`, `03-tri-bootstrap-compiler.tri`, `04-tri-runtime.tri`, `04-tri-codegen.tri` — все с префиксами 03-/04-, что подразумевает stages. `01-tri-lang-core.tri` (просмотрен) — полноценный, с 8 tests + 3 invariants + benchmarks header.

**Слабость:** не проверено, все ли из 10 существующих `.tri` проходят `t27c parse` сегодня. Если хотя бы один падает — наш new pilot будет "рядом с багом", и любые изменения в парсере ломают baseline.

**Митигация в этом Wave (первая механическая проверка перед реализацией):**
- Прогнать `bootstrap/target/release/t27c parse specs/*.tri` на всех 10 файлах, залогировать в `audit/BASELINE_PARSE.log`. При падении — либо чинить (если тривиально), либо отмечать в epic как pre-existing failure.
- Если бинарь t27c недоступен в sandbox — пилоты валидируются как **PENDING TRAIN-BOX**, честно.

**Residual risk:** без работающего `t27c` в sandbox — pilot spec-и не могут быть машинно провалидированы, только человеком. Это = deferred verification. Явный tag в отчёте.

---

## Weakness Score Card

| ID | Title | Severity | In-scope this Wave? | Mitigation quality |
|----|---|---|---|---|
| W-A | Language fragmentation 4 langs | MED | Partial (zig+rust+c only) | Deferred TS to Wave-3 |
| W-B | Spec↔code drift (нет CI regen check) | HIGH | Partial (add HR-drift, no CI) | Hint-check only |
| W-C | 20+ conflicting Wave-Loop PRs | LOW (parallel) | Avoid (new ring branch) | Isolated |
| W-D | No E2E PHI LOOP proof (experience/) | HIGH | Partial (pilot only) | Local artefact, no CI gate |
| W-E | IGLA CODER/RACE не существует | HIGH | Partial (v0.1 seed) | Honest v0.1 baseline |
| W-F | Catalog count drift 84/83/77 | MED | Add HR-count rule | Documented only |
| W-G | MCP surface duplication | MED | 1 pilot registry.tri | Proposal, not merge |
| W-H | Private repos vs public corpus | MED | Reserved slots, no pilot | Decision pending user |
| W-I | trinity#601 credential leak | HIGH (security) | Out of scope | Flagged in report |
| W-J | Python on critical path (t27c.py) | LOW | Out of scope | Flagged in report |
| W-K | 10 existing .tri parse baseline unknown | MED | Baseline parse attempt | Pre-flight check |

**Honest verdict:** 5 из 11 weaknesses = HIGH; 3 из них lediglich **partially** mitigated в этом Wave. Полный ecosystem unify — Wave-2..Wave-4.

---

## What this Wave WILL produce (locked scope)

1. **Epic** (`epic/EPIC.md`) — декомпозиция на ring-105-NNN веток, по одной на репо/модуль.
2. **6-8 пилотных `.tri` спеков** (по одному на каждый живой репо, кроме `trios-t27` и приватных где не входит).
3. **Каждый пилот** — L4-compliant: types + functions(3) + invariants(3) + tests(8) + bench(2).
4. **Baseline parse log** — `audit/BASELINE_PARSE.log` (существующие 10 `.tri` через `t27c parse`, если доступен).
5. **Codegen plan** (`epic/CODEGEN_PLAN.md`) — какой пилот в какой target (zig/rust/c/verilog) и почему.
6. **Dataset seed** (`dataset/igla-coder/v0.1/`) — 20-40 (spec, gen) пар, MANIFEST.json с SHA-256, DECONTAM.md, README.md.
7. **Report** (`report/FINAL_REPORT.md`) — токсичный вердикт, benchmark vs baseline (n pilots, LOC, test count, invariants count, bench count), 3 варианта cooperation.
8. **Skills update** — `tt-lang-t27-integration` (v1.18 delta), `scientific-works-canon` (v2.1 delta), `task-status-board` (v0.13 delta), NEW **`trinity-ecosystem`** (user + space + org scopes).
9. **git push** — новая ветка `ring-105-ecosystem-tri-rewrite`, PR body links `Closes #<N>` для каждого связанного issue, gen-diff-report.txt приложен.

## What this Wave will NOT produce

- Полный ecosystem-wide rewrite всех 7 репо (нереально за один Wave, HR-A/B/H).
- TypeScript кодоген (deferred to Wave-3).
- Retro-fix для 20+ Wave-Loop PR-ов (out of scope, HR-C).
- Полный IGLA CODER v1.0 (нужно ~500-5000 pair-ов, HR-E).
- Merge to master (только PR — merge = user decision, HR-1 async-only).
- Никакого нарушения law priority (L1 > L2 > ... > L7).

---

**END OF WEAKNESS AUDIT.**
