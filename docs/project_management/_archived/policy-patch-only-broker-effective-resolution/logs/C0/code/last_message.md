- Implemented broker-canonical effective policy resolution as patch-merge `defaults → global_patch ($SUBSTRATE_HOME/policy.yaml) → workspace_patch (<workspace>/.substrate/policy.yaml)`, with workspace discovery honoring `.substrate/workspace.disabled`.
- Updated `substrate policy current show` / `--explain` to delegate effective policy + explain to the broker; `--explain` now emits only the explain JSON on stderr (no extra note line).
- Updated broker “save approval to policy” path to write patch-only YAML (no full-policy docs); made policy `metadata` serialization deterministic.

**Baseline tests (pre/post)**
- Pre: `cargo test -p substrate-broker -- --nocapture` → ok (40 passed; includes expected “poison lock” panics in test output), `cargo test -p substrate-shell --test policy_discovery -- --nocapture` → ok (3 passed)
- Post: same commands → same outcomes

**Required checks**
- `cargo fmt` → ok
- `cargo clippy --workspace --all-targets -- -D warnings` → ok

**Task finish**
- Ran `make triad-task-finish TASK_ID="C0-code"` → ok (HEAD `29a5755d`, 2 commits)