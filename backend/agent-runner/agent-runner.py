#!/usr/bin/env python3
"""
T27 Autonomous Agent Runner
Executes tasks via Z.AI/Anthropic API with tool use (bash, read, write).
Runs as a loop: prompt → think → execute tools → observe → repeat.

Env vars:
  ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN — Z.AI key
  ANTHROPIC_BASE_URL — default https://api.z.ai/api/anthropic
  TASK_PROMPT — what to do
  MAX_TURNS — max agent turns (default 50)
"""

import json, os, subprocess, sys, time, urllib.request, urllib.error
from pathlib import Path
from datetime import datetime, timezone

# ── Config ──
API_KEY = os.environ.get("ANTHROPIC_AUTH_TOKEN") or os.environ.get("ANTHROPIC_API_KEY", "")
BASE_URL = os.environ.get("ANTHROPIC_BASE_URL", "https://api.z.ai/api/anthropic")
MODEL = os.environ.get("MODEL", "claude-sonnet-4-5-20250514")
MAX_TURNS = int(os.environ.get("MAX_TURNS", "50"))
MAX_TOKENS = int(os.environ.get("MAX_TOKENS", "8192"))
TASK_PROMPT = os.environ.get("TASK_PROMPT", "")
LOG_FILE = os.environ.get("LOG_FILE", "/tmp/agent-log.jsonl")

def log(msg):
    ts = datetime.now(timezone.utc).strftime("%H:%M:%S")
    print(f"[{ts}] {msg}", flush=True)

def log_json(entry):
    with open(LOG_FILE, "a") as f:
        f.write(json.dumps(entry, ensure_ascii=False) + "\n")

# ── Tools ──
TOOLS = [
    {
        "name": "bash",
        "description": "Execute a bash command and return stdout/stderr. Use for: listing files, running tests, git operations, any shell command.",
        "input_schema": {
            "type": "object",
            "properties": {
                "command": {"type": "string", "description": "The bash command to execute"}
            },
            "required": ["command"]
        }
    },
    {
        "name": "read_file",
        "description": "Read the contents of a file. Use for reading source code, configs, specs, docs.",
        "input_schema": {
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "File path to read"}
            },
            "required": ["path"]
        }
    },
    {
        "name": "write_file",
        "description": "Write content to a file. Creates parent directories if needed.",
        "input_schema": {
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "File path to write"},
                "content": {"type": "string", "description": "Content to write"}
            },
            "required": ["path", "content"]
        }
    },
    {
        "name": "task_complete",
        "description": "Call this when the task is fully complete. Include a summary of what was done.",
        "input_schema": {
            "type": "object",
            "properties": {
                "summary": {"type": "string", "description": "Summary of completed work"}
            },
            "required": ["summary"]
        }
    }
]

def execute_tool(name, input_data):
    """Execute a tool and return the result string."""
    if name == "bash":
        cmd = input_data.get("command", "")
        log(f"  BASH: {cmd[:120]}")
        try:
            result = subprocess.run(
                cmd, shell=True, capture_output=True, text=True, timeout=120, cwd=os.getcwd()
            )
            output = result.stdout
            if result.stderr:
                output += f"\nSTDERR:\n{result.stderr}"
            if result.returncode != 0:
                output += f"\n[exit code: {result.returncode}]"
            return output[:10000]
        except subprocess.TimeoutExpired:
            return "[ERROR: command timed out after 120s]"
        except Exception as e:
            return f"[ERROR: {e}]"

    elif name == "read_file":
        path = input_data.get("path", "")
        log(f"  READ: {path}")
        try:
            content = Path(path).read_text(errors="replace")
            return content[:20000]
        except Exception as e:
            return f"[ERROR: {e}]"

    elif name == "write_file":
        path = input_data.get("path", "")
        content = input_data.get("content", "")
        log(f"  WRITE: {path} ({len(content)} chars)")
        try:
            p = Path(path)
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(content)
            return f"Written {len(content)} chars to {path}"
        except Exception as e:
            return f"[ERROR: {e}]"

    elif name == "task_complete":
        summary = input_data.get("summary", "")
        log(f"  COMPLETE: {summary[:200]}")
        return "TASK_COMPLETE"

    return f"[ERROR: unknown tool {name}]"


