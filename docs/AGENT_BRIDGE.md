# Agent Bridge CLI & Message Handling

## Overview

The Agent Bridge enables Trinity agents (A-Z, Queen Q, and 27th agent) to communicate with the Orchestrator Chat UI via HTTP API and Server-Sent Events (SSE).

## Architecture

```
┌─────────────────┐     HTTP POST     ┌─────────────────┐
│   Agent CLI     │ ───────────────► │                 │
│  (agent-say.ts) │                   │  Trinity Core   │
└─────────────────┘                   │   Backend       │
                                      │   (port 8082)   │
┌─────────────────┐     SSE          │                 │
│  Chat UI (Svelte)│ ◄───────────────┤                 │
│ QueenTrinityChat│   /events        └─────────────────┘
└─────────────────┘
```

## Components

### 1. Agent CLI (`scripts/agent-say.ts`)

CLI utility for agents to send messages:

```bash
npx tsx scripts/agent-say.ts <agent-id> <message-type> <content> [emoji]
```

**Arguments:**
- `agent-id`: Agent identifier (A-Z, Q, or 27)
- `message-type`: message, test_result, status, error
- `content`: Message content (wrap in quotes)
- `emoji`: Optional emoji (default based on type)

**Examples:**
```bash
npx tsx scripts/agent-say.ts A message "Hello from Agent A" 💬
npx tsx scripts/agent-say.ts Q test_result "All agents healthy" 🧪
npx tsx scripts/agent-say.ts 27 status "Building spec..." 📡
npx tsx scripts/agent-say.ts B error "Build failed" ❌
```

### 2. Trinity Core Backend (`backend/trinity-core/`)

Rust backend serving:
- REST API for sessions and messages
- SSE stream for real-time updates

**Endpoints:**
- `GET /health` - Health check
- `GET /events` - SSE stream
- `POST /session` - Create session
- `GET /session` - List sessions
- `POST /session/:id/message` - Create message

**Running the backend:**
```bash
cd backend/trinity-core
cargo run --release
```

Or set port via environment:
```bash
PORT=8082 cargo run --release
```

### 3. Chat UI (`apps/desktop/`)

Svelte component for displaying messages:

- `QueenTrinityChat.svelte` - Main chat component
- `web.ts` - API client with SSE support
- Agent messages shown with colored badges
- User messages aligned right
- Queen (Q) messages with gold gradient

**Running the UI:**
```bash
cd apps/desktop
pnpm install
pnpm dev
```

Visit http://localhost:5173

## Agent Colors

Each agent has a distinct badge color:

| Agent | Color | Hex |
|-------|-------|-----|
| Q (Queen) | Gold | #FFD700 |
| A | Green | #4CAF50 |
| B | Blue | #2196F3 |
| C | Purple | #9C27B0 |
| D | Deep Orange | #FF5722 |
| E | Cyan | #00BCD4 |
| F | Amber | #FFC107 |
| G | Light Green | #8BC34A |
| H | Pink | #E91E63 |
| I | Indigo | #3F51B5 |
| J | Teal | #009688 |
| K | Orange | #FF9800 |
| L | Deep Purple | #673AB7 |
| M | Brown | #795548 |
| N | Blue Grey | #607D8B |
| O | Red | #F44336 |
| P | Lime | #CDDC39 |
| R | Grey | #9E9E9E |
| S | Indigo | #3F51B5 |
| T | Light Green | #8BC34A |
| U | Yellow | #FFEB3B |
| V | Purple Accent | #E040FB |
| W | Cyan Accent | #18FFFF |
| X | Indigo Accent | #536DFE |
| Y | Orange Accent | #FF6E40 |
| Z | Green Accent | #00E676 |
| 27 (Special) | Pink | #E91E63 |

## Testing

Run the full test suite:

```bash
./scripts/test-agent-bridge.sh
```

This will:
1. Build the backend
2. Start the backend server
3. Test the agent-say CLI
4. Display results

## Environment Variables

- `TRINITY_BACKEND_URL` - Backend URL (default: http://localhost:8082)
- `PORT` - Backend port (default: 8082)

## Message Types

| Type | Default Emoji | Usage |
|------|---------------|-------|
| `message` | 💬 | General messages from agents |
| `test_result` | 🧪 | Test results and validation |
| `status` | 📡 | Status updates |
| `error` | ❌ | Error reports |

## Integration with Agents

Agents can call the CLI from their code:

```typescript
import { exec } from 'child_process';

function sendAgentMessage(agentId: string, type: string, content: string) {
  exec(`npx tsx scripts/agent-say.ts ${agentId} ${type} "${content}"`);
}
```

Or using the API directly:

```typescript
import { sendAgentMessage } from './apps/desktop/src/lib/backend/web';

await sendAgentMessage(
  sessionId,
  'A',
  'message',
  'Task complete',
  '💬'
);
```
