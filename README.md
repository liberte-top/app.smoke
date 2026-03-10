# app.smoke

Minimal business-layer sample for the liberte.top workspace.

- `api/`: Rust + Axum API that trusts upstream identity headers.
- `web/`: Vite + Svelte UI that talks to the API through same-origin `/api/`.
- `docker-compose.yml`: local bootstrap for `api + web`.

## Purpose

- Demonstrate how a downstream app can stay auth-agnostic.
- Keep auth/session/scope enforcement out of business code.
- Give Traefik and `service.auth` a concrete consumer to integrate with later.

## Local Flow

1. Copy `.env.example` to `.env` if you want custom ports.
2. Run `docker compose up -d api web`.
3. Open `http://localhost:6180`.
4. Call the API directly on `http://localhost:4310/api/v1/*` if needed.
