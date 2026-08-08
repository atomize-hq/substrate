# EVIDENCE:R3-MAC-IMP-01 attempt-3 agent-operated native read-only contract

## Authority and source

- Orchestration: `substrate-a1-1d-5r3-mac-20260806-41f97e1c570d`.
- Evidence ID: `EVIDENCE:R3-MAC-IMP-01`.
- Dispatch nonce: `7c83b749b2360596d88553c24c52d89bdbfc7e2024170712b7d248e6b6ae31b7`.
- Platform: native supported macOS on host `local`.
- Saved project: `local-b2016f8a311fef93149e42eaec4d704d` at
  `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`.
- Remote/ref: `origin` / `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`.
- Published source commit: `3d6b2eb1b02b1a24a1e055d12e5cbdb9b0312774`.
- Published source tree: `d59cac92405f06024cfcde0a2c22a4cf1255c3d7`.
- Source implementation task: `019fded7-05c4-7c61-9f7d-5fc456bcda6d` on host `local`.
- Return meta task: `019fd8c3-b12c-7e73-919f-898600ab0f64` on host `local`.
- Artifact-gated successor: `AUTHORITY_REQUIRED:A1.1d-5R3-MAC-CLOSEOUT`.

The current user-bound local project identity above controls this dispatch. The older remote Linux
project ID embedded in preserved planning text is historical and must not replace this exact native
macOS project binding.

## Attempt-3 authority and recovered-source correction

The original agent-operated native authority remains in force, and the independently verified
R1-R6 recovery landing at `3d6b2eb1b02b1a24a1e055d12e5cbdb9b0312774` is the sole source
checkpoint for this attempt. R6-CORRECTION-01 removed piped confirmation and requires a direct
control-owned terminal launch plus an independently persisted guest proof before data-session
intent. Exercise that implementation exactly; do not recreate the rejected pipe relay. The human user
has explicitly delegated the complete native run to this evidence agent. You must derive the exact
InstallBootstrapContextV1 (IH) and PlatformBootstrapMappingV1 (PM) yourself from repository-owned
canonical resolvers and live verified state; neither is a user-supplied input. The canonical selected
Lima instance is `substrate`. You must create and operate the independent host and guest controlling
PTYs yourself and enter the literal pairing confirmation through those PTYs. Record that the human
provided advance delegated authority and that the evidence agent operated the PTYs; do not claim the
human typed the confirmation. Do not pause for or request additional user interaction.

Reverify host and guest `sudo -n true` at dispatch time before mutation. The user already
installed the root-owned sudoers authorization sentinel and delegated this native run; failure of
the current timestamp or a technical privilege prerequisite is a platform blocker, not a request
to redesign source. The root-owned
`/etc/sudoers.d/substrate-r3-mac-evidence` file is a pre-existing authorization sentinel: observe and
preserve it byte-for-byte, use sudo only within this MAC evidence packet, and never edit, remove, or
replace the sudoers file.

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
`/Users/spensermcconnell/.codex/evidence/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/EVIDENCE-R3-MAC-IMP-01/7c83b749b2360596d88553c24c52d89bdbfc7e2024170712b7d248e6b6ae31b7/`.
Use external Cargo/build/temp roots so native collection does not dirty the checkout.

No product publication is allowed. The later MAC-CLOSEOUT task alone may copy the validated
external artifact and receipt into review-control and publish those evidence-only bytes.

## Mandatory preflight

Before any host or guest mutation:

1. Prove the assigned checkout HEAD/tree and live remote ref all equal the bound source.
2. Record macOS version/architecture, tool versions, current intended principal, origin, project,
   worktree, and task correlation.
3. Derive IH canonically from the installed/selected host prefix and the current account-database
   account/UID using `InstallBootstrapContextV1` or the byte-exact public resolver in the MAC scripts.
   Derive PM for canonical instance `substrate` from two equal machine-ID observations, the guest
   account database, account-derived Lima control root, and exact transport sockets using
   `PlatformBootstrapMappingV1`. Validate canonical re-encoding and the IH commitment join. Do not
   treat the absence of prompt-supplied carrier/mapping bytes as a blocker.
