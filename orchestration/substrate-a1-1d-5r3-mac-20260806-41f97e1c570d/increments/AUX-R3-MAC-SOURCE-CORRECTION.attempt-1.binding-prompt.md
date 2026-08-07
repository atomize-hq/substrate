GENERAL MACOS SOURCE-CORRECTION TASK IDENTITY BINDING AND START AUTHORITY

Bind this exact identity before taking any action:

- thread ID: `019fd9fe-670c-7253-bf60-86f78cd373b5`
- host ID: `local`
- exact assigned worktree: `/Users/spensermcconnell/.codex/worktrees/60cc/substrate`
- dispatch nonce: `d7c49c63813094f35052883eb0a2964d28ce2df44d2c422664ea69c7c5f7072c`
- return meta task: `019fd8c3-b12c-7e73-919f-898600ab0f64` on `local`
- model/reasoning: `gpt-5.6-terra` / Extra High (`xhigh`)
- exact base HEAD: `d8a65fc8890dd37584aaeac2984c906188e5f06e`
- exact base tree: `d8f25cc993f264f0c67fe3e00180eb393e41b2e3`
- live target: `origin` `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- project: `local-b2016f8a311fef93149e42eaec4d704d`
- hydrated skill: `/Users/spensermcconnell/.codex/worktrees/60cc/substrate/.agents/skills/orchestrate-top-level-tasks`
- hydration file count/digest: `26` / `61d99190f680d3738fa824348e671e8eb84b2729bf31952ef16031c6ba24d903`

Independent pre-binding checks confirmed that the assigned worktree is clean at the exact published
base/tree and hydration verifies. Start authority is explicit. Load `AGENTS.md` and the hydrated
skill, reverify the binding and live remote, then execute the entire contract below. All editing and
review subagents must use gpt-5.6-terra at Extra High. Your final tool action must send the structured
receipt to the return meta task. Do not install on the real host and do not dispatch any successor.

---

# General macOS installation/evidence source correction

## Initialization barrier

You are a fresh mutating top-level source-correction task. You are not the official
`EVIDENCE:R3-MAC-IMP-01` task and must not install onto the real host or collect native evidence.
Do not inspect external blocker artifacts, edit, delegate, test, commit, push, or mutate any host
state until a follow-up binds your real task thread/host identity, exact assigned worktree, nonce,
published base/tree/ref, hydrated repository-local skill, and explicit start authority.

Never mutate:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- `/Users/spensermcconnell/.codex/worktrees/eb49/substrate`
- `/Users/spensermcconnell/.codex/worktrees/abc3/substrate`
- `/Users/spensermcconnell/.codex/worktrees/46c9/substrate`
- `/Users/spensermcconnell/.codex/worktrees/a943/substrate`

## Objective after binding

Starting from exact published commit `d8a65fc8890dd37584aaeac2984c906188e5f06e`, fix every
verified source defect preventing a complete canonical Substrate installation on this macOS host
and preventing a later official R3 MAC evidence run. Integrate, independently review, validate,
commit exactly once, and fast-forward publish the corrected source to
`refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`.

This is a general corrective source task, not a new official R3 packet. Its scope is bounded by the
four independently verified blockers and any directly causal regressions they expose.

## Exact verified blockers

External read-only source artifacts are under:
`/Users/spensermcconnell/.codex/auxiliary-host-prep/019fd9ed-4067-7922-8034-da8abbc8717e`.

1. **Default macOS Bash incompatibility.** The canonical invocation
   `scripts/substrate/dev-install-substrate.sh --prefix /Users/spensermcconnell/.substrate --profile release`
   exits `127` under default macOS Bash 3.2 at `exec: {context_fd}: not found`. Correct the
   installer so its canonical invocation works without the caller manipulating `PATH`. Either use
   Bash-3.2-compatible FD handling or deliberately re-exec a discovered, verified supported Bash
   with safe argv/environment continuity and a deterministic failure when unavailable.
2. **Eleven macOS Rust compilation errors.** With Bash 5, the canonical installer reaches Cargo and
   exits `101` because `shell` has exactly 11 compile errors recorded in
   `canonical-dev-install-release-bash5.stderr.txt`: unresolved world-work imports/types, missing
   `libc::SOCK_CLOEXEC`, missing `install_context`, absent `b_owned_authority`, missing
   `start_remote_member_runtime`, and one `SubstrateConfig` type mismatch. Correct the actual
   platform/cfg/integration defects. Do not hide them by deleting behavior, suppressing compilation,
   weakening features, or excluding required binaries.
3. **Pairing-ticket issuer stub.** `issue_lima_guest_pairing_ticket_v1` currently validates a
   request and then unconditionally errors. Implement the genuine host-key/XPC signed guest ticket
   path required by the R3 MAC contract: exact PM/machine/artifact/request binding, protected key
   use, P-256/SPKI/signature normalization requirements, expiry, reservation/one-use semantics,
   durable pairing-record transitions, retries, and fail-closed negatives. Do not fabricate a
   ticket or bypass XPC/audit-token/key/Keychain authority.
4. **Stale fixed project UUID.** `scripts/ci/validate_r3_native_evidence.py` hard-codes Linux project
   UUID `2ccb802f-301c-4af4-9bd5-51d22808f0a2`. Replace this with an explicit expected project-ID
   binding supplied by the trusted caller/dispatch validation surface. Update every owned caller,
   fixture, test, and control document necessary. Preserve validation of historical Linux evidence
   and fail closed when the expected ID is absent or mismatched.

## Authority and scope

- Source project: `local-b2016f8a311fef93149e42eaec4d704d` on host `local`.
- Remote/ref: `origin` / `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`.
- Expected base/tree: `d8a65fc8890dd37584aaeac2984c906188e5f06e` /
  `d8f25cc993f264f0c67fe3e00180eb393e41b2e3`.
- Publication: exactly one normal fast-forward product/test/review commit; no merge, rebase,
  force-push, reset, or clean.
- You may edit production, installer, validator, test, fixture, documentation, and review-control
  paths directly necessary for these four corrections and direct causal regressions.
- Do not expand into unrelated runtime-refactor packets, architectures, selector families,
  transports, policy redesign, or general cleanup.
- Do not mutate the real host install, `~/.substrate`, Lima state, LaunchDaemons, Keychain,
  `/etc/sudoers.d`, or official evidence artifacts. Use isolated test fixtures/temp roots only.
- Do not dispatch the install retry or official evidence successor.

## Required engineering and review process

1. Bind exact identity and reverify clean exact base/tree/live remote before edits.
2. Load `AGENTS.md`, the hydrated repository-local orchestration skill, R3 MAC/evidence contracts,
   auxiliary receipt/artifacts, and relevant GitNexus guidance.
3. Run GitNexus upstream impact analysis before every existing symbol edit. Treat HIGH/CRITICAL as
   mandatory manual review, but proceed without returning to the user when the edit remains directly
   bounded to these four corrections. Stop on a genuinely unrelated authority domain.
4. Use `gpt-5.6-terra` with Extra High reasoning for every implementation or review subagent.
   Editing ownership must be disjoint where practical. The orchestrator integrates and validates.
5. Add real regressions for default canonical installer invocation/FD behavior, all affected macOS
   cfg/compile surfaces, signed-ticket positive and fail-closed semantics, and explicit validator
   project-ID binding including preserved Linux fixtures.
6. Run focused tests plus the exact macOS compilation/build path that previously produced the 11
   errors. Prove the canonical installer reaches and completes its source build/install logic in an
   isolated non-host-mutating test root or an equivalent repository-owned fixture. Do not claim the
   real host is installed.
7. Run one fresh discovery review, remediate valid P1/P2, then a different fresh closure review.
   At most two supplemental causal cycles are permitted only for directly caused/unmasked P1/P2.
   Record P3/P4 without automatically extending the loop. CLEAN ends the loop.
8. Run full changed-path/symbol/secret/diff checks and `gitnexus_detect_changes()` before commit.
9. Reverify live remote still equals the expected base, create exactly one conventional commit, and
   push by normal fast-forward. Verify live remote equals the landed commit and the checkout is clean.
10. As the last tool action, send the return meta task a structured
    `codex.auxiliary-source-correction-receipt.v1` receipt with task/nonce/base, landed commit/tree,
    exact files, tests, review record, four-blocker dispositions, live remote equality, and successor
    `AUX-R3-MAC-HOST-PREP-INSTALL-RETRY`. A blocked receipt must preserve the worktree and describe
    exact remaining authority/scope needs.

Do not start until the identity-binding follow-up arrives.
