# world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment — decision register

This register records explicit, reviewable decisions needed to interpret and implement the Appendix authoritative contracts.

Authoritative inputs:
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md`
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md`
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/PROTOCOL.md`
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/ENV.md`

## DR-AXA-0001 — Effective policy display is V3-only (no legacy keys rendered)
Decision:
- `substrate policy show` renders the effective policy using **V3 operator-facing keys** only.
- V2 policy keys (`world_fs.mode|isolation|require_world|enforcement`, `read_allowlist`, `write_allowlist`) are not rendered as operator-facing fields.

Rationale:
- Appendix contract §1.3 defines the output contract and requires explicit allow/deny rendering.
- Appendix contract §5 explicitly forbids backwards compatibility for operator-facing surfaces.

Implementation notes:
- Avoid relying on `impl Serialize for Policy` if it would reintroduce V2 shapes.
- Encode the output contract in deterministic tests (YAML + `--json`).

## DR-AXA-0002 — “No backwards compatibility” scope (operator surfaces)
Decision:
- “No backwards compatibility” is enforced for:
  - policy patch parsing and `substrate policy set` dotted updates (host exit `2` for legacy keys),
  - effective policy display (`substrate policy show`) output shape,
  - world-agent snapshot acceptance: only `schema_version=3` is accepted, and missing `policy_snapshot` is rejected for `/v1/execute` and `start_session`.

Rationale:
- Appendix contract §5 and Appendix protocol require lockstep.

## DR-AXA-0003 — PolicySnapshotV3 fields are limited to Appendix schema
Decision:
- `PolicySnapshotV3` JSON payload shape matches `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md` §2 exactly:
  - top-level keys: `schema_version`, `world_fs`
  - no additional top-level keys (e.g., no `net_allowed`, no `limits`)
  - unknown fields are rejected by world-agent.

Rationale:
- Appendix schema is explicit about unknown field rejection and canonical JSON shape.

Implementation notes:
- World-agent must not depend on snapshot-carried `net_allowed`/`limits` in the V3 path.
  - Today, network filtering is gated by `WORLD_NETFILTER_ENABLE` and is off by default; V3 keeps existing behavior by using an empty allowed-domain list in the world spec unless/until a future contract extends the V3 snapshot schema.

## DR-AXA-0004 — Trace metadata reflects snapshot schema version
Decision:
- Trace spans report snapshot usage in a version-correct way:
  - `policy_resolution_mode` uses `snapshot_v3` when a V3 snapshot is attached, otherwise `legacy_local`.
  - `policy_snapshot_schema` equals `3` for V3 snapshots.

Rationale:
- Avoid “snapshot_v1” drift in operator diagnostics and postmortems.
- Keep trace field *names* stable while correcting the values.

