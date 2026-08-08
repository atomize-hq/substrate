R6 CORRECTION-01 CONTINUATION AUTHORITY

Resume the same task and preserved worktree only:

- thread: `019fded7-05c4-7c61-9f7d-5fc456bcda6d`
- host: `local`
- worktree: `/Users/spensermcconnell/.codex/worktrees/1bd8/substrate`
- original dispatch nonce: `1b9fe0a5eee31553d83daa75ccdde591ea5a4b797a81f14c3f3d3d839bc9cd70`
- continuation nonce: `01b6223453c9a0466c6e2f5f171989bc1f79373557a2902be57344460578b477`
- packet: `AUX-R3-MAC-EVIDENCE-RECOVERY-R6-CORRECTION-01`
- expected base/tree: `a9626bc10a1737416c1d8f7f48630850118f3e6a` / `3f7cdf20125a847a3dfaa472d3797759b5cf6737`
- target: `origin refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- preserved subject: `sha256:018791e14303b553d10286933aba6f911b41b3ad3247cf7bb3aa76092ff51462`
- original R6 review record: `sha256:12f9659045852167a8d0a6f89471f88b20237a691474c881f110cdde3d1f4485`

Load and verify read-only:

`/Users/spensermcconnell/.codex/worktrees/eb49/substrate/orchestration/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/authority-amendments/0036-r6-direct-tty-proof-retry-correction.json`

Its SHA-256 is supplied by the meta delivery record. Treat it as the complete replacement authority
for the ten open R6 findings. Preserve the current dirty worktree; do not reset, clean, restore,
rebase, archive, or donor-copy it.

## Required correction

Replace the rejected operator-terminal relay with the authorized direct-terminal design:

1. Extend the admitted ticket-issue response with the signed, fixed
   `GuestPublisherPairingOperatorLaunchV1` capability. It binds exact PM/binding/session/generation/
   expiry and measured limactl/guest-executor identity. It contains no ticket, transcript,
   confirmation, selector, arbitrary command, or path.
2. The control process verifies that capability, remeasures limactl, and spawns the exact fixed
   `limactl shell --tty=true ... guest-pairing-operator-tty-session-v1` command with direct duplicated
   `/dev/tty` stdin/stdout/stderr. Remove every XPC operator-terminal FD and program byte relay.
3. Permit only one signed canonical binding envelope as fixed operator argv. It carries no
   confirmation input and cannot select paths/commands/transports.
4. In the guest, read all confirmation values only from the controlling `/dev/tty` with echo
   suppression and guaranteed restoration. Persist exactly one fixed-path, no-follow, atomic
   `GuestPublisherPairingOperatorProofV1` containing only the bound non-reusable commitment and
   terminal/session metadata.
5. Start the separate typed data session after operator success. It must validate the independent
   proof against the ticket and full shared binding before intent publication. It may recompute the
   expected commitment only for verification; ticket/data values alone never establish authority.
6. Correct immutable-initial versus mutable-current generation, pre-intent fresh-attempt behavior,
   exact consumed-marker idempotence, and the uniform `now >= expires` no-mutation expiry boundary.
7. Remove broad public terminal APIs and wholesale guest output projection.

All ten findings in the original R6 record are mandatory remediation inputs. Do not reinterpret or
drop them.

## Scope and review

Only the existing nine R6 product/test paths may change. Minimum directly reachable private helpers
and the two exact internal common contract types named by amendment 0036 are authorized. The four
original R6 review artifacts are immutable lineage inputs. Write the correction review only to the
four `r3-mac-evidence-recovery-r6-correction-01-review-*` paths named in amendment 0036.

Run GitNexus impact where the current index can answer, plus complete manual caller/callee,
decoder/XPC, descriptor/no-follow, termios/Drop, proof, CAS/generation, replay/retry, expiry, and cfg
analysis before editing. Do not run `npx gitnexus analyze`.

Use TDD. The proof must include executable negatives for piped/automated confirmation,
data-derived authority, missing/mutated/replayed proof, non-controlling TTY, echo restoration,
initial/current generation, pre-intent loss, consume retry, equality expiry, expired no mutation,
third session, and selectors. Static assertions alone are insufficient for state-machine behavior.

Use `gpt-5.6-terra` at Extra High for every subagent and reviewer. After deterministic proof, run a
fresh correction discovery or same-subject three-lens burst and one different fresh closure review,
with at most two causally justified supplemental cycles. Re-run the combined R1-R6 audit. Validate
the correction review record after every cycle. P1/P2 remediate; P3/P4 record only; CLEAN ends.

If terminal CLEAN and all gates pass, publish exactly one normal fast-forward R6 commit and return
`LANDED_CLEAN` with `next_increment: EVIDENCE:R3-MAC-IMP-01`. Do not dispatch or execute evidence.
On any blocker, preserve this same worktree and return the exact non-advancing receipt. Sending the
receipt to meta must be your final tool action.
