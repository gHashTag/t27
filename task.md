# RING 35 — FPGA Pipeline Restoration

## Фаза 1: Разблокировка (Blocker Fix)
- [x] 1.1 Обрезать мега-строки в `specs/fpga/mac.t27` до 65 символов.
- [x] 1.2 Собрать t27c (`cd bootstrap && cargo build --release`).
- [x] 1.3 Проверить: `./bootstrap/target/release/t27c parse specs/fpga/mac.t27`.

## Фаза 2: Генерация Verilog (MAC → Top Level)
- [x] 2.1 Сгенерировать Verilog для MAC.
- [x] 2.2 Написать `specs/fpga/uart.t27`.
- [x] 2.3 Написать `specs/fpga/top_level.t27`.
- [x] 2.4 Сгенерировать Verilog для новых модулей.

## Фаза 3: Build Pipeline
- [x] 3.1 Создать `scripts/fpga/build.sh` (Docker based).
- [x] 3.2 Создать `scripts/fpga/flash.sh`.
- [x] 3.3 Создать `scripts/fpga/Makefile`.

## Фаза 4: Constraints
- [x] 4.1 Создать `specs/fpga/constraints/qmtech_a100t.xdc`.

## Фаза 5: CI Integration
- [ ] 5.1 Добавить `globstar` в `tests/run_all.sh`.
- [ ] 5.2 Обновить GitHub Actions workflow.
