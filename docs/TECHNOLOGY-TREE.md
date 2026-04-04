# Дерево технологий T27 — SWE Agent Sandbox

> **Дата:** 2026-04-04  
> **Текущее кольцо:** Ring 17 — CANOPY  
> **Статус:** PHI LOOP фаза 1 — Sandbox Infrastructure  

---

Это дерево технологий показывает путь эволюции T27 от базовой sandbox-инфраструктуры до автономного роя SWE-агентов. Каждый узел — это исследуемый или освоенный технологический элемент. Зависимости обозначены стрелками `←` (требует).

---

## Легенда

```
[✓] — Освоено / задеплоено
[~] — В разработке / текущий спринт
[ ] — Запланировано
[?] — Экспериментально / исследуется
[!] — Заблокировано (ожидает зависимости)
```

---

## RING 17: CANOPY — Текущее состояние

```
╔══════════════════════════════════════════════════════════════╗
║              RING 17: CANOPY                                 ║
║         Базовая инфраструктура агента                        ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  [✓] Control Plane API (Rust/Axum)                          ║
║       └─ REST API для управления сессиями                    ║
║       └─ Bearer token authentication                         ║
║       └─ PostgreSQL интеграция                               ║
║                                                              ║
║  [✓] PostgreSQL Session Store                                ║
║       └─ Таблица sessions (id, status, railway_ids, ...)     ║
║       └─ Таблица railway_accounts (pool)                     ║
║       └─ Таблица experience_episodes (PHI LOOP история)     ║
║                                                              ║
║  [✓] Railway API Client                                      ║
║       └─ GraphQL v2 client                                   ║
║       └─ serviceCreate / serviceDelete mutations             ║
║       └─ variableCollectionUpsert                            ║
║                                                              ║
║  [~] .tri Specification System (v0.1.0)                      ║
║       └─ sandbox.tri (текущий PHI LOOP)                      ║
║       └─ PHI LOOP engine (spec → gen → test → verdict)       ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```

---

## ФАЗА 1: Sandbox Infrastructure

*Цель: Надёжный, изолированный, быстрый запуск рабочих окружений.*

