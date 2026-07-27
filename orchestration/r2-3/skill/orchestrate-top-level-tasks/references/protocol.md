# Top-Level Task Orchestration Protocol

## Contents

1. Authority and terminology
2. State machine
3. Dispatch lifecycle
4. Receipt lifecycle
5. Independent verification
6. Concurrency
7. Failure handling
8. Bootstrap identity

## 1. Authority and terminology

Create persistent top-level tasks only when the user explicitly authorizes them. A top-level task
is user-visible and is created with `create_thread`. A subagent is internal to one top-level task
and is created with the collaboration tools.

The meta orchestrator creates fresh increment orchestrators. Increment orchestrators may create
subagents. Top-level tasks are peers in the product even though the protocol gives them logical
parent/child roles.

Never use `fork_thread` for a fresh increment. Its inherited conversation history undermines
context isolation.

## 2. State machine

Use these meta states:

```text
INITIALIZING
  -> READY
  -> DISPATCHING
  -> RUNNING
  -> RECEIPT_RECEIVED
  -> VERIFYING
  -> READY(next)
  -> EVIDENCE_RUNNING
  -> EVIDENCE_RECEIPTS_RECEIVED
  -> EVIDENCE_VERIFYING
  -> READY(gated increment)
  -> DISPATCHING
  -> RUNNING
  -> RECEIPT_RECEIVED
  -> VERIFYING
  -> COMPLETE

Any non-recoverable or authority-expanding condition:
  -> BLOCKED
```

Exactly one mutating increment may be `RUNNING` for a target ref or shared checkout.

Keep `dispatch_authorized=false` through setup and identity binding unless the user explicitly
authorized execution to start. `READY` with authorization false means configured and idle.

Read-only evidence tasks may run concurrently when:

- each uses a different platform checkout;
- their contracts prohibit publication and product mutation;
- they report only evidence;
- the final integration task remains blocked until all required receipts validate.

Evidence states keep product `active_dispatch` null and record every evidence task under
`active_evidence_dispatches` with its exact task identity, nonce, source commit/tree/ref, and
platform/project. Verified evidence records retain the receipt path/digest, platform/project, and
the same source binding.

Declare every mandatory native gate in `required_evidence`. Each entry binds an evidence ID and
platform to an exact completed source checkpoint (`after_increment`) and the immediately following
gated increment (`before_increment`). Evidence source bytes must equal the published checkpoint.
The gated increment cannot enter a mutating state until every declaration for it is verified.
`COMPLETE` requires the verified evidence ID set to equal the declared required set exactly.

## 3. Dispatch lifecycle

Before each dispatch:

1. Fetch or query the live remote.
2. Compare the live target ref to `expected_base.commit`.
3. Verify the expected tree and prerequisite ancestry.
4. Resolve the exact project from `list_projects`.
5. Read the canonical increment contract from the expected base.
6. Render a complete prompt with a random dispatch nonce.
7. Create the task with `create_thread`.
8. Resolve a real thread ID if creation returned a temporary client ID.
9. Persist the dispatch record.
10. Send an identity-binding follow-up containing the increment task's own real thread ID and host
    ID plus authority to begin that nonce-bound dispatch.
11. End the meta turn.

The prompt must include:

- exact worktree boundary;
- protected checkout boundaries;
- expected base commit/tree and target ref;
- one increment and one completion claim;
- file, symbol, test, and proof allowlists;
- dependency and non-goal boundaries;
- required skills and impact analysis;
- review and publication contract;
- meta thread/host identity and dispatch nonce;
- an initialization barrier until the increment task's own thread/host identity is bound;
- success and blocked receipt schemas;
- a rule making the receipt send the final tool action.

Immediately before publication, the increment task must verify the live target still equals its
expected base. A normal fast-forward push is permitted when authorized. A moved target returns
`BASE_DRIFT`; never silently merge, rebase, or force-push.

## 4. Receipt lifecycle

Use protocol `codex.top-level-task-receipt.v1`.

A successful receipt uses status `LANDED_CLEAN` and includes:

```json
{
  "protocol": "codex.top-level-task-receipt.v1",
  "orchestration_id": "example",
  "dispatch_nonce": "32-or-more-hex-characters",
  "increment": "EXAMPLE-1",
  "status": "LANDED_CLEAN",
  "increment_task": {
    "thread_id": "example-thread",
    "host_id": "example-host"
  },
  "expected_base": {
    "commit": "40-lowercase-hex",
    "tree": "40-lowercase-hex"
  },
  "landed": {
    "commit": "40-lowercase-hex",
    "tree": "40-lowercase-hex",
    "target_ref": "refs/heads/example",
    "live_remote": "40-lowercase-hex"
  },
  "subject_fingerprint": "sha256:64-lowercase-hex",
  "changed_files": ["path"],
  "review": {
    "packet_id": "EXAMPLE-1",
    "record_path": "/absolute/path",
    "record_sha256": "sha256:64-lowercase-hex",
    "validated": true,
    "terminal": "CLEAN",
    "p1": 0,
    "p2": 0,
    "p3_p4_disposition": "none"
  },
  "verification": {
    "required_checks_passed": true,
    "allowlist_passed": true,
    "change_detection_passed": true,
    "checkout_clean": true,
    "ahead": 0,
    "behind": 0
  },
  "next_increment": "EXAMPLE-2"
}
```

