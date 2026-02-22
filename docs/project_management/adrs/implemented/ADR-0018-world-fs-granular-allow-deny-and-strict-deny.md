# ADR-0018 — World FS Granular Allow/Deny (Read/Discover/Write) + Strict Deny (Full Isolation Only)

## Status
- Status: Accepted
- Date (UTC): 2026-01-29
- Owner(s): spenser, Substrate maintainers

## Scope
- Feature directory: `docs/project_management/_archived/world-fs-granular-allow-deny/`
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
  - `docs/project_management/adrs/implemented/ADR-0006-env-var-taxonomy-and-override-split.md`

## Related Docs
- Plan: `docs/project_management/_archived/world-fs-granular-allow-deny/plan.md`
- Spec manifest: `docs/project_management/_archived/world-fs-granular-allow-deny/spec_manifest.md`
- Decision Register: `docs/project_management/_archived/world-fs-granular-allow-deny/decision_register.md`
- Schema (authoritative): `docs/project_management/_archived/world-fs-granular-allow-deny/SCHEMA.md`
- Protocol (authoritative): `docs/project_management/_archived/world-fs-granular-allow-deny/PROTOCOL.md`
- Env var contract (authoritative): `docs/project_management/_archived/world-fs-granular-allow-deny/ENV.md`
- Impact map: `docs/project_management/_archived/world-fs-granular-allow-deny/impact_map.md`
- Manual playbook: `docs/project_management/_archived/world-fs-granular-allow-deny/manual_testing_playbook.md`
- Appendix implementation Planning Pack (Appendix A + B):
  - Plan: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/plan.md`
  - Spec manifest: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/spec_manifest.md`
  - Schema (authoritative): `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md`
  - Contract: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/contract.md`
- Appendix add-on Planning Pack (post-Appendix contract drift closures):
  - Plan: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/plan.md`
  - Spec manifest: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/spec_manifest.md`
  - Impact map: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/impact_map.md`
  - Tasks: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/tasks.json`
  - Manual playbook: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/manual_testing_playbook.md`
