# best-effort-distro-package-manager — workstream triage

Goal: propose parallelizable workstreams + sequencing gates for full planning/execution.

## Evidence (inputs + completion sentinels)

Canonical artifacts relied on:
- `docs/project_management/packs/draft/best-effort-distro-package-manager/spec_manifest.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/impact_map.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/minimal_spec_draft.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json` (`meta.slice_spec_version=2` strict pack; `meta.cross_platform=true`)

Stable sentinels:
- `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/spec-manifest/last_message.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/impact-map/last_message.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/min-spec-draft/last_message.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/CI-checkpoint/last_message.md`

Work Lift evidence (advisory):
- Pack-derived: `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/workstream-triage/pm_lift_pack.{txt,json}`
- Intake/ADR-derived: `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/workstream-triage/pm_lift_intake.{txt,json}` (ADR-0031)

## Work Lift summary (advisory; prioritize triggers + confidence)

- Intake/ADR lift: `lift_score=16`, `estimated_slices=2`, `confidence=high` (no split triggers; `risk.security_sensitive=true`)
- Pack lift (strict; impact-map touch set): `lift_score=5`, `estimated_slices=1`, `confidence=low` (missing-input flags)
  - Derived touch set counts: `create=1`, `edit=1` (no deletes/deprecations)

Takeaway:
- No split recommended; keep one pack.
- Treat as **contract-sensitive** despite the small touch set: user-facing installer semantics (exact one-liner, deterministic selection, exit-code scoping).

## Proposed workstreams

### BEDPM-PWS-contract — Operator contract + decision register (hard gate)

Goal:
- Lock the operator-visible contract for Linux installer pkg-manager selection:
  - override surfaces (`--pkg-manager`, `PKG_MANAGER`),
  - total precedence pipeline + deterministic PATH ambiguity handling,
  - stderr decision one-liner (exact text; exactly-once; timing),
  - exit-code meanings (`0|2|3|4`) and whether meanings are Linux-only vs global.

Owns (surfaces / files to create during full planning; per `spec_manifest.md`):
- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md` (DR-0001..DR-0003)
- `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md` (must encode gates + validation commands)
- `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`

Depends on:
- None (this is the upstream gate for the rest of the pack)

Full-planning slices/triads to create:
- Doc-first gate: resolve DR-0001 (`/etc/os-release` parsing), DR-0002 (PATH ambiguity precedence + warning content), DR-0003 (hermetic test seam for fake os-release input).
- Cross-pack boundary pinning: explicitly name detection/selection outputs that downstream packs (notably `persist-detected-linux-distro-pkg-manager`) will persist, and forbid persistence in this pack.

### BEDPM-PWS-installer — Implement detection + selection logic (BEDPM0 + BEDPM1)

Goal:
- Implement the installer behavior changes in `scripts/substrate/install-substrate.sh` while preserving “no behavior change on macOS/Windows”.

Owns (surfaces / touch set from `impact_map.md`):
- `scripts/substrate/install-substrate.sh`
- Verify-only: `scripts/substrate/world-enable.sh` call paths that source `install-substrate.sh` (avoid violating “exactly once” one-liner invariant via double invocation).

Depends on:
- BEDPM-PWS-contract (DRs + contract pinning must land first to avoid churn)

Full-planning slices/triads to create:
- `BEDPM0` triad: best-effort `/etc/os-release` read + safe parsing + required stderr decision one-liner emitted exactly once.
- `BEDPM1` triad: override precedence + mapping + PATH probe fallback + deterministic multi-manager warning + exit code mapping for override/selection failure modes.

### BEDPM-PWS-tests_ci — Hermetic tests + checkpoint wiring (BEDPM2 + CP1)

Goal:
- Add a hermetic test harness that makes precedence + one-liner + exit codes deterministic in CI, then wire the single checkpoint (CP1).

Owns (surfaces / touch set from `impact_map.md` + `ci_checkpoint_plan.md`):
- `tests/installers/pkg_manager_detection_test.sh`
- Planning wiring: `tasks.json` slice `*-integ` tasks + `CP1-ci-checkpoint` task wiring (once slices exist)

Depends on:
- BEDPM-PWS-contract (one-liner exact text, `source` enum, and DR-0003 test seam)
- BEDPM-PWS-installer (tests can’t pass until behavior exists; harness/spec work can proceed in parallel once contract is pinned)

Full-planning slices/triads to create:
- `BEDPM2` triad: hermetic harness (stubbed `PATH` + fake os-release) asserting precedence, one-liner content, and exit codes with no host mutation.
- CP1 execution: per `ci_checkpoint_plan.md` gates (compile parity `true`, feature smoke `false`, CI testing `"quick"`) after `BEDPM2`.

## Sequencing + gates (hard constraints)

1) **BEDPM-PWS-contract gate:** DR-0001..DR-0003 + `contract.md` must land before slice specs are treated as stable (prevents churn on precedence, one-liner timing, exit-code scoping).
2) Execute `BEDPM0` + `BEDPM1` (installer behavior), then `BEDPM2` (hermetic tests).
3) Run CP1 after `BEDPM2` (compile parity + quick CI; no feature-smoke expected unless the pack adds `FEATURE_DIR/smoke/` later).

## Risks + unknowns (to resolve during full planning)

- `/etc/os-release` parsing corner cases (quotes/whitespace, duplicate keys, comments, case-sensitivity) must be deterministic (DR-0001).
- PATH ambiguity: fixed precedence order + warning content elements must be pinned and testable (DR-0002).
- Mapping-match but binary-missing behavior (outside Fedora `dnf`→`yum` fallback) must be explicit (fallback vs fail; exit-code mapping).
- “Exactly once” decision one-liner timing must consider:
  - “no prereqs needed” paths, and
  - scripts sourcing `install-substrate.sh` (e.g., `world-enable.sh`) without double-printing.
- Exit-code meanings must be explicitly scoped (Linux-only vs global) to avoid accidental macOS/Windows behavior changes.
- Cross-pack seam: `persist-detected-linux-distro-pkg-manager` depends on this pack’s detection/selection outputs; vocabulary drift risk is high if `contract.md` is vague.

## Slice skeleton recommendations (pre-planning; required)

Starting point:
- `docs/project_management/packs/draft/best-effort-distro-package-manager/minimal_spec_draft.md` defines draft slices `BEDPM0..BEDPM2`.

Recommended changes:
- None (keep 3 slices; seams are clean: detect/report vs select/precedence vs hermetic tests).

## Follow-ups

- Populate `tasks.json` with slice `*-integ` tasks + CP1 wiring, then validate mechanically:
  - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/best-effort-distro-package-manager"`
- Re-evaluate `tasks.json` `meta.behavior_platforms_required` (currently includes `linux|macos|windows`) against the “Linux-only behavior change” contract.
- Align slice naming across artifacts (spec-manifest references `slices/C0/C1/C2` while minimal spec draft uses `BEDPM0/1/2`): pick one scheme during full planning and update non-authoritative references.
