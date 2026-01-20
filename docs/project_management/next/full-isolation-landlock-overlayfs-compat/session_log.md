# full-isolation-landlock-overlayfs-compat — session log

## START — 2026-01-20T01:52:53Z — planning — planning pack for ADR-0015 (full isolation Landlock ↔ overlayfs backing dirs)
- Feature: `docs/project_management/next/full-isolation-landlock-overlayfs-compat/`
- Branch: `testing`
- Goal: Produce an execution-ready Planning Pack for ADR-0015 under the correct feature directory, with validated tasks/prompts/smoke and zero ambiguity in specs/contracts.
- Inputs to read end-to-end:
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
  - `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - `docs/project_management/standards/PLANNING_SESSION_LOG_TEMPLATE.md`
  - `docs/project_management/next/sequencing.json`
  - `docs/project_management/next/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`
- Commands planned (if any):
  - `make planning-new-feature FEATURE=full-isolation-landlock-overlayfs-compat DECISION_HEAVY=1 CROSS_PLATFORM=1 AUTOMATION=1 BEHAVIOR_PLATFORMS=linux CI_PARITY_PLATFORMS=linux,macos,windows`
  - `python3 scripts/planning/validate_tasks_json.py --feature-dir "docs/project_management/next/full-isolation-landlock-overlayfs-compat"`
  - `make planning-validate FEATURE_DIR="docs/project_management/next/full-isolation-landlock-overlayfs-compat"`
  - `make planning-lint FEATURE_DIR="docs/project_management/next/full-isolation-landlock-overlayfs-compat"`

## END — 2026-01-20T02:12:08Z — planning — planning pack for ADR-0015 (full isolation Landlock ↔ overlayfs backing dirs)
- Summary of changes (exhaustive):
  - Created an ADR-0015 Planning Pack in `docs/project_management/next/full-isolation-landlock-overlayfs-compat/` with a single slice `C0` scoped to overlayfs backing dir derivation and Landlock allowlist extension.
  - Expanded the decision register to cover helper placement, allowlist scope, failure mode, and platform scope as explicit A/B decisions (DR-0002..DR-0005).
  - Wrote an authoritative C0 spec with deterministic runtime derivation, fail-closed rules, and validation requirements.
  - Added Linux smoke that reproduces the pre-fix failure mode (allowlisted write) and validates the fix; added macOS/Windows no-op smoke scripts as defined by contract.
  - Updated `docs/project_management/next/sequencing.json` to include this feature directory and its C0 spec.
- Files created/modified:
  - `docs/project_management/next/full-isolation-landlock-overlayfs-compat/plan.md`
  - `docs/project_management/next/full-isolation-landlock-overlayfs-compat/tasks.json`
  - `docs/project_management/next/full-isolation-landlock-overlayfs-compat/session_log.md`
  - `docs/project_management/next/full-isolation-landlock-overlayfs-compat/C0-spec.md`
  - `docs/project_management/next/full-isolation-landlock-overlayfs-compat/decision_register.md`
  - `docs/project_management/next/full-isolation-landlock-overlayfs-compat/integration_map.md`
  - `docs/project_management/next/full-isolation-landlock-overlayfs-compat/manual_testing_playbook.md`
  - `docs/project_management/next/full-isolation-landlock-overlayfs-compat/smoke/linux-smoke.sh`
  - `docs/project_management/next/full-isolation-landlock-overlayfs-compat/smoke/macos-smoke.sh`
  - `docs/project_management/next/full-isolation-landlock-overlayfs-compat/smoke/windows-smoke.ps1`
  - `docs/project_management/next/full-isolation-landlock-overlayfs-compat/execution_preflight_report.md`
  - `docs/project_management/next/full-isolation-landlock-overlayfs-compat/C0-closeout_report.md`
  - `docs/project_management/next/full-isolation-landlock-overlayfs-compat/kickoff_prompts/*`
  - `docs/project_management/next/sequencing.json`
- Rubric checks run (with results):
  - `python3 scripts/planning/validate_tasks_json.py --feature-dir "docs/project_management/next/full-isolation-landlock-overlayfs-compat"` → `0` → `OK`
  - `make planning-validate FEATURE_DIR="docs/project_management/next/full-isolation-landlock-overlayfs-compat"` → `0` → `OK`
  - `make planning-lint FEATURE_DIR="docs/project_management/next/full-isolation-landlock-overlayfs-compat"` → `0` → `OK`
- Sequencing alignment:
  - `sequencing.json` reviewed: `YES`
  - Changes required: `ADDED sprint entry for full_isolation_landlock_overlayfs_compat`
- Blockers:
  - `NONE`
- Next steps:
  - Run the Planning Quality Gate review and produce `docs/project_management/next/full-isolation-landlock-overlayfs-compat/quality_gate_report.md` before starting execution triads.

## START — 2026-01-20T02:55:34Z — planning — quality gate remediation (DR viability + macOS behavior validation)
- Goal:
  - Remediate decision register viability findings (DR-0004, DR-0005) and align the Planning Pack to include macOS as a behavior platform (Lima guest path), while keeping Windows CI parity-only.
- Commands planned:
  - `make planning-lint FEATURE_DIR="docs/project_management/next/full-isolation-landlock-overlayfs-compat"`
  - `make planning-validate FEATURE_DIR="docs/project_management/next/full-isolation-landlock-overlayfs-compat"`

## END — 2026-01-20T02:55:34Z — planning — quality gate remediation (DR viability + macOS behavior validation)
- Summary of changes (exhaustive):
  - Updated DR-0004 Option B to a viable degrade alternative; kept the selected fail-closed posture unchanged.
  - Updated DR-0005 to select Linux+macOS behavior platforms (macOS validates via Lima guest) while keeping Windows CI parity-only.
  - Updated `tasks.json` meta and integration tasks to reflect macOS behavioral smoke.
  - Replaced macOS smoke no-op with a real smoke script mirroring Linux validation.
  - Updated manual testing playbook and execution preflight report to match the new behavior platform set.
- Rubric checks run (with results):
  - `make planning-validate FEATURE_DIR="docs/project_management/next/full-isolation-landlock-overlayfs-compat"` → `0` → `OK`
  - `make planning-lint FEATURE_DIR="docs/project_management/next/full-isolation-landlock-overlayfs-compat"` → `0` → `OK`

## START — 2026-01-20T03:07:08Z — ops — F0-exec-preflight (execution preflight gate)
- Feature: `docs/project_management/next/full-isolation-landlock-overlayfs-compat/`
- Branch: `feat/full-isolation-landlock-overlayfs-compat`
- Goal: Run the execution preflight gate before starting any triad work for C0.
- Inputs reviewed end-to-end:
  - Standard: `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`
  - ADR: `docs/project_management/next/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`
  - Planning Pack: `plan.md`, `tasks.json`, `session_log.md`, `C0-spec.md`, `decision_register.md`, `integration_map.md`, `manual_testing_playbook.md`, `smoke/*`
  - CI dispatch plumbing: `Makefile`, `scripts/ci/dispatch_ci_testing.sh`, `scripts/ci/dispatch_feature_smoke.sh`, `.github/workflows/ci-testing.yml`, `.github/workflows/feature-smoke.yml`, `.github/workflows/ci-compile-parity.yml`
