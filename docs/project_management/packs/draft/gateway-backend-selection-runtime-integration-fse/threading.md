# Threading - gateway-backend-selection-runtime-integration

## Execution horizon summary

- **Active seam**: `SEAM-1`
  - This seam locks the selection/policy handoff and lands consumer alignment so runtime work can proceed on fixed inputs.
- **Next seam**: `SEAM-2`
  - This seam implements the adapter-driven runtime path from the `SEAM-1` handoff.
- **Future seams**: `SEAM-3`
  - This seam verifies parity and rollout posture after runtime realization exists and a named additional backend is chosen.

Horizon policy for this pack:

- only the active seam gets authoritative downstream deep planning by default
- the next seam starts after `SEAM-1` closes the remaining alignment work and lands implementation evidence
- the future seam remains deferred until runtime realization exists and a later rollout baseline is intentional rather than speculative

## Contract registry

- **Contract ID**: `C-01`
  - **Type**: `config`
  - **Owner seam**: `SEAM-1`
  - **Direct consumers**: `SEAM-2`, `SEAM-3`
  - **Derived consumers**: shell gateway entrypoints, broker/config readers, runtime tests
  - **Thread IDs**: `THR-01`
  - **Definition**: the integrated lifecycle selection boundary over existing config, policy, and inventory inputs: stable backend id selection, backend-id grammar, one-file-per-backend posture, filename/id consistency, deny-by-default allowlisting, and the trusted-input boundary that excludes gateway-local persistence and mutation from authorization.
  - **Canonical contract ref**: `docs/contracts/substrate-gateway-backend-adapter-selection.md`
  - **Supporting feature-local surfaces**:
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/contract.md`
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/policy-spec.md`
  - **Versioning / compat**: canonical publication stays in `docs/contracts/substrate-gateway-backend-adapter-selection.md`; this pack only aligns implementation and supporting ADR-0046 docs to it.

- **Contract ID**: `C-02`
  - **Type**: `permission`
  - **Owner seam**: `SEAM-1`
  - **Direct consumers**: `SEAM-2`, `SEAM-3`
  - **Derived consumers**: auth material sourcing logic, failure taxonomy, security review
  - **Thread IDs**: `THR-01`
  - **Definition**: the integrated lifecycle policy-evaluation and auth-sourcing boundary: fail-closed posture, host env-read gating, host-credential-read gating, no-host-fallback rules when in-world execution is required, and the precedence rules for authorized auth material.
  - **Canonical contract ref**: `docs/contracts/substrate-gateway-policy-evaluation.md`
  - **Supporting feature-local surfaces**:
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/policy-spec.md`
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/env-vars-spec.md`
  - **Versioning / compat**: reused ADR-0027 keys stay externally owned; this pack aligns implementation and supporting ADR-0046 docs to the published policy contract rather than reopening it.

- **Contract ID**: `C-03`
  - **Type**: `API`
  - **Owner seam**: `SEAM-2`
  - **Direct consumers**: `SEAM-3`
  - **Derived consumers**: world-agent service, runtime launch path, lifecycle restart handling
  - **Thread IDs**: `THR-02`
  - **Definition**: the integrated adapter realization protocol after selection succeeds: one binding lookup, required capability gate, auth handoff validation order, adapter-driven config render, launch, readiness, and restart semantics.
  - **Canonical contract ref**: `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
  - **Supporting feature-local surfaces**:
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-protocol-spec.md`
  - **Versioning / compat**: canonical publication stays in `docs/contracts/substrate-gateway-backend-adapter-protocol.md`; `SEAM-2` must implement it without widening `status --json` or operator commands.

- **Contract ID**: `C-04`
  - **Type**: `schema`
  - **Owner seam**: `SEAM-2`
  - **Direct consumers**: `SEAM-3`
  - **Derived consumers**: shared request types, integrated auth payloads, runtime artifact handling, failure reporting
  - **Thread IDs**: `THR-02`
  - **Definition**: the runtime-owned realization data surfaces needed to support more than `cli:codex`: integrated auth payload shapes, runtime config payloads, managed runtime artifact naming/permission rules, and any shared types required for adapter-driven lifecycle behavior.
  - **Canonical contract ref**: `docs/contracts/substrate-gateway-backend-adapter-schema.md`
  - **Supporting feature-local surfaces**:
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-schema-spec.md`
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/filesystem-semantics-spec.md`
  - **Versioning / compat**: schema hardening is part of `SEAM-2` implementation if needed; this pack does not treat it as a prerequisite new contract-publication phase.

