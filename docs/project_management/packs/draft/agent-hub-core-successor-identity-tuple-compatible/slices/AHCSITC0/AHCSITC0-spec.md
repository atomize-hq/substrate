# AHCSITC0-spec — operator contract and identity presentation lock

## Behavior delta (single)
- Existing: `contract.md` and the pre-planning slice skeleton define the successor command family and tuple-compatible operator surface, but the pack does not yet have an execution-ready slice that binds those contract rules to one implementation boundary across `crates/shell`, `crates/common`, and the operator docs.
- New: `AHCSITC0` becomes the authoritative contract slice for `substrate agent list`, `substrate agent status`, and `substrate agent doctor`, fixing the operator-visible identity presentation rules, the derived `backend_id` rule, the pure-agent omission rules, and the file-owner boundary for the first implementation unit.
- Why: later protocol, policy, telemetry, and parity slices depend on one frozen operator contract before they can wire lifecycle, fail-closed routing, or cross-platform validation.

## Scope
- Lock the canonical successor namespace on `substrate agent list`, `substrate agent status`, and `substrate agent doctor`, while preserving `substrate agents validate` as the inventory-validation compatibility leaf.
- Lock the operator-visible render and omission rules for `backend_id`, `execution.scope`, `role`, capability summary, `world_id`, `world_generation`, `provider`, and `auth_authority` on list, status, and doctor surfaces.
- Lock the rule that pure-agent rows and nested gateway-backed rows stay separate operator-visible records instead of one overloaded row.
- Lock the first implementation boundary on `crates/shell/src/execution/cli.rs`, `crates/shell/src/execution/agents_cmd.rs`, `crates/shell/src/execution/agent_inventory.rs`, `crates/common`, and `docs/USAGE.md`.
- Keep session-handle lifecycle, ordered deny evaluation, trace field placement, and parity proof owned by `AHCSITC1`, `AHCSITC2`, and `AHCSITC3`.

## Inputs (authoritative)
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/workstream_triage.md`

## Behavior (authoritative)

### Command-surface lock
- `AHCSITC0` owns the operator-facing contract projection for `substrate agent list`, `substrate agent status`, and `substrate agent doctor`.
- `substrate agents validate` remains inventory-only and does not become an alias for list, status, or doctor.
- `backend_id` renders only as the derived adapter identifier `<kind>:<agent_id>`.
- Pure-agent list and status rows omit `provider` and `auth_authority`.
- Nested gateway-backed rows remain a separate correlated status surface and carry `provider` plus `auth_authority`.
- `world_id` and `world_generation` render only on world-scoped pure-agent session rows.

### First implementation boundary
- `crates/shell` owns command parsing, human-readable rendering, JSON projection, and doctor output for the successor command family.
- `crates/common` owns shared identity projection helpers reused by shell and later telemetry work.
- `docs/USAGE.md` mirrors the canonical namespace and the operator-facing omission rules fixed by this slice.
- `AHCSITC0` does not move session protocol grammar into a new crate and does not create a feature-local runtime package.

## Acceptance criteria
- AC-AHCSITC0-01: `AHCSITC0` fixes `substrate agent list`, `substrate agent status`, and `substrate agent doctor` as the canonical successor namespace while preserving `substrate agents validate` as the inventory-validation compatibility leaf only.
- AC-AHCSITC0-02: Operator-facing list and status surfaces derive `backend_id` only as `<kind>:<agent_id>` and keep role assignment separate from that identifier.
- AC-AHCSITC0-03: Pure-agent list rows omit `provider`, `auth_authority`, `world_id`, and `world_generation`, while pure-agent status rows publish `world_id` and `world_generation` only for world-scoped sessions.
- AC-AHCSITC0-04: Nested gateway-backed records remain separate from pure-agent rows and are the only operator-visible rows that publish `provider` and `auth_authority`.
- AC-AHCSITC0-05: The implementation boundary for this slice stays on `crates/shell`, `crates/common`, and `docs/USAGE.md` and does not introduce a new `crates/agent-hub` package.
- AC-AHCSITC0-06: Doctor output for this slice uses the contract-owned field names and omission rules and leaves ordered deny evaluation to the later policy slice.

## Out of scope
- Capability-descriptor schema, session-handle lifecycle, and world-restart replacement rules.
- Ordered deny evaluation, nested gateway policy reuse, and trace publication rules.
- Linux, macOS, and Windows parity proof and compatibility-closeout evidence.
