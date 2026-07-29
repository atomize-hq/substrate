Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3T. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: dabfa9de2a0a2547e78778d4e86cf3a67c9e3708a9727ba5e3e1c6b772a8ffee
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3T
- packet_id: A1.1d-5R2-3T
- next_increment: EVIDENCE:R2-3Z

INCREMENT-TASK IDENTITY BARRIER

Do not edit, delegate, run implementation checks, or publish until the meta orchestrator sends a
follow-up binding your own real increment-task thread ID and host ID to this dispatch nonce. Echo
those exact IDs in every terminal receipt.

REPOSITORY BOUNDARY

Work only in the task-assigned checkout:

the task-assigned Linux Codex worktree

Never mutate these protected checkouts:

- /home/spenser/__Active_code/substrate
- /home/spenser/__Active_code/substrate-r2-3
- /home/spenser/__Active_code/substrate-r2-3-meta-orchestration
- C:\Users\spmcc\Documents\__Project_Code\substrate-r2-3

Canonical starting state:

- remote: origin
- target ref: refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
- expected base commit: 549949a20f01397e49d3819398359cea522f3a83
- expected base tree: 1b98a9a96d40a4a9e453099815e9bf11b32a4831
- required ancestor: 0f1e147fb735791b44a65099a65167cbdc1803af

Before editing, fetch/query the live remote and verify the exact base, tree, ancestry, cleanliness,
0 ahead/0 behind, and current GitNexus index. If they differ, send BASE_DRIFT or
BLOCKED_CONTRADICTION. Do not reconcile, merge, rebase, reset, clean, or force-push.

SUBAGENT ORCHESTRATION

Use repository-required model/reasoning settings for every subagent. Complete required pre-edit
impact analysis before any subagent edits an existing symbol. Give editing subagents mutually
exclusive ownership when practical. Use fresh read-only subagents for independent review. Do not
allow a reviewer to review implementation it authored.

INCREMENT CONTRACT

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3T

Use:

- using-agent-skills
- context-engineering
- source-driven-development
- api-and-interface-design
- test-driven-development
- incremental-implementation
- gitnexus-impact-analysis
- security-and-hardening
- code-review-and-quality
- git-workflow-and-versioning

This top-level task and every subagent must run on Standard/default speed, never Fast. Every
subagent must use GPT-5.4 with Extra High reasoning. Treat the global trace registration/init
boundary, compatibility posture, and policy Git lookup as HIGH/CRITICAL even when graph coverage
under-reports cfg-specific callers.

Load only:

- current status in `llm-last-mile/runtime-refactor/00-README.md`;
- the parent A1.1d-5R2-3 row, common child-subdivision rules, and R2-3T section in
  `llm-last-mile/runtime-refactor/03-phase-slice-map.md`;
- PI-118 and supporting PI-117 current-state/required-action rows in
  `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`;
- the explicit-product trace/policy and R3-exclusive lifecycle contracts in
  `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`;
- final R2-SHIM-01 and R2-DIAG-01 requirements in
  `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`; and
- `06-review-finding-inventory.md` only for actual P3/P4 deduplication.

SELECTED OUTCOME

After exact caller closure proves F, R, S1, and S2 migrated every R2-owned product consumer,
remove or make unreachable `LegacyAmbientCompatibility`, make global unbound `init_trace(None)`
fail, and remove legacy ambient policy Git lookup. Preserve the neutral global setter and every
explicit-product boundary exactly.

The accepted S2 landing at commit `549949a20f01397e49d3819398359cea522f3a83` completes the
physical-shim caller migration. Accepted F/R/S1/S2 behavior is an immutable prerequisite: T owns
only the trace compatibility cutover.

EXACT COMPLETION CLAIM

R2-3T completes final PI-118 compatibility closure. It does not alter the already-complete PI-117
explicit-product shell contract, reopen physical-shim/replay/platform migrations, change trace
writer/rotation/retention/span/replay semantics or schema, activate platform lifecycle or
forwarding, alter artifacts or deletion authority, claim native macOS/Windows evidence, complete
any R3 row, or complete the R2-3 parent packet. R2-3Z native evidence and integration closeout
remain gated after T.

EXACT ALLOWLIST

Production changes are limited to:

- `crates/trace/src/context.rs`
- `crates/trace/src/util.rs`

Test changes are limited to:

- `crates/trace/src/tests.rs`; and
- colocated `#[cfg(test)]` modules already inside the two production files only when mechanically
  required by the named symbols.

Do not edit any manifest, `Cargo.lock`, shim, replay, shell, factory, backend, installer, script,
platform, lifecycle, writer implementation file, control document, generated analyzer file,
`AGENTS.md`, or `CLAUDE.md`. If another production, dependency, or test file is required, stop
with `BLOCKED_SCOPE_EXPANSION`.

EXACT SYMBOL SCOPE

Inside `crates/trace/src/context.rs`, changes are limited to:

- `TraceContextBindingV1`;
- `TraceContext::default`;
- `TraceContext::init_trace`;
- global `init_trace`; and
- exact tests for those symbols.

Inside `crates/trace/src/util.rs`, changes are limited to:

- `get_policy_git_hash`; and
- exact tests for that symbol.

Inside `crates/trace/src/tests.rs`, changes are limited to the final compatibility expectations
and retained explicit-product regression proof.

Add no public symbol. The following are signature- and behavior-frozen:

- `set_global_trace_context`;
- `TraceContext::explicit_product`;
- `get_policy_git_hash_at`;
- writer, flush, serialization, rotation, retention, rename, and removal bodies;
- span and replay bodies and schemas; and
- every other trace API.

Minimum private enum-variant deletion/match cleanup is allowed only to eliminate or make
unreachable the named legacy posture. No caller-identity side table, inferred caller class,
process-global authority cache, fallback directory, or new compatibility variant is allowed.

PRE-EDIT SOURCE CLOSURE AND IMPACT

Before editing:

1. Fetch/query and verify the exact base/tree/ancestor, clean task checkout, 0/0 divergence, and a
   current GitNexus index for this exact task worktree.
2. Run file-qualified upstream GitNexus impact with tests for every named editable symbol and
   read-only impact for `set_global_trace_context`, `TraceContext::explicit_product`, and
   `get_policy_git_hash_at`.
3. Query every Rust caller of global `init_trace`, `TraceContext::init_trace`,
   `TraceContext::default`, `get_policy_git_hash`, `get_policy_git_hash_at`,
   `set_global_trace_context`, and `TraceContext::explicit_product`.
4. Classify every caller against accepted F/R/S1/S2 ownership. Prove no R2-owned product path
   requires legacy compatibility or ambient policy lookup.
5. Inspect physical shim, replay, shell, and platform factory call sites read-only. Prove their
   accepted explicit-product/typed behavior remains unchanged and no allowed caller migration is
   missing.
6. Inspect writer/rotation/retention/span/replay code read-only and record frozen hashes where
   adjacency could obscure the diff.
7. Record the accepted S2 fingerprint/review and prove every S2 changed file remains unchanged.

Warn before editing because this is a global compatibility cutover. Any unmigrated owned caller,
required caller edit, setter semantic change, explicit-product/API change, caller-identity table,
writer/lifecycle change, or unrelated trace schema change stops T.

FINAL COMPATIBILITY CONTRACT

`LegacyAmbientCompatibility` is temporary and non-promotable. After caller closure:

- remove its enum variant and all selection logic if source-compatible closure permits; otherwise
  make it provably unreachable without adding a replacement compatibility posture;
- `TraceContext::default()` must no longer silently create ambient product authority;
- global `init_trace(None)` without a previously registered explicit trace context must return a
  clear error before path selection, directory creation, writer initialization, or output;
- global `init_trace(Some(path))` may initialize only according to the already-landed explicit
  contract and cannot synthesize a home/profile default;
- `TraceContext::init_trace(None)` on a valid `ExplicitProduct` context continues to initialize or
  reuse exactly its bound trace target;
- repeated equal explicit-product binding remains idempotent;
- a conflicting explicit trace path remains rejected;
- `get_policy_git_hash` must no longer derive a policy Git directory from HOME, USERPROFILE,
  SUBSTRATE_HOME, CWD, repository discovery, or any ambient resolver; and
- `get_policy_git_hash_at(A)` remains the sole A-scoped behavior and is byte/behavior frozen.

The neutral setter remains registration only. It gains no path selection, context validation,
caller discrimination, rejection rule beyond its existing set-once behavior, or lifecycle
semantics. No side table may distinguish legacy versus migrated callers.

Removing compatibility must not make tests or internal helpers install ambient authority as a
workaround. Tests must register explicit contexts or assert the final unbound failure directly.

SEMANTIC FREEZE

Preserve byte/behavior-equivalent operation for:

- `TraceContext::explicit_product` construction and validation;
- `get_policy_git_hash_at` A-only missing/unreadable/invalid metadata behavior;
- neutral setter registration semantics;
- writer opening, buffering, flushing, serialization, rotation, retention, rename, and removal;
- trace/span/replay schemas, IDs, timestamps, fields, ordering, redaction, and error behavior after
  valid initialization;
- accepted shell, physical-shim, replay, and platform caller behavior; and
- all install, deployment, migration, lifecycle, forwarding, listener, process, PID, artifact,
  teardown, and deletion behavior.

T grants no R3 authority.

TEST-DRIVEN PROOF

Use only the authorized trace tests. Cover at least:

- global unbound `init_trace(None)` fails before filesystem access;
- the former legacy-default success expectation is removed/replaced by final failure;
- no HOME, USERPROFILE, SUBSTRATE_HOME, SHIM_TRACE_LOG, CWD, repository, or temp-directory value
  can cause unbound trace initialization;
- ambient `get_policy_git_hash` cannot select a Git root or return a commit;
- explicit-product A writes only `A/trace.jsonl` under conflicting ambient B;
- explicit policy Git lookup reads only A and never retries under B;
- repeated equal A initialization/reuse succeeds;
- conflicting explicit trace path rejects;
- explicit-product symlink/path checks remain intact;
- missing/unreadable/invalid policy metadata returns the landed no-commit result without fallback;
- setter behavior remains unchanged;
- writer, rotation, retention, serialization, span, replay, and redaction regression tests remain
  green; and
