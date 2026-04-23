# gateway-backend-selection-runtime-integration - execution pack

Source: `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md` + `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/`

This pack is the execution control plane for landing ADR-0046. Its purpose is to move the integrated gateway from one hardcoded `cli:codex` lifecycle path to inventory-backed backend realization without widening the existing operator command family.

This pack now consumes the existing canonical contracts under `docs/contracts/` as upstream execution constraints. It is not another contract/governance extraction. The only remaining contract work inside this pack is the narrow `SEAM-1` alignment needed to make selection and policy inputs fully deterministic in the repo surfaces that implement them.

Execution outcome:

- `SEAM-1` narrows and locks the backend-selection and policy-evaluation inputs that the code must consume, then lands consumer alignment and proof in shell, broker, config, and tests.
- `SEAM-2` implements integrated adapter lookup, capability gating, auth handoff validation, runtime config generation, managed artifacts, and lifecycle behavior from that upstream truth.
- `SEAM-3` proves parity, validation, and rollout behavior after the runtime path exists, using the named first additional integrated backend proof target `api:openai`.

Restated scope and assumptions:

- Realize ADR-0041 backend selection inside the integrated gateway lifecycle without reopening ADR-0040 ownership, ADR-0042 identity-tuple posture, or ADR-0043 tuple-policy work.
- Keep `status`, `sync`, and `restart` as the existing operator entrypoints while moving the integrated lifecycle from one hardcoded `cli:codex` path toward inventory-backed backend realization.
- Treat the existing gateway contract docs under `docs/contracts/*` as durable upstream refs that constrain implementation.
- Keep `status --json`, tuple metadata, tuple-policy surfaces, and secret-channel redesign out of scope for this pack.
- Repo evidence now includes a bounded multi-backend integrated auth/runtime handoff, so this pack records how selection, runtime behavior, and parity proof were made concrete without publishing a second contract system.

Start here:

- `scope_brief.md`
- `seam_map.md`
- `threading.md`
- `review_surfaces.md`
- `governance/remediation-log.md`

Execution horizon:

- Active seam: none
- Next seam: none currently queued
- Future seam(s): `SEAM-1`, `SEAM-2`, `SEAM-3`

Horizon inference:

- `SEAM-2` is landed because it published `THR-02` with a bounded multi-backend runtime handoff and passed its seam-exit gate.
- `SEAM-3` has now landed with a passed seam-exit gate after attaching parity, platform, and rollout proof to the canonical runtime parity contract.
- No later seam is queued behind `SEAM-3` in the current pack, so the forward window is now empty.

Policy:

- no seam currently owns the forward planning window because the terminal conformance seam has landed
- active and next seams must eventually terminate in a dedicated final `S99` `seam-exit-gate` slice once seam-local planning begins
- seams that still need a narrow contract-alignment slice may reserve `S00` for that boundary work before implementation slices
- future seams remain deferred until their execution preconditions exist
- canonical contract artifacts live under `docs/contracts/`, but this pack treats them as upstream constraints rather than publication placeholders
- feature-local ADR-0046 docs remain supporting implementation, validation, and verification surfaces

Source-pack crosswalk:

- lineage `GBSRI-01` and `seam-planning/backend-selection-and-policy.md` map to `SEAM-1`
- lineage `GBSRI-02` and `seam-planning/runtime-realization-and-artifacts.md` map to `SEAM-2`
- lineage `GBSRI-03` and `seam-planning/parity-validation-and-rollout.md` map to `SEAM-3`

Current pack posture:

- `SEAM-1` is landed and closed out as the published selection/policy handoff.
- `SEAM-2` is landed and closed out as the published runtime realization handoff.
- `SEAM-3` is landed and closed out as the published parity, validation, and rollout handoff.
- No active seam remains in this pack unless a new extraction or downstream follow-on pack is introduced.
