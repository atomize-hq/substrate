# Decision Register — llm_gateway_in_world

Standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

Scope:
- This decision register supports `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`.
- Each decision is recorded as exactly two viable options (A/B) with explicit tradeoffs and a single selection.

---

### DR-0001 — In-world bind/transport: UDS proxy vs loopback TCP inside world

**Decision owner(s):** Shell + World-agent + Gateway maintainers  
**Date:** 2026-02-08  
**Status:** Accepted  
**Related docs:** ADR-0023, ADR-0027

**Problem / Context**
- The gateway must run “in world” by default, but host-side CLIs and tools still need a stable way to reach it.

**Option A — World-agent proxies gateway over existing transport (recommended)**
- **Pros:** No new host listener; avoids “host gateway” confusion; single audited transport path.
- **Cons:** Requires HTTP proxying support in world-agent/shell plumbing; more moving parts.

**Option B — Host loopback forwarder (host listens on 127.0.0.1 and forwards into world)**
- **Pros:** Simple client compatibility; easy for tools expecting `http://127.0.0.1:<port>`.
- **Cons:** Must be documented carefully as transport-only; risk of being mistaken for a host-level egress gateway.

**Recommendation**
- **Selected:** Option A — World-agent proxies gateway over existing transport.
- **Rationale (crisp):** Preserves “in-world” semantics without introducing a host listener that can be mistaken for a host-level egress gateway; reuses the already-audited world-agent transport path across platforms.

---

### DR-0002 — Default request/response logging: metadata-only vs body logging

**Decision owner(s):** Gateway maintainers + Security  
**Date:** 2026-02-08  
**Status:** Accepted  

**Problem / Context**
- LLM requests can contain secrets. Default logging must be safe while still supporting debugging.

**Option A — Metadata-only gateway spans + backend-native structured log ingestion (recommended)**
- **Pros:**
  - Lowest risk default: Substrate does not need to persist prompts/responses.
  - Debuggability improves without changing the gateway’s default posture by ingesting backend-native structured logs where available (e.g., `cli:codex` via `codex-wrapper` JSONL rollout logs) into session logs.
  - Keeps “what is logged” aligned with the backend adapter contract (CLI adapter chooses what it emits; gateway stays boring).
- **Cons:**
  - Requires per-backend adapters to support log ingestion (not universal across all CLIs).
  - Still requires strict redaction + caps rules for ingested events (never assume upstream logs are safe).

**Option B — Body logging available via explicit opt-in**
- **Pros:**
  - Strongest “single knob” debugging for dialect translation issues.
  - Backend-agnostic: works even when a backend provides no useful structured logs.
- **Cons:**
  - Higher risk: requires rigorous redaction + caps + operator warnings and careful storage semantics.
  - Creates ambiguity about “where secrets can land” in default deployments if operators flip it on casually.

**Recommendation**
- **Selected:** Option A — Metadata-only gateway spans + backend-native structured log ingestion.
- **Rationale (crisp):** Keeps the gateway default safe while still enabling rich session logs via backend adapters (starting with `cli:codex` JSONL ingestion); avoids normalizing “prompt/response logging” as the primary debug mechanism.

**Constraints / guardrails (non-negotiable)**
- Default MUST remain metadata-only at the gateway boundary (no prompt/response bodies).
- Any backend-native log ingestion MUST:
  - apply Substrate redaction/caps before writing trace/session logs, and
  - be policy-gated if it can materially increase sensitive data exposure.

---

### DR-0003 — Policy gate location: broker-first vs gateway-local checks

**Decision owner(s):** Broker + Gateway maintainers  
**Date:** 2026-02-08  
**Status:** Accepted  

**Problem / Context**
- We need deterministic allow/deny behavior with `--explain` provenance while minimizing duplicated policy logic.

**Option A — Broker-first policy evaluation**
- **Pros:** Centralizes policy reasoning; consistent explain/provenance story; fewer policy implementations.
- **Cons:** Gateway must call into broker (or consume a snapshot) on the hot path.

**Option B — Gateway-local checks against an embedded snapshot**
- **Pros:** Fast and self-contained; simpler in-world deployment.
- **Cons:** Snapshot lifecycle and provenance must be carefully specified to avoid drift.

**Recommendation**
- **Selected:** Option A — Broker-first policy evaluation.
- **Rationale (crisp):** Reuses the existing policy resolution + snapshot shape defined by ADR-0027; avoids re-implementing policy logic inside the gateway and preserves explain/provenance expectations.

---

### DR-0004 — Correlation id minting: gateway-minted vs client-supplied

