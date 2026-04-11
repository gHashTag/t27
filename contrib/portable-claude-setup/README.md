# Portable Claude Code profile

Dotfiles-style bundle for [Claude Code](https://www.anthropic.com/claude-code): **secrets live only in `~/.claude/.env`** (never committed); `settings.json` is generated/updated by a script. This GitHub repo contains **only this folder**, not your full `~/.claude` tree.

## Contents

| Path | Purpose |
|------|---------|
| `templates/settings.template.json` | Settings skeleton **without** tokens; paths are placeholders |
| `env.example` | Variables to copy into `~/.claude/.env` |
| `scripts/sync-settings-from-env.sh` | Merges `.env` into `settings.json` (`env.*` and optional MCP commands) |
| `scripts/apply-anthropic-token-from-env.sh` | Legacy name; calls `sync-settings-from-env.sh` |
| `scripts/rotate-keys.sh` | Rotate numbered API keys (round-robin / random / health-check) |
| `scripts/check-key-health.sh` | Test all configured keys and report status |

## Clone from GitHub

```bash
git clone https://github.com/gHashTag/portable-claude-setup.git
cd portable-claude-setup
```

## New machine

1. Install `jq`, `bash`, `git`. Optionally [GitHub CLI](https://cli.github.com/) (`gh auth login`).

2. Create local Claude Code files:

   ```bash
   cp templates/settings.template.json ~/.claude/settings.json
   cp env.example ~/.claude/.env
   chmod 600 ~/.claude/.env
   ```

   Edit `~/.claude/.env` (tokens and real paths). Adjust `settings.json` for hooks, MCP, plugins if needed.

3. Inject secrets into `settings.json`:

   ```bash
   bash scripts/sync-settings-from-env.sh ~/.claude/.env ~/.claude/settings.json
   ```

4. After changing keys in `.env`, run the same command again (or add a shell alias).

## Multi-account Z.AI / `ZAI_KEY_N`

Set `ZAI_USE=2` and `ZAI_KEY_2=...`; otherwise the first non-empty `ZAI_KEY_1`, `ZAI_KEY_2`, … (numeric order) is used.

## Key Rotation

When you have multiple API keys for the same provider (e.g. several Railway accounts), the rotation scripts let you cycle through them automatically.

### Setup

Add numbered keys to `~/.claude/.env`:

```bash
# Railway accounts
RAILWAY_TOKEN_1=railway-token-aaa
RAILWAY_TOKEN_2=railway-token-bbb
RAILWAY_TOKEN_3=railway-token-ccc

# GitHub tokens
GH_TOKEN_1=ghp_aaa
GH_TOKEN_2=ghp_bbb

# OpenAI keys
OPENAI_KEY_1=sk-aaa
OPENAI_KEY_2=sk-bbb

# Z.AI / Anthropic (existing ZAI_KEY_N vars are already supported)
ZAI_KEY_1=sk-ant-aaa
ZAI_KEY_2=sk-ant-bbb
```

### Rotating keys

```bash
# Rotate all families (round-robin, default)
bash scripts/rotate-keys.sh

# Rotate a specific family
bash scripts/rotate-keys.sh --family railway

# Pick a random key
bash scripts/rotate-keys.sh --random

# Test each key, use the first healthy one
bash scripts/rotate-keys.sh --health-check

# View current state
bash scripts/rotate-keys.sh --status
```

Rotation state is stored in `~/.claude/.rotation-state.json`. After each rotation, `sync-settings-from-env.sh` runs automatically to update `settings.json`.

### Health check

Test all configured keys without rotating:

```bash
bash scripts/check-key-health.sh
```

### Cron integration

Auto-rotate every 30 minutes with a health check:

```cron
*/30 * * * * bash ~/portable-claude-setup/scripts/rotate-keys.sh --health-check
```

Or check health and alert (no rotation):

```cron
0 */6 * * * bash ~/portable-claude-setup/scripts/check-key-health.sh || echo "Key failure" | mail -s "API key down" you@example.com
```

## Security

- **Do not push** your entire `~/.claude` directory: it may include `history.jsonl`, plugin caches, and tracked `settings.json` with secrets.
- This repo should contain **only** templates and scripts—no `.env` or real tokens.

## OpenCode key rotation

Analogous to the Claude Code rotation above, `rotate-opencode-keys.sh` manages ZAI keys in `~/.local/share/opencode/auth.json`:

```bash
# Rotate ZAI keys (reads ZAI_KEY_1..N from ~/.claude/.env)
bash scripts/rotate-opencode-keys.sh

# View current state
bash scripts/rotate-opencode-keys.sh --status

# Random key selection
bash scripts/rotate-opencode-keys.sh --random
```

After rotation, `anthropic` and `zai-coding-plan` providers in `auth.json` point to the active key. All keys are also stored as `zai-1`..`zai-N` for manual provider selection.
