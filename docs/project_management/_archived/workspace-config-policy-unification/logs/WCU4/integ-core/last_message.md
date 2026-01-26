- Merged `workspace_config_policy_unification-wcu4-code` and `workspace_config_policy_unification-wcu4-test` into `workspace_config_policy_unification-wcu4-integ-core` (merge commits `21a7b5f`, `208bc3d`).
- Implemented WCU4 contract by removing default `SUBSTRATE_OVERRIDE_*` exports from installer/dev env scripts (now emit `SUBSTRATE_*`): `scripts/substrate/dev-install-substrate.sh:183`, `scripts/substrate/install-substrate.sh:922`.
- Added/ran WCU4 regression tests ensuring no override exports + clean `config current show` behavior: `crates/shell/tests/installer_env_wcu4.rs:24`, `crates/shell/tests/installer_env_wcu4.rs:38`.

- Local checks run/green: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p substrate-shell --test installer_env_wcu4`, `make integ-checks`.
- CI dispatch results:
  - Compile parity: `RUN_ID=21070329343` `https://github.com/atomize-hq/substrate/actions/runs/21070329343`
  - Feature smoke (behavior, `SMOKE_SLICE_ID=WCU4`): `RUN_ID=21070406238` `https://github.com/atomize-hq/substrate/actions/runs/21070406238`
- Finished: `make triad-task-finish TASK_ID="WCU4-integ-core"` ran clean (no extra commits).