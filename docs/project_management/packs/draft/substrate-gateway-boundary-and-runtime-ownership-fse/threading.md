# Threading - substrate-gateway-boundary-and-runtime-ownership

## Execution horizon summary

- **Active seam**: `SEAM-4`
  - This seam now consumes the published operator, schema, policy, and runtime/parity contracts and turns them into cross-doc validation and quality-gate lock-in.
- **Next seam**: `null`
  - No additional safe queued seam remains inside this pack once `SEAM-4` becomes active.
- **Future seams**: `SEAM-1`, `SEAM-2`, `SEAM-3`
  - `SEAM-1` has landed with a passed seam-exit gate and left the forward planning window.
  - `SEAM-2` has landed with a passed seam-exit gate and left the forward planning window.
  - `SEAM-3` has landed with a passed seam-exit gate and left the forward planning window.

Horizon policy for this extracted pack:

- only the active seam gets authoritative downstream deep planning by default
- there is no queued `next` seam because `SEAM-4` is the terminal seam in this pack
- `SEAM-1` has now landed with a passed seam-exit gate and left the forward planning window
- `SEAM-2` has now landed with a passed seam-exit gate and left the forward planning window
- `SEAM-3` has now landed with a passed seam-exit gate and left the forward planning window
- the remaining future seams stay landed seam briefs unless a downstream pack explicitly reopens them

## Contract registry

- **Contract ID**: `C-01`
  - **Type**: `API`
  - **Owner seam**: `SEAM-1`
  - **Direct consumers**: `SEAM-2`, `SEAM-3`, `SEAM-4`
  - **Derived consumers**: operators, shell builtins, docs, and downstream validation artifacts
  - **Thread IDs**: `THR-01`
  - **Definition**: the Substrate-owned operator boundary for `substrate world gateway sync`, `status`, and `restart`, including absent-state behavior, stable wiring entrypoint rules, stable non-secret env outputs, exit-code boundaries, and the durable ownership split against `substrate-gateway`.
  - **Canonical contract ref**: `docs/contracts/substrate-gateway-operator-contract.md`
  - **Versioning / compat**: command spelling, exit-code mapping, stable env names, and the rule that `status --json` is the machine-readable wiring authority must remain stable; additive operator prose must not redefine these semantics.

- **Contract ID**: `C-02`
  - **Type**: `schema`
  - **Owner seam**: `SEAM-2`
  - **Direct consumers**: `SEAM-3`, `SEAM-4`
  - **Derived consumers**: operator docs, tests, and any world-internal clients that consume the stable wiring surface
  - **Thread IDs**: `THR-02`
  - **Definition**: the structured output contract for `substrate world gateway status --json`, including the top-level object shape, `client_wiring.*` field family, non-secret posture, conditional presence rules, and the hard boundary against ADR-0042 additive metadata outside that family.
  - **Canonical contract ref**: `docs/contracts/substrate-gateway-status-schema.md`
  - **Versioning / compat**: field names, omission rules, and `client_wiring.*` semantics must remain compatible; additive fields require downstream revalidation when they touch operator-facing meaning.

- **Contract ID**: `C-03`
  - **Type**: `permission`
  - **Owner seam**: `SEAM-2`
  - **Direct consumers**: `SEAM-3`, `SEAM-4`
  - **Derived consumers**: policy explanations, platform parity docs, and manual validation artifacts
  - **Thread IDs**: `THR-03`
  - **Definition**: the gateway-integration decision flow over existing ADR-0027 inputs, including fail-closed in-world placement, host secret sourcing and host-to-world secret delivery boundaries, distinction between invalid integration state and dependency unavailability, and the ban on trusting gateway-local config/admin/persistence as policy inputs.
  - **Canonical contract ref**: `docs/contracts/substrate-gateway-policy-evaluation.md`
  - **Versioning / compat**: reused key paths stay externally owned, but the decision taxonomy and no-host-fallback rule must remain stable; changes require runtime and docs revalidation.

- **Contract ID**: `C-04`
  - **Type**: `API`
  - **Owner seam**: `SEAM-3`
  - **Direct consumers**: `SEAM-4`
  - **Derived consumers**: shell builtins, shared agent API clients, parity docs, and quality-gate evidence
  - **Thread IDs**: `THR-04`
  - **Definition**: the typed world-agent lifecycle/status contract and the Linux/macOS/Windows parity guarantees that let CLI behavior stay stable without raw exec probing or platform-specific operator contracts.
  - **Canonical contract ref**: `docs/contracts/substrate-gateway-runtime-parity.md`
  - **Versioning / compat**: endpoint ownership, lifecycle/status semantics, allowed divergence, and required validation evidence must stay synchronized across host and backend consumers.

## Thread registry

- **Thread ID**: `THR-01`
  - **Producer seam**: `SEAM-1`
  - **Consumer seam(s)**: `SEAM-2`, `SEAM-3`, `SEAM-4`
  - **Carried contract IDs**: `C-01`
  - **Purpose**: publish one operator boundary so every downstream seam inherits the same command family, ownership split, stable env names, and exit taxonomy.
  - **State**: `revalidated`
  - **Revalidation trigger**: command spelling, absent-state wording, stable env names, exit-code boundaries, or the Substrate versus `substrate-gateway` ownership split changes.
  - **Satisfied by**: `governance/seam-1-closeout.md` records the landed operator contract, command-family proof, absent-state evidence, and stale triggers. `threaded-seams/seam-2-status-schema-and-policy-evaluation-surface/review.md` first revalidated that handoff for downstream planning, and `threaded-seams/seam-4-validation-and-cross-doc-lock-in/review.md` now revalidates it again for the terminal conformance seam.
  - **Notes**: this is the first critical-path handoff and the main protection against archived ADR-0023 command drift. The thread remains active because `SEAM-4` still needs the same operator boundary to stay visible in playbook, docs, and quality-gate evidence.

