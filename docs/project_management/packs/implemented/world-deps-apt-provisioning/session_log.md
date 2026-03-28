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

- Checkpoint evidence refresh for `CHECKOUT_SHA=83efb8fe2a41faf2608df4fc03bc5816edb23926`:
  - Compile parity run `22962676818` → `PASS` (`https://github.com/atomize-hq/substrate/actions/runs/22962676818`)
  - Behavior smoke run `22962814918` → `FAIL` (`https://github.com/atomize-hq/substrate/actions/runs/22962814918`)
    - `macos_self_hosted` passed (`https://github.com/atomize-hq/substrate/actions/runs/22962814918/job/66647757316`)
    - `linux_self_hosted` failed because the self-hosted Linux job still exercised the candidate smoke script and tripped the `world doctor` preflight (`https://github.com/atomize-hq/substrate/actions/runs/22962814918/job/66647757455`)
    - `windows_self_hosted` still failed Case A with exit `0` (`https://github.com/atomize-hq/substrate/actions/runs/22962814918/job/66647757282`)
  - Behavior smoke run `22963272168` → `FAIL` (`https://github.com/atomize-hq/substrate/actions/runs/22963272168`)
    - `linux_self_hosted` still failed on the candidate-script preflight mismatch (`https://github.com/atomize-hq/substrate/actions/runs/22963272168/job/66655961114`)
    - `windows_self_hosted` still failed Case A with exit `0` (`https://github.com/atomize-hq/substrate/actions/runs/22963272168/job/66655960974`)
    - `macos_self_hosted` passed (`https://github.com/atomize-hq/substrate/actions/runs/22963272168/job/66655960916`)
  - Behavior smoke run `22963496713` → `FAIL` (`https://github.com/atomize-hq/substrate/actions/runs/22963496713`)
    - `linux_self_hosted` passed (`https://github.com/atomize-hq/substrate/actions/runs/22963496713/job/66660508482`)
    - `macos_self_hosted` passed (`https://github.com/atomize-hq/substrate/actions/runs/22963496713/job/66660508454`)
    - `windows_self_hosted` still failed Case A with exit `0` (`https://github.com/atomize-hq/substrate/actions/runs/22963496713/job/66660508435`)
  - Behavior smoke run `22963961331` → `FAIL` (`https://github.com/atomize-hq/substrate/actions/runs/22963961331`)
    - `linux_self_hosted` passed (`https://github.com/atomize-hq/substrate/actions/runs/22963961331/job/66662238819`)
    - `windows_self_hosted` advanced past the silent exit but failed because the launcher rename left the call sites on `-Args`, so the binary dropped into interactive no-TTY mode (`https://github.com/atomize-hq/substrate/actions/runs/22963961331/job/66662238771`)
  - Behavior smoke run `22964098783` → `CANCELLED` (`https://github.com/atomize-hq/substrate/actions/runs/22964098783`)
    - `linux_self_hosted` passed (`https://github.com/atomize-hq/substrate/actions/runs/22964098783/job/66662731537`)
    - `windows_self_hosted` advanced through Case A and then failed the unnecessary WDAP0 `world doctor` preflight because the smoke harness rewrote `HOME`/`USERPROFILE` before `wsl-warm.ps1` resolved `cargo.exe` (`https://github.com/atomize-hq/substrate/actions/runs/22964098783/job/66662731569`)
    - `macos_self_hosted` was cancelled after the superseding rerun was dispatched (`https://github.com/atomize-hq/substrate/actions/runs/22964098783/job/66662731693`)
  - Behavior smoke run `22964186614` → `PASS` (`https://github.com/atomize-hq/substrate/actions/runs/22964186614`)
    - `linux_self_hosted` passed (`https://github.com/atomize-hq/substrate/actions/runs/22964186614/job/66663052345`)
    - `windows_self_hosted` passed (`https://github.com/atomize-hq/substrate/actions/runs/22964186614/job/66663052239`)
    - `macos_self_hosted` passed (`https://github.com/atomize-hq/substrate/actions/runs/22964186614/job/66663052246`)

