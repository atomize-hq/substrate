META CONTINUATION BINDING AND START AUTHORITY - EVIDENCE ATTEMPT 10

Continue the existing idle evidence task with a fresh nonce. Bind exactly:

- task: 019fe162-c98c-7c82-94d0-155d6cab5122 / local
- worktree: /Users/spensermcconnell/.codex/worktrees/1626/substrate
- orchestration/evidence: substrate-a1-1d-5r3-mac-20260806-41f97e1c570d / EVIDENCE:R3-MAC-IMP-01
- attempt/allocation: 10 / 3
- dispatch nonce: 0a7b308d484582037972c5dface88dcfb9e7443ef343c799c4e177504a836ce8
- source commit/tree: 2511eea6e93bbdc6d24069356aa42064cd760305 / f9441d449321c31693b2238c3a33abdb222dd9dc
- target: origin refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
- project: local-b2016f8a311fef93149e42eaec4d704d at /Users/spensermcconnell/__Active_Code/atomize-hq/substrate
- skill suite: 25 skills / 54 files / sha256:e3de93f358594033ed59696a9624d2d0ac2fd0c2aa7ddf4a532810a05b262282
- contract: increments/EVIDENCE-R3-MAC-IMP-01.attempt-10.contract.md
- contract SHA-256: sha256:0b1d2daaed9f2428e3eeea10d4323269b5fbf5816bfd26a023b22592181edcac
- attempt authority amendment 0049 SHA-256: sha256:9f59f1da89c8d69b88ed8d23b3eb6072c3801565bd6f49a33a699e126918e2b9
- model/reasoning: retain current task runtime
- start authorized: true

Meta independently verified the task checkout and attached mirror are exact and clean at the new
published source; the mirror remains attached to the target branch, its target/ is absent, the
canonical Lima instance/control root are absent, fixed /usr/local/bin/limactl and the sudoers
sentinel are unchanged, and sudo -n true succeeds. Attempt 9 is accepted only as a validated blocked
artifact with exact restoration. Restart the full packet; do not reuse its actions as success.

This prompt includes the complete authoritative attempt-10 contract below. Follow it exactly. Send
one structured receipt to meta as your final tool action. Do not dispatch MAC-CLOSEOUT, archive any
task, edit source, or ask the user for interaction.

--- BEGIN COMPLETE ATTEMPT-10 CONTRACT ---

# EVIDENCE:R3-MAC-IMP-01 attempt-10 agent-operated native read-only contract

## Authority and source

- Orchestration: `substrate-a1-1d-5r3-mac-20260806-41f97e1c570d`.
- Evidence ID: `EVIDENCE:R3-MAC-IMP-01`.
- Dispatch nonce: `0a7b308d484582037972c5dface88dcfb9e7443ef343c799c4e177504a836ce8`.
- Platform: native supported macOS on host `local`.
- Saved project: `local-b2016f8a311fef93149e42eaec4d704d` at
  `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`.
- Remote/ref: `origin` / `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`.
- Published source commit: `2511eea6e93bbdc6d24069356aa42064cd760305`.
- Published source tree: `f9441d449321c31693b2238c3a33abdb222dd9dc`.
- Source implementation task: `019fded7-05c4-7c61-9f7d-5fc456bcda6d` on host `local`.
- Source compile-correction task: `019fdfa4-ba96-70e0-b68d-1d0a4cf32a83` on host `local`.
- Return meta task: `019fd8c3-b12c-7e73-919f-898600ab0f64` on host `local`.
- Artifact-gated successor: `AUTHORITY_REQUIRED:A1.1d-5R3-MAC-CLOSEOUT`.

The current user-bound local project identity above controls this dispatch. The older remote Linux
project ID embedded in preserved planning text is historical and must not replace this exact native
macOS project binding.

## Attempt-10 authority, corrected source, and retained amendments

