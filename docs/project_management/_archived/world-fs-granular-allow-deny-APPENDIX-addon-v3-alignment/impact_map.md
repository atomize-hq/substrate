# world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment — impact map

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment`
- ADR(s):
  - `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
- Spec manifest:
  - `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

### Create
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA1-spec.md` — snapshot V3 migration slice spec
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA2-spec.md` — downstream surface + docs alignment slice spec
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/smoke/macos-smoke.sh` — parity smoke wrapper (macOS)
- Windows is not supported for this add-on pack; no Windows smoke script.
- (execution gates) `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA1-closeout_report.md`
- (execution gates) `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA2-closeout_report.md`

### Edit
- `crates/broker/src/policy.rs` — effective policy serialization is V2-shaped; switch to V3 operator-facing output for Appendix A.6 compliance
- `crates/shell/src/execution/policy_cmd.rs` — `print_policy` must emit V3-compliant output and stop injecting legacy helper fields
- `crates/shell/tests/*` — add/update tests that lock the `substrate policy show` output contract (Appendix A.6)
- `crates/agent-api-types/src/lib.rs` — introduce `PolicySnapshotV3` request model and validation (`schema_version=3` only)
- `crates/shell/src/execution/policy_snapshot.rs` — emit `PolicySnapshotV3` and hash canonical JSON per Appendix schema
- `crates/world-agent/src/service.rs` + `crates/world-agent/src/pty.rs` — require and validate `PolicySnapshotV3`; reject legacy versions; update capabilities feature list
- `crates/world-agent/tests/*policy_snapshot*` — update rejection tests to enforce V3-only and explicitly reject V2
- `crates/shell/src/execution/platform/*.rs` — doctor JSON surfaces currently expose V2-derived keys; align with V3 operator keys to avoid operator-facing drift
- `crates/trace/src/span.rs` + `docs/TRACE.md` — trace metadata currently claims `snapshot_v1`; align to V3 semantics (decision DR-AXA-0004)
- Docs that still present V2 keys as operator-facing:
  - `docs/CONFIGURATION.md`
  - `docs/WORLD.md`
  - `docs/reference/env/contract.md`
  - `docs/internals/env/inventory.md`

### Deprecate
- Any remaining V2 effective-policy output fields in operator-facing surfaces — replaced by V3 display structs/serializers

### Delete
- `SUBSTRATE_WORLD_REQUIRE_WORLD` documentation entries — safe because Appendix ENV contract requires deletion and implementation already removes it at runtime

## Cascading implications (behavior/UX)

For each externally visible change, list:
- direct impact (what the operator experiences),
- cascading impact (what else must change to stay coherent),
- contradiction risks (what existing semantics would conflict).

### CLI / UX
- Change: `substrate policy show` output becomes V3-shaped (Appendix A.6).
  - Direct impact: operators see `world_fs.host_visible`, `fail_closed.routing`, `deny_enforcement`, `caged_required`, `write.enabled`, plus `discover/read/write` allow/deny lists when `host_visible=false`.
  - Cascading impact: tests and docs must stop expecting/teaching V2 keys.
  - Contradiction risks: doctor/health JSON surfaces and docs that still show V2 keys will confuse operators unless updated.

### Config / env vars / paths
- Change: snapshot protocol becomes V3-only (`schema_version=3` only) and is version-locked.
  - Direct impact: old world-agents reject new shells and vice versa; rollout must remain lockstep (Appendix PROTOCOL §3).
  - Cascading impact: agent API models, shell routing, world-agent validation, and trace metadata must align on V3.
  - Contradiction risks: any lingering acceptance of V2 snapshots or legacy policy keys violates Appendix “no backwards compatibility”.

### Policy / isolation / security posture
- Change: strict rejection of legacy snapshot schema versions and legacy policy keys.
  - Direct impact: misconfigurations fail fast (exit `2` on host; HTTP 400 / fatal WS errors in world-agent).
  - Cascading impact: error messages must remain actionable and deterministic (Appendix contract §2; preserve exit code taxonomy).
  - Contradiction risks: silently accepting legacy fields in any operator-facing path undermines the “no backwards compatibility” posture.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR-0020 (Draft): `docs/project_management/next/ADR-0020-profiles-config-policy-snapshots.md`
  - Overlap surfaces: “full policy snapshot” concepts and `substrate policy show --explain` behavior.
  - Conflict: potential naming ambiguity between Appendix “PolicySnapshotV3” (world-agent protocol snapshot) and ADR-0020 “profile policy snapshots” (complete objects).
  - Resolution: keep this add-on scoped to Appendix A+B world-agent snapshot protocol + effective policy display; do not introduce profile snapshot semantics here.

### Relevant Planning Packs (queued/unimplemented)
- Base pack: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/`
  - Overlap surfaces: Appendix A+B operator keys, schema/protocol, env contract.
  - Conflict: base pack is marked completed but implementation drift remains.
  - Resolution: this add-on pack owns drift closure + regression tests. Only change Appendix authoritative docs if a real ambiguity is discovered and resolved explicitly (decision register + doc update).

## Follow-ups (explicit)

- Decision Register entries required:
  - <DR id/title> — <what it decides>
- Spec updates required (if any):
  - `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/spec_manifest.md` — <change>
  - <spec doc> — <change>
