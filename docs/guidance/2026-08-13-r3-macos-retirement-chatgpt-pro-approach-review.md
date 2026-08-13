# R3 macOS evidence-retirement ChatGPT Pro approach review

- **Date:** 2026-08-13
- **Task/spec:** Review the security-preserving approach for prospective R3 macOS/Lima evidence-only retirement and exact recovery of one precommit-less failed attempt.
- **Review mode:** Approach review
- **Model:** GPT-5.6 Sol
- **Effort:** Pro
- **Conversation:** https://chatgpt.com/c/WEB:953af78c-4c26-44c5-8df4-79b7a70b2176
- **Landed checkpoint reviewed:** `879a8c680ba14cd93b79ba2c1a2ed598e9687c90` / tree `b201ce8f058b438ec6e8665c84b09abf0388404f`
- **Answer SHA-256:** `9c7eb958f8659862aa017f2c1a968531536575e125905459ccd50c8e5221a577`

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Current approach at review time

Implementation was paused. The failed attempt and nonbaseline machine state were preserved. A tentative uncommitted retirement implementation had been discarded after source audit found missing exhaustive DAG binding, missing protected reservation/acknowledgement transitions, broad removal, and a helper-before-signer crash gap. No evidence task or MAC-CLOSEOUT was dispatched.

## Local reconciliation

### Confirmed against the live repository

- The branch and live remote are equal at the reviewed checkpoint, and the checkout was clean before this guidance file was created.
- `PublisherBootstrapAuthorizationV1` has an optional test-retirement commitment, while current macOS R6 ticket issuance leaves the guest test-retirement commitment null.
- The repository contains retirement-related data types and partial validators, but no installed executable path implements the complete host/guest evidence-retirement contract.
- `retire_mac_system_keychain_software_signer_v1` exists, rejects deletion while the protected wrapper survives, and is not wired into an executable evidence-retirement route.
- The authoritative R3 contracts require precommit before bootstrap, exact ordered component authority, protected CAS state, externally fsynced receipts and acknowledgements, guest-before-host teardown, reverse dependency order, kill-point recovery, and final baseline parity.
- The preserved failed attempt predates a non-null retirement commitment. A prospective precommit protocol therefore cannot be treated as its normal authority.

### Advisory additions that are not yet authoritative

The response proposes explicit terminal-boundary concepts—terminal survivor set, fail-closed terminal latch, conditional finalizer grant, terminal acknowledgement, and a separately identified post-Keychain finalizer. These are plausible corrections to the helper self-deletion/restart gap, but they are not currently adopted contract types or implementation authority. They require a top-level contract/planning decision and local source reconciliation before any implementation.

The response also treats the Keychain creator/ACL behavior as unresolved research. That is correct: the exact cause of SecurityAgent presentation, the effective access-control rule, and whether a retained/reproducible code identity can perform a no-UI exact delete must be established against a disposable surrogate before the live orphan is touched.

### Reconciled sequencing recommendation for user review

- Keep the pause and preserved state.
- Use one planning/research umbrella with two separately accepted outputs:
  1. prospective terminal-authority contract amendment;
  2. exact one-attempt orphan-recovery decision record.
- If accepted, dispatch separate implementation/operation tasks with no shared wire schema or authorization type:
  1. exact orphan recovery and baseline restoration;
  2. prospective evidence-retirement implementation;
  3. later operator-run native evidence only after remote equality and mirror refresh.
- General product uninstall, upgrades, multi-install ownership, and generalized orphan recovery remain deferred unless the planning work proves one is strictly required.

No dispatch or implementation is authorized by this advisory record.

## Prompt sent

