**PRE‑PLANNING ONLY — this document is an alignment draft and MUST be deleted or retired during full planning.**

# stabilize-dev-install-helper-discovery — minimal spec draft

Authority inputs (normative):
- ADR: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
- Spec manifest: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/spec_manifest.md`
- Impact map: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/impact_map.md`

## Scope + authority

This draft defines pack-level cross-cutting defaults, precedence, and invariants that all downstream specs/plans/tasks for this pack must align on.

This draft does not define:
- Slice-specific acceptance criteria, test plans, or task graphs.
- Detailed CLI text, schemas, or implementation details.
- New scope beyond what is enumerated by the ADR + `spec_manifest.md` + `impact_map.md`.

User-facing contract authority:
- Single source of truth for user-facing contract wording: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` (per `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`).

## Defaults + precedence

Helper discovery precedence (in-scope user contract):
- `substrate world enable` resolves the helper in this exact order:
  1) `<inferred version dir>/scripts/substrate/world-enable.sh`
  2) `$SUBSTRATE_HOME/scripts/substrate/world-enable.sh`

Stable helper staging targets (in-scope user contract):
- `scripts/substrate/dev-install-substrate.sh` stages these helpers under `$SUBSTRATE_HOME/scripts/substrate/…`:
  - `world-enable.sh`
  - `install-substrate.sh`

Config posture (explicit non-scope):
- This feature introduces zero config-file, config-key, config-schema, or config-precedence changes.
- `contract.md` MUST explicitly state “no config changes” to prevent silent scope creep.

UNRESOLVED precedence (must be decided in `contract.md`):
- Precedence order among:
  - `scripts/substrate/dev-install-substrate.sh --prefix` / `scripts/substrate/dev-uninstall-substrate.sh --prefix`
  - environment variable `SUBSTRATE_HOME`
  - default value when `SUBSTRATE_HOME` is unset

## Failure posture + invariants

Fail-closed invariants:
- If the helper script is absent from all candidate locations, `substrate world enable` errors (no best-effort provisioning).

Dev-install invariants:
- `$SUBSTRATE_HOME/bin/substrate -> <repo>/target/<profile>/substrate` remains unchanged by this feature.
- `<repo>/target/scripts/substrate/*` is allowed to be absent after `cargo clean`; the `$SUBSTRATE_HOME/scripts/substrate/*` fallback remains required post dev-install.

Dev-uninstall protected-path invariants:
- `scripts/substrate/dev-uninstall-substrate.sh` deletes only files that are deterministically “managed by dev-install”.
- Dev-uninstall never deletes user-managed files under `$SUBSTRATE_HOME/scripts/substrate/…`.

Platform invariants:
- Linux + macOS: supported.
- Windows: “unsupported/no-change” for `substrate world enable` behavior in this feature.

macOS scope invariant (from impact map resolution):
- This feature guarantees helper discovery and `substrate world enable --dry-run` plan resolution on macOS dev installs.
- Successful macOS provisioning after dev-install is out of scope unless the pack explicitly adds staging for `${RELEASE_ROOT}/scripts/mac/…` (follow-up scope).

Security + redaction posture (high level):
- Ownership-guarded cleanup is a security invariant; dev-uninstall MUST NOT use recursive deletion of `$SUBSTRATE_HOME/scripts/…` as a cleanup strategy.
- This feature introduces no new telemetry/log schema surfaces; existing redaction posture remains unchanged.

## Exit-code posture

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`.
- New exit codes required for this pack: none.
- `contract.md` MUST map each feature-relevant failure mode to taxonomy exit code categories (no overrides unless explicitly declared).

## Cross-cutting seams / constraints

Touch set constraint:
- The pack’s touch set is limited to:
  - `scripts/substrate/dev-install-substrate.sh`
  - `scripts/substrate/dev-uninstall-substrate.sh`
  - `crates/shell/src/builtins/world_enable/runner/paths.rs`
  - `crates/shell/tests/world_enable.rs`
- Any expansion requires updating `impact_map.md`.

Non-goals (pack invariants):
- Production install layout under `$SUBSTRATE_HOME/versions/...` remains unchanged.
- `substrate world enable` version-directory inference logic remains unchanged.
- This pack does not introduce “bundle parity” dev installs under `$SUBSTRATE_HOME/versions/<label>/...`.

Decision Register constraints (must be resolved before execution-quality gate):
- DR-0001: helper staging mechanism decision (copy vs symlink).
- DR-0002: uninstall ownership guard decision (exact algorithm).
- DR-0003: overwrite policy when `$SUBSTRATE_HOME/scripts/substrate/*` already exists (including when destination is not dev-managed).

Cross-queue alignment constraints:
- This pack owns helper staging under `$SUBSTRATE_HOME/scripts/substrate/…`.
- ADR-0035 (`ADR-0035-summoning-wombat.md`) owns `world-agent` artifact staging and missing-artifact remediation; both workstreams must align on overwrite and ownership-guard decisions.
- Align with ADR-0003 (`ADR-0003-policy-and-config-mental-model-simplification.md`) by not introducing new env/config precedence behavior and by not reintroducing removed legacy knobs (`SUBSTRATE_PREFIX`, `--prefix` on `substrate world enable`).

## Follow-ups for full planning

- Reconcile feature directory path drift in ADR-0034 (`.../draft/dev-install-helper-discovery/` vs `.../draft/stabilize-dev-install-helper-discovery/`).
- Add explicit mapping in `plan.md` from ADR slice labels (`C0`, `C1`) to canonical slice IDs (`SDIHD0`, `SDIHD1`).
- Decide DR-0001..DR-0003 in `decision_register.md` (two options each; one selected option each).
- Define `SUBSTRATE_HOME` / `--prefix` precedence in `contract.md` and ensure scripts + playbook use the same rule.
- Reconcile Windows platform evidence requirements in `tasks.json` with “world enable unsupported on Windows” (define deterministic “expected unsupported” evidence, or mark Windows validation as N/A for this feature).
- Decide whether dev-install staging scope on macOS includes `${RELEASE_ROOT}/scripts/mac/…`; if not, explicitly scope macOS behavior to helper discovery + `--dry-run` only in `contract.md` and the manual playbook.
- Validate `paths.rs` remediation messaging for dev installs when helper discovery fails in all locations; update `contract.md` + manual playbook messaging categories if inaccurate.
- Update `docs/COMMANDS.md` to resolve `substrate world enable` flag documentation drift (`--home` vs `--prefix`) consistent with ADR-0003 + current CLI behavior.

## Draft slice skeleton (pre-planning only)

Draft; may split/merge; do not wire `tasks.json` yet.

Slice prefix (draft): `SDIHD`

- slice_id: `SDIHD0`
  - name: Stage helpers under `$SUBSTRATE_HOME`
  - intent: Stabilize helper discovery across `cargo clean` by ensuring `$SUBSTRATE_HOME/scripts/substrate/{world-enable.sh,install-substrate.sh}` exists after dev-install and by validating the fallback discovery path.
  - likely touch surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
    - `crates/shell/src/builtins/world_enable/runner/paths.rs`
    - `crates/shell/tests/world_enable.rs`
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md`
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/slices/SDIHD0/SDIHD0-spec.md`

- slice_id: `SDIHD1`
  - name: Ownership-guarded dev-uninstall cleanup
  - intent: Ensure dev-uninstall removes only dev-managed helpers under `$SUBSTRATE_HOME/scripts/substrate/…` and refuses deterministically when a destination is not dev-managed.
  - likely touch surfaces:
    - `scripts/substrate/dev-uninstall-substrate.sh`
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/decision_register.md`
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md`
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/slices/SDIHD1/SDIHD1-spec.md`

Downstream notes:
- CI-checkpoint prefers this slice list when populating the machine-readable slices list in `ci_checkpoint_plan.md` (do not validate mechanically until slice tasks exist in `tasks.json`).
- Workstream triage records proposed edits to this slice skeleton as recommendations in `workstream_triage.md` (it does not edit this file).
