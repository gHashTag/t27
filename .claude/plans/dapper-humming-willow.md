# Plan: PHI LOOP Infrastructure Bootstrap

## Context

**Что сделано:**
- Созданы state-файлы (`.trinity/state/active-skill.json`, `issue-binding.json`, `episodes.jsonl`)
- Создан базовый парсер T27 (`specs/compiler/parser.t27`)
- Обновлён SKILL.md до v1.2.0

**Текущее состояние:**
- NO active skill (status: closed)
- NO issue binding
- Uncommitted изменения в инфраструктуре (state files, SKILL.md, parser spec)

---

## Phase 1: Текущий статус — завершение инфраструктуры

### Задача
Зафиксировать текущие uncommitted изменения как "infrastructure bootstrap" без открытия skill-сессии.

### Изменения
```
git add .trinity/state/active-skill.json
git add .trinity/state/issue-binding.json
git add .trinity/experience/episodes.jsonl
git add .claude/skills/tri/SKILL.md
git add specs/compiler/parser.t27
```

### Commit message
```
feat: bootstrap PHI LOOP state infrastructure

Parser spec created for T27 language.
- State files: active-skill, issue-binding, episodes.jsonl
- SKILL.md updated to v1.2.0
```

---

## Phase 2: Что дальше?

### Вариант A: CLI `tri` (полная реализация)
- Созда Bash/CLI скрипт `tri` который:
  - Чтает/записывает state-файлы
  - Запускает команды PHI LOOP (begin, gen, test, verdict, etc.)
  - Работает как настоящая обёртка над `.trinity/`

- **Плюсы:**
  - Прямой контроль за выполнением guard-ов
  - Можно интегрировать с git hooks
  - Автономный режим может работать с state machine

- **Минусы:**
  - Нужен полный кодгенератор (AST → Zig → C/Verilog)
  - Требуется runtime для execution

### Вариант B: Простейший путь (meta-work)
- Продолжать писать спецификации в Markdown (`PHI_LOOP_CONTRACT.md`, `PHI_LOOP_REGO.md`)
- Использовать `git commit` напрямую для инфраструктуры
- НЕ пытаться реализовать компилятор в ближайшее время

---

## Вопрос

Какой путь предпочитаем?

**1. Вариант A** — CLI `tri` (полная реализация)
   - Прямой контроль выполнения PHI LOOP
   - Guard-ы работают на уровне CLI, не документация

**2. Вариант B** — Спецификации и git commit
   - Продолжаем так как сейчас
   - Guard-ы описаны в документации, но enforcement только в коде

Выбор: `1` (CLI) или `2` (спецификации)
