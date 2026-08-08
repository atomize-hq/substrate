# Increment contract — AUX-R3-MAC-EVIDENCE-RECOVERY-R6

## Objective and completion claim

Land only the recovery plan's PM-bound dual-session guest-pairing closure. Signed pairing uses
exactly one PM-bound data session and one independent PM-bound operator-TTY session. Both sessions
share one immutable authority binding and have distinct session identities. The retained attested
host control terminal is the sole source of the complete confirmation values. Transport continuity
is evidence, never authority.

## Exact source, publication, and successor

- Expected base: `a9626bc10a1737416c1d8f7f48630850118f3e6a`
- Expected tree: `3f7cdf20125a847a3dfaa472d3797759b5cf6737`
- Target: `origin refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- Required ancestor: `270f6e55e1a94b7e2f9b2667e605980d2e50579c`
- Publication: one normal fast-forward commit after R6 packet review and final combined recovery
  review are both validated terminal `CLEAN`.
- Sole receipt successor: `EVIDENCE:R3-MAC-IMP-01`.
- Do not dispatch or execute evidence. Meta independently verifies the receipt and owns evidence
  dispatch.

## Exact production and symbol fence

1. `crates/common/src/managed_artifact.rs` and `crates/common/src/lib.rs`, only:
   `GuestPublisherPairingDataSessionV1`, `GuestPublisherPairingOperatorTtySessionV1`,
   `GuestPublisherPairingSessionBindingV1`, `GuestPublisherPairingHostRecordV1`, and their exact
   canonical validation and transition helpers.
2. `crates/shell/src/execution/managed_lifecycle.rs` and
   `crates/shell/src/execution/managed_lifecycle/macos_client.rs`, only the typed issue, relay, and
   response-attestation calls for the two session tags.
3. `src/bin/substrate-lifecycle-control.rs`, only:
   `guest_publisher_pairing_direct_interactive_v1`, `display_guest_pairing_challenge_v1`,
   `issue_lima_guest_pairing_ticket_v1`, `advance_lima_guest_pairing_record_v1`, and
   `consume_lima_guest_pairing_ticket_v1`.
4. `src/bin/substrate-lifecycle-linux.rs`, only the exact PM-bound Lima guest executor,
   independent controlling-TTY opener, `publish_guest_pairing_intent_v1`,
   `validate_resumable_guest_pairing_intent_v1`, `load_transcript_input_v1`,
   `commit_guest_publisher_bootstrap_v1`, and ticket/transcript/intent verification. This is a Lima
   guest executable surface, not an ordinary Linux host provider route.
5. `src/bin/substrate-lifecycle-macos.rs`, only:
   `issue_lima_guest_pairing_ticket_v1`, `advance_lima_guest_pairing_record_v1`,
   `consume_lima_guest_pairing_ticket_v1`, `open_mac_guest_pairing_record_v1`, and
   `compare_and_swap_mac_guest_pairing_record_v1`.

Minimum immediate private helpers inside these seven production paths are authorized only when
directly reachable from a named R6 symbol and recorded in the path-and-symbol ledger. No other path,
public symbol, selector, endpoint, transport, principal, command, platform provider, or donor hunk
is mutable.

Explicitly remove or reject every `GuestPublisherPairingGuestChannelV1`,
`GuestPublisherPairingStdioPhaseV1`, `GuestPublisherPairingStdioFrameV1`,
`guest-pairing-stdio` command, one-child-channel exclusivity rule, and extra selector. Do not retain
a compatibility alias for them.

## Exact two-session authority contract

The data session carries only the signed ticket and typed `Hello`, signed transcript, and
guest-anchor frames. It never carries operator confirmation, arbitrary commands, a caller-supplied
ticket/record/key, or a second transport selector.

The operator-TTY session opens an independently controlling guest TTY, displays ticket scope and
fingerprint, and receives manual exact entry of the full fingerprint, challenge, and literal
`PAIR EXACT SUBSTRATE GUEST PUBLISHER`. It never carries ticket/transcript frames, aliases data
stdin/stdout, receives confirmation through argv/environment/file/pipe/automation, or performs a
guest mutation.

Both sessions must bind the same scope ID, `PlatformBootstrapMappingV1` commitment, guest machine
identity, source commit/tree/ref, staged guest-executor digest, Stage-1 record digest, ticket
challenge ID, host-record generation, and one fresh `pairing_session_nonce`. Their session IDs must
be distinct. The host record records both exact identities and start/end/failure observations.
Any shared-field, nonce, identity, descriptor, ticket, record-generation, or CAS mismatch preserves
state and rejects before mutation.

The attested host control is the only display source for the complete host-key fingerprint,
challenge ID, and challenge, and prints them only on its retained controlling terminal. The guest
binds only a non-reusable confirmation commitment to the protected generation-CAS record before
intent publication. Reusable confirmation input is never persisted, logged, projected, or returned.
Ticket/data/transcript/receipt/guest display cannot supply confirmation.

## Durable transition and retry table

1. Before guest intent publication, loss of either session/TTY preserves the record. A fresh
   attempt reopens both sessions and repeats host-terminal-to-guest-TTY confirmation. It never
   silently adopts ticket, nonce, or record.
2. From `GuestStateRootDurable` through `HelloDurable`, loss must exact-rejoin the same
   PM/machine/source/artifact/session nonce and generation-CAS record. Only a byte-identical durable
   hello may replay. Never mint a replacement ticket, key, nonce, or transcript.
3. From `TranscriptDurable` through `TicketConsumed`, loss may reopen transport observation only,
   replay only byte-identical durable hello/transcript, retain the consumed record, and reject
   duplicate or alternate-instance replay.
4. Any identity, descriptor, ticket, TTY, record, expiry, replay, or CAS mismatch performs no
   resume and no mutation and preserves evidence for the closed status.

Prove kill/reconnect, changed hello/nonce/key, transcript replay, stale generation, consume retry,
alternate transport observation, substitution, session swap, expiry, and deletion/Drop boundaries.

## Test and review fence

Tests may change only focused inline tests in the seven production paths plus:

- `crates/shell/tests/managed_lifecycle_v1.rs`
- `tests/mac/lifecycle_r3.sh`

Required cases include shared binding with distinct IDs, host-only display source, rejection of
data/ticket-derived confirmation, missing host terminal, missing/non-controlling guest TTY, no
stdio alias, no third session, every transition-table loss/replay/expiry row, wrong binding, stale
CAS, and Linux-host-unreachable source shape.

R6 review metadata may be created only at:

- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r6-review-authority-security.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r6-review-lifecycle-convergence.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r6-review-allowlist-evidence.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r6-review-cycle-record.json`

