# Decision Register — llm_cli_backend_engine

Standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

Scope:
- This decision register supports `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`.
- Each decision is recorded as exactly two viable options (A/B) with explicit tradeoffs and a single selection.

---

Phase 8 additive note:
- Earlier v1 decisions describe injecting secret values into in-world process environments as the host→world delivery mechanism.
- The preferred mechanism going forward is a host→world secret-channel payload delivered to the in-world gateway/manager via an inherited one-time FD/pipe auth bundle (no secret-bearing env vars in-world by default). See `docs/project_management/next/llm_gateway_in_world/decision_register.md` (DR-0018) and `docs/project_management/next/llm_gateway_in_world/specs/env_injection.md`.

### DR-0001 — CLI session strategy: persistent vs per-request

**Decision owner(s):** Shell + Engine maintainers  
**Date:** 2026-02-08  
**Status:** Accepted  
**Related docs:** ADR-0024, ADR-0027

**Problem / Context**
- CLIs vary: some are fast to spawn, others are expensive; some support structured streaming, others do not.

**Option A — Persistent session by default**
- **Pros:** Lower latency for repeated requests; more feasible for streaming; amortizes auth/setup.
- **Cons:** Harder lifecycle management; requires robust isolation and cleanup; more state to audit.

**Option B — Per-request spawn by default**
- **Pros:** Simpler; fewer long-lived processes; easier to fail closed.
- **Cons:** Higher latency; streaming may be worse or unavailable.

**Recommendation**
- **Selected:** Option A — Persistent session by default.
- **Rationale (crisp):** The initial target CLIs support streaming and benefit from amortized startup/auth costs; a persistent session also aligns with `agents.defaults.cli.mode=persistent`.

---

### DR-0002 — Streaming fallback when CLI lacks streaming: buffer+rechunk vs non-stream

**Decision owner(s):** Engine maintainers  
**Date:** 2026-02-08  
**Status:** Accepted  

**Problem / Context**
- The gateway exposes streaming dialects, but a CLI backend may only produce a final output blob.

**Option A — Buffer then re-chunk into a synthetic stream**
- **Pros:** Preserves a streaming surface for clients expecting SSE/chunked output.
- **Cons:** Can mislead clients about latency; must be clearly labeled in trace/event metadata.

**Option B — Return non-streaming response when backend lacks streaming**
- **Pros:** Honest semantics; simpler.
- **Cons:** Breaks some clients and reduces compatibility.

**Recommendation**
- **Selected:** Option B — Return non-streaming response when backend lacks streaming (v1).
- **Rationale (crisp):** The initial CLI backends are expected to support true streaming; shipping synthetic streaming now adds complexity and risk. Track buffer+rechunk (Option A) as a follow-up/tech-debt item once core routing is stable.

---

### DR-0003 — CLI prompt contract: JSON envelope vs plain text template

**Decision owner(s):** Engine + Agent adapters maintainers  
**Date:** 2026-02-08  
**Status:** Accepted  

**Problem / Context**
- We need a deterministic, testable transformation from canonical request to CLI invocation.

**Option A — JSON envelope (structured)**
- **Pros:** More testable; explicit fields; easier to version; easier to redact/cap.
- **Cons:** Some CLIs may not accept structured input; requires adapter translation.

**Option B — Plain text template**
- **Pros:** Universal; easiest to send to any CLI.
- **Cons:** Harder to evolve; brittle parsing; more prompt-injection surface.

**Recommendation**
- **Selected:** Option A — JSON envelope (structured).
- **Rationale (crisp):** A structured envelope is versionable and testable, reduces translation sprawl, and makes caps/redaction rules explicit at the adapter boundary.

---

### DR-0004 — CLI backend auth posture in-world: login-in-world vs credential forwarding

**Decision owner(s):** Engine + World + Security  
**Date:** 2026-02-08  
**Status:** Accepted  
**Related docs:** ADR-0024, ADR-0027 (“no secrets in Substrate YAML”)

**Problem / Context**
- Subscription-first CLI backends require an authenticated session. To keep the enforcement story honest, the CLI must run inside the world boundary when `llm.fail_closed.routing=true`. We need a posture for how the auth state exists inside the world.

**Option A — Login inside the world**
- **Pros:** Cleanest boundary; avoids moving credentials across host/world; aligns with “no secrets in Substrate config/policy”.
- **Cons:** UX friction (user may need to login per world VM/WSL once); must document “how to login in-world”.

