# world-fs-granular-allow-deny — impact map

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/_archived/world-fs-granular-allow-deny`
- ADR(s):
  - `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
- Spec manifest:
  - `docs/project_management/_archived/world-fs-granular-allow-deny/spec_manifest.md`

## Touch set (explicit)

### Create
- `docs/project_management/_archived/world-fs-granular-allow-deny/spec_manifest.md` — v4 planning standard required artifact (spec ownership map).
- `docs/project_management/_archived/world-fs-granular-allow-deny/impact_map.md` — v4 planning standard required artifact (replaces integration_map.md).
- `docs/project_management/_archived/world-fs-granular-allow-deny/session_log.md` — v4 planning standard required artifact.
- `docs/project_management/_archived/world-fs-granular-allow-deny/ci_checkpoint_plan.md` — bounded CI checkpoint plan for cross-platform parity.
- `docs/project_management/_archived/world-fs-granular-allow-deny/WFGAD0-spec.md` — required slice spec artifact.
- `docs/project_management/_archived/world-fs-granular-allow-deny/WFGAD1-spec.md` — required slice spec artifact.
- `docs/project_management/_archived/world-fs-granular-allow-deny/WFGAD2-spec.md` — required slice spec artifact.
- `docs/project_management/_archived/world-fs-granular-allow-deny/WFGAD3-spec.md` — required slice spec artifact.
- `docs/project_management/_archived/world-fs-granular-allow-deny/WFGAD4-spec.md` — required slice spec artifact.
- `docs/project_management/_archived/world-fs-granular-allow-deny/WFGAD5-spec.md` — required slice spec artifact.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD0-code.md` — kickoff prompt for WFGAD0 code task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD0-test.md` — kickoff prompt for WFGAD0 test task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD0-integ.md` — kickoff prompt for WFGAD0 integration task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD1-code.md` — kickoff prompt for WFGAD1 code task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD1-test.md` — kickoff prompt for WFGAD1 test task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD1-integ.md` — kickoff prompt for WFGAD1 integration task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD2-code.md` — kickoff prompt for WFGAD2 code task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD2-test.md` — kickoff prompt for WFGAD2 test task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD2-integ.md` — kickoff prompt for WFGAD2 integration task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD3-code.md` — kickoff prompt for WFGAD3 code task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD3-test.md` — kickoff prompt for WFGAD3 test task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD3-integ.md` — kickoff prompt for WFGAD3 integration task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD4-code.md` — kickoff prompt for WFGAD4 code task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD4-test.md` — kickoff prompt for WFGAD4 test task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD4-integ.md` — kickoff prompt for WFGAD4 integration task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD5-code.md` — kickoff prompt for WFGAD5 code task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD5-test.md` — kickoff prompt for WFGAD5 test task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD5-integ.md` — kickoff prompt for WFGAD5 integration task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD1-integ-core.md` — kickoff prompt for WFGAD1 integration core task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD1-integ-linux.md` — kickoff prompt for WFGAD1 linux platform-fix task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD1-integ-macos.md` — kickoff prompt for WFGAD1 macos platform-fix task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD1-integ-windows.md` — kickoff prompt for WFGAD1 windows platform-fix task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD3-integ-core.md` — kickoff prompt for WFGAD3 integration core task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD3-integ-linux.md` — kickoff prompt for WFGAD3 linux platform-fix task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD3-integ-macos.md` — kickoff prompt for WFGAD3 macos platform-fix task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD3-integ-windows.md` — kickoff prompt for WFGAD3 windows platform-fix task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD5-integ-core.md` — kickoff prompt for WFGAD5 integration core task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD5-integ-linux.md` — kickoff prompt for WFGAD5 linux platform-fix task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD5-integ-macos.md` — kickoff prompt for WFGAD5 macos platform-fix task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/WFGAD5-integ-windows.md` — kickoff prompt for WFGAD5 windows platform-fix task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/CP1-ci-checkpoint.md` — kickoff prompt for CP1 CI checkpoint task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/CP2-ci-checkpoint.md` — kickoff prompt for CP2 CI checkpoint task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/CP3-ci-checkpoint.md` — kickoff prompt for CP3 CI checkpoint task.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/FZ-feature-cleanup.md` — required for v3+ automation packs.

