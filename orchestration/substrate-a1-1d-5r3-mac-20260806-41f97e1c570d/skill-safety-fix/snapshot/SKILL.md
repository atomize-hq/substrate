---
name: orchestrate-top-level-tasks
description: Coordinate ordered or dependency-gated work across fresh, persistent top-level Codex tasks using create_thread, structured terminal receipts, independently verified remote landings or local two-commit closeouts, cross-platform project dispatch, and durable orchestration state. Use when a user explicitly asks Codex to run a multi-session implementation, migration, review, evidence campaign, or other workflow where each bounded increment should run in a fresh top-level task instead of accumulating one long conversation.
---

# Orchestrate Top-Level Tasks

Run a persistent control-plane task that creates fresh top-level increment tasks, becomes idle,
and advances only after independently validating a terminal receipt. Keep task-level subagents
inside each increment task; do not confuse them with user-visible top-level tasks.

## Load the protocol

Before creating or messaging any top-level task:

1. Resolve the absolute skill root from this loaded `SKILL.md`. This is a repository-local skill;
   never install or register it as a global Codex skill.
2. Read `references/protocol.md` completely.
3. Read `references/platform-dispatch.md` when any gate is host- or OS-specific.
4. Use `assets/meta-orchestrator-prompt-template.md` to create the persistent meta prompt.
5. Use `assets/increment-orchestrator-prompt-template.md` for each fresh increment task.
6. Use `assets/evidence-task-prompt-template.md` for native/read-only platform evidence.

Git worktrees do not copy this repository's ignored `.agents/` directory. Before sending identity
binding or start authority to any newly created meta, increment, or evidence task, hydrate the
exact repository-local skill into that task's assigned worktree and verify it:

```bash
python3 scripts/hydrate_worktree_skill.py /absolute/task/worktree
python3 scripts/hydrate_worktree_skill.py /absolute/task/worktree --check
```

Run the script from the authoritative repository-local skill root, persist its reported source,
target, file count, and digest, and do not bind the task if hydration fails. The task's initial
prompt must remain self-contained until the next turn can load the hydrated skill.

Before the first dispatch, snapshot the exact templates, references, and validator scripts needed
by the workflow into the orchestration state root and record their source path and SHA-256. Render
and validate from those durable snapshots. Increment and evidence prompts must remain
self-contained; they must not depend on a repo-local skill being present in their assigned
worktree.

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
5. Advance only from a structurally valid `LANDED_CLEAN` or `CLOSED_CLEAN` receipt and
   independent repository proof.
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
6. Resolve the meta task's exact assigned worktree, hydrate this repository-local skill into it,
   verify the hydrated digest, and persist the hydration record.
7. Send the meta task its own thread ID, host ID, orchestration ID, state path, and explicit
   dispatch posture.
8. Persist the identity binding before the first dispatch.

Represent setup-only authorization explicitly. Keep `dispatch_authorized=false` after identity
binding unless the user authorized the workflow to start. Never infer start authority from setup.

## Dispatch an increment

1. Verify the authoritative target ref still equals the state ledger's expected base. Query the
   live remote for remote-publication increments; query the exact local branch ref for local-only
   increments.
2. Load only the increment's canonical contract and common workflow envelope.
3. Render the prompt with `scripts/render_increment_prompt.py`.
4. Generate a unique dispatch nonce.
5. Create a fresh top-level task at the exact starting ref.
6. Resolve and record the real thread ID, host ID, exact assigned worktree, nonce, and expected
   base.
7. Hydrate this repository-local skill into the fresh worktree, verify its digest, and persist the
   hydration record before granting any authority.
8. Send the task an identity-binding follow-up authorizing work for that dispatch.
9. End the meta turn after dispatch; do not block on the full implementation.

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

For `CLOSED_CLEAN`, also verify the repository and exact local two-commit closeout:

```bash
python3 scripts/verify_local_closeout.py \
  /path/to/receipt.json \
  --state /path/to/state.json \
  --expected-next AUTHORITATIVE_INDEX_VALUE
```

Then independently verify:

- receipt nonce, increment, increment-task thread, and expected base match active state;
- the increment task is terminal before another task can mutate the same checkout;
- for `LANDED_CLEAN`, the live remote equals the reported landed commit and the commit is the
  expected fast-forward descendant;
- for `CLOSED_CLEAN`, the named worktree is clean, the local branch ref equals the control commit,
  the exact direct-parent chain is `expected base -> reviewed work commit -> mechanical control
  commit`, and the observed remote ref did not change;
- reported tree and exact changed-file allowlist are correct;
- subject fingerprint and review record validate;
- blocking finding and inventory rules are satisfied;
- checkout/index cleanliness claims are supported.

Persist the verified receipt and update state before dispatching the next increment. Leave the
completed task unarchived by default. Archiving is not a completion gate or a prerequisite for
successor dispatch.

## Preserve tasks and worktrees

Treat task archiving as a destructive operation: the Codex app may remove the task's assigned
worktree, and unarchiving may not restore it.

- Never automatically archive a top-level task.
- Never archive a blocked, failed, authority-required, base-drift, or otherwise non-success task.
- Never archive a task whose worktree has modified, staged, or untracked files, or whose only copy
  of a commit, review artifact, receipt, evidence artifact, or handoff lives in that worktree.
- A terminal task may remain unarchived while its successor runs. The terminal barrier, not
  archiving, prevents concurrent mutation.
- Archive only after explicit user authorization and an independent disposal-safety check proves
  the worktree is clean and all unique state is durably preserved or published. Record that proof
  before calling `set_thread_archived`.
- Never treat unarchiving as a recovery mechanism. If an archive unexpectedly removes a worktree,
  stop and report the loss before any successor dispatch.

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

Do not create the next increment. Report the exact blocker, preserve the task and its worktree
byte-for-byte, and preserve the last verified base. Do not archive the task.

## Dispatch platform evidence

Discover projects again at dispatch time. Match platform projects by path, host, and project ID;
never assume an earlier inventory is current.

Run independent read-only platform evidence tasks concurrently only when their contracts do not
mutate the shared target. Require structured evidence receipts. Bind each dispatch to its platform,
project ID/path, host ID, source checkpoint, nonce, and task identity. Apply the same bounded
terminal barrier used for increment tasks before accepting an evidence receipt. Hydrate and verify
the repository-local skill in every evidence-task worktree before sending identity binding or
read-only start authority.

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

Validate task receipts against the machine-readable schema when available:

```bash
python3 -m jsonschema \
  -i /path/to/receipt.json \
  references/task-receipt.schema.json
```

Validate native/read-only evidence:

```bash
python3 scripts/validate_evidence_receipt.py /path/to/evidence-receipt.json
```

For evidence bound to a local-only checkpoint, also run:

```bash
python3 scripts/verify_local_evidence.py /path/to/evidence-receipt.json
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
