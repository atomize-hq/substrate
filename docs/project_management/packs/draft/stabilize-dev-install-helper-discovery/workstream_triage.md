# stabilize-dev-install-helper-discovery — workstream triage

## Evidence (inputs)

- Canonical pack artifacts:
  - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/spec_manifest.md`
  - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/impact_map.md`
  - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/minimal_spec_draft.md`
  - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/ci_checkpoint_plan.md`
  - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/tasks.json` (`meta.slice_spec_version=2`, strict pack)
- ADR:
  - `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`

## Work Lift (v1; advisory)

- Discovery-time (ADR): `lift_score=3`, `estimated_slices=1`, `confidence=high`
  - Evidence: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/logs/workstream-triage/pm_lift_intake.txt`
- Pack-derived (impact map): `lift_score=12`, `estimated_slices=1`, `confidence=low` (missing contract/qa/ops/docs/risk inputs)
  - Evidence: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/logs/workstream-triage/pm_lift_pack.txt`
  - Touch evidence (from `pm_lift_pack.json`): edit=4 explicit files; crates_touched=1; create/delete/deprecate=0

## Proposed workstreams (parallelizable for full planning)

### WS1 — Contract + decisions (DRs first)

- Goal: make the user-facing behavior deterministic (staging, overwrite policy, cleanup guard, exit-code classes) before slice specs/tasks lock in wording.
- Owned surfaces:
  - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` (authoritative contract wording)
  - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/decision_register.md` (DR-0001..DR-0003; link each decision into contract)
- Dependencies:
  - Uses `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/impact_map.md` and `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/minimal_spec_draft.md` as constraints.
  - Must land before WS2 finalizes slice specs + `tasks.json` triads.
- Slices/triads to create during full planning:
  - SDIHD0/SDIHD1 specs link to contract sections; tasks reference slice specs (no new slice implied).

### WS2 — Slice specs + plan + task graph (make it executable)

- Goal: produce runnable, OS-scoped acceptance criteria and a complete triad task graph for SDIHD0/SDIHD1.
- Owned surfaces:
  - Slice specs:
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/slices/SDIHD0/SDIHD0-spec.md`
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/slices/SDIHD1/SDIHD1-spec.md`
  - Planning glue:
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/plan.md`
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/tasks.json` (triads + dependencies + checkpoint wiring)
- Dependencies:
  - Requires WS1 (contract + DR selections).
  - Should align with WS4 checkpoint wiring before `tasks.json` is considered “planning complete”.
- Proposed slices/triads (baseline from `minimal_spec_draft.md`):
  - `SDIHD0` triad(s): stage helpers under `$SUBSTRATE_HOME/scripts/substrate/...` + validate helper discovery fallback
  - `SDIHD1` triad(s): ownership-guarded dev-uninstall cleanup + deterministic refusal posture

### WS3 — Implementation seams + high-churn boundaries (pre-implementation review)

- Goal: keep execution work bounded to the 4-file touch set and reduce churn risk (scripts are high-conflict surfaces).
- Owned surfaces (touch set from `impact_map.md`):
  - `scripts/substrate/dev-install-substrate.sh` (staging under `$SUBSTRATE_HOME/scripts/substrate/...`)
  - `scripts/substrate/dev-uninstall-substrate.sh` (cleanup under `$SUBSTRATE_HOME/scripts/substrate/...`)
  - `crates/shell/src/builtins/world_enable/runner/paths.rs` (helper discovery tests + error messaging constraints)
  - `crates/shell/tests/world_enable.rs` (integration coverage for fallback + `cargo clean` scenario)
- Dependencies:
  - Must consume WS1 decisions (copy vs symlink; overwrite policy; ownership-guard algorithm), because they shape both staging and uninstall logic.
- Slices/triads to create during full planning:
  - SDIHD0/SDIHD1 task plans should explicitly keep changes confined to these files unless `impact_map.md` is updated.

### WS4 — CI checkpoint + validation artifacts (prove it safely)

- Goal: define when CI gates run and what manual/smoke evidence is required to merge without regressions.
- Owned surfaces:
  - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/ci_checkpoint_plan.md` (already drafted; wire into tasks)
  - Validation docs required by `spec_manifest.md`:
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/manual_testing_playbook.md`
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/smoke/linux-smoke.sh`
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/smoke/macos-smoke.sh`
- Dependencies:
  - Depends on WS2 slice specs (commands + assertions must align with acceptance criteria).
- CI checkpoint implications (current plan):
  - Single checkpoint `CP1` (`CP1-ci-checkpoint`) covering `SDIHD0`→`SDIHD1`.
  - Compile parity: linux/macos/windows; behavior smoke: linux/macos; CI testing: full.
  - `tasks.json` follow-up once slice tasks exist: set `meta.checkpoint_boundaries=["SDIHD1"]` and add a `CP1-ci-checkpoint` ops task wired per standard.

## Sequencing + gates (hard constraints)

1) Decisions before tasks:
   - DR-0001 (copy vs symlink), DR-0002 (ownership guard), DR-0003 (overwrite policy) must be selected and reflected in `contract.md` before SDIHD0/SDIHD1 tasks are finalized.
2) macOS “enable later” scope:
   - Current minimal spec scope guarantees macOS helper discovery + `substrate world enable --dry-run` resolution only; successful provisioning is out of scope unless touch set expands to stage `${RELEASE_ROOT}/scripts/mac/...`.
   - Full planning must ensure slice acceptance criteria and manual playbook match this scope (or explicitly expand scope + update `impact_map.md`).
3) Checkpoint wiring:
   - Before execution, `tasks.json` must include the `CP1-ci-checkpoint` task + dependencies that enforce the checkpoint boundary after SDIHD1.

## Risks + unknowns (drive follow-ups)

- High-churn seam: `scripts/substrate/dev-install-substrate.sh` is a merge-conflict hotspot; prefer minimal, well-isolated edits + deterministic messaging.
- Overwrite collisions: multi-checkout dev installs targeting the same `$SUBSTRATE_HOME` can compete for `$SUBSTRATE_HOME/scripts/substrate/*`; DR-0003 must define safe overwrite/refusal rules.
- Cleanup safety: dev-uninstall must never delete user-managed scripts; DR-0002 must define a deterministic guard + refusal behavior (exit-code taxonomy + message category).
- Helper failure remediation: `paths.rs` error messaging is production-oriented (“reinstall Substrate”); confirm remediation text is acceptable for dev installs or explicitly scope it in contract/playbook.
- Cross-queue overlap: ADR-0035 touches overlapping script surfaces; sequencing boundary recommended (helper staging/cleanup rules must remain coherent across both).

## Slice skeleton recommendations (required)

Starting point (from `minimal_spec_draft.md`):
- `SDIHD0` — Stage helpers under `$SUBSTRATE_HOME`
- `SDIHD1` — Ownership-guarded dev-uninstall cleanup

Recommendation:
- **No changes** (no `ADD`/`SPLIT`/`MERGE`/`RENAME`) given the current strict touch set and pack-derived lift (edit=4 files; 2-slice seam is stable).

## Evidence links (step completion sentinels)

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/logs/spec-manifest/last_message.md`
- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/logs/impact-map/last_message.md`
- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/logs/min-spec-draft/last_message.md`
- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/logs/CI-checkpoint/last_message.md`

## Follow-ups (for full planning)

- Reconcile ADR feature directory path drift (`.../draft/dev-install-helper-discovery/` vs `.../draft/stabilize-dev-install-helper-discovery/`) so links/tooling stay coherent.
- In `plan.md`, explicitly map ADR slice labels (`C0`, `C1`) to canonical slice IDs (`SDIHD0`, `SDIHD1`).
- Add/update `docs/COMMANDS.md` if `substrate world enable` flag docs drift (`--home` vs legacy `--prefix`) is still present.
