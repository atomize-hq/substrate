# A1.3 P0 scope-expansion evidence

## Bound Git identity

Repository path:

```text
/home/spenser/__Active_code/substrate
```

Rebind observed before creating this WIP branch:

```text
branch=feat/internal-host-orchestrator-world-dispatch-bootstrap
HEAD=247363975635d791654099c109401fcb845523c1
tree=6aa2f2892bc0d852738a6a3d0e6ee459d191a23a
parent=1ea68308bb79e387fcb874e647ba183d6da7cc4d
upstream=origin/feat/internal-host-orchestrator-world-dispatch-bootstrap
ahead=0 behind=0
```

The WIP branch was created from that HEAD:

```text
feat/a1-3-p0-scope-expansion-evidence
```

Complete `git status --short` before this evidence file was created:

```text
 M AGENTS.md
 M CLAUDE.md
 M crates/shell/src/execution/agents_cmd.rs
 M crates/shell/src/repl/async_repl.rs
 M crates/shell/tests/agent_public_control_surface_v1.rs
 M crates/shell/tests/common.rs
```

Changed-path inventory before this evidence file was created:

```text
AGENTS.md
CLAUDE.md
crates/shell/src/execution/agents_cmd.rs
crates/shell/src/repl/async_repl.rs
crates/shell/tests/agent_public_control_surface_v1.rs
crates/shell/tests/common.rs
```

The implementation/test paths are:

```text
crates/shell/src/repl/async_repl.rs
crates/shell/src/execution/agents_cmd.rs
crates/shell/tests/agent_public_control_surface_v1.rs
crates/shell/tests/common.rs
```

The generated paths excluded from the commit are:

```text
AGENTS.md
CLAUDE.md
```

## Diff stat and hunk inventory

`git diff --stat` for the four implementation/test files:

```text
 crates/shell/src/execution/agents_cmd.rs           |  20 +-
 crates/shell/src/repl/async_repl.rs                | 244 +++++++++++++++++++++
 .../shell/tests/agent_public_control_surface_v1.rs |  55 ++++-
 crates/shell/tests/common.rs                       |   6 +-
 4 files changed, 316 insertions(+), 9 deletions(-)
```

`git diff --numstat` for the four implementation/test files:

```text
18	2	crates/shell/src/execution/agents_cmd.rs
244	0	crates/shell/src/repl/async_repl.rs
51	4	crates/shell/tests/agent_public_control_surface_v1.rs
3	3	crates/shell/tests/common.rs
```

Observed per-file hunk headers:

```text
diff --git a/crates/shell/src/execution/agents_cmd.rs b/crates/shell/src/execution/agents_cmd.rs
@@ -145,7 +145,12 @@ pub(crate) fn handle_agent_command(
@@ -339,7 +344,11 @@ struct HostStartLaunchPlan {
@@ -415,6 +424,13 @@ fn run_start(args: &AgentStartArgs, cli: &Cli) -> Result<()> {
diff --git a/crates/shell/src/repl/async_repl.rs b/crates/shell/src/repl/async_repl.rs
@@ -3114,6 +3114,250 @@ fn apply_greenfield_host_start_from_authority(
diff --git a/crates/shell/tests/agent_public_control_surface_v1.rs b/crates/shell/tests/agent_public_control_surface_v1.rs
@@ -10,7 +10,7 @@ use std::collections::BTreeMap;
@@ -22,7 +22,6 @@ use std::time::{Duration, Instant};
@@ -77,9 +76,19 @@ impl AgentControlFixture {
@@ -113,7 +122,37 @@ impl AgentControlFixture {
@@ -4873,6 +4912,14 @@ fn public_start_persists_detached_session_when_hidden_owner_helper_exits() {
diff --git a/crates/shell/tests/common.rs b/crates/shell/tests/common.rs
@@ -111,10 +111,10 @@ pub fn ensure_substrate_built() {
```

## First failing focused test

Command:

```text
cargo test -q -p shell --test agent_public_control_surface_v1 public_start_persists_detached_session_when_hidden_owner_helper_exits -- --exact --nocapture
```

Failure output:

```text
thread 'public_start_persists_detached_session_when_hidden_owner_helper_exits' panicked at crates/shell/tests/agent_public_control_surface_v1.rs:4871:5:
public start should succeed even if the helper exits immediately after the startup prompt: Output { status: ExitStatus(unix_wait_status(512)), stdout: "", stderr: "runtime_start_failed: timed out waiting for authoritative owner-helper readiness for orchestration session 01a0203c-dc68-7f63-8cba-79c262254339\\n" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
public_start_persists_detached_session_when_hidden_owner_helper_exits --- FAILED

failures:

failures:
    public_start_persists_detached_session_when_hidden_owner_helper_exits

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 43 filtered out; finished in 42.79s

error: test failed, to rerun pass `-p shell --test agent_public_control_surface_v1`
```

## Scope-expansion reason

```text
SCOPE_EXPANSION: current A1.3 fence excludes control.rs.
```

## Read-only source-call evidence

`crates/shell/src/execution/agents_cmd.rs` contains public helper launch calls at these observed locations:

```text
run_start:    line 445  launch_hidden_owner_helper(&plan, cli.world, cli.no_world)
run_start:    line 480  launch_hidden_owner_helper(&plan, cli.world, cli.no_world)
run_turn:     line 718  launch_hidden_owner_helper(&plan, cli.world, cli.no_world)
run_reattach: line 801  launch_hidden_owner_helper(&plan, cli.world, cli.no_world)
```

`crates/shell/src/execution/agent_runtime/control.rs` contains these observed launch/readiness operations:

