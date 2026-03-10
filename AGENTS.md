# App.Smoke AGENTS Guide

## Current Flow
- This repository is a minimal business-layer sample.
- Runtime stack is `docker-compose` with `api + web`.
- The API trusts gateway-provided identity headers instead of implementing login/session flows.
- The web app uses same-origin `/api/` calls through nginx proxying.

## Repository Structure

```text
app.smoke/
├── api/                      # Rust + Axum sample business API
│   ├── src/
│   ├── Dockerfile
│   └── Cargo.toml
├── web/                      # Vite + Svelte sample business web
│   ├── src/
│   ├── docker/
│   ├── scripts/
│   ├── Dockerfile
│   └── package.json
├── docker-compose.yml        # Local stack orchestration
├── .env(.example)            # Local runtime parameters
└── AGENTS.md
```

## Runtime Parameters
- `API_PORT`
- `WEB_PUBLIC_PORT`
- `WEB_VITE_ENV_LABEL`

## Execution Entry
- `docker compose up -d api web`
- `cd api && cargo test --locked`
- `cd web && pnpm install --frozen-lockfile && pnpm build`

## Common Commands
- `docker compose up -d api web`
- `docker compose ps`
- `docker compose logs -f api`
- `cd api && cargo test --locked`
- `cd web && pnpm build`

## Change Policy
- Keep auth enforcement out of this repo; treat gateway headers as the identity contract.
- Keep the API small and explicit so it remains a sample, not a hidden platform.
- Keep the web app same-origin and free of environment-specific API host logic.
