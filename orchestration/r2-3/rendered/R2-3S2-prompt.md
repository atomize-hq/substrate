Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3S2. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: dbfd364ed94158b486f4eabbb61b13efc802d992e969151f6c78b60a41212117
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3S2
- packet_id: A1.1d-5R2-3S2
- next_increment: R2-3T

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
- expected base commit: e1c251ad41725efc780747ca01aca08f85874ac2
- expected base tree: b6fb36e77f83d26bdb2bdfd49f090c4830b5826f
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

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3S2

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
subagent must use GPT-5.4 with Extra High reasoning. Treat the physical shim process root and its
trace, policy, telemetry, span, and execution-log projections as HIGH/CRITICAL posture even if the
Linux index under-reports cfg-specific paths.

Load only:

- current status in `llm-last-mile/runtime-refactor/00-README.md`;
- the parent A1.1d-5R2-3 row, common child-subdivision rules, and R2-3S2 section in
  `llm-last-mile/runtime-refactor/03-phase-slice-map.md`;
- PI-118 and supporting PI-090/PI-116 current-state/required-action rows in
  `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`;
- the host-context boundary-carrier, physical-shim entry, explicit-product trace/policy, and
  R3-exclusive lifecycle contracts in `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`;
- R2-SHIM-01 and R2-DIAG-01 in
  `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`; and
- `06-review-finding-inventory.md` only for actual P3/P4 deduplication.

SELECTED OUTCOME

Bind the explicit product trace and policy inputs from the one S1-authenticated physical-shim
`InstallBootstrapContextCarrierV1` before manager initialization, policy evaluation, telemetry,
span creation, or execution logging. Reuse exactly that one binding throughout the physical shim.

The accepted S1 landing at commit `e1c251ad41725efc780747ca01aca08f85874ac2` already owns
invocation-witness recovery, current-principal binding, A-scoped manager paths, typed platform
telemetry mapping, and the final no-contextless macOS/Windows factory cutover. Those behaviors are
inputs to S2 and must not be reopened. The accepted R landing remains the replay-side typed factory
owner. S2 migrates only physical-shim trace and policy projection.

EXACT COMPLETION CLAIM

R2-3S2 completes only the physical-shim portion of PI-118. It does not claim final PI-118
completion: R2-3T remains the sole final compatibility owner. It does not change S1 invocation,
principal, manager, mapping, or factory behavior; change trace writer/rotation/retention/span or
policy semantics; activate platform lifecycle or forwarding; alter shim artifact
replacement/migration/removal; claim native macOS/Windows evidence; complete any R3 row; or
complete the R2-3 parent packet.

EXACT ALLOWLIST

Production changes are limited to:

- `crates/shim/src/exec/mod.rs`
- `crates/shim/src/exec/policy.rs`
- `crates/shim/src/logger.rs`
- exact invocation/factory projection in `crates/shim/src/exec/logging.rs`

Test changes are limited to:

- colocated `#[cfg(test)]` modules inside those four production files; and
- `crates/shim/tests/integration.rs`.

Do not edit any manifest, `Cargo.lock`, trace crate, replay crate, shell crate, shim context,
bootstrap, resolver, deployment/migration/removal code, world-backend-factory, backend
implementation crate, installer, script, platform lifecycle/forwarder/listener code, control
document, generated analyzer file, `AGENTS.md`, or `CLAUDE.md`. If another production, dependency,
or test file is required, stop with `BLOCKED_SCOPE_EXPANSION`.

EXACT SYMBOL SCOPE

Inside `crates/shim/src/exec/mod.rs`, changes are limited to:

- `run_shim`; and
- minimum private import/signature/call plumbing needed to bind and reuse the explicit product
  trace/policy input.

Inside `crates/shim/src/exec/policy.rs`, changes are limited to:

- `evaluate_policy`; and
- minimum private typed-input plumbing required to use the already-bound A for policy metadata.

Inside `crates/shim/src/logger.rs`, changes are limited to:

- `start_span`;
- `log_execution`;
- `write_log_entry`; and
- minimum private typed-input plumbing required to reuse the already-bound trace target and policy
  Git root.

Inside `crates/shim/src/exec/logging.rs`, changes are limited to:

- `collect_world_telemetry` only for exact signature/call projection mechanically required by the
  one bound input.

Add no public symbol. `evaluate_policy`, logging, and telemetry reuse the already-bound context and
may not initialize or select ambient state. Every other production symbol is byte-frozen except
minimum private field/signature/call plumbing for these named items.

PRE-EDIT SOURCE CLOSURE AND IMPACT

Before editing:

1. Fetch/query and verify the exact base/tree/ancestor, clean task checkout, 0/0 divergence, and a
   current GitNexus index for this exact task worktree.
2. Run file-qualified upstream GitNexus impact with tests for `run_shim`, `evaluate_policy`,
   `start_span`, `log_execution`, `write_log_entry`, `collect_world_telemetry`, and
   `TraceContext::explicit_product`.
