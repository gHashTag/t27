# OpenCode Usage Workflow

**CRITICAL RULE FOR AI AGENT:**
NEVER run `opencode run` in the background via hidden terminal sessions when you need to execute a large autonomous task that the user will verify.

Instead:
1. Run `opencode web`.
2. This opens a local interface in your browser (localhost).
3. The user can observe logs, the agent's chain of thought, and created files in a full UI.
4. Give the agent commands so that the process is displayed in the web interface — the user "does nothing but watches."