**Option B — Forward/mount host credential material into the world (policy-gated)**
- **Pros:** Better UX; reuses existing host login state.
- **Cons:** Risky; can become an implicit secret transport; requires tight rules + redaction + explicit operator consent and may be provider/CLI-specific.

**Recommendation**
- **Selected:** Option B — Forward/mount host credential material into the world (policy-gated).
- **Rationale (crisp):** Subscription-first UX is significantly better if existing host auth state can be reused. However, this MUST be an explicit, policy-gated mechanism and MUST NOT store secrets in Substrate YAML.

**Constraints / guardrails (non-negotiable)**
- This decision does NOT change the enforcement boundary: when effective policy has `llm.fail_closed.routing=true`, the CLI backend process MUST still execute inside the world boundary (or the request fails closed).
- “Credential forwarding” MUST be implemented as an explicit, auditable mechanism (e.g., read-only mounts/forwarders) and MUST NOT be an implicit side-effect of enabling the CLI backend.
- Substrate config/policy YAML MUST NOT embed credential material.
- If forwarding cannot be applied safely on a platform/backend, routing to that backend MUST fail closed (with actionable error text).
- Host-run “wrap subprocesses with `substrate -c ...`” is not sufficient to guarantee egress enforcement for provider backends: many CLIs perform HTTPS requests in-process (not as subprocesses).
- However, a host-run CLI/wrapper MAY still be compatible with the enforcement story if it is operating as a *client* of the in-world gateway (i.e., “frontend mode”), so that **all** network egress happens at the gateway inside the world boundary. That mode is distinct from “CLI provider backend engine” behavior.
- Concrete example (Codex):
  - If the Codex wrapper is pointed at the in-world gateway and the gateway performs the outbound request to the OpenAI/Codex endpoint, then the auth material (e.g., from `~/.codex/auth.json`) must still be made available to the component that performs outbound egress (gateway/engine), either:
    - passed per request (no persistence), or
    - forwarded/mounted into the world as an explicit, policy-gated mechanism (e.g., read-only mount), with clear redaction/caps guidance.
  - Phase 8 additive clarification: when a Substrate-owned in-world wrapper/engine is spawned and needs auth material, Substrate MUST use FD/pipe propagation from the gateway/manager to that child instead of child-process env vars when supported (cross-track rubric). Env var propagation is permitted only as a compatibility fallback.

**Tech debt / follow-up**
- Document and standardize per-CLI credential locations + forwarding mechanism(s) per platform (Lima/WSL/Linux), including redaction/caps guidance for any traces/logs that mention credential paths.

---

### DR-0005 — v1 CLI backend scope: Codex-only vs multiple CLIs

**Decision owner(s):** Engine maintainers  
**Date:** 2026-02-08  
**Status:** Accepted  
**Related docs:** ADR-0024

**Problem / Context**
- We want a clean, extensible adapter contract, but also need a realistic v1 scope. Supporting multiple CLIs immediately risks spending most effort on adapter edge cases instead of validating the core gateway→canonical→backend architecture.

**Option A — Require only `cli:codex` in v1 (others planned)**
- **Pros:**
  - Tight scope; faster to ship a working end-to-end path.
  - Lets us validate canonical IR, routing, tracing, and policy gates with one high-leverage backend.
  - Keeps the adapter interface generic while avoiding premature multi-CLI compatibility work.
- **Cons:**
  - Cross-provider routing breadth is deferred until additional adapters land.

**Option B — Require multiple CLIs in v1 (e.g., Codex + Claude Code + Gemini CLI)**
- **Pros:**
  - Proves portability early; exercises more translation paths.
- **Cons:**
  - Higher complexity and schedule risk; increases surface area for dialect/capability mismatches.

**Recommendation**
- **Selected:** Option A — Require only `cli:codex` in v1.
- **Rationale (crisp):** Codex provides the highest leverage initial backend; we keep the contract generic and add other `cli:*` adapters once the core pipeline is proven.

---

### DR-0006 — Codex auth material delivery into world for `cli:codex`

**Decision owner(s):** Engine + World + Security  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** ADR-0024, ADR-0023, ADR-0027, `docs/project_management/next/llm_gateway_in_world/specs/env_injection.md`

**Problem / Context**
- `cli:codex` requires account/token material (host login state, typically under `~/.codex/`) to be available to the in-world component that performs outbound egress when `llm.fail_closed.routing=true`.
- Substrate MUST NOT store secret values in Substrate YAML and MUST avoid leaking secret values into trace/session logs.
- We need a deterministic, cross-platform mechanism (Linux + Lima + WSL) that can be supervised by the world subsystem (`substrate world sync gateway`).

