Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3S1. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: ba687d3224a37868c2998b826012ec0742dfe10e25d50725a835235c23a90465
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3S1
- packet_id: A1.1d-5R2-3S1
- next_increment: R2-3S2

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
- expected base commit: cefe2a9363b565ca8c53f3364a975c8406e781c9
- expected base tree: 946dbd4f6d94b5e990c228065dfcf8e74d878e00
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

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3S1

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
subagent must use GPT-5.4 with Extra High reasoning. Treat the physical shim process root, its
invocation-witness resolution, current-principal observation, manager manifest intake, and
platform factory boundary as HIGH/CRITICAL posture even if the Linux index under-reports
cfg-specific callers.

Load only:

- current status in `llm-last-mile/runtime-refactor/00-README.md`;
- the parent A1.1d-5R2-3 row, common child-subdivision rules, and R2-3S1 section in
  `llm-last-mile/runtime-refactor/03-phase-slice-map.md`;
- PI-054, PI-090, PI-116, and the supporting PI-118 current-state/required-action rows in
  `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`;
- the host-context boundary-carrier, physical-shim entry, invocation-witness, manager-manifest,
  platform mapping/factory, and R3-exclusive lifecycle contracts in
  `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`;
- R2-SHIM-01, R2-GEN-01, R2-MAP-MAC-01, R2-MAP-WIN-01, and the unchanged physical-shim portions
  of R2-DIAG-01 in `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`; and
- `06-review-finding-inventory.md` only for actual P3/P4 deduplication.

SELECTED OUTCOME

Recover exactly one no-follow-validated physical-shim invocation witness, derive one canonical
`InstallBootstrapContextCarrierV1` from that witness and the current OS principal, use its A-only
manager manifest paths, and pass its complete verified `PlatformBootstrapMappingV1` explicitly to
world telemetry before command dispatch whenever platform telemetry is requested.

R2-3R left one intentional contextless caller:
`crates/shim/src/exec/logging.rs::collect_world_telemetry`. R2-3S1 must migrate that caller to
`factory_with_platform_bootstrap` (or the landed typed equivalent). After exact source closure
proves no macOS/Windows contextless product caller remains, S1 must atomically remove or fail-close
the zero-argument macOS/Windows factory. Linux retains platform-independent factory behavior.

This final factory-cutover obligation is durable authority carried from the accepted R receipt:

```text
R2-3S1 must migrate collect_world_telemetry to the authenticated typed factory input and,
after exact source closure proves no macOS/Windows contextless consumer remains, atomically remove
or fail-close the zero-argument platform factory. R2-3S1 must include
crates/world-backend-factory/src/lib.rs in its exact allowlist for that final cutover.
```

EXACT COMPLETION CLAIM

R2-3S1 completes PI-090 and PI-116 plus PI-054's final no-contextless macOS/Windows factory
condition. It supports but does not complete PI-118. It does not migrate physical-shim trace or
policy binding, change trace/policy semantics, activate platform lifecycle or forwarding,
replace/migrate/remove shim artifacts, claim native Windows mapping evidence, complete any R3 row,
or complete the R2-3 parent packet.

EXACT ALLOWLIST

Production changes are limited to:

- `crates/shim/src/context.rs`
- `crates/shim/src/exec/mod.rs`
- `crates/shim/src/exec/logging.rs`
- `crates/world-backend-factory/src/lib.rs`

Test changes are limited to:

- colocated `#[cfg(test)]` modules inside those four production files; and
- `crates/shim/tests/integration.rs`.

Do not edit any manifest, `Cargo.lock`, trace crate, replay crate, shell crate, shim bootstrap,
resolver, policy, logger, deployment/migration/removal code, backend implementation crate,
installer, script, platform lifecycle/forwarder/listener code, control document, generated
analyzer file, `AGENTS.md`, or `CLAUDE.md`. If another production/dependency/test file is required,
stop with `BLOCKED_SCOPE_EXPANSION`.

EXACT SYMBOL SCOPE

Inside `crates/shim/src/context.rs`, changes are limited to:

- `ShimContext` only for minimum typed IH/PM fields required by the selected outcome;
- `ShimContext::from_current_exe`;
- `resolve_invoked_path`;
- both cfg definitions of `find_candidate_in_dir`;
- `check_candidate`;
- new `resolve_install_bootstrap_context_from_invocation`; and
- new `current_platform_principal_v1`.

Minimum private helpers may validate no-follow path/file identity, exact `A/shims/<command>`
shape, ancestor posture, current principal, and the exact default platform mapping. They may not
introduce a second authority, ambient prefix selector, side table, cache, lifecycle operation, or
general path resolver.

