# world-deps-apt-provisioning — session log

## START — 2026-03-05 — planning — tasks/checkpoints wiring
- Feature: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
- Branch: `feat/world-deps-apt-provisioning`
- Goal: Populate `tasks.json`, CI checkpoint wiring, and kickoff prompts for triad automation.

## END — 2026-03-05 — planning — tasks/checkpoints wiring
- Summary of changes (exhaustive):
  - Populated schema v4 cross-platform triads for `WDAP0` and `WDAP1` (checkpoint-boundary platform-fix model for both).
  - Created kickoff prompts for all tasks referenced by `tasks.json`.
  - Created `plan.md` and `quality_gate_report.md`.
- Rubric checks run (with results):
  - `python3 docs/project_management/system/scripts/planning/validate_tasks_json.py --feature-dir "docs/project_management/packs/draft/world-deps-apt-provisioning"` → `PASS`
  - `python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "docs/project_management/packs/draft/world-deps-apt-provisioning"` → `FAIL` (WDAP1 spec has 10 AC bullets; v2 requires 1..8)
  - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/world-deps-apt-provisioning"` → `PASS`
  - `make planning-micro-lint FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" OWNED_PATHS="tasks.json plan.md session_log.md quality_gate_report.md kickoff_prompts slices/WDAP0/kickoff_prompts slices/WDAP1/kickoff_prompts"` → `PASS`

## START — 2026-03-08 — planning — compliance review remediation
- Feature: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
- Branch: `feat/world-deps-apt-provisioning`
- Goal: bring the pack into conformance with the current planning-system validators and standards.

## END — 2026-03-08 — planning — compliance review remediation
- Summary of changes (exhaustive):
  - Upgraded `pre-planning/workstream_triage.md` to PM_PWS_INDEX v2 with explicit accepted slice order `WDAP0`, `WDAP1`.
  - Converged the pack on the accepted two-slice model and removed orphan slice specs `WDAP2` and `WDAP3`.
  - Folded provisioning wiring responsibilities into `WDAP0-spec.md` and docs-reconciliation responsibilities into `WDAP1-spec.md`.
  - Rewrote the required-doc section in `pre-planning/spec_manifest.md` so it satisfies the current spec-manifest path validator.
  - Updated `pre-planning/ci_checkpoint_plan.md`, `pre-planning/impact_map.md`, `plan.md`, `tasks.json`, and `docs/project_management/packs/sequencing.json` to match the converged slice and checkpoint model.
  - Replaced `quality_gate_report.md` with a canonical gate report and recorded current validation evidence.
  - Refreshed stale referenced ADR executive-summary hashes so the pack's ADR drift gate passes.
- Rubric checks run (with results):
  - `python3 docs/project_management/system/scripts/planning/validate_pws_index.py --feature-dir "docs/project_management/packs/draft/world-deps-apt-provisioning"` → `PASS`
  - `python3 docs/project_management/system/scripts/planning/pre_full_planning_convergence.py --feature-dir "docs/project_management/packs/draft/world-deps-apt-provisioning"` → `PASS`
  - `python3 docs/project_management/system/scripts/planning/validate_spec_manifest.py --feature-dir "docs/project_management/packs/draft/world-deps-apt-provisioning"` → `PASS`
  - `python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "docs/project_management/packs/draft/world-deps-apt-provisioning"` → `PASS`
  - `python3 docs/project_management/system/scripts/planning/validate_slice_inventory_coherence.py --feature-dir "docs/project_management/packs/draft/world-deps-apt-provisioning" --phase execution_ready` → `PASS`
  - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/world-deps-apt-provisioning"` → `PASS`
  - `make planning-lint FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning"` → `PASS`

## START — 2026-03-09T00:56:27Z — code — WDAP0-code
- Dispatch:
  - `make triad-task-start-complete FEATURE_DIR="/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning" SLICE_ID="WDAP0"`

## START — 2026-03-09T00:56:27Z — test — WDAP0-test
- Dispatch:
  - `make triad-task-start-complete FEATURE_DIR="/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning" SLICE_ID="WDAP0"`

## END — 2026-03-09T01:20:03Z — code — WDAP0-code
- HEAD: `18112a3dbc2d8dc94c92e057258f28f122f7aadc`
- Codex last message: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/code/last_message.md`

## END — 2026-03-09T01:20:03Z — test — WDAP0-test
- HEAD: `f058e78432318f7088445832fc68e3e6741b2bb9`
- Codex last message: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/test/last_message.md`

## START — 2026-03-09T01:20:03Z — integration — WDAP0-integ-core
- Dispatch:
  - `make triad-task-start FEATURE_DIR="/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning" TASK_ID="WDAP0-integ-core" LAUNCH_CODEX=1`

## END — 2026-03-09T02:15:47Z — integration — WDAP0-integ-core
- HEAD: `8d1f1a3aa4d227fca390bac1301807d424a51435`
- Summary of changes (exhaustive):
  - Merged `world-deps-apt-provisioning-wdap0-code` and `world-deps-apt-provisioning-wdap0-test` into the integration worktree and reconciled the merged result to the WDAP0 spec.
  - Fixed WDAP0 test-module wiring so workspace clippy passes by importing shared test helpers through the existing `support` module in `crates/shell/tests/world_enable_provision_deps_wdap0.rs`.
  - Stabilized the unrelated PTY rendering regression in `crates/shell/tests/repl_world_first_rendering_v1.rs` so `make integ-checks` is green under full-suite load.
  - Updated the orchestration-branch impact map to include `crates/shell/tests/repl_world_first_rendering_v1.rs`, which was touched during integration deflake work.