```
┌──────────────────────────────────────────────────────────────────┐
│  ФАЗА 1 — SANDBOX INFRASTRUCTURE                 [~] В процессе  │
│  Дедлайн: Sprint 1 (2026-04-11)                                  │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Узел 1.1: Railway Integration                           [~]     │
│  ──────────────────────────────────────────────────────────      │
│  Описание: Полный lifecycle management через Railway API         │
│  Зависимости: PostgreSQL, Railway аккаунты                       │
│  Файлы: backend/api/src/railway.rs                               │
│  Тесты: test_create_session, test_delete_session                 │
│                                                                  │
│    ├── 1.1.1 GraphQL Client Setup                        [✓]    │
│    │         Reqwest + serde для Railway GraphQL v2              │
│    │                                                             │
│    ├── 1.1.2 Service Lifecycle (create/delete/status)    [~]    │
│    │         serviceCreate, serviceDelete mutations              │
│    │         variableCollectionUpsert для env vars              │
│    │                                                             │
│    ├── 1.1.3 Multi-Account Pool                          [~]    │
│    │         Round-robin + least-connections                     │
│    │         Reconciliation loop (60s interval)                  │
│    │                                                             │
│    └── 1.1.4 Error Recovery & Retry                      [ ]    │
│              Exponential backoff (1s, 2s, 4s, 8s)              │
│              Dead-letter queue для failed deploys               │
│                                                                  │
│  Узел 1.2: Container Loader                              [~]    │
│  ──────────────────────────────────────────────────────────      │
│  Описание: Docker-образ с OpenCode и dev-инструментами          │
│  Зависимости: ghcr.io, OpenCode upstream                        │
│  Файлы: backend/sandbox/Dockerfile                               │
│                                                                  │
│    ├── 1.2.1 Base Image (Debian/Ubuntu slim)             [✓]    │
│    │         Node.js 22, Python 3.12, Go 1.23, Rust             │
│    │                                                             │
│    ├── 1.2.2 OpenCode Web Mode Integration               [~]    │
│    │         opencode --web --port 8080                          │
│    │         Валидация: /health endpoint                         │
│    │                                                             │
│    ├── 1.2.3 Git Clone Entrypoint                        [~]    │
│    │         start.sh: clone → checkout → opencode               │
│    │         Поддержка приватных репо (GH_TOKEN)                 │
│    │                                                             │
│    └── 1.2.4 Image Optimization                          [ ]    │
│              Multi-stage build                                   │
│              Layer caching (node_modules, cargo registry)       │
│              Целевой размер: < 2 GB                             │
│                                                                  │
│  Узел 1.3: Health Check Engine                          [~]     │
│  ──────────────────────────────────────────────────────────      │
│  Описание: Async поллинг статуса sandbox до готовности          │
│  Зависимости: 1.1 Railway Integration                           │
│  Константы: HEALTH_TIMEOUT_MS=3000, STARTUP_TIMEOUT_MS=90000    │
│                                                                  │
│    ├── 1.3.1 Async Polling Loop (Tokio)                  [~]    │
│    │         interval(5s) → HTTP GET /health                     │
│    │         Starting → Active при первом 200 OK                 │
│    │                                                             │
│    ├── 1.3.2 Timeout State Machine                       [~]    │
│    │         Starting → Failed при elapsed > 90s                 │
│    │         Уведомление: события через PostgreSQL NOTIFY        │
│    │                                                             │
│    └── 1.3.3 Health Check SSE Stream                    [ ]    │
│              GET /sessions/{id}/events                           │
│              Клиент подписывается на обновления статуса         │
│                                                                  │
│  Узел 1.4: HTTP Proxy Engine                            [~]     │
│  ──────────────────────────────────────────────────────────      │
│  Описание: Прозрачный прокси к *.railway.internal               │
│  Зависимости: 1.1.2 (session имя = DNS имя)                     │
│                                                                  │
│    ├── 1.4.1 Reverse Proxy (hyper/tower)                 [~]    │
│    │         /proxy/{name}/* → http://{name}.railway.internal:8080/*
│    │         Сохранение headers, method, body                    │
│    │                                                             │
│    ├── 1.4.2 WebSocket Proxy                             [ ]    │
│    │         Поддержка Upgrade: websocket                        │
│    │         Нужно для OpenCode terminal (xterm.js)              │
│    │                                                             │
│    └── 1.4.3 SSE Proxy                                   [ ]    │
│              Transfer-Encoding: chunked passthrough              │
│              Нужно для OpenCode streaming events                 │
│                                                                  │
│  Узел 1.5: sandbox.tri Specification                    [~]     │
│  ──────────────────────────────────────────────────────────      │
│  Описание: Формальная спецификация по T27 конституции           │
│  Зависимости: SOUL.md (Article II)                              │
│  Файлы: specs/sandbox/sandbox.tri                               │
│                                                                  │
│    ├── 1.5.1 Type Definitions                            [✓]    │
│    │         Session, SandboxConfig, RailwayAccount, Error      │
│    │                                                             │
│    ├── 1.5.2 Function Signatures                         [✓]    │
│    │         create_session, delete_session, check_health, ...  │
│    │                                                             │
│    ├── 1.5.3 Tests (14 тестов)                           [✓]    │
│    │         create, delete, health, proxy, load-balance         │
│    │                                                             │
│    ├── 1.5.4 Invariants (5 инвариантов)                 [✓]    │
│    │         bounded count, valid transitions, unique ids        │
│    │                                                             │
│    └── 1.5.5 Benchmarks (4 бенчмарка)                   [✓]    │
│              latency targets для создания и проверки            │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## ФАЗА 2: SWE Agent

*Цель: Полноценный автономный SWE-агент, способный решать реальные задачи.*

```
┌──────────────────────────────────────────────────────────────────┐
│  ФАЗА 2 — SWE AGENT                              [ ] Запланировано│
│  Дедлайн: Sprint 2-3 (2026-04-25)                               │
│  Зависит от: Фаза 1 полностью                                   │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Узел 2.1: OpenCode Deep Integration                    [ ]     │
│  ──────────────────────────────────────────────────────────      │
│  Описание: Прямое управление OpenCode из Control Plane          │
│                                                                  │
│    ├── 2.1.1 OpenCode REST API Client                    [ ]    │
│    │         POST /api/session — создать сессию чата            │
│    │         POST /api/session/{id}/message — отправить задачу  │
│    │                                                             │
│    ├── 2.1.2 Task Injection Protocol                     [ ]    │
│    │         Стандартный промпт: задача + контекст репо         │
│    │         Форматирование: Markdown с метаданными T27          │
│    │                                                             │
│    ├── 2.1.3 Output Streaming                            [ ]    │
│    │         SSE events: tool_call, message, completion          │
│    │         Хранение в experience episodes                      │
│    │                                                             │
│    └── 2.1.4 Completion Detection                        [ ]    │
│              Паттерн: "Task completed" / exit code               │
│              Таймаут: 30 минут на задачу                        │
│                                                                  │
│  Узел 2.2: Task Management System                       [ ]     │
│  ──────────────────────────────────────────────────────────      │
│  Описание: Очередь задач, приоритизация, отслеживание           │
│                                                                  │
│    ├── 2.2.1 Task Queue (PostgreSQL-backed)              [ ]    │
│    │         Таблица: tasks (id, issue_id, status, priority)    │
│    │         LISTEN/NOTIFY для немедленного запуска              │
│    │                                                             │
│    ├── 2.2.2 GitHub Issue Integration                    [ ]    │
│    │         Webhook: issues.opened → создать task               │
│    │         Комментарии: статус прогресса агента                │
│    │                                                             │
│    ├── 2.2.3 Task Prioritization                         [ ]    │
│    │         Оценка сложности по labels и описанию              │
│    │         Алгоритм: EDF (Earliest Deadline First)            │
│    │                                                             │
│    └── 2.2.4 Git PR Creation                             [ ]    │
│              По завершении задачи: git push + GitHub PR API     │
│              Автоматический review request                       │
│                                                                  │
│  Узел 2.3: Experience Recorder                          [ ]     │
│  ──────────────────────────────────────────────────────────      │
│  Описание: Запись и хранение PHI LOOP эпизодов                  │
│                                                                  │
│    ├── 2.3.1 Episode Schema v1.0                         [~]    │
│    │         sandbox-init.json (текущий эпизод)                 │
│    │         Поля: spec_hash, gen_hash, tests, verdict           │
│    │                                                             │
│    ├── 2.3.2 Automatic Episode Creation                  [ ]    │
│    │         Триггер: завершение PHI LOOP цикла                  │
│    │         Хеширование: SHA-256 spec и gen файлов             │
│    │                                                             │
│    ├── 2.3.3 Verdict Calculation                         [ ]    │
│    │         toxicity: доля failed tests                         │
│    │         score: f(bench_delta, test_pass_rate)               │
│    │                                                             │
│    └── 2.3.4 Experience Search                           [ ]    │
│              Семантический поиск по эпизодам (pgvector)         │
│              Агент находит релевантный прошлый опыт             │
│                                                                  │
│  Узел 2.4: Agent Evaluation Harness                     [ ]     │
│  ──────────────────────────────────────────────────────────      │
│  Описание: Автоматическая оценка качества решений агента        │
│                                                                  │
│    ├── 2.4.1 SWE-bench Integration                       [ ]    │
│    │         Запуск стандартных SWE-bench задач                  │
│    │         Метрика: % успешно решённых                         │
│    │                                                             │
│    ├── 2.4.2 Unit Test Executor                          [ ]    │
│    │         Запуск тестов репо после изменений агента           │
│    │         Интеграция с pytest, cargo test, jest               │
│    │                                                             │
│    └── 2.4.3 Regression Guard                            [ ]    │
│              Запрет merge если тесты стали хуже                 │
│              Интеграция с GitHub CI                              │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## ФАЗА 3: Swarm Intelligence

