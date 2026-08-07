Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for AUX-R3-MAC-EVIDENCE-RECOVERY-R4. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-a1-1d-5r3-mac-20260806-41f97e1c570d
- dispatch_nonce: f71d4838dead60f64a351dfc18b394d912949f589efbd94425d24c88a0eae981
- meta_thread_id: 019fd8c3-b12c-7e73-919f-898600ab0f64
- meta_host_id: local
- increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R4
- packet_id: AUX-R3-MAC-EVIDENCE-RECOVERY-R4
- next_increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R5

INCREMENT-TASK IDENTITY BARRIER

Do not edit, delegate, run implementation checks, or publish until the meta orchestrator sends a
follow-up binding your own real increment-task thread ID and host ID to this dispatch nonce. Echo
those exact IDs in every terminal receipt.

REPOSITORY BOUNDARY

Work only in the task-assigned checkout:

/Users/spensermcconnell/.codex/worktrees/858f/substrate

Never mutate these protected checkouts:

- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate
- /Users/spensermcconnell/.codex/worktrees/eb49/substrate
- /Users/spensermcconnell/.codex/worktrees/60cc/substrate
- /Users/spensermcconnell/.codex/worktrees/9afb/substrate
- /Users/spensermcconnell/.codex/worktrees/bacb/substrate
- /Users/spensermcconnell/.codex/worktrees/05f9/substrate
- /Users/spensermcconnell/.codex/worktrees/1958/substrate
- /Users/spensermcconnell/.codex/worktrees/8fd3/substrate
- /Users/spensermcconnell/.codex/worktrees/e2c8/substrate
- /Users/spensermcconnell/.codex/worktrees/abc3/substrate
- /Users/spensermcconnell/.codex/worktrees/a943/substrate
- /Users/spensermcconnell/.codex/worktrees/2fdb/substrate
- /Users/spensermcconnell/.codex/worktrees/46c9/substrate

Canonical starting state:

- publication mode: declared by the increment contract
- remote: origin
- target ref: refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
- expected base commit: b12cb6c6e2dc326b4150291ef30676171df34f58
- expected base tree: ddcd56b3a10b02cbe1a978e8522293a0b30e2dae
- required ancestor: 270f6e55e1a94b7e2f9b2667e605980d2e50579c

Before editing, query the publication-mode authority: the live remote for `remote`, or the exact
local branch ref and worktree for `local`. Verify the exact base, tree, ancestry, cleanliness, and
any required index state. If they differ, send BASE_DRIFT or BLOCKED_CONTRADICTION. Do not
reconcile, merge, rebase, reset, clean, or force-push.

SUBAGENT ORCHESTRATION

Use repository-required model/reasoning settings for every subagent. Complete required pre-edit
impact analysis before any subagent edits an existing symbol. Give editing subagents mutually
exclusive ownership when practical. Use fresh read-only subagents for independent review. Do not
allow a reviewer to review implementation it authored.

INCREMENT CONTRACT

# Increment contract — AUX-R3-MAC-EVIDENCE-RECOVERY-R4

## Objective

Implement only the minimum protected macOS foundation already required by the landed runtime-
refactor control pack: System-Keychain non-exportable P-256 authority, canonical SPKI/P1363-low-S
handling, fixed XPC peer admission before decode, Stage-1 validation, and the minimum signed
ticket/protected host-record generation-CAS/replay core. Do not implement a pairing transport,
mapped lifecycle action, guest command, bootstrap effect, or native evidence action.

## Exact source and successor

