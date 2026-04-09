## Misalignment / follow-ups (wrapper-detected)
- None detected

## Consolidated full-planning follow-ups (wrapper-compiled)
### Gates / hard decisions
- None

### Decision Register required
- None detected

### CI/checkpoint wiring gaps
- None

### Risks + unknowns
- None

### Other follow-ups
- docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/spec_manifest.md — add an explicit internal-surface note for `crates/agent-api-types/src/lib.rs` and `crates/agent-api-client/src/lib.rs` so the dedicated world-agent transport choice is reflected in the authoritative ownership map. (sources: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md#L263`)
- docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md — lock the exact command spelling, the exit `2|3|4|5` split, and the rule that `status --json` is the operator wiring authority. (sources: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md#L264`)
- docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-schema-spec.md — lock the `client_wiring.*` field family and the additive ownership boundary against ADR-0042 tuple metadata. (sources: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md#L265`)
- docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md — state that provisioning is outside this pack and list the evidence required before any platform script or backend change lands. (sources: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md#L266`)
- docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md — replace stale `packs/active/llm_and_agent_config_policy_surface/*` links with `packs/implemented/...` when cross-ADR alignment work opens. (sources: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md#L267`)
- docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md — replace stale `packs/active/llm_and_agent_config_policy_surface/*` links with `packs/implemented/...` when cross-ADR alignment work opens. (sources: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md#L268`)
- docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md — replace stale `packs/active/llm_and_agent_config_policy_surface/*` links with `packs/implemented/...` when cross-ADR alignment work opens. (sources: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md#L269`)
- docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md — normalize self-links away from `packs/active/...` so downstream ADRs stop copying stale pack paths. (sources: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md#L270`)
- docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/spec_manifest.md — normalize self-links away from `packs/active/...` so downstream ADRs stop copying stale pack paths. (sources: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md#L271`)
- Update `pre-planning/spec_manifest.md` during full planning so the canonical slice-id section and slice-spec set match the accepted 5-slice order. (sources: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/workstream_triage.md#L228`)
- Update `pre-planning/ci_checkpoint_plan.md` during full planning so the machine-readable slice list and checkpoint boundary point at `SGBRO4`. (sources: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/workstream_triage.md#L229`)
- Fix the ADR-0040 lift vector field `risk.unknowns_high` so `make pm-lift-intake` returns valid intake evidence on the next pass. (sources: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/workstream_triage.md#L230`)
- Map `docs/CONFIGURATION.md`, `docs/USAGE.md`, `docs/WORLD.md`, and `docs/TRACE.md` to `SGBRO3` and `SGBRO4` in full planning so the repo-level docs move with the same acceptance criteria. (sources: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/workstream_triage.md#L231`)