*Цель: Параллельная работа множества агентов, разделяющих знания.*

```
┌──────────────────────────────────────────────────────────────────┐
│  ФАЗА 3 — SWARM INTELLIGENCE                    [!] Заблокировано│
│  Дедлайн: Sprint 4-6 (2026-06)                                  │
│  Зависит от: Фаза 2 полностью                                   │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Узел 3.1: Multi-Agent Orchestration                    [!]     │
│  ──────────────────────────────────────────────────────────      │
│  Описание: Координация параллельных агентов                     │
│                                                                  │
│    ├── 3.1.1 Agent Registry                              [!]    │
│    │         Реестр активных агентов и их задач                  │
│    │         Обнаружение конфликтов (тот же файл/ветка)          │
│    │                                                             │
│    ├── 3.1.2 Work Decomposition                          [!]    │
│    │         Разбивка крупных задач на подзадачи                 │
│    │         Параллельное выполнение (разные файлы)              │
│    │                                                             │
│    ├── 3.1.3 Conflict Resolution                         [!]    │
│    │         Обнаружение merge conflicts заранее                 │
│    │         Сигнализация через CRDT-подобные стратегии          │
│    │                                                             │
│    └── 3.1.4 Result Aggregation                          [!]    │
│              Слияние PR от нескольких агентов                   │
│              Приоритет: качество score из эпизодов              │
│                                                                  │
│  Узел 3.2: Shared Experience Pool                       [!]     │
│  ──────────────────────────────────────────────────────────      │
│  Описание: Общая база знаний для всех агентов роя               │
│                                                                  │
│    ├── 3.2.1 Experience Vector Store (pgvector)          [!]    │
│    │         Embedding эпизодов для семантического поиска       │
│    │         Модель: text-embedding-3-small (OpenAI)             │
│    │                                                             │
│    ├── 3.2.2 Cross-Agent Knowledge Transfer              [!]    │
│    │         Агент A решил задачу X →                           │
│    │         Агент B читает эпизоды X при схожей задаче         │
│    │                                                             │
│    ├── 3.2.3 Skill Crystallization                       [!]    │
│    │         Повторяющиеся паттерны → именованные skills        │
│    │         Хранение в .trinity/skills/*.json                   │
│    │                                                             │
│    └── 3.2.4 Negative Experience Filter                  [!]    │
│              Токсичные эпизоды (verdict.toxicity > 0.5)         │
│              Помечаются и исключаются из поиска                 │
│                                                                  │
│  Узел 3.3: Swarm Monitoring Dashboard                   [!]     │
│  ──────────────────────────────────────────────────────────      │
│                                                                  │
│    ├── 3.3.1 Real-time Agent Status View                 [!]    │
│    │         Grafana / собственный UI                            │
│    │         Метрики: active agents, tasks/hour, success rate   │
│    │                                                             │
│    ├── 3.3.2 Experience Growth Chart                     [!]    │
│    │         Кумулятивный score по времени                       │
│    │         Показывает: учится ли рой?                          │
│    │                                                             │
│    └── 3.3.3 Cost Attribution                            [!]    │
│              Стоимость Railway per task                          │
│              ROI: tokens spent vs issues closed                  │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## ФАЗА 4: Evolution

*Цель: Самосовершенствующийся рой — агенты улучшают свои стратегии автоматически.*

```
┌──────────────────────────────────────────────────────────────────┐
│  ФАЗА 4 — EVOLUTION                             [?] Исследуется  │
│  Дедлайн: Sprint 7-10 (2026-09+)                                │
│  Зависит от: Фаза 3 + 500+ опыт-эпизодов в базе                │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Узел 4.1: ASHA Strategy Optimizer                      [?]     │
│  ──────────────────────────────────────────────────────────      │
│  Описание: Async Successive Halving для оптимизации промптов    │
│  Научная база: Li et al. "Massively Parallel Hyperparameter     │
│                Optimization" (ICML 2018)                         │
│                                                                  │
│    ├── 4.1.1 Prompt Parameter Space                      [?]    │
│    │         Параметры: temperature, max_tokens, system_prompt  │
│    │         Конфигурация: .trinity/strategy/params.json        │
│    │                                                             │
│    ├── 4.1.2 ASHA Scheduler                              [?]    │
│    │         Bracket-based successive halving                    │
│    │         Ранняя остановка неперспективных конфигураций      │
│    │                                                             │
│    ├── 4.1.3 Performance Signal                          [?]    │
│    │         Сигнал: verdict.score из эпизодов                   │
│    │         Нормализация: z-score по историческим данным        │
│    │                                                             │
│    └── 4.1.4 Strategy Checkpoint                         [?]    │
│              Лучшие конфигурации → .trinity/strategy/best.json  │
│              Автоматическое применение к новым агентам           │
│                                                                  │
│  Узел 4.2: PBT Agent Training                           [?]     │
│  ──────────────────────────────────────────────────────────      │
│  Описание: Population-Based Training для эволюции агентов       │
│  Научная база: Jaderberg et al. "Population Based Training      │
│                of Neural Networks" (DeepMind 2017)               │
│                                                                  │
│    ├── 4.2.1 Agent Population                            [?]    │
│    │         10-20 параллельных агентов с разными стратегиями   │
│    │         Каждый агент имеет свою конфигурацию промпта       │
│    │                                                             │
│    ├── 4.2.2 Exploit Step (копирование лучших)           [?]    │
│    │         Нижние 20% агентов копируют стратегию верхних 20%  │
│    │         Интервал: каждые 100 задач                          │
│    │                                                             │
│    ├── 4.2.3 Explore Step (мутация)                      [?]    │
│    │         Случайная мутация скопированной стратегии           │
│    │         Диапазон: ±20% от текущих параметров               │
│    │                                                             │
│    └── 4.2.4 Diversity Maintenance                       [?]    │
│              Принудительное сохранение разнообразия             │
│              Минимум: 3 различные стратегии в топ-20%           │
│                                                                  │
│  Узел 4.3: Predictive Agent S                           [?]     │
│  ──────────────────────────────────────────────────────────      │
│  Описание: Агент, предсказывающий результат до выполнения       │
│                                                                  │
│    ├── 4.3.1 Difficulty Predictor                        [?]    │
│    │         Вход: issue text, repo stats, history              │
│    │         Выход: оценка времени и вероятности успеха          │
│    │                                                             │
│    ├── 4.3.2 Resource Pre-Allocation                     [?]    │
│    │         Сложные задачи → более мощный sandbox               │
│    │         (2 vCPU → 4 vCPU, 2 GB → 8 GB)                    │
│    │                                                             │
│    ├── 4.3.3 Proactive Sandbox Warming                   [?]    │
│    │         Анализ PR queue → предзапуск sandbox               │
│    │         Цель: время ожидания пользователя → 0               │
│    │                                                             │
│    └── 4.3.4 Failure Anticipation                        [?]    │
│              Предсказание: "эта задача требует человека"         │
│              Escalation до начала выполнения                     │
│                                                                  │
│  Узел 4.4: Self-Modifying Specifications                [?]     │
│  ──────────────────────────────────────────────────────────      │
│  Описание: Агент улучшает собственные .tri спецификации         │
│  ⚠️  Требует: строгий sandbox (нет доступа к production)        │
│                                                                  │
│    ├── 4.4.1 Spec Mutation Engine                        [?]    │
│    │         Агент предлагает изменения к sandbox.tri           │
│    │         PHI LOOP валидирует: тесты должны зелёными          │
│    │                                                             │
│    ├── 4.4.2 Constitutional Guard                        [?]    │
│    │         SOUL.md инварианты нельзя нарушить                  │
│    │         Формальная верификация предложенных изменений       │
│    │                                                             │
│    └── 4.4.3 Spec Version Control                        [?]    │
│              Полная история spec эволюции в git                  │
│              Rollback при деградации качества                    │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## Карта зависимостей