- Expected base: `b12cb6c6e2dc326b4150291ef30676171df34f58`
- Expected tree: `ddcd56b3a10b02cbe1a978e8522293a0b30e2dae`
- Target: `origin refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- Publication: one normal fast-forward commit after validated terminal review `CLEAN`
- Sole successor: `AUX-R3-MAC-EVIDENCE-RECOVERY-R5`

## Exact production fence

1. `Cargo.toml`: only add root-package `libc = "0.2"` and
   `sha2 = { workspace = true }` when required by the named macOS implementation.
2. `Cargo.lock`: only add `libc` and `sha2` to the `substrate` package dependency list. No package
   version, checksum, feature, source, or other dependency edge may change.
3. `crates/common/src/lib.rs`: only re-export the named R4 common types/helpers.
4. `crates/common/src/managed_artifact.rs`: only:
   - `MacPublisherControlAuthorityV1`
   - `LimaStageOneAuthorizationV1`
   - `GuestPublisherPairingChallengeV1`
   - `GuestPublisherPairingTicketV1`
   - `GuestPublisherPairingHostRecordV1`
   - their canonical encoders
   - `validate_lifecycle_publisher_protected_state_v1`
   - `validate_guest_publisher_pairing_ticket_v1`
   - the exact immediate SPKI, P1363-low-S, expiry, generation-CAS, and replay helpers required by
     these types.
5. `src/bin/substrate-lifecycle-macos.rs`: only:
   - `open_system_keychain_protected_state_v1`
   - `compare_and_swap_mac_publisher_protected_state_v1`
   - `open_mac_lima_guest_pairing_stage_one_record_v1`
   - `validate_mac_lima_guest_pairing_stage_one_record_for_issue_v1`
   - `export_mac_p256_spki_der_v1`
   - `normalize_mac_p256_signature_p1363_low_s_v1`
   - `run_mac_xpc_publisher_v1`
   - `accept_mac_xpc_connection_v1`
   - `attest_mac_xpc_audit_token_v1`
   - `verify_mac_control_designated_requirement_v1`
   - `issue_lima_guest_pairing_ticket_v1`.

Minimum private constants, macOS FFI declarations, serialization/error adapters, and immediate
private helpers within these same files are authorized only when directly necessary for and
reachable from a named R4 type/function. Maintain a path-and-symbol ledger proving that closure.
No generic request wrapper, new endpoint, channel/session abstraction, mapped action,
caller-selected operation, or unrelated FFI family is authorized.

## Test and review fence

- Focused inline unit tests only in the named Rust source files.
- `tests/mac/lifecycle_r3.sh` only for source-shape negatives proving audit-before-decode,
  canonical fixed key/service/account handling, Stage-1-before-issue ordering, and absence of the
  forbidden channel/action surfaces.
- Review metadata only:
  - `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r4-review-authority-security.md`
  - `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r4-review-lifecycle-convergence.md`
  - `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r4-review-allowlist-evidence.md`
  - `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r4-review-cycle-record.json`

The global `06-review-finding-inventory.md` is conditionally permitted only if a fresh reviewer
reports a valid P3/P4 that the authoritative process requires to be tracked. It remains outside the
implementation subject and creates no remediation/review cycle. No other path is authorized.

## Required security and lifecycle properties

- Use exact System Keychain service `com.substrate.lifecycle.v1` for the signing-key,
  current-anchor, bootstrap-intent, and challenge-keyed pairing-record accounts that this packet
  actually needs. The host P-256 signing key is non-exportable.
- Export and hash canonical SPKI DER. Accept exactly canonical P1363 low-S signatures; malformed
  DER/SPKI, wrong curve/key, high-S, or alternate wire representations fail closed.
- Read and attest the actual XPC connection audit token and exact designated requirement before
  decoding any request bytes. Never trust caller-supplied peer identity or operation selection.
- Open and validate the durable Stage-1 record and its PM/machine/source/artifact joins before
  ticket issue. Expired, replayed, wrong-peer, wrong-requirement, wrong-PM, wrong-machine,
  wrong-source, wrong-artifact, and stale-generation inputs fail before effect.
- Protected host-record transitions are canonical, generation-CAS guarded, one-use, and preserve
  replay/ambiguity evidence. R4 does not execute a mapped action or pairing session.

## Donor boundary

`/Users/spensermcconnell/.codex/worktrees/60cc/substrate` is read-only evidence. Manually recreate
only the selected R4 closure. Never bulk-copy its diff or mutate/reset/clean/checkout/commit it.
Explicitly discard every `lima-stdio-v1` channel/frame/phase type, generic request wrapper,
retirement prototype, raw `lima-action`, mapped action, and unrelated FFI bulk.

## Skills, impact analysis, and subagents

Load `.agents/skills/using-agent-skills/SKILL.md` first and invoke the relevant hydrated
repository-local skills, including `orchestrate-top-level-tasks`, `source-driven-development`,
`test-driven-development`, `incremental-implementation`, `security-and-hardening`,
`api-and-interface-design`, `code-review-and-quality`, and `git-workflow-and-versioning`.

Before editing every existing symbol, run GitNexus upstream impact when the existing index can
answer and perform exact manual caller/callee, cfg, FFI ownership/lifetime, Drop, Keychain, XPC,
serialization, and state-transition analysis regardless. Do not run `npx gitnexus analyze`; tracked
metadata injection is forbidden. Treat degraded/UNKNOWN results as inconclusive. Warn and review
HIGH/CRITICAL results, but a result confined to the named R4 closure is a review obligation rather
than permission to widen scope.

Every subagent and reviewer must use `gpt-5.6-terra` at Extra High reasoning under the explicit
user override. Give editing subagents disjoint ownership where practical. Reviewers must be fresh,
read-only, and independent of the implementation they review.

## TDD and deterministic proof

Use focused failing tests before each behavior change. Required proof includes:

1. Focused `substrate-common` unit tests for canonical encoders, protected-state validation,
   malformed SPKI, high-S, expiry, replay, stale generation-CAS, and binding mismatches.
2. Compile and focused test of `substrate-lifecycle-macos` on the native arm64 macOS target. The
   previously deferred missing `sha2`, `AuditTokenV1` ownership/visibility, and XPC pointer-cast
   failures are mandatory R4 closure gates.
3. `x86_64-apple-darwin` compile proof when available without host mutation; absence of that target
   is nonblocking if native arm64 proof and the exact cfg/FFI review are complete.
4. `tests/mac/lifecycle_r3.sh` source-shape negatives.
5. `cargo fmt --all -- --check`, exact path/symbol/manifest/lockfile allowlist, changed-byte secret
   scan, and `git diff --check`.
6. A manual FFI ownership/lifetime and actual-peer-before-decode review.

Do not run Linux or Windows target commands. Prove those platforms remain outside the subject by
exact path/diff inspection only. Do not install, code-sign, touch System Keychain, launch an XPC
service, mutate Lima, invoke lifecycle effects, or create official evidence.

## Causal review and publication

Freeze the exact implementation/test subject excluding review metadata. Run one same-subject
three-lens discovery burst (authority/security, lifecycle/convergence, allowlist/evidence),
consolidate and remediate valid P1/P2, then run one different fresh closure review. At most two
supplemental causal cycles are permitted only for a P1/P2 directly caused or unmasked by the
immediately preceding remediation under unchanged authority and risk. `CLEAN` ends the loop.
Valid P3/P4 are recorded and do not create remediation or review cycles.

Validate the review-cycle record with the repository validator after every cycle. Before
publication, run change detection with the existing index and complete manual fallback, require
zero open P1/P2, exact path/symbol/manifest closure, all deterministic gates, unchanged live base,
and a clean index/worktree. Publish exactly one normal fast-forward commit; never force-push,
merge, rebase, reset, or clean.

## Stops and receipt

Return `BASE_DRIFT` for target movement; `BLOCKED_CONTRADICTION` for identity, donor, manifest,
lockfile, or exact-fence contradiction; `BLOCKED_SCOPE_EXPANSION` if a new path, authority carrier,
endpoint, channel/session, selector, principal, mapped action, guest command, or unrelated FFI
surface is needed; `BLOCKED_REVIEW` for unresolved P1/P2 or invalid causal lineage. Preserve the
task and worktree exactly on every stop and never request archival.

On success send `codex.top-level-task-receipt.v1` with `LANDED_CLEAN` and
`next_increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R5` to meta task
`019fd8c3-b12c-7e73-919f-898600ab0f64` on host `local`. That send must be your final tool action.

COMMON TERMINAL CONTRACT

Before publication:

1. Complete every increment-specific check and proof gate.
2. Verify the exact file/symbol/test allowlist.
3. Run required change detection.
4. Complete and validate the bounded review sequence.
5. Require zero open blocking findings.
6. Fetch/query the live target again and require it still equals the expected base.

For publication mode `remote`, when authorized by the increment contract, create one atomic commit
and perform a normal fast-forward push of HEAD to refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap. Never force-push. Verify the live
remote equals the landed commit, refresh required indexes, and finish clean.

For publication mode `local`, create exactly the contract-authorized two-commit chain:
`expected base -> reviewed work commit -> mechanical control commit`. Keep the isolated branch and
worktree clean. Do not merge or push. Record the local branch ref, worktree, both commits/trees,
both exact changed-file inventories, their sorted union, and an unchanged remote observation.

Send a codex.top-level-task-receipt.v1 message to the meta task. For remote success, use
LANDED_CLEAN. For local success, use CLOSED_CLEAN. Include your bound increment-task thread/host
IDs, expected base, exact publication identity, changed paths, subject fingerprint, validated
review record and digest, finding disposition, checks, change detection, clean status, and next
increment.

For failure, send the exact blocked status, evidence, required authority or platform, and a
complete continuation/handoff prompt.

The send_message_to_thread call is your final tool action. After it succeeds, make no more tool
calls or repository changes. Return only the human-readable final report.

Do not generate the next increment prompt and do not begin AUX-R3-MAC-EVIDENCE-RECOVERY-R5. The meta
orchestrator owns independent verification and subsequent dispatch.