- Related ADRs / grounding:
  - Policy snapshot direction and threat model: `docs/project_management/adrs/implemented/ADR-0014-world-agent-policy-resolution-and-concurrency.md`
  - Full isolation mount/exec chokepoint: `crates/world/src/exec.rs`
  - Landlock exec wrapper: `crates/world-agent/src/internal_exec.rs`
  - Snapshot resolution on host + drift handling (REPL): `crates/shell/src/execution/policy_snapshot.rs`, `crates/shell/src/repl/async_repl.rs`
  - Full isolation Landlock overlayfs compatibility: `docs/project_management/adrs/implemented/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 2666072c19bbbc3e87633e97c357fc93ef0617688c518b83a1dd6debdb7b4896
### Changes (operator-facing)
- Add granular `allow_list` + `deny_list` for world filesystem reads/writes (and optional directory visibility)
  - Existing: `world_fs.read_allowlist` / `world_fs.write_allowlist` are allowlist-only; invalid patterns (e.g., `..`) can be accepted but ignored; there is no deny list; “allow all except secrets” cannot be expressed.
  - New: `world_fs.read|discover|write.{allow_list,deny_list}` with explicit deny-overrides-allow semantics in `world_fs.isolation=full`.
  - Why: Operators need explicit, enforceable “allow all except X” controls and reliable failure on invalid patterns (no silent ignore).
  - Links:
    - `docs/project_management/_archived/world-fs-granular-allow-deny/SCHEMA.md`
    - `docs/project_management/_archived/world-fs-granular-allow-deny/manual_testing_playbook.md`

- Make deny enforcement a true security boundary in full isolation via `world_fs.enforcement=strict`
  - Existing: Any mount-based masking (if added) would be bypassable if the workload can later `umount`/`mount` in its namespace; Landlock cannot subtract an exception once `.` is allowed.
  - New: In `world_fs.isolation=full`, deny rules are enforced via mount masking plus a strict post-setup lockdown that prevents the workload from undoing denies (cap drop + mount syscall blocking).
  - Why: Deny rules are intended to protect secrets under compromise, not only prevent accidental reads.
  - Links:
    - `docs/project_management/_archived/world-fs-granular-allow-deny/ENV.md`
    - `docs/project_management/_archived/world-fs-granular-allow-deny/impact_map.md`

- Break policy snapshot and policy YAML schemas (no backwards compatibility)
  - Existing: World-agent accepts `PolicySnapshotV1` with `read_allowlist`/`write_allowlist`, and YAML/patch formats match that.
  - New: Introduce `PolicySnapshotV2` and a V2 policy YAML schema; old keys and old snapshots become hard errors.
  - Why: This body of work cannot be expressed safely in the V1 shape; “accepted but not enforced” must be structurally impossible.
  - Links:
    - `docs/project_management/_archived/world-fs-granular-allow-deny/PROTOCOL.md`
    - `docs/project_management/_archived/world-fs-granular-allow-deny/SCHEMA.md`

## Problem / Context
- Operators need “deny overrides allow” (e.g., allow `.` but deny `./secrets/**`) to prevent accidental or malicious access to sensitive project content.
- Landlock (allowlist-only) cannot express “allow everything except X” once broad allows are granted.
- The current codebase has a known foot-gun: allowlist patterns containing `..` can be accepted by higher layers but are silently ignored during allowlist resolution, which can disable enforcement while policy *appears* set.
  - See: `crates/world-agent/src/service.rs` (`resolve_landlock_allowlist_paths` drops `..` segments).
- Full isolation command execution routes through a single chokepoint:
  - `unshare --mount … sh -c PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT` → conditional Landlock exec wrapper (`__substrate_world_landlock_exec`) → `sh -c/-lc $SUBSTRATE_INNER_CMD`.
  - See: `crates/world/src/exec.rs`, `crates/world-agent/src/internal_exec.rs`.
- If Substrate introduces mount-based deny masking without preventing later mount/umount by the workload, deny rules are bypassable in an adversarial model.

## Goals
- Provide a new `world_fs` policy contract with:
  - `read`, `discover`, `write` dimensions, each supporting `allow_list` and `deny_list`.
  - Explicit “deny overrides allow” semantics in `world_fs.isolation=full`.
  - A strict enforcement mode where denies are a hard security boundary.
- Remove “accepted but ignored” policy states:
  - Invalid patterns are rejected (hard error) at config/policy resolution time and at snapshot ingestion time.
- Keep behavior deterministic in PTY and non-PTY routes:
  - The same `policy_snapshot` schema and enforcement logic is used for `/v1/execute` and `/v1/stream start_session`.
  - Existing REPL snapshot drift handling continues to work after migrating snapshot schema versions.

## Non-Goals
- Supporting strict deny enforcement in `world_fs.isolation=workspace` (this ADR explicitly forbids it).
- Guaranteeing “dynamic” wildcard denies within a single long-running command (e.g., creating `x.pem` then reading it later in the same process invocation). Wildcard denies are snapshot-scanned per exec boundary (documented in schema/guarantees).
- Cross-platform parity for strict deny (Linux full isolation is the initial scope; other platforms are out of scope for this ADR).

## User Contract (Authoritative)

### CLI
- Operators edit policy via existing patch file mechanisms (ADR-0008/ADR-0012) and `substrate policy ...` surfaces.
- New keys (V2) are authoritative; old keys are invalid and MUST hard error.
  - Canonical keys:
    - `world_fs.enforcement` (`strict|best_effort`, full isolation only)
    - `world_fs.read.allow_list`, `world_fs.read.deny_list`
    - `world_fs.discover.allow_list`, `world_fs.discover.deny_list` (optional; default mirrors `read`)
    - `world_fs.write.allow_list`, `world_fs.write.deny_list` (required only when `mode=writable`)
- Exit codes:
  - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
  - `0`: policy set/show succeeded / execution succeeded
  - `2`: user/config error (invalid schema, invalid pattern, disallowed mode combination)
  - `4`: world enforcement failure (e.g., strict deny prerequisites not met and `require_world=true`)

### Config
- Policy schema is defined in `docs/project_management/_archived/world-fs-granular-allow-deny/SCHEMA.md`.
- Hard schema constraints (fail closed; no silent ignore):
  - Patterns MUST be project-relative; absolute paths and `..` segments are invalid.
  - `allow_list` MUST be non-empty for all configured dimensions.
  - `deny_list` defaults to empty.
  - `world_fs.enforcement` MUST be present iff at least one `deny_list` is non-empty.
  - If any `deny_list` is non-empty, `world_fs.require_world` MUST be `true`.
- Isolation constraints:
  - `world_fs.enforcement=strict` is valid only when `world_fs.isolation=full`.
  - Any `deny_list` usage is valid only when `world_fs.isolation=full`.

### Platform guarantees
- Linux:
  - Full isolation (`world_fs.isolation=full`) is the only supported mode for deny enforcement.
  - Strict mode MUST prevent the workload from undoing deny mounts (security boundary).
  - When strict prerequisites are unavailable, behavior MUST fail closed (no silent downgrade).
- macOS/Windows:
  - Out of scope for this ADR (future work: guests can add support later; until then, fail closed when strict is requested).

## Architecture Shape
- Components:
  - `crates/broker`: policy YAML schema update to V2 (no compat); validation of pattern grammar and isolation constraints.
  - `crates/agent-api-types`: introduce `PolicySnapshotV2` and update `ExecuteRequest` and WS `start_session` payloads (no compat).
  - `crates/shell`: emit `PolicySnapshotV2` for world-agent requests; REPL drift continues via snapshot hash.
  - `crates/world-agent`:
    - Convert V2 snapshot to env inputs for the mount script + helper.
    - Extend the exec helper to:
      1) apply deny mounts (masking) inside the per-command mount namespace,
      2) apply Landlock allowlists (including `discover` vs `read` split),
      3) enforce strict mode lockdown (cap drop + mount syscall blocking),
      4) exec the inner command (`sh -c/-lc $SUBSTRATE_INNER_CMD`).
  - `crates/world`: maintain the mount/exec chokepoint (`PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT`) and ensure the helper is invoked whenever V2 enforcement is required (even if Landlock allowlists are empty).
- End-to-end flow (full isolation):
  - Inputs:
    - `PolicySnapshotV2` (host-resolved effective policy)
    - mount script env vars (`SUBSTRATE_MOUNT_*`, `SUBSTRATE_INNER_*`)
  - Derived state:
    - absolute allowlists for Landlock under both in-namespace project roots (`/project` and `$SUBSTRATE_MOUNT_PROJECT_DIR`)
    - deny plans (project-relative patterns) applied by helper after mounts exist
  - Actions:
    - mount script sets up minimal root + overlay bind-mounts
    - helper applies deny mounts and strict lockdown before executing the workload
  - Outputs:
    - denied operations are deterministic:
      - discover/read denies return `EACCES` (`Permission denied`)
      - write denies return `EROFS` (`Read-only file system`)
    - strict mode blocks bypass via `mount/umount` by the workload

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` (new workstream to be added for ADR-0018)
- Dependencies:
  - Requires ADR-0014 snapshot direction (host-resolved policy snapshot is authoritative).
  - Must remain compatible with ADR-0015 overlayfs backing dir allowlist derivation for full isolation writable mode.

## Security / Safety Posture
- Fail-closed rules (full isolation):
  - If `world_fs.enforcement=strict` is requested but strict prerequisites cannot be applied, the world execution MUST fail (no best-effort fallback).
  - If the policy requires Landlock (e.g., `read.allow_list` narrower than `.`), but Landlock is unsupported, execution MUST fail closed when `require_world=true`.
- Invariants:
  - No silent ignore of invalid patterns (`..`, absolute paths).
  - Deny masks must be applied before any user code runs.
  - In strict mode, the workload must not retain mount authority in its namespace.
- Observability:
  - Execution failures must surface clear diagnostics explaining which prerequisite failed (Landlock unsupported, strict lockdown unavailable, etc.).

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - V2 policy schema parsing/validation in broker (invalid keys rejected, invalid patterns rejected).
  - V2 snapshot serialization/hashing stability in shell.
- Integration tests (Linux):
  - Full isolation: allow `.` + deny `./secrets/**` returns `EACCES` for reads and cannot be bypassed by attempted `umount` in strict mode.
  - Discover/read split: allow discover `.` but deny read for specific file(s) produces “visible but not readable” behavior (when configured).
  - Wildcard denies (`**/*.pem`) enforced for matching files present at exec start (documented limitation for within-process creation).

### Manual validation
- Manual playbook is required and authoritative: `docs/project_management/_archived/world-fs-granular-allow-deny/manual_testing_playbook.md`

### Smoke scripts
- Linux smoke script(s) MUST live under `docs/project_management/_archived/world-fs-granular-allow-deny/smoke/` (added during execution triads).

## Rollout / Backwards Compatibility
- No backwards compatibility is provided:
  - Old policy YAML keys are invalid and MUST hard error.
  - `PolicySnapshotV1` is rejected by world-agent once this ADR is implemented.
- Operators must update:
  - policy patch file schema to V2, and
  - host shell + world-agent together (version lockstep).

## Decision Summary
- Decision Register: `docs/project_management/_archived/world-fs-granular-allow-deny/decision_register.md`
  - DR-0001 through DR-0008

## Decision Detail (Authoritative) — Deny enforcement posture (`strict` vs `best_effort`)

This section is authoritative for the deny enforcement posture decision previously summarized as DR-0003.

**Problem / Context**
- Deny masking (mount-based subtraction) is only a security boundary if the workload cannot later undo it via mount-family syscalls.
- Some legitimate tools/workloads require mount operations inside the world; strictly blocking mount syscalls would break those workflows.

**Option A — Strict-only denies**
- Meaning: If any `deny_list` is non-empty, enforcement is always strict (no policy lever).
- Pros:
  - Strong security posture by default; denies are always a hard boundary under adversarial workloads.
  - Fewer operator foot-guns (cannot accidentally select an unsafe mode when relying on denies to protect secrets).
- Cons:
  - Breaks workloads that require mount operations inside the world even when the operator accepts the risk.
  - Increases rollout friction (strict prerequisites become mandatory for any deny usage).
- Cascading implications:
  - Broker/schema must reject `best_effort` entirely whenever denies exist.
  - Documentation must state that “deny implies strict” and that strict prerequisites are required.
- Risks:
  - Operators may choose to avoid deny lists entirely to preserve workflow compatibility, leading to weaker real-world security.
- Unlocks:
  - Simplifies enforcement logic (single posture).
- Quick wins:
  - Fewer configuration states to test.

**Option B — Policy lever (`strict|best_effort`)**
- Meaning: When denies are used, the operator must explicitly choose `world_fs.enforcement=strict|best_effort`.
- Note: This option intentionally enables both enforcement postures as an explicit operator lever (it is a single A/B decision selection, not an “Option A and Option B” selection).
- Pros:
  - Allows strict mode to be a true security boundary while still permitting compatibility workflows under best-effort.
  - Forces explicitness: the operator must opt into strict vs best-effort rather than silently “best effort”.
- Cons:
  - Adds a foot-gun: `best_effort` is not a security boundary under adversarial workloads.
  - Adds configuration and validation complexity (must tie `enforcement` presence to deny usage).
- Cascading implications:
  - Broker/schema must enforce: `world_fs.enforcement` MUST be present iff any `deny_list` is non-empty.
  - Strict mode must fail closed when strict prerequisites cannot be applied and `require_world=true`.
  - Best-effort mode must be clearly documented as not preventing deliberate bypass by a malicious workload.
- Risks:
  - Operators may mistakenly select `best_effort` while expecting strict security.
- Unlocks:
  - Enables incremental adoption: operators can start with best-effort for compatibility, then move to strict when ready.
- Quick wins:
  - Avoids blocking adoption for mount-dependent tooling.

**Recommendation**
- Selected: Option B — policy lever (`strict|best_effort`).
- Rationale (crisp): Strict deny must be available as a real security boundary, but deny-as-a-feature must not categorically block mount-dependent workflows; requiring an explicit `enforcement` choice plus fail-closed strict behavior provides both safety and operability.

**Follow-up tasks (planning mapping)**
- Schema/validation wiring: `WFGAD0-*` (tie `enforcement` to deny usage; reject invalid combos).
- Deny masking semantics: `WFGAD3-*` (masking + deterministic error behavior).
- Strict lockdown security boundary: `WFGAD5-*` (bypass prevention; fail closed when strict prerequisites missing).

---

## Appendix A (Authoritative) — Policy Schema Renames + Semantics (No Compatibility)

**Date (UTC):** 2026-02-03  
**Status:** Accepted (appendix; supersedes naming in this ADR body)  

This appendix locks the intent-driven policy surface for world filesystem controls and resolves
operator confusion caused by implementation-leaky names (`require_world`, `mode`, `enforcement`,
`isolation=workspace|full`).

### A.1 Goals (explicit)
- Names should communicate operator intent without reading docs.
- The schema must not force “set two keys in one command” circular edits.
- Deny semantics must remain safe by default: no deny lists without an explicit enforcement posture.
- No backwards compatibility: legacy keys are hard errors once this appendix is implemented.

### A.2 Policy patch schema (YAML) — `world_fs` (V3)

```yaml
world_fs:
  # Host path visibility / rootfs isolation:
  # - true  => host paths remain nameable (former isolation=workspace)
  # - false => host paths are not nameable (former isolation=full)
  host_visible: true|false

  # Routing behavior when the world backend is unavailable / handshake fails:
  fail_closed:
    routing: true|false

  # Deny enforcement posture (only meaningful when any deny_list is non-empty):
  # - strict        => deny rules must be a hard security boundary; fail if strict cannot be enforced
  # - prefer_strict => use strict when available; otherwise fall back without failing
  # - weak          => deny rules are applied but are not a hard boundary (workload may undo/bypass)
  deny_enforcement: strict|prefer_strict|weak

  # Directory visibility (full isolation only).
  discover:
    allow_list: [ <pattern>, ... ]
    deny_list:  [ <pattern>, ... ]

  # File read access (full isolation only).
  read:
    allow_list: [ <pattern>, ... ]
    deny_list:  [ <pattern>, ... ]

  # Project write behavior (always valid).
  write:
    enabled: true|false
    # Full isolation only:
    allow_list: [ <pattern>, ... ]
    deny_list:  [ <pattern>, ... ]