This attempt is additionally bound to meta authority amendment `0040-r3-mac-canonical-installer-target-output` at `authority-amendments/0040-r3-mac-canonical-installer-target-output.json` (SHA-256 `sha256:cdaaa4ad590d4a110dd8ae5b03c36ac4833abd64e92e0e0ad055826d78495c14`). Attempt 6 proved that the unmodified canonical installer owns and requires `REPO_ROOT/target/<profile>`. Amendment 0040 permits exactly that ignored installer-owned `target/` tree in the attached mirror for this attempt, requires it to be absent before use and after restoration, and authorizes no source edit or other repository-side output. This bounded contract correction supersedes only the attempt-6 external-output/byte-clean contradiction; every other source, platform, MAC-only, direct-TTY, restoration, and successor fence remains in force.

Attempt 10 is also bound to amendment `0041-r3-mac-canonical-installer-fixed-path-retry` at `authority-amendments/0041-r3-mac-canonical-installer-fixed-path-retry.json` (SHA-256 `sha256:3eb4250e6995c6dd91614778b7d448dc74b680e199a47d2e754330cdd0eeed38`). Attempt 7 restored exactly but incorrectly started the canonical installer with an ambient PATH that resolved `/opt/homebrew/bin/limactl`. This amendment adds no action or scope. It requires the exact installer process itself to start with `CARGO_TARGET_DIR` unset and `PATH=/usr/local/bin:/Users/spensermcconnell/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin`, after proving `type -P limactl` is `/usr/local/bin/limactl`. The Homebrew Lima binary must not be executed.

The published source is also the independently reviewed correction from amendment
`0042-r3-mac-limactl-home-provenance-source-correction` (SHA-256
`sha256:815ed0977d5affb77560fb63de0e5474e2f0f7e2db7aeeefd3ca34bb9d164fbe`). Inside the canonical
installer, the privileged fixed-limactl version probe must retain `env -i`, the fixed privileged
PATH, and the already validated absolute no-follow limactl image; it must validate `/var/empty` as
a root-owned non-symlink directory without group/other write bits and supply `HOME=/var/empty` only
to that probe. Do not forward caller HOME into that privileged probe and do not change the
installer-start HOME required by amendment 0041.

Meta also independently validated amendment `0048-r3-mac-fixed-install-source-checkout-refresh` (SHA-256
`sha256:ed278102cb7f12efbffae67575722c41685fa262a9654b423da07f8c19e45bf6`). The attached mirror is
clean and attached to the exact target branch at the new source, and this evidence checkout is clean
and detached at the same commit/tree. The mirror `target/` path and canonical Lima instance/control
root are absent. Reverify, but do not repeat or reinterpret that administrative preparation.

Attempt 10 is authorized by amendment `0049-r3-mac-evidence-attempt-10-fixed-install-retry` (SHA-256
`sha256:9f59f1da89c8d69b88ed8d23b3eb6072c3801565bd6f49a33a699e126918e2b9`). It restarts the complete
official packet from a fresh pre-mutation baseline and carries forward only the exact 0040/0041
installer exceptions plus the landed 0042 and 0047 source corrections and verified 0048 checkout state.

The original agent-operated native authority remains in force, and the independently verified
R1-R6 recovery, AArch64 guest compile correction, limactl HOME provenance correction, and fixed install-sequence correction landing at `2511eea6e93bbdc6d24069356aa42064cd760305` is the sole source
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

Reverify host and guest `sudo -n true` at dispatch time before mutation. Before dispatch, meta
invalidated the host sudo cache twice and independently verified durable `sudo -n true` success.
The root-owned authorization sentinel had owner/group/mode `root:wheel 0440` and SHA-256
`f4f85d5b2674a6505d84ee75de0a3cc8bc883bace01733bcab3b5fbd6efd4f0d`. The user delegated
this native run; a later technical privilege failure is a platform blocker, not authority to redesign
source. The root-owned
`/etc/sudoers.d/substrate-r3-mac-evidence` file is a pre-existing authorization sentinel: observe and
preserve it byte-for-byte, use sudo only within this MAC evidence packet, and never edit, remove, or
replace the sudoers file.

## Identity and continuation barrier

This is a fresh nonce-bound attempt in the same preserved evidence task. Before any discovery or
evidence action, rebind and verify the real task thread ID `019fe162-c98c-7c82-94d0-155d6cab5122`,
host `local`, exact assigned worktree `/Users/spensermcconnell/.codex/worktrees/1626/substrate`,
hydrated repository-local skill digest, this attempt-10 nonce, exact source commit/tree/ref, and live
remote. Stop on mismatch. Attempts 1-9 and their artifacts are diagnostic inputs only and may not satisfy any
attempt-10 gate.

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

