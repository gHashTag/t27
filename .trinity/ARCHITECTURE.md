# TRIOS ECOSYSTEM — Architecture

**Date:** 2026-04-19  
**Branch:** `feat/trios-migration-finalize`  
**Status:** DRAFT — Planning Document

---

## 1. Существующие Zig репозитории (LIVE)

| # | Модуль / Репо | Zig ver | Build | Статус | SSOT link README | Submodule |
|---|---|---|---|---|---|---|
| 1 | Golden Float / GF16 | `zig-golden-float` | 0.16 ✅ | ✅ | ✅ | [docs](https://github.com/gHashTag/zig-golden-float) |
| 2 | Physics / Quantum | `zig-physics` | 0.16 ✅ | ✅ | ✅ | [docs](https://github.com/gHashTag/zig-physics) |
| 3 | HDC / VSA | `zig-hdc` | 0.16 ✅ | ✅ | ✅ | — | — |
| 4 | Sacred Geometry | `zig-sacred-geometry` | ⚠️ NO VENDOR | Submodule not initialized | — |
| 5 | Crypto-mining | `zig-crypto-mining` | 0.16 ✅ | ✅ | ✅ | — | — |

**Примечание:** Sacred geometry (φ-attention) уже реализован в `zig-physics/src/gravity/sacred_geometry/`, отдельный Zig репозиторий не требуется.

---

## 2. Кандидаты на внедрение — HIGH priority

### 2.1 `trinity-fpga` — FPGA Synthesis

| Параметр | Статус | Описание |
|----------|----------|-------------|
| Источник | DRAFT | [t27](https://github.com/gHashTag/trinity-fpga) — VIBEE T27 synthesis для FPGA |
| Интеграция | PENDING | Интеграция с TRIOS server MCP tools |
| Зависимости | DRAFT | VIBEE Core, Triton FFGAs (external) |
| Сложность | HIGH | Требует глубокое знание FPGA synthesis |

**Этапы внедрения:**
1. Изучить VIBEE spec и существующую тринитую инфраструктуру
2. Создать слой интеграции в trios-server для управления FPGA
3. Разработать pipeline: CLI request → FPGA compilation → verification → deployment

---

### 2.2 `trinity-brain` — Neural Interface

| Параметр | Статус | Описание |
|----------|----------|-------------|
| Источник | DRAFT | [t27](https://github.com/gHashTag/trinity-brain) — Brain simulation interface |
| Интеграция | PENDING | Интеграция с trios-agents (AI agents) |
| Зависимости | DRAFT | VIBEE Core, Neural network libraries (external) |
| Сложность | MEDIUM | Мосты к существующим AI системам |

**Этапы внедрения:**
1. Изучить архитектуру trios-agents
2. Реализовать bidirectional communication agents ↔ FPGA/brain
3. Создать протоколы для hardware neural inference

---

## 3. Кандидаты на внедрение — MEDIUM priority

### 3.1 `trinity-bio` — Biological Computation

| Параметр | Статус | Описание |
|----------|----------|-------------|
| Источник | DRAFT | [t27](https://github.com/gHashTag/trinity-bio) — Bio-computation primitives |
| Интеграция | PENDING | Интеграция с zig-physics (quantum simulations) |
| Зависимости | DRAFT | VIBEE Core, zig-physics |
| Сложность | LOW | Специализированный домен, низкий приоритет |

**Этапы внедрения:**
1. Создать stub для bio-operations в trios-physics
2. Разработать протоколы для classical-quantum hybrid computing

---

### 3.2 `trinity-websocket` — WebSocket Layer

| Параметр | Статус | Описание |
|----------|----------|-------------|
| Источник | DRAFT | — WebSocket транспорт для TRIOS |
| Интеграция | PENDING | Интеграция в trios-server |
| Зависимости | DRAFT | trios-server |
| Сложность | LOW | Низкая сложность, чистая инфраструктура |

**Этапы внедрения:**
1. Создать `crates/trios-ws` crate
2. Реализовать WebSocket handlers для MCP
3. Тестирование производительности и безопасности

---

### 3.3 `trinity-rpc` — RPC Layer

| Параметр | Статус | Описание |
|----------|----------|-------------|
| Источник | DRAFT | — RPC протокол для trios |
| Интеграция | PENDING | Интеграция в trios-server |
| Зависимости | DRAFT | trios-server |
| Сложность | MEDIUM | Стандартный RPC, требуются интеграционные тесты |

**Этапы внедрения:**
1. Выбрать RPC фреймворк (json-rpc, tarpc)
2. Реализовать bidirectional RPC над MCP
3. Создать middleware для авторизации и rate limiting

---

## 4. Кандидаты на внедрение — LOW priority

### 4.1 `trinity-rpc` — I/O Layer

| Параметр | Статус | Описание |
|----------|----------|-------------|
| Источник | DRAFT | — Асинхронный I/O для TRIOS |
| Интеграция | PENDING | Интеграция с trios-server |
| Зависимости | DRAFT | trios-server, async runtime |
| Сложность | LOW | Может отложить до завершения RPC/WebSocket |

**Этапы внедрения:**
1. Изучить async I/O паттерны в trios-core
2. Реализовать файловые MCP операции (async read/write)
3. Тестирование производительности

---

### 4.2 `trinity-tests` — Testing Framework

| Параметр | Статус | Описание |
|----------|----------|-------------|
| Источник | DRAFT | — Unit и интеграционные тесты для TRIOS |
| Интеграция | PENDING | Интеграция в CI/CD |
| Зависимости | DRAFT | — |
| Сложность | LOW | Инфраструктурная задача |

**Этапы внедрения:**
1. Создать `crates/trios-test` crate
2. Добавить property-based тесты для всех модулей
3. Интеграция с GitHub Actions

---

### 4.3 `trinity-firebird` — Firebird Database

| Параметр | Статус | Описание |
|----------|----------|-------------|
| Источник | DRAFT | — SQL базы данных для TRIOS |
| Интеграция | PENDING | Интеграция через MCP (trios-server) |
| Зависимости | DRAFT | trios-server, Firebird driver |
| Сложность | LOW | Зависит от выбранной БД |

**Этапы внедрения:**
1. Выбрать SQL фреймворк (sqlx, libpq)
2. Создать MCP адаптер для базы данных
3. Реализовать миграции и backup

---

## 5. Кандидаты на вынос — НИЖНИЙ приоритет

### 5.1 `trinity-mcp` — Model Context Protocol

| Параметр | Статус | Описание |
|----------|----------|-------------|
| Источник | DRAFT | — MCP (Model Context Protocol) для TRIOS |
| Интеграция | — | Встроено в trios-server |
| Зависимости | DRAFT | — |
| Сложность | LOW | Уже реализован, не требует внедрения |

---

## 6. Технический долг — Technical Debt

| Область | Статус | Действие |
|----------|----------|-------------|
| FFI stubs | 404 | Все Zig FFI crate используют stub mode вместо реальных функций | Не критично, но снижает производительность |
| Sacred geometry в zig-physics | Внедрённая архитектура, требует рефакторинга | Текущая реализация работает |
| zig-sacred submodule 404 | Репозиторий не найден, sacred geometry уже в zig-physics | Не требует срочного решения |

---

## 7. Стратегия внедрения

### 7.1 Phased Approach

1. **Фаза 1 — Стабилизация FFI (CURRENT)**
   - Все 4 working Zig vendors green ✅
   - FFI feature flag добавлен и работает
   - Временные stub mode acceptable для ранних этапов

2. **Фаза 2 — Основной стек (NEXT)**
   - Выбор одного HIGH priority кандидата для MVP
   - Рекомендация: `trinity-fpga` или `trinity-brain`
   - Причины: 
     - FPGA synthesis — максимальный контроль над железом
     - Neural inference — критически важен для AGI applications
   - Создать выделенный воркспейс проекта для кандидата

3. **Фаза 3 — Расширение (FUTURE)**
   - Добавить MEDIUM приоритет кандидатов по одному
   - Создать интегрированную архитектуру (websocket, rpc, tests)

---

## 8. Приоритет действий сегодня

| # | Действие | Приоритет | Статус |
|---|---|----------|----------|
| 1 | Документация архитектуры | HIGH | ✅ СОХРАНЕНО |
| 2 | Разрешить zig-sacred geometry | LOW | ✅ ДОКУМЕНТИРОВАНО (в zig-physics) |
| 3 | Continue Zig 0.16 work | MEDIUM | ✅ НЕ ТРЕБУЕТСЯ |

---

**Примечание:** Документ ARCHITECTURE.md создан как основа для планирования и не должен содержать оперативные задачи. Для реализации конкретных кандидатов следует создавать отдельные файлы (например, `.trinity/ARCHITECTURE-FPGA.md` для trinity-fpga).

---

**Дата обновления:** 2026-04-19
