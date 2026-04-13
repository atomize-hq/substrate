## Archived Gateway Admin UI

This directory preserves the retired gateway admin/config web surface as reference-only source.

What it previously did:
- served the browser-based admin UI at `GET /`
- exposed `/api/config/json` and `/api/reload` for config inspection and mutation
- coordinated browser-driven provider/model editing against the gateway config file
- reused the active OAuth HTTP endpoints for browser setup flows

Why it was retired:
- the gateway no longer supports browser-driven model/provider configuration
- direct config-file editing is now the supported configuration path
- the runtime should not imply that the archived admin surface is a supported authority

What remains active:
- inference routes under `/v1/...`
- `/health`
- OAuth HTTP endpoints under `/api/oauth/...` and `/auth/callback`
- statusline, routing-history, and tracing evidence surfaces

Relevant context:
- `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `crates/gateway/README.md`

Revival conditions:
- a future decision explicitly restores browser-driven configuration as a supported runtime surface
- the active router reintroduces the admin page and UI-only config endpoints intentionally
- the archived HTML and handler logic are reviewed against current config, OAuth, and provider contracts before reuse

Contents:
- `admin.html`: the retired browser UI asset
- `server_admin_ui.rs`: the retired UI-only handlers and route wiring snippet extracted from `src/server/mod.rs`

This directory is intentionally outside the active module graph.
