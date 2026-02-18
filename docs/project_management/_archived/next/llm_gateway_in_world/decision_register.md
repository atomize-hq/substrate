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
**Related docs:** ADR-0023, ADR-0027 (“no secrets in Substrate YAML”), `docs/project_management/standards/SECRETS_DELIVERY_CHANNEL_RUBRIC.md`

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
**Related docs:** ADR-0023, `docs/project_management/_archived/next/llm_gateway_in_world/specs/http_surface.md`

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

---

### DR-0009 — Standard env var names for client wiring + injected backend auth

**Decision owner(s):** Shell + Gateway + Engine maintainers; Security  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** ADR-0023, ADR-0024, ADR-0027, DR-0007, DR-0008, `docs/project_management/_archived/next/llm_gateway_in_world/specs/env_injection.md`

**Problem / Context**
- We need stable env var names for:
  1) client wiring (base URLs) so OpenAI/Anthropic-compatible clients can route to the Substrate gateway, and
  2) injected backend auth fields used by in-world gateway/engine processes (never printed, never persisted).
- We must avoid:
  - leaking secrets in output/logs/spans,
  - coupling the contract to provider/CLI-specific env var naming conventions,
  - and “dual meaning” where a base URL points sometimes to Substrate and sometimes to providers.

**Option A — Substrate-owned env var names + Substrate-owned injected auth naming scheme (recommended)**
- **Pros:**
  - Stable contract we control; consistent redaction rules; backend-generic for future `cli:*` and `api:*` adapters.
  - Base URL semantics remain unambiguous: these always point to Substrate’s gateway front door (never to upstream providers).
  - Keeps provider endpoints and secret material out of “operator visible” wiring output.
- **Cons:**
  - Some wrappers/clients need a mapping layer (read Substrate vars and set their internal config).
  - Slightly longer env var names, but the secret-bearing ones are internal-only.
- **Cascading implications:**
  - Client wiring env vars printed by `substrate world status gateway` (human + `--json`):
    - `SUBSTRATE_LLM_OPENAI_BASE_URL` (value: Substrate gateway OpenAI-dialect base URL)
    - `SUBSTRATE_LLM_ANTHROPIC_BASE_URL` (value: Substrate gateway Anthropic-dialect base URL)
  - Injected backend auth env var naming scheme (values never printed):
    - `SUBSTRATE_LLM_BACKEND_AUTH_<KIND>_<NAME>_<FIELD>` (UPPER_SNAKE_CASE)
    - v1 (`cli:codex`) injected fields:
      - `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID`
      - `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN`
- Any env var with prefix `SUBSTRATE_LLM_BACKEND_AUTH_` MUST be treated as secret-bearing and MUST be redacted/capped everywhere (trace spans/events, structured errors, stdout/stderr).
  - Phase 8 additive clarification: these identifiers are the canonical *auth field names*. When Substrate-owned engines/wrappers receive auth via FD/pipe (DR-0017), the payload keys MUST still use the same `SUBSTRATE_LLM_BACKEND_AUTH_*` names so redaction/caps rules remain uniform.
- **Risks:**
  - If mapping glue is sloppy, clients may still leak provider endpoints or secrets in their own logs; mitigate with wrapper hardening + explicit “do not print” guidance.
- **Unlocks:**
  - Future backends can add injected fields without reshaping the operator-facing “client wiring” contract.
  - Uniform secret-handling posture across both CLI and API backends.
- **Quick wins / low-hanging fruit:**
  - Implement v1 using codex-wrapper: consume `SUBSTRATE_LLM_*_BASE_URL` for routing; treat `SUBSTRATE_LLM_BACKEND_AUTH_*` as canonical auth field names and deliver their values to Substrate-spawned engine processes via FD/pipe by default (DR-0017), falling back to env vars only when required for compatibility.

**Option B — Use provider/CLI-native env vars as the primary contract**
- **Pros:**
  - Potentially “drop-in” for some tools that already honor provider env vars.
- **Cons:**
  - Freezes ambiguous third-party naming (`OPENAI_BASE_URL` vs `OPENAI_API_BASE`, etc.).
  - Higher collision risk with user environments; harder to maintain; mixes wiring with secrets more easily.
