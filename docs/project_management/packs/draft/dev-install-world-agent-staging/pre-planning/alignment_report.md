## Misalignment / follow-ups (wrapper-detected)
- None detected

## Consolidated full-planning follow-ups (wrapper-compiled)
### Gates / hard decisions
- None

### Decision Register required
- DR-0001 — Decide where missing-`world-agent` preflight is implemented (Rust runner vs helper script vs installer helper), given default helper output suppression and the “fail before privileged steps” invariant. (sources: `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/impact_map.md#L200`)
- DR-0002 — Define dev meaning of `scripts/substrate/dev-install-substrate.sh --no-world` (skip provisioning only vs skip all world-related build outputs). (sources: `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/impact_map.md#L201`)
- DR-0003 — Define profile mapping for staging `world-agent` (`release` only vs match `dev-install --profile`), and reconcile any UX/defaults drift. (sources: `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/impact_map.md#L202`)
- DR-0004 — Define overwrite/idempotency rules for staged `bin/(linux/)world-agent` paths under `<repo>/target/bin/`. (sources: `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/impact_map.md#L203`)

### CI/checkpoint wiring gaps
- None

### Risks + unknowns
- None

### Other follow-ups
- docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md — reconcile internal option label inconsistency and update Related Docs paths to the canonical pre-planning locations used by this pack (`pre-planning/spec_manifest.md`, `pre-planning/impact_map.md`). (sources: `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/impact_map.md#L205`)
- docs/project_management/packs/draft/dev-install-world-agent-staging/tasks.json — reconcile `behavior_platforms_required` with Linux-only behavior change (either narrow to Linux or add deterministic “no change/unsupported” validations for macOS/Windows); add `meta.checkpoint_boundaries`; populate triad tasks + kickoff prompt paths. (sources: `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/impact_map.md#L206`)
- docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md — pin exit code for missing staged binary (taxonomy-aligned) + minimum remediation string; pin whether both staged paths are required and the `--dry-run` behavior for the missing-artifact path. (sources: `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/impact_map.md#L207`)
- Path canonicalization: `pre-planning/impact_map.md` lists `docs/project_management/packs/draft/dev-install-world-agent-staging/ci_checkpoint_plan.md` as the create path, but the pre-planning artifact lives at `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/ci_checkpoint_plan.md`; choose one canonical path in full planning and update references. (sources: `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/workstream_triage.md#L175`)
- Touch-set alignment: `pre-planning/minimal_spec_draft.md` lists `scripts/substrate/install-substrate.sh` under DIWAS1 likely touch surfaces, but `pre-planning/impact_map.md` does not currently list it; confirm whether it is actually in-scope and update the touch set or minimal spec accordingly. (sources: `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/workstream_triage.md#L176`)
- ADR hygiene (do not do in this workstream): ADR-0035 has internal option-label inconsistency and stale Related Docs links; reconcile during full planning per spec-manifest follow-ups. (sources: `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/workstream_triage.md#L177`)

