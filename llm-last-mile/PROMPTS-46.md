# PROMPTS-46: Packet Orchestration Prompts For Slice 46

Source spec: [SPEC-46-internal-retained-host-progress-ack-bootstrap.md](./SPEC-46-internal-retained-host-progress-ack-bootstrap.md)  
Source plan: [PLAN-46.md](./PLAN-46.md)  
Source tasks: [TASKS-46.md](./TASKS-46.md)  
Current branch at prompt authoring time: `feat/internal-host-orchestrator-world-dispatch-bootstrap`  
Worker implementation skill: `/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill: `/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is grounded only in the live Slice `46` spec/plan/tasks stack and current repo truth. Do not use any `ORCH_PLAN*` file as a reference when running them.

## Packet 1 Prompt

```text
/goal Land Slice 46 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-46.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-46.md

Mission:
- Land Packet 1 only: Typed Progress Ack Contract And Policy Surface.
- Do not start Packet 2.
- Keep the slice bounded to adding the typed host `progress_ack` payload on `continue_world_worker` and the dedicated deny-by-default host progress-ack policy gate.

Before editing:
1. Read SPEC-46, PLAN-46, TASKS-46, then inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/policy_model.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/broker/src/policy.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/broker/src/effective_policy.rs
2. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
3. Run GitNexus impact analysis before editing any production symbol you change.
4. Stay strictly within Packet 1 scope.

Packet 1 scope:
- Task 1.1: Add the typed host `progress_ack` payload contract.
- Task 1.2: Add the dedicated deny-by-default host progress-ack policy gate.

Out of scope:
- Packet 2, 3, or 4 work
- deterministic rendering changes beyond what is strictly required to admit the Packet 1 payload shape
- delivery-summary wording beyond what is strictly required for the Packet 1 gate and payload
- durable progress review
- `control_directive`, `fork_command`, or worker-event widening
- active-ephemeral identity work
- transport schema widening in `transport-api-types` or `world-service`
- Family 2 work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- After implementation, run the Packet 1 verification commands:
  - `cargo test -p shell dispatch_contract -- --nocapture`
  - `cargo test -p shell policy_model -- --nocapture`
  - `cargo test -p substrate-broker -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-46 / PLAN-46 / TASKS-46 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 1 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 1 checkpoint:
- typed host `progress_ack` is a valid `continue_world_worker` payload
- the new host progress-ack gate is deny-by-default
- explanation-ready denials exist for disallowed typed host `progress_ack`
- the slice still remains delivery-only and non-durable

Implementation subagent prompt:
/goal Land Slice 46 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-46.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-46.md first. Work only on Task 1.1 and Task 1.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/agent_runtime/dispatch_contract.rs, crates/shell/src/execution/policy_model.rs, crates/broker/src/policy.rs, and crates/broker/src/effective_policy.rs. Keep Packet 2 through Packet 4 work out of scope. Run cargo test -p shell dispatch_contract -- --nocapture, cargo test -p shell policy_model -- --nocapture, and cargo test -p substrate-broker -- --nocapture. Final message must state whether Packet 1 is checkpoint-green, what symbols changed, what verification ran, whether Packet 2 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 46 Packet 1 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-46.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-46.md. Review only Packet 1 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 46 Packet 1 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-46.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-46.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 1 issues without widening scope. Re-run the relevant Packet 1 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 46 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-46.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-46.md

Mission:
- Land Packet 2 only: Deterministic Rendering And Delivery-Only Summary Semantics.
- Do not start Packet 3.
- Keep the slice bounded to deterministic typed host `progress_ack` rendering and acknowledgement-only delivery summaries.

Before editing:
1. Read SPEC-46, PLAN-46, TASKS-46, then inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs
2. Verify Packet 1 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 2 scope.

Packet 2 scope:
- Task 2.1: Render typed host `progress_ack` deterministically over the retained member-turn seam.
- Task 2.2: Keep live delivery summaries strictly acknowledgement-only.

Out of scope:
- Packet 3 or 4 work
- durable progress-review or inbox behavior
- new worker event classes or causation surfaces
- `control_directive`, `fork_command`, or `control_ack` changes
- active-ephemeral identity work
- transport schema widening in `transport-api-types` or `world-service`
- Family 2 work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1 and Task 2.2.
- After implementation, run the Packet 2 verification commands:
  - `cargo test -p shell dispatch_contract -- --nocapture`
  - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-46 / PLAN-46 / TASKS-46 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 2 checkpoint:
- accepted typed host `progress_ack` messages render deterministically
- exact retained-worker targeting remains mandatory
- successful summaries stay explicit that acknowledgement is delivery-only

