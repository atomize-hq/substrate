---
seam_id: SEAM-2
status: closed
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-2-adapter-protocol-and-schema/slice-99-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - ./seam-1-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - backend-id selection semantics change after landing
    - protocol lifecycle or capability subset changes after landing
    - schema field inventory changes after landing
    - ADR-0017 or ADR-0028 handoff wording changes after landing
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-2 Adapter protocol and schema

This closeout records the realized `SEAM-2` handoff and publishes `THR-02` for downstream `SEAM-3` consumption.

## Seam-exit gate record

- **Source artifact**: `../threaded-seams/seam-2-adapter-protocol-and-schema/slice-99-seam-exit-gate.md`
- **Landed evidence**:
  - `docs/contracts/gateway/backend-adapter-protocol.md`
  - `docs/contracts/gateway/backend-adapter-schema.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-protocol-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-schema-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threaded-seams/seam-2-adapter-protocol-and-schema/seam.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threaded-seams/seam-2-adapter-protocol-and-schema/review.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threading.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threaded-seams/seam-2-adapter-protocol-and-schema/slice-00-c-03-c-04-contract-definition.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threaded-seams/seam-2-adapter-protocol-and-schema/slice-1-dispatch-lifecycle-and-owner-line.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threaded-seams/seam-2-adapter-protocol-and-schema/slice-2-schema-subset-and-fail-closed-capability-rules.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threaded-seams/seam-2-adapter-protocol-and-schema/slice-3-adoption-surfaces-and-verification.md`
- **Contracts published or changed**: `C-03`, `C-04`
- **Threads published / advanced**: `THR-02`
- **Review-surface delta**: one deterministic adapter dispatch lifecycle, one bounded adopted schema subset, and one explicit local-to-external owner line for ADR-0017 and ADR-0028. No second Substrate control plane was introduced.
- **Planned-vs-landed delta**: the planned closeout-backed handoff landed as specified; the durable outputs are the canonical protocol and schema contracts, the seam-local execution baselines, and the publication of `THR-02`.
- **Downstream stale triggers raised**:
  - capability ids or extension-key subset changes
  - request, response, error, or session-handle schema changes
  - ADR-0017 or ADR-0028 owner-line wording changes
  - any widening of the adapter boundary after publication
- **Remediation disposition**:
  - `REM-002`: carried forward in `governance/remediation-log.md` as a non-blocking SEAM-2 follow-up for schema alignment; it does not block `THR-02` publication.
  - `REM-003`: carried forward in `governance/remediation-log.md` as a non-blocking SEAM-2 follow-up for protocol and owner-line alignment; it does not block `THR-02` publication.
- **Promotion blockers**: none for the SEAM-2 exit gate
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: `REM-002`, `REM-003` remain open in the pack remediation log as carried-forward follow-up, but they are not exit-gate blockers.
- **Carried-forward remediations**:
  - `REM-002`
  - `REM-003`