```
Фаза 1 ─────────────────────────────────────────────────►
  1.1 Railway Integration
    │
    ├──► 1.2 Container Loader
    │         │
    │         └──► 1.3 Health Check Engine
    │                   │
    │                   └──► 1.4 HTTP Proxy Engine
    │
    └──► 1.5 .tri Specification System
              │
              ▼
Фаза 2 ─────────────────────────────────────────────────►
         2.1 OpenCode Integration ◄─── 1.4 Proxy Engine
              │
              ├──► 2.2 Task Management System
              │         │
              │         ├──► 2.3 Experience Recorder
              │         │         │
              │         │         └──► 2.4 Evaluation Harness
              │         │
              ▼         ▼
Фаза 3 ─────────────────────────────────────────────────►
         3.1 Multi-Agent Orchestration ◄─── 2.2, 2.3
              │
              ├──► 3.2 Shared Experience Pool ◄─── 2.3, 2.4
              │         │
              │         └──► 3.3 Swarm Dashboard
              │
              ▼
Фаза 4 ─────────────────────────────────────────────────►
         4.1 ASHA Optimizer ◄─── 3.2 Experience Pool
              │
              ├──► 4.2 PBT Training ◄─── 3.1 Orchestration
              │         │
              │         └──► 4.3 Predictive Agent S
              │
              └──► 4.4 Self-Modifying Specs ◄─── 1.5, 3.2
```

---

## Метрики прогресса по фазам

| Фаза | Ключевая метрика | Целевое значение | Текущее |
|---|---|---|---|
| 1: Infrastructure | Sandbox startup time | < 90 с | ~70 с (est.) |
| 1: Infrastructure | Health check latency | < 3000 мс | — |
| 2: SWE Agent | SWE-bench solve rate | > 20% | — |
| 2: SWE Agent | Tasks/day (1 агент) | > 10 | — |
| 3: Swarm | Параллельных агентов | 20 | — |
| 3: Swarm | Tasks/day (рой) | > 200 | — |
| 4: Evolution | Score growth/month | > 5% | — |
| 4: Evolution | Solve rate (PBT) | > 40% | — |

---

## История версий дерева

| Версия | Дата | Изменения |
|---|---|---|
| 0.1.0 | 2026-04-04 | Первичная версия: Фазы 1-4, Ring 17 CANOPY |

---

*Это дерево является живым документом. Обновляется при каждом завершённом PHI LOOP цикле.*  
*Следующее обновление: Sprint 1 Review (2026-04-11)*