## END — 2026-03-11T16:56:23Z — checkpoint — CP1-ci-checkpoint
- Validated HEAD: `83efb8fe2a41faf2608df4fc03bc5816edb23926`
- Compile parity: `22962676818` (`https://github.com/atomize-hq/substrate/actions/runs/22962676818`)
- Behavior smoke: `22964186614` (`https://github.com/atomize-hq/substrate/actions/runs/22964186614`)

## START — 2026-03-11T17:31:00Z — integration — WDAP0-integ
- Dispatch:
  - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" TASK_ID="WDAP0-integ" LAUNCH_CODEX=1`

## END — 2026-03-11T17:36:00Z — integration — WDAP0-integ
- HEAD: `e0ef4123ca8a841ca49c4c2c148931e2badb095c`
- Codex last message: `docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/integ/last_message.md`

## START — 2026-03-11T17:29:20Z — code — WDAP1-code
- Dispatch:
  - `make triad-task-start-complete FEATURE_DIR="/home/spenser/__Active_code/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning" SLICE_ID="WDAP1"`

## START — 2026-03-11T17:29:20Z — test — WDAP1-test
- Dispatch:
  - `make triad-task-start-complete FEATURE_DIR="/home/spenser/__Active_code/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning" SLICE_ID="WDAP1"`

## END — 2026-03-11T17:43:27Z — code — WDAP1-code
- HEAD: `dd32463667e03c288f58986bc5182163e26d59ef`
- Codex last message: `/home/spenser/__Active_code/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP1/code/last_message.md`

## END — 2026-03-11T17:43:27Z — test — WDAP1-test
- HEAD: `bd50c2170f01497db0aa44773faed30641920faf`
- Codex last message: `/home/spenser/__Active_code/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP1/test/last_message.md`

## START — 2026-03-11T17:43:27Z — integration — WDAP1-integ-core
- Dispatch:
  - `make triad-task-start FEATURE_DIR="/home/spenser/__Active_code/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning" TASK_ID="WDAP1-integ-core" LAUNCH_CODEX=1`

## END — 2026-03-11T17:57:57Z — integration — WDAP1-integ-core
- HEAD: `f0ce46505852de5935086ce7cdaa5dbde68cf52e`
- Codex last message: `/home/spenser/__Active_code/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP1/integ-core/last_message.md`

## START — 2026-03-11T19:45:07Z — checkpoint — CP2-ci-checkpoint
- Dispatch:
  - `Checkpoint bookkeeping on orchestration checkout for WDAP1 validating CHECKOUT_SHA=f0ce46505852de5935086ce7cdaa5dbde68cf52e`

## START — 2026-03-11T19:52:21Z — integration — WDAP1-integ-linux
- Dispatch:
  - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" TASK_ID="WDAP1-integ-linux" TASK_PLATFORM=linux LAUNCH_CODEX=1`

## START — 2026-03-11T19:52:21Z — integration — WDAP1-integ-macos
- Dispatch:
  - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" TASK_ID="WDAP1-integ-macos" TASK_PLATFORM=macos LAUNCH_CODEX=1`

## START — 2026-03-11T19:52:21Z — integration — WDAP1-integ-windows
- Dispatch:
  - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" TASK_ID="WDAP1-integ-windows" TASK_PLATFORM=windows LAUNCH_CODEX=1`

## END — 2026-03-11T22:02:32Z — integration — WDAP1-integ-linux
- HEAD: `985e740acd3c5b252f6c48a59f202dfc97c3dca0`
- Feature smoke: `22975895887` https://github.com/atomize-hq/substrate/actions/runs/22975895887
- Codex last message: `/home/spenser/__Active_code/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP1/integ-linux/last_message.md`

## END — 2026-03-11T22:02:32Z — integration — WDAP1-integ-macos
- Result: no macOS-only fix required; latest verified behavior smoke PASS
- Feature smoke: `22975719947` https://github.com/atomize-hq/substrate/actions/runs/22975719947
- Codex last message: `/home/spenser/__Active_code/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP1/integ-macos/last_message.md`

## END — 2026-03-11T22:02:32Z — integration — WDAP1-integ-windows
- Result: completed as no-op after operator decision to treat Windows as unsupported for WDAP1 behavior validation
- Prior failing smoke (no longer blocking WDAP1 behavior gates): `22976062346` https://github.com/atomize-hq/substrate/actions/runs/22976062346
- Codex last message: `/home/spenser/__Active_code/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP1/integ-windows/last_message.md`