**Option A — Forward/mount host Codex auth files into the world (read-only, policy-gated)**
- **Pros:** Preserves Codex-native auth format and semantics; avoids creating a new “auth env var” contract; keeps `cli:codex` adapter closer to upstream behavior; enables “subscription-first” UX by reusing the host login state.
- **Cons:** Requires a well-defined, auditable host→world file forwarding/mount mechanism; expands the set of sensitive artifacts readable inside the world.
- **Cascading implications:** Requires specifying allowed host source path(s), an in-world destination path contract, and the spawn-time pointer mechanism (Codex-specific env/flags; not gateway wiring).
- **Risks:** Misconfiguration could over-share host files into the world; credential paths could appear in logs; platform-specific differences (Lima/WSL/Linux) could cause confusing auth failures.
- **Unlocks:** A generalized “credential file forwarding” posture for other subscription CLIs that store auth state as files (later).
- **Quick wins / low-hanging fruit:** Start Codex-only with a narrow allowlist and read-only semantics; add status that reports “auth material present” without exposing details.

**Option B — Extract needed Codex auth fields on host and inject into in-world process env (no auth files in-world)**
- **Pros:** Reuses the env-injection delivery mechanism; avoids making auth files present inside world storage; can be implemented without a file-forwarding/mount subsystem.
- **Cons:** Requires defining Codex-specific injected env var names/shape (a new mini-contract); secret values live in process env/memory; may diverge from upstream Codex behavior over time.
- **Cascading implications:** Must define:
  - what exact auth fields are extracted,
  - the injected env var names,
  - rotation semantics (restart vs replace),
  - and strict redaction/caps rules for any errors.
- **Risks:** Env vars can be observable to same-user processes (OS property); accidental emission in debug logs; future Codex auth shape changes could break extraction logic.
- **Unlocks:** A single unified secret delivery mechanism for both `api:*` and `cli:*` backends (env injection).
- **Quick wins / low-hanging fruit:** Implement quickly for Codex-only without needing world filesystem forwarding.

**Recommendation**
- **Selected:** Option B — Extract needed Codex auth fields on host and inject into in-world process env (no auth files in-world)
- **Rationale (crisp):** Avoids adding a new cross-platform file-forwarding/mount mechanism in v1 while still keeping egress in-world; secret values stay out of Substrate YAML and are injected into the in-world gateway/manager process environment with strict redaction/caps. When the in-world gateway/manager spawns a Substrate-owned wrapper/engine process, it MUST propagate secrets to that child via FD/pipe rather than env vars when supported (see `docs/project_management/standards/SECRETS_DELIVERY_CHANNEL_RUBRIC.md` and `docs/project_management/next/llm_gateway_in_world/decision_register.md` (DR-0017)).

Phase 8 additive clarification:
- DR-0012 upgrades the preferred in-world delivery mechanism so secret values do not live in the in-world gateway/manager process environment by default (FD/pipe auth bundle), while keeping the host-side extraction posture intact.

---

### DR-0007 — Codex auth source on host for in-world injection (`cli:codex`)

**Decision owner(s):** Engine + World + Security  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** ADR-0024, ADR-0027 (“no secrets in Substrate YAML”), DR-0006

**Problem / Context**
- DR-0006 establishes that `cli:codex` auth is injected into the in-world gateway/manager process environment (no auth files in-world).
- Phase 8 additive clarification: DR-0012 upgrades the preferred in-world delivery mechanism to an FD/pipe auth bundle (no secret-bearing env vars in-world by default).
- We still need a deterministic way for host-side Substrate components to obtain the necessary auth values safely, without pushing operators toward exporting secrets broadly.

**Option A — Operator-provided env vars only (no host file reads)**
- **Pros:** Simple; no new “read host credential file” behavior; easy to reason about from a security standpoint.
- **Cons:** Worse UX; increases setup friction and operator error; encourages secrets to be exported broadly in shells/CI environments.
- **Cascading implications:** Document a required set of Codex auth env vars that must be present when running `substrate world sync gateway` (or when spawning the in-world CLI engine), including rotation expectations.
- **Risks:** Accidental leakage via shell history/process listings; brittle failures when env is missing/partial.
- **Unlocks:** Fastest implementation path with minimal host-side parsing logic.
- **Quick wins / low-hanging fruit:** Strict “missing env var” fail-closed errors that name variables only (never values).

