---
seam_id: SEAM-1
seam_slug: azure-foundry-runtime-transport
status: decomposed
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-1-azure-foundry-runtime-transport.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-2-closeout.md
    - crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md
    - crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-4-closeout.md
    - crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-5-closeout.md
  required_threads:
    - THR-06
  stale_triggers:
    - live Azure runtime requirements for auth, deployment path, or `api-version` prove incompatible with the current transport assumptions captured in `gateway/src/providers/openai.rs`
    - the landed `C-03` or `C-04` contracts change the intended `Kimi-K2-Thinking` or `Kimi-K2.5` routing posture
    - non-Azure OpenAI-compatible providers would regress unless Azure transport logic is isolated behind an explicit provider-mode boundary
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S4
  status: pending
open_remediations: []
---
# SEAM-1 - Azure Foundry Runtime Transport

## Seam Brief (Restated)

- **Goal / value**: freeze the owned `C-07` Azure Foundry runtime transport contract so the gateway can express Azure auth handling, deployment-scoped request targets, `api-version`, and Kimi deployment mapping as first-class provider behavior without reopening the landed public or orchestration contracts above it.
- **Type**: `integration`
- **Scope**
  - **In**:
    - define the owned `C-07` contract for Azure auth mode, deployment URL construction, `api-version` propagation, and provider/model mapping for `Kimi-K2-Thinking` and `Kimi-K2.5`
    - create or tighten the provider-mode boundary that keeps Azure request construction isolated from other OpenAI-compatible providers
    - align config, model-mapping, and example surfaces with the transport contract
    - add deterministic verification that proves URL, header, query, and request-body construction without live credentials
  - **Out**:
    - reopening `C-02`, `C-03`, `C-04`, `C-05`, or `C-06`
    - redesigning the Anthropic `/v1/messages` surface or planner/executor policy
    - live smoke validation, operator troubleshooting, or redacted evidence capture that belong to `SEAM-2`
    - broad refactors across unrelated OpenAI-compatible providers
- **Touch surface**:
  - `gateway/src/providers/openai.rs`
  - `gateway/src/providers/registry.rs`
  - `gateway/src/providers/mod.rs`
  - `gateway/src/cli/mod.rs`
  - `gateway/config/default.example.toml`
  - `gateway/config/models.example.toml`
  - `gateway/README.md`
  - transport-focused verification surfaces under `gateway/tests/` or provider-local test modules
- **Verification**:
  - `C-07` is concrete enough only if the seam names one Azure transport rule set for auth headers, deployment-scoped target construction, `api-version`, and routing-ready model/deployment mapping under a bounded provider-mode boundary.
  - pre-exec readiness does not require the final accepted contract artifact to be published yet, but it does require the contract rules and verification checklist to be concrete enough that implementation can proceed without guessing.
  - deterministic tests must prove Azure-specific URL, header, query, and request-body construction while also proving non-Azure OpenAI-compatible flows keep their prior behavior.
  - config and example surfaces must express both intended Azure Kimi deployments without exposing planner/executor roles as public backend identities.
- **Basis posture**:
  - **Currentness**: `current`
  - **Upstream closeouts assumed**:
    - `crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-2-closeout.md`
    - `crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md`
    - `crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-4-closeout.md`
    - `crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-5-closeout.md`
  - **Required threads**: `THR-06`
  - **Stale triggers**:
    - Azure runtime truth changes the required auth header posture, deployment URL shape, or `api-version` semantics
    - routing truth for `Kimi-K2-Thinking` or `Kimi-K2.5` changes upstream and invalidates the planned model/deployment mapping
    - Azure handling bleeds into generic OpenAI-compatible behavior and breaks the provider-mode isolation assumed here
- **Threading constraints**
  - **Upstream blockers**: none; the upstream gateway basis is already landed and `THR-06` is ready for contract-definition work
  - **Downstream blocked seams**: `SEAM-2`
  - **Contracts produced**: `C-07`
  - **Contracts consumed**: landed `C-02`, `C-03`, `C-04`, `C-05`, and `C-06` basis constraints

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S4`
- **Why this seam needs an explicit exit gate**: `SEAM-2` cannot safely promote on "Azure transport code exists"; it needs closeout-backed proof that `C-07` landed, that `THR-06` advanced from `defined` to `published`, and that downstream stale triggers are explicit if live Azure reality forced any transport deltas.
- **Expected contracts to publish**: `C-07`
- **Expected threads to publish / advance**: `THR-06` from `defined` to `published`
- **Likely downstream stale triggers**:
  - `SEAM-2` if landed Azure auth, deployment, or `api-version` semantics differ from the contract frozen in `S1`
  - `SEAM-2` if the final transport implementation needs config fields or routing semantics that were not planned into `C-07`
  - future work outside this pack if Azure transport assumptions prove host-local or credential-topology specific instead of remaining deployment-boundary compatible
- **Expected closeout evidence**:
  - a canonical `C-07` contract source that names auth, URL, query, mapping, and isolation rules
  - landed code-local evidence showing Azure-specific request construction behind a bounded provider-mode/helper boundary
  - deterministic transport tests covering both Kimi deployments and non-regression expectations for non-Azure OpenAI-compatible providers
  - config/example surfaces that match the landed transport contract closely enough for `SEAM-2` to consume as basis
  - closeout accounting for any planned-versus-landed deltas that would force downstream revalidation

## Slice index

- `S1` -> `slice-1-freeze-azure-runtime-transport-contract.md`
- `S2` -> `slice-2-implement-azure-provider-transport-boundary.md`
- `S3` -> `slice-3-lock-config-examples-and-transport-tests.md`
- `S4` -> `slice-4-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
