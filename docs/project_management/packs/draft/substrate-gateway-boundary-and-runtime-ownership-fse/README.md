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

- Active seam: `SEAM-4`
- Next seam: `null`
- Future seam(s): `SEAM-1`, `SEAM-2`, `SEAM-3`

Horizon inference:

- `SEAM-1` has landed with a passed seam-exit gate and remains outside the forward planning window.
- `SEAM-2` has landed with a passed seam-exit gate and published the schema and policy handoff for downstream consumers.
- `SEAM-3` has now landed with a passed seam-exit gate and published the typed runtime/parity handoff for downstream consumers.
- `SEAM-4` is now the active seam because conformance and cross-doc lock-in can consume the published operator, schema, policy, and runtime/parity contracts directly.
- No additional safe `next` seam remains inside this pack, so the forward window now terminates at `SEAM-4`.

Policy:

- only the active seam is eligible for authoritative downstream deep planning by default
- there is no queued `next` seam because `SEAM-4` is the terminal seam in this pack
- active and next seams must eventually terminate in a dedicated final `S99` `seam-exit-gate` slice once seam-local planning begins
- seams that own undefined contract boundaries may reserve `S00` as a contract-definition boundary slice once seam-local planning begins
- `SEAM-1` has now landed with a passed seam-exit gate and left the forward planning window
- `SEAM-2` has now landed with a passed seam-exit gate and left the forward planning window
- `SEAM-3` has now landed with a passed seam-exit gate and left the forward planning window
- future seams remain landed seam briefs unless a downstream pack explicitly reopens them
- canonical contract docs live in `docs/contracts/` and must remain descriptive-only

Source-pack crosswalk:

- `pre-planning/spec_manifest.md`, `pre-planning/minimal_spec_draft.md`, and ADR-0040 operator contract language map chiefly to `SEAM-1`
- `pre-planning/spec_manifest.md`, `pre-planning/impact_map.md`, and the `SGBRO-PWS-schema_inventory` lane in `pre-planning/workstream_triage.md` map chiefly to `SEAM-2`
- the selected typed world-agent option in `pre-planning/impact_map.md`, the parity requirements in `pre-planning/ci_checkpoint_plan.md`, and the `SGBRO-PWS-world_agent_profile` lane map chiefly to `SEAM-3`
- `pre-planning/ci_checkpoint_plan.md`, the docs-validation and task-checkpoint workstreams in `pre-planning/workstream_triage.md`, and the cross-doc touch set in `pre-planning/impact_map.md` map chiefly to `SEAM-4`
