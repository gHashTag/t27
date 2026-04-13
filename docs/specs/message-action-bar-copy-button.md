# Spec: Message Action Bar with Copy Button
**Spec ID:** UI-001
**Ring:** UI-01 (User Interface Foundations)
**Status:** Draft
**Priority:** P1 (User Experience)
**Created:** 2026-04-11

---

## L1 TRACEABILITY
**Issue:** TBD (to be created)
**Component:** Frontend / Chat UI

---

## Problem Statement

Current chat interface lacks message action controls. Users cannot copy message text without manual text selection, which:
- Inconsistent with modern AI chat UX patterns
- Hinders workflow efficiency
- Particularly problematic on mobile devices
- Prevents quick reuse of generated content

---

## Goal

Implement message action bar with Copy button under every assistant message, enabling one-click text copying without selection.

---

## Scope (MVP)

### In Scope
- Copy button under every assistant message
- Full message text copying (plain text)
- Visual feedback (Copied state)
- Desktop and mobile support
- Accessibility (keyboard navigation, ARIA labels)

### Out of Scope
- Copy raw markdown
- Copy selected fragment
- Retry/Regenerate button
- Like/Dislike
- Export to file
- User message actions

---

## Acceptance Criteria

### AC-001: Action Bar Placement
- [ ] Action bar appears below every assistant message
- [ ] Bar aligned to message bottom edge
- [ ] On desktop: visible or appears on hover
- [ ] On mobile: always visible
- [ ] Does not overlap message content

### AC-002: Copy Button
- [ ] Copy button is first action in bar
- [ ] Icon represents copy action
- [ ] Minimum hit area: 36x36 px
- [ ] ARIA label: "Copy message"

### AC-003: Copy Behavior
- [ ] Clicking Copy copies full message text
- [ ] Clipboard uses `navigator.clipboard.writeText()`
- [ ] Fallback to `execCommand('copy')` for compatibility
- [ ] Copied text excludes UI elements (buttons, metadata, citations)

### AC-004: Success Feedback
- [ ] Button state changes to "Copied" after successful copy
- [ ] Icon changes to checkmark
- [ ] Returns to original state after 2 seconds
- [ ] Toast notification appears on error

### AC-005: Edge Cases
- [ ] Empty messages: Copy button disabled
- [ ] Streaming messages: Copy available after completion
- [ ] Long messages: Copy works without performance impact
- [ ] Code blocks: Copied as plain text
- [ ] Tables: Copied as readable text

### AC-006: Accessibility
- [ ] All buttons focusable via Tab
- [ ] Enter/Space activates Copy
- [ ] Color contrast meets WCAG AA
- [ ] Screen reader announces action

---

## Invariant Laws

| Law | Application |
|-----|-------------|
| L3 PURITY | ASCII-only source files, English identifiers |
| L4 TESTABILITY | Test, invariant, bench blocks required |
| L7 UNITY | No new .sh scripts; use existing build system |

---

## Component Architecture

```
MessageCard (existing)
  ├── MessageContent (existing)
  └── MessageActions (NEW)
      ├── CopyMessageButton (NEW)
      └── MoreButton (placeholder, hidden in MVP)
```

### Type Definitions

```typescript
// ui/types/message.ts
export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  isStreaming?: boolean;
}

export interface MessageActionsProps {
  message: ChatMessage;
  onCopy?: (text: string) => void;
}
```

---

## Technical Implementation

### Hook: useCopyToClipboard