- **Thread ID**: `THR-02`
  - **Producer seam**: `SEAM-2`
  - **Consumer seam(s)**: `SEAM-3`, `SEAM-4`
  - **Carried contract IDs**: `C-02`
  - **Purpose**: keep machine-readable status, wiring discovery, and ADR-0042 boundary semantics single-source before runtime and docs consume them.
  - **State**: `revalidated`
  - **Revalidation trigger**: top-level JSON shape, `client_wiring.*` field family, omission rules, or the boundary against ADR-0042 additive metadata changes.
  - **Satisfied by**: `governance/seam-2-closeout.md` records the landed schema contract, durable contract mirror, and downstream stale triggers for the status envelope and `client_wiring.*` family. `threaded-seams/seam-4-validation-and-cross-doc-lock-in/review.md` revalidates that handoff against the current manual-testing, docs, and quality-gate touch set.
  - **Notes**: this thread now carries the revalidated machine-readable status and wiring contract into the active terminal conformance seam.

- **Thread ID**: `THR-03`
  - **Producer seam**: `SEAM-2`
  - **Consumer seam(s)**: `SEAM-3`, `SEAM-4`
  - **Carried contract IDs**: `C-03`
  - **Purpose**: keep fail-closed placement, secret delivery, and gateway-local non-trust rules single-source across runtime and validation seams.
  - **State**: `revalidated`
  - **Revalidation trigger**: reused ADR-0027 keys, no-host-fallback posture, exit boundary taxonomy, or trust-boundary rules change.
  - **Satisfied by**: `governance/seam-2-closeout.md` records the landed policy contract, durable contract mirror, and downstream stale triggers for fail-closed placement and non-trust rules. `threaded-seams/seam-4-validation-and-cross-doc-lock-in/review.md` revalidates that handoff against the current validation and doc-lock-in scope.
  - **Notes**: this thread now carries the revalidated policy-evaluation and trust-boundary contract into the active terminal conformance seam.

- **Thread ID**: `THR-04`
  - **Producer seam**: `SEAM-3`
  - **Consumer seam(s)**: `SEAM-4`
  - **Carried contract IDs**: `C-04`
  - **Purpose**: publish the typed lifecycle/status transport and parity evidence contract that cross-doc validation and quality-gate work will lock in.
  - **State**: `revalidated`
  - **Revalidation trigger**: typed world-agent endpoint shape, shell/client integration path, allowed divergence list, or Linux/macOS/Windows evidence requirements change.
  - **Satisfied by**: `governance/seam-3-closeout.md` records the landed `C-04` publication, the S1 typed runtime boundary in `8c0bd439`, the S2 parity evidence update in `4511b3a5`, the durable contract mirror in `docs/contracts/substrate-gateway-runtime-parity.md`, and the targeted verification reruns for the shared client, the `gateway_runtime_parity` target-local route-shape tests, and the shell gateway tests; host-local runtime-dependent `world-agent` cases self-skipped outside Linux/VM support. `threaded-seams/seam-4-validation-and-cross-doc-lock-in/review.md` revalidates that handoff for the active conformance seam.
  - **Notes**: this thread now carries the revalidated typed lifecycle/status and parity-evidence contract into `SEAM-4`. Provisioning changes remain out of scope for this pack and were not pulled into the published contract.

## Dependency graph

```mermaid
flowchart LR
  S1["SEAM-1"] -- "THR-01 / C-01" --> S2["SEAM-2"]
  S1 -- "THR-01 / C-01" --> S3["SEAM-3"]
  S1 -- "THR-01 / C-01" --> S4["SEAM-4"]
  S2 -- "THR-02 / C-02" --> S3
  S2 -- "THR-02 / C-02" --> S4
  S2 -- "THR-03 / C-03" --> S3
  S2 -- "THR-03 / C-03" --> S4
  S3 -- "THR-04 / C-04" --> S4
```

## Critical path

1. `SEAM-1` first:
   - the operator boundary had to become unambiguous before downstream schema, policy, runtime, or docs work could safely consume it
   - this seam published the main guardrail against archived command spellings and ownership drift
2. `SEAM-2` second:
   - once the operator boundary was fixed and published, the status schema and policy-evaluation surface could become the active authoritative contract layer
   - this seam is now the main guardrail against schema drift and fail-closed drift
3. `SEAM-3` third:
   - runtime transport and platform parity should consume published status/policy truth rather than invent it
   - this seam is the main guardrail against shell-probe and platform-private behavior becoming contract
4. `SEAM-4` fourth:
   - cross-doc validation and quality-gate evidence only make sense after the upstream contracts and runtime/parity rules are concrete enough to verify

## Workstreams

- **Operator boundary lane**
  - Primary seam: `SEAM-1`
  - Focus: command family, absent-state behavior, stable env names, exit taxonomy, ownership split
- **Schema and policy inventory lane**
  - Primary seam: `SEAM-2`
  - Focus: `status --json`, `client_wiring.*`, fail-closed policy flow, secret-delivery trust boundary
- **Typed runtime and parity lane**
  - Primary seam: `SEAM-3`
  - Focus: typed world-agent lifecycle/status transport, shared client wiring, Linux/macOS/Windows guarantees
- **Conformance and checkpoint lane**
  - Primary seam: `SEAM-4`
  - Focus: manual validation, docs alignment, plan/task wiring, checkpoint evidence, pack quality gate

Workstream note:

- These are grouping labels only. Remediation ownership remains seam-only.