```rust
let store = AgentRuntimeStateStore::new()?;
let plan_path = persist_hidden_owner_helper_launch_plan(&store, plan)?;
let mut child = command.spawn().with_context(|| {
    format!(
        "failed to spawn hidden owner-helper for orchestration session {}",
        plan.session.orchestration_session_id
    )
})?;
if let Err(err) = wait_for_hidden_owner_helper_readiness(&store, plan) {
    let reconciled = if plan.mode == OwnerHelperMode::Start
        && hidden_owner_helper_readiness_timed_out(&err)
    {
        Some(reconcile_hidden_owner_helper_start_timeout(&store, plan))
    } else {
        None
    };
```

The same file contains this observed readiness loop:

```rust
let readiness = store.classify_hidden_owner_helper_launch_readiness(
    plan.orchestration_session_id(),
    plan.participant_id(),
    plan.requires_internal_session_id(),
)?;
let startup_prompt_ready = if plan.startup_prompt.is_some() {
    start_launch_startup_prompt_is_accepted_or_terminal(store, plan)?
} else {
    true
};
if startup_prompt_ready
    && (readiness == super::state_store::HiddenOwnerHelperLaunchReadiness::ReadyAttached
        || (matches!(plan.mode, OwnerHelperMode::Start | OwnerHelperMode::ResumeOneTurn)
            && matches!(readiness, super::state_store::HiddenOwnerHelperLaunchReadiness::ReadyDetached(_))))
{
    return Ok(());
}
if started_at.elapsed() >= OWNER_HELPER_READY_TIMEOUT {
    anyhow::bail!("{}{}", OWNER_HELPER_READY_TIMEOUT_ERROR_PREFIX, plan.orchestration_session_id());
}
```

`crates/shell/src/execution/agents_cmd.rs` routes the hidden helper to the REPL:

```rust
let plan = load_hidden_owner_helper_launch_plan(&args.plan_file)?;
remove_hidden_owner_helper_launch_plan(&args.plan_file)?;
crate::repl::async_repl::run_hidden_owner_helper(
    plan,
    install_context.context.intended_host_principal.clone(),
)
```

`crates/shell/src/repl/async_repl.rs` contains this observed owner-helper session persistence:

```rust
let manifest = owner_helper_manifest(&descriptor, plan)?;
let orchestration_session = Arc::new(Mutex::new(owner_helper_orchestration_session(
    plan, &manifest,
)));
state_store
    .persist_orchestration_session(
        &orchestration_session
            .lock()
            .expect("hidden owner-helper orchestration session mutex poisoned"),
    )?;
```

The same file contains this observed participant persistence in the non-authority-managed branch:

```rust
if !authority_managed {
    let persist_participant_result = {
        let manifest_guard = manifest.lock().expect("runtime manifest mutex poisoned");
        startup_context.store.persist_participant(&manifest_guard)
    };
```

`crates/shell/src/execution/agent_runtime/auto_attach.rs` contains this separate caller:

```text
line 362: let receipt = match launch_hidden_owner_helper(&plan, world, no_world) {
```

## GitNexus impact results

Observed upstream impact summaries:

```text
handle_agent_command: LOW; direct=0; processes_affected=0; modules_affected=0
run_start: LOW; impactedCount=1; direct=1; processes_affected=1; modules_affected=1; direct caller=handle_agent_command
run_turn: LOW; impactedCount=1; direct=1; processes_affected=1; modules_affected=1; direct caller=handle_agent_command
run_reattach: LOW; impactedCount=1; direct=1; processes_affected=1; modules_affected=1; direct caller=handle_agent_command
run_owner_helper: LOW; direct=0; processes_affected=0; modules_affected=0
build_attach_launch_plan_with_store: LOW; impactedCount=7; direct=3; processes_affected=1; modules_affected=2
public_start_persists_detached_session_when_hidden_owner_helper_exits: LOW; direct=0; processes_affected=0; modules_affected=0
AgentControlFixture::new_with_fake_codex: MEDIUM; direct=8
AgentControlFixture::command: LOW; direct=4
ensure_substrate_built: LOW; direct=1; process=substrate_shell_driver
```

Observed unindexed/UNKNOWN results from `npx gitnexus impact <name> --direction upstream --repo substrate-current`:

```text
owner_helper_shell_config: Target not found; impactedCount=0; risk=UNKNOWN
resolve_host_orchestrator_bootstrap: Target not found; impactedCount=0; risk=UNKNOWN
prepare_hidden_owner_helper_runtime: Target not found; impactedCount=0; risk=UNKNOWN
run_hidden_owner_helper: Target not found; impactedCount=0; risk=UNKNOWN
ensure_substrate_built: ambiguous; impactedCount=0; risk=UNKNOWN; candidates were crates/broker/src/tests.rs and crates/shell/tests/common.rs
```

GitNexus change detection after this evidence file was created:

```text
Changes: 6 files, 11 symbols
Affected processes: 10
Risk level: high
```

The GitNexus output listed the six tracked modified paths and did not list this untracked evidence path.

GitNexus change detection after the five requested paths were staged:

```text
Changes: 2 files, 2 symbols
Affected processes: 0
Risk level: low

Changed symbols:
  undefined GitNexus — Code Intelligence → AGENTS.md
  undefined GitNexus — Code Intelligence → CLAUDE.md
```

## Snapshot statements

- No `crates/shell/src/execution/agent_runtime/control.rs` code was changed.
- No A1.3 success or completion claim is made.
- This snapshot is evidence for a later docs-only A1.3-P0 authority amendment.
- The partial implementation is not landable under the current packet.