## END — 2026-03-11T22:02:32Z — checkpoint — CP2-ci-checkpoint
- Checkout SHA: `985e740acd3c5b252f6c48a59f202dfc97c3dca0`
- CI compile parity PASS: `22975896193` https://github.com/atomize-hq/substrate/actions/runs/22975896193
- Linux behavior smoke PASS: `22975895887` https://github.com/atomize-hq/substrate/actions/runs/22975895887
- macOS behavior smoke PASS: `22975719947` https://github.com/atomize-hq/substrate/actions/runs/22975719947
- Windows behavior smoke de-scoped for WDAP1 on 2026-03-11; `tasks.json` `meta.behavior_platforms_required=["linux","macos"]`

## START — 2026-03-11T22:03:08Z — integration — WDAP1-integ
- Dispatch:
  - `make triad-task-start-integ-final FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" SLICE_ID="WDAP1" LAUNCH_CODEX=1`

## END — 2026-03-11T22:14:44Z — integration — WDAP1-integ
- HEAD: `232b5bea0207c42c2ed4b40b8618898887e40a15`
- Validation: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, WDAP1-focused tests, `make integ-checks`
- Merge result: merged `world-deps-apt-provisioning-wdap1-integ` back to `feat/world-deps-apt-provisioning`
- Codex last message: `/home/spenser/__Active_code/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP1/integ/last_message.md`

## START — 2026-03-13T15:27:11Z — integration — WDAP0-integ-core (post-landing follow-up)
- Goal: land the reserved internal-profile guard, transient provisioning posture, and validator-clean WDAP0 pack updates on `feat/world-deps-apt-provisioning` without introducing a new slice/task id.

## END — 2026-03-13T15:37:46Z — integration — WDAP0-integ-core (post-landing follow-up)
- Base HEAD: `eed1863c6c2827893ced4177cc01690bfcd53874`
- Summary of changes (exhaustive):
  - Kept the shell-side `SUBSTRATE_WORLD_REQUEST_PROFILE` guard that rejects reserved internal WDAP profiles in `crates/shell/src/execution/routing/dispatch/world_ops.rs`.
  - Kept the `world-agent` transient `systemd-run` wrapper for `profile=world-deps-provision` in `crates/world-agent/src/service.rs`.
  - Updated `docs/CONFIGURATION.md`, `docs/WORLD.md`, and `docs/internals/world/deps.md` so the internal WDAP profiles are explicitly reserved and not selectable through the generic env override.
  - Folded the real supported-guest non-dry-run validation requirement into `AC-WDAP0-07` in `slices/WDAP0/WDAP0-spec.md` and refreshed `manual_testing_playbook.md` / `pre-planning/ci_checkpoint_plan.md` to match the validator-clean contract.
  - Fixed `docs/project_management/system/scripts/triad/feature_cleanup.sh` so the documented cleanup target runs on macOS bash (no `mapfile` dependency) and normalizes relative-vs-absolute worktree paths during dry-run cleanup accounting.
- Local gates run (with results):
  - `cargo test -p shell current_world_request_profile --lib` → `PASS`
  - `cargo test -p shell --test world_enable_provision_deps_wdap0` → `PASS`
  - `cargo test -p shell --test world_deps_apt_fail_early_wdap1` → `PASS`
  - `cargo test -p world-agent --lib` → `PASS`
  - `cargo fmt --all -- --check` → `PASS`
  - `cargo clippy --workspace --all-targets -- -D warnings` → `PASS`
  - `make integ-checks` → `PASS`

## START — 2026-03-13T15:37:46Z — checkpoint — CP1-ci-checkpoint (WDAP0 evidence refresh)
- Goal: refresh WDAP0 checkpoint evidence for the post-landing follow-up on `feat/world-deps-apt-provisioning` using the existing `CP1-ci-checkpoint` task id.

## END — 2026-03-13T15:38:24Z — checkpoint — CP1-ci-checkpoint (WDAP0 evidence refresh)
- Base HEAD: `eed1863c6c2827893ced4177cc01690bfcd53874`
- Planning validators:
  - `python3 docs/project_management/system/scripts/planning/validate_tasks_json.py --feature-dir "docs/project_management/packs/draft/world-deps-apt-provisioning"` → `PASS`
  - `python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "docs/project_management/packs/draft/world-deps-apt-provisioning"` → `PASS`
  - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/world-deps-apt-provisioning"` → `PASS`