- **Contract ID**: `C-05`
  - **Type**: `state`
  - **Owner seam**: `SEAM-3`
  - **Direct consumers**: none inside this pack
  - **Derived consumers**: validation artifacts, compatibility notes, smoke scripts, downstream rollout review
  - **Thread IDs**: `THR-03`
  - **Definition**: parity and rollout proof for the selected-backend lifecycle: Linux/macOS/Windows validation expectations, `cli:codex` regression floor, explicit unsupported-backend behavior, and later first-additional-backend proof.
  - **Canonical contract ref**: `docs/contracts/substrate-gateway-runtime-parity.md`
  - **Supporting feature-local surfaces**:
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/platform-parity-spec.md`
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/compatibility-spec.md`
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/manual_testing_playbook.md`
  - **Consumed external authorities**:
    - `docs/contracts/substrate-gateway-operator-contract.md`
    - `docs/contracts/substrate-gateway-policy-evaluation.md`
  - **Versioning / compat**: the runtime-parity contract owns lifecycle/status parity; future additional-backend compatibility publication is deferred until that rollout work actually begins.

## Thread registry

- **Thread ID**: `THR-01`
  - **Producer seam**: `SEAM-1`
  - **Consumer seam(s)**: `SEAM-2`, `SEAM-3`
  - **Carried contract IDs**: `C-01`, `C-02`
  - **Purpose**: make the existing selection and policy contracts executable in repo consumers so runtime realization does not infer truth from the current Codex-only path.
  - **State**: `published`
  - **Revalidation trigger**: selection order, backend inventory rules, allowlist semantics, auth precedence, or policy failure taxonomy changes.
  - **Satisfied by**: `governance/seam-1-closeout.md` plus evidence that shell, broker, config/policy surfaces, and supporting ADR-0046 docs align to `docs/contracts/substrate-gateway-backend-adapter-selection.md` and `docs/contracts/substrate-gateway-policy-evaluation.md`.
  - **Notes**: this thread is already published at the canonical-contract level; `SEAM-1` exists to align consumers and close the remaining narrow ambiguity, not to invent new ownership.

- **Thread ID**: `THR-02`
  - **Producer seam**: `SEAM-2`
  - **Consumer seam(s)**: `SEAM-3`
  - **Carried contract IDs**: `C-03`, `C-04`
  - **Purpose**: land one integrated runtime realization path that parity and rollout can verify without inventing binding, capability, auth, or artifact behavior.
  - **State**: `defined`
  - **Revalidation trigger**: binding lookup rules, capability gates, auth handoff validation, runtime payload shapes, artifact naming, readiness semantics, or restart behavior changes.
  - **Satisfied by**: `governance/seam-2-closeout.md` plus evidence that shell, `world-agent`, and shared agent-api surfaces implement the published adapter-protocol and runtime-owned schema surfaces without widening unrelated external ownership.
  - **Notes**: this thread must not publish tuple metadata or status-schema widening as part of runtime realization.

- **Thread ID**: `THR-03`
  - **Producer seam**: `SEAM-3`
  - **Consumer seam(s)**: none inside this pack
  - **Carried contract IDs**: `C-05`
  - **Purpose**: verify parity and later rollout posture after the runtime path exists.
  - **State**: `defined`
  - **Revalidation trigger**: first-additional-backend baseline changes, parity matrix changes, unsupported-backend failure posture changes, or `cli:codex` regression guarantees change.
  - **Satisfied by**: `governance/seam-3-closeout.md` plus validation evidence across Linux/macOS/Windows once a later additional backend is intentionally chosen.
  - **Notes**: this thread is intentionally deferred; it is not a blocker for the current execution target.

## Dependency graph

```mermaid
flowchart LR
  S1["SEAM-1"] -- "THR-01 / C-01 + C-02" --> S2["SEAM-2"]
  S2 -- "THR-02 / C-03 + C-04" --> S3["SEAM-3"]
  S3 -- "THR-03 / C-05" --> OUT["Downstream rollout and release governance"]
```

## Critical path

1. `SEAM-1` first:
   - lock the selection and policy handoff in implementation surfaces
   - finish the narrow `REM-001` / `REM-002` alignment work and land consumer evidence
2. `SEAM-2` second:
   - implement adapter lookup, capability gating, auth validation, config render, manifests, readiness, and restart behavior using the `SEAM-1` handoff
   - any schema hardening happens only as needed to land runtime behavior
3. `SEAM-3` third:
   - validate parity and rollout after the runtime path exists
   - choose and prove an additional backend only when the project is ready for that rollout step

## Workstreams

- **Selection and policy implementation lane**
  - Primary seam: `SEAM-1`
  - Focus: selected-backend source of truth, allowlists, auth precedence, inventory/root alignment, trusted-input boundary, broker/shell/config consumer evidence
- **Runtime realization lane**
  - Primary seam: `SEAM-2`
  - Focus: binding lookup, capability gates, auth validation, config render, artifact semantics, launch and restart order
- **Parity and rollout lane**
  - Primary seam: `SEAM-3`
  - Focus: regression matrix, unsupported-backend behavior, Linux/macOS/Windows evidence, later additional-backend rollout proof

Workstream note:

- These lanes follow the old `GBSRI-*` lineage but the current pack treats them as execution work, not seam-extraction outputs.
