# Background Agent on Railway

This is the companion repo for ["I Built a Remote Coding Agent Platform on Railway (OpenCode, Claude Code, Codex)
"](https://youtu.be/A-beOnncri8) (a video I created in partnership with [Railway](https://railway.com?referralCode=P06La2&utm_medium=social&utm_source=youtube&utm_campaign=sid)).

[![](./readme-assets/thumbnail.png)](https://youtu.be/A-beOnncri8)

It is a full-stack demo that provisions AI agents into sandbox sessions running on Railway. (inspired by [Ramp's internal agent "Inspect"](https://builders.ramp.com/post/why-we-built-our-background-agent))

## Deployment Template!

[![Deploy on Railway](https://railway.com/button.svg)](https://railway.com/deploy/background-agent?referralCode=P06La2&utm_medium=social&utm_source=youtube&utm_campaign=sid)

## Video Progression:

1. Run opencode locally
2. Run opencode inside a container
3. Deploy to Railway
4. Add code-server to container image
5. Use the Railway API to deploy
6. Create a custom API for the control plane
7. Add a frontend (+ add proxy) 
8. Install other coding agents
9. Configure GitHub access and pre-clone the repo

## Development Notes

### Project structure

- `packages/api`: Express + TypeScript API, Railway integration, session management, and proxying.
- `packages/web`: React + Vite frontend.
- `packages/sandbox`: Sandbox container image used by session environments.
- `docker-compose.yml`: Local API + Postgres + sandbox containers for development.

### Prerequisites

- [Mise](https://mise.jdx.dev/)
- Docker + Docker Compose

### Local development

1. Start local infra:

   ```bash
   docker compose up -d
   ```

2. Configure API environment variables:

   ```bash
   cp packages/api/.env.example packages/api/.env
   ```

3. Install dependencies:

   ```bash
   pnpm install --dir packages/api
   pnpm install --dir packages/web
   ```

4. Run API and web app in separate terminals:

   ```bash
   pnpm --dir packages/api dev
   pnpm --dir packages/web dev
   ```

The API runs on `http://localhost:3000` and the web app runs on `http://localhost:5173`.

### Useful API commands

```bash
pnpm --dir packages/api db:generate
pnpm --dir packages/api db:migrate
pnpm --dir packages/api build
pnpm --dir packages/api start
```

### Useful web commands

```bash
pnpm --dir packages/web build
pnpm --dir packages/web preview
```
