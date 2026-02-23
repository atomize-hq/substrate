### S3 — Strict-mode onramp plan (gated, opt-in, post-calibration)

- **User/system value**: Provide a safe, explicit path from advisory lift signals to calibrated enforcement in new-format packs, without breaking legacy workflows.
- **Scope (in/out)**:
  - In:
    - Define:
      - what “strict mode” means,
      - how it is gated (pack metadata),
      - which invariants are candidates for enforcement,
      - rollout stages and calibration criteria.
    - Provide an opt-in mechanism (docs + Makefile target) that can fail a run only when strict mode is explicitly selected.
  - Out:
    - Enabling strict mode by default.
    - Adding new semantics to `pm_lift.py` (strict enforcement can be implemented as a wrapper consuming CONTRACT-3).
- **Acceptance criteria**:
  - Strict gating is keyed off:
    - `tasks.json meta.slice_spec_version >= 2` (eligible), and
    - an explicit user opt-in (env var / make target).
  - Candidate invariants are documented with:
    - rationale,
    - how to measure from `CONTRACT-3` fields,
    - false-positive risks and calibration requirements.
- **Dependencies**:
  - Consumes: `CONTRACT-3:pm_lift_emit_json_v1` for invariant checks
  - References: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` for policy constraints
- **Verification**:
  - A strict-mode doc example shows:
    - an eligible pack,
    - a non-eligible legacy pack,
    - the different outcomes (advisory vs strict failure).

#### S3.T1 — Write the strict-mode spec (gating + stages + invariants)

- **Outcome**: A reviewable spec that makes promotion to enforcement an explicit decision.
- **Inputs/outputs**:
  - Inputs:
    - `work_lift_v1_seams/seam-5-advisory-workflow-integration.md` (strict-mode notes)
    - `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (constraints)
  - Outputs:
    - Add a short doc, e.g.:
      - `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_STRICT_MODE.md`
    - Link it from:
      - `PLANNING_WORK_LIFT_ADVISORY.md` (S1)
- **Implementation notes**:
  - Define eligibility:
    - strict gating applies only when `meta.slice_spec_version >= 2` (new format).
  - Define candidate invariants (initial list; do not enable by default):
    - `contract.behavior_deltas == 1` (example; confirm actual field name in lift vector / pm_lift derived),
    - `estimated_slices <= 3` for single ADR candidates (example),
    - “no missing required inputs” once schema/rubric is stable.
  - Define promotion criteria:
    - N calibration runs over real packs,
    - acceptable false positive rate,
    - documented exceptions.
- **Acceptance criteria**:
  - Strict mode is clearly described as opt-in and post-calibration.

Checklist:
- Implement:
  - Write gating logic in words and reference exact metadata key path.
  - List candidate invariants with “not enabled yet” labels.
- Validate:
  - Ensure it does not contradict advisory-first constraints.
- Cleanup:
  - Keep it decision-oriented; avoid implementation detail sprawl.

#### S3.T2 — Add an opt-in strict check wrapper (consumes CONTRACT-3; not default)

- **Outcome**: A mechanism that can be turned on for specific packs when ready.
- **Inputs/outputs**:
  - Inputs:
    - `pm_lift.py ... --emit-json` output (CONTRACT-3)
    - `tasks.json` metadata
  - Outputs:
    - New script, e.g.:
      - `docs/project_management/system/scripts/planning/pm_lift_strict_check.py`
    - New Makefile target, e.g.:
      - `pm-lift-strict` (opt-in; fails only when strict mode is selected and pack is eligible)
- **Implementation notes**:
  - Default behavior:
    - if pack is legacy (`meta.slice_spec_version < 2`): print “not eligible” and exit 0.
    - if pack is eligible but strict opt-in not set: exit 0 (advisory).
    - if strict opt-in set: enforce the invariant set selected by a config file or hard-coded minimal list (documented).
- **Acceptance criteria**:
  - No enforcement happens without explicit opt-in.

Checklist:
- Implement:
  - Parse `tasks.json` meta, run lift, evaluate invariants.
- Validate:
  - Exercise eligible vs non-eligible pack flows.
- Cleanup:
  - Keep invariant evaluation transparent (print which rule failed and which field drove it).

