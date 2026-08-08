# AUX-R3-MAC-SOURCE-CHECKOUT-REFRESH — start authority

Rebind this existing idle general auxiliary preparation task for one narrow source-checkout
refresh outside official native evidence.

## Identity

- Task thread/host: `019fe147-e300-72c2-a394-a53f43185ceb` / `local`.
- Task worktree: `/Users/spensermcconnell/.codex/worktrees/b4fe/substrate`.
- Meta return thread/host: `019fd8c3-b12c-7e73-919f-898600ab0f64` / `local`.
- Orchestration: `substrate-a1-1d-5r3-mac-20260806-41f97e1c570d`.
- Packet: `AUX-R3-MAC-SOURCE-CHECKOUT-REFRESH`.
- Dispatch nonce: `2d8c4c920d22e32e72c951edc1b52f74198f6d85764b9900d201d2e06cdb8bc3`.
- Authority amendment: `/Users/spensermcconnell/.codex/worktrees/eb49/substrate/orchestration/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/authority-amendments/0043-r3-mac-source-checkout-refresh.json`.
- Amendment SHA-256: `sha256:62ae8b3aec5cb677b3cef28f7e53bd31328d4b4c10b5dac297ff85a8d2e5e940`.

Load and follow the complete hydrated repository-local skill suite, especially
`using-agent-skills` and `orchestrate-top-level-tasks`. This is administrative preparation, not a
product increment and not official evidence. Do not spawn subagents.

## Exact transition

The live remote target
`origin refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap` now equals:

- commit `a06b3fd26ceee4ece7e5bd1481981755eafcedb5`
- tree `f4c396d0e28a43405a9a2b1d49a450d6df545c84`

It is a verified fast-forward descendant of:

- commit `1126b907df6e38043e9071da222f6c6a377b341c`
- tree `3a29fa323a8b9099d894c11d98cc085ecf4807c5`

Refresh exactly two clean checkouts and nothing else:

1. Attached canonical-installer mirror:
   `/Users/spensermcconnell/.codex/evidence/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/AUX-R3-MAC-EVIDENCE-PLATFORM-PREREQUISITE-PREP/11c27a5ee57be5b99a3c7457c4f511b14b5747ee4b2b0492874a416d5fab02a3/source/substrate`.
   It must begin at the old commit/tree, attached to the exact target branch, clean, with zero
   untracked paths and absent `target/`. Fetch exactly the target ref and use only a normal
   fast-forward merge to the exact new commit. It must remain attached to the same branch.
2. Idle official-evidence identity/audit checkout:
   `/Users/spensermcconnell/.codex/worktrees/1626/substrate`.
   It must begin clean and detached at the old commit/tree. Fetch exactly the target ref and switch
   the already-detached checkout to detached exact new commit. Do not create or move any branch.

Before either update, independently reverify every precondition in amendment 0043. If anything
differs, stop without mutation and return a blocked auxiliary receipt. Do not use reset, clean,
rebase, force, stash, or broad fetch/refspecs.

## Verification

After the two exact updates, prove:

- both checkouts have exact new HEAD/tree;
- the mirror remains attached to the target branch and the evidence checkout remains detached;
- both have clean index/worktree and zero untracked paths;
- mirror `target/` remains absent;
- origin target remains exact new commit and old is its ancestor;
- task worktree `/Users/spensermcconnell/.codex/worktrees/b4fe/substrate` remains clean and unchanged;
- protected checkout `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` remains clean and unchanged;
- `/usr/local/bin/limactl` remains root:wheel 0555, single-link regular, SHA-256
  `c126697daa485aff79552a34413e14c0e2efd43675845bee1a152f31845f1455`;
- the canonical `substrate` Lima instance and its control root remain absent;
- `/etc/sudoers.d/substrate-r3-mac-evidence` remains root:wheel 0440, SHA-256
  `f4f85d5b2674a6505d84ee75de0a3cc8bc883bace01733bcab3b5fbd6efd4f0d`.

Store command logs and the receipt outside every checkout under a fresh nonce-scoped subdirectory
of the existing auxiliary platform-prep external root. Do not modify the earlier 52-file manifest
or receipt; create a separate refresh record.

## Hard fences

No product/control/review source edit, stage, commit, push, installer, build, code-sign, native
evidence, Lima mutation, Keychain, launchd, sudoers, known-hosts, prefix, pairing, lifecycle,
retirement, MAC-CLOSEOUT, Windows/WSL/Linux-host work, successor dispatch, reset/clean/rebase,
task archival, or worktree disposal.

## Receipt

Return one `codex.auxiliary-source-checkout-refresh-receipt.v1` JSON containing:

- orchestration, packet, nonce, task thread/host/worktree;
- amendment path/digest;
- exact pre/post identities and ref posture for both updated checkouts;
- live remote and ancestry verification;
- cleanliness and mirror-target absence;
- fixed Lima, instance absence, sudoers, protected checkout, and task-worktree parity;
- external log/manifest paths and SHA-256 values;
- `status: READY` on success, or a precise blocker and handoff on failure;
- confirmation that all prohibited actions remained absent.

Send the receipt to the meta task with `codex_app__send_message_to_thread`. That send must be your
final tool action. After it succeeds, perform no further tool or external action and return only a
concise human-readable report. Do not dispatch official evidence yourself.
