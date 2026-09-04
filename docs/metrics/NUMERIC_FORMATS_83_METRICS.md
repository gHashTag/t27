# Ключевые метрики 83 числовых форматов каталога Trinity (t27)

Сводка по единому источнику истины (SSOT) репозитория `gHashTag/t27`, ветка `master`.

Дата сборки: 28.06.2026. Источники данных:
- Метрики (биты, S/E/M, bias, кластер, статус, назначение): `gen/numeric/formats_catalog.json` (77 форматов) + `specs/numeric/gf{10,14,48,96,512,1024}.t27` (6 GF-форматов, отсутствующих в JSON-каталоге).

> **ИСТОЧНИК НЕДОСТУПЕН, И УДАЛЁН ИМЕННО ЗА ЭТУ ЦИФРУ.**
> `gen/numeric/formats_catalog.json` убран коммитом `aa01dd4f1` —
> *"fix(gen): untrack stale gen/numeric catalog artifacts (**drift 77 vs SSOT
> 83**) (Closes #1120)"*. То есть файл выкинули за расхождение 77 против 83, а
> этот документ до сих пор ссылается на него как на источник и печатает **77**.
> `gen/` в `.gitignore`, файл не восстанавливается из рабочего дерева. Цифру 77 нельзя
> перевзять по названному источнику. Первоисточник, который МОЖНО перевзять без
> сборки, — сам каталог: `grep -c 'CATALOG:' specs/numeric/formats_catalog.t27`
> даёт **109** записей сегодня (83 на момент W602, 92 после `08adcc39f`, 109
> после `b92872507`). Таблица ниже — снимок 28.06.2026 и читается как запись, а
> не как текущее состояние.
- SW-conformance (kind, n_vectors): `conformance/vectors/INDEX_all_formats.json`.
- HW-статус: реальные прогоны на плате AX7203 (XC7A200T) в сессии 28.06.2026.
- Якорь тождества: φ² + φ⁻² = 3. Препринт: arXiv:2606.05017.

> Производные величины (значащие десятичные цифры, динамический диапазон) посчитаны по IEEE-подобной модели нормальных чисел и являются [смоделировано: оценка], а не нормативными значениями из спецификации.

---

## 1. Сводка по столбцам (честный счёт)

| Уровень доказательства | Счёт | Тег |
|---|---|---|
| **SW-bitexact** (независимый оракул, abs_error=0) | **62 / 83** | [смоделировано / verified в SW] |
| SW self-consistent (encode↔decode roundtrip, без 2-го witness) | 6 / 83 | [слабее: требует внешнего оракула] |
| SW structural (нет числовых векторов) | 15 / 83 | [открытая задача] |
| **decode-HW** (bit-exact на AX7203) | **4 / 83** — `bfloat16`, `int8`, `nf4`, `fp8_e4m3` | [измерено на железе; 3 из 4 exhaustive] |
| **compute-HW** (bit-exact на AX7203) | **2 / 83** — `gf6`, `gf8` (ADD) | [измерено на железе: 512/512 bit-exact каждый] |

Всего SW-векторов в каталоге: **2493**. Всего форматов: **83** (22 GoldenFloat + 61 внешних/референсных). Итого HW-ячеек за сессию 28.06.2026: **6** (0→6).

> ⚠️ encoding ≠ compute ≠ FPGA. SW-bitexact = модель сравнивается с моделью. HW-ячейка = реальные LUT/DSP на кристалле Artix-7 выдали бит-в-бит ожидаемое. Это разные уровни доказательства — не смешивать.
>
> ⚠️ **Два тира доказательств HW** (в таблице помечены): **E** = полная цепь доказательств опубликована на #199 (CI run + SHA + flash/UART-лог): `gf8`, `gf6`, `bfloat16`. **C** = self-report локального агента + дизайн на main, но UART-лог ещё НЕ опубликован на #199: `int8`, `nf4`, `fp8_e4m3`. Рекомендация: попросить локального агента отправить UART-логи этих трёх на #199 для перевода C→E.
>
> ⚠️ Покрытие: `int8`/`nf4`/`fp8_e4m3` — **exhaustive** (256/256, 16/16, 256/256, весь код-пространство); `bfloat16` — 8 corner-кодов; `gf8`/`gf6` compute — 512 векторов §3.5 (репрезентативно, НЕ exhaustive 65536).

---

## 2. Приоритизация следующих HW-ячеек (по «доказательному весу»)

Критерий: широта применения (актуальность для ML/индустрии) × простота переноса (малые биты, готовый RTL-декодер/clean-ядро, уже пройден 2-oracle SW). Шаблон переноса проверен и воспроизводим: single-decoder/compute-дизайн → CI synth → `gh api .../artifacts/{id}/zip` → openocd flash (AL321, IDCODE 0x13636093) → UART verify (CP2102N @160000).

Статус на конец сессии: decode-HW уже 4/83 (bf16+int8+nf4+fp8), compute-HW 2/83 (gf6+gf8). Ниже — следующие кандидаты.

### Ближайший шаг (уже почти готово)

| Приоритет | Столбец | Формат | Состояние |
|---|---|---|---|
| 1 | decode-HW | `posit8` | bitstream готов, скачан; прошивка зависла после 6 быстрых циклов JTAG → нужен reset/power-cycle платы → `--fmt 4` → decode-HW 4→5 |

### decode-HW: следующие после posit8

| Приоритет | Формат | Бит | Почему высокий вес |
|---|---|--:|---|
| 1 | `fp8_e5m2` | 8 | Второй FP8-вариант (широкий диапазон, activations) — парный к fp8_e4m3 |
| 2 | `fp4_e2m1` | 4 | Extreme-quant инференс; самый узкий → простой exhaustive (16 кодов) |
| 3 | `mxfp4` / `mxfp8` | 4/8 | Microscaling (ally) — OCP-стандарт, растёт в индустрии |

### compute-HW: 3 следующих ячейки

| Приоритет | Формат | Бит | Состояние / почему |
|---|---|--:|---|
| 1 | `gf16` | 16 | Production-ядро Trinity; clean-дизайн на pre-fix коммите 5d572ccdd — нужен re-trigger CI |
| 2 | `gf12` | 12 | Mid-range; clean-дизайн/workflow НЕ существует — надо создать `gf12_clean_ax7203.v` |
| 3 | `gf4` | 4 | Вырожденный край (bias=0) — отдельное ядро; exhaustive 256 кодов тривиален |

**Рекомендация:** (1) сначала добить **posit8** (reset платы → decode-HW 5/83) — битстрим уже готов; (2) параллельно попросить локального агента опубликовать UART-логи int8/nf4/fp8 на #199 (C→E); (3) затем **gf16 compute** (re-trigger CI) — флагманский GF-формат даёт максимум веса.

---

## 3. Полная таблица метрик (83 формата, по кластерам)

Колонки: `Бит` — разрядность; `S/E/M` — знак/экспонента/мантисса; `Знач. цифр` — оценка десятичной точности [смоделировано]; `Дин. диап. (дек)` — десятичные порядки нормальных чисел [смоделировано]; `φ-расст.` — близость к φ-выравниванию (из каталога); `SW-conf.` — уровень софт-conformance из SSOT; `n_vec` — число conformance-векторов; `decode-HW`/`compute-HW` — реально измерено на AX7203.

### GoldenFloat (Trinity) (22)

| Формат | Бит | S/E/M | Bias | Знач. цифр | Дин. диап. (дек) | φ-расст. | SW-conf. | n_vec | decode-HW | compute-HW | Назначение |
|---|--:|---|--:|--:|--:|--:|---|--:|---|---|---|
| `gfternary` | 2 | 1/0/2 | 0 | 0.9 | — | 0.0 | bitexact | 4 | 0 | 0 | bulk layers (hybrid) |
| `gf4` | 4 | 1/1/2 | 0 | 0.9 | -0.1 | 0.118 | bitexact | 16 | 0 | 0 | proof-of-concept |
| `mxgf4` | 4 | 1/1/2 | 0 | 0.9 | -0.1 | 0.118 | bitexact | 16 | 0 | 0 | OPEN R&D: phi-aligned MX-4 candidate |
| `gf6` | 6 | 1/2/3 | 1 | 1.2 | 0.6 | 0.05 | bitexact | 64 | 0 | **1**  [измерено: 512/512 bit-exact; E:#199 artifact 7931202948] | OPEN R&D: bridge GF4-GF8; FP6 E2M3 hint |
| `mxgf6` | 6 | 1/2/3 | 1 | 1.2 | 0.6 | 0.05 | bitexact | 64 | 0 | 0 | OPEN R&D: phi-aligned MX-6 candidate |
| `gf8` | 8 | 1/3/4 | 3 | 1.5 | 1.8 | 0.132 | bitexact | 256 | 0 | **1**  [измерено: 512/512 bit-exact; E:#199 post-fix c0d24cac2] | edge / sensors |
| `gf8_bfp` | 8 | 1/3/4 | 3 | 1.5 | 1.8 | 0.132 | bitexact | 256 | 0 | 0 | OPEN R&D: LLM-quantization-friendly GF8 |
| `gf10` | 10 | 1/3/6 | 3 | 2.1 | 1.8 | — | bitexact | 8 | 0 | 0 | расширенный диапазон GF-семейства |
| `gf12` | 12 | 1/4/7 | 7 | 2.4 | 4.2 | 0.047 | bitexact | 8 | 0 | 0 | mid-range / audio |
| `gf14` | 14 | 1/5/8 | 15 | 2.7 | 9.0 | — | self-cons. | — | 0 | 0 | расширенный диапазон GF-семейства |
| `gf16` | 16 | 1/6/9 | 31 | 3.0 | 18.7 | 0.049 | bitexact | — | 0 | 0 | training and inference (production) |
| `gf_lns_hybrid` | 16 | 1/6/9 | 31 | 3.0 | 18.7 | 0.049 | bitexact | 8 | 0 | 0 | OPEN R&D: dual-space arithmetic |
| `gf20` | 20 | 1/7/12 | 63 | 3.9 | 37.9 | 0.035 | bitexact | 8 | 0 | 0 | high-precision edge |
| `gf24` | 24 | 1/9/14 | 255 | 4.5 | 153.5 | 0.025 | bitexact | 8 | 0 | 0 | server inference |
| `gf32` | 32 | 1/12/19 | 2047 | 6.0 | 1232.1 | 0.014 | bitexact | 8 | 0 | 0 | fp32 drop-in |
| `gf48` | 48 | 1/18/29 | 131071 | 9.0 | 78912.3 | — | self-cons. | — | 0 | 0 | расширенный диапазон GF-семейства |
| `gf64` | 64 | 1/24/39 | 8388607 | 12.0 | 5050444.4 | 0.003 | bitexact | 8 | 0 | 0 | scientific / double |
| `gf96` | 96 | 1/36/59 | 34359738367 | 18.1 | 20686623783.0 | — | self-cons. | — | 0 | 0 | расширенный диапазон GF-семейства |
| `gf128` | 128 | 1/48/79 | 0 | 24.1 | 84732411018727.1 | 0.008 | self-cons. | — | 0 | 0 | OPEN R&D: phi-aligned binary128 alternative |
| `gf256` | 256 | 1/97/158 | 0 | 47.9 | 4.770010683626838e+28 | 0.005 | structural | 0 | 0 | 0 | OPEN R&D: phi-aligned binary256 alternative |
| `gf512` | 512 | 1/195/316 | 25108406941546723055343157692830665664409421777856138051583 | 95.4 | 1.511676726548657e+58 | — | self-cons. | — | 0 | 0 | расширенный диапазон GF-семейства |
| `gf1024` | 1024 | 1/391/632 | 2521728396569246669585858566409191283525103313309788586748690777871726193375821479130513040312634601011624191379636223 | 190.6 | 1.5182317765699572e+117 | — | self-cons. | — | 0 | 0 | расширенный диапазон GF-семейства |

### IEEE-754 binary (5)

| Формат | Бит | S/E/M | Bias | Знач. цифр | Дин. диап. (дек) | φ-расст. | SW-conf. | n_vec | decode-HW | compute-HW | Назначение |
|---|--:|---|--:|--:|--:|--:|---|--:|---|---|---|
| `binary16` | 16 | 1/5/10 | 15 | 3.3 | 9.0 | 0.118 | bitexact | 8 | 0 | 0 | GPU activations, inference |
| `binary32` | 32 | 1/8/23 | 127 | 7.2 | 76.5 | 0.27 | bitexact | 8 | 0 | 0 | industry default |
| `binary64` | 64 | 1/11/52 | 1023 | 16.0 | 615.9 | 0.406 | bitexact | 8 | 0 | 0 | scientific computing |
| `binary128` | 128 | 1/15/112 | 16383 | 34.0 | 9863.2 | 0.484 | bitexact | 8 | 0 | 0 | high-precision simulations |
| `binary256` | 256 | 1/19/236 | 262143 | 71.3 | 157825.5 | 0.538 | bitexact | 8 | 0 | 0 | astronomy, cryptography |

### ML low-precision (7)

| Формат | Бит | S/E/M | Bias | Знач. цифр | Дин. диап. (дек) | φ-расст. | SW-conf. | n_vec | decode-HW | compute-HW | Назначение |
|---|--:|---|--:|--:|--:|--:|---|--:|---|---|---|
| `fp4_e2m1` | 4 | 1/2/1 | 1 | 0.6 | 0.5 | 1.382 | bitexact | 16 | 0 | 0 | extreme quant inference |
| `fp6_e2m3` | 6 | 1/2/3 | 1 | 1.2 | 0.6 | 0.049 | bitexact | 64 | 0 | 0 | mantissa-heavy quant |
| `fp6_e3m2` | 6 | 1/3/2 | 3 | 0.9 | 1.7 | 0.882 | bitexact | 64 | 0 | 0 | aggressive quant inference |
| `fp8_e4m3` | 8 | 1/4/3 | 7 | 1.2 | 4.2 | 0.715 | bitexact | — | **1**  [измерено: 256/256 exhaustive; C:design on main, log≠#199] | 0 | inference, gradient ranges |
| `fp8_e5m2` | 8 | 1/5/2 | 15 | 0.9 | 9.0 | 1.882 | bitexact | — | 0 | 0 | activations, wide range |
| `bfloat16` | 16 | 1/8/7 | 127 | 2.4 | 76.5 | 0.525 | bitexact | — | **1**  [измерено: 8/8 corner; E:#199 run 28326217079] | 0 | training (range > precision) |
| `tf32` | 19 | 1/8/10 | 127 | 3.3 | 76.5 | 0.27 | bitexact | 8 | 0 | 0 | A100/H100 mixed precision |

### Microscaling (MX) (3)

| Формат | Бит | S/E/M | Bias | Знач. цифр | Дин. диап. (дек) | φ-расст. | SW-conf. | n_vec | decode-HW | compute-HW | Назначение |
|---|--:|---|--:|--:|--:|--:|---|--:|---|---|---|
| `mxfp4` | 4 | 1/2/1 | 1 | 0.6 | 0.5 | 1.382 | bitexact | — | 0 | 0 | extreme quant |
| `mxfp6` | 6 | 1/3/2 | 3 | 0.9 | 1.7 | 0.882 | bitexact | 64 | 0 | 0 | aggressive inference |
| `mxfp8` | 8 | 1/4/3 | 7 | 1.2 | 4.2 | 0.715 | bitexact | 256 | 0 | 0 | LLM inference |

### Квантование (tuned) (2)

| Формат | Бит | S/E/M | Bias | Знач. цифр | Дин. диап. (дек) | φ-расст. | SW-conf. | n_vec | decode-HW | compute-HW | Назначение |
|---|--:|---|--:|--:|--:|--:|---|--:|---|---|---|
| `nf4` | 4 | 0/0/4 | 0 | 1.5 | — | -1.0 | bitexact | 16 | **1**  [измерено: 16/16 exhaustive; C:design on main, log≠#199] | 0 | LLM weight quantization (quantile-based on N(0 |
| `afp` | 16 | 1/8/7 | 127 | 2.4 | 76.5 | -1.0 | structural | 0 | 0 | 0 | efficient training |

### Posit / Unum III (8)

| Формат | Бит | S/E/M | Bias | Знач. цифр | Дин. диап. (дек) | φ-расст. | SW-conf. | n_vec | decode-HW | compute-HW | Назначение |
|---|--:|---|--:|--:|--:|--:|---|--:|---|---|---|
| `posit8` | 8 | 1/2/0 | 0 | 0.3 | 0.3 | -1.0 | bitexact | 256 | 0 | 0 | inference |
| `takum8` | 8 | 1/0/0 | 0 | 0.3 | — | -1.0 | bitexact | 256 | 0 | 0 | IEEE-754 backward-compatible tapered |
| `posit16` | 16 | 1/2/0 | 0 | 0.3 | 0.3 | -1.0 | bitexact | 8 | 0 | 0 | mixed-precision training |
| `takum16` | 16 | 1/0/0 | 0 | 0.3 | — | -1.0 | structural | 0 | 0 | 0 | single-rule ladder counterexample |
| `posit32` | 32 | 1/2/0 | 0 | 0.3 | 0.3 | -1.0 | bitexact | 8 | 0 | 0 | f32 replacement |
| `takum32` | 32 | 1/0/0 | 0 | 0.3 | — | -1.0 | structural | 0 | 0 | 0 | tapered fp32-class |
| `posit64` | 64 | 1/2/0 | 0 | 0.3 | 0.3 | -1.0 | bitexact | 8 | 0 | 0 | f64 replacement |
| `takum64` | 64 | 1/0/0 | 0 | 0.3 | — | -1.0 | structural | 0 | 0 | 0 | tapered fp64-class |

### Логарифмические (LNS) (4)

| Формат | Бит | S/E/M | Bias | Знач. цифр | Дин. диап. (дек) | φ-расст. | SW-conf. | n_vec | decode-HW | compute-HW | Назначение |
|---|--:|---|--:|--:|--:|--:|---|--:|---|---|---|
| `lns8` | 8 | 1/7/0 | 0 | 0.3 | 37.6 | -1.0 | bitexact | 256 | 0 | 0 | DSP, signal processing |
| `lns16` | 16 | 1/15/0 | 0 | 0.3 | 9863.2 | -1.0 | bitexact | 5 | 0 | 0 | log-domain training (mul -> add) |
| `lns32` | 32 | 1/31/0 | 0 | 0.3 | 646456992.3 | -1.0 | bitexact | 5 | 0 | 0 | log-domain DSP |
| `lns64` | 64 | 1/63/0 | 0 | 0.3 | 2.7765116442616786e+18 | -1.0 | bitexact | 5 | 0 | 0 | scientific log-domain |

### Расширенные float (3)

| Формат | Бит | S/E/M | Bias | Знач. цифр | Дин. диап. (дек) | φ-расст. | SW-conf. | n_vec | decode-HW | compute-HW | Назначение |
|---|--:|---|--:|--:|--:|--:|---|--:|---|---|---|
| `x87_fp80` | 80 | 1/15/64 | 16383 | 19.6 | 9863.2 | -1.0 | bitexact | 8 | 0 | 0 | legacy long double on x86 |
| `double_double` | 128 | 2/22/104 | 0 | 31.6 | 1262610.4 | -1.0 | bitexact | 8 | 0 | 0 | software extended precision |
| `quad_double` | 256 | 4/44/208 | 0 | 62.9 | 5295775688669.6 | -1.0 | bitexact | 8 | 0 | 0 | astrophysics, quad-precision sims |

### IEEE-754 decimal (3)

| Формат | Бит | S/E/M | Bias | Знач. цифр | Дин. диап. (дек) | φ-расст. | SW-conf. | n_vec | decode-HW | compute-HW | Назначение |
|---|--:|---|--:|--:|--:|--:|---|--:|---|---|---|
| `decimal32` | 32 | 1/11/20 | 101 | 6.3 | 615.6 | -1.0 | bitexact | 7 | 0 | 0 | banking, GAAP |
| `decimal64` | 64 | 1/13/50 | 398 | 15.4 | 2465.1 | -1.0 | bitexact | 7 | 0 | 0 | financial databases |
| `decimal128` | 128 | 1/17/110 | 6176 | 33.4 | 39455.7 | -1.0 | bitexact | 8 | 0 | 0 | audit ledgers |

### Integer / fixed-point (8)

| Формат | Бит | S/E/M | Bias | Знач. цифр | Дин. диап. (дек) | φ-расст. | SW-conf. | n_vec | decode-HW | compute-HW | Назначение |
|---|--:|---|--:|--:|--:|--:|---|--:|---|---|---|
| `bcd` | 0 | 0/0/0 | 0 | 0.3 | — | -1.0 | structural | 0 | 0 | 0 | calculators, GAAP |
| `q_format` | 0 | 1/0/0 | 0 | 0.3 | — | -1.0 | structural | 0 | 0 | 0 | audio DSP, fixed-point ML |
| `int4` | 4 | 1/0/3 | 0 | 1.2 | — | -1.0 | bitexact | 16 | 0 | 0 | aggressive quantization |
| `int8` | 8 | 1/0/7 | 0 | 2.4 | — | -1.0 | bitexact | 256 | **1**  [измерено: 256/256 exhaustive; C:design on main, log≠#199] | 0 | INT8 inference, per-channel scale |
| `int16` | 16 | 1/0/15 | 0 | 4.8 | — | -1.0 | bitexact | 7 | 0 | 0 | DSP, embedded ML |
| `int32` | 32 | 1/0/31 | 0 | 9.6 | — | -1.0 | bitexact | 7 | 0 | 0 | general CPU integer |
| `int64` | 64 | 1/0/63 | 0 | 19.3 | — | -1.0 | bitexact | 7 | 0 | 0 | databases, timestamps |
| `int128` | 128 | 1/0/127 | 0 | 38.5 | — | -1.0 | bitexact | 7 | 0 | 0 | crypto, big-int |

### Исторические/вендорные (10)

| Формат | Бит | S/E/M | Bias | Знач. цифр | Дин. диап. (дек) | φ-расст. | SW-conf. | n_vec | decode-HW | compute-HW | Назначение |
|---|--:|---|--:|--:|--:|--:|---|--:|---|---|---|
| `ibm_hfp32` | 32 | 1/7/24 | 64 | 7.5 | 37.9 | -1.0 | bitexact | 8 | 0 | 0 | legacy mainframe |
| `ms_mbf32` | 32 | 1/8/23 | 129 | 7.2 | 76.5 | -1.0 | bitexact | 8 | 0 | 0 | MS BASIC legacy |
| `vax_f` | 32 | 1/8/23 | 128 | 7.2 | 76.5 | -1.0 | bitexact | 8 | 0 | 0 | DEC legacy |
| `cray_float` | 64 | 1/15/48 | 16384 | 14.8 | 9863.2 | -1.0 | bitexact | 8 | 0 | 0 | Cray legacy |
| `ibm_hfp64` | 64 | 1/7/56 | 64 | 17.2 | 37.9 | -1.0 | bitexact | 8 | 0 | 0 | legacy mainframe |
| `ms_mbf64` | 64 | 1/8/55 | 129 | 16.9 | 76.5 | -1.0 | bitexact | 8 | 0 | 0 | MS BASIC legacy |
| `vax_d` | 64 | 1/8/55 | 128 | 16.9 | 76.5 | -1.0 | bitexact | 8 | 0 | 0 | DEC legacy double |
| `vax_g` | 64 | 1/11/52 | 1024 | 16.0 | 615.9 | -1.0 | bitexact | 8 | 0 | 0 | DEC legacy |
| `ibm_hfp128` | 128 | 1/7/120 | 64 | 36.4 | 37.9 | -1.0 | bitexact | 8 | 0 | 0 | legacy mainframe |
| `vax_h` | 128 | 1/15/112 | 16384 | 34.0 | 9863.2 | -1.0 | bitexact | 8 | 0 | 0 | DEC quad |

### Сжатие (4)

| Формат | Бит | S/E/M | Bias | Знач. цифр | Дин. диап. (дек) | φ-расст. | SW-conf. | n_vec | decode-HW | compute-HW | Назначение |
|---|--:|---|--:|--:|--:|--:|---|--:|---|---|---|
| `block_fp` | 0 | 0/0/0 | 0 | 0.3 | — | -1.0 | structural | 0 | 0 | 0 | per-tile shared exponent |
| `shared_exp` | 0 | 0/0/0 | 0 | 0.3 | — | -1.0 | structural | 0 | 0 | 0 | LLM quantization |
| `stochastic_rounding` | 0 | 0/0/0 | 0 | 0.3 | — | -1.0 | structural | 0 | 0 | 0 | training small networks at low precision |
| `per_channel_scale` | 8 | 1/0/7 | 0 | 2.4 | — | -1.0 | structural | 0 | 0 | 0 | standard quant inference |

### Теоретические (4)

| Формат | Бит | S/E/M | Bias | Знач. цифр | Дин. диап. (дек) | φ-расст. | SW-conf. | n_vec | decode-HW | compute-HW | Назначение |
|---|--:|---|--:|--:|--:|--:|---|--:|---|---|---|
| `minifloat` | 0 | 1/0/0 | 0 | 0.3 | — | -1.0 | structural | 0 | 0 | 0 | design space of GF4/GF8/GF12/GF16 |
| `tapered_fp` | 0 | 1/0/0 | 0 | 0.3 | — | -1.0 | structural | 0 | 0 | 0 | variable mantissa via regime bits |
| `unum_i` | 0 | 1/0/0 | 0 | 0.3 | — | -1.0 | structural | 0 | 0 | 0 | interval arithmetic |
| `unum_ii` | 0 | 0/0/0 | 0 | 0.3 | — | -1.0 | structural | 0 | 0 | 0 | lookup-table real arithmetic; not GF-comparabl |

---

## 4. Известные оговорки честности

- `gf128`, `gf256` показывают `bias=0` — это значение из текущих экспериментальных спецификаций SSOT (не финализированы), а не «отсутствие смещения». Помечены статусом Experimental.
- `gf256` — единственный GoldenFloat со статусом `structural` (нет числовых векторов); остальные крупные GF (`gf14/48/96/512/1024`) — `self-consistent` (слабее bitexact).
- Все 22 формата кластера GoldenFloat имеют `gf_relation: self`; 61 внешний формат — референсные (competitor/ally/orthogonal) для сравнения, не часть собственной архитектуры Trinity.
- Динамический диапазон для gf512/gf1024 посчитан в log-домене из-за переполнения double — порядок верен, последние цифры — оценка.

Источник истины: [t27 / FORMAT-SPEC-001.json](https://github.com/gHashTag/t27/blob/master/conformance/FORMAT-SPEC-001.json), [INDEX_all_formats.json](https://github.com/gHashTag/t27/blob/master/conformance/vectors/INDEX_all_formats.json), [gen/numeric/formats_catalog.json](https://github.com/gHashTag/t27/blob/master/gen/numeric/formats_catalog.json). HW-доказательства: [trinity-fpga #199](https://github.com/gHashTag/trinity-fpga/issues/199).
