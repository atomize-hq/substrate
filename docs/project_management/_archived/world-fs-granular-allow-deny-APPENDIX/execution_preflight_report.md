# Execution Preflight Report — world-fs-granular-allow-deny-appendix

This report is updated during execution triads.

## Metadata
- Date (UTC): `2026-02-07T01:12:32Z`
- Feature dir: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX`
- Orchestration branch: `feat/world-fs-granular-allow-deny-appendix`
- Git:
  - Branch: `feat/world-fs-granular-allow-deny-appendix`
  - HEAD: `304fbad62411edf9661cd3ca46c575304c08d201` (`304fbad6`)

## Recommendation
RECOMMENDATION: ACCEPT

## Notes
- The local branch `feat/world-fs-granular-allow-deny-appendix` was created from `origin/feat/world-fs-granular-allow-deny` at `304fbad6` because no `origin/feat/world-fs-granular-allow-deny-appendix` ref was present at preflight time.

## Evidence
- Planning lint: `make planning-lint FEATURE_DIR="docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX"`
- Planning validate: `make planning-validate FEATURE_DIR="docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX"`

### `make planning-lint`
Command:
- `make planning-lint FEATURE_DIR="docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX"`

Observed output:
```text
scripts/planning/lint.sh --feature-dir "docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX"
== Planning lint: docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX ==
-- Smoke script scaffold scan
-- Hard-ban scan
-- Ambiguity scan
-- JSON validity
-- tasks.json invariants
OK: tasks.json validation passed: docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/tasks.json
-- spec_manifest.md required-doc existence
-- ci_checkpoint_plan.md invariants
-- ADR Executive Summary drift (if ADRs found/referenced)
OK: docs/project_management/next/ADR-0006-env-var-taxonomy-and-override-split.md executive summary hash matches
OK: docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md executive summary hash matches
-- Kickoff prompt sentinel
-- Manual playbook smoke linkage (if present)
-- Sequencing alignment
-- Sequencing spine validity (completed sprints)
OK: completed sprint paths resolve
OK: planning lint passed
exit:0
```

### `make planning-validate`
Command:
- `make planning-validate FEATURE_DIR="docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX"`

Observed output:
```text
python3 scripts/planning/validate_tasks_json.py --feature-dir "docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX"
OK: tasks.json validation passed: docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/tasks.json
exit:0
```

### `bash -n` smoke scaffold
Command:
- `bash -n docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/smoke/linux-smoke.sh`

Observed output:
```text
exit:0
```
