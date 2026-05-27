# ADR-0035 — Make `substrate world enable` work after `dev-install-substrate.sh --no-world`

## Status
- Status: Draft
- Date (UTC): 2026-02-21
- Owner(s): TBD (ASSUMPTION: Substrate shell + installer maintainers)

## Stable Curated ADR

- Current stable ADR: `docs/adr/implemented/ADR-0035-make-substrate-world-enable-work-after-dev-install-no-world.md`
- This project-management file remains the planning-rich historical source retained for
  compatibility while `docs/project_management/**` is being retired.

## Scope
- Feature directory: `docs/project_management/packs/draft/dev-install-world-service-staging/` (ASSUMPTION)
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md`

## Related Docs (links only)
- Intake: `docs/project_management/intake/adrs/summoning_wombat_adr_intake.md`
- Plan: `docs/project_management/packs/draft/dev-install-world-service-staging/plan.md`
- Tasks: `docs/project_management/packs/draft/dev-install-world-service-staging/tasks.json`
- Spec manifest: `docs/project_management/packs/draft/dev-install-world-service-staging/pre-planning/spec_manifest.md`
- Slice specs:
  - `docs/project_management/packs/draft/dev-install-world-service-staging/slices/DIWAS0/DIWAS0-spec.md`
  - `docs/project_management/packs/draft/dev-install-world-service-staging/slices/DIWAS1/DIWAS1-spec.md`
- Contract: `docs/project_management/packs/draft/dev-install-world-service-staging/contract.md`
- Decision Register: `docs/project_management/packs/draft/dev-install-world-service-staging/decision_register.md`
- Impact Map: `docs/project_management/packs/draft/dev-install-world-service-staging/pre-planning/impact_map.md`
- Manual Playbook: `docs/project_management/packs/draft/dev-install-world-service-staging/manual_testing_playbook.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 0a57c0f7a17a1bfcd2e32616e9867526a6d2909e55caa44948c51f017741fbbb

### Changes (operator-facing)
- Linux dev installs done with `--no-world` become “enable later” ready without extra manual build steps.
  - Existing: `scripts/substrate/dev-install-substrate.sh --no-world` disables the world in `~/.substrate/config.yaml` *and* can leave the inferred “version dir” (often `<repo>/target/`) without a `bin/(linux/)world-service` artifact. Later, `substrate world enable` can fail with low-level errors (missing `world-service`, readiness probe failures) that read like breakage.
  - New: On Linux, `scripts/substrate/dev-install-substrate.sh --no-world` still stages a `world-service` binary into the version-dir layout used by the `world-enable.sh` helper (`<repo>/target/bin/world-service` and/or `<repo>/target/bin/linux/world-service`). `substrate world enable` either provisions successfully or fails early with a single actionable remediation if the artifact is still missing.
  - Why: Make “install with `--no-world`, enable later” a reliable, repeatable dev workflow; avoid confusing downstream failures that don’t point to the missing artifact root cause.
  - Links:
    - `scripts/substrate/dev-install-substrate.sh` (dev install layout + `ensure_release_bin_bridge`)
    - `scripts/substrate/install-substrate.sh` (`provision_linux_world` expects `bin/(linux/)world-service`)
    - `scripts/substrate/world-enable.sh` (enable helper uses `RELEASE_ROOT` / version dir)
    - `crates/shell/src/builtins/world_enable/runner.rs` (`substrate world enable` runner updates `world.enabled`)

## Problem / Context
- `scripts/substrate/dev-install-substrate.sh --no-world` writes `~/.substrate/config.yaml` with `world.enabled: false`, which is correct, but it also commonly leaves dev installs without a staged `world-service` under the helper’s inferred version directory (typically `<repo>/target/bin/(linux/)world-service`).
- `substrate world enable` (via `scripts/substrate/world-enable.sh` and `scripts/substrate/install-substrate.sh`) assumes the provisioning artifact exists in the version dir, and missing-artifact errors can surface late and/or be misinterpreted as general world breakage rather than “missing binary artifact”.
- Production installs ship a bundle that already contains `world-service`, so this is primarily a dev-install sharp edge.

## Goals
- After `scripts/substrate/dev-install-substrate.sh --no-world` on Linux, a developer can run `substrate world enable` and provisioning is execution-ready (no manual `cargo build -p world-service` step required).
- If `world-service` is missing anyway (misconfigured checkout, partial build, etc.), `substrate world enable` fails early with a single, correct, actionable remediation (build/stage guidance) instead of downstream socket/readiness errors.
- Preserve production install behavior: `scripts/substrate/install-substrate.sh --no-world` remains enable-able later without regression.

## Non-Goals
- Redesigning world isolation, overlay/copy-diff behavior, or the world-service API.
- Expanding world-deps inventory coverage or changing dependency semantics.
- Making `substrate world enable` supported on Windows (remains unsupported).
- Reworking systemd unit hardening/capabilities beyond what is necessary for this slice.

