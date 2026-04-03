# План: Миграция .t27 спецификаций из trinity-w1

## Контекст

Целевой репозиторий `/Users/playra/t27` уже содержит значительную часть структуры:
- ✅ **Сделано**: specs/math/* (constants, sacred_physics), specs/numeric/* (GF4-32, TF3, phi_ratio), compiler/parser, compiler/codegen/zig/verilog, conformance/*, docs/*, architecture/*, .trinity/*
- ❌ **Отсутствует**: specs/vsa/ops.t27, specs/isa/registers.t27, specs/nn/attention.t27, specs/nn/hslm.t27, specs/fpga/mac.t27, specs/queen/lotus.t27, compiler/codegen/c/codegen.t27, compiler/runtime/runtime.t27, compiler/lexer.t27

Исходный репозиторий `/Users/playra/trinity-w1` содержит Zig код который нужно экстрагировать в .t27 спецификации:
- src/tri/math/ - sacred formula, constants
- src/vsa/ - vector operations
- src/hslm/ - HSLM attention
- src/isa/, src/fpga/ - registers, MAC operations
- src/tri/ - orchestration, cells

## Цель

Создать недостающие .t27 спецификации на основе существующего Zig кода в trinity-w1, сохраняя архитектурную целостность и зависимости в graph_v2.json.

## План выполнения через tri-cell

### Шаг 1: Сохранить план как документ

**Действие**: Сохранить этот план в `t27/docs/migration-plan-vsa-nn-fpga-queen.md`
Это станет каноническим reference для всех будущих агентов.

### Шаг 2: Создать три-cell для миграции

**Пример протокола**:
```bash
tri cell begin --issue <N> --episode migrate-trinity-w1-specs
tri cell checkpoint --step "VSA ops spec skeleton created"
tri cell checkpoint --step "ISA registers spec skeleton created"
tri cell checkpoint --step "NN attention/HSLM specs created"
tri cell checkpoint --step "FPGA MAC + Queen Lotus specs created"
tri cell checkpoint --step "compiler C codegen/runtime/lexer specs created"
tri cell checkpoint --step "graph_v2.json updated with new nodes/edges/order"
tri cell seal
tri verdict --toxic
tri experience save --episode migrate-trinity-w1-specs
git add .
git commit -m "feat: migrate VSA/NN/FPGA/Queen specs from trinity-w1"
git push
```

### Шаг 3: Исполнение по шагам (в порядке из плана)

**Детальное исполнение**:

1. **VSA ops** → `t27/specs/vsa/ops.t27` из `src/vsa/agent/core.zig` и `src/vsa/common.zig`
   - Функции: bind, unbind, bundle, similarity, trit_cosine
   - use base::types, base::ops

2. **ISA registers** → `t27/specs/isa/registers.t27` из `src/tri27/` или `src/isa/`
   - Функции: Register, RegisterFile, R0-R26, Coptic encoding
   - use base::types

3. **NN attention** → `t27/specs/nn/attention.t27` из `src/hslm/attention.zig`
   - Функции: sacred_attention, d_k^(-φ³) kernel
   - use math::constants, base::types, numeric::gf16

4. **HSLM** → `t27/specs/nn/hslm.t27` из `src/hslm/` и `src/tri/brain/`
   - Функции: HSLM, forward, backward, phase
   - use nn::attention, math::sacred_physics, numeric::gf16

5. **FPGA MAC** → `t27/specs/fpga/mac.t27` из `src/fpga/`
   - Функции: ZeroDSP_MAC, LUT, MAC cycle
   - use base::types, base::ops, isa::registers

6. **Queen Lotus** → `t27/specs/queen/lotus.t27` из `src/tri/queen/` или `src/tri/cell.zig`
   - Функции: 6-phase orchestrate, phase management, cell infrastructure
   - use nn::hslm, compiler::runtime

7. **C codegen** → `t27/compiler/codegen/c/codegen.t27` из существующих Zig codegen
   - Функции: CCodeGen, emit_c, c_ast, c_headers
   - use compiler::parser, compiler::runtime

8. **Runtime** → `t27/compiler/runtime/runtime.t27` из существующего runtime
   - Функции: T27Runtime, init, execute, shutdown
   - use base::types

9. **Lexer** → `t27/compiler/parser/lexer.t27` на основе parser.t27
   - Функции: Lexer, tokenize, Token, TokenType
   - Dependencies: parser uses lexer

10. **Обновление graph_v2.json**
    - Добавить 8 новых узлов
    - Добавить зависимости
    - Обновить topological_order

## Файлы для создания/модификации

| Файл | Действие | Ключевые элементы |
|-------|-----------|------------------|
| t27/specs/vsa/ops.t27 | CREATE | bind, unbind, bundle, similarity |
| t27/specs/isa/registers.t27 | CREATE | Register, RegisterFile, R0-R26 |
| t27/specs/nn/attention.t27 | CREATE | sacred_attention, d_k^(-φ³) kernel |
| t27/specs/nn/hslm.t27 | CREATE | HSLM, forward, backward, phase |
| t27/specs/fpga/mac.t27 | CREATE | ZeroDSP_MAC, LUT, MAC cycle |
| t27/specs/queen/lotus.t27 | CREATE | 6-phase, orchestrate, cell |
| t27/compiler/codegen/c/codegen.t27 | CREATE | CCodeGen, emit_c, c_ast |
| t27/compiler/runtime/runtime.t27 | CREATE | T27Runtime, init, execute |
| t27/compiler/parser/lexer.t27 | CREATE | Lexer, tokenize, TokenType |
| t27/architecture/graph_v2.json | UPDATE | new nodes, edges, topological_order |

## Критерий готовности миграции

- [ ] Все девять `.t27` файлов созданы
- [ ] graph_v2.json обновлён
- [ ] tri-cell sealed и committed
- [ ] План сохранён как документ в `t27/docs/migration-plan-vsa-nn-fpga-queen.md`
