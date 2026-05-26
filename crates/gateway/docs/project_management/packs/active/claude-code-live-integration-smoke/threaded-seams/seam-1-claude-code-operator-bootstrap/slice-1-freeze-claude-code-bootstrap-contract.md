---
slice_id: S1
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: landed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - docs/foundation/azure-foundry-c07-runtime-transport-contract.md changes Azure setup, deployment mapping, or startup assumptions that the bootstrap contract depends on
    - docs/foundation/azure-foundry-c08-operator-verification-contract.md changes the pre-smoke evidence or redaction posture that this slice must freeze
    - gateway/README.md or the config examples drift enough that the contract would encode the wrong operator sequence
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
threads:
  - THR-08
contracts_produced:
  - C-09
contracts_consumed:
  - C-03
  - C-04
  - C-05
  - C-07
  - C-08
open_remediations: []
candidate_subslices: []
---
### S1 - Freeze The Claude Code Bootstrap Contract

- **User/system value**: later implementation and downstream seam planning inherit one concrete operator bootstrap contract instead of rediscovering setup order, file paths, or evidence hooks from code and scattered README sections.
- **Scope (in/out)**:
  - In: define the owned `C-09` contract artifact, the canonical bootstrap order, Azure prerequisite posture, Claude Code attachment rules, and the minimum redaction-safe evidence hooks required before live smoke begins.
  - Out: landing the bootstrap assets themselves, running real smoke sessions, or writing the troubleshooting ownership boundary.
- **Acceptance criteria**:
  - one canonical `C-09` landing path is named: `docs/foundation/claude-code-c09-operator-bootstrap-contract.md`
  - the contract states the bootstrap order from Azure prerequisites through gateway config, startup validation, statusline or tracing enablement, and Claude Code launch
  - the contract keeps `Kimi-K2-Thinking`, `Kimi-K2.5`, and Azure deployment names in internal routing context rather than public product identity
  - the contract names the minimum pre-smoke evidence posture that `SEAM-2` can later assume without rereading runtime code
- **Dependencies**: `../../threading.md`, `crates/gateway/docs/project_management/packs/active/azure-foundry-provider-transport/governance/seam-1-closeout.md`, `crates/gateway/docs/project_management/packs/active/azure-foundry-provider-transport/governance/seam-2-closeout.md`, `docs/foundation/azure-foundry-c07-runtime-transport-contract.md`, `docs/foundation/azure-foundry-c08-operator-verification-contract.md`, `docs/foundation/anthropic-messages-c03-contract.md`, `docs/foundation/planner-executor-c04-policy-contract.md`, `docs/foundation/substrate-boundary-c05-contract.md`
- **Verification**:
  - a reviewer can explain the canonical bootstrap path and minimum evidence posture by reading the contract alone
  - pass condition: `S2` can implement docs, examples, and helper surfaces without inventing any setup semantics
- **Rollout/safety**: keep secrets redacted, keep the contract capability-oriented, and avoid promoting loopback convenience into deployment truth.
- **Review surface refs**: `review.md#r1---bootstrap-workflow-that-should-land`, `review.md#r2---evidence-chain-the-bootstrap-seam-must-make-explicit`

For a contract-definition slice that produces an owned contract:

- make the contract rules concrete enough that the producer seam can later satisfy `gates.pre_exec.contract`
- include a narrow verification plan with artifact path, evidence-hook expectations, boundary rules, and pass/fail checks
- do not require the final accepted contract artifact to exist before the producer seam can become `exec-ready`

#### S1.T1 - Freeze Bootstrap Prerequisites And Config Boundaries

- **Outcome**: `C-09` names the required Azure credentials, deployment identifiers, config files, and gateway startup assumptions without redefining `C-07`.
- **Inputs/outputs**: inputs from `C-07`, `C-08`, README, config examples; output is the prerequisite and config section of `C-09`
- **Thread/contract refs**: `THR-08`, `C-09`, `C-07`, `C-08`
- **Implementation notes**: keep the operator story above the provider seam and make clear which data remains internal mapping detail
- **Acceptance criteria**: an operator can tell which config file and model mapping surfaces matter before launching the gateway
- **Test notes**: reviewer can map each prerequisite to an existing runtime or config anchor without reading provider code
- **Risk/rollback notes**: if config surfaces drift before execution, mark the slice basis stale rather than guessing

Checklist:
- Implement: define the prerequisite and config sections of `C-09`
- Test: compare against README and config examples
- Validate: confirm the boundary language stays aligned with `C-05`
- Cleanup: remove any contradictory bootstrap phrasing from the plan if discovered

#### S1.T2 - Freeze Claude Code Attachment And Evidence-Hook Rules

- **Outcome**: `C-09` names the canonical `ANTHROPIC_BASE_URL` and placeholder API key posture plus statusline and tracing expectations.
- **Inputs/outputs**: inputs from `gateway/README.md`, `gateway/src/main.rs`, `gateway/src/server/mod.rs`, and `gateway/src/cli/mod.rs`; output is the attachment and evidence-hook section of `C-09`
- **Thread/contract refs**: `THR-08`, `C-09`, `C-08`, `C-05`
- **Implementation notes**: distinguish required evidence hooks from optional convenience and keep redaction posture explicit
- **Acceptance criteria**: an operator can tell what to enable before `SEAM-2` smoke and what artifacts later count as proof
- **Test notes**: reviewer can point to the actual runtime anchors for `last_routing.json`, `trace.jsonl`, and `install-statusline`
- **Risk/rollback notes**: if evidence hooks prove insufficient during implementation, open a seam-owned remediation instead of widening public identity

Checklist:
- Implement: define the Claude Code attachment and evidence-hook sections of `C-09`
- Test: compare against current CLI and server behavior
- Validate: confirm the evidence posture stays redaction-safe
- Cleanup: remove any bootstrap note that implies statusline or tracing are self-evident

#### S1.T3 - Freeze The Contract Artifact Path And Verification Checklist

- **Outcome**: the seam has one canonical `C-09` artifact path and a reviewer checklist that can gate `exec-ready`.
- **Inputs/outputs**: inputs from threading, review questions, and prior sections; output is the artifact-location and verification section of `C-09`
- **Thread/contract refs**: `THR-08`, `C-09`
- **Implementation notes**: the checklist should confirm sequence, evidence hooks, and boundary language, not post-exec publication
- **Acceptance criteria**: `SEAM-1` can later pass the contract gate without waiting for closeout publication work
- **Test notes**: reviewer can answer the checklist from the contract alone
- **Risk/rollback notes**: do not expand the checklist into smoke execution or troubleshooting tasks that belong to later seams

Checklist:
- Implement: freeze the artifact path and verification checklist
- Test: dry-run the checklist against the planned contract sections
- Validate: ensure all open questions are resolved into explicit contract statements
- Cleanup: remove duplicate verification bullets from later slices if they belong here