The same fresh three-lens R6 review must also perform the final combined implementation audit of the
entire recovery source range `d4bd729de8d344dfc567aaa62203838f68c2c895..candidate`, with
exact changed-file union, packet-to-path attribution, cross-packet joins, and R1-R6 receipt/review
lineage recorded in the R6 reports. If that combined audit finds a valid P1/P2 outside the R6 source
fence, stop for explicit meta adjudication rather than editing an earlier packet's frozen surface.

`llm-last-mile/runtime-refactor/06-review-finding-inventory.md` is conditionally permitted only for
mechanical tracking of a valid new P3/P4. Do not remediate P3/P4. No other documentation or review
path is authorized.

Use the repository-defined causal sequence: one discovery review or same-subject three-lens burst,
remediate valid P1/P2, then one different fresh closure reviewer. At most two supplemental cycles
may address P1/P2 directly caused or unmasked by the immediately preceding remediation under
unchanged authority/risk. `CLEAN` ends the loop. Validate the record with
`review-control/validate_review_cycle.py` after every cycle and before publication.

## Frozen surfaces and native boundary

Run-only and immutable: all scripts except already-landed R5 wrappers, all Windows code, ordinary
Linux host provider behavior, direct SSH/VSock/TCP selectors, native evidence artifacts, historical
R3 MAC review files, and R1-R5 product/review files except as combined read-only review subjects.
Do not install, code-sign, bootstrap, pair, or mutate Keychain, XPC, Lima, known-hosts, sockets, or
any native state. The official evidence task alone proves real sessions and restoration.

