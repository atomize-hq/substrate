---
seam_id: SEAM-2
seam_slug: json-health-disable-attribution
type: integration
status: proposed
execution_horizon: next
plan_version: v1
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - SEAM-1 closeout changes message bodies, precedence truth, or fallback posture
    - health or shim reporting changes root-object shape or nested doctor behavior
    - queued JSON envelope or provisioning work changes top-level payload expectations
    - tokenized path/env rendering rules change after this seam is reviewed
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
  planned_location: reserved_final_slice
  status: pending
open_remediations: []
---

# SEAM-2 - JSON + health disable attribution

- **Source pack mapping**: corresponds to `DHO1` in the deep-researched source plan.
- **Goal / value**: publish the same disable-attribution truth as stable top-level JSON and health output so tooling and operators have one deterministic explanation path across doctor and health.
- **Scope**
  - In:
    - additive top-level `world_disable_reason` and `world_disable_source` for host doctor, world doctor, and health JSON
    - health text parity with the doctor message-body contract
    - nested doctor/shim plumbing needed so `substrate health --no-world` preserves `cli_flag` attribution end-to-end
    - Linux/macOS/Windows parity for JSON field names, enum values, and redaction rules where the surface exists
  - Out:
    - new exit codes or failure semantics
    - any rename or removal of existing JSON fields
    - replay warnings or world-adjacent messaging outside doctor and health
    - changes to precedence or world enablement behavior
- **Primary interfaces**
  - Inputs:
    - `C-01` exact message-body contract from `SEAM-1`
    - `C-02` effective-winner and redaction truth from `SEAM-1`
    - health and shim reporting paths
    - existing doctor JSON payload anchors and health root object
  - Outputs:
    - `C-03` additive JSON schema contract
    - health text output that mirrors doctor attribution exactly
    - health JSON and doctor JSON fields that carry the same disable source without text scraping
- **Key invariants / rules**:
  - top-level `world_disable_reason` and `world_disable_source` emit only when the effective world state is disabled and omit together when it is enabled
  - enum vocabulary stays `cli_flag | override_env | workspace_patch | global_patch | default | source_unknown`
  - `world_disable_source.key` is always `world.enabled`; `layer` matches the reason; `value_display` stays `false`
  - conditional JSON keys are stable: `flag=--no-world`, `env=SUBSTRATE_OVERRIDE_WORLD`, `path_display` only `<workspace>/.substrate/workspace.yaml` or `$SUBSTRATE_HOME/config.yaml`
  - health text must reuse the same exact message bodies as doctor text; nested doctor invocation paths must not lose CLI attribution
- **Dependencies**
  - Direct blockers:
    - `SEAM-1` closeout must publish `C-01` and `C-02` so the seam consumes recorded upstream truth rather than inferred intent
    - health/shim reporting must be able to carry the shared attribution model without mutating it
    - adjacent JSON envelope work must preserve top-level field placement for this feature's additive contract
  - Transitive blockers:
    - disabled-status UX and provisioning queues may reframe health output while this seam is still provisional
    - future terminology migration could rename layers while keeping meaning constant, forcing schema-note updates
  - Direct consumers:
    - operators running `substrate health`
    - automation and CI reading doctor/health JSON
  - Derived consumers:
    - future JSON envelope work
    - provisioning-related health messaging that must preserve disable-attribution fields and wording
- **Touch surface**:
  - `crates/shell/src/builtins/health.rs`
  - `crates/shell/src/builtins/shim_doctor/report.rs`
  - `crates/shell/src/execution/platform/mod.rs`
  - doctor JSON renderers and health JSON emitters
  - source-plan schema spec, `DHO1` spec, manual playbook, smoke evidence, and checkpoint plan surfaces
- **Verification**:
  - This seam **consumes** upstream contracts from `SEAM-1`, so verification later depends on accepted upstream closeout evidence for `C-01` and `C-02`.
  - This seam **produces** the additive JSON contract `C-03`; at seam-brief depth readiness means field placement, emit/omit rules, enum set, and health-parity behavior are concrete enough for seam-local planning and implementation.
  - Downstream seam-local review should verify top-level placement on every in-scope payload, enabled-case omission, health text parity, CLI-flag preservation through nested paths, and cross-platform parity.
- **Risks / unknowns**:
  - Risk: health and shim paths reword or suppress the doctor-originated message body.
    - De-risk plan: make `THR-02` explicit, require parity tests, and review nested `--no-world` flows separately from plain doctor flows.
  - Risk: queued JSON envelope work changes root-object shape and invalidates one stable JSONPath.
    - De-risk plan: keep this seam provisional until upstream/output-shape assumptions are revalidated and capture any drift in closeout stale triggers.
  - Risk: redaction or conditional-key behavior diverges across platforms.
    - De-risk plan: anchor all fields to one schema contract and run platform parity checks at the full checkpoint before seam exit.
- **Rollout / safety**:
  - additive only; existing JSON consumers that ignore unknown fields remain compatible
  - health and doctor stay aligned on disable-attribution truth; no new knobs or policy surfaces are introduced
  - unsupported Windows host-doctor posture remains unchanged while world doctor and health still gain the structured fields where supported
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `next` because its correctness depends on the upstream doctor-text contract becoming published truth first
  - Which threads matter most: `THR-01` carries the shared attribution model; `THR-02` is the parity guardrail for health and nested doctor flows
  - What the first seam-local review should focus on: top-level field placement, enabled-case omission, nested CLI-flag preservation, and compatibility with adjacent health/JSON work
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-03`
  - Threads likely to advance: `THR-01` should move toward `revalidated`; `THR-02` should move toward `published` once health parity is proven
  - Review-surface areas likely to shift after landing: health root-object shape, doctor/health JSON examples, and operator guidance for when fields are omitted
  - Downstream seams most likely to require revalidation: future JSON envelope work, provisioning-related health packs, and any surface that reuses the disable-attribution schema or wording
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