Inside `crates/shim/src/exec/mod.rs`, only `run_shim` plus minimum import/signature/call plumbing
may change.

Inside `crates/shim/src/exec/logging.rs`, changes are limited to:

- `collect_world_telemetry`;
- `ManagerHintEngine::new`;
- `manifest_paths`;
- `manifest_overlay_path`; and
- `repo_manifest_path`.

Minimum private helpers may validate/project the same typed IH/PM. No hint matching, payload,
deduplication, diagnostic wording, fs-diff, or span semantics may change.

Inside `crates/world-backend-factory/src/lib.rs`, only the cfg-specific zero-argument `factory`
definitions, their exact tests, and minimum import/cfg plumbing required for the final cutover may
change. The landed `factory_with_platform_bootstrap`, its canonicalization/validation, and its
tests are byte/behavior frozen. Linux zero-argument behavior remains equivalent. On macOS and
Windows, the zero-argument entry must be removed from the public surface if source-compatible
closure permits, or fail closed before backend construction. It may not call an ambient/default
backend constructor.

Every other production symbol is byte-frozen except minimum field/signature/call plumbing required
by the named items.

PRE-EDIT SOURCE CLOSURE AND IMPACT

Before editing:

1. Fetch/query and verify the exact base/tree/ancestor, clean task checkout, 0/0 divergence, and a
   current GitNexus index for this exact task worktree.
2. Run file-qualified upstream GitNexus impact with tests for every named production symbol,
   every cfg zero-argument `factory`, and the landed typed factory entry/canonicalizer.
3. Perform exact textual caller closure for every `ShimContext::from_current_exe`,
   `resolve_invoked_path`, `ManagerHintEngine::new`, `collect_world_telemetry`,
   `world_backend_factory::factory()`, and `factory_with_platform_bootstrap` call.
4. Inspect the landed R typed factory API and the macOS/Windows typed backend constructors
   read-only. Prove the physical shim supplies every required value without ambient selection.
5. Inspect shim bypass/bootstrap, resolver, policy, logger, trace, deployment, and migration paths
   read-only. Prove they require no edit and freeze their behavior.
6. Inspect every manager manifest fallback and every invocation-witness path, including
   absolute, relative, and bare argv forms plus Windows PATHEXT behavior.
7. Record the accepted R fingerprint/review and prove R's replay files remain unchanged.

Warn before editing because the physical shim root and factory cutover are HIGH/CRITICAL by
contract. A second contextless caller, required manifest/dependency change, inability to preserve
bypass/command resolution, unreviewed public schema effect, new authority source, trace/policy
change, lifecycle reachability, or action/deletion change stops S1.

PHYSICAL-SHIM INVOCATION AND IH CONTRACT

`argv[0]` is an invocation witness, never arbitrary prefix authority:

- Absolute argv must be validated at that exact path.
- Relative argv may be resolved against CWD only to recover the exact invocation pathname and must
  then pass the same witness checks.
- Bare argv may enumerate only absolute, nonempty PATH entries. Collect all candidates; PATH order
  is not precedence. Exactly one valid candidate must remain.
- A candidate must be an installed shim link/copy at exactly
  `<normalized A>/shims/<nonempty command>`, and its no-follow identity must resolve to the running
  `substrate-shim` target.
- The candidate and its `A/shims` ancestry must pass the applicable no-follow ownership/path
  checks. A symlink/reparse substitution in an ancestor is rejected.
- Zero candidates, multiple candidates, a same-name unrelated executable, a forged target, an
  invalid/noncanonical path, or a candidate outside `A/shims` fails before manager, policy,
  telemetry, trace/span, logging, or command dispatch.
- CWD and PATH may recover the invocation witness only. They may not select a fallback A.
  `HOME`, `USERPROFILE`, `SUBSTRATE_HOME`, `SUBSTRATE_ROOT`, inherited H/R/carriers, a repository
  path, or compile-time paths may not select A.

Derive a canonical `InstallBootstrapContextV1`/carrier from A and the current host principal.
Unix must resolve current account+UID by exact account-database round trip. Windows must resolve
the current token account+SID and required Known Folder through the already-authorized OS surface.
The recomputed principal must match any inherited projection. Missing, malformed, conflicting, or
forged principal/context input fails before dispatch.

Bypass mode retains its command-resolution/execution semantics, but it does not gain permission to
skip the no-follow witness/current-principal validation required to identify the physical shim.

MANAGER MANIFEST CONTRACT

After IH validation, normal product manager loading consumes only:

- base: `A/manager_hooks.yaml`
- optional overlay: `A/manager_hooks.local.yaml`

