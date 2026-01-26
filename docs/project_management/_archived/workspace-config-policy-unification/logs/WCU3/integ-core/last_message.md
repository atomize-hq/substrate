- Merged `workspace_config_policy_unification-wcu3-code` + `workspace_config_policy_unification-wcu3-test` into `workspace_config_policy_unification-wcu3-integ-core` and fixed drift so WCU3 Phase B is green (keys + mutations + provenance + no-write invalid-enum behavior). Key files: `crates/shell/src/execution/config_cmd.rs`, `crates/shell/src/execution/config_model.rs`, `crates/shell/tests/config_world_deps_phase_b.rs`.
- Fixed `make integ-checks` failure by aligning global config init expectations to sparse patch semantics in `crates/shell/tests/config_init.rs`.
- Fixed feature smoke failure (“global_patch before workspace_patch”) by making `--explain` serialization deterministic *and* layer-precedence ordered in `crates/shell/src/execution/config_model.rs`.

- Local gates run and passing on HEAD `4661a3658062dd5a9b4fb4e7ef34ee5f166ec7c2`: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, targeted tests, `make integ-checks`.
- CI compile parity (this HEAD): run `21055341242` `https://github.com/atomize-hq/substrate/actions/runs/21055341242` (success).
- CI behavioral smoke (WCU3, this HEAD): run `21055388858` `https://github.com/atomize-hq/substrate/actions/runs/21055388858` (success; linux/macos/windows).

- Completed end checklist: `make triad-task-finish TASK_ID="WCU3-integ-core"`.