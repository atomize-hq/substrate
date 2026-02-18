# Decision Register — World-Agent Policy Snapshot

This decision register scopes decisions required to align world-agent enforcement with the Substrate
effective policy model while removing concurrency hazards from process-global broker state.

Related ADR:
- `docs/project_management/adrs/implemented/ADR-0014-world-agent-policy-resolution-and-concurrency.md`
- `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md`

---

### DR-0001 — World-Agent Caller Authorization Boundary

**Decision owner(s):** Substrate core team  
**Date:** 2026-01-18  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0014-world-agent-policy-resolution-and-concurrency.md`

**Problem / Context**
- World-agent is a privileged system component (root service on Linux) accessed over a local
  transport (Linux UDS `/run/substrate.sock`, forwarded transports on macOS/WSL).
- Policy snapshot ingestion expands the effective “control surface” of the agent, so the caller
  authorization boundary must be explicit and testable.

**Option A — OS-level authorization (socket ACL is the boundary)**
- **Pros:** Matches current deployment (root service + `root:substrate 0660` socket), minimal new surface area, clear operator model.
- **Cons:** Multi-user hosts require correct group membership hygiene; any member of the socket group can submit requests.
- **Cascading implications:** Policy snapshot trust is tied to socket access; audit requirements shift to socket permissions and group membership.
- **Risks:** Misconfigured socket permissions can widen access to privileged execution.
- **Unlocks:** Enables policy snapshot rollout without introducing a new auth subsystem.
- **Quick wins / low-hanging fruit:** Add explicit docs and doctor output that treat socket ACL as the boundary.

**Option B — In-protocol authorization (caller identity verified per request)**
- **Pros:** Enables per-user policy selection on shared agents; provides a path to multi-tenant correctness.
- **Cons:** Requires transport-specific identity plumbing (SO_PEERCRED on Linux UDS; forwarded identity on macOS/WSL), increases complexity and failure modes.
- **Cascading implications:** Requires a stable identity contract across transports; impacts agent-api-types schemas and doctor reporting.
- **Risks:** Identity spoofing bugs become high severity.
- **Unlocks:** Allows a shared system agent with per-user policy isolation.
- **Quick wins / low-hanging fruit:** None without a full threat model and transport audit.

**Recommendation**
- **Selected:** Option A — OS-level authorization (socket ACL is the boundary)
- **Rationale (crisp):** Substrate already provisions access to the root-run agent via a dedicated socket group; policy snapshot alignment requires consistent policy semantics first, not a new auth system.

**Follow-up tasks (explicit)**
- Update doctor outputs to include socket ownership/mode and group membership guidance.
- Document that socket ACL is the authorization boundary for world-agent requests.

---

### DR-0002 — Policy Snapshot Trust Model

**Decision owner(s):** Substrate core team  
**Date:** 2026-01-18  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0014-world-agent-policy-resolution-and-concurrency.md`

**Problem / Context**
- Current world-agent derives enforcement inputs by reading broker state inside the agent process.
- This is incompatible with guest deployments (macOS Lima / Windows WSL) where host-global policy is
  not reliably visible, and it is unsafe under concurrent requests due to shared mutable broker state.

**Option A — Host-resolved policy snapshot is authoritative for enforcement**
- **Pros:** Single source of truth for policy merge; works when agent cannot see host-global policy; eliminates the global broker concurrency hazard when enforcement uses only snapshot inputs.
- **Cons:** Requires adding snapshot fields to the agent API and plumbing through shell routing.
- **Cascading implications:** World-agent must reject or ignore local policy resolution for enforcement-relevant decisions when a snapshot is provided.
- **Risks:** If an unauthorized caller can submit requests, they can submit permissive snapshots; this is bounded by DR-0001.
- **Unlocks:** Enables deterministic QA of policy behavior across platforms and backends.
- **Quick wins / low-hanging fruit:** Fix the Linux “global patch mismatch” immediately by aligning service `SUBSTRATE_HOME`, then pivot to snapshots for full parity.

**Option B — World-agent resolves policy locally (no snapshot ingestion)**
- **Pros:** No API/schema changes; keeps policy “near enforcement”.
- **Cons:** Cannot guarantee parity on macOS/WSL guests; preserves concurrency hazard; continues ambiguity around global policy home on a system service.
- **Cascading implications:** Requires making `$SUBSTRATE_HOME` meaningful inside the agent across platforms, which is not a stable contract.
- **Risks:** Policy drift across components remains an evergreen regression class.
- **Unlocks:** None beyond avoiding near-term schema changes.
- **Quick wins / low-hanging fruit:** None; the root cause remains.