```typescript
// ui/hooks/useCopyToClipboard.ts

type CopyState = 'idle' | 'success' | 'error';

export interface UseCopyToClipboardReturn {
  state: CopyState;
  copy: (text: string) => Promise<boolean>;
  reset: () => void;
}

export function useCopyToClipboard(): UseCopyToClipboardReturn {
  const [state, setState] = useState<CopyState>('idle');

  const copy = useCallback(async (text: string): Promise<boolean> => {
    if (!text) return false;

    try {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        await navigator.clipboard.writeText(text);
      } else {
        // Fallback for older browsers
        const textarea = document.createElement('textarea');
        textarea.value = text;
        textarea.style.position = 'fixed';
        textarea.style.opacity = '0';
        document.body.appendChild(textarea);
        textarea.select();
        const success = document.execCommand('copy');
        document.body.removeChild(textarea);
        if (!success) throw new Error('execCommand failed');
      }

      setState('success');
      return true;
    } catch (error) {
      setState('error');
      return false;
    }
  }, []);

  const reset = useCallback(() => {
    setState('idle');
  }, []);

  return { state, copy, reset };
}
```

### Helper: getCopyableMessageText

```typescript
// ui/utils/messageUtils.ts

export function getCopyableMessageText(message: ChatMessage): string {
  // Normalize line endings
  let text = message.content.replace(/\r\n/g, '\n');

  // Remove multiple empty lines
  text = text.replace(/\n{3,}/g, '\n\n');

  // Trim leading/trailing whitespace
  text = text.trim();

  return text;
}
```

### Component: MessageActions

```typescript
// ui/components/chat/MessageActions.tsx

import { useCopyToClipboard } from '../../hooks/useCopyToClipboard';
import { getCopyableMessageText } from '../../utils/messageUtils';
import { useEffect } from 'react';

interface MessageActionsProps {
  message: ChatMessage;
}

export function MessageActions({ message }: MessageActionsProps) {
  const { state, copy, reset } = useCopyToClipboard();

  const handleCopy = async () => {
    const text = getCopyableMessageText(message);
    await copy(text);
  };

  useEffect(() => {
    if (state === 'success') {
      const timer = setTimeout(reset, 2000);
      return () => clearTimeout(timer);
    }
  }, [state, reset]);

  if (message.role !== 'assistant') {
    return null; // Only show for assistant messages in MVP
  }

  return (
    <div className="message-actions" data-testid="message-actions">
      <button
        className="action-button"
        onClick={handleCopy}
        disabled={!message.content || message.isStreaming}
        aria-label="Copy message"
        title="Copy message"
      >
        {state === 'success' ? (
          <CheckIcon aria-hidden="true" />
        ) : (
          <CopyIcon aria-hidden="true" />
        )}
        <span className="action-label">
          {state === 'success' ? 'Copied' : 'Copy'}
        </span>
      </button>
    </div>
  );
}
```

---

## Test Requirements

### Unit Tests

```t27
spec ui/components/chat/MessageActions.test.ts

test "Copy button copies message text" {
    message = {
        id = "msg-001"
        role = "assistant"
        content = "Hello, world!"
        timestamp = 1649872345000
    }

    result = render(<MessageActions message={message} />)

    # Simulate copy click
    click_button(result, "Copy message")

    # Verify clipboard contains message text
    assert_clipboard_equals("Hello, world!")

    # Verify success state
    assert_text_visible(result, "Copied")
}

test "Copy button disabled for empty messages" {
    message = {
        id = "msg-002"
        role = "assistant"
        content = ""
        timestamp = 1649872345000
    }

    result = render(<MessageActions message={message} />)

    assert_button_disabled(result, "Copy message")
}

test "Copy button hidden for user messages" {
    message = {
        id = "msg-003"
        role = "user"
        content = "Hello!"
        timestamp = 1649872345000
    }

    result = render(<MessageActions message={message} />)

    assert_not_visible(result, "message-actions")
}

invariant "Copy state resets after 2 seconds" {
    # State transition: idle -> success -> idle
    # Time constraint: success state duration = 2000ms ± 100ms
}

bench "Copy operation performance" {
    # Operation time < 100ms for messages up to 10KB
}
```

---

## Integration Requirements

### Integration Points

1. **MessageCard Component**
   - Import `MessageActions` at bottom of card
   - Pass full message object as prop

