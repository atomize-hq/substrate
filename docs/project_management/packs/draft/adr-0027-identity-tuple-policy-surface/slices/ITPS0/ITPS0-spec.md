# ITPS0-spec — contract and schema lock for tuple-axis policy

## Behavior delta (single)
- Existing: ADR-0043, the pre-planning pack, and the implemented ADR-0027 contract all describe the tuple-axis surface, but they do not yet lock one authoritative explain surface, one tuple-vocabulary publication rule, and one schema grammar for `llm.constraints.*`.
- New: `ITPS0` publishes the canonical contract and schema lock for `llm.constraints.routers`, `llm.constraints.providers`, `llm.constraints.protocols`, and `llm.constraints.auth_authorities`, including the authoritative `substrate policy current show --explain` surface, the exact exit-code mapping, and the exact token grammar.
- Why: downstream slices need one stable contract before they define policy ordering, telemetry inventory, compatibility text, and validation packaging.

## Scope
- Publish the contract lock in `contract.md`.
- Publish the schema lock in `tuple-policy-schema-spec.md`.
- Close `DR-ITPS-01` and `DR-ITPS-02` inside the slice-owned outputs.

Likely touch surfaces (authoritative for this slice):
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tuple-policy-schema-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS0/ITPS0-spec.md`

## Behavior (authoritative)

### Inputs and gate role
- `ITPS0` consumes:
  - `pre-planning/alignment_report.md`
  - `pre-planning/workstream_triage.md`
  - `pre-planning/impact_map.md`
  - `pre-planning/spec_manifest.md`
  - `pre-planning/minimal_spec_draft.md`
  - ADR-0042
  - ADR-0043
  - the implemented ADR-0027 contract and schema pack
- `ITPS0` is the first gate for the feature pack.
- `ITPS1`, `ITPS2`, and `ITPS3` inherit `ITPS0` outputs verbatim for contract, schema, and tuple-vocabulary decisions.

### Decision lock
- `DR-ITPS-01`: tuple-policy publication reuses `identity_tuple` and `placement_posture`.
- `DR-ITPS-02`: `substrate policy current show --explain` is the authoritative merged inspection command for `llm.constraints.*`.

### Contract outputs
`contract.md` must define:
- the authoritative merged inspection command
- the `stdout` and `stderr` contract for `--explain`
- the exact tuple-axis deny wording family
- the feature-local exit-code mapping
- the Linux/macOS/Windows parity statement

### Schema outputs
`tuple-policy-schema-spec.md` must define:
- the four owned key paths under `llm.constraints.*`
- per-key type and effective default
- replace semantics across workspace and global policy patches
- exact snake_case and dotted-id grammar
- accepted and rejected YAML shapes
- the explicit omission of `client` as a standalone policy key in v1

## Acceptance criteria
- AC-ITPS0-01: `contract.md` states that `substrate policy current show --explain` is the authoritative merged view for `llm.constraints.*`.
- AC-ITPS0-02: `contract.md` states that tuple-policy publication reuses `identity_tuple` and `placement_posture`.
- AC-ITPS0-03: `contract.md` maps tuple-policy schema invalidity to exit code `2` and tuple-axis mismatch denial to exit code `5`.
- AC-ITPS0-04: `contract.md` records the exact mismatch-detail patterns for router, provider, protocol, and auth-authority denial.
- AC-ITPS0-05: `tuple-policy-schema-spec.md` defines `llm.constraints.routers`, `llm.constraints.providers`, `llm.constraints.protocols`, and `llm.constraints.auth_authorities`.
- AC-ITPS0-06: `tuple-policy-schema-spec.md` does not introduce `llm.constraints.clients`.
- AC-ITPS0-07: `tuple-policy-schema-spec.md` defines `[]` as unconstrained and workspace-over-global replace semantics for every owned key.
- AC-ITPS0-08: `tuple-policy-schema-spec.md` defines token grammar that matches the current snake_case and dotted-id validators.

## Out of scope
- ordered runtime evaluation beyond the contract-level deny wording
- telemetry field inventory and redaction tables
- compatibility and rollout text
- manual testing matrix
- `tasks.json`, `plan.md`, and kickoff-prompt wiring
