---
slice_id: S1
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: provisional
  basis_ref: seam.md#basis
  stale_triggers:
    - "Any change to net_allowed canonicalization/validation rules"
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
contracts_produced:
  - C-01
contracts_consumed: []
open_remediations:
  - REM-001
candidate_subslices: []
---
### S1 - Publish Snapshot V3 `net_allowed` contract (C-01)

- **User/system value**: make `net_allowed` an explicit cross-boundary input so the world does not rely on hidden in-guest broker state for allowlists.
- **Scope (in/out)**:
  - In: Snapshot V3 schema field + canonicalization/validation helpers + unit tests.
  - Out: host/world-agent plumbing (covered in `S2`/`S3`).
- **Acceptance criteria**:
  - `PolicySnapshotV3.net_allowed: Vec<String>` exists with `#[serde(default)]`.
  - Canonicalization rules are explicit and test-locked, including:
    - trim + drop empty
    - dedupe
    - collapse any `"*"` presence to exactly `["*"]`
    - reject non-`"*"` wildcard forms (e.g., `"*.example.com"`) when enforcement is requested
  - Hostname normalization posture is explicitly decided (casefolding + IDNA) and resolves `REM-001`.
- **Dependencies**:
  - `../../threading.md` (`C-01`, `THR-01`)
- **Verification**:
  - `cargo test -p agent-api-types` (unit tests for canonicalization/validation)
- **Rollout/safety**:
  - Additive-only: missing `net_allowed` in older snapshots defaults via serde to `[]` (or an explicitly decided default), with no behavior change unless downstream opt-in gating requests isolation.
- **Review surface refs**:
  - `../../review_surfaces.md` (R2/R3)
  - `review.md` (mismatch hotspots: normalization drift, wildcard semantics)

#### S1.T1 - Decide and document hostname normalization rules

- **Outcome**: explicit rules for casefolding + IDNA posture that every consumer uses.
- **Inputs/outputs**:
  - In: `REM-001`, `scope_brief.md` assumptions, current broker semantics (evidence).
  - Out: a single normalization helper used by snapshot builder + world-agent plumbing.
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - Record the exact posture (e.g., lowercasing ASCII, applying IDNA to Unicode hostnames, rejecting invalid labels) and ensure tests cover tricky cases.
- **Acceptance criteria**:
  - `REM-001` can be moved to `resolved` once merged evidence exists.
  - Unit tests cover at least: uppercase hostnames, leading/trailing whitespace, Unicode labels, and `"*"` canonicalization.
- **Test notes**: keep tests in `agent-api-types` so drift is caught early.
- **Risk/rollback notes**: normalization changes are a downstream stale trigger and must be captured in seam closeout.

Checklist:
- Implement: normalization helper + validation rules
- Test: unit tests for canonicalization + normalization cases
- Validate: ensure serde default/back-compat behavior is explicitly asserted
- Cleanup: remove any duplicate ad-hoc normalization code paths