2. **Clipboard Permissions**
   - Request on first user interaction
   - Handle permission denials gracefully

3. **Mobile Detection**
   - Use responsive design (CSS media queries)
   - Test on iOS Safari and Chrome Mobile

---

## Design Specifications

### Desktop Styles

```css
.message-actions {
  display: flex;
  gap: 8px;
  padding: 8px 12px;
  margin-top: 8px;
  background-color: rgba(0, 0, 0, 0.02);
  border-radius: 10px;
  align-items: center;
}

.action-button {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border: none;
  background: transparent;
  color: #666;
  border-radius: 6px;
  cursor: pointer;
  min-height: 36px;
  min-width: 36px;
  font-size: 14px;
  transition: all 0.15s ease;
}

.action-button:hover:not(:disabled) {
  background-color: rgba(0, 0, 0, 0.06);
  color: #333;
}

.action-button:active:not(:disabled) {
  background-color: rgba(0, 0, 0, 0.1);
}

.action-button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.action-label {
  font-size: 13px;
  font-weight: 500;
}

/* Success state */
.action-button[data-state="success"] {
  color: #10b981;
}
```

### Mobile Styles

```css
@media (max-width: 768px) {
  .message-actions {
    background-color: transparent;
    padding: 6px 0;
  }

  .action-button {
    padding: 8px 12px;
    min-height: 40px;
    min-width: 40px;
  }

  .action-label {
    display: block;
  }
}
```

---

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Copy operation time | < 100ms | Performance mark |
| Render time per message | < 16ms | Frame budget |
| Bundle size increase | < 5KB | Build output |

---

## Security Considerations

1. **Clipboard Access**
   - Only write to clipboard, never read
   - Sanitize text before writing (prevent XSS in rich environments)

2. **Content Truncation**
   - Limit copied text to prevent memory issues
   - Warn for messages > 100KB

---

## Accessibility Requirements

| Requirement | Implementation |
|-------------|----------------|
| Keyboard navigation | Tab index, Enter/Space activation |
| Screen reader | ARIA labels, live regions for feedback |
| Color contrast | WCAG AA (4.5:1 minimum) |
| Focus visible | Clear focus indicator |
| Touch targets | Minimum 36x36 px |

---

## Rollout Plan

### Phase 1: Implementation (Week 1)
- [ ] Create component structure
- [ ] Implement `useCopyToClipboard` hook
- [ ] Create `MessageActions` component
- [ ] Write unit tests

### Phase 2: Integration (Week 2)
- [ ] Integrate into `MessageCard`
- [ ] Add styling
- [ ] Test on desktop browsers
- [ ] Test on mobile devices

### Phase 3: Launch (Week 3)
- [ ] Feature flag integration
- [ ] Gradual rollout (10% → 50% → 100%)
- [ ] Monitor analytics
- [ ] Collect user feedback

---

## Success Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Copy button usage rate | > 15% of assistant messages | TBD |
| Copy success rate | > 99% | TBD |
| User satisfaction | > 4.0/5.0 | TBD |
| Mobile usage share | > 30% of copies | TBD |

---

## Open Questions

1. Should copy include citation references?
   - *Decision for MVP: No, plain text only*

2. Should user messages have copy button?
   - *Decision for MVP: No, assistant only*

3. Feature flag strategy?
   - *Decision: Use per-user flag, 100% rollout after validation*

---

## Dependencies

### External Dependencies
- None (uses native Clipboard API)

### Internal Dependencies
- `ChatMessage` type definition
- `MessageCard` component
- Toast notification system

---

## References

- [Clipboard API - MDN](https://developer.mozilla.org/en-US/docs/Web/API/Clipboard_API)
- [WCAG 2.1 AA Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [t27 Constitutional Laws](../T27-CONSTITUTION.md)

---

**φ² + φ⁻² = 3 | TRINITY**