**Decision owner(s):** Gateway + Trace + Agent Hub maintainers  
**Date:** 2026-02-08  
**Status:** Accepted  
**Related docs:** ADR-0023, ADR-0017, ADR-0028 (Phase 8 circle-back)

**Problem / Context**
- LLM requests need stable ids for joins (`orchestration_session_id`, `run_id`, `thread_id`, etc.) and must remain non-spoofable. Clients (Codex/Claude Code) may also provide their own request ids.

**Option A — Gateway mints Substrate ids (recommended); client ids are treated as hints**
- **Pros:** Non-spoofable; consistent across all clients; easy to enforce required fields.
- **Cons:** Requires explicit mapping fields if we want to preserve client-provided ids for debugging.

**Option B — Client supplies ids (gateway trusts/passes through)**
- **Pros:** Simple; preserves client semantics.
- **Cons:** Spoofable; inconsistent; harder to guarantee deterministic joins and auditability.

**Recommendation**
- **Selected:** Option A — Gateway mints Substrate ids; client ids are treated as hints.
- **Rationale (crisp):** Prevents spoofing and ensures join stability across all clients while still allowing optional preservation of client-provided ids for debugging.

---

### DR-0005 — Include passthrough backend in v1: yes vs defer

**Decision owner(s):** Gateway + Manager maintainers  
**Date:** 2026-02-08  
**Status:** Accepted  
**Related docs:** ADR-0023, ADR-0024

**Problem / Context**
- A “passthrough/record” backend can forward a request to the provider without dialect translation beyond minimal framing, enabling early value (record/redact/metrics) even before richer backend adapters ship.

**Option A — Include passthrough backend in v1**
- **Pros:** Fastest path to “front door” value; easy compatibility for existing clients; good for early trace/audit rollout.
- **Cons:** Requires carefully defining what counts as “passthrough” in-world; may complicate policy gates if multiple auth modes are supported.

**Option B — Defer passthrough backend; ship only CLI/API engines first**
- **Pros:** Fewer semantics to lock; reduces scope.
- **Cons:** Delays early adoption and testing of gateway framing/trace without needing CLI adapters to be perfect.

**Recommendation**
- **Selected:** Option B — Defer passthrough backend; ship only CLI/API engines first.
- **Rationale (crisp):** `cli:codex` + `api:*` cover the bulk of intended v1 usage while keeping semantics tight (clear auth + policy gates). Passthrough introduces a third execution/auth shape with higher “what exactly was forwarded” ambiguity; it can be added once the gateway’s core contracts are proven.

---

### DR-0006 — API backend configuration source: agent inventory vs config keys

**Decision owner(s):** Gateway + Manager + Broker maintainers  
**Date:** 2026-02-08  
**Status:** Accepted  
**Related docs:** ADR-0023, ADR-0027

**Problem / Context**
- We want an `api:*` backend mode soon (BYOK keys / direct provider API calls) without reshaping config/policy surfaces repeatedly.

**Option A — Represent API backends as agent inventory items (`kind: api`)**
- **Pros:**
  - Reuses the strict, one-file-per-backend inventory model (ADR-0027).
  - Keeps backend identity uniform across CLI and API (`<kind>:<name>`).
  - Enables per-backend restriction-only `policy_overlay` tightening.
- **Cons:**
  - Requires extending the agent file schema with `config.api.*` fields (endpoint/model defaults/auth mechanism) in an additive-only way.

**Option B — Add dedicated `llm.backends.api.*` config keys**
- **Pros:**
  - Keeps “LLM routing” config localized under `llm.*`.
- **Cons:**
  - Reintroduces dynamic backend registries in a strict schema (harder to keep strict without inventing a map schema).
  - Drifts from the inventory pattern already chosen for CLI backends.

**Recommendation**
- **Selected:** Option A — Represent API backends as agent inventory items (`kind: api`).
- **Rationale (crisp):** It preserves a single strict inventory model for dynamic backends (CLI and API) and keeps identity + allowlisting uniform via `<kind>:<name>` without introducing new dynamic config maps under `llm.*`.

---

### DR-0007 — API key handling for `api:*` backends: env injection vs secret-provider integration

**Decision owner(s):** Gateway + World + Security  
**Date:** 2026-02-08  
**Status:** Accepted  
**Related docs:** ADR-0023, ADR-0027 (“no secrets in Substrate YAML”)

**Problem / Context**
- `api:*` backends (direct provider HTTP) require API keys/tokens.
- Substrate MUST NOT store secrets in `config.yaml`, `policy.yaml`, or agent inventory YAML.
- We still need a deterministic way to make secrets available to the in-world gateway/engine when it is the component performing outbound egress.