- **Cascading implications:**
  - Substrate must commit to a compatibility matrix of third-party env var names and semantics.
  - Increased risk that “wiring output” accidentally includes secret-bearing vars.
- **Risks:**
  - Footguns (overriding user provider config); long-term support burden; higher probability of secret leakage.
- **Unlocks:**
  - Faster onboarding for a narrow slice of tooling (at the cost of contract bloat).
- **Quick wins / low-hanging fruit:**
  - Minimal up-front mapping logic, but costs accrue quickly as more tools are supported.

**Recommendation**
- **Selected:** Option A — Substrate-owned env var names + Substrate-owned injected auth naming scheme
- **Rationale (crisp):** Keeps wiring semantics unambiguous (always Substrate), keeps secrets out of operator-visible output, and provides a backend-generic scheme we can extend for future `cli:*` and `api:*` adapters with uniform redaction rules.

---

### DR-0010 — Visibility of client wiring in `substrate world status gateway`

**Decision owner(s):** Shell + Gateway maintainers  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** ADR-0023, DR-0008, DR-0009

**Problem / Context**
- Substrate is the front door; client wiring base URLs always point to Substrate, not upstream providers.
- Printing wiring exports by default is noisy and can mislead operators into thinking manual wiring is a required step for normal usage.
- We still need a stable machine-readable contract for wrappers/tests and an operator/debug surface.

**Option A — Show client wiring by default (human + `--json`)**
- **Pros:** Always visible; copy/paste friendly; fewer “where is it bound?” questions.
- **Cons:** Noisy for normal use; encourages treating wiring as a “required step”; expands the default output contract.
- **Cascading implications:** `substrate world status gateway` becomes partly a wiring command rather than purely status/health.
- **Risks:** Confusion about whether users must manually wire clients; increased contract stability burden.
- **Unlocks:** Immediate manual interoperability with arbitrary clients.
- **Quick wins / low-hanging fruit:** None; this is essentially “keep current default”.

**Option B — Hide wiring by default; show only under `--debug`, and always include in `--json` (recommended)**
- **Pros:** Default output stays “status + health + routing summary”; wiring remains available for debugging and automation.
- **Cons:** Slightly more discovery friction (operators must know `--debug` exists).
- **Cascading implications:** Define:
  - default human output: status/health + configured/allowed/selected backends + last error (no wiring exports),
  - `--debug`: includes wiring exports + bind details,
  - `--json`: always includes non-secret `client_wiring.*` fields.
- **Risks:** Wrappers that depend on human output must switch to `--json`.
- **Unlocks:** Cleaner UX aligned with “Substrate is always the front door,” while preserving an automation/debug contract.
- **Quick wins / low-hanging fruit:** Minimal doc + output change; wrappers consume `--json` and ignore human formatting.

**Recommendation**
- **Selected:** Option B — Hide wiring by default; show only under `--debug`, and always include in `--json`
- **Rationale (crisp):** Keeps the default command meaning “status/health” while retaining a stable, non-secret wiring contract for tooling and debugging.

---

### DR-0011 — Upstream provider endpoint configuration for `api:*` backends: inventory value vs env reference

**Decision owner(s):** Gateway + Engine + Security  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** ADR-0027 (agent inventory + “no secrets in YAML”), DR-0006, DR-0007

**Problem / Context**
- `api:*` backends require an upstream provider endpoint/base URL (non-secret) plus auth material (secret, handled separately).
- We need a strict, deterministic way to configure the upstream base URL without turning it into a secret-storage channel or introducing yet another env indirection knob.

**Option A — Store upstream base URLs directly in agent inventory YAML (recommended)**
- **Pros:** Simple and explicit; stable and inspectable; avoids additional env indirection; keeps routing/allowlisting identity (`api:<name>`) co-located with its endpoint; works well with sparse YAML layering via inventory precedence.
- **Cons:** Requires schema and validation work (URL parsing, normalization); operators must edit agent inventory files to change endpoints.
- **Cascading implications:**
  - Extend agent file schema (ADR-0027) for `config.kind=api` to include `config.api.base_url: string` (non-secret).
  - Add safety validation:
    - reject credential-bearing URLs (userinfo `user:pass@`),
    - reject query parameters that look like secrets,
    - and normalize/truncate for logging (never log full URL if it contains suspicious components).
  - Enforce that actual egress is still governed by `net_allowed` at the world boundary.
