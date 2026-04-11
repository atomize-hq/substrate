# Substrate gateway backend adapter contract - seam extraction

Source:
- `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/`

This pack captures seam briefs, authoritative threading, pack-level review surfaces, seam-exit intent, and governance scaffolds for ADR-0041. It stays intentionally one level above seam-local decomposition.

- Start here: `scope_brief.md`
- Seam overview: `seam_map.md`
- Threading: `threading.md`
- Pack review surfaces: `review_surfaces.md`
- Governance: `governance/remediation-log.md`

Execution horizon:

- Active seam: `SEAM-1`
- Next seam: `SEAM-2`

Policy:

- only the active seam is eligible for authoritative downstream deep planning by default
- the next seam may later receive seam-local review and slices, and only provisional deeper planning
- active and next seams must eventually terminate in a dedicated final `S99` `seam-exit-gate` slice once seam-local planning begins
- seams that own undefined contracts may reserve `S00` as a contract-definition boundary slice once seam-local planning begins
- future seams remain seam briefs
- canonical contract docs live in `docs/contracts/` and must remain descriptive-only

Assumptions carried by this extraction:

- `SEAM-1` is the critical-path active seam because backend-id semantics, allowlist evaluation, failure taxonomy, and the published adapter-visible status boundary must be concrete before protocol or parity work can safely narrow.
- `SEAM-2` is the queued next seam because it consumes `SEAM-1` contract truth and is likely to need only provisional deeper planning until the status subset and owner lines are fixed.
- `SEAM-3` stays future because parity, compatibility, and validation proof are downstream conformance work that depends on the first two seams landing their contract truth first.