`ManagerHintEngine::new` and path helpers receive typed IH/A explicitly. A conflicting ambient B,
`SUBSTRATE_MANAGER_MANIFEST`, `dirs::home_dir`, common ambient substrate-home resolution,
compile-time `CARGO_MANIFEST_DIR`, repository discovery, CWD, or PATH cannot select either normal
product manifest. An existing separately labeled diagnostic override may remain only if source
closure proves it is explicit, isolated, non-product, and cannot satisfy S1 proof; otherwise it
must fail closed. Missing A-base manifest preserves the existing no-engine behavior. Manifest
parsing, rule selection, hint matching, emission/deduplication, and output are frozen.

TELEMETRY MAPPING AND FINAL FACTORY CUTOVER

When world telemetry is requested, `collect_world_telemetry` receives the validated host carrier
and its exact verified platform mapping explicitly. It may not reread environment, trace state,
home/profile, CWD, repository location, default socket/pipe, ambient Lima/WSL selection, or a
contextless factory.

Because physical shim commands expose no instance selector, direct platform telemetry may use only
the exact product default Lima/WSL instance and default normalized pipe under the control-pack
mapping rules. Observation may be read-only and mapping-only. S1 may not create/start/stop a
VM/distro, provision, launch/replace/kill/wait a forwarder, bind/unlink a socket/pipe, mutate a
listener, delete a PID/artifact, retry convergence, or exercise teardown/drop behavior. If the
required existing platform state cannot be verified without lifecycle action, fail before factory
construction and report telemetry unavailable/failure according to the existing non-authority
error semantics; do not fall back to an ambient/default backend.

The telemetry factory call uses the exact validated pair and any already-required explicit project
path derived from the same request/witness, never ambient CWD or repository discovery.

Immediately before final cutover, exact source closure must prove:

- `collect_world_telemetry` no longer calls the zero-argument factory;
- no other macOS/Windows product caller calls it;
- the R replay caller uses only the typed factory; and
- the only remaining zero-argument use, if any, is platform-independent Linux internal behavior.

Then remove or fail-close the zero-argument macOS/Windows factory before backend construction.
Tests must prove ambient/default constructor paths are unreachable. The typed factory remains
unchanged.

SEMANTIC AND LIFECYCLE FREEZE

Preserve existing behavior for:

- shim recursion/depth/session/caller tracking and clean command search;
- bypass and real-binary selection/execution;
- manager manifest parsing, platform resolution, rules, hint matching, and payloads;
- broker/profile/policy evaluation;
- trace context, policy Git lookup, span creation/finish, execution-log schema/redaction, and
  writer/rotation/retention;
- telemetry world handle, fs-diff request, warning/nonfatal collection semantics after valid
  factory construction; and
- all deployment, replacement, migration, recursive removal, installer, platform lifecycle,
  forwarding, socket/pipe/listener, process, PID, and deletion behavior.

S1 does not bind the explicit product trace/policy posture; S2 owns that physical-shim migration
and T owns final global compatibility removal.

TEST-DRIVEN PROOF

Use deterministic temp fixtures, fakes, and colocated seams. Cover at least:

- absolute, relative, and bare invocation witnesses;
- exact `A/shims/<command>` shape and no-follow file identity;
- absolute/nonempty PATH entry filtering, zero candidates, multiple candidates independent of
  PATH order, unrelated earlier executable, symlink/reparse ancestors, wrong target, and
  noncanonical/empty command rejection;
- current Unix account+UID and Windows account+SID construction/round trip, plus forged/conflicting
  principal rejection before dispatch;
- custom A under conflicting ambient B with zero B access;
- exact A-base/A-overlay manager paths, missing base behavior, and no ambient/repo/manifest-env
  fallback in normal product mode;
- explicit telemetry carrier+mapping projection and typed factory call;
- missing/malformed/mismatched/wrong-platform/wrong-transport/wrong-commitment input fails before
  factory/backend/session construction;
- zero-argument macOS/Windows factory removal or fail-closed behavior and unchanged Linux behavior;
- source closure showing no remaining contextless macOS/Windows caller;
- unchanged bypass, resolver, policy, trace, logging, hint, fs-diff, and command-dispatch
  differentials; and
- no test performs deployment, replacement, migration, removal, real VM/WSL realization,
  forwarding, provisioning, listener/socket/pipe mutation, process replacement/kill/wait, PID
  deletion, teardown, or convergence.

REQUIRED VERIFICATION

Run:

- `cargo fmt --all -- --check`;
- `cargo check --locked -p substrate-shim`;
- `cargo test --locked -p substrate-shim -- --nocapture`;
- `cargo clippy --locked -p substrate-shim --all-targets -- -D warnings`;
- `cargo check --locked -p world-backend-factory`;
- `cargo test --locked -p world-backend-factory -- --nocapture`;
- `cargo clippy --locked -p world-backend-factory --all-targets -- -D warnings`;
- exact focused context/witness, manager-manifest, telemetry, and factory-cutover filters;
- honest macOS/Windows target check/test-no-run attempts when supported, reporting unavailable
  SDK/linker/toolchain as limitations rather than success or product regression;
- external cfg-focused harnesses only when native cfg paths compile out on the Linux host, copying
  only the allowlisted subject plus minimum unchanged dependencies and stubbing all lifecycle I/O;
- exact source assertions for unique witness, current-principal binding, A-only manager paths,
  explicit typed telemetry, no contextless platform factory caller, typed-factory byte identity,
  no trace/policy migration, and no lifecycle reachability;
- exact frozen hashes or semantic comparisons for bypass, resolver, policy, logger/trace,
  deployment/migration/removal, factory typed path, and platform lifecycle behavior;
- `git diff --check` and staged `git diff --cached --check`;
- exact unstaged/staged file, symbol, and test allowlist checks;
- confirmation that every manifest, lockfile, replay/shell/trace/backend implementation,
  installer/script/lifecycle file, control document, and generated artifact is unchanged; and
- `gitnexus_detect_changes()` before commit, with every reported flow inspected.

Linux-native commands that compile out macOS/Windows paths are baseline checks only. Static or
harness proof must be labeled honestly. Native Windows mapping remains owned by R2-3Z.

SUBJECT FINGERPRINT

After deterministic formatting/checks and before discovery review:

1. record the pre-edit base commit;
2. build a sorted manifest containing that commit plus every exact changed subject path, its Git
   mode (or `NEW`), and `git hash-object --no-filters` blob ID (or `MISSING`);
3. include only paths inside the exact S1 production/test allowlist; and
4. SHA-256 the manifest and use `sha256:<digest>` as the review subject fingerprint.

BOUNDED REVIEW

Persist and validate one actual V1 review-cycle JSON record with packet ID `A1.1d-5R2-3S1` using
`llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py`. Use fresh read-only
lenses for:

1. invocation-witness uniqueness/no-follow identity/current-principal construction, ambiguous-path
   rejection, and no ambient prefix authority;
2. A-only manager manifest/overlay selection, explicit telemetry IH/PM validation, exact typed
   factory projection, and no lifecycle activation; and
3. final zero-argument platform-factory cutover/source closure, exact file/symbol/test scope,
   frozen bypass/policy/trace/logger/deployment/lifecycle behavior, GitNexus undercoverage, and
   static/native-proof honesty.

Apply the common causal bounded-review contract: one complete-subject discovery burst, one
consolidated P1/P2 remediation, one different-fresh closure, and at most two immediately causal
supplemental cycles. P1/P2 block. Valid unfixed P3/P4 must be deduplicated or added to `06` only
through the separately authorized inventory path. The final record must validate, end CLEAN, have
zero open P1/P2, and completely dispose P3/P4. CLEAN is terminal.

LANDING GATES

Before commit, require:

- exact S1 production/test/symbol containment;
- all required checks or honestly preserved pre-existing/environment limitations;
- independently recomputable subject fingerprint;
- validated CLEAN V1 review record;
- verified typed-factory byte/behavior identity;
- exact source closure proving no contextless macOS/Windows product caller remains;
- verified zero-argument macOS/Windows removal/fail-closed cutover and Linux parity;
- `gitnexus_detect_changes()` with every reported flow inspected;
- clean staged diff with no generated/analyzer churn; and
- live target still equal to the expected base commit/tree.

Then create one Conventional Commit, normal fast-forward push `HEAD` to the target ref, verify live
commit/tree and 0/0 divergence, refresh GitNexus on the landed commit, restore only
analyzer-generated `AGENTS.md`/`CLAUDE.md` churn, require the index current at the landed commit,
and finish with a completely clean task worktree. Never force-push.

TERMINAL RECEIPT

Send exactly one `codex.top-level-task-receipt.v1` message to the bound meta task as the last
external action. For success use `LANDED_CLEAN` and include task identity/nonce, expected base,
landed commit/tree/live ref, sorted changed paths, subject fingerprint, validated review record
and digest, finding disposition, checks/limitations, GitNexus landed-index-current confirmation,
clean 0/0 status, completion/non-claim boundary, and `next_increment=R2-3S2`.

For failure, send the exact blocked status, evidence, required authority/platform, and a copy-ready
handoff prompt. Do not begin, render, or dispatch R2-3S2.

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

Do not generate the next increment prompt and do not begin R2-3S2. The meta
orchestrator owns independent verification and subsequent dispatch.