**Recommendation**
- **Selected:** Option A — Host-resolved policy snapshot is authoritative for enforcement
- **Rationale (crisp):** Policy merge is a host-facing contract and must be consistent across execution paths; snapshots remove both guest visibility issues and the shared-state concurrency hazard.

**Follow-up tasks (explicit)**
- Extend `crates/agent-api-types::ExecuteRequest` with a versioned policy snapshot payload.
- Update shell routing to attach the snapshot to world-agent requests.
- Update world-agent to use the snapshot (not process-global broker state) for enforcement inputs.

---

### DR-0003 — Policy Snapshot Payload Shape (Minimum Necessary)

**Decision owner(s):** Substrate core team  
**Date:** 2026-01-18  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0014-world-agent-policy-resolution-and-concurrency.md`

**Problem / Context**
- World-agent needs policy-derived inputs for filesystem isolation (mode/isolation/allowlists) and
  network allowlists (allowed domains). It does not need command allow/deny decisions, which are
  enforced in the host shell/shim layer.
- The Substrate policy file remains the authoritative user-facing contract. The “policy snapshot”
  is a transport payload for world-agent enforcement inputs, not a replacement policy model.

**Option A — Minimal enforcement snapshot (world + network + limits only)**
- **Pros:** Small, stable surface area; avoids duplicating command policy in the agent; reduces drift and version skew impact.
- **Cons:** Requires clear definitions for what is “enforcement-relevant” at the agent.
- **Cascading implications:** Host must continue to enforce command allow/deny in broker; agent enforces only world/network/limits.
- **Risks:** If the boundary is unclear, future features may accidentally reintroduce command policy into the agent.
- **Unlocks:** Makes snapshot evolution tractable with fewer breaking changes.
- **Quick wins / low-hanging fruit:** Implement with new `PolicySnapshotV1` struct and a strict schema version.

**Option B — Full effective policy snapshot (including cmd_* fields)**
- **Pros:** A single payload mirrors the CLI `policy current show` output.
- **Cons:** Encourages duplicated command enforcement in the agent; increases attack surface and complexity; increases future coupling.
- **Cascading implications:** Requires explicit policy precedence between host broker and agent broker for command enforcement.
- **Risks:** Divergent implementations create inconsistent allow/deny behavior.
- **Unlocks:** None required for current world-agent enforcement needs.
- **Quick wins / low-hanging fruit:** None.

**Recommendation**
- **Selected:** Option A — Minimal enforcement snapshot (world + network + limits only)
- **Rationale (crisp):** World-agent must enforce isolation primitives, not re-implement the broker; minimizing the snapshot prevents redundant policy surfaces.

**Follow-up tasks (explicit)**
- Define `PolicySnapshotV1` containing: `world_fs` (mode/isolation/require_world/read_allowlist/write_allowlist), `net_allowed`, `limits`.
- Update tracing schema to record snapshot metadata (hash/version) on completion spans.
- Add an integration test that proves `cmd_denied` remains enforced by the host broker before world-agent execution (exit `126` in enforce mode), and that world-agent enforcement remains limited to world/network/limits.

---

### DR-0004 — Where Allowlist Canonicalization Runs

**Decision owner(s):** Substrate core team  
**Date:** 2026-01-18  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0014-world-agent-policy-resolution-and-concurrency.md`

**Problem / Context**
- The world-agent currently resolves allowlist patterns into:
  - writable mount prefixes (`SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST`)
  - Landlock read/write path allowlists (`SUBSTRATE_WORLD_FS_LANDLOCK_*`)
  using the agent’s computed `project_dir` (guest-visible path semantics).

**Option A — Host sends merged patterns; agent canonicalizes to enforcement inputs**
- **Pros:** Correct under guest path mappings; reuses existing world-agent resolution logic; host remains the canonical policy merge point.
- **Cons:** Agent still implements canonicalization logic and must keep it consistent with the policy contract.
- **Cascading implications:** The snapshot must include the merged pattern lists (not already-resolved guest paths).
- **Risks:** Canonicalization bugs remain in the agent, but they become purely deterministic transformations of snapshot inputs.
- **Unlocks:** Cross-platform parity without host needing to understand guest filesystem layout.
- **Quick wins / low-hanging fruit:** Replace broker reads in world-agent with snapshot reads while preserving existing resolver functions.

