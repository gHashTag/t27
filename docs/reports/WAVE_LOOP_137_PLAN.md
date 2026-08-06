# План выполнения Wave Loop 137

## Цель
Выполнить стандартный цикл расширения тестового покрытия и обновления конкурентной разведки в рамках IGLA CODER / IGLA RACE.

## Контекст

- Предыдущий луп **W136** коснулся спек: `systolic_array`, `systolic_ternary`, `ternary_mac`, `adder_tree`, `opcodes`, `yosys`, `backend`, `ternary_gemm`.
- Согласно правилу **ротационного исключения**, W137 работает с **противоположным пулом**: `rtl`, `eda`, `cordic_fixed`, `bram_weights`, `cordic`, `cordic_top`, `formal`, `gemm`.
- Все эти спеки имеют 21–22 теста и являются наиболее слабыми.

## Этапы выполнения

### 1. Аудит слабых мест (завершён)
- Подсчёт `test`/`bench` в `specs/igla/**/*.t27`:
  - `rtl.t27`: 21
  - `bram_weights.t27`: 22
  - `cordic.t27`: 22
  - `cordic_fixed.t27`: 22
  - `cordic_top.t27`: 22
  - `eda.t27`: 22
  - `formal.t27`: 22
  - `gemm.t27`: 22
- Выбраны 8 целевых спек для W137 (пул, исключённый в W136).

### 2. Исследование конкурентов (завершено)
- Анализ `COMPETITIVE_POSITIONING.md` и текущего `benchmark.t27`.
- Выявлены 2 документированных, но ещё не закодированных конкурента:
  1. **Myo Oo** (Zenodo, June 2026) — HIGH. 11 физических констант из геометрии границы E₈.
  2. **Alvarez, Izaurieta & Quinzacara** (arXiv:2601.19734, Feb 2026) — HIGH. Единый лагранжиан гравитации + Янга-Миллса + фермионы через Clifford-алгебру.

### 3. Декомпозиция задач
- Добавить по **2 теста** в каждую из 8 спек (итого **+16 тестов**).
- Добавить **2 функции конкурентов** и соответствующие тесты в `benchmark.t27`.
- Обновить дату в `COMPETITIVE_POSITIONING.md` на W137/147.

### 4. Реализация
| Спека | Тест 1 | Тест 2 |
|-------|--------|--------|
| `rtl.t27` | `rtl_mux_4to1_behavior` | `rtl_posedge_detector_pulse` |
| `eda.t27` | `eda_elapsed_time_zero` | `eda_tool_version_parse` |
| `cordic_fixed.t27` | `cordic_fixed_iter_12` | `cordic_fixed_identity_angle` |
| `bram_weights.t27` | `bram_weights_zero_bias` | `bram_weights_negative_index_guard` |
| `cordic.t27` | `cordic_rot_mode_identity` | `cordic_vec_mode_zero_y` |
| `cordic_top.t27` | `cordic_top_pipeline_depth` | `cordic_top_ready_idle` |
| `formal.t27` | `formal_assert_true_always` | `formal_cover_counter_max` |
| `gemm.t27` | `gemm_1x1_scalar` | `gemm_transpose_equiv` |

### 5. Верификация
- Перегенерировать `seal` для всех 9 затронутых файлов.
- Запустить `./scripts/tri suite --repo-root .`.
- Убедиться в отсутствии регрессий (**ожидаемый результат: 565/565 PASS → 566/566?** Нет, 565 + 0 регрессий = 565. Количество тестов увеличится, но прогон всё равно должен быть PASS).

### 6. Завершение
- Создать `WAVE_LOOP_137_REPORT.md`.
- Сформировать `WAVE_LOOP_137_COOPERATION.md` (3 варианта).
- Зафиксировать изменения коммитом с `Closes #1061`.
- Обновить память проекта (`wave-loop-137.md`).

## Критерии приёмки
- [ ] 16 новых тестов добавлены в 8 спек.
- [ ] 2 новых конкурента интегрированы в `benchmark.t27`.
- [ ] Все seal совпадают.
- [ ] Suite проходит без ошибок (PASS).
- [ ] Отчёт и варианты сотрудничества созданы.
