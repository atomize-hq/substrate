# Debug Regression Ledger

## How to read this ledger

- **Resolved baseline** means a specific observed failure has a trustworthy fix or live proof that must not regress. It does **not** promote the surrounding architecture seam.
- **Partially resolved** means a narrow behavior works while the target ownership model remains wrong or unproven.
- **Unresolved** means the target behavior lacks an implementation and proof gate.
- Historical diagnoses are retained only when they define a permanent negative or regression test.

Primary source memos:

- [`../../RUN_WORLD_TASK_DEBUG_CANONICAL.md`](../../RUN_WORLD_TASK_DEBUG_CANONICAL.md)
- [`../../CONTINUE_WORLD_WORKER_BLOCKING_DEVIATION_DEBUG.md`](../../CONTINUE_WORLD_WORKER_BLOCKING_DEVIATION_DEBUG.md)
- [`../../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md`](../../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md)

## Canonical issue ledger

Canonical content: [`evidence/canonical-issue-ledger.md#canonical-issue-ledger`](evidence/canonical-issue-ledger.md#canonical-issue-ledger).

### A1.2a-WB gate assignment

Canonical content: [`evidence/canonical-issue-ledger.md#a12a-wb-gate-assignment`](evidence/canonical-issue-ledger.md#a12a-wb-gate-assignment).

## A0 closeout evidence

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a0-closeout-evidence`](a1-2-earlier-histories/evidence-regression.md#a0-closeout-evidence).

## A1.1d mandatory-review state

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a11d-mandatory-review-state`](a1-2-earlier-histories/evidence-regression.md#a11d-mandatory-review-state).

## A1.1d-5I installer/bootstrap audit record

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a11d-5i-installerbootstrap-audit-record`](a1-2-earlier-histories/evidence-regression.md#a11d-5i-installerbootstrap-audit-record).

## A1.1e explicit-home policy proof requirement

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a11e-explicit-home-policy-proof-requirement`](a1-2-earlier-histories/evidence-regression.md#a11e-explicit-home-policy-proof-requirement).

## A1.2a, A1.2a-WB, and A1.2a-S recorded result

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a12a-a12a-wb-and-a12a-s-recorded-result`](a1-2-earlier-histories/evidence-regression.md#a12a-a12a-wb-and-a12a-s-recorded-result).

## B0 closeout evidence

Canonical content: [`b1-b2-1/evidence-regression.md#b0-closeout-evidence`](b1-b2-1/evidence-regression.md#b0-closeout-evidence).

## B1 production-path sequencing blocker and disposition

Canonical content: [`b1-b2-1/evidence-regression.md#b1-production-path-sequencing-blocker-and-disposition`](b1-b2-1/evidence-regression.md#b1-production-path-sequencing-blocker-and-disposition).

## B1/B2.1 `RegressionMasked` stop and ownership disposition

Canonical content: [`b1-b2-1/evidence-regression.md#b1b21-regressionmasked-stop-and-ownership-disposition`](b1-b2-1/evidence-regression.md#b1b21-regressionmasked-stop-and-ownership-disposition).

## B1/B2.1-R0 recorded result

Canonical content: [`b1-b2-1/evidence-regression.md#b1b21-r0-recorded-result`](b1-b2-1/evidence-regression.md#b1b21-r0-recorded-result).

## B3.2a and B3.2a-WA recorded result

Canonical content: [`b1-b2-1/evidence-regression.md#b32a-and-b32a-wa-recorded-result`](b1-b2-1/evidence-regression.md#b32a-and-b32a-wa-recorded-result).

## B1/B2.1 core recovery and B1/B2.1-0 recorded result

Canonical content: [`b1-b2-1/evidence-regression.md#b1b21-core-recovery-and-b1b21-0-recorded-result`](b1-b2-1/evidence-regression.md#b1b21-core-recovery-and-b1b21-0-recorded-result).

## B3.1 recorded result

Canonical content: [`b3-1-c1/evidence-regression.md#b31-recorded-result`](b3-1-c1/evidence-regression.md#b31-recorded-result).

## C1 recorded result

Canonical content: [`b3-1-c1/evidence-regression.md#c1-recorded-result`](b3-1-c1/evidence-regression.md#c1-recorded-result).

## A1.2b recorded result

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a12b-recorded-result`](a1-2-earlier-histories/evidence-regression.md#a12b-recorded-result).

## Baseline behaviors that all tracks preserve

These rows define permanent behavior contracts, not current pass claims. Any row explicitly marked
open remains a blocking regression gate until its named owner and real-path proof close it.

| Gate ID | Baseline | Required proof |
|---|---|---|
| **RG-BASE-01** | Public world-scoped start → turn/reattach → stop; **open, blocking, and not waived** | A1.2 must supply exact parked-successor `Attach`/`ResumeOneTurn` application. The held A1.3-P0 and A1.3 records remain preserved but cannot be dispatched. A1.3-P1 must atomically land the new authority-managed public Start/turn/reattach transport path, exact startup/post-turn actor-event resolution on the real public CLI/helper/REPL path, and only the strictly mechanical projection of already HSA-authorized correlation through the exact retained-turn `orchestrator_world_dispatch.rs` acceptance-context construction/projection seam and into the existing B1 acceptance-context field, without legacy authority writes, readiness/process inference, auto-attach drift, B1 semantic ownership changes, or C1/B3.1 semantic changes. Exact session and world binding survive; stop reaches durable terminal truth even when transport posture changes. Stale lifecycle/world-binding overwrite remains rejected. Prove unsafe other-principal authority fails closed separately, then run the positive smoke against an owner-only private bootstrap home with exact type/`0700`, descriptor identity/replacement safety, qualified access/default `NoData` under authoritative mode bits, and no effective other-principal authority; `NoData` is not physical-absence proof. This gate is neither permanently expected to fail nor successful until that real-path wall is green. |
| **RG-BASE-02** | REPL first-dispatch `run_world_task` binding repair | Correct binding succeeds; stale/mismatched world generation fails closed; no generic binding synthesis on unrelated surfaces. A host runtime may carry exact parent-session world binding while its descriptor and participant manifest remain host-scoped. |
| **RG-BASE-03** | Parked host ordinary-command and continuity parity | Unprefixed `ls`/`pwd` remain usable; policy-required `cd ../` cage denial remains; later targeted host turn reuses session/UAA continuity; public CLI parity stays green. |
| **RG-BASE-04** | Retained spawn/fork/exact continue/exact stop plus ambiguity close | Exact source and child handles route correctly; backend-only follow-up with multiple retained workers fails closed; source detached stop and child live-transport stop remain valid. |
| **RG-DIFF-01** | Monotonic broad-suite differential; a historical failure becoming a pass is neither automatic success nor automatic regression | Apply the exact-name and normalized-signature transition gate in `04`. Preserve complete inventories and artifact hashes; prove every historical failure-to-pass transition through the exact baseline and current full production dispatcher; inspect test and assertion diffs; map the causal change to the owning slice and symbol; and prove no direct resolver, transport, receipt, supervisor, helper, bypass, weakened enforcement, renamed/replaced/ignored test, weakened assertion, removed behavior, or unrelated capability loss. The exact fourteen-test B1/B2.1 inventory above retains every historical name and original production-level assertion. A lower-level substitute is `RegressionMasked`; any otherwise uncertain transition is `BaselineRegressionAmbiguous`. Either classification keeps the owning closeout open, and neither can count as `FailToPass` proof. Independent review is mandatory. |

## Cross-gate smoke scenarios

### S1 — Authority survives episode loss

1. Start a world-scoped host session and retain a worker.
2. Kill the current helper/episode and remove its private socket.
3. Read durable authority, worker manifest, and any active receipt.
4. Reattach or perform sanctioned durable closeout.
5. Prove no stale episode write regressed the authority revision.

Covers: `RG-AUTH-01`, `RG-AUTH-02`, `RG-CLOSE-01`, `RG-BASE-01`.

### S1A — Revision-bound host transitions survive helper loss and replay

1. Before any production Start, classify genuinely absent, interrupted-initialization, valid-existing, unsupported-pre-A1, and corrupt/unsupported stores. Reject malformed, partial, unreadable, symlinked, permission-invalid, and unsupported roots without generating a replacement store ID. Persist the same normalized physical bootstrap home in the init marker, greenfield certificate, and root; pass its opened handle through the bounded StateStore/config/policy/inventory entry points; change/remove ambient home values between each call and prove no reread; reject a copied/rebound authority tree; and crash/restart after marker, key, root, and marker-cleanup steps without changing store/home identity.
2. Completely and safely enumerate both pre-A1 session/participant authority collections before certificate publication and every semantic transaction. Prove component-by-component no-follow traversal from the trusted home rejects ancestor symlinks, unsafe owner/mode/ACL, cross-device rebinding, and scan-to-publication identity replacement. Prove validated empty collections permit greenfield certification, while any artifact is `UnsupportedLegacyState` without parsing/conversion and unreadable, permission-invalid, symlinked, or partially enumerable collections fail closed. Every pre-A1 read-decide-write-remove-rename transaction retains the opened trusted physical root, descendant handles, exact identity, activation observation, and root lock through final file/directory `fsync`; it never rereads `SUBSTRATE_HOME`, uses ambient CWD, reconstructs an absolute descendant path, or releases the lock early. In a real subprocess, rename/replace/rebind the lexical root after lock acquisition and prove original/replacement identities explicitly: the replacement receives zero writes, temps, removals, or `fsync`-dependent publication, and a lock on the former root is never represented as protection for the replacement. Serialize every in-repository pre-A1 writer on the same root lock before activation; after init-marker or root publication it rejects before mutation. No missing row or compatibility projection satisfies absence.
3. Exercise every A1 object kind against its exact V1 schema, file bytes, typed path, and commitment variant/domain. Reject cross-kind/schema/domain substitution; require `run_present=0x01` and the exact non-empty parent-intent run ID for every sensitive domain, with golden negative fixtures for absent/empty/different runs. Crash before/after each temp-file fsync and rename: recognized operation-bound temps are removed and never promoted, while unsafe/unrecognized temps fail closed. For pending initialization, validate a reused key only against the init marker and re-fsync the file plus `keys/` before root publication. Crash between first-use kind/version directory creation and every directory/parent fsync; only safe empty closed directories survive without object authority. Crash after object rename but before root commit; restart must classify the safe unindexed file as a retained non-authoritative orphan, and only a retry with the complete ref, exact parent intent/run/store context, and verified bytes may adopt it. Exact retry must treat every reservation, tombstone, transition intent, issuer-request index, application-journal, and object-index entry as semantic authority occupancy and may join only complete matching committed state. Rotate and retire HMAC keys across crashes before/after key publication, root selection, retirement root commit, and key deletion; after reconciliation and immediately before root publication, validate the complete candidate against the exact locked root/revision, home/store identity, key registry/files, active/verification/retired constraints, reachable objects, semantic maps/indexes/journal, empty pre-A1 collections, and unchanged physical root. Old referenced keys remain verification-only, key loss or post-reconciliation invalidity fails before publication with zero semantic mutation, and crash/restart plus exact retry converge.
4. Issue a public A1.2 `Start` intent against greenfield-certificate-proven `ExpectedAbsent`, then terminate delivery once while `Issued`, once after plan load/removal but before claim, once while `Claimed`, once after initial authority birth, and once after input acceptance but before success is observed.
5. Restart and retry the exact intent; prove missing plan transport is reproduced from the retained payload, pre-application cases safely resume/reconcile, and committed/accepted cases join one immutable result without another namespace, authority revision, participant allocation, or input delivery. Prove payload cleanup occurs only after exact terminal handoff, with crashes/failures before and after the `ReleaseEligible` root commit, transport deletion, object-directory fsync, and `Released` root commit for both present and already-absent transport bytes.
6. Repeat public CLI and real REPL `Attach` and `ResumeOneTurn` against exact revisions, including bounded auto-attach plan production, and prove the same bootstrap-home/workspace/world/descriptor/attach/resume/policy commitments reach application.
7. Reject stale revision, substituted plan, cross-kind ref, payload/hash mismatch, wrong caller/source/target/lineage, wrong bootstrap home/workspace/world generation, missing/copied greenfield certificate, compatibility-derived absence, pre-A1 artifacts, missing key, expired intent, superseded claim, and conflicting replay before authority mutation.
8. Run public world-scoped start → reattach → stop and prove exact session/world binding plus durable terminal truth are unchanged.

The deterministic parked-successor portion of this scenario is:

1. Persist current `Active` / `ParkedResumable` durable authority.
2. Remove all episode-local PID, handle, helper-readiness, and prompt-stream state.
3. Invoke the public prompt-bearing turn or explicit reattach path.
4. Resolve the exact current authority revision/hash and authoritative lineage.
5. Issue and apply `ResumeOneTurn` for the public turn or `Attach` for explicit reattach.
6. Preserve the exact session identity and world binding.
7. Revision-authorize exactly one successor participant and retain immutable input handoff when
   present.
8. Launch the helper/REPL execution episode only after durable application.
9. Retry the exact request and join the committed intent/application without a second successor.
10. Stop the same session and reach durable terminal truth.

Covers: `RG-AUTH-01`, `RG-AUTH-02`, `RG-AUTH-03`, `RG-BASE-01`, `RG-BASE-02`.

### S2 — Receipt, supervisor, obligation, cancel

1. Accept one ephemeral task from its exact pre-terminal B0 acknowledgement, durably claim the
   stream in B2.1 before any subsequent frame, and prove the production path never attempts legacy
   active-task registration. Blocking inspect/wait and narrowly adapted cancel consume exact
   receipt/supervisor truth, and foreground drop does not erase the accepted or supervised work.
2. Continue an exact retained worker, persist its B1 accepted active-run identity, hand the same B0
   stream to B2.1 with no observation gap, and prove the foreground may remain a blocking waiter
   without retaining observation ownership.
3. Normalize one supported provider attention shape before `AgentEvent` construction, emit one
   B3.1 event with exact target/run/thread/class/attention/causation identity, and prove B2.1
   journals it durably and C1 materializes one obligation before the terminal event. Prove an
   ambiguous or malformed attention-driving provider shape fails before emission rather than
   becoming progress or no-attention, and prove host JSON-pointer parsing cannot repair it. For
   every post-ack retained `Event` frame, prove either the typed member reaches the journal/cut or
   the stream and Complete cut fail closed; no event is silently dropped or left untyped.
4. Drop callers and restart the supervisor during both accepted families, then replay duplicate
   frames/events and prove the claim, journal, receipt, obligation, ledger revision, watermark, and
   terminal state remain idempotent. Missing exact terminal truth remains interrupted/nonterminal.
5. Query C1 before and after the exact terminal cut; prove Pending first, then Complete for both
   empty/no-attention and non-empty/attention dispositions without consulting projections. The
   Complete snapshot's ordered event vector must join exhaustively to every retained B2.1 `Event`
   ref in scope, including typed non-attention events.
6. After B2.2/B3.2, receive the accepted receipt before terminal exit and cancel by
   `active_run_id` in the same host turn.

Covers: `RG-EVENT-01`, `RG-RECEIPT-02`, `RG-RECEIPT-03`, `RG-SUP-01`, `RG-SUP-02`,
`RG-MSG-01`, `RG-CANCEL-01`, `RG-OBL-01`, `RG-OBL-02`, `RG-OBS-01`.

### S3 — World-UAA mediation under narrowed policy

1. Spawn/continue world Codex with write narrowed to `src/parser.rs`.
2. Prove shell and provider-native edit can modify that file.
3. Prove shell, direct edit, and write-capable tool cannot modify `README.md` or escape root.
4. Repeat with broker unavailable and require fail-closed behavior.
5. Join each operation to the receipt's immutable policy hash.

Covers: `RG-UAA-01`, `RG-UAA-02`, `RG-UAA-03`, `RG-POLICY-01`, `RG-POLICY-02`.

### S4 — Host visibility and failure truth

1. Run equivalent writes under host-visible and full-isolation policies.
2. Verify exact visibility/reconciliation semantics.
3. Force a non-zero UAA turn after a permitted write.
4. Prove durable failed active-run state, retained authority behavior, diagnostics, and file visibility outcome independently.

Covers: `RG-SYNC-01`, `RG-WORKER-EXIT-01`, `RG-OBS-01`.

### S5 — Existing gateway carrier preservation and direct Codex adoption

1. Run the existing launcher/consumer regression baseline: validated `GatewayAuthBundleV1`, raw-secret env scrubbing, one-time gateway consumption, and fresh bundle delivery on restart.
2. Resolve a credential source on the host and launch the exact managed in-world gateway without copying the secret payload into a world-visible runtime home.
3. Start direct world Codex with a per-worker Substrate-owned projection that points provider traffic at that gateway; do not seed a host auth file in contract-correct mode.
4. Join the envelope, accepted policy snapshot, projection identity, exact gateway receiver, and non-secret handoff evidence for the same world generation.
5. Prove the UAA child and descendants cannot inherit the secret FD or raw secret and that no secret-bearing runtime-native file is created.
6. Exercise duplicate consume, wrong gateway/world generation, expiry, unavailable handoff, and unavailable gateway adoption; require fail-closed behavior.
7. Exercise the explicitly named compatibility copy mode separately and prove it is logged, has retirement metadata, and cannot satisfy contract-promotion evidence.
8. Inspect logs/traces/manifests for secret payloads, secret-bearing paths, and reusable secret-derived hashes; none may appear.

Covers: `RG-CONFIG-02`, `RG-CONFIG-03`, `RG-CONFIG-04`, `RG-UAA-02`, `RG-UAA-03`, `RG-OBS-01`.

## Closeout rule

An implementation PR may mark a ledger row resolved only when:

1. its owning crosswalk seam has the correct owner and call path for that behavior;
2. the named permanent gate passes on the real path;
3. adjacent resolved baselines remain green; and
4. the evidence distinguishes durable success from transport/process success.

### R2-2 historical failed integration closeout and remaining-seam correction

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#r2-2-historical-failed-integration-closeout-and-remaining-seam-correction`](a1.1d-5r2-2f/evidence-regression.md#r2-2-historical-failed-integration-closeout-and-remaining-seam-correction).

## A1.1d-5R2-2F0-HC empirical closure record

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0-hc-empirical-closure-record`](a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0-hc-empirical-closure-record).

### Environment-inventory correction evidence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#environment-inventory-correction-evidence`](a1.1d-5r2-2f/evidence-regression.md#environment-inventory-correction-evidence).

### Preflight and preservation

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#preflight-and-preservation`](a1.1d-5r2-2f/evidence-regression.md#preflight-and-preservation).

### Forced overlap matrix

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#forced-overlap-matrix`](a1.1d-5r2-2f/evidence-regression.md#forced-overlap-matrix).

### Bounded clean-code broad evidence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#bounded-clean-code-broad-evidence`](a1.1d-5r2-2f/evidence-regression.md#bounded-clean-code-broad-evidence).

## A1.1d-5R2-2F0 historical parallel artifact recovery and authority correction

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0-historical-parallel-artifact-recovery-and-authority-correction`](a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0-historical-parallel-artifact-recovery-and-authority-correction).

### Bounded recovery result

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#bounded-recovery-result`](a1.1d-5r2-2f/evidence-regression.md#bounded-recovery-result).

### Corrected semantic and concurrency authority

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#corrected-semantic-and-concurrency-authority`](a1.1d-5r2-2f/evidence-regression.md#corrected-semantic-and-concurrency-authority).

### Canonical candidate preservation, proof, review, and stop boundary

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#canonical-candidate-preservation-proof-review-and-stop-boundary`](a1.1d-5r2-2f/evidence-regression.md#canonical-candidate-preservation-proof-review-and-stop-boundary).

## A1.1d-5R2-2F readiness-boundary evidence ledger

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f-readiness-boundary-evidence-ledger`](a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f-readiness-boundary-evidence-ledger).

### Reviewer finding and correction disposition

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#reviewer-finding-and-correction-disposition`](a1.1d-5r2-2f/evidence-regression.md#reviewer-finding-and-correction-disposition).

## A1.1d-5R2-2F5-PD evidence and decision ledger

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f5-pd-evidence-and-decision-ledger`](a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f5-pd-evidence-and-decision-ledger).

### Verified starting state and retained evidence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#verified-starting-state-and-retained-evidence`](a1.1d-5r2-2f/evidence-regression.md#verified-starting-state-and-retained-evidence).

### Exact blocked-candidate preservation

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#exact-blocked-candidate-preservation`](a1.1d-5r2-2f/evidence-regression.md#exact-blocked-candidate-preservation).

### Control-pack contradiction and source decision

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#control-pack-contradiction-and-source-decision`](a1.1d-5r2-2f/evidence-regression.md#control-pack-contradiction-and-source-decision).

### Security decision

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#security-decision`](a1.1d-5r2-2f/evidence-regression.md#security-decision).

## A1.1d-5R2-2F final evidence and decision ledger

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f-final-evidence-and-decision-ledger`](a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f-final-evidence-and-decision-ledger).

### Verified source and preservation identities

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#verified-source-and-preservation-identities`](a1.1d-5r2-2f/evidence-regression.md#verified-source-and-preservation-identities).

### Blocked-donor hunk disposition

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#blocked-donor-hunk-disposition`](a1.1d-5r2-2f/evidence-regression.md#blocked-donor-hunk-disposition).

### Final runtime behavior and proof

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#final-runtime-behavior-and-proof`](a1.1d-5r2-2f/evidence-regression.md#final-runtime-behavior-and-proof).

### Final decision and next node

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#final-decision-and-next-node`](a1.1d-5r2-2f/evidence-regression.md#final-decision-and-next-node).
## Renewed R2-2 publication decision ledger

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#renewed-r2-2-publication-decision-ledger`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#renewed-r2-2-publication-decision-ledger).

## A1.1d-5R2-2-B1 broad-wall invocation evidence correction

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#a11d-5r2-2-b1-broad-wall-invocation-evidence-correction`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#a11d-5r2-2-b1-broad-wall-invocation-evidence-correction).

### Verified B0 evidence and causal reconstruction

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#verified-b0-evidence-and-causal-reconstruction`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#verified-b0-evidence-and-causal-reconstruction).

### Baseline and future classification authority

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#baseline-and-future-classification-authority`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#baseline-and-future-classification-authority).

## Closeout-remediation planning ledger (historical RP0 checkpoint)

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#closeout-remediation-planning-ledger-historical-rp0-checkpoint`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#closeout-remediation-planning-ledger-historical-rp0-checkpoint).

### Immutable planning checkpoint

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#immutable-planning-checkpoint`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#immutable-planning-checkpoint).

### Renewed-closeout blocker record

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#renewed-closeout-blocker-record`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#renewed-closeout-blocker-record).

### R1 source-closure evidence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#r1-source-closure-evidence`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#r1-source-closure-evidence).

### P1 failed-harness evidence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#p1-failed-harness-evidence`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#p1-failed-harness-evidence).

### P1 source-closure and decision record

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#p1-source-closure-and-decision-record`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#p1-source-closure-and-decision-record).

## RP3/RP4/RP5 closeout ledger

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#rp3rp4rp5-closeout-ledger`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#rp3rp4rp5-closeout-ledger).
## Review-process calibration

RP4 exposed a process failure without exposing a product or test regression: a blanket requirement
for four clean reviews allowed findings about agent-created cache-only orchestration to expand the
product-proof acceptance surface. Repeated fix/review attempts then improved bespoke evidence
tooling rather than the selected Substrate outcome. Human disposition correctly preserved the raw
`REQUEST_CHANGES` review while accepting the independently evaluable product proof.

The prospective correction is owned by the development-review contract in `04`:

- `P1`/`P2` block only on demonstrated impact to the selected contract, gate, scope, or completion
  claim;
- one discovery cycle, one consolidated remediation, one closure cycle, and at most two directly
  causal supplemental cycles bound automatic work;
- `CLEAN` is terminal, mechanical-only deltas do not create review cycles, and unrelated or expanded
  blockers stop for authority rather than widening scope; and
- valid non-blocking review/process debt is retained in `06`, separate from this product regression
  ledger.

The three RP4 persistence findings are registered as `RR-RF-0001` through `RR-RF-0003`. Their raw
review files, hashes, and original verdict remain unchanged. This calibration authorizes no
controller/supervisor remediation and changes no RP3/RP4/RP5 proof result.

## A1.1d-5R2-4 terminal evidence ledger

Canonical content: [`a1.1d-5r2-4/evidence-regression.md#a11d-5r2-4-terminal-evidence-ledger`](a1.1d-5r2-4/evidence-regression.md#a11d-5r2-4-terminal-evidence-ledger).

## A1.1d-5R3 planned proof and regression ledger (archived for active scheduling)

Canonical content: [`a1.1d-5r3/evidence-regression.md#a11d-5r3-planned-proof-and-regression-ledger-archived-for-active-scheduling`](a1.1d-5r3/evidence-regression.md#a11d-5r3-planned-proof-and-regression-ledger-archived-for-active-scheduling).

### Source-closure and risk ledger

Canonical content: [`a1.1d-5r3/evidence-regression.md#source-closure-and-risk-ledger`](a1.1d-5r3/evidence-regression.md#source-closure-and-risk-ledger).

### Candidate and manifest matrices

Canonical content: [`a1.1d-5r3/evidence-regression.md#candidate-and-manifest-matrices`](a1.1d-5r3/evidence-regression.md#candidate-and-manifest-matrices).

### Lifecycle convergence and preservation matrices

Canonical content: [`a1.1d-5r3/evidence-regression.md#lifecycle-convergence-and-preservation-matrices`](a1.1d-5r3/evidence-regression.md#lifecycle-convergence-and-preservation-matrices).

### Baseline and product behavior wall

Canonical content: [`a1.1d-5r3/evidence-regression.md#baseline-and-product-behavior-wall`](a1.1d-5r3/evidence-regression.md#baseline-and-product-behavior-wall).

### `R3-NATIVE-LINUX-01`

Canonical content: [`a1.1d-5r3/evidence-regression.md#r3-native-linux-01`](a1.1d-5r3/evidence-regression.md#r3-native-linux-01).

### Packet-local provider evidence gates

Canonical content: [`a1.1d-5r3/evidence-regression.md#packet-local-provider-evidence-gates`](a1.1d-5r3/evidence-regression.md#packet-local-provider-evidence-gates).

### `R3-NATIVE-MAC-01`

Canonical content: [`a1.1d-5r3/evidence-regression.md#r3-native-mac-01`](a1.1d-5r3/evidence-regression.md#r3-native-mac-01).

### `R3-NATIVE-WIN-01`

Canonical content: [`a1.1d-5r3/evidence-regression.md#r3-native-win-01`](a1.1d-5r3/evidence-regression.md#r3-native-win-01).

### Final decision rule

Canonical content: [`a1.1d-5r3/evidence-regression.md#final-decision-rule`](a1.1d-5r3/evidence-regression.md#final-decision-rule).
## R3 implementation status append

Canonical content: [`a1.1d-5r3/evidence-status.md#r3-implementation-status-append`](a1.1d-5r3/evidence-status.md#r3-implementation-status-append).

### `A1.1d-5R3-MANIFEST`

Canonical content: [`a1.1d-5r3/evidence-status.md#a11d-5r3-manifest`](a1.1d-5r3/evidence-status.md#a11d-5r3-manifest).

### `A1.1d-5R3-LINUX`

Canonical content: [`a1.1d-5r3/evidence-status.md#a11d-5r3-linux`](a1.1d-5r3/evidence-status.md#a11d-5r3-linux).

### `A1.1d-5R3-LINUX-CLOSEOUT`

Canonical content: [`a1.1d-5r3/evidence-status.md#a11d-5r3-linux-closeout`](a1.1d-5r3/evidence-status.md#a11d-5r3-linux-closeout).

### `A1.1d-5R3-MAC`

Canonical content: [`a1.1d-5r3/evidence-status.md#a11d-5r3-mac`](a1.1d-5r3/evidence-status.md#a11d-5r3-mac).
## A1.1d-5R3-MAC attempt-4 remediation status (2026-08-06)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06`](r3-mac-evidence-recovery/evidence-regression.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06).
## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN regression status (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-plan-regression-status-2026-08-07`](r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-plan-regression-status-2026-08-07).
## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN regression-command and status correction (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-plan-regression-command-and-status-correction-2026-08-07`](r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-plan-regression-command-and-status-correction-2026-08-07).
## AUX-R3-MAC-EVIDENCE-RECOVERY-R3 recovery-current validator invocation (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07`](r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07).

## AUX-R3-MAC-SYSTEM-KEYCHAIN-SOFTWARE-SIGNER-CORRECTION regression ledger (2026-08-10)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-system-keychain-software-signer-correction-regression-ledger-2026-08-10`](r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-system-keychain-software-signer-correction-regression-ledger-2026-08-10).
## R3 macOS retirement/orphan planning and G2/G3 stop ledger (2026-08-13)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#r3-macos-retirementorphan-planning-and-g2g3-stop-ledger-2026-08-13`](r3-mac-evidence-recovery/evidence-regression.md#r3-macos-retirementorphan-planning-and-g2g3-stop-ledger-2026-08-13).
## Current cross-lane regression ledger (2026-08-20; controlling)

Canonical content: [`macos-dev-parity/evidence-regression.md#current-cross-lane-regression-ledger-2026-08-20-controlling`](macos-dev-parity/evidence-regression.md#current-cross-lane-regression-ledger-2026-08-20-controlling).
