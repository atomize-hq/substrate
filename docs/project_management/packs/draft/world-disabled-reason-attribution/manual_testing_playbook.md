# world-disabled-reason-attribution — manual testing playbook (authoritative)

Standard:
- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`

## Scope

Validates:
- `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/platform-parity-spec.md`

Commands and tests in scope:
- replay stderr assertions in `crates/shell/tests/replay_world.rs`
- `replay_strategy` trace assertions in `crates/shell/tests/replay_world.rs`

## Smoke scripts (required)
- Linux: `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/linux-smoke.sh`
- macOS: `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/macos-smoke.sh`
- Windows: `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/windows-smoke.ps1`

Slice selection:
- `SUBSTRATE_SMOKE_SLICE_ID=WDRA0|WDRA1|WDRA2` (default: `WDRA2`)

## Case 1 — replay-local opt-out fragments stay unchanged

Run:
```bash
cargo test -p shell --test replay_world replay_no_world_flag_reports_world_toggle -- --exact --nocapture
cargo test -p shell --test replay_world replay_env_override_reports_world_toggle -- --exact --nocapture
```

Expected:
- both tests pass
- replay keeps the exact fragments `--no-world flag` and `SUBSTRATE_REPLAY_USE_WORLD=disabled`

## Case 2 — effective override-env attribution

Run:
```bash
cargo test -p shell --test replay_world replay_recorded_host_origin_attributes_override_env -- --exact --nocapture
cargo test -p shell --test replay_world replay_trace_strategy_emits_world_disable_source_for_override_env -- --exact --nocapture
```

Expected:
- both tests pass
- replay stderr includes `world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled`
- replay telemetry emits:
  - `origin_reason_code = world_disabled_override_env`
  - `world_disable_source.layer = override_env`
  - `world_disable_source.env = SUBSTRATE_OVERRIDE_WORLD`

## Case 3 — workspace and global config attribution

Run:
```bash
cargo test -p shell --test replay_world replay_recorded_host_origin_attributes_workspace_config -- --exact --nocapture
cargo test -p shell --test replay_world replay_recorded_host_origin_attributes_global_config -- --exact --nocapture
cargo test -p shell --test replay_world replay_trace_strategy_emits_world_disable_source_for_workspace_config -- --exact --nocapture
cargo test -p shell --test replay_world replay_trace_strategy_emits_world_disable_source_for_global_config -- --exact --nocapture
```

Expected:
- all tests pass
- workspace case uses `<workspace>/.substrate/workspace.yaml`
- global case uses `$SUBSTRATE_HOME/config.yaml`
- no absolute host path appears in stderr or telemetry

## Case 4 — redaction and omission rules

Run:
```bash
cargo test -p shell --test replay_world replay_recorded_host_origin_redacts_absolute_config_paths -- --exact --nocapture
cargo test -p shell --test replay_world replay_trace_strategy_omits_world_disable_source_for_replay_opt_out -- --exact --nocapture
```

Expected:
- all tests pass
- redaction test proves tokenized path displays only
- omission test proves replay-local opt-outs do not emit `world_disable_source`

## Cross-platform smoke wrapper check

Run one platform-local smoke script from the repo root after `cargo build --bin substrate`:
- Linux: `bash docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/linux-smoke.sh`
- macOS: `bash docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/macos-smoke.sh`
- Windows: `pwsh -File docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/windows-smoke.ps1`

Expected:
- exit `0`
- slice-scoped smoke runs the test filters for the selected `SUBSTRATE_SMOKE_SLICE_ID`