4. Prove supported Lima and SSH availability; bind the existing canonical instance, its disposition
   and machine identity, clean platform-control root, LaunchDaemon/System-Keychain baseline,
   known-hosts/socket/process state, sudoers authorization sentinel, and unrelated state sentinels.
5. Capture and hash the canonical pre-action baseline required by `04` and `05`.
6. Use the landed R6 direct-terminal flow from a real visible Terminal/PTY under the user's advance
   delegated authority. The control process must own the host controlling terminal and the fixed
   `limactl --tty=true` child must open the independent guest controlling TTY. Enter the fingerprint,
   challenge, and literal through that real interactive guest TTY. A pipe, ordinary non-TTY stdin,
   file, environment variable, synthetic proof, or byte-relay bypass is forbidden. Record both TTY
   identities and the durable operator-proof/data-session join. Do not wait for the user; use
   available agent-operated Terminal/computer-control tooling for literal keystrokes.
7. Perform the required restored same-filesystem disposable guest `/var/lib/substrate` atomic
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
kill; non-PTY/file/environment pairing bypass; false claims that the user typed the confirmation;
passive-health remediation; modification/removal of the sudoers authorization sentinel; final
native-gate claims; MAC-CLOSEOUT; WIN/UNIX work; or any successor dispatch. Do not spawn subagents.

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
  --expected-source-commit 3d6b2eb1b02b1a24a1e055d12e5cbdb9b0312774 \
  --expected-source-tree d59cac92405f06024cfcde0a2c22a4cf1255c3d7 \
  --expected-source-ref refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap \
  --expected-product-project-id local-b2016f8a311fef93149e42eaec4d704d \
  --expected-gated-successor AUTHORITY_REQUIRED:A1.1d-5R3-MAC-CLOSEOUT
```

Create an external `codex.top-level-evidence-receipt.v1`. Its source publication mode is remote,
`source.live_remote` must equal `source.commit`, its artifact SHA-256 must equal the validated
artifact bytes, gates must be sorted/unique, and it must confirm prohibited actions did not run and
the checkout is unchanged. Validate it with the hydrated skill validator's single positional
receipt argument. Do not claim that validator checks a successor; the artifact validator does.

For unavailable platform or technical prerequisites use `BLOCKED_PLATFORM_HANDOFF_REQUIRED`.
Prompt-supplied IH/PM and additional live user input are not prerequisites under this amendment. For
an available run that fails or cannot restore use `BLOCKED_NATIVE_EVIDENCE`. Include source,
environment, exact failure, restoration facts, authority required, and a complete continuation
handoff.

Send the single structured evidence receipt JSON to the bound meta task with
`codex_app__send_message_to_thread`. That send must be your final tool action. After it succeeds,
make no further tool call or external action and return only the human-readable report.


## Attempt-3 mandatory recovery gates

Before native mutation, prove the landed R1-R6 source contains and the invoked binaries exercise:
the Bash-3.2 installer fix, macOS compile closure, dispatch-bound validator project ID, Keychain/XPC
ticket issuer, typed mapped lifecycle submission, direct-terminal operator launch, immutable guest
operator proof, independent data-session proof validation, current-generation retry, exact consumed
retry, and uniform expiry preservation. Run the canonical macOS dev install from the exact source
with `/opt/homebrew/bin/bash` only if the default shell wrapper selects it as the repository-owned
compatibility path. The selected prefix `/Users/spensermcconnell/.substrate` must already be owner
501 mode 0700 and must remain contract-valid.

The historical failed attempt artifacts are diagnostic only and cannot satisfy any gate. In
particular, do not use the historical project UUID `2ccb802f-301c-4af4-9bd5-51d22808f0a2`; the
validator must receive the exact dispatch-bound local project ID above.

The source checkout must remain byte-clean throughout. Native host/guest mutations are permitted
only under the recorded baseline and restoration plan. If any repository-owned native action
fails after mutation, restore first and return `BLOCKED_NATIVE_EVIDENCE` with exact parity facts.
