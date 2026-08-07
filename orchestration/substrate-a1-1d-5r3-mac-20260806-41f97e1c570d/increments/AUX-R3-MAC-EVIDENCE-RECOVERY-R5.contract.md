# Increment contract — AUX-R3-MAC-EVIDENCE-RECOVERY-R5

## Objective

Land only the trusted mapped-lifecycle control bridge defined by the recovery plan. The shell and
fixed wrapper may submit the two ordinary typed branches—`stage_one_create` and
`post_pm_action`—and nothing else. A Stage-1 record never becomes forwarding, post-PM, or publisher-
bootstrap authority. The direct publisher-bootstrap branch remains hidden from scripts and is
admitted only through a separately issued `PublisherBootstrapAuthorizationV1` joined to the
retained interactive terminal/channel.

## Exact source and successor

- Expected base: `05d655fa1458a276f179d12057cbda772cc51eb6`
- Expected tree: `845694f733fccbfcd3703957bf4976da0ed59b40`
- Target: `origin refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- Publication: one normal fast-forward commit after validated terminal review `CLEAN`
- Sole successor: `AUX-R3-MAC-EVIDENCE-RECOVERY-R6`

## Exact production and symbol fence

1. `crates/shell/src/execution/managed_lifecycle.rs`:
   `ManagedLifecycleControlRequestV1`,
   `LifecyclePublisherClientV1::submit_publisher_request_v1`, and the exact closed tag decoder.
2. `crates/shell/src/execution/managed_lifecycle/macos_client.rs`:
   `submit_stage_one_absent_instance_create_v1`, `submit_post_pm_managed_action_v1`,
   `open_mac_xpc_channel_v1`, and `attest_mac_publisher_response_v1`.
3. `src/bin/substrate-lifecycle-control.rs`:
   `submit_mapped_lifecycle_v1`, `publisher_bootstrap_direct_interactive_v1`,
   `read_exact_bootstrap_confirmation_v1`, and the immediate branch/terminal admission functions.
4. `src/bin/substrate-lifecycle-macos.rs`:
   `handle_mac_publisher_request_v1`, `execute_mac_managed_action_v1`,
   `publish_mac_action_receipt_v1`, audit-token-before-decode XPC admission, and only the immediate
   descriptor-preserving known-host/SSH-UDS predecessor helpers.
5. `scripts/mac/lima-lifecycle.sh`:
   `load_mapped_lifecycle_v1`, `invoke_mac_lifecycle_control_v1`, and exactly two literal wrapper
   branches: `stage_one_create` and `post_pm_action`.

Minimum immediate private helpers inside these five paths are authorized only when directly
reachable from a named R5 symbol and recorded in a path-and-symbol ledger. No other path, public
symbol, wrapper selector, helper invocation, transport, endpoint, principal, generic request,
forwarding implementation, guest-session surface, or cross-platform adapter is mutable.

## Closed request map

`stage_one_create` maps only to absent `mac.lima.instance` / `Create` before PM finalization and
must carry the complete `InstallBootstrapContextCarrierV1`, pre-PM mapping,
`LimaStageOneAuthorizationV1`, and `ExecutorBuildEvidenceV1` joins. It rejects every other pair,
pre-existing/ambiguous instance, post-PM work, forwarding, ticket issue, guest projection, reuse,
or an unjoined create receipt.

`post_pm_action` accepts only the complete canonical `ManagedLifecyclePublisherRequestV1` plus
carrier/mapping/build joins and only the existing closed MAC role/action rows in the current
`04-contracts-and-gates.md` table:

- `mac.lima.instance`: `Start`, `Stop`, `Remove`, `Restore` only;
- the table-listed actions for `mac.lima.staged-workspace`, `mac.lima.guest-binary(kind)`,
  `mac.lima.guest-unit(kind)`, `mac.lima.guest-directory(kind)`, `mac.lima.guest-group`,
  `mac.lima.guest-membership(principal)`, `mac.lima.guest-private-home`,
  `mac.lima.layout-sentinel`, `mac.lima.publisher-executor`,
  `mac.lima.publisher-state-directory`, `mac.lima.publisher-service-unit`,
  `mac.lima.publisher-socket-unit`, `mac.lima.publisher-signing-key`,
  `mac.lima.publisher-current-anchor`, `mac.lima.publisher-bootstrap-intent`, and
  `mac.host.known-hosts-entry`;
- the table's fixed coupled `mac.lima.guest-service-state(kind)` actions.

Copy this map by reference; do not extend it. Non-requestable endpoint/Mach-service rows, every
`mac.publisher.*` bootstrap/service-state row, unknown fields/tags, and every unlisted pair are
decoder errors before XPC or effect. Tests must enumerate all accepted pairs and representative
plus generative unlisted-pair rejection.

`publisher_bootstrap_direct_interactive` is not accepted by the wrapper or ordinary submit path.
It is reachable only from hidden direct-interactive `substrate-lifecycle-control publisher-bootstrap`
and requires an independently issued, nonserialized `PublisherBootstrapAuthorizationV1` delivered
over the retained bootstrap channel and joined to the controlling terminal, image, build, peer,
and exact bootstrap record. Missing terminal/channel/authorization, script invocation, serialization,
or replay rejects before effect.

Required order is: tag validation -> fixed XPC audit token/designated requirement before decode ->
branch-specific recomputation -> exact Stage-1 absent create, complete post-PM join, or independent
direct-bootstrap authorization -> one mapped action -> durable receipt -> caller observation.
Ambiguous or unreceipted action stops preserving-first and cannot be blindly retried.

## Descriptor preservation

Known-host and SSH-UDS/socket predecessors may be opened only using retained descriptor and
no-follow identity. Record manifest/receipt identity before unlink, replacement, timeout,
cancellation, handle drop, or retry. Prove no `StreamLocalBindUnlink` replacement, no endpoint
fallback, and exact preserve/rejoin after every failure boundary. Do not edit any forwarding
implementation.

## Test and review fence

Tests may change only:

- `crates/shell/tests/managed_lifecycle_v1.rs`
- `tests/mac/lifecycle_r3.sh`

Required negatives include raw helper/action/generic request; missing/stale carrier, mapping, or
build; every invalid role/action; Stage-1 reuse for forwarding/post-PM; Stage-1-as-bootstrap;
missing/replayed direct bootstrap authorization; absent/noncontrolling terminal; audit-after-decode;
unreceipted/ambiguous receipt; caller ticket/record/key/operation; no-follow predecessor
substitution; unlink/timeout/cancellation/Drop/retry loss; and unattested XPC response.

Review metadata only:

- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r5-review-authority-security.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r5-review-lifecycle-convergence.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r5-review-allowlist-evidence.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r5-review-cycle-record.json`