## Out of Scope
- “World-disabled UX” cleanup: `substrate health` / `substrate shim doctor` / world-deps probing behavior when `world.enabled: false`.
- Doctor/health messaging improvements about *why* the world is disabled (config vs CLI flag vs env).
- Dev-install “bundle parity” under `$SUBSTRATE_HOME/versions/<label>/{bin,scripts,config}` (layout parity with production installs).
- macOS Lima path changes (Linux-first; macOS may need a follow-up ADR).

## Options

### Option A — Build/stage `world-service` during `substrate world enable` when missing
When enable detects a dev-install version dir and cannot find `bin/world-service` or `bin/linux/world-service`, it performs a targeted build (`cargo build -p world-service --release` or selected profile) and stages the binary before provisioning.

### Option B — Always build/stage `world-service` during dev-install even with `--no-world` (recommended for this ADR)
Change `scripts/substrate/dev-install-substrate.sh --no-world` so it still builds `world-service` (Linux host) and stages it into the version-dir layout (via the existing `ensure_release_bin_bridge`), but continues to skip provisioning/systemd and keeps `world.enabled: false` until enable is run later.

### Option C — Keep behavior; improve failure messaging + docs (explicit 2-step enable)
Do not change build/stage behavior. Instead, add preflight checks so enable detects missing `world-service` early and exits with explicit remediation (`cargo build -p world-service --release` / rerun dev-install).

## Recommendation
- Choose Option A when the project wants a single-command enable experience even if dev-install skipped building `world-service`, and it is acceptable for enable to depend on a source checkout + `cargo`.
- Choose Option B when the project wants enable to remain provisioning-only (no builds), and can accept slightly slower/bigger dev installs in exchange for a reliable “enable later” flow.
- Choose Option C when the project wants the smallest behavior delta and accepts a multi-step dev enable workflow.
- Recommended for this ADR: **Option B**, plus the early, deterministic preflight behavior from Option C (so missing artifacts never fall through to confusing downstream failures).

## Slice Decomposition
- C0 — Deterministic preflight when `world-service` is missing (Linux).
  - Add an early check in the enable path to detect missing `bin/(linux/)world-service` under the helper’s `RELEASE_ROOT` (version dir) and error with a single actionable remediation, rather than continuing into systemd/socket/readiness checks.
- C1 — Dev-install `--no-world` stages `world-service` (Linux).
  - Ensure `scripts/substrate/dev-install-substrate.sh --no-world` still produces the staged artifact(s) (`target/bin/world-service` and `target/bin/linux/world-service`) that `scripts/substrate/install-substrate.sh` expects for provisioning.

## User Contract (Authoritative)

### CLI
- Commands:
  - `scripts/substrate/dev-install-substrate.sh --no-world` (Linux):
    - Builds (or ensures the build of) `world-service` for the selected dev-install profile (`debug` or `release`).
    - Stages `world-service` into the inferred version dir layout via `ensure_release_bin_bridge`, so `<repo>/target/bin/world-service` and `<repo>/target/bin/linux/world-service` exist after dev install.
    - Does **not** provision systemd units and leaves `world.enabled: false` in `~/.substrate/config.yaml`.
  - `substrate world enable` (Linux):
    - Uses the existing helper-script flow (`scripts/substrate/world-enable.sh`) and provisions systemd using the staged `world-service` from the inferred version directory.
    - If the staged `world-service` artifact is missing, exits early with a remediation message that includes at least one of:
      - `cargo build -p world-service --release` and re-run `scripts/substrate/dev-install-substrate.sh --no-world`, or
      - re-run dev install without `--no-world` if the user wants the installer to manage the artifact end-to-end.
