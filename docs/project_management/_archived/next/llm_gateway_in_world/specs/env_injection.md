# spec — llm_gateway_in_world: secret delivery (v1 legacy env injection + v1.1 FD/pipe auth bundle)

Historical evidence only. This spec preserves ADR-0023-era secret delivery planning and does not define the current operator boundary.
The live operator contract is `docs/contracts/gateway/operator-contract.md`.

Phase 8 additive upgrade: v1.1 introduces a preferred secret-channel payload + in-world FD/pipe delivery path so secret values do not live in in-world process environments by default (see DR-0018).

Authoritative inputs:
- ADR-0023: `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`
- ADR-0027: `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- Decision: `docs/project_management/_archived/next/llm_gateway_in_world/decision_register.md` (DR-0007)
- Cross-track standard: `docs/project_management/standards/SECRETS_DELIVERY_CHANNEL_RUBRIC.md`

## Requirements
- Substrate MUST NOT store secret values in:
  - `$SUBSTRATE_HOME/config.yaml`
  - `$SUBSTRATE_HOME/policy.yaml`
  - `$SUBSTRATE_HOME/agents/*.yaml`
  - workspace equivalents
- Substrate MAY store only secret *references* (env var names) in inventory/config.
- Secret values MUST be redacted from:
  - stdout/stderr
  - trace spans/events
  - structured errors
- Missing secrets MUST fail closed with actionable error messages that include the env var name(s), never their values.

## Delivery mechanism (v1)
- Note: v1.1 is the preferred mechanism once implemented; v1 remains documented as the legacy compatibility path.
- Historical command surface (v1):
  - `substrate world sync gateway`
  - `substrate world sync gateway --restart`
- The sync/restart path collects secret values from the host process environment and passes them across the existing world-agent transport as part of the spawn request.
- The world-agent spawns the gateway/engine process inside the session world with those env vars set in the process environment.
- Secrets are in-memory only from Substrate’s perspective: Substrate does not write them to disk.

## Delivery mechanism (v1.1 preferred) — secret-channel payload + in-world FD/pipe delivery

Phase 8 additive upgrade (see `docs/project_management/_archived/next/llm_gateway_in_world/decision_register.md` (DR-0018)):

- Host-side secret sourcing remains the same as v1:
  - `api:*` secrets sourced from host env var values (names-only in inventory; gated by `llm.secrets.env_allowed`).
  - `cli:*` secrets sourced from host credential reads when applicable (gated by `agents.host_credentials.read.allowed_backends`) and/or explicit env overrides if supported by the backend adapter.
- Host→world transport MUST treat secret values as a secret-channel payload on the gateway spawn request (never logged/printed).
- The world-agent MUST deliver the secret-channel payload to the in-world gateway/manager via an inherited one-time FD/pipe channel (not via secret-bearing env vars) and SHOULD do so by:
  - writing an auth bundle payload to a pipe, and
  - passing the read-end FD to the gateway/manager process.
- The gateway/manager MUST read the bundle once, close the FD promptly, and retain secrets in memory only.

## In-world propagation (gateway/manager → Substrate-spawned engines)

Even when secret values enter the in-world gateway/manager (via legacy env injection or via the v1.1 auth bundle), Substrate MUST minimize secret exposure within the in-world process tree:

- If the gateway/manager spawns a Substrate-owned backend engine/wrapper process (e.g., `cli:codex` wrapper), the gateway/manager MUST deliver secret values to that child via a one-time FD/pipe secret channel (not via child-process env vars), following:
  - `docs/project_management/standards/SECRETS_DELIVERY_CHANNEL_RUBRIC.md`
- If FD/pipe delivery is not supported for a specific engine/wrapper on a specific platform, env var delivery into that child is permitted as a compatibility fallback, but MUST remain:
  - secret-bearing (never printed),
  - redacted/capped everywhere,
  - and scoped to the smallest possible process tree.

## Env var naming (contract)
- Historical client wiring env vars (non-secret; may be printed by `substrate world status gateway`):
  - `SUBSTRATE_LLM_OPENAI_BASE_URL`
  - `SUBSTRATE_LLM_ANTHROPIC_BASE_URL`
- Auth bundle pointer env vars (non-secret; safe to print):
  - `SUBSTRATE_LLM_AUTH_BUNDLE_FD: int`
- Injected backend auth env vars (secret-bearing; MUST never be printed; MUST always be redacted/capped):
  - `SUBSTRATE_LLM_BACKEND_AUTH_<KIND>_<NAME>_<FIELD>`
  - v1 (`cli:codex`):
    - `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID`
    - `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN`
  - `api:*` backends:
    - each `api:*` agent inventory item declares required host env var *names* in `config.api.auth.env`.
    - the gateway sync/restart path collects those host env var values and injects them into the in-world gateway/engine spawn environment (values never printed/persisted).
    - injection is permitted only when effective policy allowlists the env var names via `llm.secrets.env_allowed` (deny-by-default).
    - in-world injection uses Substrate-owned injected env var names of the form:
      - `SUBSTRATE_LLM_BACKEND_AUTH_API_<NAME>_<FIELD>`
      - Example: `api:openai` reading host `OPENAI_API_KEY` injects `SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY`.

In v1.1 (preferred), `SUBSTRATE_LLM_BACKEND_AUTH_*` are still the canonical **auth field names**, but their **values** are not carried in the in-world process environment by default. Instead, the auth bundle payload read from `SUBSTRATE_LLM_AUTH_BUNDLE_FD` MUST use those same keys so redaction/caps rules remain uniform across env-based and FD/pipe-based delivery.

## Rotation / updates
- If a secret value changes, operators restart the gateway session (or re-run historical `substrate world sync gateway` if it is defined as idempotent with “replace env” semantics).
- The exact idempotency/replace semantics are implementation-defined but MUST remain fail-closed and must not leak secrets in logs.
  - Note: in v1.1 (FD/pipe bundle), rotation still requires a restart or re-sync that re-delivers a fresh bundle; the FD/pipe payload is explicitly read-once.

## “Without persisting to disk” (clarification)
- “Not persisted to disk” means Substrate does not write secret *values* into files on host or in-world storage. They exist only:
  - in the in-world gateway/engine process environment and memory (legacy v1), or in memory only (v1.1 preferred), and
  - as provided by the operator to the host process environment at invocation time.
- Note: secrets set as environment variables are typically readable by same-user processes via `/proc/<pid>/environ` on Linux; this is an OS property. v1.1 reduces this exposure by avoiding secret-bearing env vars in-world by default. The threat model for who can read process environments and process memory inside a world should be documented as part of gateway hardening.
