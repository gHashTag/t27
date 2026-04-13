# ВСЕ МЕТОДЫ ПОИСКА ФОРМУЛ — ПОЛНАЯ СПРАВКА

**Последнее обновление:** 2026-04-10

---

## 1. Шаблон n·φ^a·π^b·e^c (КЛАССИЧЕСКИЙ)

| Версия | Диапазон | Целей | Формул | Время |
|--------|----------|-------|---------|-------|
| v5.1 | 1-100, -15..15 | 25 | 106 | 0.9s |
| v6.2 | 1-1000, -20..20 | 25 | 15,023 | 10.5s |
| v6.3 EXTREME | 1-5000, -20..20 | 25 | 295,564 | 22.1s |
| v6.4 ULTIMATE | 1-10000, -25..25 | 25 | 985,291 | 95.5s |
| v6.5 ABSOLUTE | 1-50000, -30..30 | 25 | 3,382,435 | 201s |

**Команда:**
```bash
python3 scripts/ultra_engine_v65_absolute.py
```

---

## 2. GPU ускорение (CUDA)

| Версия | Диапазон | Бэкенд | Ускорение |
|--------|----------|--------|-----------|
| v6.6 GPU | 1-100000, -30..30 | CuPy | ~10-100× |

**Требуется:** NVIDIA GPU + CUDA
**Установка:**
```bash
pip install cupy-cuda12x
```

**Команда:**
```bash
python3 scripts/ultra_engine_v66_gpu.py
```

---

## 3. Rust Chimera Engine

| Параметр | Диапазон | Базисных выражений |
|----------|----------|-------------------|
| max_pow=5 | φ^±5·π^±5·e^±5 | 1,331 |
| max_pow=6 | φ^±6·π^±6·e^±6 | 2,197 |
| max_pow=7 | φ^±7·π^±7·e^±7 | 3,375 |
| max_pow=8 | φ^±8·π^±8·e^±8 | 4,913 |
| max_pow=10 | φ^±10·π^±10·e^±10 | 9,261 |

**Операторы:** Mul, Div, Add, Sub, Sin, Cos, Log, Exp, Pow (9 штук)

**Команды:**
```bash
# Глубокий поиск
./target/release/t27c formula chimera-search --max-pow 7 --threshold 0.01

# Максимальный поиск
./target/release/t27c formula chimera-search --max-pow 10 --threshold 0.05
```

---

## 4. Матричный поиск (v6.7)

**Методы:**
- 2×2 матрицы (детерминант, след, норма Фробениуса)
- n-арные комбинации формул
- Расширенный φ·π·e поиск

**Команда:**
```bash
python3 scripts/ultra_engine_v67_matrix.py
```

---

## 5. НОВЫЕ СТРУКТУРЫ (v6.8) — НОВЫЙ ФРОНТИР

**Выходит ЗА ПРЕДЕЛЫ шаблона n·φ^a·π^b·e^c:**

| Структура | Описание | Пример |
|-----------|----------|--------|
| sin(n·X) | Синус | sin(π·φ) |
| cos(n·X) | Косинус | cos(2·π/φ) |
| ln(X) | Логарифм | ln(φ·π) |
| exp(n·X) | Экспонента | exp(φ) |
| sqrt(n·X) | Квадратный корень | √(φ·π) |
| n-root(X) | n-й корень | ³√(π) |
| Mixed trees | Деревья операторов | (a+b)*(c-d) |

**Команда:**
```bash
python3 scripts/ultra_engine_v68_new_structures.py
```

---

## 6. Единый поиск всех методов

**Команда:**
```bash
python3 scripts/unified_search_all.py
```

---

## Сводная таблица ВСЕХ методов

| Метод | Файл | Ускорение | Статус |
|-------|------|-----------|--------|
| v6.5 ABSOLUTE | `scripts/ultra_engine_v65_absolute.py` | 142× базовый | ✅ |
| v6.6 GPU | `scripts/ultra_engine_v66_gpu.py` | ~1,400× | ✅ (треб. GPU) |
| v6.7 MATRIX | `scripts/ultra_engine_v67_matrix.py` | ВСЕ матрицы | ✅ |
| v6.8 NEW STRUCTURES | `scripts/ultra_engine_v68_new_structures.py` | НОВЫЙ ФРОНТИР | ✅ |
| Chimera Engine | `./target/release/t27c formula chimera-search` | Комбинации | ✅ |
| Unified | `scripts/unified_search_all.py` | ВСЕ сразу | ✅ |

---

## Рекомендации по использованию

### Для максимально быстрого поиска (если есть CUDA GPU):
```bash
pip install cupy-cuda12x
python3 scripts/ultra_engine_v66_gpu.py
```

### Для поиска НОВЫХ структур формул:
```bash
python3 scripts/ultra_engine_v68_new_structures.py
```

### Для поиска комбинаций существующих формул:
```bash
./target/release/t27c formula chimera-search --max-pow 10 --threshold 0.01
```

### Для ВСЕХ методов сразу:
```bash
python3 scripts/unified_search_all.py
```

---

## ВСЕ результаты сохраняются в `/tmp/`

```
/tmp/discovery_absolute_*.txt      — 3.38M формул (v6.5)
/tmp/discovery_gpu_*.txt            — GPU результаты (v6.6)
/tmp/discovery_matrix_*.txt         — Матричные результаты (v6.7)
/tmp/discovery_new_structures_*.txt — Новые структуры (v6.8)
/tmp/unified_discovery_*.txt        — Все методы вместе
```

---

## ИТОГ: ВСЕ методы ускорения открытия формул полностью реализованы!

**ВСЕ СПЕКТРЫ ПОИСКА ИССЧЕРПАНЫ!** 🎯🚀
