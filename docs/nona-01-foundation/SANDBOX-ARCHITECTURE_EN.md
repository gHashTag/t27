# T27 Sandbox System Architecture

> **Version:** 0.1.0
> **Date:** 2026-04-04
> **Status:** PHI LOOP — SPEC phase
> **Author:** agent:perplexity-computer

---

## Table of Contents

1. [Overview](#1-overview)
2. [Architecture diagram](#2-architecture-diagram)
3. [Components](#3-components)
4. [Execution flow](#4-execution-flow)
5. [Load balancing across accounts](#5-load-balancing-across-accounts)
6. [Security model](#6-security-model)
7. [Cost analysis](#7-cost-analysis)
8. [Comparison with alternatives](#8-comparison-with-alternatives)
9. [PHI LOOP — compliance with principles](#9-phi-loop--compliance-with-principles)
10. [Technology tree](#10-technology-tree)
11. [5 unfair advantages of Trinity](#11-5-unfair-advantages-of-trinity)

---

## 1. Overview

The T27 sandbox system is an **efficient infrastructure for executing SWE-agent tasks**. Each sandbox represents an isolated container on the Railway platform, running OpenCode in web interface mode. The agent gets access to the git repository, LLM tools (Anthropic, OpenAI), and command line — all within one protected environment.

**Key properties:**

| Property | Value |
|----------|-------|
| Startup time | < 90 seconds |
| Max concurrent sessions | 100 |
| Isolation | Railway internal network |
| Authentication | Token-based (Bearer) |
| State storage | PostgreSQL (Control Plane) |
| Traffic routing | HTTP-proxy through Railway internal DNS |

The system follows the **T27 constitutional law (SOUL.md)**: each module has a `.tri` specification with tests, every change passes through PHI LOOP.

---

## 2. Architecture diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           CLIENT                                        │
│                    (browser / CLI / API client)                         │
└─────────────────────────┬───────────────────────────────────────────────┘
                          │ HTTPS
                          ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      CONTROL PLANE API                                  │
│                  (Rust / Axum, Railway Cloud)                           │
│                                                                         │
│   ┌─────────────┐  ┌──────────────┐  ┌───────────────┐                │
│   │  /sessions  │  │  /sessions/  │  │  /proxy/{name}│                │
│   │  POST / GET │  │  {id} DELETE │  │  /* (any    │                │
│   │             │  │              │  │  HTTP method)  │                │
│   │   └──────┬──────┘  └──────┬───────┘  └──────┬────────┘                │
│          │                │                 │                          │
│          └────────┬────────┘                 │                          │
│                   │                         │                          │
│   ┌───────────────▼──────────┐   ┌──────────▼──────────────────┐      │
│   │    Session Manager       │   │       Proxy Engine           │      │
│   │  (create/delete/     │   │       (name resolution →         │      │
│   │   status monitoring)    │   │   railway.internal DNS)      │      │
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
│   │   sandbox-a1          │       │   sandbox-b1          │               │
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
│         ════════════════════════════════════════                    │
│         Isolated from public internet                             │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Components

### 3.1 Sandbox Container

An isolated Docker container launched on Railway when creating a session.

**Image contents:**

```
ghcr.io/t27/sandbox:latest
├── OpenCode (latest version, mode --web)
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
|----------|-------------|
| `REPO_URL` | HTTPS URL of git repository |
| `GH_TOKEN` | GitHub token for private repos |
| `ANTHROPIC_API_KEY` | Anthropic Claude API key |
| `OPENAI_API_KEY` | OpenAI API key |
| `BRANCH` | Branch to checkout (default: main) |
| `T27_SESSION_ID` | UUID of session (for tracing) |

### 3.2 Control Plane API

REST API in Rust (Axum framework), managing session lifecycle.

**Endpoints:**

| Method | Path | Description |
|--------|-------|-------------|
| `POST` | `/sessions` | Create new session |
| `GET` | `/sessions` | List all sessions |
| `GET` | `/sessions/{id}` | Get session by ID |
| `DELETE` | `/sessions/{id}` | Delete session |
| `GET/POST/...` | `/proxy/{name}/*path` | Proxy to sandbox |
| `GET` | `/health` | Healthcheck Control Plane |

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
    └─────────┘                    └─────────────┘                 └─────────┘
```

### 3.3 Railway Integration

Interaction with Railway REST API v2 for managing services.

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

**Health polling:**

After creating Control Plane, a goroutine starts which every `HEALTH_POLL_INTERVAL` (5s) calls `http://<session_name>.railway.internal:8080/health`. On success — status changes to `Active`. On exceeding `STARTUP_TIMEOUT_MS` (90s) — changes to `Failed`.

### 3.4 OpenCode Web UI

[OpenCode](https://opencode.ai) — open-source SWE agent with web interface, running inside sandbox.

**Capabilities:**

- Code work via LLM (Claude, GPT-4o)
- Built-in terminal
- File browsing and editing
- Git operations (commit, push, PR)
- Event streaming (SSE) for progress display

**T27 Integration:**

Control Plane API proxies all HTTP requests from user directly to OpenCode, using Railway internal network (no exit to public internet).

---

## 4. Execution flow

### 4.1 Creating session (Happy Path)

```
User           Control Plane API       Railway API          Sandbox Container
     │                       │                     │                      │
     │  POST /sessions        │                     │                      │
     │  {repo_url, task, ...} │                     │                      │
     │ ──────────────────────►│                     │                      │
     │                        │ select_account()    │                      │
     │                        │ (least-loaded acct) │                      │
     │                        │                     │                      │
     │                        │ serviceCreate()     │                      │
     │                        │ ────────────────────►                      │
     │                        │                     │  Deploy container     │
     │                        │ ◄──────────────────│                      │
     │                        │ {service_id}        │                      │
     │                        │                     │                      │
     │                        │ Write Session(Starting) to DB              │
     │                        │                     │                      │
     │ 202 Accepted          │                     │      ← ~60-90s →     │
     │  {session}             │                     │        Container     │
     │ ◄──────────────────────│                     │        starts up     │
     │                        │                     │                      │
     │                        │ Poll health every 5s│        starts up     │
     │                        │──────────────────────────────────────────►│
     │                        │                     │                      │
     │                        │◄────────────────────────────────────────── │
     │                        │                     │                      │
     │                        │                     │   HTTP 200 /health   │
     │                        │                     │                      │
     │                        │◄──────────────────────────────────────────│
     │                        │                     │                      │
     │                        │ Update Session(Active) in DB               │
     │                        │                     │                      │
     │                        │                     │                      │
     │  GET /sessions/{id}    │                     │                      │
     │ ──────────────────────►│                     │                      │
     │  {status: "Active"}    │                     │                      │
     │ ◄──────────────────────│                     │                      │
```

### 4.2 Proxying requests

```
User     Control Plane API      Railway Internal Net    OpenCode
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
     │                  │ 200 {tasks: [...]}    │                  │
     │                  │                        │                  │
     │                  │ 200 {tasks}     │                        │                  │
     │                  │◄────────────────│                        │                  │
```

### 4.3 Deleting session

```
User     Control Plane API     Railway API
     │                  │                   │
     │  DELETE          │                   │
     │  /sessions/{id}  │                   │
     │ ────────────────►│                   │
     │                  │ Update(Terminating)│
     │                  │                   │
     │                  │ serviceDelete()   │
     │                  │ ──────────────────►
     │                  │ Boolean: true      │
     │                  │◄──────────────────│                   │
     │                  │                   │
     │                  │ Update(Deleted)    │
     │                  │ 200 {true}      │                   │
     │                  │◄────────────────│                   │
```

---

## 5. Load balancing across accounts

Railway has limits on number of services per account. T27 uses a **pool of accounts** with flexible selection strategy.

### Account selection algorithm

```
select_account(accounts: []RailwayAccount) -> RailwayAccount:
    1. Filter accounts at limit
    2. Find minimum active_sessions among remaining
    3. Among accounts with minimum — choose with smallest index
    4. Increment active_sessions of selected account (optimistically)
    5. Return account
```

**Example distribution (10 accounts × 10 sessions = 100 sessions):**

```
Account | Limit | Active sessions | Status
────────┼───────┼─────────────────┼─────────
   A    │ 10   │       10        │ Full
   B    │ 10   │        9        │ ✓ Selected (1 slot)
   C    │ 10   │        8        │ ✓ Available
   ...  │  ...  │       ...       │ ...
```

**Account monitoring:**

Every 60 seconds Control Plane checks `active_sessions` in memory against actual value from DB (reconciliation loop), preventing drift on failures.

---

## 6. Security model

### 6.1 Authentication and authorization

```
Incoming request
      │
      ▼
┌─────────────────────────────────────┐
│ Bearer Token Middleware              │
│                                     │
│ Authorization: Bearer <TOKEN>        │
│                                     │
│ Validation:                          │
│  • Header presence               │
│  • Match with T27_API_TOKEN (env) │
│  • Constant-time comparison         │
│    (protects from timing attacks)       │
└─────────────┬───────────────────────┘
              │ 401 Unauthorized (on mismatch)
              │ or
              ▼ continue processing
```

**Control Plane secrets (Railway env vars):**

| Variable | Type | Rotation |
|----------|-----|---------|
| `T27_API_TOKEN` | Random UUID v4 | Rotate on compromise |
| `RAILWAY_TOKEN_A..N` | Railway API tokens | Quarterly rotation |
| `DATABASE_URL` | PostgreSQL connection string | On password change |

### 6.2 Network isolation

```
Public internet
       │
       │ HTTPS (only through Control Plane proxy)
       ▼
┌─────────────────┐
│ Control Plane  │
│ (public URL)   │
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

**Isolation guarantees:**
- Sandbox containers **have no public URL** — only accessible via proxy
- Railway internal network is isolated from other projects/accounts
- Each sandbox has only its own API keys (not shared)
- Git operations use one-time token (not persistent credentials)

### 6.3 Sandbox resource limits

```
Sandbox container:
  CPU:     2 vCPU (burst to 4)
  RAM:     2 GB
  Disk:    10 GB (ephemeral, deleted on stop)
  Network:  1 Gbps (Railway internal), limited egress
  Time:   TTL not set (managed by Control Plane)
```

---

## 7. Cost analysis

### 7.1 Railway Pricing (2026)

| Resource | Price |
|---------|-------|
| vCPU | $0.000463/minute |
| RAM | $0.000231/minute per 512 MB |
| Egress | $0.10/GB |

### 7.2 Cost per session

```
Configuration: 2 vCPU, 2 GB RAM

Cost per minute:
  CPU: 2 × $0.000463 = $0.000926/minute
  RAM: 4 × $0.000231 = $0.000924/minute
  Total ≈ $0.00185/minute ≈ $0.111/hour

30-minute session (typical task):
  ≈ $0.055 per session

100 sessions × 8 hours/day × 30 days:
  ≈ $2,664/month (at 100% utilization)
  ≈ $266/month (at 10% utilization — realistic for MVP)
```

### 7.3 Comparison of payment models

| Approach | Cost/month (MVP) | Cost/month (scale) |
|----------|------------------|------------------|
| T27 Railway (pay-as-you-go) | ~$50-300 | ~$2,000-10,000 |
| E2B (managed sandboxes) | ~$200 | ~$5,000+ |
| Dedicated VMs (EC2 t3.medium) | ~$500 (fixed) | ~$5,000+ |
| Modal | ~$100-500 | ~$3,000+ |

---

## 8. Comparison with alternatives

| Criterion | T27 Railway | E2B | Modal | Fly.io | Local Docker |
|----------|--------------|-----|-------|------------|
| **Startup time** | 60-90 s | ~500 ms | ~1-3 s | 10-30 s | ~5 s |
| **Isolation** | ✓ Full | ✓ Full | ✓ Full | ✓ Full | ✗ Host-network |
| **Scaling** | 100+ | 1000+ | 1000+ | 100+ | Limited |
| **Image control** | ✓ Full | Partial | Partial | ✓ Full | ✓ Full |
| **Vendor lock-in** | Medium | High | High | Medium | None |
| **GPU support** | ✗ | ✓ | ✓ | ✓ | Depends |
| **Cost (MVP)** | ★★★★★ | ★★★ | ★★★★ | ★★★★ | ★★★★★ |
| **OpenCode integration** | ✓ Native | Custom | Custom | Custom | ✓ Native |
| **Multi-account pool** | ✓ Built-in | ✗ | ✗ | ✗ | N/A |
| **PHI LOOP compatibility** | ✓ | ✗ | ✗ | ✗ | ✗ |

**Why Railway for T27:**

1. **Simple deployment**: Railway CLI + Dockerfile = working service in minutes
2. **Internal network**: Built-in isolated network without VPC configuration
3. **GraphQL API**: Full lifecycle control from code
4. **Pay-as-you-go**: No minimum bill — ideal for MVP

**Drawbacks of Railway and how T27 compensates:**

| Issue | Compensation |
|--------|--------------|
| Slow startup (60-90 s) | Pre-warming pool (TODO: phase 3) |
| Account limit | Multi-account pool with load balancing |
| No GPU | Inference via API (not local) |

---

## 9. PHI LOOP — compliance with principles

PHI LOOP is a continuous improvement cycle in T27:

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

**Compliance status with SOUL.md:**

| Requirement | Status |
|-----------|--------|
| Spec before code | ✓ `sandbox.tri` created |
| Tests in spec | ✓ 14 tests in `.tri` |
| Episode json | ✓ `sandbox-init.json` created |
| Invariants | ✓ 5 invariants defined |
| Benchmarks | ✓ 4 benchmarks defined |

---

## 10. Technology tree

*(Detailed tree — in `TECHNOLOGY-TREE.md`)*

```
ring 17: CANOPY (current state)
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

## 11. 5 unfair advantages of Trinity

### 1. PHI LOOP as built-in CI/CD of intelligence

Competitors (E2B, Modal) provide infrastructure, but **don't have built-in improvement cycles**. T27 PHI LOOP ensures every change passes through `spec → gen → test → verdict` — agent is **required** to prove that its changes improve the system before they're committed.

### 2. Multi-Account Pool without single point of failure

Competitors use a single account/token. T17 originally designed a **horizontal pool** of Railway-accounts with least-connections load balancing. Even if one account reaches its limit or gets blocked — the system continues operating.

### 3. Railway Internal Network as free VPC

E2B and Modal require separate private network configuration. Railway provides `*.railway.internal` DNS **for free** within the project — all sandbox containers are isolated from the internet without additional costs for VPC, NAT Gateway, or PrivateLink.

### 4. .tri Specification as Single Source of Truth

Code, tests, and documentation can diverge. In T17, the `.tri` file is the **single source of truth** — from it the test scaffold, API documentation, and contracts between services are generated. This eliminates the classic "documentation is stale" error completely.

### 5. Experience Episodes as long-term agent memory

Each PHI LOOP cycle writes `episode.json` with hashes of spec, gen, test results, and verdict. Over time, the system accumulates **computational evolutionary history** — the agent can analyze which changes improved metrics in the past, and apply those patterns to new tasks. Competitors have nothing comparable.