```text
We are already working on this project task.

Task/spec: Review the best security-preserving approach for completing a native macOS/Lima R3 evidence-only publisher retirement protocol and recovering one already-failed disposable evidence attempt whose host System-Keychain P-256 signer survives but whose publisher service registration and generic-password state have already been removed. The review must reconcile the live landed implementation with the existing authoritative R3 contracts and plans. It must determine whether the prospective evidence-retirement protocol and the precommit-less orphan recovery should be one top-level task, ordered separate tasks, or require a contract amendment before implementation. This is advisory/planning guidance only; do not produce a code patch.

Current approach: Repository-derived facts are listed first; explicit inferences are labeled.

SELF-CONTAINED BOUNDED REVIEW PACKET

1. Live landed checkpoint
- Branch and live remote are equal and clean at commit 879a8c680ba14cd93b79ba2c1a2ed598e9687c90, tree b201ce8f058b438ec6e8665c84b09abf0388404f.
- The immediately preceding landed chain contains focused macOS Stage-1/R6 corrections, including fixed Lima atomic-publisher arguments, readiness observation/classification, fresh supplementary groups, canonical R6 response fields, handoff from the admitted prefix, and the latest attested R6 predecessor-replacement response field set.
- The latest commit accepts exactly four replacement response fields: status, r6_pairing_predecessor_v1, r6_pairing_continuation_v1, and xpc_attestation; it rejects unknown fields and has a focused regression.
- The latest manual native attempt got through signed Stage-1 and into R6 pairing. Its final observed failure was the now-fixed predecessor-replacement response validator mismatch.
- No retirement implementation was landed after that point. A tentative local implementation was audited, found unsound, and completely discarded. The worktree is clean at the public landed checkpoint above.

2. Current landed product surface relevant to retirement
- PublisherBootstrapAuthorizationV1 already has an optional test_retirement_commitment. Product bootstrap is expected to keep it null; evidence may precommit it.
- Existing common types include GuestPublisherRetirementReservationV1, PublisherTestRetirementCommitmentV1, PublisherTestRetirementAuthorizationV1/ReceiptV1/AcknowledgementV1, GuestPublisherTestRetirementCommitmentV1/AuthorizationV1/ReceiptV1/AcknowledgementV1, and unused-reservation proof/acknowledgement types.
- Some existing validators are only partial. For example, the landed GuestPublisherTestRetirementAuthorizationV1 presently binds reservation_id, ticket_sha256, a compact commitment, harness_public_key, and signature; it does not yet encode the full contract-described ticket-core, exact platform/build/machine/dispatch scope, exhaustive component DAG, before/after observations, counters, expiry, retained receipt-parent identity, and related joins.
- The currently issued macOS R6 guest ticket explicitly sets guest_test_retirement_commitment to null.
- The current executable macOS closed FD3 routes are publisher bootstrap and publisher service-state install/retire. There is no executable host or guest test-retirement route wired into the installed control.
- The current ordinary publisher service-state retirement is independently crash-journaled for launchd bootout and exact helper/plist/provenance file deletion. It is not the full evidence retirement protocol and does not delete lifecycle Keychain records or the signer.
- A landed function retire_mac_system_keychain_software_signer_v1(scope_id) exists. It takes the exact protected-state account lock, refuses deletion while the protected wrapper exists, derives the fixed scoped key tag, calls an exact Security.framework delete, and verifies final absence. It is not wired to an executable retirement path.
- The P-256 key is a permanent software EC key in the explicit System Keychain, selected by exact application tag and label. Queries include kSecUseAuthenticationUIFail and a single-keychain search list. The delete query additionally binds the exact SecKey reference through kSecMatchItemList.

3. Authoritative contract and plan requirements
The authoritative files are:
- llm-last-mile/runtime-refactor/03-phase-slice-map.md
- llm-last-mile/runtime-refactor/04-contracts-and-gates.md
- llm-last-mile/runtime-refactor/05-debug-regression-ledger.md

Their relevant requirements are:
- Product install/uninstall never retires the final publisher/anchor. Retirement is evidence-only and available solely in a disposable exact-absence scope that precommits external harness authority before bootstrap.
- The harness must create its ephemeral Ed25519 key and retained external receipt directory before host bootstrap. It signs a PublisherTestRetirementAuthorizationV1 binding the bootstrap core with the optional retirement slot forced null, complete ordered component identities/DAG and dispositions, baseline absence, source/build/platform/evidence/dispatch scope, counter limit, nonce/expiry, external directory identity, and exactly one preallocated macOS guest reservation.
- The final bootstrap and generation-one anchor must immutably carry the retirement commitment. No precommit means no normal retirement path.
- The macOS host DAG must enumerate every test-created bootstrap intent, executor, plist, signing key, protected current-anchor wrapper, Mach service, service state, and the preallocated host guest-pairing record. Apple platform parents must be pre-existing dependencies and not deletion targets.
- The guest reservation must preallocate the challenge ID, host-pairing component ID, and every exact guest component UUID/role/target and before-state before host bootstrap.
- Guest retirement is separately acyclic: canonicalize the guest ticket with its optional commitment forced null; the harness signs a full GuestPublisherTestRetirementAuthorizationV1 over that core and exact scope; the final host-signed ticket may differ only by the commitment.
- The protected reservation state machine is Reserved -> TicketIssued -> GuestConsumed -> Retired, with signed protected CAS transitions. GuestConsumed is the one-use grant before the first guest mutation. An unused branch requires externally acknowledged protected proof before ProvenUnused -> Retired.
- Guest teardown must precede host teardown. Each publisher signs a pre-teardown receipt; the harness fsyncs the canonical receipt and parent, hashes it, signs/fsyncs a separate acknowledgement; the publisher verifies and durably records the acknowledgement hash before any destructive step.
- Teardown touches only the exact precommitted DAG, in dependency-reverse order. Pre-existing dependencies are restored/preserved, not deleted. Host pairing state remains until guest receipt, acknowledgement, removal observations, and baseline parity are verified.
- The current-anchor/protected wrapper and bootstrap intent cannot be erased before all evidence they authenticate is external and acknowledged.
- Kill points bracket receipt/signature/fsync/hash/acknowledgement, every protected CAS, every component removal, and final parity. Crash retry must exact-join external receipts plus component observations; ambiguity preserves state.
- The evidence result is clean only after exact baseline parity. Native macOS evidence and MAC-CLOSEOUT must not be dispatched by this work.

4. Exact failed-attempt state and recovery problem
- The failed attempt was created before any evidence-retirement commitment was populated; its commitment is therefore product-null/precommit-less.
- Its launchd publisher registration is absent.
- Fourteen exact lifecycle generic-password records were deleted during attempted cleanup.
- The scoped System-Keychain P-256 signer remains.
- The selected evidence prefix remains, including copies of substrate-lifecycle-control and substrate-lifecycle-macos from the older landed checkpoint. Those binaries contain no executable test-retirement route.
- The Lima instance named substrate remains running.
- The evidence source mirror intentionally remains at the earlier commit 729e444c7dc8c136269409952ff2325bd810bd0f; it has not been refreshed because the machine is not at baseline.
- Both a legacy cleanup query and the product-equivalent exact kSecMatchItemList query unexpectedly launched SecurityAgent despite kSecUseAuthenticationUIFail. They were terminated without approval. No prompt was approved and no blind retry is acceptable.
- The generic records that would have authenticated current scope/admission/anchor state are already gone, so the landed retire_mac_system_keychain_software_signer_v1 cannot satisfy its normal protected-state join through a newly wired prospective route.
- Inference: a prospective precommit protocol cannot honestly retroactively authorize this old attempt. Recovery needs a distinct authority model, an explicit human-approved Keychain operation, or a proof that the old creator-bound executable identity can still perform the exact deletion without broadening authority. Do not assume which is possible without inspecting Apple Security behavior and the retained binary/code identity.

5. Rejected tentative implementation and why it was stopped
A tentative uncommitted implementation tried to add a harness and host/guest receipt routes. It was discarded because source audit found all of the following:
- It generated synthetic component UUIDs rather than binding the real exhaustive host/guest component DAG, roles, targets, and before-state.
- It did not implement protected reservation transitions or durably bind acknowledgement hashes into protected CAS state.
- It used broad directory removal rather than exact reverse-order component retirement.
- Its host flow called ordinary service retirement first, deleting the privileged helper before deleting Keychain state and the signer. A crash after helper deletion would remove the only authorized retry executor and recreate the orphan-signer problem.
- It lacked the contract-required kill-point matrix and complete crash/retry proof.
This was treated as a genuine architecture/security boundary, not a request to weaken validation.

6. Candidate scoping decision now under review
Candidate A: one top-level task plans and implements both the complete prospective evidence-only retirement protocol and bounded recovery of this old precommit-less orphan.
Candidate B: first land the full prospective protocol, then separately authorize and implement old-attempt recovery.
Candidate C: first research/resolve old-attempt recovery and restore baseline, then land the prospective protocol.
Candidate D: amend/split the contract because the exact helper/executor deletion order and final-signer retirement cannot be made crash-recoverable under the current authority model.
Broad product uninstall, publisher upgrade/replacement across code identities, general orphan recovery, multi-install ownership, and last-claim retirement should remain deferred unless one is strictly required to solve this exact evidence blocker.

7. Questions that need direct answers
- Is the authoritative prospective protocol internally coherent on macOS, especially the ordering among protected wrapper, signer, helper executable, launchd registration, external acknowledgement, and final parity?
- What durable executor/authority must survive until the last signer/Keychain deletion, and who may delete that executor after it can no longer authenticate through the retired publisher state?
- Should the external harness be authorized to perform only the final filesystem removal after receiving a publisher-signed terminal receipt, while all Keychain/signer deletion remains creator-bound? If not, what safer handoff avoids a helper self-deletion crash gap?
- Can the precommit-less orphan be recovered without retroactively minting retirement authority? Identify acceptable trust roots and explicitly reject unsafe ones.
- Which candidate sequence A/B/C/D is best, and what is the minimal top-level planning/research boundary before implementation?
- Which existing contract fields/types are structurally insufficient and must change before implementation?
- What exact kill-point/retry state machine and negative tests are necessary to make the next slice landable?

Current approach is therefore to pause implementation, preserve the failed attempt, obtain this independent approach review, reconcile it with live repository truth, and only then decide the next dispatch. Repository-derived statements above are factual summaries; the observations marked inference are not treated as established authority.

Project sources: No ChatGPT Project source configuration exists for this repository, so no remote Project sources were inferred or modified. Treat this prompt as the complete bounded review packet. Do not infer access to excluded source. The named files and symbols are provided for traceability, but the review must operate only on the facts reproduced here.

Evidence, blockers, constraints, and validation: Do not propose approving unexpected Keychain UI, weakening kSecUseAuthenticationUIFail, broad security-tool deletion, caller-selected key tags/paths/actions, retroactive precommit, wildcard cleanup, or deletion based on path/liveness alone. Do not dispatch native evidence or MAC-CLOSEOUT. Preserve the existing failed attempt until an authorized exact recovery design exists. Any recommendation must distinguish prospective clean-attempt retirement from recovery of this already-precommit-less orphan. It must also distinguish the minimum evidence blocker from deferred general uninstall/upgrade/multi-install work. Validation must include canonical/golden-vector tests, exact component-DAG positive/negative cases, protected CAS transition tests, crash/retry kill points around every authority and destructive boundary, macOS compile/static fixtures, and one later manual native run performed by the operator only after landed remote equality, exact cleanup, and evidence-mirror refresh.

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Is the current approach the best way forward? Give a direct keep, adjust, or pivot recommendation; tradeoffs; what to keep/change first; inspections; validation; and pitfalls.

```