**Option B — Host sends fully resolved prefixes/paths; agent applies them directly**
- **Pros:** Agent becomes a pure executor; fewer policy semantics in the agent.
- **Cons:** Host must know guest path layout (Lima/WSL); increases coupling to backend-specific path mapping; brittle across platforms.
- **Cascading implications:** Host must compute both `/project/...` and host-absolute paths used inside the cage.
- **Risks:** Path mapping drift becomes a frequent cross-platform regression.
- **Unlocks:** None required for current parity goals.
- **Quick wins / low-hanging fruit:** None.

**Recommendation**
- **Selected:** Option A — Host sends merged patterns; agent canonicalizes to enforcement inputs
- **Rationale (crisp):** The agent is the only component that can reliably interpret paths in its own mount namespace and cage environment.

**Follow-up tasks (explicit)**
- Add snapshot fields for `read_allowlist` and `write_allowlist` patterns (already merged).
- Update world-agent code paths to compute prefixes/paths from snapshot patterns, not from broker state.

---

### DR-0005 — Snapshot Schema Versioning and Upgrade Behavior

**Decision owner(s):** Substrate core team  
**Date:** 2026-01-18  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0014-world-agent-policy-resolution-and-concurrency.md`

**Problem / Context**
- Shell and world-agent versions can be skewed (release bundles, dev installs, partial upgrades).
- Policy snapshot payloads must be forward-compatible enough to avoid silent mis-enforcement.

**Option A — Explicit schema version; strict validation; legacy fallback path**
- **Pros:** Deterministic behavior; prevents silent parsing drift; supports staged rollout by allowing older clients to keep working temporarily.
- **Cons:** Requires defining fallback behavior and a deprecation window.
- **Cascading implications:** World-agent must surface a clear error when it cannot interpret a snapshot; shell must surface that error deterministically.
- **Risks:** If fallback is retained indefinitely, the old concurrency hazard persists for legacy clients.
- **Unlocks:** Enables safe rollout across Linux/macOS/WSL without requiring perfectly synchronized upgrades.
- **Quick wins / low-hanging fruit:** Introduce `policy_snapshot.schema_version = 1` and treat absence as “legacy mode” for a bounded period.

**Option B — Unversioned snapshot fields (best-effort parsing)**
- **Pros:** Fastest to implement.
- **Cons:** Silent misinterpretation is plausible; QA cannot reliably assert which semantics were applied.
- **Cascading implications:** Future schema evolution is constrained by backward ambiguity.
- **Risks:** High; security posture degrades under drift.
- **Unlocks:** None.
- **Quick wins / low-hanging fruit:** None.

**Recommendation**
- **Selected:** Option A — Explicit schema version; strict validation; legacy fallback path
- **Rationale (crisp):** Isolation enforcement must be deterministic; strict versioning prevents silent behavior drift while enabling controlled rollout.

**Follow-up tasks (explicit)**
- Extend `agent-api-types` with `PolicySnapshotV1 { schema_version: 1, ... }`.
- Add a world-agent response field or trace metadata indicating `policy_resolution_mode: snapshot_v1 | legacy_local`.
- Add a deprecation spec that removes legacy local policy resolution in world-agent after a defined release window.

---

### DR-0006 — Policy Cache Refresh Semantics (Host Resolver)

**Decision owner(s):** Substrate core team  
**Date:** 2026-01-18  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0014-world-agent-policy-resolution-and-concurrency.md`

**Problem / Context**
- The host is the canonical policy resolver (DR-0002), so the host must define a deterministic
  “policy refresh” contract when policy patch files change.
- The policy model already supports external edits (user edits YAML directly, or runs `substrate
  policy ... set` in another terminal). The shell must apply the updated policy without requiring a
  restart contract.

**Option A — Resolve effective policy for every executed command**
- **Pros:** Deterministic and simple; no cache invalidation rules; matches the current shell behavior of refreshing policy for the current cwd before execution.
- **Cons:** More filesystem reads (global + workspace patch) on command-heavy workflows.
- **Cascading implications:** The host must attach a fresh policy snapshot to each world-agent request.
- **Risks:** Performance regressions in extremely high-rate command streams; mitigate by measuring before optimizing.
- **Unlocks:** Eliminates an entire class of “stale policy cache” regressions during policy refactors.
- **Quick wins / low-hanging fruit:** None required; this is a behavioral contract, not a new optimization.

