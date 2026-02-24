# Concrete Remediator Log — Work Lift v1 seams

Generated at: 2026-02-22T21:29:39-05:00

Input report: `concrete-audit.report.json`

Repo commit: `65f8f54ae83a0fcb99a31bf6527dfc6324b9a8c4`

## Triage (bucket)

- Bucket A (local clarification): CA-0001, CA-0012, CA-0013, CA-0014
- Bucket B (code-defined contract evidence): CA-0003, CA-0004, CA-0005, CA-0008, CA-0010
- Bucket C (decision-log-defined standard evidence): CA-0006, CA-0007, CA-0009
- Bucket E (decision required): CA-0011 (strict-mode promotion criteria + strict pack invariant)

Evidence sources used:

- `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (D3/D6/D7/D8/D9)
- `docs/project_management/system/scripts/planning/pm_lift.py` (CLI + scoring + `--emit-json`)
- `docs/project_management/system/scripts/planning/validate_impact_map.py` (Touch Set normalization + `--emit-json`)
- `Makefile` (existing planning-lint conventions)

## Issue closure log

### CA-0001 — Fixed

Requirement: make normativity explicit and define the cutover to published PM system artifacts.

- Evidence:
  - Seam docs already identify contract IDs and published artifact targets (`work_lift_v1_seams/threading.md`).
- Changes:
  - Clarified pre- vs post-implementation source-of-truth semantics and conflict rules in `work_lift_v1_seams/README.md`.
  - Added an explicit “contract artifact map” (cutover targets) in `work_lift_v1_seams/README.md`.

### CA-0002 — Fixed

Requirement: pin CONTRACT-2 JSON shape + precedence + version selection + missing-version behavior.

- Evidence:
  - Canonical v1 scoring math and mapping live in `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` D7/D9.
  - Current tool behavior uses baked-in defaults if config missing (`docs/project_management/system/scripts/planning/pm_lift.py:58-72`).
- Changes:
  - Added a concrete, fully-specified v1 JSON shape and interpretation rules to `work_lift_v1_seams/threaded-seams/seam-2-lift-model-config-v1/slice-1-contract-2-model-config.md`.
  - Strengthened immutability language in `work_lift_v1_seams/seam-2-lift-model-config-v1.md`.

### CA-0003 — Fixed

Requirement: define CONTRACT-3 `--emit-json` machine contract (types/enums/order/error/exit behavior).

- Evidence:
  - Required keys and emission behavior are implemented in `docs/project_management/system/scripts/planning/pm_lift.py` (`_print_result`: around `pm_lift.py:451+`).
  - Confidence + triggers construction is in `docs/project_management/system/scripts/planning/pm_lift.py:191-226`.
- Changes:
  - Added a concrete v1 contract section (types, enums, ordering, exit codes, stable trigger IDs, required derived keys) to `work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-1-contract-3-emit-json.md`.

### CA-0004 — Fixed

Requirement: define CONTRACT-4 base path + normalization + strict/legacy semantics + `dir_prefixes` derivation.

- Evidence:
  - Path normalization + bans are implemented in `docs/project_management/system/scripts/planning/validate_impact_map.py:155-190`.
  - Repo-root-relative interpretation is implemented via git repo root (`validate_impact_map.py:79-91`) and filesystem checks (`validate_impact_map.py:261-279`).
  - `--emit-json` shape and `dir_prefixes` derivation is implemented in `validate_impact_map.py:297-312`.
  - Strict/legacy mode gating is implemented in `validate_impact_map.py:66-76` and `validate_impact_map.py:331-336`.
- Changes:
  - Added a concrete v1 contract section (required keys/types, token rules, exit codes, mode gating) to `work_lift_v1_seams/threaded-seams/seam-4-pack-derived-lift-inputs/slice-1-contract-4-impact-map-emit-json.md`.

### CA-0005 — Fixed

Requirement: pin the `pm_lift.py` CLI surface (subcommands, flags, `--emit-json` composition).

- Evidence:
  - CLI arguments are defined by argparse in `docs/project_management/system/scripts/planning/pm_lift.py:476-510`.
- Changes:
  - Rewrote “how to run” recipes to use the canonical CLI flags (`--intake`, `--feature-dir`, `--git-range`, `--emit-json`) in `work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-1-advisory-workflow-docs-and-make-targets.md`.

### CA-0006 — Fixed

Requirement: enumerate v1 field set + required/null rules locally (not only by reference to D6).

- Evidence:
  - Canonical embedded JSON example exists in `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` D3.
  - Tool treats missing/`null` numeric fields equivalently (`pm_lift.py:_num`: `pm_lift.py:96-105`).
- Changes:
  - Added a complete v1 field inventory (all JSON paths + types + null/required rules + unknown-key policy) to `work_lift_v1_seams/threaded-seams/seam-1-lift-vector-schema-and-rubric/slice-1-contract-1-schema.md`.

### CA-0007 — Fixed

Requirement: make null semantics consistent and per-field concrete.

- Evidence:
  - Decision log states “null treated as 0” behavior (`WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` D7).
  - Tool behavior: missing/`null` numeric values generate missing inputs (`pm_lift.py:96-105`, `pm_lift.py:206-211`).
- Changes:
  - Made “missing vs null equivalence” explicit and tied it to CONTRACT-3 outputs in `work_lift_v1_seams/seam-1-lift-vector-schema-and-rubric.md` and the CONTRACT-1 slice doc.

### CA-0008 — Fixed

Requirement: define missing schema/config behavior for advisory vs strict.

- Evidence:
  - Model config is optional today (`pm_lift.py:58-72`).
  - Schema is not required for basic parsing today (marker+JSON parsing: `pm_lift.py:75-94`).
- Changes:
  - Defined v1 advisory behavior and delegated strict failures to the strict wrapper (SEAM-5 S3) in `work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-2-config-backed-scoring-and-validation.md`.

### CA-0009 — Fixed

Requirement: specify prefix expansion defaults, determinism boundary, and failure-mode contract.

- Evidence:
  - Prefix expansion is implemented for `from-impact-map` via `git ls-files` (`pm_lift.py:249-263`) and used in effective count math (`pm_lift.py:302-317`).
  - Expansion failures are treated as empty expansions (`pm_lift.py:253-262`).
- Changes:
  - Pinned default behavior (enabled by default, repo-root-relative, failure = empty expansion) in `work_lift_v1_seams/seam-4-pack-derived-lift-inputs.md` and aligned the threaded seam doc.

### CA-0010 — Fixed

Requirement: pin `confidence` allowed values and deterministic mapping.

- Evidence:
  - Tool confidence is `high` by default and `low` when missing inputs exist (`pm_lift.py:206-211`).
  - Derived-pack runs force `confidence=low` when `dir_prefixes` exists (`pm_lift.py:363-372`).
- Changes:
  - Pinned `confidence` enum to `high|low` for v1 and defined rules in `work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-1-contract-3-emit-json.md`.

### CA-0011 — Fixed (decision introduced)

Requirement: remove placeholders and pin strict-mode invariant selection + promotion thresholds.

- Evidence:
  - Pack eligibility gating pattern exists in strict validators via `tasks.json.meta.slice_spec_version` (`validate_impact_map.py:66-76`, `validate_impact_map.py:331-336`).
  - No prior authoritative numeric promotion thresholds existed.
- Decisions introduced:
  - `docs/decisions/concrete-remediation-decisions.md` (CRD-0001, CRD-0002)
- Changes:
  - Pinned strict opt-in mechanism, strict invariant set (intake vs pack), exit codes, and promotion thresholds in `work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-3-strict-mode-onramp-plan.md`.

### CA-0012 — Fixed

Requirement: replace wildcard/alternative deliverables with exact file paths.

- Evidence:
  - Planning standards directory and filenames exist today (`docs/project_management/system/standards/planning/*`).
- Changes:
  - Replaced wildcard planning doc touch surfaces with pinned file lists in:
    - `work_lift_v1_seams/seam-5-advisory-workflow-integration.md`
    - `work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/seam.md`
  - Pinned advisory doc outputs in `work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-1-advisory-workflow-docs-and-make-targets.md`.
  - Pinned the SEAM-2 goldens output doc path in `work_lift_v1_seams/threaded-seams/seam-2-lift-model-config-v1/slice-2-goldens-and-conformance.md`.

### CA-0013 — Fixed

Requirement: specify the exact test command used for verification.

- Evidence:
  - Existing planning tests use `unittest` (`docs/project_management/system/scripts/planning/tests/test_check_adr_exec_summary.py:3`).
- Changes:
  - Pinned the command line in `work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-3-goldens-and-conformance.md`.

### CA-0014 — Fixed

Requirement: remove unbounded “roughly” goal phrasing adjacent to execution expectations.

- Evidence:
  - The mapping to slices/checkpoints is defined in `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` D9.
- Changes:
  - Replaced “roughly” with a concrete reference to D8–D9 mapping in `work_lift_v1_seams/scope_brief.md`.