- source closure confirms no owned caller still needs legacy/default ambient behavior.

Run at minimum:

- `cargo fmt --all -- --check`;
- `cargo check --locked -p substrate-trace`;
- `cargo test --locked -p substrate-trace -- --nocapture`;
- `cargo test --locked -p substrate-trace --doc`;
- `cargo clippy --locked -p substrate-trace --all-targets -- -D warnings`;
- focused unbound-failure, explicit-product A/B, policy Git, repeated-init, conflict, symlink,
  missing-metadata, writer, rotation, and retention tests;
- exact textual caller/source closure across the workspace;
- frozen-byte verification for `set_global_trace_context`, `TraceContext::explicit_product`,
  `get_policy_git_hash_at`, writer/rotation/retention/span/replay bodies;
- `git diff --check`;
- exact unstaged/staged allowlist checks; and
- pre-publication `gitnexus_detect_changes()` with every affected flow inspected.

Native evidence is not independently required for T beyond honest migrated-caller proof. Do not
claim final macOS/Windows evidence; the evidence gate before R2-3Z owns that accounting.

SUBJECT FINGERPRINT AND REVIEW

After the final subject settles:

1. Build a deterministic manifest containing the exact expected base plus each allowed changed
   path's staged mode, blob, stage, and pathname.
2. SHA-256 that exact manifest and use `sha256:<digest>` as the subject fingerprint.
3. Run fresh read-only bounded review lenses covering:
   - complete caller closure and final unbound/ambient failure;
   - preservation of explicit-product A/B and policy Git behavior;
   - setter/API/writer/rotation/retention/span/replay freeze, allowlist, and proof honesty.
4. Remediate every valid P1/P2 in scope, recompute the fingerprint, and rerun closure review.
5. Causally disposition every P3/P4 against `06-review-finding-inventory.md`.
6. Validate the final review-cycle JSON and require zero open P1/P2 before publication.

STOP CONDITIONS

Stop with the matching blocked receipt if:

- the task worktree or live target differs from the nonce-bound base/tree;
- any owned caller still relies on legacy/default ambient behavior;
- any caller file or other production/test file outside the exact allowlist requires change;
- removing compatibility requires changing the setter, explicit-product API, A-scoped policy API,
  writer/rotation/retention/span/replay behavior, or trace schema;
- any new compatibility variant, ambient selector, inferred caller identity, or side table is
  proposed;
- lifecycle, forwarding, replacement, teardown, PID, artifact, or deletion authority becomes
  reachable;
- required tests cannot run and no honest equivalent proof is authorized; or
- review ends with any open P1/P2.

PUBLICATION AND RECEIPT

Before commit, require the live product ref still equals
`549949a20f01397e49d3819398359cea522f3a83` with tree
`1b98a9a96d40a4a9e453099815e9bf11b32a4831`. Stage only the exact authorized files, inspect the
staged diff, rerun allowlist/diff/fingerprint/review checks, create one Conventional Commit, and
normal fast-forward push `HEAD` to
`refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`. Never force-push.

Verify the live ref equals the landed commit/tree, refresh GitNexus on the landed tree, restore only
analyzer-generated count churn, and finish with a clean task worktree and 0/0 divergence.

Send one `codex.top-level-task-receipt.v1` terminal message to the meta task. A successful receipt
must use `LANDED_CLEAN`, echo the bound task identity and nonce, include expected/landed
commit/tree, exact changed paths, fingerprint, validated review record/digest and finding
disposition, checks and limitations, GitNexus result, clean status, exact final PI-118 completion
boundary, and `next_increment: EVIDENCE:R2-3Z`.

The send to the meta task is your final tool action. Do not dispatch evidence and do not begin or
render R2-3Z.

COMMON TERMINAL CONTRACT

Before publication:

1. Complete every increment-specific check and proof gate.
2. Verify the exact file/symbol/test allowlist.
3. Run `gitnexus_detect_changes()` and inspect every affected flow.
4. Complete and validate the bounded review sequence.
5. Require zero open blocking findings.
6. Fetch/query the live target again and require it still equals the expected base.

When authorized by the increment contract, create one atomic commit and perform a normal
fast-forward push of `HEAD` to refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap. Never force-push. Verify the live remote equals the
landed commit, refresh GitNexus, restore analyzer-only generated count changes if necessary, and
finish clean.

Send a `codex.top-level-task-receipt.v1` message to the meta task. For success, use
`LANDED_CLEAN` and include your bound increment-task thread/host IDs, expected base, landed
commit/tree, changed paths, subject fingerprint, validated review record and digest, finding
disposition, checks, GitNexus result, clean status, and next increment.

For failure, send the exact blocked status, evidence, required authority or platform, and a
complete continuation/handoff prompt.

The `send_message_to_thread` call is your final tool action. After it succeeds, make no more tool
calls or repository changes. Return only the human-readable final report.

Do not generate the next increment prompt and do not begin EVIDENCE:R2-3Z. The meta
orchestrator owns independent verification and subsequent dispatch.
