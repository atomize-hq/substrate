---
seam_id: SEAM-5
seam_slug: substrate-compatible-boundary
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-5-substrate-compatible-boundary.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-2-closeout.md
    - ../../governance/seam-3-closeout.md
    - ../../governance/seam-4-closeout.md
  required_threads:
    - THR-02
    - THR-03
    - THR-04
    - THR-05
  stale_triggers:
    - "`docs/foundation/anthropic-messages-c03-contract.md` changes public session continuation or thin-adapter assumptions in a way that alters external identity or loop behavior"
    - "`docs/foundation/azure-kimi-c02-normalized-event-contract.md` changes normalized event semantics or stable field guarantees in a way that alters downstream structured-event assumptions"
    - "`docs/foundation/planner-executor-c04-policy-contract.md` changes policy or state-handoff assumptions in a way that alters public identity or boundary rules"
    - public docs, config, or event-shape work starts exposing internal roles or raw provider streams
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S3
  status: passed
open_remediations: []
---
# SEAM-5 - Substrate Compatible Boundary

## Seam Brief (Restated)

- **Goal / value**: define the external boundary that keeps one logical backend identity, replaceable deployment/auth factoring, and normalized structured events stable for later Substrate integration.
- **Type**: `conformance`
- **Scope**
  - **In**:
    - public capability naming and one logical backend identity
    - deployment and auth factoring that keeps loopback-local dev as a convenience rather than a contract
    - downstream structured-event boundary for shell or agent-hub consumers
    - drift guards, docs, and conformance evidence that keep upstream seams from leaking internal roles or raw provider streams
  - **Out**:
    - direct Substrate implementation
    - host-specific operational glue as the defining architecture
    - ownership of the Anthropic public API or Azure provider parser themselves
- **Touch surface**:
  - public-facing config and naming docs
  - transport and auth adapter boundaries
  - downstream event schema or adapter surfaces
  - conformance tests and drift guards
- **Verification**:
  - public configuration and docs describe one stable backend identity rather than separate internal roles
  - deployment and auth factoring preserve an in-world-compatible path beyond localhost
  - downstream consumers can rely on stable structured events rather than raw provider chunks
  - the seam can publish `C-05` and `C-06` without forcing later seams to reverse-engineer runtime identity or transport choices
- **Basis posture**:
  - **Currentness**: `current`
  - **Upstream closeouts assumed**: `../../governance/seam-2-closeout.md`, `../../governance/seam-3-closeout.md`, `../../governance/seam-4-closeout.md`
  - **Required threads**: `THR-02`, `THR-03`, `THR-04`, `THR-05`
  - **Stale triggers**:
    - `C-03` changes public session continuation or thin-adapter assumptions in a way that alters the external boundary
    - `C-04` changes internal policy or state-handoff assumptions in a way that alters public identity rules
    - `C-02` changes normalized event semantics or stable field guarantees in a way that alters downstream event shape
- **Threading constraints**
  - **Upstream blockers**: `THR-02`, `THR-03`, and `THR-04` are current inputs; `THR-05` is the seam-owned outbound thread
  - **Downstream blocked seams**: none in this pack
  - **Contracts produced**: `C-05`, `C-06`
  - **Contracts consumed**: `C-02`, `C-03`, `C-04`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S3`
- **Why this seam needs an explicit exit gate**: downstream integration work cannot safely assume the external boundary is correct until one logical backend identity, replaceable deployment/auth factoring, and normalized structured events are backed by closeout evidence rather than hopeful naming
- **Expected contracts to publish**: `C-05`, `C-06`
- **Expected threads to publish / advance**: `THR-05` from `identified` to `published`
- **Likely downstream stale triggers**:
  - future Substrate packs if public identity, transport assumptions, or event-shape constraints drift
  - any later adapter seam if normalized structured events start depending on raw provider stream details
- **Expected closeout evidence**:
  - canonical `C-05` and `C-06` contract notes or equivalent internal sources of truth
  - verification that public configuration and docs present one stable backend identity
  - verification that deployment and auth remain replaceable
  - verification that downstream consumers consume normalized structured events rather than raw provider chunks

## Slice index

- `S1` -> `slice-1-freeze-public-identity-and-deployment-boundary.md`
- `S2` -> `slice-2-deliver-structured-events-and-drift-guards.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-5-closeout.md`
