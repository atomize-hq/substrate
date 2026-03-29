# Remediation Log - Best-Effort Distro Package Manager

## Open remediations

```yaml
remediation_id: REM-001
origin_phase: post_exec
source_gate: closeout
related_seam: SEAM-07
related_slice: S5
related_thread: THR-09
related_contract: C-11
related_artifact: docs/project_management/packs/draft/best-effort-distro-package-manager-fse/governance/seam-07-closeout.md
severity: blocking
status: open
owner_seam: SEAM-07
blocked_targets:
  - seam: SEAM-07
    field: status
    value: closed
summary: SEAM-07 checkpoint evidence is recorded, but quick CI run 23711510594 failed on Linux shell lint before downstream readiness could be published as a clean checkpoint-backed handoff
required_fix: resolve or explicitly disposition the ShellCheck SC2221/SC2222 failure in scripts/substrate/install-substrate.sh, then rerun the SEAM-07 checkpoint decision without widening this seam into unrelated upstream implementation work
resolution_evidence: []
```

Canonical entry shape for future additions:

```yaml
remediation_id: REM-001
origin_phase: pre_exec
source_gate: review
related_seam: SEAM-01
related_slice: null
related_thread: THR-01
related_contract: C-01
related_artifact: docs/project_management/packs/draft/best-effort-distro-package-manager-fse/seam-01-os-release-input-parser.md
severity: blocking
status: open
owner_seam: SEAM-01
blocked_targets:
  - seam: SEAM-01
    field: status
    value: exec-ready
summary: parser-input review found unresolved ambiguity in alternate-input absence semantics
required_fix: make the selected-input fallback and invalid-path posture concrete in the seam-local review artifact without widening the hook contract
resolution_evidence: []
```

Rules:

- use seam ownership only
- use `blocked_targets: []` for non-blocking entries
- use canonical blocked target values only: `proposed`, `decomposed`, `exec-ready`, `in-flight`, `landed`, `closed`, `active`, `next`, `future`

## Resolved remediations

None yet.