- **Risks:** Misconfigured endpoint could point to unexpected upstreams; mitigated by `net_allowed` and by requiring explicit policy allowlisting (`llm.allowed_backends`).
- **Unlocks:** Clean BYOK-style `api:*` story without inventing a new config map family; easier later to add per-backend endpoint overrides in workspace inventory.
- **Quick wins / low-hanging fruit:** Start with `api:openai` as the first concrete example and keep the schema minimal (`base_url` only).

**Option B — Reference env var names for upstream base URLs**
- **Pros:** Easy runtime overrides; avoids editing inventory files; aligns with “env injection” mental model.
- **Cons:** Adds indirection and footguns; increases reliance on process environments; makes behavior less inspectable and harder to `--explain`.
- **Cascading implications:**
  - Add `config.api.base_url_env: string` to inventory schema and define precedence between value vs env.
  - Must ensure base URL env vars are treated as non-secret but still validated to avoid credential-bearing URLs.
- **Risks:** Inconsistent behavior across shells/CI; harder debugging (“which env was set?”).
- **Unlocks:** Fast local experimentation with alternate endpoints.
- **Quick wins / low-hanging fruit:** Quick to ship but increases long-term drift risk.

**Recommendation**
- **Selected:** Option A — Store upstream base URLs directly in agent inventory YAML
- **Rationale (crisp):** Base URLs are non-secret; keeping them explicit in strict inventory YAML avoids needless env indirection while preserving a clean, auditable backend contract gated by policy allowlists and `net_allowed`.

---

### DR-0012 — Client wiring base URL reachability: in-world-only vs host-reachable bridge

**Decision owner(s):** Shell + World-agent + Gateway maintainers  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** ADR-0023, DR-0001, DR-0010

**Problem / Context**
- We expose OpenAI/Anthropic-compatible HTTP surfaces in the in-world gateway.
- We selected DR-0001 Option A (world-agent proxy transport) to avoid a persistent host listener that could be mistaken for a host-level egress gateway.
- We must decide whether the “base URLs” we expose via `substrate world status gateway --json` are intended to be reachable from the host, or only from inside the world boundary.

**Option A — In-world-only base URLs (recommended)**
- **Pros:** Fully consistent with “in-world gateway” posture and DR-0001 Option A; no host listener needed; simplest threat model.
- **Cons:** Host tools cannot directly point at the gateway; they must run in-world or go through Substrate orchestration paths that execute the client/backend in-world.
- **Cascading implications:** Client wiring output MUST clearly label these base URLs as “reachable from inside the world boundary” so operators do not attempt to use them directly from the host.
- **Risks:** Confusion if someone tries to use the URL from the host; mitigated by labeling and docs (`--debug` output should say “in-world only”).
- **Unlocks:** Cleanest enforcement story; avoids “localhost tunnel” confusion entirely.
- **Quick wins / low-hanging fruit:** No additional host proxy/forwarder required.

**Option B — Host-reachable base URLs via an explicit transport bridge**
- **Pros:** Easier manual debugging and compatibility with arbitrary host tools.
- **Cons:** Reintroduces a host listener/bridge surface (even if transport-only) that can be misunderstood as host egress; more moving parts.
- **Cascading implications:** Define the bridge lifecycle + binding + port selection + security story; ensure it cannot become a policy bypass.
- **Risks:** Footguns + broader attack surface; higher documentation/support burden.
- **Unlocks:** “Point any host client at Substrate” workflows.
- **Quick wins / low-hanging fruit:** Faster ad-hoc interoperability for some tools.

**Recommendation**
- **Selected:** Option A — In-world-only base URLs
- **Rationale (crisp):** Matches the chosen transport posture and the “Substrate as front door + in-world egress” story without adding a host bridge that can be misinterpreted.

---