**Option A — Environment-variable injection into the in-world gateway/engine (no persistence)**
- **Pros:**
  - Simple to implement and consistent with existing CLI/env-based operator workflows.
  - Avoids creating a new “Substrate secrets file” family.
  - Keeps secrets out of config/policy/agent inventory by storing only *references* (env var names) in inventory.
- **Cons:**
  - Requires clear redaction/caps rules so secrets never land in logs/spans.
  - Operators need a reliable way to set those env vars for the gateway/engine process across platforms.

**Option B — Host secret-provider integration (keychain/1Password/etc.) with a narrow host→world delivery mechanism**
- **Pros:**
  - Better UX; avoids exporting secrets into environments.
  - Can centralize auditing/rotation per provider.
- **Cons:**
  - Higher complexity and platform variance.
  - Requires defining a new privileged “secret delivery” surface and its policy gates.

**Recommendation**
- **Selected:** Option A — Environment-variable injection into the in-world gateway/engine (no persistence).
- **Rationale (crisp):** It ships fastest without introducing a new secrets file family; inventory can reference env var names while keeping key material out of Substrate YAML. This also composes with future secret-provider integrations as an additive improvement.

**Constraints / guardrails (non-negotiable)**
- Agent inventory MAY store only *env var names* (references), never key material.
- The gateway/engine MUST apply strict redaction/caps so secret values do not land in logs, spans, or error messages.
- Missing env vars MUST fail closed with actionable errors (identify which env var name is missing, not its value).
- Delivery mechanism (v1): the host-side world subsystem that ensures the in-world gateway/engine is running for the active session world MUST:
  - collect the required env var *values* from its own process environment, and
  - pass them across the existing world-agent transport as part of the in-world gateway/engine spawn request,
  so the in-world process receives them in its environment without persisting them to disk.
  - v1 command surface: `substrate world sync gateway` and `substrate world sync gateway --restart` are the explicit lifecycle entrypoints for this behavior.

---

### DR-0008 — Client wiring output contract for `substrate world status gateway`

**Decision owner(s):** Shell + Gateway maintainers  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** ADR-0023, `docs/project_management/next/llm_gateway_in_world/specs/http_surface.md`

**Problem / Context**
- Operators need a stable, low-footgun way to wire OpenAI/Anthropic-compatible clients to the in-world gateway after `substrate world sync gateway`.
- This output becomes a compatibility contract (consumed by wrappers/CLIs). We must decide whether we “own” 3rd-party env var names or only Substrate-owned wiring fields.

**Option A — Substrate-owned wiring keys only**
- **Pros:** Stable contract we control; avoids freezing 3rd-party env var names; avoids collisions/overrides of user provider env; cleaner long-term maintenance.
- **Cons:** Some clients need a small shim (e.g., a wrapper reads Substrate wiring JSON and maps internally).
- **Cascading implications:** `substrate world status gateway --json` becomes the primary contract (e.g., `client_wiring.openai_base_url`, `client_wiring.anthropic_base_url`), and human output can print Substrate-owned `export SUBSTRATE_LLM_*` lines without making 3rd-party names contractual.
- **Risks:** Slight initial UX friction for users expecting `eval`-style “just works” wiring.
- **Unlocks:** We can evolve wiring outputs/transports without breaking downstream tools; keeps the gateway “boring” and policy-focused.
- **Quick wins / low-hanging fruit:** Implement stable `--json` + a “Client wiring” human block that prints Substrate-owned exports for base URL routing only (no secrets).

**Option B — Contractual `eval`-ready exports including common 3rd-party env var names**
- **Pros:** Immediate ergonomics for some tools; fewer wrapper shims.
- **Cons:** We’d be freezing ambiguous names (`OPENAI_BASE_URL` vs `OPENAI_API_BASE`, etc.) and supporting them indefinitely; higher chance of clobbering user env.
- **Cascading implications:** Requires a maintained alias matrix and documentation; makes future client support messier.
- **Risks:** Footguns + long-term compatibility burden; “supported contract surface” grows quickly.
- **Unlocks:** Faster onboarding for a subset of clients that already honor those vars.
- **Quick wins / low-hanging fruit:** Provide a single `eval` block that sets both Substrate vars and selected 3rd-party vars.

**Recommendation**
- **Selected:** Option A — Substrate-owned wiring keys only
- **Rationale (crisp):** Keep the wiring contract tight and under Substrate control, avoid env-collision footguns, and let wrappers (starting with `cli:codex`) consume a stable JSON contract.