```

### A.3 Defaults (explicit; deterministic)
- `world_fs.host_visible` defaults to `true` (former `isolation=workspace`).
- `world_fs.fail_closed.routing` defaults to `false` (host fallback allowed).
- `world_fs.write.enabled` defaults to `true`.
- `deny_list` defaults to `[]` when omitted (for any dimension where deny_list is allowed).
- If `discover` is omitted (and `world_fs.host_visible=false`), it defaults to `read` (same allow/deny).
- In full isolation (`world_fs.host_visible=false`), if `read.allow_list` or `write.allow_list` is omitted, it defaults to `["."]`
  (the entire project).

### A.4 Validation rules (hard errors; zero ambiguity)

#### A.4.1 Routing invariants
- If `world_fs.write.enabled=false`, then `world_fs.fail_closed.routing` MUST be `true`.
  - Rationale: `write.enabled=false` is an end-to-end guarantee. Host fallback would re-enable writes.

#### A.4.2 Full-isolation-only keys
- If `world_fs.host_visible=true`:
  - `world_fs.read` MUST be omitted.
  - `world_fs.discover` MUST be omitted.
  - `world_fs.write.allow_list` / `world_fs.write.deny_list` MUST be omitted.
  - Any deny list usage MUST be rejected as invalid config (hard error).

#### A.4.3 Allow/deny shape
- For `read`, `discover`, and `write` (when applicable):
  - `allow_list` MUST be non-empty after defaulting.
  - A path is permitted iff it matches at least one `allow_list` entry AND matches no `deny_list` entry.
  - `deny_list` overrides `allow_list`.

#### A.4.4 Deny enforcement posture (breaking the circular-edit foot-gun)
- If any `deny_list` is non-empty (in any dimension):
  - `world_fs.deny_enforcement` MUST be present.
- If all `deny_list` values are empty (or omitted):
  - `world_fs.deny_enforcement` MAY be present.
  - If present in this state, it is a stored preference and has no behavioral effect until a deny_list becomes non-empty.

### A.5 Failure taxonomy (explicit)
- Policy/config hard errors MUST fail before execution (host exit code `2`).
- World enforcement failures (e.g., strict deny requested but strict prerequisites cannot be applied) MUST fail closed as a
  world execution failure (host exit code `4`), not by silently downgrading.
  - Exception: `deny_enforcement=prefer_strict` explicitly allows downgrade without failing.
- Policy deny decisions that are enforced by Substrate **before** executing a command (for example, broker-level command
  denies) MUST use the canonical “safety/policy violation” exit code `5` per
  `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`.
- Filesystem deny rules in this ADR are enforced in-world via OS mechanisms (Landlock + deny masking). Therefore they MUST
  manifest as deterministic errno strings (e.g., `Permission denied`, `Read-only file system`, `Operation not permitted`)
  and the workload MUST exit non-zero, but the exact numeric exit code is workload-defined (Substrate MUST NOT attempt to
  translate it to `5`).

### A.6 Output contract (effective policy display)
- `substrate policy show` MUST render `discover`, `read`, and `write` in the effective policy output when `world_fs.host_visible=false`,
  including explicit `deny_list: []` when empty, so operators can discover and edit the knobs without reading docs.
- When `discover` is defaulted from `read`, the effective output MUST still show `discover` explicitly with its effective allow/deny lists.

---

## Appendix B (Authoritative) — Routing Fail-Closed + Caging Requirement + REPL Exit Semantics (No Compatibility)

**Date (UTC):** 2026-02-05  
**Status:** Accepted (appendix; supersedes naming in the backlog item and in prior drafts)  

This appendix finalizes the operator-facing semantics for:

- routing fail-closed (previously misnamed `world_fs.require_world`), and
- caging as a policy scope boundary control (not only a UX preference), and
- REPL exit transparency + return-cwd behavior.

All rules below are normative and MUST be implemented exactly as written. There is **no backwards
compatibility**: legacy keys and env var names are hard errors once this appendix is implemented.

### B.1 Goals (explicit)
- Names communicate intent without reading docs.
- Policy scope boundaries cannot be bypassed by `cd ..` (no silent “fall back to global policy”).
- Operators can predict what happens when the world backend is unavailable, and when the world is
  explicitly disabled (`--no-world` / `SUBSTRATE_WORLD=disabled`).
- REPL exit behavior is explicit, testable, and configurable.

### B.2 Rename `world_fs.require_world` → `world_fs.fail_closed.routing`

#### B.2.1 Policy key (authoritative)
The policy knob `world_fs.require_world` MUST be deleted and replaced with:

```yaml
world_fs:
  fail_closed:
    routing: true|false