### DR-0013 — `api:*` auth env var references: explicit per-backend list vs backend-contract inference

**Decision owner(s):** Gateway + Engine + Security  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** ADR-0027 (strict agent inventory schema), DR-0007, `docs/project_management/_archived/next/llm_gateway_in_world/specs/env_injection.md`

**Problem / Context**
- `api:*` backends require auth material but Substrate YAML must not store secret values.
- The world sync/gateway spawn path still needs to know which env vars must be present so it can fail closed and inject them into the in-world gateway/engine process environment.

**Option A — Explicit list of required env var names in agent inventory (recommended)**
- **Pros:** Backend-generic for any provider; explicit and auditable; works with strict schema; supports multiple vars; no provider-specific code paths needed to know what to request.
- **Cons:** Slightly more config per `api:*` backend; env var names must be typed correctly.
- **Cascading implications:**
  - Extend the `config.kind=api` agent schema to include `config.api.auth.env: [string]` (env var names only).
  - `substrate world sync gateway` MUST fail closed with actionable errors that list missing env var *names* (never values).
  - Any env var values injected MUST be treated as secret-bearing and must be redacted/capped everywhere.
- **Risks:** Mis-typed env var name leads to startup failure; mitigated by clear errors and optional doctor output.
- **Unlocks:** Multi-provider `api:*` without code changes; clean BYOK path.
- **Quick wins / low-hanging fruit:** Start with `api:openai` and `config.api.auth.env: [\"OPENAI_API_KEY\"]`.

**Option B — Backend-contract inference (no per-backend env list)**
- **Pros:** Less config in agent inventory files.
- **Cons:** Requires provider-specific inference code and a maintained mapping; harder to support custom providers or nonstandard auth; less explainable.
- **Cascading implications:** Gateway/engine must hardcode required env var names per provider/backend id.
- **Risks:** Drift as providers change; higher long-term maintenance.
- **Unlocks:** Minimal initial YAML for a single provider.
- **Quick wins / low-hanging fruit:** Fastest for OpenAI-only prototypes, but expensive later.

**Recommendation**
- **Selected:** Option A — Explicit list of required env var names in agent inventory
- **Rationale (crisp):** Keeps the system backend-generic and auditable while preserving strict schema and fail-closed behavior.

---

### DR-0014 — Policy gate for injected secret env var names: explicit allowlist vs backend-contract-only

**Decision owner(s):** Broker + Gateway + Security  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** ADR-0027 (policy surface ownership), DR-0007, DR-0013, `docs/project_management/_archived/next/llm_gateway_in_world/specs/env_injection.md`

**Problem / Context**
- Inventory can declare required secret env var *names* (e.g., `config.api.auth.env`) and the world sync path injects their values into an in-world process environment.
- Even though secret values are never stored in YAML, the ability to read arbitrary host env vars and inject them into in-world processes is powerful and must be policy-governed.

**Option A — Add explicit policy allowlist for injectable env var names (recommended)**
- **Pros:** Auditable and explicit; prevents inventory from “asking for” arbitrary host env vars; supports least-privilege security posture and clearer `--explain` provenance.
- **Cons:** Adds a policy knob and setup friction (policy must include required names).
- **Cascading implications:**
  - Add strict policy key: `llm.secrets.env_allowed: [string]` (deny-by-default).
  - The sync/restart path MUST fail closed if any requested injection name is not in `llm.secrets.env_allowed`.
  - Applies to all secret env injection into in-world gateway/engine spawn env (both `api:*` and any `cli:*` injected auth fields).
- **Risks:** Misconfiguration blocks backends until policy is updated; mitigated by actionable errors and `--explain` output.
- **Unlocks:** Safe expansion to additional providers/backends; prevents accidental overreach into unrelated host secrets.
- **Quick wins / low-hanging fruit:** Start with a minimal list (e.g., `OPENAI_API_KEY`) and expand explicitly as new backends land.