## Read-only checkout, canonical-installer output exception, and external artifact boundary

The assigned evidence worktree is an identity/audit checkout only. It must begin and end at the
exact published commit/tree with a clean index and untracked set. The sole installer/build source is
the independently verified attached mirror
`/Users/spensermcconnell/.codex/evidence/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/AUX-R3-MAC-EVIDENCE-PLATFORM-PREREQUISITE-PREP/11c27a5ee57be5b99a3c7457c4f511b14b5747ee4b2b0492874a416d5fab02a3/source/substrate`.
That mirror has symbolic HEAD `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
and exact commit/tree `2511eea6e93bbdc6d24069356aa42064cd760305` /
`f9441d449321c31693b2238c3a33abdb222dd9dc`, so canonical installer provenance records the bound
ref instead of `detached`.

Amendment 0040 permits one narrow repository-side output: the unmodified canonical installer and
Cargo may create ignored build output only inside the exact real-directory path `<attached
mirror>/target`. Before use, prove that path is absent and capture the no-follow identity of every
parent component. Do not set `CARGO_TARGET_DIR` for the canonical installer. Do not edit, copy,
patch, wrap, or substitute the installer. Do not create any other tracked, untracked, or ignored
byte in the mirror. The assigned task worktree, protected checkout, and every other repository
checkout remain immutable.

All command logs, captures, separately required AArch64 guest-executor output, staging not owned by
the canonical installer, receipts, and the final artifact must remain under the external attempt-10
nonce-scoped directory
`/Users/spensermcconnell/.codex/evidence/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/EVIDENCE-R3-MAC-IMP-01/0a7b308d484582037972c5dface88dcfb9e7443ef343c799c4e177504a836ce8/`.
Before terminal receipt, stop all installer/build children, retain the required artifact identities,
then remove only the exact pre-absent task-created mirror `target/` after revalidating device/inode,
owner, real-directory, and no-symlink identity. Prove `target/` is absent and the mirror is again
attached to the bound ref, exact commit/tree, tracked-clean, and has zero non-ignored untracked
paths. Any inability to restore this exact absence is `BLOCKED_NATIVE_EVIDENCE`.

No product publication is allowed. The later MAC-CLOSEOUT task alone may copy the validated
external artifact and receipt into review-control and publish those evidence-only bytes.

## Mandatory preflight

Before any host or guest mutation:

1. Prove the assigned checkout HEAD/tree and live remote ref all equal the bound source.
2. Record macOS version/architecture, tool versions, current intended principal, origin, project,
   worktree, and task correlation.
3. Derive IH canonically from the installed/selected host prefix and the current account-database
   account/UID using `InstallBootstrapContextV1` or the byte-exact public resolver in the MAC scripts.
   Prove the canonical `substrate` instance and control directory are absent before Stage-1. Do not
   attempt to derive final PM before the authorized Stage-1 create/start effect. After Stage-1 creates
   the owned instance, derive PM from two equal machine-ID observations, the guest account database,
   account-derived Lima control root, and exact transport sockets using `PlatformBootstrapMappingV1`.
   Validate canonical re-encoding and the IH commitment join. Do not treat the absence of
   prompt-supplied carrier/mapping bytes as a blocker.
4. Prove supported SSH availability and use only the fixed root-owned no-follow Lima image at
   `/usr/local/bin/limactl`, selected by controlled
   `PATH=/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin`. Prove its exact SHA-256, CDHash, version,
   owner/mode, and no-follow identity. Capture the absent-instance disposition, clean platform-control
   root, LaunchDaemon/System-Keychain baseline, known-hosts/socket/process state, sudoers
   authorization sentinel, and unrelated state sentinels.
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

Prohibited: repository mutation or publication except the exact temporary ignored attached-mirror
`target/` tree authorized by amendment 0040; ambient `auto_select`; alternate endpoint,
transport, instance, principal, or prefix; unowned instance destruction; unrelated known-hosts,
Keychain, launchd, network, control-state, home, or VM mutation; path-existence-only unlink; broad
kill; non-PTY/file/environment pairing bypass; false claims that the user typed the confirmation;
passive-health remediation; modification/removal of the sudoers authorization sentinel; final
native-gate claims; MAC-CLOSEOUT; WIN/UNIX work; or any successor dispatch. Do not spawn subagents.

## Restoration and proof

On success or failure, execute the recorded restoration plan, restore the attached-mirror
`target/` path to exact absence, and prove exact parity for staged and
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
  --expected-source-commit 2511eea6e93bbdc6d24069356aa42064cd760305 \
  --expected-source-tree f9441d449321c31693b2238c3a33abdb222dd9dc \
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


## Attempt-10 mandatory recovery gates

Before native mutation, prove the landed R1-R6 plus AArch64 compile correction and limactl HOME provenance correction source contains and the invoked binaries exercise:
the Bash-3.2 installer fix, macOS compile closure, dispatch-bound validator project ID, Keychain/XPC
ticket issuer, typed mapped lifecycle submission, direct-terminal operator launch, immutable guest
operator proof, independent data-session proof validation, current-generation retry, exact consumed
retry, and uniform expiry preservation. Run the canonical macOS dev install from the exact source
with `/opt/homebrew/bin/bash` only if the default shell wrapper selects it as the repository-owned
compatibility path. The selected prefix `/Users/spensermcconnell/.substrate` must already be owner
501 mode 0700 and must remain contract-valid.

For the canonical installer, invoke exactly from the attached mirror with `/usr/bin/env -u CARGO_TARGET_DIR PATH=/usr/local/bin:/Users/spensermcconnell/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin HOME=/Users/spensermcconnell /opt/homebrew/bin/bash scripts/substrate/dev-install-substrate.sh ...`. Immediately before exec, record that Bash `type -P` resolves `limactl`, `cargo`, `rustc`, and `bash` to `/usr/local/bin/limactl`, `/Users/spensermcconnell/.cargo/bin/cargo`, `/Users/spensermcconnell/.cargo/bin/rustc`, and `/opt/homebrew/bin/bash`. Apply this environment to the installer process from its first instruction; a controlled PATH used only by preflight or later child commands does not satisfy the gate.

Before the canonical installer, prove the current source contains the landed fixed-provenance
behavior and run `/bin/bash tests/mac/dev_install_limactl_provenance_home_r3.sh` or an exact stronger
source-owned equivalent. This proof does not replace the native installer run. During the real run,
record that the privileged fixed-limactl probe succeeds with its root-controlled `HOME=/var/empty`
and still binds the exact fixed executable identity.

The historical attempts 1-9 and their blocked artifacts are diagnostic only and cannot satisfy any gate. In
particular, do not use the historical project UUID `2ccb802f-301c-4af4-9bd5-51d22808f0a2`; the
validator must receive the exact dispatch-bound local project ID above.

The assigned checkout and tracked source tree must remain byte-clean throughout; the sole attached-mirror `target/` exception is governed by amendment 0040 and must return to absence. Native host/guest mutations are permitted
only under the recorded baseline and restoration plan. If any repository-owned native action
fails after mutation, restore first and return `BLOCKED_NATIVE_EVIDENCE` with exact parity facts.


## Attempt-10 bound platform prerequisites

The AArch64 Linux guest build prerequisite is already provisioned and must be used exactly: Rust
toolchain `1.89.0-aarch64-apple-darwin`, target `aarch64-unknown-linux-gnu`, Zig `0.16.0`, and
external linker wrapper
`/Users/spensermcconnell/.codex/evidence/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/AUX-R3-MAC-AARCH64-TOOLCHAIN-PREP/d514db6915877692c799086ce6c96daad6bcd0417d87a1bdf9ba44c2c578da53/linker/aarch64-linux-gnu-zig-cc`.
Set `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER` to that exact wrapper for the external locked
offline guest executor build. The source correction at `2511eea6e93bbdc6d24069356aa42064cd760305`
has already passed that exact cross-target check and build. Do not install or substitute another
toolchain, and do not treat the corrected C `c_char` ABI declarations as a new blocker without
reproducing a current exact-source failure.

The evidence task must preserve the sudoers sentinel byte-for-byte. It may use the existing
NOPASSWD authorization only for actions already allowed by this MAC evidence packet. It must not
modify sudoers, broaden the packet, or request user interaction.

## Attempt-10 verified platform-preparation inputs

Meta independently validated auxiliary platform-preparation receipt
`receipts/AUX-R3-MAC-EVIDENCE-PLATFORM-PREREQUISITE-PREP.json` with its 52-file external artifact
manifest, and source-checkout refresh receipt `receipts/AUX-R3-MAC-SOURCE-CHECKOUT-REFRESH-02.json`
with its verified 19-file manifest, before this dispatch. Use these exact recovered prerequisites; do not redo or reinterpret
the administrative cleanup:

- fixed Lima executable `/usr/local/bin/limactl`, regular non-symlink `root:wheel 0555`, one link,
  SHA-256 `c126697daa485aff79552a34413e14c0e2efd43675845bee1a152f31845f1455`, CDHash
  `b49590ab735afa18681b5423b8224e84462de04e`, version 2.1.1;
- the pre-R3 unmanifested `substrate` collision and `/Users/spensermcconnell/.lima/substrate`
  control directory are absent; no other Lima instances existed or changed;
- the attached exact-source mirror named above is clean, attached to the bound branch, and
  remote-equal at the bound commit/tree;
- the sudoers sentinel remains `root:wheel 0440`, SHA-256
  `f4f85d5b2674a6505d84ee75de0a3cc8bc883bace01733bcab3b5fbd6efd4f0d` and `sudo -n true`
  succeeds.

Use the attached mirror as the working directory for the canonical dev installer and every
source-relative native command. The canonical installer alone uses the amendment-0040 mirror
`target/` exception; all other target/output roots remain external. The Stage-1
absence gate now passes by construction: the official packet itself must create the exact owned
instance, observe its machine identity, finalize PM, execute the rest of the packet, and restore the
recorded baseline according to the landed contract. Do not adopt an ambient instance or treat the
prior administrative deletion as lifecycle evidence.


## Attempt-10 fixed install route and evidence-only acceptance boundary

The landed source at 2511eea6e93bbdc6d24069356aa42064cd760305 is the reviewed closure for the attempt-9 blocker. Before mutation,
run the source-owned focused regressions for prestage artifact routing, lifecycle, compile surface,
Bash 3.2 installer behavior, and limactl HOME provenance. These are preflight checks only and do not
replace native evidence.

The canonical installer must itself build, verify, and atomically stage exactly the four existing
source-defined AArch64 Linux artifact roles used by the signed manifest. The historical
mac.lima.guest-binary(world) role is not an executable Cargo target and must not be invented.
Do not manually copy an external build into the prefix, introduce another ingress, or add/rename a
binary. Preserve host ExecutorBuildEvidenceV1 as host-only.

After Stage-1 create and PM finalization, the privileged macOS executor must derive and execute only
the landed private literal install-only sequence from the signed Stage-1 manifest. It must not
accept a caller selector, traverse the exhaustive action catalogue, or execute Stop, Remove, or
Restore. It must install the already built/staged artifacts and required guest configuration,
enable/start the required socket/service through the existing typed actions, and use the landed
durable receipt recovery law for exact retry. Prove the installed executable bit and no-follow
artifact identity checks exercised by the reviewed correction.

Amendment 0047 SHA-256 is
sha256:6bd998d38beb552233b6360ac7c02c482a7c2c096ea6c6bbb6e09ba5f6c222d4.
The independently verified source refresh is amendment 0048 SHA-256
sha256:ed278102cb7f12efbffae67575722c41685fa262a9654b423da07f8c19e45bf6,
receipt SHA-256 sha256:f114327cce88c0784d86b9e56147e75e082b9060d68ed1ee285c56ae4f16d3a4,
and external manifest SHA-256
sha256:04c68c13f11ac27c91a4914843291d65474d4f863342897e1169181f5bbcc703.

Attempt 9 is diagnostic only. No attempt-9 build, action, artifact, or receipt satisfies attempt 10.
Start from the verified absent canonical instance/control root and complete the full packet. If a
new repository-owned native action legitimately fails, restore exact parity first and return the
appropriate blocked receipt. Do not redesign or edit source in this evidence task.

--- END COMPLETE ATTEMPT-10 CONTRACT ---
