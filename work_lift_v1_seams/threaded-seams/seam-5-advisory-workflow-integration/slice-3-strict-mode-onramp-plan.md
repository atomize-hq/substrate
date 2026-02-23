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
  - Define the strict opt-in mechanism (pinned):
    - Environment variable: `PM_LIFT_STRICT=1`
    - Makefile target: `pm-lift-strict`
  - Define the invariant set as an explicit, versioned list (v1 strict):
    - The strict checker MUST support two contexts:
      - `--intake <path>` (ADR/intake markdown strict check)
      - `--feature-dir <pack_dir>` (Planning Pack strict check)
    - Intake strict invariants (all MUST pass):
      - `confidence == "high"`
      - `missing_inputs` is empty
      - `vector.contract.behavior_deltas == 1`
      - `estimated_slices <= 3`
    - Pack strict invariants (all MUST pass):
      - Pack eligibility: `tasks.json.meta.slice_spec_version >= 2` (otherwise: print “not eligible” and exit 0)
      - `validate_impact_map.py --emit-json` reports `dir_prefixes == []` (prefix entries are forbidden in strict pack checks)
      - `python3 docs/project_management/system/scripts/planning/pm_lift.py from-impact-map --feature-dir <pack_dir> --emit-json` succeeds and conforms to CONTRACT-3
  - Define promotion criteria for enabling strict checks by default (pinned, measurable):
    - Promotion requires >= 20 calibration runs across >= 10 distinct eligible packs.
    - Acceptable false-positive rate for strict failures is <= 5% across those runs.
    - Any exceptions MUST be documented as an explicit allowlist entry (path + rationale) in the strict-mode standard doc.
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
    - if strict opt-in not set (`PM_LIFT_STRICT!=1`): exit 0 (advisory).
    - if strict opt-in set (`PM_LIFT_STRICT=1`): enforce the v1 strict invariants defined in S3.T1 and fail with exit code `1` if any invariant fails.
  - Exit codes (pinned):
    - `0` = pass (or not eligible, by design)
    - `1` = strict invariant failure
    - `2` = usage/tooling error (missing args, tool execution failure, non-JSON output)
- **Acceptance criteria**:
  - No enforcement happens without explicit opt-in.

Checklist:
- Implement:
  - Parse `tasks.json` meta, run lift, evaluate invariants.
- Validate:
  - Exercise eligible vs non-eligible pack flows.
- Cleanup:
  - Keep invariant evaluation transparent (print which rule failed and which field drove it).
