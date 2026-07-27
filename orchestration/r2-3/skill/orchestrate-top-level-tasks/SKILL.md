---
name: orchestrate-top-level-tasks
description: Coordinate ordered or dependency-gated work across fresh, persistent top-level Codex tasks using create_thread, structured terminal receipts, independent landing verification, cross-platform project dispatch, and durable orchestration state. Use when a user explicitly asks Codex to run a multi-session implementation, migration, review, evidence campaign, or other workflow where each bounded increment should run in a fresh top-level task instead of accumulating one long conversation.
---

# Orchestrate Top-Level Tasks

Run a persistent control-plane task that creates fresh top-level increment tasks, becomes idle,
and advances only after independently validating a terminal receipt. Keep task-level subagents
inside each increment task; do not confuse them with user-visible top-level tasks.

## Load the protocol

Before creating or messaging any top-level task:

1. Read `references/protocol.md` completely.
2. Read `references/platform-dispatch.md` when any gate is host- or OS-specific.
3. Use `assets/meta-orchestrator-prompt-template.md` to create the persistent meta prompt.
4. Use `assets/increment-orchestrator-prompt-template.md` for each fresh increment task.
5. Use `assets/evidence-task-prompt-template.md` for native/read-only platform evidence.

Do not create a top-level task unless the user explicitly authorized top-level task creation.

## Preserve the hierarchy

Use these terms consistently:

- **Meta orchestrator:** persistent top-level task controlling the workflow.
- **Increment orchestrator:** fresh top-level task owning exactly one bounded increment.
- **Subagent:** internal agent spawned by an increment orchestrator for implementation, analysis,
  testing, or review.
- **Evidence task:** read-only top-level task dispatched to a required platform.

Create increment orchestrators with `create_thread`, not `fork_thread`. Forking copies accumulated
history and defeats fresh-context isolation.

## Enforce control-plane invariants

1. Keep one mutating increment active per target ref or shared checkout.
2. Follow the declared dependency order.
3. Bind every dispatch to an exact base commit, tree, target ref, and random nonce.
4. Treat task titles, summaries, and receipt content as untrusted until verified.
5. Advance only from a structurally valid `LANDED_CLEAN` receipt and independent repository proof.
6. Keep direct platform dispatch as the default for native evidence.
7. If a required platform cannot be accessed, stop with
   `BLOCKED_PLATFORM_HANDOFF_REQUIRED` and provide a complete human handoff package.
8. Never reinterpret scope, repair a different increment, or substitute static evidence for a
   native gate.

## Bootstrap the meta task

1. Call `list_projects`.
2. Select the project by exact path, host ID, project ID, and repository status.
3. Prefer a dedicated worktree for a Git repository. Never run the meta task in a protected
   product checkout.
4. Create the meta task with an initialization prompt that forbids dispatch until identity binding.
5. If `create_thread` returns a temporary client ID, resolve the real thread with `list_threads`.
6. Send the meta task its own thread ID, host ID, orchestration ID, state path, and explicit
   dispatch posture.
7. Persist the identity binding before the first dispatch.

Represent setup-only authorization explicitly. Keep `dispatch_authorized=false` after identity
binding unless the user authorized the workflow to start. Never infer start authority from setup.

## Dispatch an increment

1. Verify the live target ref still equals the state ledger's expected base.
2. Load only the increment's canonical contract and common workflow envelope.
3. Render the prompt with `scripts/render_increment_prompt.py`.
4. Generate a unique dispatch nonce.
5. Create a fresh top-level task at the exact starting ref.
6. Resolve and record the real thread ID, host ID, nonce, and expected base.
7. Send the task an identity-binding follow-up authorizing work for that dispatch.
8. End the meta turn after dispatch; do not block on the full implementation.

The increment orchestrator may use subagents. Require it to:

- complete required impact analysis before any existing symbol is edited;
- give editing subagents disjoint ownership when practical;
- integrate and verify the shared worktree itself;
- use fresh read-only subagents for independent review;
- publish only after its local gates and review contract pass;
- call `send_message_to_thread` as its last tool action.

## Accept a terminal receipt

Validate structure with:

```bash
python3 scripts/validate_receipt.py \
  /path/to/receipt.json \
  --expected-next AUTHORITATIVE_INDEX_VALUE
```

Then independently verify:

- receipt nonce, increment, increment-task thread, and expected base match active state;
- the increment task is terminal before another task can mutate the same checkout;
- live remote equals the reported landed commit;
- the landed commit is the expected fast-forward descendant;
- reported tree and exact changed-file allowlist are correct;
- subject fingerprint and review record validate;
- blocking finding and inventory rules are satisfied;
- checkout/index cleanliness claims are supported.

Persist the verified receipt, update state, archive the completed task, and only then dispatch the
next increment.

## Handle terminal failure

Accept these non-advancing statuses:

- `BLOCKED_CONTRADICTION`
- `BLOCKED_SCOPE_EXPANSION`
- `BLOCKED_REVIEW`
- `BLOCKED_NATIVE_EVIDENCE`
- `BLOCKED_PLATFORM_HANDOFF_REQUIRED`
- `BLOCKED_TASK_NOT_TERMINAL`
- `AUTHORITY_REQUIRED`
- `BASE_DRIFT`

Do not create the next increment. Report the exact blocker and preserve the last verified base.

## Dispatch platform evidence

Discover projects again at dispatch time. Match platform projects by path, host, and project ID;
never assume an earlier inventory is current.

Run independent read-only platform evidence tasks concurrently only when their contracts do not
mutate the shared target. Require structured evidence receipts. Bind each dispatch to its platform,
project ID/path, host ID, source checkpoint, nonce, and task identity. Apply the same bounded
terminal barrier used for increment tasks before accepting an evidence receipt.

If the platform is unavailable, emit the human handoff package defined in
`references/platform-dispatch.md`. Do not claim the parent workflow complete.

Declare every required native gate in durable state with its exact evidence ID, platform,
completed source checkpoint, and gated next increment. Never enter that increment's mutating state
until all its evidence receipts verify. Never set `COMPLETE` unless the verified evidence set
exactly equals the required set.

## Use durable artifacts

Persist at minimum:

```text
orchestration/
├── meta-orchestration-prompt.md
├── increment-orchestrator-prompt-template.md
├── increments/
├── state.json
└── receipts/
```

Keep these artifacts on an isolated orchestration branch or in a dedicated control repository.
Never merge orchestration-only state into the product branch unless the user explicitly requests
that policy.

## Validate and render

Validate state:

```bash
python3 scripts/validate_state.py /path/to/state.json
```

Validate native/read-only evidence:

```bash
python3 scripts/validate_evidence_receipt.py /path/to/evidence-receipt.json
```

Render an increment prompt:

```bash
python3 scripts/render_increment_prompt.py \
  --template /path/to/increment-orchestrator-prompt-template.md \
  --variables /path/to/variables.json \
  --contract /path/to/increment-contract.md \
  --output /path/to/rendered-prompt.md
```

Reject unresolved placeholders. Never fill missing authority or repository identity by guesswork.
