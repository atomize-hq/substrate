# TASKS-31.25: Prompt-Free Fresh Attach Remediation

Source spec: [SPEC-31_25.md](./SPEC-31_25.md)  
Source plan: [PLAN-31_25.md](./PLAN-31_25.md)  
Primary gap context: [TASKS-31.md](./TASKS-31.md), [CLOSEOUT-31-packet-4.md](./CLOSEOUT-31-packet-4.md)  
Phase: `TASKS`  
Execution model: three separate `/incremental-implementation` sessions  
Status: draft for review on 2026-05-29

## Execution Packets

This remediation should be implemented as three separate sessions.

- Packet 1 removes the forbidden prompt-free fresh attach runtime path.
- Packet 2 preserves persisted-truth planning and fail-closed attach outcomes.
- Packet 3 corrects tests, docs, and closeout truth and runs the validation wall.

Do not start the next packet until the current packet checkpoint is green.

## Packet 1: Remove Prompt-Free Fresh Attach Runtime Fallback

Session goal:

1. make control-only attach continuity-only in active runtime paths for this slice;
2. remove backend helpers that execute prompt-free fresh attach;
3. replace fallback with explicit fail-closed behavior.

### Tasks

- [ ] Task 1.1: Stop control-only attach startup from falling through to fresh attach execution
  - Acceptance: when continuity session truth is absent, control-only attach startup fails closed before backend launch instead of calling a fresh attach helper; continuity-backed attach behavior remains unchanged.
  - Verify: `cargo test -p shell --lib reattach -- --nocapture`
  - Files:
    - [`crates/shell/src/repl/async_repl.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
    - [`crates/shell/src/execution/agents_cmd.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

- [ ] Task 1.2: Remove prompt-free fresh attach helpers from backend adapter plumbing
  - Acceptance: there is no active `run_fresh_attach_control`-style helper for Codex or Claude in the runtime path; adapter code only exposes sanctioned prompt-bearing launch and continuity-backed attach behavior.
  - Verify: `cargo test -p shell prompt_fulfillment -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/prompt_fulfillment.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/prompt_fulfillment.rs)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. prompt-free fresh attach is no longer executable through the active control-only runtime path;
2. continuity-backed attach still works;
3. fail-closed diagnostics replace fallback behavior.

## Packet 2: Preserve Planning And Fail Closed On Fresh-Needed Cases

Session goal:

1. keep attach planning rooted in persisted `HostAttachContract` truth;
2. explicitly classify fresh-needed cases as unsupported in this slice;
3. keep manual and automatic attach outcomes coherent.

### Tasks

- [ ] Task 2.1: Preserve continuity-first planning while rejecting fresh-needed execution
  - Acceptance: automatic attach and manual attach planning still resolve from persisted truth; continuity-available cases proceed; continuity-missing/fresh-needed cases fail closed before backend launch with stable explanation-ready diagnostics.
  - Verify: `cargo test -p shell --lib auto_attach -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs)
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

- [ ] Task 2.2: Preserve manual/automatic attach interop under fail-closed fresh-needed outcomes
  - Acceptance: manual `reattach` and automatic attach do not duplicate launches; fresh-needed cases settle or persist outcome state coherently without implying attached ownership; continuity-backed success cases remain unchanged.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agents_cmd.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
    - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs)
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. persisted-truth planning remains intact;
2. fresh-needed cases fail closed before backend launch;
3. manual and automatic attach continue to share one coherent authority model.

## Packet 3: Correct Tests, Docs, And Closeout Truth

Session goal:

1. remove tests that normalize forbidden fresh attach behavior;
2. write corrective slice-specific docs and closeout truth;
3. run the final validation wall.

### Tasks

- [ ] Task 3.1: Replace fresh-attach-without-continuity regression expectations with fail-closed expectations
  - Acceptance: tests no longer assert that fresh attach without continuity is valid startup behavior; regression coverage instead asserts explicit fail-closed behavior and preserved continuity-backed attach.
  - Verify:
    - `cargo test -p shell --lib hidden_owner_helper_attach_startup_allows_fresh_attach_without_continuity -- --nocapture`
    - `cargo test -p shell --lib auto_attach_launch_plan_falls_back_to_fresh_when_continuity_is_unavailable -- --nocapture`
  - Files:
    - [`crates/shell/src/repl/async_repl.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
    - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs)

- [ ] Task 3.2: Write remediation-specific docs and closeout language
  - Acceptance: slice `31.25` artifacts clearly state the restored fail-closed contract and the deferred status of any future sanctioned fresh attach mechanism; the corrective record stands on its own without rewriting slice-31 planning docs.
  - Verify: manual diff review plus `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Files:
    - [`llm-last-mile/SPEC-31_25.md`](/home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-31_25.md)
    - [`llm-last-mile/PLAN-31_25.md`](/home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-31_25.md)
    - [`llm-last-mile/TASKS-31_25.md`](/home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-31_25.md)
    - `llm-last-mile/CLOSEOUT-31_25-*.md`

- [ ] Task 3.3: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell suites, and full workspace tests pass after the remediation; closeout wording matches the observed green wall.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Files:
    - No planned source edits; this is the validation gate after Tasks 3.1 and 3.2.

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. tests pin the restored fail-closed contract;
2. the corrective slice carries the corrected contract and deferred-scope truth;
3. the validation wall is green.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.

## Notes For Implementation

- Packet 1 should be a minimal runtime correction. Do not broaden into planner or closeout cleanup there.
- Packet 2 should preserve the good Packet 4 work while making fresh-needed behavior truthful and fail closed.
- Packet 3 is where the corrective written story gets recorded. Keep it self-contained in slice `31.25` rather than turning this into a rewrite of slice `31`.