**Option B — Read host Codex auth file by default + allow env override**
- **Pros:** Best UX; reuses existing Codex login state automatically; avoids telling users to export secrets; aligns with subscription-first posture.
- **Cons:** Introduces a narrow host file read that must be explicitly allowed/policy-gated and carefully redacted.
- **Cascading implications:** Must define:
  - allowed host source path(s) (e.g., `~/.codex/auth.json`),
  - extracted fields (minimal set required for in-world auth),
  - strict redaction/caps rules (never log values; avoid logging full paths when possible),
  - and env override behavior (explicit env vars take precedence over file reads).
- **Risks:** If the allowlist is too broad, it becomes an implicit secret reader; upstream file schema changes could break extraction (requires robust parsing + clear errors).
- **Unlocks:** “Subscription-first” feels native; the pattern generalizes to other CLIs later (read known auth file → inject minimal fields) without introducing file-forwarding/mount semantics.
- **Quick wins / low-hanging fruit:** Codex-only exact-path allowlist + schema validation + redaction; add a `substrate world status gateway` indicator like “codex auth available: yes/no” (no details).

**Recommendation**
- **Selected:** Option B — Read host Codex auth file by default + allow env override
- **Rationale (crisp):** Preserves subscription-first UX without requiring users to export secrets, while keeping host file access narrow, explicit, and policy-gated with strict redaction.

---

### DR-0008 — Policy gate for host credential reads (backend-generic)

**Decision owner(s):** Broker + Engine + Security  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** ADR-0027, DR-0006, DR-0007

**Problem / Context**
- DR-0007 allows host-side Substrate components to read a CLI’s existing login state (e.g., `~/.codex/auth.json`) to extract minimal auth fields for in-world injection (DR-0006).
- Host credential reads are security-sensitive and must not be an implicit side-effect of simply enabling a backend.
- The control must be generic (not Codex-specific) so additional `cli:*` adapters can be added later without reshaping the policy surface.

**Option A — Explicit, backend-generic policy allowlist (recommended)**
- **Pros:** Auditable and explicit: policy states which backend ids are permitted to perform host credential reads; composes with strict schema and `--explain` provenance; generic across future backends.
- **Cons:** Adds a new policy knob that must be documented and maintained.
- **Cascading implications:** Introduce `agents.host_credentials.read.allowed_backends: [<kind>:<name>]` (deny-by-default); empty means no backend may read host credential material; selection is per-backend id (e.g., `cli:codex` now; other `cli:*` later).
- **Risks:** Misconfiguration can block a backend unexpectedly; requires clear errors and `--explain` output that points to this gate.
- **Unlocks:** Clean path to add more CLI backends later while keeping a tight security posture (opt-in per backend).
- **Quick wins / low-hanging fruit:** Ship with default `[]` and include an “enable recipe” that adds only `cli:codex` when desired.

**Option B — No dedicated policy key; treat as implied by enabling a backend**
- **Pros:** Fewer policy keys; slightly faster to implement initially.
- **Cons:** Hidden capability creep: host credential reads become an implicit behavior; harder to audit, explain, and deny explicitly.
- **Cascading implications:** Documentation must describe host credential reads as adapter behavior; `--explain` cannot point to a dedicated gate.
- **Risks:** Surprising security posture; hard to unwind later without breaking users.
- **Unlocks:** Minimal schema churn.
- **Quick wins / low-hanging fruit:** None beyond documentation.

**Recommendation**
- **Selected:** Option A — Explicit, backend-generic policy allowlist
- **Rationale (crisp):** Host credential reads are sensitive enough to require an explicit, auditable gate; using backend ids keeps it generic for future `cli:*` adapters.

---

### DR-0009 — Host credential read source paths: backend-contract fixed vs policy-configured

**Decision owner(s):** Engine + Security  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** DR-0007, DR-0008, ADR-0027 (“strict schema” + policy surface ownership)

**Problem / Context**
- We allow host-side Substrate components to read a CLI’s existing login state by default (DR-0007) and we gate that action via policy (`agents.host_credentials.read.allowed_backends`, DR-0008).
- We still need to decide whether the set of host file paths Substrate may read is:
  - fixed by each backend adapter contract (least-privilege), or
  - configurable via policy (more flexible, higher footgun risk).
- This decision must remain backend-generic so additional `cli:*` adapters can be added without reshaping the policy surface repeatedly.