**Option B — Host caches effective policy with deterministic invalidation**
- **Pros:** Reduces filesystem reads; can improve interactive throughput.
- **Cons:** Requires explicit invalidation rules that are correct across workspace boundaries.
- **Cascading implications:** Requires defining a cache key (workspace root + policy patch paths) and invalidation triggers that guarantee next-command visibility.
- **Risks:** Stale policy causes incorrect enforcement; this is a security regression class.
- **Unlocks:** Performance headroom for future high-frequency workflows.
- **Quick wins / low-hanging fruit:** Implement “stat + mtime/size change detection” before each command.

**Recommendation**
- **Selected:** Option B — Host caches effective policy with deterministic invalidation
- **Rationale (crisp):** Policy changes must take effect on the immediate next command, and caching is still valuable; a cache that performs change detection before each command preserves the user contract while avoiding redundant parsing when inputs are unchanged.

**Follow-up tasks (explicit)**
- Define the exact policy cache contract in the spec:
  - The shell must compute a cache key from: workspace root (or “no workspace”), global policy patch path, workspace policy patch path.
  - Before each executed command, the shell must `stat` the policy patch file(s) and invalidate the cache when metadata changes (mtime or size).
  - When `substrate policy global|workspace set|reset|init` runs in the same process, the shell must invalidate the cache after the write completes.
- Add trace fields to record the policy snapshot hash so QA can detect policy drift across executions.
- Add an integration test that edits a policy patch file and proves the change takes effect on the next command without restarting the shell.

---

### DR-0007 — Multi-User Linux Deployment Model

**Decision owner(s):** Substrate core team  
**Date:** 2026-01-18  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0014-world-agent-policy-resolution-and-concurrency.md`

**Problem / Context**
- Linux currently provisions a systemd system service (`root`), shared by the `substrate` group.
- Policy snapshot direction must define the supported multi-user posture.

**Option A — Per-user world-agent service (systemd user units)**
- **Pros:** Eliminates shared-agent multi-tenant concerns; aligns agent process environment with the invoking user’s home by construction.
- **Cons:** Requires provisioning changes (lingering, per-user sockets), changes failure modes, increases support surface across distros.
- **Cascading implications:** Socket discovery must become per-user; scripts and doctor output must change materially.
- **Risks:** Operational complexity increases; provisioning becomes more fragile on locked-down systems.
- **Unlocks:** Strong per-user isolation for policy and execution metadata.
- **Quick wins / low-hanging fruit:** None; it is a platform-level re-architecture.

**Option B — Shared system service remains supported; snapshots remove per-user policy coupling**
- **Pros:** Matches current install model; avoids provisioning churn; snapshot model removes dependence on the agent’s own policy home.
- **Cons:** Multi-user systems depend on correct group membership; policy enforcement remains user-driven rather than centrally administered.
- **Cascading implications:** Authorization boundary remains the socket ACL (DR-0001).
- **Risks:** Misconfigured group membership can widen access to privileged execution.
- **Unlocks:** Cross-platform parity with minimal provisioning changes.
- **Quick wins / low-hanging fruit:** Maintain existing socket group hygiene guidance and enhance doctor outputs.

**Recommendation**
- **Selected:** Option B — Shared system service remains supported; snapshots remove per-user policy coupling
- **Rationale (crisp):** Substrate already operates with a shared agent model on Linux; snapshots decouple per-user policy state without requiring a service model migration.

**Follow-up tasks (explicit)**
- Update provisioning docs to treat the socket group as the access boundary and to document the implications on multi-user hosts.
- Add an explicit “multi-user posture” section to the policy snapshot spec.

---

### DR-0008 — Observability Contract for Policy Snapshots

**Decision owner(s):** Substrate core team  
**Date:** 2026-01-18  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0014-world-agent-policy-resolution-and-concurrency.md`, `docs/TRACE.md`

**Problem / Context**
- QA and operators need to know which policy semantics were applied for a given execution.
- The new snapshot path must be visible in traces and doctor outputs.

**Option A — Emit explicit snapshot metadata in trace and doctor outputs**
- **Pros:** Deterministic QA; supports incident triage; enables replay tooling to detect policy drift vs recorded inputs.
- **Cons:** Requires extending trace schema and doctor payloads.
- **Cascading implications:** Trace consumers must tolerate new fields; redaction rules must apply to snapshot content and hashes.
- **Risks:** Logging raw policy content can leak sensitive patterns; mitigate by logging a hash and provenance, not full contents.
- **Unlocks:** Enables automated regression checks that assert `policy_resolution_mode` and snapshot hash presence.
- **Quick wins / low-hanging fruit:** Add `policy_resolution_mode` + `policy_snapshot_schema` + `policy_snapshot_hash` to completion spans.

