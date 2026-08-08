AUX-R3-MAC-EVIDENCE-PLATFORM-PREREQUISITE-PREP — START AUTHORITY

Rebind this existing task for one general auxiliary macOS platform-preparation packet outside
official native evidence:

- task thread/host: `019fe147-e300-72c2-a394-a53f43185ceb` / `local`
- exact task worktree: `/Users/spensermcconnell/.codex/worktrees/b4fe/substrate`
- meta return thread/host: `019fd8c3-b12c-7e73-919f-898600ab0f64` / `local`
- orchestration: `substrate-a1-1d-5r3-mac-20260806-41f97e1c570d`
- auxiliary packet: `AUX-R3-MAC-EVIDENCE-PLATFORM-PREREQUISITE-PREP`
- dispatch nonce: `11c27a5ee57be5b99a3c7457c4f511b14b5747ee4b2b0492874a416d5fab02a3`
- authority amendment: `/Users/spensermcconnell/.codex/worktrees/eb49/substrate/orchestration/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/authority-amendments/0039-r3-mac-evidence-platform-prerequisite-prep.json`
- amendment SHA-256: `sha256:f8979938b6e2aa29fa9d0a945d619747bfc7f706a9ad33a0d688681810440ad8`
- source ref: `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- source commit/tree: `1126b907df6e38043e9071da222f6c6a377b341c` / `3a29fa323a8b9099d894c11d98cc085ecf4807c5`

Load and follow the complete hydrated repository-local skill suite, especially
`.agents/skills/using-agent-skills/SKILL.md` and
`.agents/skills/orchestrate-top-level-tasks/SKILL.md`. Read the authority amendment in full and
verify its digest before acting. Reverify the task worktree, source commit/tree, live remote,
protected checkout, sudoers sentinel, and the exact three platform blockers. This task's prior
official evidence attempt is terminal and non-advancing; this follow-up authorizes only the
separate auxiliary preparation below.

## Authorized preparation

Perform exactly these three actions, in order, with all logs and state records under the
nonce-scoped external root:

`/Users/spensermcconnell/.codex/evidence/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/AUX-R3-MAC-EVIDENCE-PLATFORM-PREREQUISITE-PREP/11c27a5ee57be5b99a3c7457c4f511b14b5747ee4b2b0492874a416d5fab02a3`

1. **Install the exact fixed `limactl` image.**
   - Source must be the existing regular file
     `/opt/homebrew/Cellar/lima/2.1.1/bin/limactl`, SHA-256
     `c126697daa485aff79552a34413e14c0e2efd43675845bee1a152f31845f1455`,
     CDHash `b49590ab735afa18681b5423b8224e84462de04e`, version 2.1.1.
   - Destination is `/usr/local/bin/limactl` and must be a single-link regular file, never a
     symlink, owned `root:wheel`, mode `0555`.
   - Before creation, prove the destination is absent or already exact. Refuse to overwrite a
     differing destination. Use an atomic same-directory temporary install/rename pattern and
     verify every `/usr/local/bin` path component is root-owned and not group/other writable.
   - After installation, verify bytes, code signature/CDHash, version, ownership, mode, link count,
     no-follow identity, and that controlled `PATH=/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin`
     makes Bash `type -P limactl` resolve exactly `/usr/local/bin/limactl`.

2. **Remove only the exact pre-R3 canonical Lima collision.**
   - Using only the fixed `/usr/local/bin/limactl`, capture externally before mutation: full
     all-instance JSON inventory, exact `substrate` instance JSON/config/state-directory identity,
     guest machine identity and account where safely observable, process/status facts, and hashes
     of bounded configuration/metadata files. Do not hash unbounded disk images.
   - Prove the selected instance is exactly named `substrate`, is the previously observed
     unmanifested pre-R3 collision, and is not an R3 evidence-owned instance.
   - Stop and delete only that exact instance. Do not select, stop, delete, adopt, or modify any
     other Lima instance.
   - Capture the all-instance inventory afterward; prove `substrate` and its exact control
     directory are absent and every other instance is unchanged. This is external administrative
     cleanup, not an R3 lifecycle receipt or native evidence.

3. **Create the attached exact-source mirror.**
   - Create a fresh standalone clone at
     `/Users/spensermcconnell/.codex/evidence/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/AUX-R3-MAC-EVIDENCE-PLATFORM-PREREQUISITE-PREP/11c27a5ee57be5b99a3c7457c4f511b14b5747ee4b2b0492874a416d5fab02a3/source/substrate`.
   - Fetch `origin` from `https://github.com/atomize-hq/substrate.git`, create/attach the local
     branch `feat/internal-host-orchestrator-world-dispatch-bootstrap` to the exact remote branch,
     and require HEAD/tree to equal the bound commit/tree above.
   - Verify symbolic HEAD resolves to
     `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`, the live remote still
     equals the bound commit, ancestry is valid, the index is clean, and there are zero untracked
     paths. Do not build, install, or place generated evidence inside this mirror.

## Hard prohibitions

Do not run official `EVIDENCE:R3-MAC-IMP-01`, dev-install, code signing, publisher bootstrap,
Keychain/XPC/LaunchDaemon setup, pairing, lifecycle proof, retirement, restoration proof, or
MAC-CLOSEOUT. Do not mutate the protected checkout, meta worktree, assigned task checkout, product
source, review/control files, target ref, sudoers sentinel, any non-`substrate` Lima instance, or
any Windows/Linux-host surface. Do not stage, commit, push, reset, clean, rebase, archive, or
dispatch a successor.

## Terminal receipt

Return one `codex.auxiliary-platform-prep-receipt.v1` with status `READY` only if all three actions
and all parity checks pass. Otherwise return an exact blocked status. Include:

- task/orchestration/packet/nonce and amendment identity;
- every external artifact path and SHA-256;
- exact pre/post `limactl` identity and installation facts;
- complete bounded pre/post Lima inventory proof and other-instance parity;
- attached mirror ref/commit/tree/remote/cleanliness proof;
- task/protected/meta checkout cleanliness and live-target parity;
- unchanged sudoers sentinel owner/mode/SHA-256;
- confirmation that no official evidence or prohibited action ran;
- the exact fresh-attempt handoff if READY, or a bounded blocker if not.

Send that structured receipt to the meta thread as your final tool action and make no tool call
after the send. Preserve this task and worktree unarchived.