**Option A — Backend-contract fixed paths only**
- **Pros:** Least privilege by default; simplest to audit; avoids introducing a broad “paths allowlist” policy surface that is easy to accidentally widen.
- **Cons:** Less flexible for non-standard setups (custom credential locations, enterprise-managed paths); upstream path/schema changes require adapter updates.
- **Cascading implications:**
  - Each backend adapter documents its fixed host credential source path(s) (e.g., Codex may read `~/.codex/auth.json`).
  - Operators who cannot use the fixed path use DR-0007’s env override path (no host file reads) rather than expanding a path allowlist.
- **Risks:** Users with unusual environments may hit friction; platform quirks (different home layouts) must be handled carefully in adapters.
- **Unlocks:** Fast, safe v1 with minimal additional policy complexity; keeps credential-read behavior tightly scoped per backend id.
- **Quick wins / low-hanging fruit:** Implement fixed-path reads for `cli:codex` first with robust “missing file / schema mismatch” errors that never print secret values.

**Option B — Policy-configured allowed host paths**
- **Pros:** Maximum flexibility (enterprise path layouts, multiple auth locations, future CLIs with different layouts) without adapter changes.
- **Cons:** Higher footgun risk; requires defining a safe path-pattern language and canonicalization rules; increases policy surface area and review burden.
- **Cascading implications:**
  - Introduce an additional strict policy key that controls allowed read paths (exact paths or tightly-scoped patterns).
  - `--explain` provenance must attribute both the backend allowlist gate (DR-0008) and the path allowlist gate.
- **Risks:** Misconfigured patterns could unintentionally allow reading arbitrary sensitive files; harder to test/explain across platforms; drift between intent and actual file matches.
- **Unlocks:** Easier rollout of additional `cli:*` backends and enterprise environments without code changes.
- **Quick wins / low-hanging fruit:** Start with exact-path-only (no globs) and add patterns later if needed.

**Recommendation**
- **Selected:** Option A — Backend-contract fixed paths only
- **Rationale (crisp):** Keeps the host credential read surface minimal and auditable in v1 while still providing a safe escape hatch via env override, without adding a broad path allowlist knob.

---

### DR-0010 — Missing host credentials behavior: hard fail vs implicit fallback

**Decision owner(s):** Engine + Security  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** DR-0007, DR-0009, ADR-0027 (policy gating + strictness)

**Problem / Context**
- Host-side auth extraction uses backend-contract fixed paths by default (DR-0009), with explicit env override supported (DR-0007).
- We must define behavior when the default auth source is missing/unreadable or schema-invalid, without leaking secrets and while preserving fail-closed posture.

**Option A — Hard fail closed with actionable error (recommended)**
- **Pros:** Deterministic and auditable; secure fail-closed posture; avoids silent “works sometimes” behavior; easier debugging.
- **Cons:** Slightly more friction for first-time setup when the user expects existing login state but isn’t authenticated.
- **Cascading implications:** Error output MUST:
  - state that host credential material was not available/valid (without printing secret values),
  - point to the supported remediation paths (login to the CLI, or set explicit env override vars if supported),
  - and `--explain` MUST attribute gating decisions (including `agents.host_credentials.read.allowed_backends` when relevant).
- **Risks:** Users get stuck if error messaging is poor; mitigated by clear doctor/status output.
- **Unlocks:** Clean semantics when `llm.fail_closed.routing=true`; consistent behavior across platforms.
- **Quick wins / low-hanging fruit:** Add a `substrate world status gateway` indicator like `codex_auth: missing|present` (no details), plus a concise remediation hint.

**Option B — Soft fallback to env override path if file missing**
- **Pros:** Smoother UX in some environments where env override values are already present.
- **Cons:** Adds implicit behavior that can hide misconfiguration; encourages reliance on stale/broad env exports; harder to reason about and audit.
- **Cascading implications:** Must define precedence rules and how to report which source was used (without leaking).
- **Risks:** Debugging confusion; inconsistent behavior across shells/CI; accidental use of unexpected credentials.
- **Unlocks:** Slightly easier onboarding for some setups.
- **Quick wins / low-hanging fruit:** None; still requires careful reporting and redaction rules.

**Recommendation**
- **Selected:** Option A — Hard fail closed with actionable error
- **Rationale (crisp):** Keeps security + debuggability tight; env overrides should be explicit, not an implicit fallback.

---

### DR-0011 — Host credential read execution location: host-side preflight vs in-world on-demand

**Decision owner(s):** Engine + World + Security  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** ADR-0023, ADR-0024, ADR-0027, DR-0006..DR-0010