**Option B — No new observability fields (implicit behavior)**
- **Pros:** No schema changes.
- **Cons:** QA cannot assert correctness; drift becomes invisible and hard to debug.
- **Cascading implications:** Makes replay and audit weaker.
- **Risks:** High; regressions persist.
- **Unlocks:** None.
- **Quick wins / low-hanging fruit:** None.

**Recommendation**
- **Selected:** Option A — Emit explicit snapshot metadata in trace and doctor outputs
- **Rationale (crisp):** Substrate’s core promise is observable, replayable execution; policy application must be visible and testable.

**Follow-up tasks (explicit)**
- Extend trace schema to record snapshot metadata (no raw policy content).
- Extend `GET /v1/doctor/world` and `substrate world doctor --json` to report whether snapshot enforcement is supported and active.

---

### DR-0009 — Command Policy Enforcement Location

**Decision owner(s):** Substrate core team  
**Date:** 2026-01-18  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0014-world-agent-policy-resolution-and-concurrency.md`, `docs/BROKER.md`

**Problem / Context**
- The policy file includes command controls (`cmd_allowed`, `cmd_denied`, `cmd_isolated`,
  `require_approval`, `allow_shell_operators`).
- The snapshot direction changes how world-agent receives policy-derived inputs; it must not
  accidentally move command gating into the agent or create conflicting enforcement sites.

**Option A — Command policy is enforced by the host broker only**
- **Pros:** Matches current architecture: shims + shell consult broker and can deny before execution; avoids duplicating command parsing/enforcement in the agent.
- **Cons:** Direct calls to world-agent (bypassing Substrate shell/shims) are not subject to command allow/deny enforcement.
- **Cascading implications:** The snapshot must not include `cmd_*` fields; world-agent remains an execution backend, not a policy gate.
- **Risks:** Operators may assume world-agent enforces command policy for all callers; docs must make the boundary explicit.
- **Unlocks:** Keeps the snapshot payload small and reduces future coupling.
- **Quick wins / low-hanging fruit:** Document the enforcement boundary in the spec and in world-agent docs.

**Option B — Defense-in-depth: world-agent also enforces command policy**
- **Pros:** Direct agent API callers are still constrained by policy; reduces “bypass the shell” concerns.
- **Cons:** Requires passing command policy fields and implementing consistent parsing/semantics in the agent; increases API surface area and versioning burden.
- **Cascading implications:** Requires defining precedence between host broker decisions and agent broker decisions.
- **Risks:** Divergent enforcement semantics become a high-severity regression class.
- **Unlocks:** Stronger multi-tenant posture if world-agent becomes an explicit shared execution API.
- **Quick wins / low-hanging fruit:** None; this is a large scope increase.

**Recommendation**
- **Selected:** Option A — Command policy is enforced by the host broker only
- **Rationale (crisp):** Substrate’s policy gate is the broker in the shell/shim layer; keeping command enforcement out of world-agent avoids duplicated semantics and preserves a single point of truth.

**Follow-up tasks (explicit)**
- Update the snapshot spec to explicitly exclude `cmd_*` keys and state where they are enforced.
- Add a manual testing section that demonstrates `cmd_denied` behavior is unchanged by the snapshot work.

---

### DR-0010 — Config Cache Parity With Policy Cache

**Decision owner(s):** Substrate core team  
**Date:** 2026-01-18  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`, `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`

**Problem / Context**
- The Substrate operator contract treats config and policy as patch-merged, scope-aware inputs that can be updated via CLI (`config set`, `policy set`) or by editing YAML files.
- If the shell introduces caching for policy resolution, config must maintain parity: config changes must take effect on the immediate next command without requiring a restart contract.
- World-deps work (ADR-0011) increases the importance of config correctness in high-frequency workflows.

**Option A — Cache policy only; leave config resolution unchanged**
- **Pros:** Smaller scope for the snapshot track; avoids touching config code paths.
- **Cons:** Creates a semantic mismatch between policy and config update visibility; increases QA burden and operator confusion.
- **Cascading implications:** Requires documenting separate refresh behaviors for config and policy.
- **Risks:** Drift between config/policy semantics becomes a recurring regression class.
- **Unlocks:** None beyond reducing near-term work.
- **Quick wins / low-hanging fruit:** None.

