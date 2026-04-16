# ChatGPT Codex Auth Handoff Contract

This document is the descriptive source of truth for the gateway's ChatGPT Codex auth-handoff boundary.
It freezes how integrated and standalone modes deliver and resolve `account_id` plus access-token material for the Codex route without turning gateway-local persistence into the owner of host credentials.

## Scope

This contract covers:

- the integrated-mode owner line between Substrate host preflight, secret delivery, and in-world gateway consumption
- the bounded standalone compatibility path for local Codex auth material
- the closed `cli:codex` auth field set the gateway consumes
- account-id resolution order, explicit-over-JWT precedence, and the JWT claim path used only as bounded fallback
- provider-side `ChatGPT-Account-ID` injection and pre-upstream failure posture

This contract does not own:

- the OAuth browser flow, token refresh UX, or provider registration details
- the route compatibility matrix for `backend-api/codex/responses`
- a generic redesign of gateway-wide token persistence outside this route-specific auth boundary

## Integrated-Mode Owner Line

- Substrate owns policy-gated host credential reads, auth-state validation, and host-to-world delivery for the ChatGPT Codex auth material required by this route.
- The gateway bootstrap selects integrated versus standalone auth mode before provider construction; provider code consumes the selected auth source and does not become the owner of host credential reads or trust-boundary decisions.
- Current v1 integrated delivery may place secret-bearing values for the closed `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_*` field set directly in the in-world gateway/manager process environment.
- That v1 env-based delivery remains subordinate to the Substrate-owned trust boundary above: host credential reads stay policy-gated, values are not authoritative because they are in env vars, and the gateway remains only a consumer of the delivered auth context.
- The preferred additive direction remains a secret-channel payload plus inherited FD/pipe-style auth-bundle delivery so secret values do not live in the in-world process environment by default.
- The canonical integrated-mode field identifiers are:
  - `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID`
  - `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN`

Runtime bootstrap note:

- The effective `llm.gateway.mode` posture is applied outside the provider boundary.
- Current gateway bootstrap may carry that posture via `SUBSTRATE_LLM_GATEWAY_MODE=in_world|host_only`.
- `in_world` selects the integrated auth source for this route.
- `host_only` selects the bounded standalone compatibility source for gateway-local operation.

The gateway must treat those field names as the authoritative integrated-mode inputs for this route, whether the current runtime receives their values through v1 env delivery or a later auth-bundle carrier.

## Standalone Compatibility Mode

- Standalone mode exists only for gateway-local operation outside a Substrate-managed in-world deployment.
- Standalone mode may resolve local Codex auth state, for example `~/.codex/auth.json` or an equivalent local auth-context carrier for the same Codex account.
- The local gateway OAuth token store at `~/.substrate-gateway/oauth_tokens.json` may carry token material for local OAuth flows, but it is not authoritative account-id storage on its own because the current schema stores access and refresh tokens without a first-class `account_id`.
- Standalone compatibility must remain subordinate to the integrated owner line above; it must not redefine who owns account identity in integrated deployment.

## Resolution And Precedence Rules

The selected mode determines which auth context is consulted first.

### Integrated mode

1. Use explicit `account_id` from the Substrate-delivered auth context.
2. If explicit `account_id` is absent, use the JWT-derived fallback from the same Substrate-delivered OAuth access token.
3. If neither source yields an account id, fail before the upstream call.

### Standalone mode

1. Use explicit `account_id` from local Codex auth state.
2. If explicit `account_id` is absent, use the JWT-derived fallback from the same local OAuth access token represented by that auth state.
3. If neither source yields an account id, fail before the upstream call.

### Shared precedence constraints

- Explicit `account_id` always wins over the JWT-derived value when both are present.
- JWT extraction is compatibility fallback only. It does not define ownership.
- The JWT fallback reads the `chatgpt_account_id` field inside the `https://api.openai.com/auth` object in the OAuth access token payload currently handled by the provider adapter.
- Integrated mode must not require direct reads of `~/.codex/auth.json` or other host-local auth files inside the in-world gateway runtime.

## Provider Consumption Contract

- The provider request builder consumes resolved auth context and injects:
  - `Authorization: Bearer <access_token>`
  - `ChatGPT-Account-ID: <resolved account_id>`
- The provider path remains a consumer of resolved auth context. Auth-source selection is explicit and external to the provider.
- Provider code must not perform host-local auth-file reads for integrated mode.
- A JWT-only helper may remain as an implementation detail for bounded fallback, but only behind the resolution order above.

## Failure Posture

- If the selected mode cannot resolve any valid `account_id`, the gateway fails before the upstream call.
- That failure uses the normal gateway auth error envelope rather than a transport retry or partial upstream attempt.
- Route execution must not silently drop the `ChatGPT-Account-ID` header, substitute a guessed value, or defer the failure until the upstream request returns.

## Verification Anchors

Implementation and regression evidence for this contract should land against:

- `crates/gateway/src/auth/token_store.rs`
- `crates/gateway/src/auth/oauth.rs`
- `crates/gateway/src/server/oauth_handlers.rs`
- `crates/gateway/src/providers/openai.rs`
- `crates/gateway/src/server/mod.rs`
- `crates/gateway/docs/OAUTH_SETUP.md`
- `crates/gateway/docs/OAUTH_TESTING.md`

Required verification additions for this contract:

- `crates/gateway/src/providers/openai.rs`
  - `codex_oauth_request_builder_prefers_explicit_account_id_over_jwt_fallback`
  - `codex_oauth_request_builder_uses_jwt_fallback_only_when_explicit_account_id_is_absent`
  - `codex_oauth_request_builder_fails_before_upstream_when_account_id_is_unresolvable`
- integration or handler coverage proving the integrated path does not require local auth-file reads to establish `ChatGPT-Account-ID`
- documentation updates that distinguish standalone local OAuth/token material from the integrated Substrate-owned handoff boundary

Verification passes only when:

- explicit `account_id` wins whenever both explicit and JWT-derived values exist
- JWT fallback is used only when the explicit field for the selected mode is absent
- unresolved identity fails before any upstream request is sent and remains classified as an auth failure
- integrated operation no longer depends on gateway-local token persistence or host-local auth files to establish `ChatGPT-Account-ID`
