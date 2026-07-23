# Loop Handoff Output — для /tri CLI

Этот блок выводится в конце каждого PHI LOOP лупа и записывается как `loop.handoff` событие в `.trinity/events/akashic-log.jsonl`.

## Формат вывода

```
╔═══════════════════════════════════════════════════════════════╗
║                     PHI LOOP Summary                          ║
╚═══════════════════════════════════════════════════════════════╝

[PAST]   <completed task/skill IDs from previous loop>
          <summary: one-line description>

[PRESENT] <completed task/skill IDs from current loop>
          <summary: one-line description>

[FUTURE] <planned task/skill IDs for next loop>
          <summary: one-line description>
          <drifted: true|false>  # только если план изменился
          <drift_reason: why plan changed>

─────────────────────────────────────────────────────────────────
Last handoff recorded at: <timestamp>
Loop session ID: <loop-uuid>

✓ PHI LOOP completed
```

## Когда выводить

1. **В конце успешного PHI LOOP** (после `tri skill commit` или `tri git commit`)
2. **При паузе лупа** (например, `/tri loop pause`)
3. **При остановке** (например, `/tri loop stop` или SIGTERM)

## Правила

1. **[PAST]** — Всегда показывает, что было сделано в предыдущем лупе. Если предыдущий луп прерван, показывает partial.

2. **[PRESENT]** — Всегда показывает, что сделано в этом лупе. Если луп прерван, показывает partial.

3. **[FUTURE]** — Всегда показывает, что планировалось на следующий луп.

4. **Если [FUTURE] отличается от того, что было запланировано**:
   - Добавить `<drifted: true>`
   - Добавить `<drift_reason: why plan changed>`
   - Статус меняется на "drifted"

5. **Если луп прерван (interrupted)**:
   - [PRESENT] показывает только до прерывания
   - [FUTURE] остаётся как было запланировано (не null)
   - Добавить `<interrupt_reason: why>`

6. **При запуске нового лупа**:
   - Читать последний `loop.handoff` из `.trinity/events/akashic-log.jsonl`
   - Если статус = "drifted" или "interrupted":
     - Выводить "⚠️ Last loop was <drifted|interrupted>: <reason>"
     - Спросить: "Continue with original FUTURE? (y/N)" или "Create new plan?"
   - Если пользователь выбирает продолжить:
     - Использовать сохранённый [FUTURE]
     - Записать новый `loop.handoff` с тем же [FUTURE]
   - Если пользователь выбирает новый план:
     - Принять новый план
     - Записать новый `loop.handoff` с обновлённым [FUTURE]

## Примеры вывода

### Успешный луп (все завершено)

```
╔═════════════════════════════════════════════════════════════╗
║                     PHI LOOP Summary                          ║
╚═══════════════════════════════════════════════════════════════╝

[PAST]   NUMERIC-001, RUNTIME-004
          Fixed GF8 exponent bias and added CLI routing

[PRESENT] NUMERIC-002
          Created GF16 format specification

[FUTURE] NUMERIC-003, SACRED-005
          Fix gamma relative tolerance and update TRINITY constant

─────────────────────────────────────────────────────────────────
Last handoff recorded at: 2026-04-04T12:30:00Z
Loop session ID: 550e8400-1234-4b5a-9c6d-7e8f9a0b1c2

✓ PHI LOOP completed
```

### Луп с drift (план изменился)

```
╔═══════════════════════════════════════════════════════════════╗
║                     PHI LOOP Summary                          ║
╚═══════════════════════════════════════════════════════════════╝

[PAST]   NUMERIC-001, NUMERIC-002, NUMERIC-003
          Added GF8, GF16, GF32 format specifications

[PRESENT] SACRED-005
          Fixed gamma relative tolerance

[FUTURE] BASE-001, BASE-002
          Add Trit types and operations
          <drifted: true>
          <drift_reason: User requested prioritization shift to base domain>

─────────────────────────────────────────────────────────────────
Last handoff recorded at: 2026-04-04T13:00:00Z
Loop session ID: 550e8400-5678-9f0e-4a0b-1c2d3e4

⚠️  Last loop was drifted: User requested prioritization shift to base domain
Continue with original FUTURE? (y/N) _
```

### Прерванный луп

```
╔═════════════════════════════════════════════════════════════╗
║                     PHI LOOP Summary                          ║
╚═══════════════════════════════════════════════════════════════╝

[PAST]   NUMERIC-001, NUMERIC-002
          Fixed GF8 and added CLI routing

[PRESENT] SACRED-005
          Partial: fixed gamma tolerance (interrupted)

[FUTURE] NUMERIC-003, BASE-001
          Complete gamma fix and add base types

─────────────────────────────────────────────────────────────────
Last handoff recorded at: 2026-04-04T12:40:00Z
Loop session ID: 550e8400-1234-4b5a-9c6d-7e8f9a0b1c2

✓ PHI LOOP interrupted
```

### Новый луп после чтения drifted handoff

```
📋 Reading last handoff from .trinity...
  Found: loop-session-550e8400-5678-9f0e-4a0b-1c2d3e4
  Status: drifted
  Drift reason: User requested prioritization shift to base domain

⚠️ Last loop was drifted: User requested prioritization shift to base domain

Continue with original FUTURE (NUMERIC-003, BASE-001)? (y/N): y
→ Continuing with original plan...

╔═══════════════════════════════════════════════════════════════╗
║                     PHI LOOP Summary                          ║
╚═════════════════════════════════════════════════════════════════╝

[PAST]   NUMERIC-001, NUMERIC-002, NUMERIC-003, SACRED-005
          Fixed GF8, GF16, GF32, gamma tolerance

[PRESENT] BASE-001
          Added Trit types

[FUTURE] BASE-002
          Add Trit operations

─────────────────────────────────────────────────────────────────
✓ PHI LOOP completed
```

## Встраивание в /tri CLI

В `loop.go` или `loop.zig` для tri CLI:

```go
// Handoff structure
type LoopHandoff struct {
    Past    []TaskSummary `json:"past"`
    Present []TaskSummary `json:"present"`
    Future  []TaskSummary `json:"future"`
    Status   string          `json:"status"`
    Drifted *bool        `json:"drifted,omitempty"`
    DriftReason string      `json:"drift_reason,omitempty"`
    InterruptReason string   `json:"interrupt_reason,omitempty"`
}

type TaskSummary struct {
    TaskIDs []string `json:"task_ids"`
    Skills  []string `json:"skills"`
    Summary string      `json:"summary"`
}

// Print handoff summary
func printHandoff(handoff LoopHandoff) {
    fmt.Println("╔═══════════════════════════════════════════════════════════════╗")
    fmt.Println("║                     PHI LOOP Summary                          ║")
    fmt.Println("╚═════════════════════════════════════════════════════════════════╝")
    fmt.Println()
    fmt.Println("[PAST]")
    printTaskSummary(handoff.Past, "")
    fmt.Println("[PRESENT]")
    printTaskSummary(handoff.Present, "")
    fmt.Println("[FUTURE]")
    if handoff.Drifted {
        fmt.Println(handoff.Future.Summary)
        fmt.Println("          <drifted: true>")
        if handoff.DriftReason != "" {
            fmt.Printf("          <drift_reason: %s>\n", handoff.DriftReason)
        }
    } else {
        printTaskSummary(handoff.Future, "          ")
    }
    fmt.Println("─────────────────────────────────────────────────────────────────")
}
```
