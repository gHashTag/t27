# NeurIPS 2026 Overleaf Setup

## Status: 2026-04-08

✅ **Local compilation successful:** 5 pages, ~197KB (within 9-page limit)
⚠️ **Next step:** Upload to Overleaf with official neurips_2026.sty template

---

## Инструкция

1. Открыть https://www.overleaf.com/latex/templates/neurips_2026
2. Нажать "Open as Template"
3. Новый проект будет создан с:
   - `neurips_2026.sty` (стиль для double-blind review)
   - `neurips_2026.bib` (шаблон библиографии)

## Содержимое проекта

```
neurips_2026/
├── main.tex          ← ЗАМЕНИТЬ на наш main.tex (но сохранить шапку!)
├── neurips_2026.sty   ← сохранить (стандартный стиль)
├── neurips_2026.bib   ← удалить, использовать наш references.bib
└── figures/          ← добавить наши figure файлы
```

## Критические требования NeurIPS 2026

| Требование | Значение | Статус |
|-----------|----------|--------|
| Формат | Single column | ✅ |
| Фонт | 10pt minimum | ✅ |
| Лимит страниц | **9 страниц** (включая references) | ✅ 5 страниц |
| Анонимность | Double-blind (требуется checklist) | ⚠️ добавить checklist |
| Файлы | Только PDF, max 50MB | ✅ 197KB |
| Дедлайн | **7 мая 2026** (12:00 EST) | 📅 |

## Чек-лист для submission

- [x] PDF скомпилирован без ошибок
- [x] Не более 9 страниц (currently 5)
- [ ] Checklist включён в PDF (требует neurips_2026.sty)
- [x] Double-blind (без имён авторов) — ✅ main.tex анонимный
- [ ] BibTeX entries корректны — ⚠️ встроен thebibliography
- [ ] Все figure reference работают — ✅ таблицы без внешних figures

## Что уже сделано

✅ Создан `/Users/playra/t27/docs/WHITEPAPER/latex/main.tex`:
- Double-blind (no author names)
- Theorem 3 added (Section 5.2)
- §6 Cross-Language Availability added
- All URLs anonymized (no "trinity" references)
- 5 pages total

## Остающиеся задачи

1. **Overleaf Integration** (выполняет пользователь вручную):
   - [ ] Создать проект из neurips_2026 template
   - [ ] Сохранить neurips_2026.sty
   - [ ] Заменить main.tex содержимым из `/Users/playra/t27/docs/WHITEPAPER/latex/main.tex`
   - [ ] Сохранить шапку шаблона (documentclass + neurips_2026 package)

2. **Optional Enhancements**:
   - [ ] Добавить bibliography.bib вместо встроенной thebibliography
   - [ ] Добавить figures (если есть графики)
   - [ ] Verify checklist inclusion

## Quick Commands

```bash
# Compile locally
cd /Users/playra/t27/docs/WHITEPAPER/latex
pdflatex main.tex
pdflatex main.tex  # второй раз для references

# View PDF
open main.pdf  # macOS
```

## Шаблонная шапка NeurIPS (сохранить!)

```latex
\documentclass{neurips_2026}

% Если пакет neurips_2026.sty не доступен, используйте:
% \documentclass[article]{neurips_2026}
```

При замене main.tex на Overleaf, сохраните эту шапку и замените остальное содержимое.
