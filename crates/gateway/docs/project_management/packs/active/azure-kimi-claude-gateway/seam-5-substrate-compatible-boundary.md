---
seam_id: SEAM-5
seam_slug: substrate-compatible-boundary
type: conformance
status: landed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-2-closeout.md
    - governance/seam-3-closeout.md
    - governance/seam-4-closeout.md
  required_threads:
    - THR-02
    - THR-03
    - THR-04
    - THR-05
  stale_triggers:
    - any later `SEAM-3` contract or closeout change that alters the published public surface or thin-adapter boundary requires revalidation
    - any later `SEAM-4` policy contract or closeout change that alters public identity or session-handoff assumptions requires revalidation
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
  planned_location: reserved_final_slice
  status: passed
open_remediations: []
---

# SEAM-5 - Substrate Compatible Boundary

- **Goal / value**: lock the gateway into one logical backend identity, a replaceable in-world-compatible deployment boundary, and normalized structured downstream events so future Substrate integration is an integration task instead of a rewrite.
- **Scope**
  - In:
    - public backend identity and capability-oriented naming
    - deployment and auth factoring that keeps loopback-local dev as a convenience rather than a contract
    - downstream structured-event boundary for shell or agent-hub consumption
    - drift guards, docs, and conformance evidence that keep upstream seams from leaking internal roles or raw provider streams
  - Out:
    - direct Substrate implementation
    - host-specific operational glue as the defining architecture
    - ownership of the Anthropic public API or Azure provider parser themselves
- **Primary interfaces**
  - Inputs:
    - `C-02` normalized event contract
    - `C-03` Anthropic Messages gateway contract
    - `C-04` internal policy contract
    - the Substrate memo and ADR 0005 through ADR 0007 constraints
  - Outputs:
    - `C-05` external identity and deployment-boundary contract
    - `C-06` downstream structured-event contract
    - conformance and drift-guard evidence
- **Key invariants / rules**:
  - the gateway presents one logical backend identity to external consumers
  - internal planner/executor and provider-normalization details remain internal
  - local loopback transport is not the only viable deployment boundary
  - downstream integrations consume normalized structured events rather than raw provider streams
- **Dependencies**
  - Direct blockers:
    - `SEAM-2` via `THR-02`
    - `SEAM-3` via `THR-03`
    - `SEAM-4` via `THR-04`
  - Transitive blockers:
    - `SEAM-1` via `THR-01`
  - Direct consumers:
    - future Substrate integration work outside this pack
  - Derived consumers:
    - operators, policy surfaces, shell or agent-hub consumers
- **Touch surface**:
  - public-facing config and naming docs
  - transport and auth adapter boundaries
  - downstream event schema or adapter surfaces
  - conformance tests and drift guards
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Verify public configuration and docs describe one stable backend identity rather than separate internal roles.
  - Verify deployment and auth factoring preserve an in-world-compatible path beyond localhost.
  - Verify downstream consumers can rely on stable structured events rather than raw provider chunks.
- **Risks / unknowns**:
  - Risk: earlier seams may accidentally freeze internal model roles or loopback assumptions into public docs and config.
  - De-risk plan: make this seam the explicit conformance owner and require drift guards before closeout.
  - Risk: normalized events may remain too provider-shaped for downstream consumers.
  - De-risk plan: seam-local review should compare actual landed events against downstream rendering and persistence needs, not just provider convenience.
- **Rollout / safety**:
  - defer hard conformance lock-in until upstream seams land enough truth to justify it
  - use this seam to prevent future Substrate integration from inheriting accidental local-dev architecture
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is now `future` because the seam has landed, published `THR-05`, and left the pack's forward execution window
  - Which threads matter most: `THR-03`, `THR-04`, `THR-05`
  - What the first seam-local review should focus on: public identity drift, transport/auth factoring, and structured-event sufficiency for downstream consumers
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-05`, `C-06`
  - Threads likely to advance: `THR-05`
  - Review-surface areas likely to shift after landing: `R2`, `R3`
  - Downstream seams most likely to require revalidation: future Substrate packs and any later API-adapter seams
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
