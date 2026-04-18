# T27 Sandbox System Architecture

> **Version:** 0.1.0
> **Date:** 2026-04-04
> **Status:** PHI LOOP — SPEC phase
> **Actor:** agent:perplexity-computer

---

## Table of Contents

1. [Overview](#1-overview)
2. [Architecture Diagram](#2-architecture-diagram)
3. [Components](#3-components)
4. [Execution Flow](#4-execution-flow)
5. [Account Load Balancing](#5-account-load-balancing)
6. [Security Model](#6-security-model)
7. [Cost Analysis](#7-cost-analysis)
8. [Comparison with Alternatives](#8-comparison-with-alternatives)
9. [PHI LOOP — Principle Compliance](#9-phi-loop--principle-compliance)
10. [Technology Tree](#10-technology-tree)
11. [5 Unfair Advantages of Trinity](#11-5-unfair-advantages-of-trinity)

---

## 1. Overview

The T27 Sandbox System is an **ephemeral infrastructure for SWE agent task execution**. Each sandbox is an isolated container on the Railway platform, running OpenCode in web interface mode. The agent gains access to a git repository, LLM tools (Anthropic, OpenAI), and command line — all in a single secure environment.

**Key Properties:**

| Property | Value |
|---|---|
| Startup Time | < 90 seconds |
| Max Concurrent Sessions | 100 |
| Isolation | Railway internal network |
| Authentication | Token-based (Bearer) |
| State Storage | PostgreSQL (Control Plane) |
| Traffic Routing | HTTP-proxy via Railway internal DNS |

The system follows the **T27 constitutional law (SOUL.md)**: every module has a `.tri` specification with tests, and every change goes through PHI LOOP.

---

## 2. Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           USER                                          │
│                    (browser / CLI / API client)                         │
└─────────────────────────┬───────────────────────────────────────────────┘
                          │  HTTPS
                          ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      CONTROL PLANE API                                  │
│                  (Rust / Axum, Railway Cloud)                           │
│                                                                         │
│   ┌─────────────┐  ┌──────────────┐  ┌───────────────┐                │
│   │  /sessions  │  │  /sessions/  │  │  /proxy/{name}│                │
│   │  POST / GET │  │  {id} DELETE │  │  /* (any       │                │
│   │             │  │              │  │  HTTP method)  │                │
│   └──────┬──────┘  └──────┬───────┘  └──────┬────────┘                │
│          │                │                 │                          │
│          └────────┬────────┘                 │                          │
│                   │                         │                          │
│   ┌───────────────▼──────────┐   ┌──────────▼──────────────────┐      │
│   │    Session Manager       │   │       Proxy Engine           │      │
│   │  (create/delete/         │   │  (name resolution →          │      │
│   │   status monitoring)     │   │   railway.internal DNS)      │      │
│   └───────────────┬──────────┘   └─────────────────────────────┘      │
│                   │                                                     │
│   ┌───────────────▼──────────┐   ┌─────────────────────────────┐      │
│   │   Railway Account Pool   │   │       PostgreSQL DB           │      │
│   │  [token_A] [token_B] ... │   │   (sessions, accounts,       │      │
│   │   round-robin balancer   │   │    audit log, episodes)      │      │
│   └───────────────┬──────────┘   └─────────────────────────────┘      │
└───────────────────┼─────────────────────────────────────────────────────┘
                    │  Railway API (HTTPS)
                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         RAILWAY CLOUD                                   │
│                                                                         │
│   Account A                     Account B                              │
│   ┌─────────────────────┐       ┌─────────────────────┐               │
│   │ sandbox-a1          │       │ sandbox-b1          │               │
│   │ ┌─────────────────┐ │       │ ┌─────────────────┐ │               │
│   │ │ OpenCode WebUI  │ │       │ │ OpenCode WebUI  │ │               │
│   │ │ :8080           │ │       │ │ :8080           │ │               │
│   │ │                 │ │       │ │                 │ │               │
│   │ │ git clone repo  │ │       │ │ git clone repo  │ │               │
│   │ │ + LLM tools     │ │       │ │ + LLM tools     │ │               │
│   │ └─────────────────┘ │       │ └─────────────────┘ │               │
│   │                     │       │                     │               │
│   │ sandbox-a2  ...     │       │ sandbox-b2  ...     │               │
│   └─────────────────────┘       └─────────────────────┘               │
│                                                                         │
│         Railway Internal Network (*.railway.internal)                  │
│         ════════════════════════════════════════════                    │
│         Isolated from public internet                                    │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Components

### 3.1 Sandbox Container

An isolated Docker container launched on Railway when a session is created.

**Image Contents:**

```
ghcr.io/t27/sandbox:latest
├── OpenCode (latest, --web mode)
├── git, curl, ripgrep, fd
├── Node.js 22 LTS + pnpm
├── Python 3.12 + pip + uv
├── Go 1.23
├── Rust 1.78 (toolchain)
└── Entrypoint: /app/start.sh
```

**Entrypoint (`start.sh`):**

```bash
#!/bin/bash
set -euo pipefail

# Clone repository
if [ -n "${REPO_URL:-}" ]; then
  git clone --depth=1 --branch "${BRANCH:-main}" \
    "https://${GH_TOKEN}@${REPO_URL#https://}" /workspace
fi

# Launch OpenCode in web mode
exec opencode --web --port 8080 --dir /workspace
```

**Environment variables injected by Control Plane:**

| Variable | Description |
|---|---|
| `REPO_URL` | HTTPS URL of git repository |
| `GH_TOKEN` | GitHub token for private repos |
| `ANTHROPIC_API_KEY` | Anthropic Claude key |
| `OPENAI_API_KEY` | OpenAI key |
| `BRANCH` | Branch to checkout (default: main) |
| `T27_SESSION_ID` | Session UUID (for tracing) |

### 3.2 Control Plane API

REST API in Rust (Axum framework) managing session lifecycle.

**Endpoints:**

| Method | Path | Description |
|---|---|---|
| `POST` | `/sessions` | Create new session |
| `GET` | `/sessions` | List all sessions |
| `GET` | `/sessions/{id}` | Get session by ID |
| `DELETE` | `/sessions/{id}` | Delete session |
| `GET/POST/...` | `/proxy/{name}/*path` | Proxy to sandbox |
| `GET` | `/health` | Control Plane healthcheck |

**Session states and transitions:**

```
   create_session()
         │
         ▼
    ┌─────────┐    health OK     ┌────────┐
    │Starting │ ─────────────►  │ Active │
    └─────────┘                  └───┬────┘
         │                          │
         │ timeout > 90s             │ delete_session()
         ▼                          ▼
    ┌────────┐   delete_session() ┌─────────────┐   Railway done  ┌─────────┐
    │ Failed │ ─────────────────► │ Terminating │ ──────────────► │ Deleted │
    └────────┘                    └─────────────┘                 └─────────┘
```

### 3.3 Railway Integration

Interaction with Railway REST API v2 for service management.

**Operations:**

```rust
// Create service
POST https://backboard.railway.com/graphql/v2
Mutation: serviceCreate(input: ServiceCreateInput) -> Service

// Set environment variables
Mutation: variableCollectionUpsert(input: VariableCollectionUpsertInput)

// Deploy (apply configuration)
Mutation: serviceInstanceRedeploy(serviceId: String)

// Delete service
Mutation: serviceDelete(id: String) -> Boolean
```

**Health Polling:**

After creation, Control Plane spawns a goroutine that queries `http://<session_name>.railway.internal:8080/health` every `HEALTH_POLL_INTERVAL` (5s). On success — status transitions to `Active`. After `STARTUP_TIMEOUT_MS` (90s) — transitions to `Failed`.

### 3.4 OpenCode Web UI

[OpenCode](https://opencode.ai) — open-source SWE agent with web interface, running inside sandbox.

**Capabilities:**

- Code work via LLM (Claude, GPT-4o)
- Built-in terminal
- File viewing and editing
- Git operations (commit, push, PR)
- Server-Sent Events (SSE) for progress streaming

**T27 Integration:**

Control Plane API proxies all user HTTP requests directly to OpenCode, using Railway internal network (no public internet exit).

---

## 4. Execution Flow

### 4.1 Session Creation (Happy Path)

```
User                 Control Plane API        Railway API          Sandbox Container
     │                       │                     │                      │
     │  POST /sessions        │                     │                      │
     │  {repo_url, task, ...} │                     │                      │
     │ ──────────────────────►│                     │                      │
     │                        │ select_account()    │                      │
     │                        │ (least-loaded acct) │                      │
     │                        │                     │                      │
     │                        │ serviceCreate()     │                      │
     │                        │ ────────────────────►                      │
     │                        │                     │ Deploy container     │
     │                        │ ◄───────────────────│                      │
     │                        │ {service_id}        │                      │
     │                        │                     │                      │
     │                        │ Write Session(Starting) to DB              │
     │                        │                     │                      │
     │  202 Accepted          │                     │      ← ~60-90s →     │
     │  {session}             │                     │                      │
     │ ◄──────────────────────│                     │        Container     │
     │                        │ Poll health every 5s│        starts up     │
     │                        │──────────────────────────────────────────►│
     │                        │                     │   HTTP 200 /health   │
     │                        │◄──────────────────────────────────────────│
     │                        │                     │                      │
     │                        │ Update Session(Active) in DB               │
     │                        │                     │                      │
     │  GET /sessions/{id}    │                     │                      │
     │ ──────────────────────►│                     │                      │
     │  {status: "Active"}    │                     │                      │
     │ ◄──────────────────────│                     │                      │
```

### 4.2 Request Proxying

```
User              Control Plane API       Railway Internal Net    OpenCode
     │                  │                        │                  │
     │  GET /proxy/     │                        │                  │
     │  sandbox-abc/    │                        │                  │
     │  api/tasks       │                        │                  │
     │ ────────────────►│                        │                  │
     │                  │ Resolve session name   │                  │
     │                  │ → sandbox-abc          │                  │
     │                  │                        │                  │
     │                  │ GET http://sandbox-abc.railway.internal:8080/api/tasks
     │                  │ ───────────────────────────────────────────►│
     │                  │                        │                  │
     │                  │◄────────────────────────────────────────── │
     │                  │  200 {tasks: [...]}    │                  │
     │                  │                        │                  │
     │  200 {tasks}     │                        │                  │
     │ ◄────────────────│                        │                  │
```

### 4.3 Session Deletion

```
User              Control Plane API     Railway API
     │                  │                   │
     │  DELETE          │                   │
     │  /sessions/{id}  │                   │
     │ ────────────────►│                   │
     │                  │ Update(Terminating)│
     │                  │                   │
     │                  │ serviceDelete()   │
     │                  │ ──────────────────►
     │                  │ Boolean: true      │
     │                  │◄──────────────────│
     │                  │                   │
     │                  │ Update(Deleted)    │
     │  200 {true}      │                   │
     │ ◄────────────────│                   │
```

---

## 5. Account Load Balancing

Railway has service limits per account. T27 uses an **account pool** with a hybrid selection strategy.

### Account Selection Algorithm

```
select_account(accounts: []RailwayAccount) -> RailwayAccount:
    1. Filter accounts at limit
    2. Find minimum active_sessions among remaining
    3. Among accounts with minimum — choose smallest index
    4. Increment active_sessions of selected account (optimistically)
    5. Return account
```

**Distribution Example (10 accounts × 10 sessions = 100 sessions):**

```
Account │ Limit │ Active Sessions │ Status
────────┼───────┼─────────────────┼─────────
   A    │  10   │       10        │ Full
   B    │  10   │        9        │ ✓ Selected (1 slot)
   C    │  10   │        8        │ ✓ Available
   ...  │  ...  │       ...       │ ...
```

**Account Monitoring:**

Every 60 seconds, Control Plane reconciles `active_sessions` in memory with actual DB values, preventing drift on failures.

---

## 6. Security Model

### 6.1 Authentication and Authorization

```
Incoming Request
      │
      ▼
┌─────────────────────────────────────┐
│ Bearer Token Middleware              │
│                                     │
│ Authorization: Bearer <TOKEN>        │
│                                     │
│ Validation:                          │
│  • Header presence                  │
│  • Match T27_API_TOKEN (env)        │
│  • Constant-time comparison         │
│    (timing attack protection)       │
└─────────────┬───────────────────────┘
              │ 401 Unauthorized (if no match)
              │ or
              ▼ continue processing
```

**Control Plane Secrets (Railway env vars):**

| Variable | Type | Rotation |
|---|---|---|
| `T27_API_TOKEN` | Random UUID v4 | Manual, on compromise |
| `RAILWAY_TOKEN_A..N` | Railway API tokens | Quarterly |
| `DATABASE_URL` | PostgreSQL connection string | On password change |

### 6.2 Network Isolation

```
Public Internet
       │
       │ HTTPS (only through Control Plane proxy)
       ▼
┌─────────────────┐
│  Control Plane  │
│  (public URL)   │
└────────┬────────┘
         │ railway.internal (isolated network)
         │ NO direct public access to sandbox
         ▼
┌─────────────────────────────────────┐
│   Railway Internal Network          │
│                                     │
│   sandbox-abc.railway.internal:8080 │
│   sandbox-def.railway.internal:8080 │
│   ...                               │
│                                     │
│   postgres.railway.internal:5432    │
└─────────────────────────────────────┘
```

**Isolation Guarantees:**
- Sandbox containers have **no public URL** — accessible only via proxy
- Railway internal network isolated from other projects/accounts
- Each sandbox has its own API keys (not shared)
- Git operations use one-time token (not persistent credentials)

### 6.3 Sandbox Resource Limits

```
Sandbox container:
  CPU:     2 vCPU (burst to 4)
  RAM:     2 GB
  Disk:    10 GB (ephemeral, deleted on stop)
  Network: 1 Gbps (Railway internal), limited egress
  Time:    No TTL set (managed by Control Plane)
```

---

## 7. Cost Analysis

### 7.1 Railway Pricing (2026)

| Resource | Price |
|---|---|
| vCPU | $0.000463/min |
| RAM | $0.000231/min per 512 MB |
| Egress | $0.10/GB |

### 7.2 Cost Per Session

```
Configuration: 2 vCPU, 2 GB RAM

Cost per minute:
  CPU:  2 × $0.000463 = $0.000926/min
  RAM:  4 × $0.000231 = $0.000924/min
  Total ≈ $0.00185/min ≈ $0.111/hour

30-minute session (typical task):
  ≈ $0.055 per session

100 sessions × 8 hours/day × 30 days:
  ≈ $2,664/month (at 100% utilization)
  ≈ $266/month (at 10% utilization — realistic for MVP)
```

### 7.3 Pricing Model Comparison

| Approach | Cost/Month (MVP) | Cost/Month (scale) |
|---|---|---|
| T27 Railway (pay-as-you-go) | ~$50-300 | ~$2,000-10,000 |
| E2B (managed sandboxes) | ~$200 | ~$5,000+ |
| Dedicated VMs (EC2 t3.medium) | ~$500 (fixed) | ~$5,000+ |
| Modal | ~$100-500 | ~$3,000+ |

---

## 8. Comparison with Alternatives

| Criterion | T27 Railway | E2B | Modal | Fly.io | Local Docker |
|---|---|---|---|---|---|
| **Startup Time** | 60-90 s | ~500 ms | ~1-3 s | 10-30 s | ~5 s |
| **Isolation** | ✓ Full | ✓ Full | ✓ Full | ✓ Full | ✗ Host network |
| **Scaling** | 100+ | 1000+ | 1000+ | 100+ | Limited |
| **Image Control** | ✓ Full | Partial | Partial | ✓ Full | ✓ Full |
| **Vendor Lock-in** | Medium | High | High | Medium | None |
| **GPU Support** | ✗ | ✗ | ✓ | ✓ | Depends |
| **Cost (MVP)** | ★★★★★ | ★★★ | ★★★★ | ★★★★ | ★★★★★ |
| **OpenCode Integration** | ✓ Native | Custom | Custom | Custom | ✓ Native |
| **Multi-account Pool** | ✓ Built-in | ✗ | ✗ | ✗ | N/A |
| **PHI LOOP Compatibility** | ✓ | ✗ | ✗ | ✗ | ✗ |

**Why Railway for T27:**

1. **Simple deployment**: Railway CLI + Dockerfile = working service in minutes
2. **Internal network**: Built-in isolated network without VPC configuration
3. **GraphQL API**: Full control over lifecycle from code
4. **Pay-as-you-go**: No minimum charge — ideal for MVP
5. **Transparency**: No proprietary runtime — only Docker

**Railway Drawbacks and How T27 Compensates:**

| Problem | Compensation |
|---|---|
| Slow startup (60-90 s) | Pre-warming pool (TODO: phase 3) |
| Account limit | Multi-account pool with balancing |
| No GPU | Inference via API (not local) |

---

## 9. PHI LOOP — Principle Compliance

PHI LOOP is T27's continuous improvement cycle:

```
    ┌─────────────────────────────────────────────────────┐
    │                                                      │
    │   SPEC ──► GEN ──► TEST ──► VERDICT ──► (new cycle)│
    │    │         │        │          │                   │
    │    │         │        │          └──► experience/    │
    │    │         │        │               episodes/      │
    │    │         │        │               *.json          │
    │    │         │        └──► pytest / cargo test        │
    │    │         └──► Rust/TypeScript code                 │
    │    └──► sandbox.tri (this file)                      │
    │                                                      │
    └─────────────────────────────────────────────────────┘
```

**SOUL.md Compliance Status:**

| Requirement | Status |
|---|---|
| Spec before code | ✓ `sandbox.tri` created |
| Tests in spec | ✓ 14 tests in `.tri` |
| Episode json | ✓ `sandbox-init.json` created |
| Invariants | ✓ 5 invariants defined |
| Benchmarks | ✓ 4 benchmarks defined |

---

## 10. Technology Tree

*(Detailed tree in `TECHNOLOGY-TREE.md`)*

```
Ring 17: CANOPY (current state)
    │
    ├── Phase 1: Sandbox Infrastructure  ← WE ARE HERE
    │     ├── Railway Integration (API client)
    │     ├── Container Loader (Dockerfile)
    │     ├── Health Check Engine
    │     └── PostgreSQL Session Store
    │
    ├── Phase 2: SWE Agent
    │     ├── OpenCode Integration
    │     ├── Task Management System
    │     └── Experience Recorder
    │
    ├── Phase 3: Swarm Intelligence
    │     ├── Multi-Agent Collaboration
    │     └── Shared Experience Pool
    │
    └── Phase 4: Evolution
          ├── ASHA Strategy Optimizer
          ├── PBT Agent Training
          └── Predictive Agent S
```

---

## 11. 5 Unfair Advantages of Trinity

### 1. PHI LOOP as Built-in CI/CD for Reason

Competitors (E2B, Modal) provide infrastructure but **lack a built-in improvement cycle**. T27 PHI LOOP ensures every change passes through `spec → gen → test → verdict` — the agent is **literally required** to prove that their changes improve the system before they are committed.

### 2. Multi-Account Pool with No Single Point of Failure

Competitors use a single account/token. T23 designs a **horizontal pool** of Railway accounts with least-connections balancing from day one. Even if one account hits its limit or is blocked — the system continues operating.

### 3. Railway Internal Network as Free VPC

E2B and Modal require separate private network configuration. Railway provides `*.railway.internal` DNS **free** within the project — all sandbox containers are isolated from the internet without additional VPC, NAT Gateway, or PrivateLink costs.

### 4. .tri Specification as Single Source of Truth

Code, tests, and documentation can drift. In T27, the `.tri` file is the **single source of truth** — from it, test scaffolds, API documentation, and service contracts are generated. This eliminates the "documentation is stale" class of errors entirely.

### 5. Experience Episodes as Agent Long-Term Memory

Every PHI LOOP cycle records an `episode.json` with spec hashes, gen hashes, test results, and verdict. Over time, the system builds a **computable evolution history** — the agent can analyze which past changes improved metrics and apply those patterns to new tasks. Competitors have nothing like this.