```

**Meaning (zero ambiguity):**

- `routing=false`: if Substrate cannot route an execution to a world backend (backend unavailable,
  handshake fails, world-agent unreachable), Substrate MAY fall back to host execution.
- `routing=true`: if Substrate cannot route an execution to a world backend for any reason, Substrate
  MUST NOT fall back to host execution. It MUST fail closed.

This key is about **routing fallback**, not about whether the “world feature” is enabled in config.

Important: `world_fs.host_visible` and `world_fs.fail_closed.routing` are independent.

- `world_fs.host_visible=false` configures how execution MUST behave *when routed to the world* (host paths not nameable).
- `world_fs.fail_closed.routing=false` explicitly allows host fallback if routing to the world fails.
- Therefore, if an operator sets `world_fs.host_visible=false` and `world_fs.fail_closed.routing=false`, host fallback MAY
  occur and host paths will be visible during that fallback. Substrate MUST emit a high-signal warning when this happens so
  the operator cannot miss that isolation was not achieved for that execution.

Warning contract (required content; message format is otherwise implementation-defined):

- The warning MUST be printed to stderr.
- The warning MUST contain these substrings:
  - `world routing failed; falling back to host`
  - `world_fs.host_visible=false was requested`
  - `world_fs.fail_closed.routing=false allows fallback`

#### B.2.2 Exported state env var (output-only)
The exported state env var `SUBSTRATE_WORLD_REQUIRE_WORLD` MUST be deleted and replaced with:

- `SUBSTRATE_WORLD_FAIL_CLOSED_ROUTING=1|0`

Rules:

- This env var is derived from the effective policy and is **output-only** (it MUST NOT be consumed as
  an override input; override inputs remain `SUBSTRATE_OVERRIDE_*` per ADR-0006).
- It MUST be set consistently for shell, shimmed subprocesses, and world-agent requests so telemetry,
  replay, and diagnostics observe the same effective state.

#### B.2.3 Failure taxonomy for routing fail-closed
- If `world_fs.fail_closed.routing=true` and the world is explicitly disabled by operator intent
  (e.g., `--no-world`, `SUBSTRATE_WORLD=disabled`, or an equivalent effective-config disable), this is
  a policy/config incompatibility and MUST hard error before execution (host exit code `2`).
- If `world_fs.fail_closed.routing=true` and the world is enabled but routing fails at runtime:
  - If routing fails because a required world dependency is unavailable (for example: world-agent socket unreachable,
    handshake failure, transport unavailable), Substrate MUST fail closed with host exit code `3` (“required dependency
    unavailable”).
  - If routing fails because the selected world mode/feature is not supported or prerequisites are missing (for example:
    required capabilities are unavailable and the feature is not allowed to degrade), Substrate MUST fail closed with host
    exit code `4` (“not supported / missing prerequisites”).

### B.3 Policy-level caging requirement: `world_fs.caged_required`

#### B.3.1 Policy key (authoritative)
Introduce a policy key:

```yaml
world_fs:
  caged_required: true|false
