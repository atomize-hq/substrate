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

- Active seam: `SEAM-2`
- Next seam: `SEAM-3`

Policy:

- only the active seam is eligible for authoritative downstream deep planning by default
- the next seam may later receive seam-local review and slices, and only provisional deeper planning
- active and next seams must eventually terminate in a dedicated final `S99` `seam-exit-gate` slice once seam-local planning begins
- seams that own undefined contracts may reserve `S00` as a contract-definition boundary slice once seam-local planning begins
- future seams remain seam briefs
- canonical contract docs live in `docs/contracts/` and must remain descriptive-only

Current promotion state:

- `SEAM-1` is closed and no longer in the forward planning window because its closeout published `THR-01` and recorded a passed seam-exit gate.
- `SEAM-2` is the active seam because the `SEAM-1` closeout published the stable selection and status-boundary handoff needed to begin seam-local protocol/schema planning.
- `SEAM-3` is now the queued next seam because it remains the direct downstream consumer of `SEAM-2` and still depends on `THR-02` publication before it can become active.
