# world-sync — impact map

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/active/world-sync`
- ADR(s) / upstream contracts:
  - `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
  - `docs/project_management/adrs/implemented/ADR-0004-world-overlayfs-directory-enumeration-reliability.md`
  - `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`
  - `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
- Spec manifest:
  - `docs/project_management/packs/active/world-sync/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

### Create
- `docs/project_management/packs/active/world-sync/filesystem-semantics-spec.md` — authoritative path/diff semantics for sync
- `docs/project_management/packs/active/world-sync/platform-parity-spec.md` — authoritative per-platform support contract
- `docs/project_management/packs/active/world-sync/internal-git-spec.md` — authoritative internal git contract
- `docs/project_management/packs/active/world-sync/WS1-spec.md` .. `WS7-spec.md` — per-slice specs
- `docs/project_management/packs/active/world-sync/WS0-closeout_report.md` — per-slice closeout gate report (WS0)
- `docs/project_management/packs/active/world-sync/WS1-closeout_report.md` .. `WS7-closeout_report.md` — per-slice closeout gate reports
- `docs/project_management/packs/active/world-sync/kickoff_prompts/*` — kickoff prompts for all tasks in `tasks.json`
- `docs/project_management/packs/active/world-sync/quality_gate_report.md` — planning quality gate evidence (required before execution)
- (implementation-time; code/test surface)
  - `crates/shell/src/execution/workspace_sync_cmd.rs` (or similar) — new workspace sync/checkpoint/rollback handlers
  - `crates/shell/tests/workspace_sync.rs` — CLI contract tests (exit codes, guards, flags)
  - `crates/shell/tests/workspace_checkpoint_rollback.rs` — internal git tests
  - `crates/world/src/sync_apply.rs` (or similar) — apply pending overlay diffs to workspace (Linux/macOS)

### Edit
- `docs/project_management/next/sequencing.json` — update `world_sync` sprint sequence ids/paths to `WS*`
- `crates/shell/src/execution/cli.rs` — add `workspace sync|checkpoint|rollback` subcommands + flags
- `crates/shell/src/execution/workspace_cmd.rs` — route new `WorkspaceAction` variants; shared guardrails for workspace errors
- `crates/shell/src/execution/workspace.rs` — helper functions for internal git dir + sync state as needed
- `crates/shell/src/execution/routing/dispatch/world_ops.rs` — pending diff discovery (non-PTY and PTY) and (later) apply hooks
- `crates/world/src/overlayfs/utils.rs` and/or `crates/world/src/overlayfs/mod.rs` — expose deterministic “pending diff” computation for session upperdir
- `crates/world/src/copydiff.rs` — (if used) align pending diff discovery/apply semantics with overlayfs path model
- `docs/USAGE.md` and `docs/CONFIGURATION.md` — document new workspace sync/checkpoint/rollback surfaces and config keys
- `docs/reference/paths/layout.md` — ensure internal git path is documented as `.substrate/git/repo.git/`

### Deprecate
- None (greenfield; no compatibility surfaces are introduced by this feature).

### Delete
- None.

## Cascading implications (behavior/UX)

For each externally visible change, list:
- direct impact (what the operator experiences),
- cascading impact (what else must change to stay coherent),
- contradiction risks (what existing semantics would conflict).

### CLI / UX
- Change: new workspace-scoped commands `workspace sync|checkpoint|rollback`
  - Direct impact:
    - Operators can explicitly apply pending world-session diffs to the host workspace (or preview via `--dry-run`).
    - Operators can record/restore internal checkpoints independent of the user repo.
  - Cascading impact:
    - CLI help/usage and docs must clearly separate:
      - `world deps sync` (manager/tool parity), and
      - `workspace sync` (filesystem diff apply).
    - Smoke scripts and the manual playbook must validate stable exit codes and key output strings.
  - Contradiction risks:
    - Any lingering references to legacy `substrate sync` or legacy `.substrate-git/` paths create operator confusion; those references must not appear in the world-sync specs.

### Config / env vars / paths
- Change: `sync.*` config keys become execution-relevant for workspace sync behavior.
  - Direct impact:
    - Effective config values determine default direction/conflict policy and auto-sync behavior.
  - Cascading impact:
    - `contract.md` must be the single authoritative place where key names, defaults, and precedence are specified.
    - Path invariants must remain consistent with ADR-0008: all workspace state lives under `.substrate/`.
  - Contradiction risks:
    - Any docs claiming protected excludes include `.substrate-git/**` conflict with current code (`PROTECTED_EXCLUDES`).

### Policy / isolation / security posture
- Change: applying pending world diffs is a security-sensitive mutation operation.
  - Direct impact:
    - Sync apply MUST refuse protected-path mutations and large diffs (exit `5`).
  - Cascading impact:
    - The “pending diff” model must be auditable (summary output; trace hooks if added).
    - Auto-sync must not silently apply changes; it must log and obey configured policies.
  - Contradiction risks:
    - “Best-effort” platform behavior would create false guarantees; unsupported paths must be explicit (exit `4`).

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
  - Overlap surfaces: workspace root discovery, `.substrate/` layout, protected excludes
  - Conflict: **no** (world-sync adopts ADR-0008 invariants)
  - Resolution: enforced via DR-0003 and contract/spec ownership.
- `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`
  - Overlap surfaces: persistent PTY sessions + “session close” lifecycle events
  - Conflict: **no**, but world-sync auto-sync must hook into the same lifecycle
  - Resolution: WS3/WS4/WS5 explicitly define non-PTY vs PTY behavior and required retrieval semantics.

### Relevant Planning Packs (queued/unimplemented)
- `docs/project_management/_archived/world-fs-granular-allow-deny/` (implemented/active track)
  - Overlap surfaces: filesystem enforcement, allow/deny semantics, “what paths can be written”
  - Conflict: **potential** if sync apply semantics attempt to bypass enforced policy
  - Resolution: world-sync treats sync apply as a host mutation operation with strict protected-path refusal and explicit backend capability checks (platform-parity-spec).

## Follow-ups (explicit)

- Decision Register entries required:
  - `docs/project_management/packs/active/world-sync/decision_register.md` — DR-0001..DR-0005 are required and must remain aligned to tasks and specs.
- Spec updates required (if any):
  - `docs/project_management/packs/active/world-sync/spec_manifest.md` — keep the required spec list in lockstep with created files; do not add “extra” specs without updating ownership.
