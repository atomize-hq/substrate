# PROMPTS-59: Packet Orchestration Prompts For Slice 59

Source spec: [SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md](./SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md)  
Source plan: [PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md](./PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md)  
Source tasks: [TASKS-59.md](./TASKS-59.md)  
Current branch at prompt authoring time: `feat/internal-host-orchestrator-world-dispatch-bootstrap`  
Worker implementation skill: `/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill: `/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is grounded only in the live Slice `59` spec/plan/tasks stack and current repo truth. Do not use any `ORCH_PLAN*` file as authority when running them.

## Packet 1 Prompt

```text
/goal Land Slice 59 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md

Mission:
- Land Slice 59 Packet 1 only: Fail-Closed Runtime Truth And Remediation.
- Do not start Packet 2.
- Keep the slice bounded to world-scoped runtime-realizability gating, world-vs-host truth separation, and stable remediation diagnostics.

Before editing:
1. Read SPEC-59, PLAN-59, and TASKS-59 first.
2. Inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs
   - the narrowest adjacent Packet 1 test seams under /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
5. Stay strictly within Packet 1 scope.

Packet 1 scope:
- Task 1.1: Add world-scoped runtime-realizability gating before bootstrap.
- Task 1.2: Emit stable remediation diagnostics for missing guest runtime truth.

Out of scope:
- Packet 2, 3, or 4 work
- world-deps package delivery
- UAA dependency bumping or runtime version-resolution integration
- installer or docs provisioning flags
- Slice 58 placement-aware selector/config migration
- any host-runtime fallback that treats host `which codex` as authoritative world truth

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- After implementation, run the Packet 1 verification commands:
  - `cargo test -p shell agent_runtime::validator -- --nocapture`
  - `cargo test -p shell dispatch_contract -- --nocapture`
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-59 / PLAN-59 / TASKS-59 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 1 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 1 checkpoint:
- world-scoped Codex no longer claims launchability from host `which` success alone
- validator/materialization fails before retained worker bootstrap when guest runtime is absent
- remediation guidance is explicit
- host-scoped Codex behavior remains unchanged

Implementation subagent prompt:
/goal Land Slice 59 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md first. Work only on Task 1.1 and Task 1.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/agent_runtime/validator.rs, crates/shell/src/execution/orchestrator_world_dispatch.rs, and adjacent Packet 1 tests, touching crates/world-service/src/member_runtime.rs only if bounded defense-in-depth or wording alignment is truly required by Packet 1. Keep Packet 2 through Packet 4 work out of scope, and do not land world-deps package delivery, installer surfaces, Slice 58 selector/config migration, or host-runtime fallback behavior. Run cargo test -p shell agent_runtime::validator -- --nocapture, cargo test -p shell dispatch_contract -- --nocapture, and cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture. Final message must state whether Packet 1 is checkpoint-green, what symbols changed, what verification ran, whether Packet 2 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 59 Packet 1 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md. Review only Packet 1 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 59 Packet 1 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 1 issues without widening scope. Re-run the relevant Packet 1 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 59 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md

Mission:
- Land Slice 59 Packet 2 only: Codex Guest Runtime Package Or Runtime Bundle.
- Do not start Packet 2.5.
- Keep the slice bounded to the `codex-runtime` world-deps package, the published `unified-agent-api = "=0.3.7"` dependency alignment, UAA-backed validated version selection, and explicit proof of self-contained-vs-bundle runtime posture.

Before editing:
1. Read SPEC-59, PLAN-59, and TASKS-59 first.
2. Verify Packet 1 is already landed and checkpoint-green on the current tree.
3. Inspect the live code and package seams in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_deps/
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/Cargo.toml
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/Cargo.toml
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/Cargo.toml
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Cargo.lock
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/deps/
4. Inspect the UAA public runtime-support authority in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/CHANGELOG.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/agent_api/src/lib.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/agent_api/src/runtime_support.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/runtime-support-contract.md
5. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
6. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
7. Stay strictly within Packet 2 scope.

Packet 2 scope:
- Task 2.1: Author the Codex world-deps package/bundle and install script.
- Task 2.2: Align the published UAA dependency wiring to `0.3.7`.
- Task 2.3: Integrate UAA-backed validated version selection and record the verified runtime dependency posture.

Out of scope:
- Packet 3 or 4 work
- installer-time provisioning flags and install-surface help/docs
- Slice 58 placement-aware selector/config migration
- a new generic artifact transport/checksum/archive system
- host-runtime fallback or floating “latest” Codex install logic

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1, Task 2.2, and Task 2.3.
- After implementation, run the Packet 2 verification commands:
  - `cargo test -p shell world_deps -- --nocapture`
  - `rg -n 'unified-agent-api.*0\\.3\\.7|unified-agent-api-codex.*0\\.3\\.7|unified-agent-api-claude-code.*0\\.3\\.7' /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/Cargo.toml /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/Cargo.toml /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/Cargo.toml /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Cargo.lock`
  - targeted tests for the UAA-backed version-resolution path if added
  - the packet-specific guest smoke proof command(s) that prove the runtime posture
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-59 / PLAN-59 / TASKS-59 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 2 checkpoint:
- the guest-visible `codex` entrypoint resolves from `/var/lib/substrate/world-deps/bin`
- package installs are idempotent
- the verified self-contained-vs-bundle outcome is explicit
- the path does not rely on host NVM/npm state
- version selection comes from the published `unified-agent-api = "=0.3.7"` Rust API rather than downstream duplicated logic

Implementation subagent prompt:
/goal Land Slice 59 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md first. Work only on Task 2.1, Task 2.2, and Task 2.3. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in the world-deps package/install seams, crates/shell/Cargo.toml, crates/gateway/Cargo.toml, crates/world-service/Cargo.toml, Cargo.lock, and the narrowest Substrate runtime-selection code that must call `agent_api::resolve_runtime_support(\"codex\", target_triple)`. Keep Packet 2.5 through Packet 4 work out of scope, and do not land installer flag surfaces, Slice 58 migration, host-runtime fallback, or floating latest-release resolution. Run cargo test -p shell world_deps -- --nocapture, the exact rg verification for the 0.3.7 dependency pins, and any targeted UAA-backed runtime-resolution tests you add, plus the guest smoke proof needed to record whether Codex is self-contained or requires a wider bundle. Final message must state whether Packet 2 is checkpoint-green, what symbols changed, what verification ran, whether Packet 2.5 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 59 Packet 2 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md. Review only Packet 2 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 59 Packet 2 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether Packet 2.5 is unblocked.
- If anything is not green, say explicitly that Packet 2.5 must not begin.
```

## Packet 2.5 Prompt

```text
/goal Land Slice 59 Packet 2.5 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md

Mission:
- Land Slice 59 Packet 2.5 only: Guest-Tuple Fail-Closed And Host/World Separation Remediation.
- Do not start Packet 3.
- Keep the slice bounded to resolving the remaining Packet 2 review disagreement around guest-tuple truth gaps, including UAA-validated Linux guest tuples that are not yet fully mapped by Substrate, plus host-runtime leakage.

Before editing:
1. Read SPEC-59, PLAN-59, and TASKS-59 first.
2. Verify Packet 2 is already landed and verification-green on the current tree.
3. Inspect the live code and tests in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_deps/
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs
   - the narrowest adjacent Packet 2/2.5 tests under /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/
4. Reconfirm the current published UAA runtime-support authority for Codex before changing behavior that depends on supported guest tuples.
5. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
6. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
7. Stay strictly within Packet 2.5 scope.

Packet 2.5 scope:
- Task 2.5.1: Keep guest-target truth explicit and fail closed on unsupported or unmapped guest tuples.
- Task 2.5.2: Prove host Codex cannot satisfy world runtime truth and make the separation explicit.

Out of scope:
- Packet 3 or 4 work
- installer-time provisioning surfaces
- Slice 58 placement-aware selector/config migration
- editing UAA published support truth itself inside this packet unless a separately landed/published prerequisite is already available and this packet only needs to consume it honestly
- host-runtime fallback, silent target remapping, or any logic that treats host Codex as interchangeable with guest Codex

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.5.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.5.1 and Task 2.5.2.
- After implementation, run the Packet 2.5 verification commands:
  - `cargo test -p shell world_deps -- --nocapture`
  - `cargo test -p shell dispatch_contract -- --nocapture`
  - `cargo test -p shell agent_runtime::validator -- --nocapture`
  - targeted regression tests for unsupported or validated-but-unmapped guest tuples, guest-target derivation, and host-PATH leakage if added
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 2.5 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2.5 against SPEC-59 / PLAN-59 / TASKS-59 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2.5 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 2.5 checkpoint:
- unsupported or validated-but-unmapped guest tuples fail closed before Substrate claims the world runtime is installed or launchable
- host Codex presence on `PATH` does not make world Codex runtime truth pass
- host-scoped and world-scoped Codex runtime truth remain explicitly separate
- the remaining Packet 2 review disagreement is resolved without widening into installer work or Slice 58 migration

Implementation subagent prompt:
/goal Land Slice 59 Packet 2.5 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md first. Work only on Task 2.5.1 and Task 2.5.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed to make unsupported or validated-but-unmapped guest tuples fail closed with explicit guest-target truth and to prove host Codex cannot satisfy world Codex runtime truth. Keep Packet 3 and Packet 4 work out of scope, and do not land installer flag surfaces, Slice 58 migration, or silent target broadening through host runtime. Run cargo test -p shell world_deps -- --nocapture, cargo test -p shell dispatch_contract -- --nocapture, cargo test -p shell agent_runtime::validator -- --nocapture, and any targeted regression tests you add for unsupported or validated-but-unmapped guest tuples and host-PATH leakage. Final message must state whether Packet 2.5 is checkpoint-green, what symbols changed, what verification ran, whether Packet 3 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 59 Packet 2.5 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md. Review only Packet 2.5 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 2.5 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 59 Packet 2.5 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 2.5 issues without widening scope. Re-run the relevant Packet 2.5 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2.5 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 2.5 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether Packet 3 is unblocked.
- If anything is not green, say explicitly that Packet 3 must not begin.
```

## Packet 3 Prompt

```text
/goal Land Slice 59 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md

Mission:
- Land Slice 59 Packet 3 only: Prod/Dev Installer-Time Provisioning Support.
- Do not start Packet 4.
- Keep the slice bounded to the public `--provision-agent-runtime <runtime_family>` installer surface, with `codex` as the only implemented value in this slice, and to explicit install-then-sync operator truth.

Before editing:
1. Read SPEC-59, PLAN-59, and TASKS-59 first.
2. Verify Packet 2.5 is already landed and checkpoint-green on the current tree.
3. Inspect the live code and docs in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install.sh
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/world-enable.sh
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/INSTALLATION.md
4. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
5. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
6. Stay strictly within Packet 3 scope.

Packet 3 scope:
- Task 3.1: Add the prod installer/runtime-provisioning surface.
- Task 3.2: Add the dev installer/runtime-provisioning surface.

Out of scope:
- Packet 4 work
- new runtime families beyond `codex`
- Slice 58 placement-aware selector/config migration
- world-deps package authoring or UAA version-resolution work except narrow integration needed to call the already-landed Packet 2 path
- hidden host-runtime fallback behavior

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- After implementation, run the Packet 3 verification commands:
  - `bash /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh --help`
  - `bash /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install.sh --help`
  - `bash /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh --help`
  - inspect the resulting help/usage output for `--provision-agent-runtime <runtime_family>` and explicit install-then-sync wording
  - targeted script tests or dry-run validation if available
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-59 / PLAN-59 / TASKS-59 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 3 checkpoint:
- both prod and dev installers expose an explicit runtime-provisioning flag
- that flag is generic to runtime family even though Slice 59 only implements `codex`
- help/docs state that the flag provisions/install as needed and then runs sync
- installer UX does not imply host-runtime fallback
- operator truth is consistent across surfaces

Implementation subagent prompt:
/goal Land Slice 59 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md first. Work only on Task 3.1 and Task 3.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and docs changes needed in scripts/substrate/install-substrate.sh, scripts/substrate/install.sh, scripts/substrate/dev-install-substrate.sh, scripts/substrate/world-enable.sh, and docs/INSTALLATION.md so `--provision-agent-runtime <runtime_family>` is explicit, supports `codex` only in this slice, fails closed for unsupported values, and makes the install-then-`substrate world deps current sync` behavior impossible to miss. Keep Packet 4 and Slice 58 work out of scope. Run bash scripts/substrate/install-substrate.sh --help, bash scripts/substrate/install.sh --help, bash scripts/substrate/dev-install-substrate.sh --help, inspect the resulting help output for the new flag and explicit sync wording, and run any targeted script tests or dry-run validation you add. Final message must state whether Packet 3 is checkpoint-green, what symbols changed, what verification ran, whether Packet 4 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 59 Packet 3 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md. Review only Packet 3 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 59 Packet 3 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 3 issues without widening scope. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 59 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md

Mission:
- Land Slice 59 Packet 4 only: End-To-End Proof And Slice 58 Handoff Gate.
- This packet is the final validation and bounded closeout gate for Slice 59.
- Keep the slice bounded to proving runtime truth end-to-end, recording the final validation story, and stopping before any Slice 58 placement-aware migration begins.

Before editing:
1. Read SPEC-59, PLAN-59, and TASKS-59 first.
2. Verify Packet 3 is already landed and checkpoint-green on the current tree.
3. Inspect the live code and docs touched by Packets 1 through 3 plus the validation seams they rely on.
4. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
5. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
6. Stay strictly within Packet 4 scope.

Packet 4 scope:
- Task 4.1: Run the final validation wall and smoke proof.

Out of scope:
- Slice 58 implementation
- new runtime-feature work beyond bounded follow-up required to make the Packet 4 validation wall honest
- widening the slice into additional artifact systems, new runtime families, or selector/config redesign

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 4.1.
- After implementation, run the Packet 4 verification commands:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p shell agent_runtime::validator -- --nocapture`
  - `cargo test -p shell dispatch_contract -- --nocapture`
  - `cargo test -p shell world_deps -- --nocapture`
  - `cargo test -p world-service member_runtime -- --nocapture`
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - packet-specific guest smoke proof command(s) captured in the implementation session
- If implementation needs bounded follow-up edits to make the validation wall honest, keep them strictly inside explicit Slice 59 surfaces, then rerun the affected verification commands.
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 4 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-59 / PLAN-59 / TASKS-59 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 4 checkpoint:
- fake world launchability is gone
- guest runtime delivery is real and documented
- installer support is live on both surfaces
- Slice 58 can now change config/selector shape without reopening runtime semantics

Implementation subagent prompt:
/goal Land Slice 59 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md first. Work only on Task 4.1. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Run the full Packet 4 validation wall and guest smoke proof. If any validation failure requires additional code or doc changes, keep the follow-up strictly bounded to explicit Slice 59 surfaces only and do not begin Slice 58 work. Run cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test -p shell agent_runtime::validator -- --nocapture, cargo test -p shell dispatch_contract -- --nocapture, cargo test -p shell world_deps -- --nocapture, cargo test -p world-service member_runtime -- --nocapture, cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture, and the packet-specific guest smoke proof command(s). Final message must state whether Packet 4 is checkpoint-green, what symbols changed, what verification ran, whether any reopen condition was discovered, and whether Slice 59 is ready for final closeout and Slice 58 handoff.

Review subagent prompt:
Review the committed Slice 59 Packet 4 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md. Review only Packet 4 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 59 Packet 4 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 4 issues without widening scope. Re-run the relevant Packet 4 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether Slice 59 is ready for final closeout and Slice 58 handoff.
- If anything is not green, say explicitly that Slice 58 must not begin.
```