**Option B — Apply the same cache/refresh contract to config resolution**
- **Pros:** Preserves semantic parity between config and policy; enables consistent QA expectations; avoids “one updates immediately, the other is sticky” behavior.
- **Cons:** Requires defining a config cache key and invalidation triggers similar to policy; increases implementation scope.
- **Cascading implications:** Shell must `stat` relevant config patch files (global config and workspace config) before each executed command and invalidate caches on changes.
- **Risks:** Incorrect invalidation can apply stale config and break world-deps behavior.
- **Unlocks:** Clear operator contract across config + policy; reduces future refactor risk.
- **Quick wins / low-hanging fruit:** Reuse the same “cache key + metadata change detection” mechanism used for policy.

**Recommendation**
- **Selected:** Option B — Apply the same cache/refresh contract to config resolution
- **Rationale (crisp):** Config and policy are both patch-merged operator inputs; parity requires that changes to either are visible on the immediate next command without restart.

**Follow-up tasks (explicit)**
- Extend the snapshot spec to include the config refresh contract (global + workspace patch paths, cache keys, and invalidation rules).
- Add an integration test that edits `config.yaml` and `workspace.yaml` and proves the next command reflects the new config without restarting the shell.

---

### DR-0011 — Cross-Platform Behavioral Contract for Policy Snapshots

**Decision owner(s):** Substrate core team  
**Date:** 2026-01-18  
**Status:** Accepted  
**Related docs:** `docs/ISOLATION_SUPPORT_MATRIX.md`, `docs/project_management/adrs/implemented/ADR-0014-world-agent-policy-resolution-and-concurrency.md`

**Problem / Context**
- Policy snapshots exist to make policy semantics consistent across execution paths and platforms (Linux native, macOS Lima, Windows WSL).
- The world-agent runs in different places depending on platform (host vs guest), and local policy file visibility differs across those environments.
- A behavioral contract is required so QA can validate parity and operators can predict outcomes when snapshots are unavailable or unsupported.

**Option A — Strict parity contract with explicit fallback**
- **Pros:** QA can assert deterministic behavior across platforms; removes reliance on guest visibility of `$SUBSTRATE_HOME`; matches the snapshot motivation.
- **Cons:** Requires capability gating and explicit error paths when snapshot enforcement is unavailable.
- **Cascading implications:** `substrate world doctor --json` and trace metadata must report whether snapshot enforcement was used; the shell must fail closed when policy requires world execution and the agent cannot honor snapshots.
- **Risks:** Misconfigured deployments can fail more often until provisioning is corrected; this is preferable to silent drift.
- **Unlocks:** Enables macOS/WSL parity without copying host-global policy files into the guest.
- **Quick wins / low-hanging fruit:** Add a single, explicit trace field for `policy_resolution_mode` and a stable doctor flag for snapshot support.

**Option B — Platform-specific best-effort with documented drift**
- **Pros:** Minimizes short-term strictness; fewer errors in partially provisioned environments.
- **Cons:** QA cannot validate uniform semantics; policy behavior can differ by platform even when using the same policy file.
- **Cascading implications:** Documentation must enumerate per-platform deviations and keep them in sync with code; drift becomes an expected maintenance cost.
- **Risks:** Silent drift becomes a recurring regression class, including in security-sensitive isolation behavior.
- **Unlocks:** None aligned with Substrate’s “observable and replayable” goals.
- **Quick wins / low-hanging fruit:** None.

**Recommendation**
- **Selected:** Option A — Strict parity contract with explicit fallback
- **Rationale (crisp):** Snapshots exist to eliminate platform-dependent policy resolution; strict parity with explicit fallback keeps behavior deterministic and testable.

**Follow-up tasks (explicit)**
- Add a platform parity section to the snapshot spec that states:
  - The host must attach a policy snapshot to world-agent requests on every platform where world-agent is used.
  - The world-agent must ignore local policy resolution for enforcement inputs when a valid snapshot is present.
  - When a snapshot is missing or invalid, the shell must record `policy_resolution_mode` in trace output and must fail closed if policy requires world execution.
- Update doctor outputs to report snapshot support and last-known `policy_resolution_mode`.
- Add smoke validation that runs on Linux/macOS/WSL and asserts the same policy file produces the same enforcement behavior (filesystem write allowlist, net allowlist, and limits once enforced).