## ChatGPT Pro response

```text
Worked for 24m 34s
Recommendation: Adjust

Keep the implementation pause, preserve the failed attempt exactly as it is, and retain the core prospective model: precommit before bootstrap, exact host/guest DAGs, guest-before-host teardown, externally fsynced receipts and acknowledgements, protected CAS, and fail-closed ambiguity handling.

Change the task boundary before any implementation:

Choose Candidate D as the gating decision, then execute in Candidate C order.

In practical terms:

Amend the prospective contract to define a crash-recoverable terminal authority handoff.

Separately research and authorize recovery of the existing precommit-less orphan.

Recover that attempt and establish exact baseline parity.

Only then implement and validate the prospective evidence-retirement protocol.

The two efforts may share one planning umbrella, but they must not share an implementation route, authorization type, or claim of authority. The old attempt must never be converted into a normally precommitted retirement attempt.

1. Direct assessment of the current approach

The current approach is directionally correct but not yet sufficient for dispatch.

Keep

The clean landed checkpoint at 879a8c680ba14cd93b79ba2c1a2ed598e9687c90.

The discarded tentative implementation staying discarded.

The failed machine state remaining frozen.

Product bootstrap keeping test_retirement_commitment = null.

Evidence retirement requiring a non-null commitment before bootstrap.

No retroactive commitment or reconstructed protected state.

Guest teardown before host teardown.

External receipt durability and acknowledgement before destruction.

Exact component identities, observations, dispositions, and reverse dependency order.

kSecUseAuthenticationUIFail, with no blind retry when SecurityAgent appears.

Deferral of general uninstall, upgrade, code-identity replacement, multi-install ownership, and generalized orphan recovery.

Change first

Before coding, split the work into two authority domains:

Domain	Authority source	Permitted use
Prospective evidence retirement	Harness precommit carried immutably by bootstrap and anchor	Future disposable attempts only
Existing orphan recovery	Original Keychain ACL-recognized creator identity or an explicit, one-time operator recovery authorization	This exact already-failed attempt only

The prospective route must reject the old attempt because its commitment is null. The recovery route must be incapable of creating or populating a retirement commitment.

2. Is the prospective protocol internally coherent on macOS?

Its principal safety invariants are coherent, but the protocol is not implementation-complete at its terminal boundary.

The problem is not simply the component removal order. There are actually two different graphs:

Resource dependency graph — determines reverse-order component removal.

Retirement authority graph — determines which executor and trust material must remain available to perform and recover later removals.

The current contract describes the first much more completely than the second.

A resource DAG alone can incorrectly make the helper appear removable before the signer. Operationally, however, the helper is an authority dependency of the signer deletion. That authority dependency must be explicit.

Required terminal partial order

The safe macOS order is:

Complete guest retirement, receipt acknowledgement, removal observations, and guest baseline parity.

Only then remove the host guest-pairing record.

Produce the host pre-teardown receipt.

Have the harness atomically persist and fsync that receipt and its parent.

Have the harness separately sign, persist, and fsync the acknowledgement.

Record the receipt and acknowledgement hashes in protected CAS.

Remove ordinary nonterminal host components in exact reverse-DAG order.

While the protected wrapper, signer, helper executable, plist, and launchd registration still exist, produce and acknowledge a terminal finalizer grant.

Persist a fixed, fail-closed terminal latch that prevents the service from accepting ordinary bootstrap or publisher operations.

Delete all remaining creator-bound lifecycle Keychain records.

Delete the protected wrapper last among those generic-password records.

Delete the P-256 signer last among all Keychain operations, using the exact creator-authorized process and exact SecKey identity.

Verify signer absence while the helper process and service installation still exist.

A separate exact finalizer performs launchd bootout and deletes the helper, plist, provenance files, and terminal latch.

The external harness establishes and signs exact final baseline parity.

The existing retire_mac_system_keychain_software_signer_v1 rule that refuses deletion while the protected wrapper exists is compatible with this ordering—but only after the terminal handoff has been durably established. Without that handoff, the wrapper-to-signer interval recreates the current orphan condition.

Why the terminal grant is necessary

The publisher cannot sign a truthful receipt saying “the signer is absent” after it has deleted its own signer.

Therefore, the publisher-signed object before signer deletion must not be modeled as a final receipt. It must be a conditional handoff authorization:

Given this exact precommitted attempt, these exact external receipt and acknowledgement hashes, this exact terminal CAS version, this exact remaining component set, and this exact finalizer code identity, the finalizer may act only after independently observing that the lifecycle records and signer are absent.

The later signer-absence observation belongs in the finalizer/harness closeout record, not in a pre-signed publisher receipt.

3. What executor and authority must survive?

Two distinct executors are needed.

Creator-bound Keychain executor

The publisher helper—or another executable proven to satisfy the same actual Security.framework ACL—is the only executor that should perform:

Exact lifecycle generic-password deletion.

Protected-wrapper deletion.

Exact P-256 signer deletion.

Final absence verification of those Keychain objects.

The following must remain until signer absence has been verified:

The creator-authorized executable.

Its launchd registration or another already-authorized invocation mechanism.

Its plist and provenance.

The exact terminal handoff record.

A fixed terminal latch preventing fresh bootstrap behavior.

The external receipt directory and acknowledged terminal grant.

“Same Team ID,” “same filename,” or “same path” is not enough. The relevant criterion is whichever designated requirement, ACL, access-control object, or creator identity the Keychain actually enforces.

Post-Keychain finalizer

After the signer is absent, a separate finalizer may perform only:

Exact launchd bootout.

Verification that the expected service is absent.

Exact helper, plist, provenance, journal, and latch removal.

Parent-directory fsyncs.

Final component and baseline observations.

It must have no Keychain deletion operation and no caller-selected paths, labels, tags, service identifiers, or actions.

The best form is a closed one-shot route in a pre-existing canonical privileged executor outside the publisher retirement DAG. Its code identity and action schema must be included in the original retirement authorization. It should consume the signed terminal grant and exact observations, not arbitrary command arguments.

If no such pre-existing executor exists, the contract must define ownership and cleanup of a separate finalizer. Do not introduce another root-owned self-deleting executable; that merely moves the same crash gap one component later.

Role of the external harness

The harness should be the authority and evidence custodian, not an unrestricted root shell.

It can authorize the final filesystem stage, but privileged effects should pass through the narrow finalizer. The harness supplies:

The original precommit.

Durable external receipt storage.

Receipt acknowledgements.

The terminal-grant countersignature.

The final parity record.

This is safer than giving the harness a general privileged deletion facility.

4. Required restart behavior after the wrapper is gone

A terminal latch is important because a crash after signer deletion may leave the launchd service installed.

Without a latch, absent lifecycle state and an absent signer might look like a fresh bootstrap condition. A restarted helper must not recreate either.

The latch should be established and fsynced before wrapper deletion and should have these semantics:

Its presence disables all ordinary product and bootstrap operations.

With the signer still present, the helper may verify the terminal grant against that exact signer and resume only creator-bound Keychain finalization.

With the signer absent, the helper performs no mutation and accepts no bootstrap request; it exits or reports terminalizing status.

Only the external finalizer removes the latch after bootout and exact file cleanup.

The latch need not grant authority. Its safest property is fail-closed inhibition. Actual authority remains in the precommitted and signed terminal grant.

5. Can the precommit-less orphan be safely recovered?

Yes, potentially—but not through the normal retirement protocol and not through unattended authority inferred from the surviving signer.

There are only two acceptable authority roots for the signer deletion.

Acceptable authority root 1: proven original creator-bound identity

This route is acceptable only if inspection demonstrates that an exact retained or reproducible executable identity is accepted by the System Keychain for the particular key and can perform the exact deletion without UI.

Required proof includes:

Exact retained Mach-O hash and code signature.

Designated requirement, Team ID, signing identifier, and relevant entitlements.

Exact relationship between that identity and the key’s actual access-control policy.

Exact System Keychain selection.

Exactly one matching key.

Exact fixed application tag and label derivation.

Bound public-key/SPKI fingerprint.

Exact SecKey reference used in the delete query.

Demonstrated no-UI behavior in an isolated reproduction before touching the live orphan.

The retained old binaries currently have no executable retirement route. The existence of a dormant library function elsewhere does not make those binaries capable of invoking it. Rebuilding a new executable with similar signing metadata also cannot be assumed to satisfy the original key’s access control.

Acceptable authority root 2: explicit one-time operator recovery authorization

If the original ACL-recognized route cannot be demonstrated, the operator may act as the new recovery authority for this exact orphan.

That does not retroactively authorize the original attempt. It authorizes an exceptional recovery operation after failure.

The authorization must bind:

This exact failed attempt and machine.

The exact surviving key fingerprint and fixed tag derivation.

The expected absence of the fourteen lifecycle records.

The expected absence of the launchd registration.

The exact retained evidence prefix identity.

The exact Lima instance identity, not merely the name substrate.

The exact residual component list.

One fixed action sequence.

A short admission window and one-use recovery counter.

No wildcard paths, generic tags, or caller-selected component list.

A deliberately planned, operator-observed Keychain approval could be part of this route, but only after the invoking executable and exact key have been verified and the prompt is expected. The two already-observed unexpected SecurityAgent launches must not be approved or blindly repeated.

If an interactive operation is chosen, cancellation, process termination, or connection loss must always be followed by a fresh exact presence query before any further action.

Unsafe trust roots that must be rejected

None of the following independently establishes recovery authority:

Root or sudo possession.

The scope_id string.

Key tag or label alone.

A path to the old executable.

The Lima instance name.

Launchd registration absence.

The fact that generic records are missing.

A new harness Ed25519 key.

A newly synthesized retirement commitment.

Recreated generic-password state.

The surviving P-256 signer signing its own deletion authorization.

Source-commit ancestry.

File liveness or directory ownership alone.

A broad security command or wildcard Keychain query.

Reinstalling the old launchd service merely to regain access.

The surviving signer may help establish object identity. It does not create retirement authority.

6. Candidate decision
Candidate A — one implementation task

Reject.

It conflates mutually incompatible trust roots:

Prospective retirement derives authority from immutable precommit.

Old-attempt recovery necessarily derives authority from an exceptional operator decision or the original Keychain ACL.

Sharing an implementation route would create pressure to let the recovery route mint, bypass, or imitate normal retirement authority.

Candidate B — prospective protocol first

Reject as the immediate sequence.

It would prevent future orphans but leave the current machine nonbaseline, block meaningful native validation, and risk landing the terminal authority model without first resolving the actual SecurityAgent and creator-identity behavior exposed by the orphan.

Candidate C — recover first, then prospective protocol

Operationally correct, but incomplete by itself.

Recovery should occur before the later native prospective run. However, proceeding directly to recovery without first defining its distinct authority boundary risks creating an ad hoc generalized orphan-cleanup route.

Candidate D — contract amendment and split

Best choice.

Candidate D should be executed in this order:

Read-only research sufficient to settle the creator-bound Keychain and finalizer questions.

Contract amendment that explicitly separates normal retirement from exceptional recovery.

One exact orphan recovery and baseline restoration.

Prospective protocol implementation.

Later operator-run native evidence.

This preserves Candidate C’s useful operational ordering without bypassing the missing contract boundary.

7. Minimal top-level planning boundary

Create one non-implementation planning/research task with exactly two outputs.

Output A: prospective terminal-authority amendment

It must define:

Resource DAG versus authority/holdback graph.

Creator-bound Keychain executor identity.

External finalizer identity and closed capabilities.

Terminal survivor set.

Terminal latch.

Terminal grant and acknowledgement.

Admission-expiry versus retry semantics.

Authority after protected-wrapper deletion.

Authority after signer deletion.

Final harness-signed parity record.

Output B: exact orphan-recovery decision record

It must determine:

Whether the original creator-bound identity can perform the exact deletion without UI.

What caused the two SecurityAgent launches.

Whether an explicit human-approved operation is required.

The exact residual ownership inventory.

The bounded one-time recovery sequence.

Conditions under which recovery must stop and preserve the orphan.

After those outputs, create two separate implementation/operation tasks:

One-attempt recovery task, permanently bound to this failed attempt.

Prospective evidence-retirement implementation task.

No code path or wire schema should be accepted by both.

8. Types and fields that must change
Definitely insufficient from the bounded packet
GuestPublisherTestRetirementAuthorizationV1

It must bind at least:

Canonical guest ticket core with optional commitment forced null.

Final ticket rule: only the commitment may differ.

Host reservation ID.

Preallocated challenge ID.

Host-pairing component ID.

Exact guest machine and platform mapping.

Exact source, build, evidence, and dispatch scope.

Complete ordered guest DAG.

Component UUID, role, target, dependency parents, and disposition.

Exact before-state and expected after-state.

Guest executor ELF identity.

Host finalizer identity for terminal guest residue.

Counter limit, nonce, admission expiry, and one-use semantics.

External receipt directory and retained parent identity.

Receipt and acknowledgement protocol version.

Terminal handoff types

No listed type currently provides the missing final authority handoff. At minimum the contract needs equivalents of:

PublisherTestRetirementTerminalGrantV1

PublisherTestRetirementTerminalAcknowledgementV1

PublisherTestRetirementTerminalLatchV1

PublisherTestRetirementFinalizerReceiptV1

A guest equivalent is also required unless the existing reservation and host finalizer model fully and explicitly covers guest terminal residue.

Existing types that require field-level audit and likely expansion
PublisherTestRetirementAuthorizationV1

It must commit to:

Bootstrap core with the retirement slot forced null.

Complete host DAG and dispositions.

Baseline absence.

Exact platform/build/machine/evidence/dispatch scope.

Harness public key.

External receipt-parent identity.

Exactly one guest reservation.

Creator-bound Keychain executor identity.

External finalizer code identity.

Allowed terminal action schema.

Terminal latch location/identity.

Terminal survivor set.

Admission expiry and counter semantics.

PublisherTestRetirementCommitmentV1

A compact commitment may remain compact if it hashes one canonical complete authorization. It must not become a second divergent source of truth.

It must, directly or through the committed authorization digest, bind:

Host authorization digest.

DAG digest.

Harness key.

Receipt-parent identity.

Guest reservation.

Finalizer identity and capability digest.

Terminal-latch descriptor.

GuestPublisherRetirementReservationV1

It needs:

Challenge ID.

Host-pairing component ID.

Every guest UUID, role, target, parent, disposition, and before-state.

Exact ticket-core digest.

Guest-authorization digest or commitment slot.

Current reservation state and CAS version.

Counter, expiry-at-admission, and one-use grant identity.

Exact guest machine/platform mapping.

Receipt and acknowledgement types

Both host and guest receipt families need to bind:

Authorization and commitment digests.

Current protected CAS version.

Exact component observations.

Exact removal plan digest.

Receipt byte hash.

Retained receipt-parent identity.

Acknowledgement hash.

Expected next state.

Any remaining terminal component subset.

No uncommitted or caller-supplied path list.

Component descriptor

A simple dependency list is insufficient. Each descriptor needs enough information to distinguish:

Exact resource identity.

Before-state.

Expected after-state.

Delete, restore, or preserve disposition.

Resource dependency parents.

Authority holdback or required_until phase.

Creator/executor identity where relevant.

Host Mach-O identity versus guest ELF identity.

Fixed target and no-follow provenance rules.

9. Prospective durable state machine

A landable model would use states equivalent to the following.

State	Durable authority	Mutation permitted
HarnessPrepared	Harness key, signed authorization, fsynced external directory	None
HostCommitted	Final bootstrap/anchor commitment and protected CAS	Bootstrap-created components only
Reserved	Protected reservation CAS	No guest mutation
TicketIssued	Signed ticket and protected CAS	No guest mutation
GuestConsumed	One-use protected CAS grant	First and subsequent exact guest mutations
GuestReceiptAcknowledged	External receipt/ack plus protected hash	Guest teardown
GuestRetiring	Protected progress CAS and exact observations	One exact guest component at a time
GuestRetired	Guest parity and protected state	Host pairing record may now be removed
HostReceiptAcknowledged	External host receipt/ack plus protected hash	Host teardown
HostRetiring	Protected progress CAS	Nonterminal host components
TerminalGrantAcknowledged	Protected CAS, external grant/ack, terminal latch	Creator-bound Keychain finalization
KeychainTerminalizing	Terminal grant, latch, exact signer still present	Generic records, wrapper, signer
SignerAbsent	External grant plus exact absence observations	Finalizer-only service/file operations
PublisherServiceAbsent	Finalizer receipt and observations	Final residual cleanup
BaselineParity	Harness-signed, fsynced terminal record	No further retirement mutation

The unused guest branch must be separate:

Reserved or TicketIssued → externally acknowledged ProvenUnused → Retired

It must be impossible to reach ProvenUnused after GuestConsumed.

Expiry rule

Expiry should control admission, not completion.

Once a protected CAS has admitted a retirement or terminal handoff, a crash occurring after the original wall-clock expiry must not brick the attempt. Continuation is then controlled by exact attempt identity, nonce, monotonic counter, signed state, and component observations.

10. Kill-point and retry requirements
External receipt and acknowledgement

Place kill points around:

Temporary receipt creation.

Canonical byte write.

File fsync.

Atomic rename.

Receipt-directory fsync.

Parent-directory fsync.

Receipt hash calculation.

Acknowledgement signing.

Acknowledgement write and fsync.

Protected CAS recording of both hashes.

No destructive action may begin until all required durability and CAS steps are complete.

Retry must compare exact canonical bytes, hashes, signatures, and retained directory identity. A conflicting existing file is ambiguity, not permission to overwrite.

Every protected CAS transition

Use an intent/observation pattern:

CAS records exact action intent, expected before-state digest, and component ID.

Perform one effect.

Observe exact resulting state.

CAS records completion and observation digest.

On retry:

Expected before-state: perform the action.

Expected after-state: commit the observed completion.

Anything else: preserve state and fail closed.

Every component removal

Kill points are needed:

Before descriptor/no-follow adoption.

After descriptor adoption but before mutation.

Immediately after the effect.

After effect observation.

Before and after parent-directory fsync.

Before progress-CAS completion.

An absent path alone is not sufficient. Absence must join the exact component identity, expected before-state, action intent, and parent observation.

Protected-wrapper deletion

Preconditions:

Terminal grant and acknowledgement are externally durable.

Their hashes are in protected CAS.

Terminal latch is durable.

Exact creator-bound helper and signer are present.

All earlier Keychain records scheduled before the wrapper are absent.

Retry classification:

Wrapper present, signer present: retry exact wrapper deletion.

Wrapper absent, signer present: resume signer deletion from the terminal grant.

Wrapper absent, signer absent: creator-bound stage is complete; finalizer takes over.

Any unexpected key multiplicity or identity mismatch: halt.

Signer deletion

Kill points:

Before exact key query.

After obtaining exactly one SecKey reference.

Before delete call.

After delete call.

After exact absence verification.

Before terminal progress observation is made durable.

Retry classification:

Exactly the bound signer remains: exact retry may be allowed.

No bound signer remains: deletion is committed.

More than one match, different fingerprint, authentication UI request, unclassified OSStatus, or inability to constrain the search to the System Keychain: halt.

No fallback query is permitted.

Launchd bootout and final filesystem removal

The external finalizer records exact intent before each operation.

For bootout:

Exact service still present with expected executable/provenance: retry.

Exact service absent: commit effect after verifying no mismatched registration exists.

Service label present with different provenance: ambiguity; halt.

For files:

Exact adopted inode/digest present: unlink and fsync parent.

Absent after recorded intent: commit only after exact parent and component observations.

Different object at the target: preserve and halt.

Final parity

Kill points bracket:

Every absence/preservation observation.

Canonical final record write.

File fsync.

Rename.

Directory and parent fsync.

Harness signature.

The result is not clean until the fully durable harness-signed parity record exists.

11. Orphan-recovery state machine

The recovery should use a much smaller, one-attempt state machine.

State	Requirement
Frozen	No mutation; preserve signer, prefix, Lima instance, binaries, logs, and source mirror
Identified	Exact key, code identity, attempt, prefix, Lima identity, and missing-state inventory captured
RecoveryAuthorized	Original creator ACL proven or one-time operator authorization established
SignerAbsent	Exact operation performed and exact System-Keychain absence verified
ResidualCleanup	Only exact attempt-owned prefix/Lima/components removed with external progress journal
BaselineParity	Exact baseline established; only then refresh evidence mirror

Recovery must halt if exact ownership of the Lima instance or prefix contents cannot be established. The name substrate and the selected path are not sufficient by themselves.

The fourteen already-deleted generic-password records should be treated as observed irreversible absence. They must not be recreated merely to satisfy the normal signer-retirement helper.

12. Required inspections before implementation or recovery
Keychain and SecurityAgent inspection

Determine:

The precise query that caused each SecurityAgent launch.

Whether the UI was requested by the item ACL, System Keychain access, key use, delete operation, or another query attribute.

The exact returned or interrupted OSStatus.

Whether kSecUseAuthenticationUIFail was attached to the effective query.

Whether the query was constrained to the explicit System Keychain and exact key reference.

The key’s public-key fingerprint, application tag, label, class, token/software status, and accessible access-control metadata.

Which executable identity originally created or successfully accessed it.

Whether a same-designated-requirement rebuild is accepted or whether the ACL is narrower.

This must first be reproduced with a disposable surrogate key or disposable host state, not the live orphan.

Retained executable inspection

For the old retained binaries, record:

SHA-256.

Code directory hash.

Designated requirement.

Signing identifier.

Team ID.

Entitlements.

Architecture.

Provenance to the earlier landed checkpoint.

Whether any existing closed route can reach the exact signer-delete function.

Do not infer reachability from the function existing elsewhere in the source.

Launchd inspection

Establish:

Exact service label and domain.

Whether the job is on-demand or automatically restarted.

Mach service behavior after the backing signer and state disappear.

Existing service-state journal semantics.

Whether a terminal latch can prevent fresh bootstrap after restart.

Whether the external finalizer can idempotently boot out the service under effect-visible ambiguity.

Component inventory inspection

Build the actual component inventory from authoritative constructors and signed artifacts, not from the tentative implementation:

All host component UUIDs, roles, targets, parents, before-states, and dispositions.

All guest component UUIDs and reservations.

Every one of the fourteen lifecycle accounts and its deletion dependencies.

Signer identity and fixed tag derivation.

Bootstrap intent, current-anchor wrapper, service state, plist, helper, Mach service, provenance, pairing record, executor, and evidence-prefix artifacts.

Lima instance identity and guest state.

Pre-existing dependencies that must be preserved.

13. Validation required for a landable prospective slice
Canonical and signature validation

Golden canonical bytes for every host and guest authorization, commitment, reservation, receipt, acknowledgement, terminal grant, and parity record.

Cross-language signature vectors for harness Ed25519 and publisher P-256.

Null-slot normalization vectors.

Unknown-field rejection.

Field mutation, list reordering, duplicate entry, and alternate canonical representation rejection.

Final guest ticket differs from its null-core only by the commitment.

Component-DAG validation

Positive cases must include a complete real host DAG and complete real guest DAG.

Negative cases must cover:

Missing signer, helper, plist, service state, wrapper, intent, Mach service, executor, or pairing component.

Extra or synthetic UUID.

Duplicate component.

Cycle.

Missing parent.

Wrong parent.

Delete disposition for a pre-existing dependency.

Incorrect before-state.

Role/target mismatch.

Symlink or no-follow violation.

Host Mach-O/guest ELF identity substitution.

Terminal survivor omitted or scheduled too early.

Finalizer identity not precommitted.

Reservation and CAS validation

Only Reserved -> TicketIssued -> GuestConsumed -> Retired.

GuestConsumed occurs before first guest mutation.

Duplicate consumption rejected.

Wrong challenge, host pairing ID, or guest component set rejected.

Unused proof cannot follow consumption.

Receipt acknowledgement hash must be in protected CAS before teardown.

Host teardown rejected until guest retirement and parity.

Host pairing deletion rejected before guest retirement.

Skipped, stale, duplicate, conflicting, and rollback CAS transitions rejected.

Terminal-boundary negatives

Wrapper deletion before terminal grant acknowledgement.

Signer deletion while wrapper remains.

Service bootout before signer absence.

Helper deletion before Keychain finalization.

Finalizer attempting any Keychain action.

Finalizer accepting caller-selected paths, service labels, or key tags.

Finalizer grant for an uncommitted attempt.

Restart with terminal latch attempting fresh bootstrap.

Signer absent without a valid terminal grant.

Receipt claiming future signer absence.

Terminal grant or acknowledgement stored under the wrong external-directory identity.

Crash/retry matrix

Inject a kill before and after:

Every canonical write.

Every fsync and rename.

Every hash and acknowledgement.

Every CAS transition.

Every component mutation.

Wrapper deletion.

Signer delete call.

Signer absence verification.

Launchd bootout.

Each file unlink.

Each parent fsync.

Final parity write and signature.

Every test must demonstrate one of three explicit classifications: before-effect, after-effect, or ambiguous. Ambiguous state must never advance.

macOS-specific validation

Before a manual run:

macOS compile coverage for all closed routes.

Static fixture validation for Security.framework query composition.

Static fixture validation for exact System Keychain search-list selection.

Static fixture validation for kSecUseAuthenticationUIFail.

Exact SecKey reference binding.

Closed launchd service targets.

No caller-selected tags, labels, paths, or actions.

No executable test-retirement route when commitment is null.

Only after the landed branch and live remote are equal, the one-off recovery has produced exact baseline parity, and the evidence mirror has been refreshed should the operator perform one later manual native attempt. MAC-CLOSEOUT remains outside this work.

14. Principal pitfalls

Treating the ordinary service-state retire route as the first host teardown step.
This deletes the only creator-bound retry executor too early.

Calling a pre-signer-deletion document a terminal receipt.
It cannot attest a future deletion. It is a conditional finalizer grant.

Using one DAG for both resources and retirement authority.
The helper’s authority holdback is not necessarily represented by normal resource dependencies.

Letting expiry terminate crash recovery.
Expiry should gate admission; an admitted retirement must remain retryable.

Allowing an absent signer and absent state to look like fresh bootstrap.
The terminal latch must force closed behavior.

Assuming similar code signing equals Keychain authorization.
The actual ACL behavior must be inspected and reproduced.

Using path, label, instance name, or root privilege as ownership proof.
These are observations or execution privileges, not authority.

Generalizing the orphan route.
The recovery must remain bound to this attempt and must not become product uninstall or reusable orphan cleanup.

Refreshing the source mirror before parity.
That would destroy part of the retained evidence and blur the recovery boundary.

Deleting the finalizer through its own authority chain.
Use a pre-existing external executor or separately owned cleanup mechanism; do not create a second self-deletion gap.

Bottom line

The present pause-and-preserve posture is correct. The next action should not be implementation.

The authoritative prospective model is salvageable without weakening it, but it needs a contract amendment that explicitly introduces a terminal survivor set, terminal latch, publisher-signed conditional finalizer grant, and a separate closed filesystem finalizer. The creator-bound publisher must remain available through exact Keychain and signer deletion; only after signer absence may the finalizer boot out and remove the publisher service.

The existing precommit-less orphan must be recovered under a separate one-time authority based either on demonstrated original Keychain creator identity or deliberate operator approval of the exact key. It must never be retroactively admitted to the prospective retirement protocol.

Therefore: Adjust; select D as the contract decision, execute recovery before prospective implementation as in C, and keep the two implementation authorities completely separate.
```