# ── API Call ──
def call_api(messages):
    """Call Z.AI/Anthropic API."""
    body = json.dumps({
        "model": MODEL,
        "max_tokens": MAX_TOKENS,
        "tools": TOOLS,
        "messages": messages,
    }).encode()

    headers = {
        "Content-Type": "application/json",
        "x-api-key": API_KEY,
        "anthropic-version": "2023-06-01",
    }

    req = urllib.request.Request(f"{BASE_URL}/v1/messages", data=body, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=120) as resp:
            return json.loads(resp.read())
    except urllib.error.HTTPError as e:
        error_body = e.read().decode() if e.fp else ""
        log(f"  API ERROR {e.code}: {error_body[:300]}")
        return {"error": {"message": error_body, "status": e.code}}
    except Exception as e:
        log(f"  API ERROR: {e}")
        return {"error": {"message": str(e)}}


# ── System Prompt ──
def build_system():
    parts = [
        "You are an autonomous SWE agent working in the T27 project.",
        "You have tools: bash (run commands), read_file, write_file, task_complete.",
        "Always read SOUL.md first — it is the constitutional law of this project.",
        "Follow the PHI LOOP: spec → gen → test → verdict → experience → commit.",
        "When done, call task_complete with a summary.",
        "Be thorough but efficient. Prefer small targeted changes.",
    ]
    # Add SOUL.md content if exists
    soul = Path("SOUL.md")
    if soul.exists():
        parts.append(f"\n--- SOUL.md (excerpt) ---\n{soul.read_text()[:3000]}")
    return "\n".join(parts)


# ── Main Loop ──
def main():
    if not API_KEY:
        log("ERROR: No API key set (ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN)")
        sys.exit(1)

    if not TASK_PROMPT:
        log("ERROR: No TASK_PROMPT set")
        sys.exit(1)

    log("═" * 50)
    log("  T27 AUTONOMOUS AGENT")
    log("═" * 50)
    log(f"Model: {MODEL}")
    log(f"Base URL: {BASE_URL}")
    log(f"Task: {TASK_PROMPT[:200]}")
    log(f"Max turns: {MAX_TURNS}")
    log(f"CWD: {os.getcwd()}")
    log("═" * 50)

    system = build_system()
    messages = [{"role": "user", "content": TASK_PROMPT}]

    for turn in range(1, MAX_TURNS + 1):
        log(f"\n{'─'*40} Turn {turn}/{MAX_TURNS} {'─'*40}")

        response = call_api([{"role": "user", "content": system + "\n\nUser task:\n" + TASK_PROMPT}] if turn == 1 else messages)

        if "error" in response:
            log(f"API error, retrying in 10s...")
            time.sleep(10)
            continue

        # Extract response content
        assistant_content = response.get("content", [])
        stop_reason = response.get("stop_reason", "")
        model = response.get("model", "?")
        usage = response.get("usage", {})

        log(f"  Model: {model} | Tokens: in={usage.get('input_tokens',0)} out={usage.get('output_tokens',0)} | Stop: {stop_reason}")

        # Log to file
        log_json({"turn": turn, "model": model, "usage": usage, "stop_reason": stop_reason,
                   "content_types": [c.get("type") for c in assistant_content]})

        # Add assistant response to messages
        messages.append({"role": "assistant", "content": assistant_content})

        # Process text blocks
        for block in assistant_content:
            if block.get("type") == "text":
                text = block.get("text", "")
                log(f"  TEXT: {text[:300]}")

        # Process tool calls
        if stop_reason == "tool_use":
            tool_results = []
            task_done = False

            for block in assistant_content:
                if block.get("type") == "tool_use":
                    tool_name = block.get("name", "")
                    tool_input = block.get("input", {})
                    tool_id = block.get("id", "")

                    result = execute_tool(tool_name, tool_input)

                    if result == "TASK_COMPLETE":
                        task_done = True
                        tool_results.append({
                            "type": "tool_result",
                            "tool_use_id": tool_id,
                            "content": "Task marked as complete."
                        })
                    else:
                        tool_results.append({
                            "type": "tool_result",
                            "tool_use_id": tool_id,
                            "content": result
                        })

            messages.append({"role": "user", "content": tool_results})

            if task_done:
                log("\n" + "═" * 50)
                log("  TASK COMPLETED SUCCESSFULLY")
                log("═" * 50)
                return 0

        elif stop_reason == "end_turn":
            log("Agent finished (end_turn)")
            return 0

        else:
            log(f"Unexpected stop_reason: {stop_reason}")

    log(f"\nMax turns ({MAX_TURNS}) reached")
    return 1


if __name__ == "__main__":
    sys.exit(main())
