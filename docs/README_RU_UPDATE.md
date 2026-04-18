## Документация на русском языке

Документация на русском языке для Trinity S³AI / t27.

## Быстрый старт

```bash
# Клонирование
git clone https://github.com/gHashTag/t27.git
cd t27

# Парсинг спецификации (канонический CLI)
./scripts/tri parse specs/base/types.t27

# Генерация Zig кода
./scripts/tri gen-zig specs/numeric/gf16.t27

# Или Verilog для FPGA
./scripts/tri gen-verilog specs/fpga/mac.t27

# Или C код
./scripts/tri gen-c specs/base/ops.t27

# Верификация seal
./scripts/tri seal specs/numeric/gf16.t27 --verify

# Запуск тестов
./scripts/tri test
```

## Архитектура проекта

Проект организован в 5 колец (Rings) с эволюционным расширением:

```
STRAND I   - Base         : типы, операции, константы          (Rings 0-8)
STRAND II  - Numeric+VSA : GF4-GF32, TF3, φ, VSA ops     (Rings 9-11)
STRAND III - Compiler+FPGA : парсер, MAC, ISA регистры        (Rings 12-14)
STRAND IV  - Queen+NN     : оркестрация, HSLM, внимание      (Rings 14-17)
STRAND V   - AR (CLARA)     : логика, доказательства, datalog, RESTRAINT, VSA, FPGA, нейросети (Rings 18-24)
```

## Основные компоненты

### 📚 [`specs/`](../specs/) — Технические спецификации

Директория `specs/` содержит спецификации на формате `.t27` (технический английский).

| Раздел | Описание | Файлы |
|--------|---------|----------|
| `base/` | Базовые типы и операции (2 спецификации) | `types.t27`, `ops.t27` |
| `numeric/` | Численные форматы (10 спецификаций) | `gf16.t27` (GoldenFloat), `tf3.t27` (TensorFlow), `phi.t27` |
| `math/` | Священные константы (2 спецификации) | `sacred_physics.t27` |
| `ar/` | Ternary логика и AR pipeline (7 спецификаций) | `ternary_logic.t27`, `datalog_engine.t27`, `proof_trace.t27`, `restraint.t27`, `explainability.t27`, `asp_solver.t27`, `composition.t27` |
| `isa/` | Набор инструкций (1 спецификация) | `registers.t27` |
| `fpga/` | FPGA модуль (1 спецификация) | `mac.t27` |
| `queen/` | Оркестрация Queen (1 спецификация) | `lotus.t27` |
| `vsa/` | Vector Symbolic Architecture (1 спецификация) | `vector_symbolic.t27` |
| `compiler/` | Компилятор (15 спецификаций) | Содержит lexer, parser, codegen и др. |
| `benchmarks/` | Бенчмарки (много спецификаций) | `trinity_cognitive_probe_runner.t27` и др. |

### 📖 [`compiler/`](../compiler/) — Компилятор t27

Компилятор на Rust, который генерирует Zig, Verilog и C из `.t27` спецификаций.

| Раздел | Описание |
|--------|---------|
| `lexer/` | Лексер для `.t27` | `lexer.t27` |
| `parser/` | Парсер `.t27` → AST | `parser.t27` |
| `codegen/` | Генерация Zig, Verilog, C | `zig/`, `verilog/`, `c/` |
| `cli/` | CLI интерфейс | Содержит команды parse, gen, gen-zig, gen-verilog, gen-c, seal, test, check-now |

### 📖 [`bootstrap/`](../bootstrap/) — Stage-0 компилятор

Компилятор на Rust (stage-0).

| Команды | Описание |
|--------|---------|
| `cargo build` | Сборка Release компилятора |
| `cargo build --release` | Сборка Production компилятора |

### 📖 [`tests/`](../tests/) — Тесты

Тесты для проверки спецификаций и конформности.

| Тип | Описание |
|--------|---------|
| Unit tests | Специальные тесты в `.t27` спецификациях (`test {}` блок) |
| Conformance tests | В `conformance/` — JSON векторы для проверка backend |
| Integration tests | Полные тесты компилятора |

### 📖 [`scripts/`](../scripts/) — CLI скрипты

CLI утилиты `./scripts/tri/` для управления проектом.

| Скрипт | Описание |
|--------|---------|
| `parse` | Парсинг `.t27` спецификаций |
| `gen-zig` | Генерация Zig кода |
| `gen-verilog` | Генерация Verilog для FPGA |
| `gen-c` | Генерация C кода |
| `seal` | Верификация и сохранение seal (SHA-256) |
| `test` | Запуск всех тестов |
| `check-now` | Проверка актуальности проекта |

### 🌐 [Английский README](../README.md) — Полная документация

Полная документация проекта на английском языке. Содержит:
- Архитектуру (5 колец RINGS 0-30)
- Описание всех компонентов
- PHI LOOP workflow
- Текущий статус разработок

### 🌐 [Русский README](README_RU.md) — Основной файл (этот файл)

Основной файл репозитория на русском языке с ссылками на английскую документацию. Содержит:
- Быстрый старт
- Структуру проекта
- Основные компоненты
- Связи с другими проектами

---

**Последнее обновление:** 2026-04-16
