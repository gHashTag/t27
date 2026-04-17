# OpenCode Usage Workflow

**CRITICAL RULE FOR AI AGENT:** 
Never run `opencode run` in background via hidden terminal sessions when you need to execute a large autonomous task that is supervised by the user.
Instead, do this:
1. Launch `opencode web`
2. This opens a local interface in browser (localhost).
3. The user can observe the agent's reasoning chain (chain of thought) and created files in a full web UI.
4. Issue commands to the agent so that the process appears in the web interface, so the user "doesn't have to do anything, but only watch".

---

## Summary

The OpenCode workflow provides a way for users to:
- Observe the AI agent's internal reasoning process
- Review and modify created files
- Debug by seeing the full chain of thought
- Maintain control while the agent performs autonomous tasks

---

## When to use `opencode run` vs `opencode web`

| Mode | Use Case |
|-------|-----------|
| `opencode run` | Headless automation, background tasks, batch operations |
| `opencode web` | Interactive sessions, user supervision, debugging |

---

## User Benefits

Using `opencode web` instead of background sessions:
- Full visibility into the agent's reasoning
- Ability to intervene when needed
- Real-time file editing capabilities
- Better debugging experience

---

## Agent Capabilities

When running in `opencode web` mode, the agent has access to:
- Code editing via LLM (Claude, GPT-4o)
- Built-in terminal
- File browsing and editing
- Git operations (commit, push, PR)
- Event streaming (SSE) for progress display