- Exit codes:
  - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md` (no overrides in this ADR).

### Config
- Files and locations (precedence):
  1. `$SUBSTRATE_HOME/config.yaml` (default: `~/.substrate/config.yaml`): authoritative persisted toggle for `world.enabled`.
  2. CLI flag `substrate world enable --home <path>`: sets the effective `$SUBSTRATE_HOME` for this invocation.
- Schema:
  - `world.enabled`: boolean.
    - `false`: world isolation is disabled and the systemd service may be absent/unprovisioned.
    - `true`: world provisioning has completed and world backends may be used.

### Platform guarantees
- Linux: supported by this ADR.
- macOS: no change in this ADR (follow-up may be required for dev-install parity with Lima guest agent needs).
- Windows: `substrate world enable` remains unsupported.

## Architecture Shape
- Components:
  - `scripts/substrate/dev-install-substrate.sh`: ensure `world-service` is included in the build/staging step even when `--no-world` is set (Linux only).
  - `scripts/substrate/install-substrate.sh` (`provision_linux_world`): remains the provisioning implementation; this ADR relies on its expectation that `world-service` exists under `version_dir/bin/(linux/)world-service`.
  - `scripts/substrate/world-enable.sh`: orchestrates provisioning and post-provision checks; adds/uses preflight error messaging for missing `world-service`.
  - `crates/shell/src/builtins/world_enable/*`: CLI runner responsible for locating the helper, invoking it, and setting `world.enabled: true` on success.
- End-to-end flow:
  - Inputs: repo checkout, `dev-install-substrate.sh` flags (`--profile`, `--no-world`), later `substrate world enable` (`--home`, `--dry-run`).
  - Derived state: dev-install “version dir” is inferred as `<repo>/target/` (dev install); helper `RELEASE_ROOT` is computed relative to the helper script location.
  - Actions:
    - Dev-install stages required binaries (including `world-service`) into `<repo>/target/bin/(linux/)…`.
    - Enable runs the helper; helper provisions systemd and performs post-provision checks; CLI flips `world.enabled: true` only after successful provisioning verification.
  - Outputs: deterministic enable behavior for dev installs; either successful provisioning or a single missing-artifact remediation.

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → TBD
- Prerequisite integration task IDs: none
- Adjacent work (not required for this ADR):
  - A follow-up ADR to skip world-backend probing when `world.enabled: false` for `substrate health` / `substrate shim doctor`.

## Work Lift (discovery estimate)

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "touch": {
    "create_files": null,
    "edit_files": 2,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 1,
    "boundary_crossings": null
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 0,
    "behavior_deltas": 1
  },
  "qa": { "new_test_files": null, "new_test_cases": null },
  "docs": { "new_docs_files": 0 },
  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },
  "risk": {
    "cross_platform": true,
    "security_sensitive": false,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": null
  },
  "notes": "Discovery estimate; dev-install/no-world follow-up enable path (artifact staging + deterministic remediation)."
}
```
<!-- PM_LIFT_VECTOR:END -->

## Security / Safety Posture
- Fail-closed rules:
  - Provisioning remains explicit and privileged (systemd/sudo); this ADR does not introduce silent privilege escalation.
  - If the `world-service` provisioning artifact is missing, enable must stop before attempting systemd/socket/readiness checks.
- Protected paths/invariants:
  - No `cargo build` should run under `sudo` (if Option A is ever chosen in the future, the build must remain unprivileged and clearly messaged).
  - `--dry-run` remains no-change.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - Add/extend tests around enable preflight error formatting (location TBD: Rust runner if implemented there; otherwise script-level test harness in the feature pack).
- Integration tests:
  - Add a Linux-only integration test that runs `scripts/substrate/dev-install-substrate.sh --no-world --profile release` in a workspace checkout, then asserts `test -x target/bin/linux/world-service` (or equivalent staged location) holds post-install.

### Manual validation
- Manual playbook: `docs/project_management/packs/draft/dev-install-world-service-staging/manual_testing_playbook.md` (TBD), covering:
  - Run `scripts/substrate/dev-install-substrate.sh --no-world --profile release`.
  - Confirm staged artifacts exist under `<repo>/target/bin/(linux/)world-service`.
  - Run `substrate world enable --dry-run` and confirm no missing-artifact warning is produced (Linux).
  - Run `substrate world enable` (privileged) and confirm `world.enabled: true` is written to `$SUBSTRATE_HOME/config.yaml` after successful provisioning.

### Smoke scripts
- Linux: `docs/project_management/packs/draft/dev-install-world-service-staging/smoke/linux-smoke.sh` (TBD)
- macOS: none (out of scope)
- Windows: none (out of scope; `world enable` unsupported)

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none (production installs unchanged; dev-install behavior only)

## Decision Summary
- Decision Register entries (if applicable):
  - `docs/project_management/packs/draft/dev-install-world-service-staging/decision_register.md`:
    - DR-0001 (where to implement enable preflight: Rust runner vs helper script vs installer helper)
    - DR-0002 (dev meaning of `--no-world`: “skip provisioning only” vs “skip all world-related build outputs”)
    - DR-0003 (profile mapping for staging `world-service`: release-only vs match `dev-install --profile`)
    - DR-0004 (overwrite policy if staged `world-service` already exists in `target/bin/(linux/)`)
    - DR-0005 (accepted staged path sufficiency rule: accept either vs require both)
- Options (required; at least two):
  - A) Build/stage `world-service` during `substrate world enable` when missing (requires a source checkout + cargo toolchain).
  - B) Always build/stage `world-service` during `dev-install-substrate.sh --no-world` (recommended for this ADR).
  - C) Keep behavior; improve failure messaging + docs (explicit 2-step enable).
- Selection:
  - Chosen: B (plus the early, deterministic preflight behavior from Option C).
  - Rationale: Keeps `substrate world enable` provisioning-focused and deterministic while making the dev `--no-world -> enable later` workflow reliable without requiring ad-hoc manual builds.
  - Choose B when: we want a reliable dev workflow that does not depend on `cargo` being available at enable time.
  - Choose A when: we strongly prefer "enable builds what it needs" and can reliably detect a source checkout + cargo environment without surprising privileged behavior.