3. Perform exact textual caller closure for every named symbol plus `TraceContext::default`,
   `TraceContext::init_trace`, global `init_trace`, `get_policy_git_hash`,
   `get_policy_git_hash_at`, and `set_global_trace_context`.
4. Inspect the landed S1 IH/PM binding, manager initialization, typed telemetry call, and factory
   cutover read-only. Prove the same S1 carrier can supply every S2 input without reconstruction or
   ambient selection.
5. Inspect the landed explicit-product trace API and policy Git API in the trace crate read-only.
   Prove S2 can consume their public surface without changing the trace crate.
6. Inspect shim bypass, command resolution, manager hints, policy evaluation, logger, trace span,
   execution log, writer/rotation/retention, and platform telemetry paths read-only. Record frozen
   bodies/hashes where adjacency makes the diff ambiguous.
7. Record S1's accepted subject fingerprint and validated review digest, and prove
   `crates/shim/src/context.rs`, `crates/world-backend-factory/src/lib.rs`, and all R replay files
   remain unchanged.

Warn before editing because this is the physical product process root. A required trace-crate
change, second authority source, ambient trace/policy selector, caller-identity side table,
writer/rotation/retention change, policy semantic change, S1 regression, lifecycle reachability,
or action/deletion change stops S2.

EXPLICIT PRODUCT TRACE/POLICY CONTRACT

After S1 validates the physical invocation witness and constructs the one canonical host carrier,
`run_shim` must construct the additive explicit-product trace posture from that carrier before any
manager, policy, telemetry, span, or execution-log consumer.

The bound product projection is:

- trace output: exactly `<normalized A>/trace.jsonl`;
- policy Git root: exactly normalized A;
- principal/commitment: the same already-validated S1 carrier;
- no independent H, R, home/profile, repository, CWD, or environment selector.

The existing `TraceContext::explicit_product` boundary is consumed exactly as landed. S2 may not
change its signature or implementation. The neutral global setter retains its exact semantics.
Repeated binding of the same A reuses the same bound context; conflicting trace target, policy Git
root, carrier, or commitment fails before manager, policy, telemetry, span, logging, or command
dispatch.

`SHIM_TRACE_LOG`, `HOME`, `USERPROFILE`, `SUBSTRATE_HOME`, `SUBSTRATE_ROOT`, CWD, PATH, repository
discovery, compile-time paths, trace contents, and legacy inherited H/R values cannot select or
replace the product trace or policy root. Inherited projections may only be checked for equality
where the landed contract already requires it. Missing or malformed product metadata fails closed;
there is no ambient fallback.

The physical-shim path must not use `TraceContext::default`,
`LegacyAmbientCompatibility`, unbound global `init_trace(None)`, or ambient
`get_policy_git_hash`. It must use the bound trace context and A-scoped policy Git lookup exposed
by the landed trace API. T owns removing or making unreachable global compatibility after all
owned callers are migrated; S2 must not edit that compatibility implementation.

ORDERING AND SINGLE-BINDING GUARANTEE

The successful physical-shim order is:

1. validate S1 invocation witness and current principal;
2. construct/revalidate the one S1 host carrier;
3. bind the explicit product trace/policy projection from that carrier;
4. initialize manager hints from the same carrier;
5. evaluate policy using the same bound policy root;
6. create/finish spans using the same trace context;
7. collect typed world telemetry using the same S1 carrier/mapping;
8. write the execution log to the same A trace target.

Do not reconstruct a carrier or A in policy/logger/telemetry helpers. Do not create a second trace
context or cache/side table. Do not allow manager, policy, telemetry, span, or logging to run before
the explicit product binding succeeds.

`evaluate_policy`, `start_span`, `log_execution`, and `write_log_entry` receive or otherwise reuse
the bound input explicitly. They cannot call a common ambient substrate-home resolver, read an
environment-selected policy directory, use CWD/repository discovery, or invoke default trace
initialization. Policy content, mode, decisions, redaction, environment hashing, execution-log
schema, span fields, error/nonfatal semantics, and output formatting remain unchanged.

SEMANTIC AND LIFECYCLE FREEZE

Preserve byte/behavior-equivalent operation for:

- S1 invocation-witness resolution, current-principal binding, IH/PM validation, manager manifest
  selection, and typed platform factory behavior;
- shim recursion/depth/session/caller tracking, bypass, real-binary selection, process execution,
  exit handling, and command search;
- manager parsing, rules, matching, payloads, deduplication, and emission;
- broker/profile/policy decision semantics;
- trace serialization/schema, writer, flush, rotation, retention, rename, and removal;
- span creation/finish semantics and telemetry world handle/fs-diff behavior after valid
  construction; and
- deployment, replacement, migration, recursive removal, installers, platform lifecycle,
  forwarding, sockets/pipes/listeners, processes, PIDs, and deletion.

S2 grants no R3 action authority.