The preserved donor `/Users/spensermcconnell/.codex/worktrees/60cc/substrate` is read-only evidence.
Manually recreate only the selected R6 closure. Never copy a donor tree, bulk-apply its diff, mutate,
reset, clean, checkout, commit, or publish the donor.

## Skills, impact analysis, and agents

Load `.agents/skills/using-agent-skills/SKILL.md` first and invoke the relevant hydrated
repository-local skills, including `orchestrate-top-level-tasks`, `source-driven-development`,
`test-driven-development`, `incremental-implementation`, `security-and-hardening`,
`api-and-interface-design`, `code-review-and-quality`, and `git-workflow-and-versioning`.

Before editing each existing symbol, run GitNexus upstream impact when the existing index can
answer and always complete exact manual caller/callee, enum/decoder, guest/TTY/data-session,
descriptor/no-follow, timeout/cancellation/Drop, receipt, replay/retry, CAS, and cfg analysis. Do
not run `npx gitnexus analyze` or permit tracked context injection. Degraded/UNKNOWN graph output is
inconclusive. HIGH/CRITICAL results confined to this named closure are mandatory review inputs, not
permission to widen scope.

Every subagent and reviewer uses `gpt-5.6-terra` at Extra High reasoning under the explicit user
override. Keep editing ownership disjoint where practical. Reviewers are fresh, read-only, and
independent of implementation authors.

## TDD and deterministic proof

Use focused failing tests before behavior changes. Required proof includes:

1. Focused common, shell client/control, macOS executor, and exact Lima guest-executor tests.
2. `cargo test -p shell --test managed_lifecycle_v1 -- --nocapture` or the exact workspace-valid
   equivalent.
3. `cargo test --bin substrate-lifecycle-macos -- --nocapture` and focused common MAC tests.
4. `cargo check --bin substrate-lifecycle-control --bin substrate-lifecycle-macos`.
5. `cargo check --target x86_64-apple-darwin --bin substrate-lifecycle-control --bin substrate-lifecycle-macos`.
6. Focused `substrate-lifecycle-linux` guest-only tests/build available on this macOS host, plus
   static proof no ordinary Linux host provider route became reachable.
7. `bash tests/mac/lifecycle_r3.sh`, `cargo fmt --all -- --check`, and `git diff --check`.
8. Exact path/symbol/public-API allowlist, changed-byte credential scan, frozen Windows/WSL
   byte-identical hashes, Linux-body differential for every shared/Linux-named path, and positive
   macOS cfg proof.

Authority amendment `0016-r3-mac-only-platform-proof` supersedes stale planning text that makes
Linux or Windows target compilation blocking. Do not provision cross toolchains, run Windows or
ordinary Linux target compile as a gate, remediate their baseline failures, or change their
behavior. Static hashes/differentials and the Lima guest-only proof are the authorized boundary.

Before commit, run `gitnexus_detect_changes` when available. If unavailable, record the exact manual
fallback. Requery the live remote and require it still equals the expected base. Publish exactly one
normal fast-forward commit, verify the remote equals the commit with 0/0 divergence, and finish with
a clean task worktree/index.

## Terminal receipt

Send one `codex.top-level-task-receipt.v1`. Success is `LANDED_CLEAN` and must include the bound
task identity, expected base/tree, commit/tree/live remote, R6 subject fingerprint, full changed
paths, validated R6 review record/digest, explicit combined R1-R6 audit result and range, P1/P2 and
P3/P4 disposition, checks, change detection, MAC-only containment, and
`next_increment: EVIDENCE:R3-MAC-IMP-01`.

On any stop, preserve this task/worktree without reset, clean, rebase, archive, or destructive
reconciliation and return the exact blocker, evidence, required authority, and continuation prompt.
Sending the receipt to meta must be the final tool action.
