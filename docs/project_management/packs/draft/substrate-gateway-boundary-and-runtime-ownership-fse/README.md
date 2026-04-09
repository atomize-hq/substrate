# substrate-gateway-boundary-and-runtime-ownership - seam extraction

Source: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/`

This pack re-expresses the ADR-0040 pre-planning authority set as a governance-ready seam pack for `feature-seam-extractor-v2-5`.
It preserves the ownership split, operator contract, policy/trust boundary, typed runtime direction, and cross-platform review expectations without prematurely creating seam-local slices.

Restated scope and assumptions:

- Clarify the durable ownership split between Substrate and `substrate-gateway` for integrated gateway operation.
- Keep Substrate as the owner of policy evaluation, world placement, lifecycle control, host-to-world secret delivery, operator UX, and canonical tracing.
- Keep `substrate-gateway` as the owner of the in-world front door, provider/planner/executor internals, and normalized event generation.
- Reuse ADR-0027 config and policy keys; this feature does not introduce a new config family or a second policy file family.
- Keep `substrate world gateway status --json` as the authoritative machine-readable wiring surface, with stable non-secret env outputs `SUBSTRATE_LLM_OPENAI_BASE_URL` and `SUBSTRATE_LLM_ANTHROPIC_BASE_URL`.
- Treat the accepted five-slice planning spine in `pre-planning/workstream_triage.md` as the best available critical-path signal, but lift it into four seam briefs instead of reproducing slice-level planning.

Start here:

- `scope_brief.md`
- `seam_map.md`
- `threading.md`
- `review_surfaces.md`
- `governance/remediation-log.md`

Execution horizon:

- Active seam: `SEAM-1`
- Next seam: `SEAM-2`

Horizon inference:

- `SEAM-1` is inferred as the active seam because every downstream surface depends on one authoritative operator boundary for command spelling, absent-state behavior, exit taxonomy, stable wiring semantics, and the Substrate versus `substrate-gateway` ownership split.
- `SEAM-2` is inferred as the next seam because the status schema and policy-evaluation surface can be bounded immediately after the operator contract is locked, and the pre-planning pack already narrows that work to one schema/policy inventory seam.
- `SEAM-3` and `SEAM-4` stay future seams because typed world-agent/runtime parity and cross-doc validation should consume published upstream contracts rather than force speculative deeper planning now.

Policy:

- only the active seam is eligible for authoritative downstream sub-slices by default
- the next seam may later receive seam-local review + slices, and only provisional deeper planning
- the next seam is likely to need only spike-grade provisional work around exact field tables and decision tables rather than another broad seam split
- active and next seams must eventually terminate in a dedicated final `S99` `seam-exit-gate` slice once seam-local planning begins
- seams that own undefined contract boundaries may reserve `S00` as a contract-definition boundary slice once seam-local planning begins
- future seams remain seam briefs
- canonical contract docs live in `docs/contracts/` and must remain descriptive-only

Source-pack crosswalk:

- `pre-planning/spec_manifest.md`, `pre-planning/minimal_spec_draft.md`, and ADR-0040 operator contract language map chiefly to `SEAM-1`
- `pre-planning/spec_manifest.md`, `pre-planning/impact_map.md`, and the `SGBRO-PWS-schema_inventory` lane in `pre-planning/workstream_triage.md` map chiefly to `SEAM-2`
- the selected typed world-agent option in `pre-planning/impact_map.md`, the parity requirements in `pre-planning/ci_checkpoint_plan.md`, and the `SGBRO-PWS-world_agent_profile` lane map chiefly to `SEAM-3`
- `pre-planning/ci_checkpoint_plan.md`, the docs-validation and task-checkpoint workstreams in `pre-planning/workstream_triage.md`, and the cross-doc touch set in `pre-planning/impact_map.md` map chiefly to `SEAM-4`