```

**Meaning (zero ambiguity):**

- `caged_required=false`: caging is a user/workflow preference controlled by config (`world.caged`).
- `caged_required=true`: caging is a mandatory safety boundary for this policy scope.

#### B.3.2 “Caging” definition (normative)
When caging is enabled, Substrate MUST prevent the interactive shell from changing the working
directory outside a single deterministic boundary (`cage_root`). Attempts to leave MUST be blocked
or bounced back to `cage_root`, and Substrate MUST print a human-readable note indicating the
boundary and the destination that was rejected.

#### B.3.3 Cage root derivation (scope boundary; not an implementation detail)
On REPL/session start, Substrate MUST record:

- `entered_cwd`: the host cwd where the REPL/session was launched, and
- the effective policy scope for `entered_cwd` (workspace-scoped policy vs global policy).

If and only if the effective policy for `entered_cwd` has `world_fs.caged_required=true`, Substrate
MUST define:

- If `entered_cwd` is inside a workspace: `cage_root = workspace_root`
- Else: `cage_root = entered_cwd`

This is a **policy scope boundary** rule. It is intentionally defined in terms of workspace scope
and launch location so users cannot escape a workspace-scoped policy by `cd ../` and silently
switching to a different policy.

#### B.3.4 Policy/config compatibility (hard error; no silent override)
If the effective policy for `entered_cwd` has `world_fs.caged_required=true`, then:

- The effective config MUST have `world.caged=true`.
- If the effective config has `world.caged=false`, Substrate MUST hard error before starting the
  REPL or executing commands (host exit code `2`).

Rationale: `caged_required=true` is a safety boundary. A silent override would be surprising and
would mask an explicit operator choice to run uncaged.

Additionally:

- Any CLI flag or env override that requests “uncaged” behavior MUST be rejected as invalid when
  `world_fs.caged_required=true` (host exit code `2`).

#### B.3.5 Compatibility with `world.anchor_mode`
When `world_fs.caged_required=true`:

- `world.anchor_mode=follow-cwd` MUST be rejected as invalid config (host exit code `2`).

Rationale: `follow-cwd` defines the “root” to move with the cwd and disables meaningful caging; it
is incompatible with a stable policy scope boundary.

### B.4 REPL exit transparency + `repl.exit_cwd`

#### B.4.1 Exit note (always-on transparency)
On `exit`/`quit` (and `Ctrl+D` if treated as exit), if `world_cwd != entered_cwd`, Substrate MUST
print a note:

- `substrate: note: returning to host cwd: <path>`

The `<path>` MUST be the resulting host cwd after applying `repl.exit_cwd` rules below (including
any required fallback behavior).

Important (host shell semantics; zero ambiguity):

- A standalone `substrate` process cannot change the parent shell’s working directory.
- Therefore, `repl.exit_cwd=entered` requires no special mechanism (the parent shell remains at
  `entered_cwd` by default).
- `repl.exit_cwd=last_world` requires a supported shell integration that can apply the chosen
  `<path>` after the REPL exits (for example, a shell function/wrapper installed by Substrate init
  scripts).
- When that integration is not active, Substrate MUST still compute `<path>` and print the note, but
  the parent shell’s cwd will remain unchanged (so the observed behavior matches `entered`).

If `repl.exit_cwd=last_world` is selected but Substrate cannot safely return the host to the last
world cwd (e.g., the path is not representable on the host, or does not exist on the host at exit),
Substrate MUST:

- fall back to returning to `entered_cwd`, and
- print an additional note explaining the fallback and the reason.

#### B.4.2 Config knob (authoritative)
Add a config key:

```yaml
repl:
  exit_cwd: entered|last_world
```

Meaning (zero ambiguity):

- `entered`: on REPL exit, set the host cwd to `entered_cwd`.
- `last_world`: on REPL exit, attempt to set the host cwd to the last observed `world_cwd`.
  - This is best-effort and MUST be safe: if the last observed `world_cwd` is not representable on
    the host (e.g., a full-isolation-only path like `/project/...`) or if the target directory does
    not exist on the host at exit, Substrate MUST fall back to `entered_cwd` and emit the additional
    fallback note required by B.4.1.
  - If the shell integration needed to apply a host cwd change is not active, this setting is
    effectively “note-only”: the host cwd remains `entered_cwd`.

Defaults:

- `repl.exit_cwd` defaults to `entered`.

### B.5 Layer ownership (hard rules)
- Policy owns safety boundaries and routing constraints:
  - `world_fs.host_visible`
  - `world_fs.fail_closed.routing`
  - `world_fs.caged_required`
  - allow/deny lists and `world_fs.deny_enforcement`
- Config owns user interaction preferences and defaults that do not weaken policy:
  - `world.anchor_mode`, `world.anchor_path` (subject to policy compatibility constraints)
  - `world.caged` (only when not required by policy)
  - `repl.exit_cwd`
