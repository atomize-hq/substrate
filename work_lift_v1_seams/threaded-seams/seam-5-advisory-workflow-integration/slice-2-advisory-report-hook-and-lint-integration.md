### S2 — Optional advisory report hook (non-fatal) + lint integration

- **User/system value**: Surface lift signals where planners already look (lint/check output), without blocking and without turning lift into “the only number”.
- **Scope (in/out)**:
  - In:
    - Add an opt-in report command that:
      - runs `pm_lift ... --emit-json`,
      - prints a short, stable human summary (score, estimated_slices, top triggers, missing inputs, confidence).
    - Optionally hook this report into existing planning lint scripts as a **non-fatal** advisory section when enabled.
  - Out:
    - Any change to `pm_lift.py` output semantics (owned by SEAM-3 / CONTRACT-3).
    - Mandatory execution during `lint` by default.
- **Acceptance criteria**:
  - The report:
    - is stable across runs given the same inputs,
    - is resilient to additive keys in CONTRACT-3,
    - never blocks by default (exit code 0 unless the underlying `pm_lift` invocation fails).
  - Lint integration:
    - gated behind an explicit flag/env var: set `PM_LIFT_ADVISORY=1`
    - prints advisories but does not fail the lint run.
- **Dependencies**:
  - Consumes: `CONTRACT-3:pm_lift_emit_json_v1`
  - Blocked-by: SEAM-3 S1 (publish/lock CONTRACT-3) for a stable field set and error semantics
- **Verification**:
  - A fixture intake file and a fixture Planning Pack produce a consistent summary.
  - Lint output includes the advisory block when enabled and omits it when disabled.

#### S2.T1 — Define the “advisory report” UX contract (summary fields + wording)

- **Outcome**: A crisp UX spec that prevents the report from drifting into bespoke, unreviewable heuristics.
- **Inputs/outputs**:
  - Inputs:
    - `work_lift_v1_seams/threading.md` (CONTRACT-3 field list)
    - `work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-1-contract-3-emit-json.md` (exact semantics)
  - Outputs:
    - A short section in `PLANNING_WORK_LIFT_ADVISORY.md` describing:
      - which JSON fields are displayed,
      - how triggers/missing inputs are prioritized (top N),
      - phrasing rules (avoid prescriptive “must split” language unless strict mode).
- **Acceptance criteria**:
  - A reviewer can assess report changes against the UX spec.

Checklist:
- Implement:
  - Specify top-level summary lines and their mapping to CONTRACT-3 fields.
- Validate:
  - Ensure it emphasizes triggers + confidence over score alone.
- Cleanup:
  - Keep it tight; no extra heuristics beyond what `pm_lift` already emits.

#### S2.T2 — Add a wrapper script that prints the advisory report (consumes `--emit-json`)

- **Outcome**: A small, stable wrapper so Makefile/lint can share the same output.
- **Inputs/outputs**:
  - Inputs:
    - `docs/project_management/system/scripts/planning/pm_lift.py`
  - Outputs (new file; avoid editing `pm_lift.py` here):
    - `docs/project_management/system/scripts/planning/pm_lift_report.py`
- **Implementation notes**:
  - Invoke `pm_lift.py ... --emit-json` as a subprocess and parse JSON.
  - Only depend on stable CONTRACT-3 fields; treat unknown keys as ignorable.
  - Exit codes:
    - propagate non-zero only if `pm_lift.py` fails to run or returns non-JSON.
    - otherwise exit 0 (advisory).
- **Acceptance criteria**:
  - Report matches the UX contract from S2.T1.

Checklist:
- Implement:
  - Parse JSON; print summary; keep output short.
- Validate:
  - Run on intake and pack paths.
- Cleanup:
  - Ensure stdout is the report; errors go to stderr.

#### S2.T3 — Add opt-in lint integration (non-fatal)

- **Outcome**: Lift advisories appear in the lint flow only when explicitly enabled.
- **Inputs/outputs**:
  - Inputs:
    - `docs/project_management/system/scripts/planning/lint.sh`
    - `docs/project_management/system/scripts/planning/lint.ps1`
  - Outputs:
    - Optional block that calls `pm_lift_report.py` when:
      - `PM_LIFT_ADVISORY=1` (or equivalent) is set, and
      - a target pack/intake path is provided or inferable.
- **Acceptance criteria**:
  - With flag off: lint output unchanged.
  - With flag on: advisory block appears and does not cause lint failure on “high lift”; only tool errors fail.

Checklist:
- Implement:
  - Add flag gating and clear “advisory only” labeling in output.
- Validate:
  - Run lint with and without the flag.
- Cleanup:
  - Keep integration optional and easy to disable.