- Local gates run (with results):
  - `cargo fmt --all --check` → `PASS`
  - `cargo clippy --workspace --all-targets -- -D warnings` → `PASS`
  - `cargo test -p shell --test world_enable_provision_deps_wdap0 -- --nocapture` → `PASS`
  - `bash tests/mac/installer_parity_fixture.sh --scenario sync-deps-remediation` → `PASS`
  - `make integ-checks` → `PASS`
  - `make triad-task-finish TASK_ID="WDAP0-integ-core"` → `PASS`
- Behavioral smoke preflight:
  - `cargo build --bin substrate && PATH="$PWD/target/debug:$PATH" bash "docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/macos-smoke.sh"` → `FAIL`
  - Failure is in the smoke harness, not the WDAP0 implementation: Case A captures file contents with command substitution (`out="$(cat ...)"`), which strips the trailing newline before comparing against `expected_stdout=$'smoke-apt-a\nsmoke-apt-b=1\n'`. The actual file bytes match the expected output including the trailing newline.
- Next step:
  - Run `CP1-ci-checkpoint` from the orchestration checkout. Do not dispatch checkpoint CI from the WDAP0 integration worktree.

## END — 2026-03-09T02:21:16Z — integration — WDAP0-integ-core
- HEAD: `8d1f1a3aa4d227fca390bac1301807d424a51435`
- Codex last message: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/integ-core/last_message.md`

## START — 2026-03-10T23:33:02Z — checkpoint — CP1-ci-checkpoint
- Dispatch:
  - Resume-safe wrapper checkpoint bookkeeping on orchestration checkout for `WDAP0` validating `CHECKOUT_SHA=8d1f1a3aa4d227fca390bac1301807d424a51435`
- Checkpoint evidence:
  - Compile parity run `22929114889` → `PASS` (`https://github.com/atomize-hq/substrate/actions/runs/22929114889`)
  - Behavior smoke run `22929212356` → `CANCELLED` after `linux_self_hosted` failed preflight (`/run/substrate.sock` missing), `macos_self_hosted` failed on the smoke harness newline-comparison bug, and `windows_self_hosted` remained queued
  - Behavior smoke run `22931331235` (workflow ref `world-deps-apt-provisioning-wdap0-integ-macos`) → `CANCELLED` after `macos_self_hosted` passed and `linux_self_hosted` failed because `sudo -n` was unavailable on the Linux runner, preventing `scripts/linux/world-provision.sh --profile release --skip-build --sudo-noninteractive`
  - Hosted Windows smoke runs `22932886416`, `22933186355`, and `22933497530` were operator-authorized exploratory reruns on GitHub-hosted Windows and all failed inside `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/windows-smoke.ps1`
  - Hosted Linux smoke run `22933937566` against `CHECKOUT_SHA=83efb8fe2a41faf2608df4fc03bc5816edb23926` (workflow ref `world-deps-apt-provisioning-wdap0-integ-macos`) → `FAIL`; `linux_hosted` reached the smoke script but failed Case A preflight because `substrate world doctor` reported the world backend unhealthy (`https://github.com/atomize-hq/substrate/actions/runs/22933937566/job/66561113342`)

## END — 2026-03-11T02:28:44Z — integration — WDAP0-integ-macos
- HEAD: `83efb8fe2a41faf2608df4fc03bc5816edb23926`
- Validation:
  - Local macOS smoke preflight with `SUBSTRATE_SMOKE_SLICE_ID=WDAP0` → `PASS`
  - `macos_self_hosted` in run `22931331235` → `PASS` (`https://github.com/atomize-hq/substrate/actions/runs/22931331235/job/66553365246`)
- Branch/worktree:
  - Branch: `world-deps-apt-provisioning-wdap0-integ-macos`
  - Worktree: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/wt/world-deps-apt-provisioning-wdap0-integ-macos`

- Checkpoint evidence refresh for `CHECKOUT_SHA=83efb8fe2a41faf2608df4fc03bc5816edb23926`:
  - Compile parity run `22954393969` → `PASS` (`https://github.com/atomize-hq/substrate/actions/runs/22954393969`)
  - Behavior smoke run `22954501646` → `FAIL` (`https://github.com/atomize-hq/substrate/actions/runs/22954501646`)
    - `linux_self_hosted` failed runner preflight because `/run/substrate.sock` is missing (`https://github.com/atomize-hq/substrate/actions/runs/22954501646/job/66627480526`)
    - `windows_self_hosted` failed Case A because `world enable --provision-deps --dry-run` exited `0` instead of expected exit `4` (`https://github.com/atomize-hq/substrate/actions/runs/22954501646/job/66627480389`)
    - `macos_self_hosted` failed on the known newline-comparison smoke harness bug (`https://github.com/atomize-hq/substrate/actions/runs/22954501646/job/66627480282`)
