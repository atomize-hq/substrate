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

- Active seam: none
- Next seam: none
- Future seam(s): `SEAM-1`, `SEAM-2`, `SEAM-3`

Policy:

- no active seam remains in the forward planning window because all three seams are now closed
- no next seam remains in the forward planning window; any new next seam requires an explicit horizon decision
- `SEAM-3` has closed with a passed seam-exit gate and left the forward planning window
- downstream planning now binds to closeout-backed truth from `SEAM-1`, `SEAM-2`, and `SEAM-3`
- canonical contract docs live in `docs/contracts/` and must remain descriptive-only

Current promotion state:

- `SEAM-1` is closed and no longer in the forward planning window because its closeout published `THR-01` and recorded a passed seam-exit gate.
- `SEAM-2` is closed and no longer in the forward planning window because its closeout published `THR-02` and recorded a passed seam-exit gate.
- `SEAM-3` is closed and no longer in the forward planning window because its closeout recorded a passed seam-exit gate and revalidated `THR-01` and `THR-02`.
- No active seam remains in this pack after the `SEAM-3` closeout.