`next_increment` is not free-form workflow authority. Validate it against the authoritative
increment index with `validate_receipt.py --expected-next`. Use the reserved value
`EVIDENCE:<increment>` when the landed increment transitions to a declared evidence gate before
that increment, and `COMPLETE` only for the final closeout.

Blocked receipts omit `landed` and include:

```json
{
  "blocker": {
    "summary": "Concise blocker",
    "details": "Evidence-backed details",
    "required_authority": "Exact user or environment action",
    "handoff_prompt": "Complete continuation prompt when applicable"
  }
}
```

Native/read-only evidence uses protocol `codex.top-level-evidence-receipt.v1`. Validate it with
`validate_evidence_receipt.py`. A clean evidence receipt binds:

- evidence task thread/host identity;
- platform;
- source ref, commit, tree, and live remote;
- project ID/path, host, OS, and tool versions;
- artifact path and SHA-256;
- exact gates;
- prohibited-action and unchanged-checkout confirmations.

`BLOCKED_PLATFORM_HANDOFF_REQUIRED` must include the complete human handoff prompt.

The increment task sends the receipt to the meta task with `send_message_to_thread`. That call is
its last tool action. It may then return a human-readable final answer without further tools.

## 5. Independent verification

Treat the receipt as a claim. The meta task verifies:

1. Schema and authoritative successor with
   `validate_receipt.py --expected-next <index-value>`.
2. Correlation:
   - orchestration ID;
   - dispatch nonce;
   - increment;
   - active increment-task thread.
3. Task terminal barrier:
   - match the receipt's task thread/host to active state;
   - call `wait_threads` with a bounded 120-second wait;
   - permit at most three consecutive terminal-barrier waits;
   - if the task still has not settled, stop as `BLOCKED_TASK_NOT_TERMINAL`;
   - never launch the next mutating task while the prior task can still act.
4. Remote truth:
   - target ref equals `landed.commit`;
   - `landed.live_remote` equals `landed.commit`;
   - expected base is an ancestor;
   - commit count and topology meet the increment policy.
5. Tree and diff:
   - reported tree matches;
   - changed paths are exact;
   - no protected path changed.
6. Review:
   - review record exists and validates;
   - subject fingerprint recomputes;
   - terminal status is CLEAN;
   - blocking priorities are zero;
   - lower-priority inventory disposition is complete.
7. Final hygiene:
   - index state is current if required;
   - published increment-task status is clean;
   - no later remote advance occurred.

Only after verification may the meta task persist the receipt and advance.

Evidence receipts use the same terminal barrier before acceptance:

- match the evidence task thread/host to active state;
- call `wait_threads` for 120 seconds at most three consecutive times;
- stop as `BLOCKED_TASK_NOT_TERMINAL` if the task has not settled;
- never clear an active evidence dispatch or begin the gated increment while its task can act.

After both task termination and receipt verification, persist the receipt digest and its exact
task/platform/project/source bindings in `verified_evidence`. Clear evidence dispatches only when
the complete current required-evidence set has verified.

## 6. Concurrency

Permit:

- one mutating increment per target ref;
- subagents inside the active increment task;
- multiple read-only reviewers;
- independent read-only evidence tasks on different platforms.

Forbid:

- pre-creating all increment tasks;
- overlapping publications to the same ref;
- a later increment starting from an unverified receipt;
- reusing a completed increment task for a different increment;
- asking a completed task to generate the next authoritative prompt.

## 7. Failure handling

Stop and preserve the last verified base for:

- repository/base contradiction;
- scope expansion;
- unexpected high-risk authority root;
- unresolved blocking review finding;
- unavailable required platform;
- missing human authority;
- moved remote target;
- malformed or unverifiable receipt.

The meta task may send a correction request to the same increment task only while that increment
remains active and the correction stays inside its existing authority. Otherwise require a new
dispatch or user direction.

## 8. Bootstrap identity

The creator learns the meta task's thread ID only after `create_thread`. Bootstrap in two phases:

1. Create the meta task with `INITIALIZING` status and a rule prohibiting dispatch.
2. Send a follow-up containing:
   - meta thread ID;
   - meta host ID;
   - orchestration ID;
   - state path;
   - target ref;
   - explicit dispatch posture.

The meta task persists that identity binding before dispatch. Identity binding does not itself
authorize the first increment. Do not identify the meta task by title or summary.

Use `bind_meta_identity.py` to transition from `INITIALIZING` to `READY` without granting dispatch
authority. A later explicit user instruction may set `dispatch_authorized=true`.
