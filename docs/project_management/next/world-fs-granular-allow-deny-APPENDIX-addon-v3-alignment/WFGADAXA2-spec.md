# WFGADAXA2-spec — Downstream surfaces + docs alignment (post-V3 story)

## Scope
- Remove operator-facing drift that still presents V2 keys as canonical after Appendix V3:
  - doctor/health JSON surfaces
  - trace metadata values and trace docs
  - canonical operator docs (`docs/CONFIGURATION.md`, `docs/WORLD.md`, env contract docs)

## Authoritative requirements
- Appendix operator keys and “no backwards compatibility” posture:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md` §1 and §5
- Env var contract:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/ENV.md` (delete `SUBSTRATE_WORLD_REQUIRE_WORLD`; exported state is `SUBSTRATE_WORLD_FAIL_CLOSED_ROUTING`)
- Trace metadata decision (this add-on):
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/decision_register.md` (DR-AXA-0004)

## Behavior
Required end-state:
1. Doctor/health JSON surfaces do not present V2 policy keys as operator-facing configuration:
   - Remove (or clearly reframe as derived/internal-only) fields like `world_fs_mode`, `world_fs_isolation`, `world_fs_require_world` from:
     - `substrate host doctor --json`
     - `substrate world doctor --json`
     - shim doctor/health JSON (as applicable)
   - Replace with V3 operator-facing keys where these surfaces need to expose policy state:
     - `world_fs.host_visible`
     - `world_fs.fail_closed.routing`
     - `world_fs.write.enabled`
     - `world_fs.deny_enforcement` (when relevant)
     - `world_fs.caged_required`
2. Trace metadata values are version-correct:
   - When a snapshot is attached, `policy_resolution_mode` reflects V3 (`snapshot_v3`) and `policy_snapshot_schema=3`.
   - Update `docs/TRACE.md` to match.
3. Canonical docs are updated so operators are not taught V2 keys:
   - `docs/CONFIGURATION.md` (replace `world_fs.mode|require_world` references with V3 story)
   - `docs/WORLD.md` (replace `world_fs.require_world` references with `world_fs.fail_closed.routing`)
   - `docs/reference/env/contract.md` and `docs/internals/env/inventory.md` (remove `SUBSTRATE_WORLD_REQUIRE_WORLD`; document `SUBSTRATE_WORLD_FAIL_CLOSED_ROUTING` if missing)

## Acceptance criteria
- Tests:
  - Update/add tests that prevent regression back to V2 operator-facing fields for doctor/health and trace metadata.
- Commands (integration gate):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p substrate-shell -p substrate-trace --tests -- --nocapture`
  - `make integ-checks`
- Docs checks:
  - `rg -n "world_fs\\.require_world|world_fs\\.mode|SUBSTRATE_WORLD_REQUIRE_WORLD" docs/CONFIGURATION.md docs/WORLD.md docs/reference/env/contract.md docs/internals/env/inventory.md` returns no operator-facing references after the update (allowed exceptions: archived docs under `docs/project_management/_archived/`).

## Out of scope
- Changing the underlying internal derived fields used for legacy internal plumbing (if any) beyond what is required to remove operator-facing drift.