TEST-DRIVEN PROOF

Use deterministic temp fixtures and colocated seams. Cover at least:

- custom A under conflicting ambient B writes only `A/trace.jsonl`;
- policy Git metadata is read only from A;
- repeated use of the same A/context reuses the binding;
- conflicting trace target, policy root, carrier, or commitment rejects before downstream work;
- missing/unreadable/invalid policy metadata returns the landed no-commit result without fallback;
- missing/malformed product binding cannot fall back to default/ambient trace or policy state;
- manager, policy, telemetry, span, and execution logging all reuse the same bound A/carrier;
- zero B filesystem access under conflicting `HOME`, `USERPROFILE`, `SUBSTRATE_HOME`,
  `SUBSTRATE_ROOT`, `SHIM_TRACE_LOG`, CWD, PATH, and repository values;
- the physical shim no longer reaches `LegacyAmbientCompatibility`, default trace initialization,
  or ambient policy lookup;
- S1 typed telemetry and manager tests remain green;
- writer, rotation, retention, span, policy, redaction, schema, bypass, recursion, execution, and
  error behaviors remain unchanged; and
- no platform lifecycle, forwarding, deletion, migration, or replacement path becomes reachable.

Run at minimum:

- `cargo fmt --all -- --check`;
- `cargo check --locked -p substrate-shim`;
- `cargo test --locked -p substrate-shim -- --nocapture`;
- `cargo clippy --locked -p substrate-shim --all-targets -- -D warnings`;
- focused tests for every new trace/policy projection and every remediated finding;
- honest available macOS/Windows compile-only attempts proportionate to cfg-specific call
  plumbing, with environment limitations reported rather than counted as proof;
- exact source assertions/caller closure;
- frozen-byte verification for trace writer/rotation/retention and policy decision bodies;
- `git diff --check`;
- exact unstaged/staged allowlist checks; and
- pre-publication `gitnexus_detect_changes()` with every affected flow inspected.

Native Unix physical-shim evidence may be recorded here. R2-3Z owns final cross-platform evidence
accounting. Static/cross-target compilation is not native macOS/Windows evidence.

SUBJECT FINGERPRINT AND REVIEW

After the final subject settles:

1. Build an exact manifest of every allowed changed path with mode and blob hash.
2. Compute and report a SHA-256 subject fingerprint from that manifest.
3. Run fresh read-only bounded review lenses covering:
   - physical-shim ordering, single carrier/context reuse, and fail-before-dispatch behavior;
   - trace target/policy Git A binding, ambient B non-authority, and zero B access;
   - frozen trace writer/rotation/retention/span/policy semantics, allowlist, and proof honesty.
4. Remediate every valid P1/P2 in scope, recompute the fingerprint, and rerun closure review.
5. Disposition every P3/P4 causally against `06-review-finding-inventory.md`.
6. Validate the final review-cycle JSON and require zero open P1/P2 before publication.

STOP CONDITIONS

Stop with the matching blocked receipt if:

- the task worktree or live target differs from the nonce-bound base/tree;
- any production/test change outside the exact allowlist is required;
- `TraceContext::explicit_product` or `get_policy_git_hash_at` requires a trace-crate edit;
- the S1 carrier cannot provide the complete trace/policy binding without reconstruction;
- any default trace init, ambient policy directory, repository/home/CWD/PATH selector, or side table
  remains reachable from the migrated physical-shim path;
- writer/rotation/retention/span/policy semantics must change;
- S1 witness/manager/mapping/factory behavior must be reopened;
- lifecycle, forwarding, replacement, teardown, PID, artifact, or deletion authority becomes
  reachable;
- a required supported test cannot run and no honest equivalent proof is authorized; or
- review ends with any open P1/P2.

PUBLICATION AND RECEIPT

Before commit, require the live product ref still equals
`e1c251ad41725efc780747ca01aca08f85874ac2` with tree
`b6fb36e77f83d26bdb2bdfd49f090c4830b5826f`. Stage only the exact authorized files, inspect the
staged diff, rerun allowlist/diff/fingerprint/review/state checks, create one Conventional Commit,
and normal fast-forward push `HEAD` to
`refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`. Never force-push.

Verify the live ref equals the landed commit/tree, refresh GitNexus on the landed tree, restore only
analyzer-generated count churn, and finish with a clean task worktree and 0/0 divergence.

Send one `codex.top-level-task-receipt.v1` terminal message to the meta task. A successful receipt
must use `LANDED_CLEAN`, echo the bound task identity and nonce, include the expected and landed
commit/tree, exact changed paths, fingerprint, validated review record/digest and finding
disposition, checks and limitations, GitNexus result, clean status, the exact PI-118 physical-shim
completion boundary, and `next_increment: R2-3T`.

The send to the meta task is your final tool action. Do not begin or render R2-3T.

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

Do not generate the next increment prompt and do not begin R2-3T. The meta
orchestrator owns independent verification and subsequent dispatch.