`06-review-finding-inventory.md` is conditionally permitted only to mechanically track a valid new
P3/P4. Do not remediate or reclassify `RR-RF-0005` in this packet. No other path is authorized.

## Frozen surfaces

Do not change `scripts/mac/lima-warm.sh`, `scripts/mac/lima-stop.sh`, forwarding implementation,
guest-session code, common R4 ticket/record contracts, Windows code, ordinary Linux host code, or
evidence artifacts. Do not add a new role/action pair, selector, endpoint, transport, principal,
generic session broker, raw `lima-action`, direct helper path, or caller-selected XPC operation.

The preserved donor `/Users/spensermcconnell/.codex/worktrees/60cc/substrate` is read-only evidence.
Manually recreate only the selected R5 closure; never bulk-copy, mutate, reset, clean, checkout,
commit, or publish the donor.

## Skills, analysis, and agent settings

Load `.agents/skills/using-agent-skills/SKILL.md` first and invoke the relevant hydrated repository-
local skills, including `orchestrate-top-level-tasks`, `source-driven-development`,
`test-driven-development`, `incremental-implementation`, `security-and-hardening`,
`api-and-interface-design`, `code-review-and-quality`, and `git-workflow-and-versioning`.

Before editing each existing symbol, run GitNexus upstream impact when the existing index can
answer and always perform exact manual shell/caller/callee, enum/decoder, XPC, descriptor/no-follow,
timeout/cancellation/Drop, receipt, retry, and cfg analysis. Do not run `npx gitnexus analyze` or
inject tracked metadata. Treat degraded/UNKNOWN graph output as inconclusive. HIGH/CRITICAL results
confined to this named R5 closure are mandatory review inputs, not permission to widen scope.

Every subagent and reviewer uses `gpt-5.6-terra` at Extra High reasoning under the explicit user
override. Keep editing ownership disjoint where practical. Reviewers are fresh, read-only, and
independent of implementation authors.

## TDD and deterministic proof

Use focused failing tests before behavior changes. Required proof:

1. Focused shell client/control/executor tests and `crates/shell/tests/managed_lifecycle_v1.rs`.
2. `tests/mac/lifecycle_r3.sh`, including closed pair enumeration and forbidden-surface checks.
3. Native arm64 macOS compile/check for all touched binaries/crates.
4. `cargo check --target x86_64-apple-darwin` for the touched shell/control/MAC binary surface.
5. Descriptor preservation matrix covering unlink, timeout, cancellation, Drop, ambiguous receipt,
   and exact retry/rejoin.
6. Manual wrapper/XPC/descriptor caller review, `cargo fmt --all -- --check`, exact path/symbol
   allowlist, changed-byte secret scan, and `git diff --check`.

Run no Linux or Windows target command and make no cross-platform remediation. Prove platform
containment through exact static diff/cfg review. Do not install, code-sign, touch Keychain, launch
an XPC service, invoke the helper, mutate Lima/known-host/socket state, perform a lifecycle action,
or create native evidence.

## Review and publication

Freeze the exact seven-path implementation/test subject excluding review metadata and conditional
inventory bookkeeping. Run one same-subject three-lens discovery burst—authority/security,
lifecycle/convergence, allowlist/evidence—remediate valid P1/P2, then one different fresh closure
review. At most two supplemental causal cycles may address only P1/P2 directly caused or unmasked
by the immediately preceding remediation under unchanged authority and risk. `CLEAN` ends the
loop. Valid P3/P4 are mechanically tracked in `06` and create no remediation/review cycle.

Validate the review-cycle record after every cycle. Before publication, run change detection with
the existing index plus manual fallback, require zero open P1/P2, exact path/symbol/role/action/
descriptor closure, complete deterministic proof, unchanged live base, and clean index/worktree.
Publish exactly one normal fast-forward commit; never force-push, merge, rebase, reset, or clean.

## Stops and receipt

Return `BASE_DRIFT` for target movement; `BLOCKED_CONTRADICTION` for identity/donor/fence mismatch;
`BLOCKED_SCOPE_EXPANSION` if an unlisted path/symbol, new role/action, endpoint, transport, selector,
principal, cross-platform adapter, forwarding implementation, or R6 guest-session surface is
needed; and `BLOCKED_REVIEW` for unresolved P1/P2 or invalid causal lineage. Preserve the task and
worktree exactly on every stop and never request archival.

On success send `codex.top-level-task-receipt.v1` with `LANDED_CLEAN` and
`next_increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R6` to meta task
`019fd8c3-b12c-7e73-919f-898600ab0f64` on host `local`. That send must be the final tool action.