**Option B — No extra policy key; treat requested names as part of backend contract**
- **Pros:** Fewer knobs; faster onboarding; less policy churn.
- **Cons:** Inventory could request unrelated host env vars; harder to audit/deny without disabling the backend entirely.
- **Cascading implications:** Security posture relies on code review + strict schema only; `--explain` can’t point to a dedicated injection allowlist gate.
- **Risks:** Capability creep and hidden exfil vectors via env injection.
- **Unlocks:** Minimal config for early prototypes.
- **Quick wins / low-hanging fruit:** None; strict redaction/caps still required.

**Recommendation**
- **Selected:** Option A — Add explicit policy allowlist for injectable env var names
- **Rationale (crisp):** The power to read host secrets and inject them into in-world processes should never be implicitly granted by inventory; an explicit allowlist keeps injection least-privilege and explainable.

---

### DR-0015 — Scope of `llm.secrets.env_allowed`: host env reads only vs all injected secret env names

**Decision owner(s):** Broker + Gateway + Security  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** DR-0014, DR-0013, ADR-0027, `docs/project_management/_archived/next/llm_cli_backend_engine/decision_register.md` (host credential read gates)

**Problem / Context**
- We introduced `llm.secrets.env_allowed` to prevent arbitrary host env var reads/injection.
- We also inject secret-bearing auth material for `cli:codex`, but that material is sourced from host credential files (gated by `agents.host_credentials.read.allowed_backends`) rather than host env.
- We need to define whether `llm.secrets.env_allowed` is:
  - narrowly scoped to host env reads, or
  - a universal gate for all secret env vars injected into in-world processes.

**Option A — Gate host env reads only (recommended)**
- **Pros:** Clear separation of concerns; keeps `llm.secrets.env_allowed` aligned to “which host env vars can we read”; avoids requiring policy to list internal injected var names; simpler operator UX.
- **Cons:** Secret-bearing injected vars derived from host credential files (e.g., `cli:codex`) are not additionally gated by `llm.secrets.env_allowed` (they remain governed by the host-credential-read gate + strict redaction).
- **Cascading implications:** Define:
  - `llm.secrets.env_allowed` gates only reads from host process environment for injection (primarily `api:*` via `config.api.auth.env`).
  - Secret injection derived from host credential reads remains gated by `agents.host_credentials.read.allowed_backends` and related CLI-engine decisions.
- **Risks:** Two gates to understand (host env reads vs host credential file reads); mitigated by `--explain` clarity and docs.
- **Unlocks:** Keeps policy minimal while still preventing “inventory asks for arbitrary host env vars”.
- **Quick wins / low-hanging fruit:** Easy enable recipe: add `OPENAI_API_KEY` to `llm.secrets.env_allowed` when enabling `api:openai`.

**Option B — Gate all secret env injection names (strictest)**
- **Pros:** Single universal gate for all secret env var names injected into in-world processes.
- **Cons:** Forces policy to enumerate internal injected var names (e.g., `SUBSTRATE_LLM_BACKEND_AUTH_*`), increasing surface area and operator friction; mixes “source=host env” and “source=host credential file” concerns.
- **Cascading implications:** `llm.secrets.env_allowed` must include every secret env var name injected for any backend; missing names fail closed even when secrets are sourced from files.
- **Risks:** Higher breakage risk when internal injected var names evolve; more policy churn.
- **Unlocks:** A single conceptual gate at the cost of more complexity.
- **Quick wins / low-hanging fruit:** None; increases setup steps.

**Recommendation**
- **Selected:** Option A — Gate host env reads only
- **Rationale (crisp):** Keeps `llm.secrets.env_allowed` focused on controlling which host env vars may be read, while `cli:*` secret material sourced from host credential files is controlled by a separate, explicit gate; avoids forcing operators to enumerate internal injected var names.

---

### DR-0016 — `api:*` secret injection name in-world: preserve upstream name vs map to Substrate-owned name

**Decision owner(s):** Gateway + Engine + Security  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** DR-0009, DR-0013, DR-0014, `docs/project_management/_archived/next/llm_gateway_in_world/specs/env_injection.md`

**Problem / Context**
- For `api:*` backends we read host env vars named in `config.api.auth.env` (names-only) and inject values into the in-world gateway/engine spawn environment.
- We must decide whether the in-world process receives the secret under the upstream/provider env var name (e.g., `OPENAI_API_KEY`) or a Substrate-owned injected name.

