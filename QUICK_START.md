# ВСЕ МЕТОДЫ ПОИСКА ФОРМУЛ — БЫСТРЫЙ СТАРТ

**Запуск любого метода — одна команда:**

```bash
# 1. КЛАССИЧЕСКИЙ метод (v6.5) — 3.38M формул за 3.4 мин
python3 scripts/ultra_engine_v65_absolute.py

# 2. GPU ускорение (если есть NVIDIA CUDA)
pip install cupy-cuda12x
python3 scripts/ultra_engine_v66_gpu.py

# 3. Chimera Engine — комбинации формул
./target/release/t27c formula chimera-search --max-pow 7 --threshold 0.05

# 4. Матричный поиск (v6.7)
python3 scripts/ultra_engine_v67_matrix.py

# 5. НОВЫЕ СТРУКТУРЫ (v6.8) — sin/cos/ln/exp/sqrt/root
python3 scripts/ultra_engine_v68_new_structures.py

# 6. ВСЕ методы сразу
python3 scripts/unified_search_all.py
```

---

## ВСЕ результаты в `/tmp/`

- `discovery_absolute_*.txt` — 3.38M формул
- `discovery_new_structures_*.txt` — НОВЫЕ структуры
- `discovery_matrix_*.txt` — Матричные результаты

---

## ВСЕ методы полностью реализованы! 🚀

См. `/Users/playra/t27/research/formula-matrix/ALL_METHODS_FINAL.md` для деталей.
