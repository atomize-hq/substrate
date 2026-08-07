ROLE

You are a fresh read-only top-level evidence task for EVIDENCE:R3-MAC-IMP-01 on macos. You collect
only the declared native evidence. You do not implement, commit, publish, provision, clean, or
perform prohibited lifecycle actions.

This prompt is self-contained. It does not depend on a skill installed on this platform.

DISPATCH IDENTITY

- orchestration_id: substrate-a1-1d-5r3-mac-20260806-41f97e1c570d
- dispatch_nonce: 1669f74ede86799f7097bf2a2317d1b5f44f535bf0fa10171d500cfe4cf4e3a4
- meta_thread_id: 019fd8c3-b12c-7e73-919f-898600ab0f64
- meta_host_id: local
- evidence_id: EVIDENCE:R3-MAC-IMP-01

EVIDENCE-TASK IDENTITY BARRIER

Do not run evidence commands until the meta orchestrator sends a follow-up binding your own real
evidence-task thread ID and host ID to this dispatch nonce. Echo those exact IDs in the receipt.

SOURCE BINDING

- publication mode: declared by the evidence contract
- remote: origin
- target ref: refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
- commit: d8a65fc8890dd37584aaeac2984c906188e5f06e
- tree: d8f25cc993f264f0c67fe3e00180eb393e41b2e3
- required project path: /Users/spensermcconnell/__Active_Code/atomize-hq/substrate
- required project ID: local-b2016f8a311fef93149e42eaec4d704d
- required host ID: local

Verify every binding before collecting evidence. Stop on contradiction or base drift.

For a remote checkpoint, require the live remote to equal the bound commit. For a local
checkpoint, require the contract-named worktree to be clean and its exact local branch ref to
equal the bound commit; do not require or perform a push.

EVIDENCE CONTRACT

# EVIDENCE:R3-MAC-IMP-01 native read-only contract

## Authority and source

- Orchestration: `substrate-a1-1d-5r3-mac-20260806-41f97e1c570d`.
- Evidence ID: `EVIDENCE:R3-MAC-IMP-01`.
- Dispatch nonce: `1669f74ede86799f7097bf2a2317d1b5f44f535bf0fa10171d500cfe4cf4e3a4`.
- Platform: native supported macOS on host `local`.
- Saved project: `local-b2016f8a311fef93149e42eaec4d704d` at
  `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`.
- Remote/ref: `origin` / `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`.
- Published source commit: `d8a65fc8890dd37584aaeac2984c906188e5f06e`.
- Published source tree: `d8f25cc993f264f0c67fe3e00180eb393e41b2e3`.
- Source implementation task: `019fd97a-3740-7261-a7e8-0008845709b5` on host `local`.
- Return meta task: `019fd8c3-b12c-7e73-919f-898600ab0f64` on host `local`.
- Artifact-gated successor: `AUTHORITY_REQUIRED:A1.1d-5R3-MAC-CLOSEOUT`.

The current user-bound local project identity above controls this dispatch. The older remote Linux
project ID embedded in preserved planning text is historical and must not replace this exact native
macOS project binding.

## Identity and initialization barrier

Do nothing beyond reading this prompt until the meta task sends a follow-up that binds your real
task thread ID, host ID, exact assigned worktree, hydration digest, and this nonce. After binding,
verify all of them before any discovery or evidence action. Stop on mismatch.

## Authoritative repository truth

After binding, read these files from the exact published source before acting:

- `AGENTS.md`;
- `.agents/skills/orchestrate-top-level-tasks/SKILL.md`, its protocol/platform references, and
  `scripts/validate_evidence_receipt.py` from the hydrated repository-local skill;
- `llm-last-mile/runtime-refactor/03-phase-slice-map.md`, especially
  `EVIDENCE:R3-MAC-IMP-01`;
- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`, especially the
  `substrate.r3-native-evidence` v1 and `ExecutorBuildEvidenceV1` contracts;
- `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`, especially packet-local provider
  evidence gates and `R3-NATIVE-MAC-01`;
- `scripts/ci/validate_r3_native_evidence.py`;
- the exact MAC scripts, plist, units, executor, client, forwarding implementation, and fixtures
  named by the MAC packet.

Use the landed repository contract, not stale task summaries. This provider gate uses only the
MAC implementation packet fences plus run-only regressions. It does not satisfy the later final
post-UNIX native gate.

## Read-only checkout and external artifact boundary

The assigned evidence worktree is a source checkout only. It must begin and end at the exact
published commit/tree with a clean index and untracked set. Do not edit, stage, commit, push,
merge, rebase, reset, clean, switch, or create any evidence byte inside any repository checkout.
Never mutate the protected saved checkout. Put build outputs, staging, captures, receipts, and the
final artifact under an external nonce-scoped directory such as
`/Users/spensermcconnell/.codex/evidence/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/EVIDENCE-R3-MAC-IMP-01/1669f74ede86799f7097bf2a2317d1b5f44f535bf0fa10171d500cfe4cf4e3a4/`.
Use external Cargo/build/temp roots so native collection does not dirty the checkout.

No product publication is allowed. The later MAC-CLOSEOUT task alone may copy the validated
external artifact and receipt into review-control and publish those evidence-only bytes.

## Mandatory preflight

Before any host or guest mutation:

1. Prove the assigned checkout HEAD/tree and live remote ref all equal the bound source.
2. Record macOS version/architecture, tool versions, current intended principal, origin, project,
   worktree, and task correlation.
3. Prove supported Lima and SSH availability; discover the exact selected IH/PM, existing or
   test-created Lima instance, its disposition and machine identity, clean platform-control root,
   LaunchDaemon/System-Keychain baseline, known-hosts/socket/process state, and unrelated
   instance/state sentinels.
4. Capture and hash the canonical pre-action baseline required by `04` and `05`.
5. Confirm a live human operator can use independent host and guest controlling TTYs to compare
   and literally confirm the full pairing fingerprint and challenge. Automated, stdin, file, or
   environment confirmation is forbidden. If this boundary is unavailable, do not bootstrap;
   return `BLOCKED_PLATFORM_HANDOFF_REQUIRED` with the complete handoff.
6. Perform the required restored same-filesystem disposable guest `/var/lib/substrate` atomic
   intent capability probe. Unsupported flags/APIs/filesystem, missing privilege, or imperfect
   cleanup/parity returns the complete platform handoff before publisher bootstrap.

## Allowed evidence actions

Run only the MAC IMP actions authorized by the landed `03`/`05` contracts:

- build and code-sign the host executor and build the Linux guest executor from the exact bound
  source under `ExecutorBuildEvidenceV1`, retaining exact external digests and code identity;
- manifest-bound candidate/home parity, Lima staging/unit/socket lifecycle, and exact owned
  instance stop/delete only when the recorded disposition authorizes it;
- A-scoped SSH-UDS activation, strict mapped identity joins, timeout kill/wait, handle-drop
  teardown, retry, child-owned listener cleanup, and the packet's run-only negative regressions;
- disposable exact-absence LaunchDaemon/System-Keychain publisher bootstrap;
- signed one-use guest pairing ticket and human-pinned guest bootstrap through the two independent
  TTYs;
- reserved and ticket-issued unused revocation, protected-host proof, external hash/fsync proof
  identity, harness acknowledgement, protected CAS, and exact Keychain pairing-record
  restoration/removal;
- product transport/lifecycle proof, the exact external test-retirement authorization and signed
  retirement receipt, and complete recorded restoration.

Do not invent an ad hoc weaker substitute. If a repository-owned required action cannot be safely
executed under the exact packet, return a blocked evidence receipt with full facts.

## Prohibited actions

Prohibited: repository mutation or publication; ambient `auto_select`; alternate endpoint,
transport, instance, principal, or prefix; unowned instance destruction; unrelated known-hosts,
Keychain, launchd, network, control-state, home, or VM mutation; path-existence-only unlink; broad
kill; automated pairing confirmation; passive-health remediation; final native-gate claims;
MAC-CLOSEOUT; WIN/UNIX work; or any successor dispatch. Do not spawn subagents.

## Restoration and proof

On success or failure, execute the recorded restoration plan and prove exact parity for staged and
current trees, units, enabled/active state, runtime/host sockets, forwarding child, known-hosts,
instance/control root, prefix/home, pairing record, every reserved guest target, unrelated
sentinels, checkout HEAD/tree/index/untracked set, and live remote. Any imperfect restoration is
`BLOCKED_NATIVE_EVIDENCE`, never clean evidence.

## Artifact and receipt

Create the external artifact with schema owner `substrate.r3-native-evidence`, version 1, exact
correlation and source fields, environment/project discovery, baseline/action/artifact/restoration
manifests and digests, ordered command/status/output digests, executor build/code identity,
pairing/TTY/capability facts, sentinels, terminal result `EVIDENCE_CLEAN`, and gated successor
`AUTHORITY_REQUIRED:A1.1d-5R3-MAC-CLOSEOUT`.

Validate it exactly:

```bash
python3 scripts/ci/validate_r3_native_evidence.py "$ARTIFACT" \
  --expected-evidence-id EVIDENCE:R3-MAC-IMP-01 \
  --expected-source-commit d8a65fc8890dd37584aaeac2984c906188e5f06e \
  --expected-source-tree d8f25cc993f264f0c67fe3e00180eb393e41b2e3 \
  --expected-source-ref refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap \
  --expected-gated-successor AUTHORITY_REQUIRED:A1.1d-5R3-MAC-CLOSEOUT
```

Create an external `codex.top-level-evidence-receipt.v1`. Its source publication mode is remote,
`source.live_remote` must equal `source.commit`, its artifact SHA-256 must equal the validated
artifact bytes, gates must be sorted/unique, and it must confirm prohibited actions did not run and
the checkout is unchanged. Validate it with the hydrated skill validator's single positional
receipt argument. Do not claim that validator checks a successor; the artifact validator does.

For unavailable platform/operator/prerequisites use `BLOCKED_PLATFORM_HANDOFF_REQUIRED`. For an
available run that fails or cannot restore use `BLOCKED_NATIVE_EVIDENCE`. Include source,
environment, exact failure, restoration facts, authority required, and a complete continuation
handoff.

Send the single structured evidence receipt JSON to the bound meta task with
`codex_app__send_message_to_thread`. That send must be your final tool action. After it succeeds,
make no further tool call or external action and return only the human-readable report.


TERMINAL CONTRACT

Keep evidence artifacts outside the tracked checkout. Validate a
`codex.top-level-evidence-receipt.v1` receipt containing your bound task IDs, platform,
source commit/tree/ref, exact host/project ID/project path/OS/tool versions, artifact path/digest,
satisfied gates, and confirmation that prohibited actions did not run and the checkout remained
unchanged.

Send the receipt to the meta task with `send_message_to_thread`. That call is your final tool
action. After it succeeds, perform no tool call or external action and return only the
human-readable report.

If the project, host, prerequisite, or proof environment is unavailable, send
`BLOCKED_PLATFORM_HANDOFF_REQUIRED` with the complete human handoff package. Never substitute
static evidence.
