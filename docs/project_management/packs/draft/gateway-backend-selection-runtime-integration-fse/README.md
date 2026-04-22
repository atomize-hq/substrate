# gateway-backend-selection-runtime-integration - seam extraction

Source: `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md` + `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/`

This pack captures a governance-ready v2.5 seam extraction for ADR-0046 using the existing pre-planning pack as basis. It intentionally stays one level above seam-local decomposition: no seam-local planning docs, slice files, candidate subslices, or execution units are created here.

Restated scope and assumptions:

- Realize ADR-0041 backend selection inside the integrated gateway lifecycle without reopening ADR-0040 ownership, ADR-0042 identity-tuple posture, or ADR-0043 tuple-policy work.
- Keep `status`, `sync`, and `restart` as the existing operator entrypoints while moving the integrated lifecycle from one hardcoded `cli:codex` path toward inventory-backed backend realization.
- Treat ADR-0040, ADR-0041, and the current `docs/contracts/*` gateway docs as external authorities and evidence, not editable ownership surfaces or seam-owned publication targets for this extraction.
- Keep `status --json`, tuple metadata, and tuple-policy surfaces out of scope for this pack.
- Repo evidence still shows only a Codex-shaped integrated auth/runtime path today, so unresolved expansion points remain explicit remediations rather than inferred decisions.
- `GBSRI-*` ids and `seam-planning/*.md` paths are lineage only in this run; they inform the seam map but are not required outputs.

Start here:

- `scope_brief.md`
- `seam_map.md`
- `threading.md`
- `review_surfaces.md`
- `governance/remediation-log.md`

Execution horizon:

- Active seam: `SEAM-1`
- Next seam: `SEAM-2`
- Future seam(s): `SEAM-3`

Horizon inference:

- `SEAM-1` is active because the repo still lacks a fixed contract for backend inventory roots, filename rules, and auth-source precedence beyond the current Codex-only code path.
- `SEAM-2` is next because runtime realization work is blocked on `SEAM-1` publishing the selection/policy truth, and because the remaining runtime questions are still contract-authority-sensitive enough to stay provisional.
- `SEAM-3` stays future because parity, validation, and rollout proof depend on both published upstream contracts and a named first additional integrated backend baseline that does not yet exist in repo truth.

Policy:

- only the active seam is eligible for authoritative downstream deep planning by default
- the next seam may later receive seam-local review and only provisional deeper planning
- active and next seams must eventually terminate in a dedicated final `S99` `seam-exit-gate` slice once seam-local planning begins
- seams that own undefined contracts may reserve `S00` as a contract-definition boundary slice once seam-local planning begins
- future seams remain seam briefs
- seam-owned ADR-0046 deltas publish into the feature-local docs under `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/`
- external `docs/contracts/*` docs remain descriptive authoritative dependencies and must remain descriptive-only

Source-pack crosswalk:

- lineage `GBSRI-01` and `seam-planning/backend-selection-and-policy.md` map to `SEAM-1`, but this extractor emits only the seam brief and governance scaffolds
- lineage `GBSRI-02` and `seam-planning/runtime-realization-and-artifacts.md` map to `SEAM-2`, but this extractor emits only the seam brief and governance scaffolds
- lineage `GBSRI-03` and `seam-planning/parity-validation-and-rollout.md` map to `SEAM-3`, but this extractor emits only the seam brief and governance scaffolds