- Supported guest non-dry-run evidence (macOS Lima, WDAP0 Case 2.4):
  - Package selected: `cowsay` (absent before run, installed after run)
  - Evidence directory: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/manual-20260313T152711Z`
  - Before guest query: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/manual-20260313T152711Z/real-apt-before.txt` → empty / no `install ok installed`
  - Provisioning stdout/stderr: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/manual-20260313T152711Z/stdout-real.txt`, `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/manual-20260313T152711Z/stderr-real.txt`
  - Exit code: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/manual-20260313T152711Z/exit-code.txt` → `0`
  - After guest query: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/manual-20260313T152711Z/real-apt-after.txt` → `install ok installed`
  - Dry-run follow-up: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/manual-20260313T152711Z/sync-dry-run-stdout.txt`, `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/manual-20260313T152711Z/sync-dry-run-stderr.txt`
- Validation note:
  - The authoritative before/after package checks now use `limactl shell substrate dpkg-query ...` because a normal workspace-isolated `substrate --world` shell masks `/var/lib/dpkg`; see `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/manual-20260313T152711Z/world-dpkg-query.txt` (empty) and `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/manual-20260313T152711Z/world-dpkg-query.stderr.txt`.
- WDAP1 regression check:
  - `cargo test -p shell --test world_deps_apt_fail_early_wdap1` remains `PASS`; no CP2 artifact change required for this follow-up.

## UPDATE — 2026-03-13T15:38:24Z — ops — FZ-feature-cleanup
- Dry-run preflight:
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1` initially failed on macOS because `docs/project_management/system/scripts/triad/feature_cleanup.sh` used `mapfile`, which is unavailable in the repo's default bash on this host.
  - After the portability fix, `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1 FORCE=1` → `PASS` with summary:
    - `REMOVED_WORKTREES=6`
    - `PRUNED_LOCAL_BRANCHES=6`
    - `PRUNED_REMOTE_BRANCHES=0`
- Remaining blocker:
  - The real cleanup run is still pending because `feature_cleanup.sh` correctly refuses non-dry-run execution while the orchestration checkout has uncommitted WDAP follow-up changes.

## END — 2026-03-28T19:45:00Z — maintenance — FZ-feature-cleanup
- Feature: `docs/project_management/packs/draft/world-deps-apt-provisioning`
- Branch: `feat/world-deps-apt-provisioning`
- Commands run (verbatim) + exit codes:
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1 FORCE=1` (exit `0`)
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" REMOVE_WORKTREES=1 PRUNE_LOCAL=1 FORCE=1` (initial exit `2`; stale prunable worktree registrations still pointed at missing paths)
  - `git worktree prune --verbose` (exit `0`)
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1 FORCE=1` (exit `0`)
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" REMOVE_WORKTREES=1 PRUNE_LOCAL=1 FORCE=1` (exit `0`)
- Stdout summary block from the successful cleanup run:
  ```
  REMOVED_WORKTREES=0
  PRUNED_LOCAL_BRANCHES=6
  PRUNED_REMOTE_BRANCHES=0
  ```
- Post-cleanup state:
  - `git worktree list` shows only the orchestration checkout.
  - `git branch --list 'world-deps-apt-provisioning*'` returns no remaining slice branches.
- Manual validation closeout:
  - Re-ran the macOS-applicable manual smoke matrix from `docs/project_management/packs/draft/world-deps-apt-provisioning/manual_testing_playbook.md` against the installed dev binary `~/.substrate/bin/substrate` on `2026-03-28`.
  - Preconditions plus Cases `2.2`, `2.3`, `2.5`, `3.1`, `3.2`, `3.3`, and `3.4` matched the pack contract.
  - Case `2.4` showed the expected installed-binary helper lookup behavior before override; the underlying guest provisioning path succeeded when `SUBSTRATE_WORLD_ENABLE_SCRIPT="$HOME/.substrate/scripts/substrate/world-enable.sh"` was supplied, which matches the landed implementation posture for this pack.
  - Conclusion: no open WDAP pack work remains; pack execution, validation, and cleanup are complete.