Implementation subagent prompt:
/goal Land Slice 46 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-46.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-46.md first. Work only on Task 2.1 and Task 2.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/agent_runtime/dispatch_contract.rs and crates/shell/src/execution/orchestrator_world_dispatch.rs. Keep Packet 3 and Packet 4 work out of scope, and keep summaries acknowledgement-only. Run cargo test -p shell dispatch_contract -- --nocapture and cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture. Final message must state whether Packet 2 is checkpoint-green, what symbols changed, what verification ran, whether Packet 3 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 46 Packet 2 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-46.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-46.md. Review only Packet 2 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 46 Packet 2 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-46.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-46.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether Packet 3 is unblocked.
- If anything is not green, say explicitly that Packet 3 must not begin.
```

## Packet 3 Prompt

```text
/goal Land Slice 46 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-46.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-46.md

Mission:
- Land Packet 3 only: Regression Coverage And No-Durable-Side-Effect Proof.
- Do not start Packet 4.
- Keep the slice bounded to proving typed host `progress_ack` stays non-durable and does not widen existing progress/control/fork behavior.

Before editing:
1. Read SPEC-46, PLAN-46, TASKS-46, then inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs
2. Verify Packet 2 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 3 scope.

Packet 3 scope:
- Task 3.1: Prove typed host `progress_ack` stays non-durable.
- Task 3.2: Preserve bounded semantics for existing progress/control/fork flows.

Out of scope:
- Packet 4 work
- durable progress review or obligation-closeout models
- new host-message envelope or explicit event-id causation redesign
- `control_directive`, `fork_command`, or active-ephemeral widening
- Family 2 work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- After implementation, run the Packet 3 verification commands:
  - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-46 / PLAN-46 / TASKS-46 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 3 checkpoint:
- allowed typed host `progress_ack` delivery leaves no durable obligation side effects
- `progress_update` remains non-attention-driving by default
- summaries remain honest about acknowledgement-only delivery
- active-ephemeral identity and Family-2 work remain out of scope

Implementation subagent prompt:
/goal Land Slice 46 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-46.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-46.md first. Work only on Task 3.1 and Task 3.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/orchestrator_world_dispatch.rs and directly adjacent shell tests. Keep progress acknowledgement non-durable and preserve bounded progress/control/fork behavior. Run cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture and cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture. Final message must state whether Packet 3 is checkpoint-green, what symbols changed, what verification ran, whether Packet 4 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 46 Packet 3 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-46.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-46.md. Review only Packet 3 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 46 Packet 3 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-46.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-46.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 3 issues without widening scope. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 3 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether Packet 4 is unblocked.
- If anything is not green, say explicitly that Packet 4 must not begin.
```

## Packet 4 Prompt

```text
/goal Land Slice 46 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-46.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-46.md

Mission:
- Land Packet 4 only: Docs Alignment And Final Validation.
- This packet assumes Packets 1 through 3 are already landed and checkpoint-green.
- Keep the slice bounded to docs/config truth and the final validation wall.

Before editing:
1. Read SPEC-46, PLAN-46, TASKS-46, then inspect the live docs and config surfaces in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-46.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-46.md
2. Verify Packet 3 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 4 scope.

Packet 4 scope:
- Task 4.1: Align planning and config truth without widening the slice.
- Task 4.2: Run the final validation wall.

Out of scope:
- reopening Packets 1 through 3 unless validation proves a bounded in-scope follow-up is required
- durable progress review or inbox projection
- active-ephemeral identity work
- transport redesign
- Family 2 work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 4.1 and Task 4.2.
- After implementation, run the Packet 4 verification commands:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p shell dispatch_contract -- --nocapture`
  - `cargo test -p shell policy_model -- --nocapture`
  - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - `cargo test -p substrate-broker -- --nocapture`
  - `cargo test --workspace -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 4 docs/validation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-46 / PLAN-46 / TASKS-46 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 4 checkpoint:
- the Slice 46 surface is safely bounded
- config/docs truth is honest
- the validation wall is green

Implementation subagent prompt:
/goal Land Slice 46 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-46.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-46.md first. Work only on Task 4.1 and Task 4.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Align docs and config truth without widening scope, then run the full Packet 4 validation wall. If a validation failure requires more code or doc changes, keep any follow-up bounded to explicit Slice 46 surfaces only. Run cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test -p shell dispatch_contract -- --nocapture, cargo test -p shell policy_model -- --nocapture, cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture, cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture, cargo test -p substrate-broker -- --nocapture, and cargo test --workspace -- --nocapture. Final message must state whether Packet 4 is checkpoint-green, what symbols changed, what verification ran, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 46 Packet 4 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-46.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-46.md. Review only Packet 4 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 46 Packet 4 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-46.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-46.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 4 issues without widening scope. Re-run the relevant Packet 4 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether Slice 46 is fully review-clean.
- If anything is not green, say explicitly that Slice 46 is not ready to close out.
```