### Edit
- `docs/project_management/_archived/world-fs-granular-allow-deny/tasks.json` — upgrade to schema v4 with triad automation enabled.
- `docs/project_management/_archived/world-fs-granular-allow-deny/plan.md` — update to reference spec_manifest and impact_map and document guardrails.
- `docs/project_management/_archived/world-fs-granular-allow-deny/kickoff_prompts/` — replace legacy C0 kickoff prompts with WFGAD0-WFGAD5 kickoff prompts with v4 automation workflow and required sentinel.
- `docs/project_management/_archived/world-fs-granular-allow-deny/requirements_traceability.md` — replace legacy integration_map reference with impact_map.
- `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md` — update Related Docs list to include spec_manifest and impact_map.
- `docs/project_management/next/sequencing.json` — add sprint entry for this Planning Pack directory.

### Deprecate
- `docs/project_management/_archived/world-fs-granular-allow-deny/integration_map.md` — legacy doc retained for existing links; authoritative replacement is impact_map.md.

### Edit (production implementation; derived from ADR-0018)
- `crates/broker/` — policy schema v2 (breaking) validation and pattern grammar enforcement.
- `crates/agent-api-types/src/lib.rs` — introduce PolicySnapshotV2 and request payload updates.
- `crates/shell/src/execution/policy_snapshot.rs` — emit PolicySnapshotV2.
- `crates/world-agent/src/service.rs` — accept PolicySnapshotV2 only; build helper env; fail closed on invalid snapshot.
- `crates/world-agent/src/pty.rs` — accept PolicySnapshotV2 only for session start payloads.
- `crates/world-agent/src/internal_exec.rs` — apply deny masks and strict lockdown before workload exec.
- `crates/world/src/exec.rs` — maintain full isolation chokepoint invocation and helper routing.
- `crates/world/src/landlock.rs` — discover/read split behavior support as required by the spec.

## Cascading implications (behavior/UX)

### CLI / UX
- Change: Operators can express deny-overrides-allow for reads/writes (and optionally discover) in full isolation.
  - Direct impact: Policies can represent allow-all-except patterns deterministically.
  - Cascading impact: Validation must fail closed on invalid patterns and invalid isolation mode usage.
  - Contradiction risks: Existing allowlist-only behavior and any silent-ignore of invalid patterns conflicts with the v2 contract.

### Config / env vars / paths
- Change: Helper-side deny masking and strict lockdown are controlled via env inputs derived from PolicySnapshotV2.
  - Direct impact: Helper must execute whenever helper-side enforcement is required.
  - Cascading impact: Env schema parsing and validation failures must fail closed with deterministic exit codes.
  - Contradiction risks: Any best-effort fallback behavior conflicts with strict deny security boundary requirements.

### Policy / isolation / security posture
- Change: Strict deny makes denies a security boundary in full isolation.
  - Direct impact: Workload cannot undo deny mounts via mount/umount in strict mode.
  - Cascading impact: World execution must fail closed when strict prerequisites are unavailable.
  - Contradiction risks: Non-strict or downgraded execution when strict is requested violates the contract.

## Cross-queue scan (ADRs + Planning Packs)

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/implemented/ADR-0014-world-agent-policy-resolution-and-concurrency.md`
  - Overlap surfaces: policy snapshot generation and host↔world-agent snapshot ownership.
  - Conflict: no
  - Resolution (explicit): ADR-0014 establishes host-resolved snapshot authority; ADR-0018 extends the snapshot schema to v2.
- ADR: `docs/project_management/adrs/implemented/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`
  - Overlap surfaces: Linux full isolation + Landlock behavior interactions with overlayfs roots.
  - Conflict: no
  - Resolution (explicit): ADR-0018 preserves full isolation chokepoints and extends Landlock allowlist semantics; overlayfs compatibility remains a required constraint.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/_archived/world-fs-granular-allow-deny`
  - Overlap surfaces: N/A (this Planning Pack)
  - Conflict: no
  - Resolution (explicit): N/A

## Follow-ups (explicit)
- Decision Register entries required:
  - NONE
- Spec updates required:
  - `docs/project_management/_archived/world-fs-granular-allow-deny/spec_manifest.md` — keep required-doc list consistent with the Planning Pack artifact set.