**Problem / Context**
- The `cli:codex` adapter posture reads host credential material (policy-gated) to extract minimal auth fields and inject them into the in-world process environment.
- We must decide where this credential read + extraction happens so the boundary stays auditable and consistent across platforms (Linux + Lima + WSL) without creating new secret RPC surfaces.

**Option A — Host-side shell/world subsystem performs credential preflight (at `substrate world sync gateway`)**
- **Pros:** Clear boundary (host reads host, world never sees host FS); aligns with the gateway lifecycle entrypoint; easy to enforce policy gates before any read; works even if the in-world manager is down (because it is part of bringing it up).
- **Cons:** Requires host-side implementation work (though conceptually uniform across platforms).
- **Cascading implications:** `substrate world sync gateway` performs: policy gate checks → host auth read/extract → spawn/restart in-world gateway/engine with injected env (no persistence).
- **Risks:** If host-side preflight is buggy, gateway start fails; mitigated by actionable errors + `--explain`.
- **Unlocks:** Deterministic lifecycle/rotation semantics; keeps secrets out of world storage; avoids introducing a bidirectional secret request protocol.
- **Quick wins / low-hanging fruit:** Reuse the existing env injection mechanics already chosen for `api:*` backends (DR-0007 in the gateway decision register).

**Option B — In-world manager requests credentials from host on-demand**
- **Pros:** Centralizes logic inside the gateway/manager.
- **Cons:** Requires a new bidirectional secret request protocol; adds caching/rotation semantics; larger security review surface; increases risk of accidental persistence/logging inside world-agent/gateway layers.
- **Cascading implications:** Define and secure a host↔world secret RPC, including policy evaluation, audit events, and redaction guarantees.
- **Risks:** Complexity + security footguns; harder to keep “no secrets persisted” story crisp.
- **Unlocks:** Potentially less host-side orchestration code in the long run.
- **Quick wins / low-hanging fruit:** None; this is a larger design jump.

**Recommendation**
- **Selected:** Option A — Host-side shell/world subsystem performs credential preflight
- **Rationale (crisp):** Keeps the boundary simple and auditable (host reads host; world receives injected values at spawn time), avoids inventing a new secret RPC surface, and aligns with `substrate world sync gateway` as the lifecycle entrypoint.

---

### DR-0012 — Host→world delivery of `cli:codex` auth to in-world gateway/manager: env injection vs FD/pipe auth bundle

**Decision owner(s):** Engine + World + Security  
**Date:** 2026-02-13  
**Status:** Accepted  
**Related docs:** `docs/project_management/standards/SECRETS_DELIVERY_CHANNEL_RUBRIC.md`, `docs/project_management/next/llm_gateway_in_world/decision_register.md` (DR-0018)

**Problem / Context**
- `cli:codex` auth material is sourced on the host (from policy-gated host credential reads and/or explicit env overrides).
- v1 describes injecting those secret values into the in-world gateway/manager process environment at spawn time.
- Phase 8 requires an additive upgrade so secret values do not live in in-world process environments by default.

**Option A — Continue v1: inject secret values into the in-world gateway/manager process environment**
- **Pros:** Minimal change; aligns with earlier v1 language.
- **Cons:** Secret values are present in the in-world process environment; higher accidental disclosure risk; conflicts with “FD/pipe by default” rubric posture when Substrate spawns the consumer.

**Option B — Preferred: deliver auth via an inherited one-time FD/pipe bundle into the in-world gateway/manager (recommended)**
- **Pros:** Keeps secrets out of in-world process env by default; scopes auth to the intended consumer; aligns with the cross-track secrets rubric and the gateway host→world decision (DR-0018).
- **Cons:** Requires a small amount of spawn plumbing and a stable “bundle payload” contract.
- **Cascading implications:**
  - The host→world gateway spawn request MUST treat auth values as a secret-channel payload (never logged/printed).
  - The world-agent MUST deliver those values to the in-world gateway/manager via a one-time FD/pipe auth bundle.
  - An env var MAY be used to convey the FD number (non-secret; safe to print), e.g. `SUBSTRATE_LLM_AUTH_BUNDLE_FD: int`.
  - The bundle keys MUST use the canonical `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_*` field names so redaction/caps rules remain uniform with legacy env injection.

**Recommendation**
- **Selected:** Option B — FD/pipe auth bundle into the in-world gateway/manager.
- **Rationale (crisp):** Substrate spawns the in-world gateway/manager and can provide auth via an inherited one-time FD/pipe channel, avoiding secret-bearing env vars while preserving strict, policy-gated host sourcing.