**Option A — Inject using the upstream/provider env var name**
- **Pros:** Simplest; compatible with existing provider SDK expectations; fewer mapping rules.
- **Cons:** Normalizes provider-native secret env vars inside the world; less uniform with existing `SUBSTRATE_LLM_BACKEND_AUTH_*` conventions.
- **Cascading implications:** The in-world API engine reads provider-native env var names directly.
- **Risks:** More “ambient secret naming” inside world; harder to keep secret naming consistent across providers.
- **Unlocks:** Quick integration with off-the-shelf SDKs.
- **Quick wins / low-hanging fruit:** Fastest to implement for `api:openai`.

**Option B — Map host env reads to Substrate-owned injected env var names (recommended)**
- **Pros:** Uniform secret naming scheme; reduces reliance on provider conventions; keeps provider-native secret env vars out of the in-world environment; aligns with `SUBSTRATE_LLM_BACKEND_AUTH_<KIND>_<NAME>_<FIELD>`.
- **Cons:** Requires a mapping rule and the API engine must read Substrate-owned names (not provider-native).
- **Cascading implications:**
  - Host reads the names in `config.api.auth.env` (still gated by `llm.secrets.env_allowed`).
  - In-world injection uses `SUBSTRATE_LLM_BACKEND_AUTH_API_<NAME>_<FIELD>` (e.g., `SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY`).
  - The in-world API engine reads Substrate-owned injected names only.
- **Risks:** Slightly more adapter logic; must ensure strict redaction/caps and never print values.
- **Unlocks:** Cleaner multi-provider future without env var naming collisions; consistent redaction rules (“anything `SUBSTRATE_LLM_BACKEND_AUTH_*` is secret”).
- **Quick wins / low-hanging fruit:** Reuse the same redaction/caps handling already planned for `SUBSTRATE_LLM_BACKEND_AUTH_*`.

**Recommendation**
- **Selected:** Option B — Map to Substrate-owned injected env var names
- **Rationale (crisp):** Keeps secret naming uniform and Substrate-controlled, avoids ambient provider-native secret env vars in-world, and aligns with our existing injected-auth naming scheme.

---

### DR-0017 — Secret delivery to Substrate-spawned engine processes: env vars vs inherited one-time FD/pipe

**Decision owner(s):** Gateway + Engine + Security  
**Date:** 2026-02-13  
**Status:** Accepted  
**Related docs:** `docs/project_management/standards/SECRETS_DELIVERY_CHANNEL_RUBRIC.md`, `docs/project_management/_archived/next/llm_gateway_in_world/specs/env_injection.md`

**Problem / Context**
- DR-0007 selects env injection as the v1 host→world delivery mechanism for getting secret values into the in-world gateway/manager process without persisting them to disk.
- Once inside the world, the gateway/manager may spawn Substrate-owned backend engines/wrappers (e.g., a `cli:*` wrapper process) that need secret values.
- We must decide whether the gateway/manager propagates those secrets to Substrate-spawned child processes via:
  - child-process environment variables, or
  - an inherited one-time FD/pipe secret channel.

**Option A — Propagate secrets to child engines via env vars**
- **Pros:** Simplest; no additional spawn plumbing; matches existing env-injection patterns.
- **Cons:** Expands secret exposure across the process tree; increases accidental disclosure risk (debug logs, `/proc/<pid>/environ`); harder to guarantee “smallest possible blast radius”.
- **Cascading implications:** Requires treating all `SUBSTRATE_LLM_BACKEND_AUTH_*` env vars as secret-bearing across *every* spawned engine process and their descendants.

**Option B — Propagate secrets to Substrate-spawned child engines via FD/pipe (recommended)**
- **Pros:** Keeps secrets out of child-process env by default; scopes secrets to the intended consumer; aligns with the cross-track secrets rubric.
- **Cons:** Requires a small amount of spawn plumbing and a stable “auth payload” read contract for wrappers/engines.
- **Cascading implications:**
  - For Substrate-owned wrappers/engines, the gateway/manager MUST provide a one-time auth payload over an inherited FD/pipe.
  - An env var MAY be used to convey the FD number (non-secret) (e.g., `SUBSTRATE_LLM_BACKEND_AUTH_FD: int`), following the rubric.
  - Env var propagation remains permitted only as a compatibility fallback when FD/pipe is not available for a specific engine/platform.

