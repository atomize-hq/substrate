# Policy Snapshot Deprecation Spec — Remove Legacy Local Policy Resolution in World-Agent

Related decisions (authoritative):
- `docs/project_management/_archived/world-agent-policy-snapshot/decision_register.md` (DR-0002, DR-0005, DR-0008, DR-0011)

Related ADR:
- `docs/project_management/next/ADR-0014-world-agent-policy-resolution-and-concurrency.md`

## Goal
Remove world-agent’s legacy local policy resolution (broker reads inside the agent process) after a bounded compatibility window, so world-agent enforcement becomes a deterministic function of host-provided snapshots.

## Definitions
- **Snapshot mode**: world-agent enforces `world_fs` + `net_allowed` + `limits` using `PolicySnapshotV1` provided by the host.
- **Legacy local mode**: world-agent derives enforcement inputs by resolving policy locally inside the agent process.
- **Release Rₛ**: the first Substrate release that ships end-to-end PolicySnapshotV1 support (host attaches snapshots; world-agent enforces them).

## Deprecation window (single release window)
- Legacy local mode MUST be supported only during **Release Rₛ**.
- Legacy local mode MUST be removed in the immediately following release (**Release Rₛ+1**).

## Removal gate (single gating condition)
Legacy local mode removal in **Release Rₛ+1** is gated solely on the following condition:
- The host Substrate shell MUST attach `PolicySnapshotV1` to every world-agent execution request path it uses (non-PTY and PTY) when the connected world-agent indicates snapshot support.

Snapshot support indication:
- World-agent indicates PolicySnapshotV1 ingestion support by advertising a stable capability signal (e.g., including `policy_snapshot_v1` in `GET /v1/capabilities` feature flags).

## Behavior during the deprecation window (Release Rₛ)

### World-agent request handling
- When `policy_snapshot` is present and valid:
  - world-agent MUST use snapshot mode.
  - world-agent MUST ignore legacy local policy resolution for enforcement inputs.
- When `policy_snapshot` is missing:
  - world-agent MUST use legacy local mode.
  - world-agent MUST report `policy_resolution_mode="legacy_local"` via trace/doctor observability fields (per DR-0008).
- When `policy_snapshot` is present but invalid:
  - world-agent MUST reject the request deterministically (HTTP 400 / bad request classification).

### Host behavior for version skew
- If the connected world-agent indicates PolicySnapshotV1 support:
  - the host MUST send `policy_snapshot` on world-agent requests.
- If the connected world-agent does not indicate PolicySnapshotV1 support:
  - the host MUST follow the fallback rules in `docs/project_management/_archived/world-agent-policy-snapshot/policy-snapshot-spec.md`.

## Behavior after removal (Release Rₛ+1 and later)
- When `policy_snapshot` is present and valid:
  - world-agent MUST use snapshot mode.
- When `policy_snapshot` is missing:
  - world-agent MUST reject the request deterministically (HTTP 400 / bad request classification).
- When `policy_snapshot` is present but invalid:
  - world-agent MUST reject the request deterministically (HTTP 400 / bad request classification).

## Observability requirements
- Trace output MUST record `policy_resolution_mode` for every command completion:
  - `"snapshot_v1"` when snapshot mode was used.
  - `"legacy_local"` when legacy local mode was used (Release Rₛ only).
- Doctor output MUST report:
  - snapshot ingestion support, and
  - the active policy resolution mode used by the agent for the most recent request, or an equivalent deterministic indicator.

## Compatibility rules (operator-facing)
- Old host → new agent:
  - Works only in Release Rₛ (agent uses legacy local mode when snapshot is missing).
  - Fails in Release Rₛ+1 and later (agent rejects missing snapshots).
- New host → old agent:
  - Host MUST treat “agent lacks snapshot support” as a compatibility constraint and follow the explicit fallback rules (fail closed when `world_fs.require_world=true`; otherwise host fallback is permitted).

## Acceptance criteria (spec-level)
- The deprecation spec defines exactly:
  - one release window (Release Rₛ only), and
  - one gating condition for removal (host always attaches snapshots on world-agent paths).
- The spec defines deterministic behavior for:
  - snapshot present + valid,
  - snapshot missing,
  - snapshot present + invalid,
  both during the window and after removal.