**Recommendation**
- **Selected:** Option B — Propagate secrets to Substrate-spawned child engines via FD/pipe.
- **Rationale (crisp):** Env injection is a pragmatic v1 host→world mechanism, but within the world Substrate should default to the least-exposure channel for Substrate-spawned components.

---

### DR-0018 — Host→world secret delivery to the in-world gateway/manager: env injection vs secret-channel + FD/pipe

**Decision owner(s):** World + Gateway + Security  
**Date:** 2026-02-13  
**Status:** Accepted  
**Related docs:** DR-0007, `docs/project_management/standards/SECRETS_DELIVERY_CHANNEL_RUBRIC.md`, `docs/project_management/_archived/next/llm_gateway_in_world/specs/env_injection.md`

**Problem / Context**
- DR-0007 selects env injection as the v1 host→world delivery mechanism for getting secret values into the in-world gateway/engine process without persisting them to disk.
- Phase 8 requires tightening “end-to-end secret channel” semantics so secret values do not live in in-world process environments by default (consistent with the shared secrets rubric and with DR-0017’s in-world propagation posture).
- We need an additive upgrade that keeps the v1 mechanism available, but makes the preferred path:
  - explicit,
  - minimal exposure (no secret-bearing env vars in-world by default),
  - and fail-closed when required secrets are missing.

**Option A — Continue v1: inject secret values into the gateway/manager process environment**
- **Pros:** Minimal implementation complexity; matches DR-0007 and existing spec language.
- **Cons:** Secret values are present in the in-world process environment (OS-level exposure such as `/proc/<pid>/environ`); broader accidental disclosure risk; makes “no secret env by default” impossible.
- **Cascading implications:** All `SUBSTRATE_LLM_BACKEND_AUTH_*` env vars remain secret-bearing and must be redacted/capped everywhere and never printed by default.

**Option B — Additive upgrade: host→world secret-channel payload + in-world FD/pipe delivery to gateway/manager (recommended)**
- **Pros:** Keeps secrets out of in-world process env by default; scopes secrets to the intended consumer; aligns with the cross-track secrets rubric and the toolbox token approach (`*_TOKEN_FD` pointers).
- **Cons:** Requires a small amount of additional spawn plumbing (world-agent writes a one-time payload to a pipe/FD and passes only a pointer).
- **Cascading implications:**
  - Host-side secret sourcing remains unchanged and must remain policy-gated:
    - host env reads gated by `llm.secrets.env_allowed` (DR-0015 / ADR-0027),
    - host credential file reads gated by `agents.host_credentials.read.allowed_backends` (ADR-0027 / CLI engine DRs).
  - The world-agent/gateway spawn request MUST carry secret values only via a secret-channel payload (never via printed/exported env).
  - The in-world gateway/manager MUST receive the secret-channel payload via an inherited one-time FD/pipe and load it into memory.
  - An env var MAY be used to convey the FD number (non-secret) but it MUST NOT use the `SUBSTRATE_LLM_BACKEND_AUTH_*` secret-bearing env family. Prefer a pointer name such as:
    - `SUBSTRATE_LLM_AUTH_BUNDLE_FD: int` (non-secret; safe to print)
  - The auth payload keys MUST use the canonical `SUBSTRATE_LLM_BACKEND_AUTH_<KIND>_<NAME>_<FIELD>` field names (even when values are not carried as env vars) so redaction/caps rules remain uniform.
  - Legacy v1 env injection remains permitted only as a compatibility path (implementation-defined), but MUST NOT be the default once Option B is implemented.

**Recommendation**
- **Selected:** Option B — Secret-channel payload + in-world FD/pipe delivery to gateway/manager.
- **Rationale (crisp):** Substrate spawns and controls the in-world gateway/manager process, so we can use an inherited one-time FD/pipe channel to avoid placing secrets in the in-world process environment while preserving strict, policy-gated host secret sourcing.
