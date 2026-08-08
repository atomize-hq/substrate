# AUX-R3-MAC-LIMACTL-HOME-PROVENANCE-CORRECTION contract

## Identity and source

- Orchestration: `substrate-a1-1d-5r3-mac-20260806-41f97e1c570d`
- Packet: `AUX-R3-MAC-LIMACTL-HOME-PROVENANCE-CORRECTION`
- Dispatch nonce: `f2776d7aa9dd669dd9ff2a885ee50536ac9c01da991257d806850cb8be82a19b`
- Existing task: `019fdfa4-ba96-70e0-b68d-1d0a4cf32a83` on `local`
- Exact worktree: `/Users/spensermcconnell/.codex/worktrees/2f78/substrate`
- Expected base: `1126b907df6e38043e9071da222f6c6a377b341c` / `3a29fa323a8b9099d894c11d98cc085ecf4807c5`
- Target: `origin refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- Authority amendment: `authority-amendments/0042-r3-mac-limactl-home-provenance-source-correction.json`
- Amendment SHA-256: `sha256:815ed0977d5affb77560fb63de0e5474e2f0f7e2db7aeeefd3ca34bb9d164fbe`

Rebind the exact task/worktree/base/tree/live remote/nonce before editing. The worktree must be clean,
zero-staged, and exact. Stop on drift. Never mutate the protected checkout.

## Required skill workflow

Load `.agents/skills/using-agent-skills/SKILL.md` first, then apply the repository-local
`orchestrate-top-level-tasks`, `source-driven-development`, `test-driven-development`,
`incremental-implementation`, `debugging-and-error-recovery`, `security-and-hardening`,
`code-review-and-quality`, and `git-workflow-and-versioning` skills. Read the runtime-refactor
control pack and review-control README before implementation. Use gpt-5.6-terra at Extra High for
all independent reviews.

## Proven blocker

Official evidence attempt 8 validated and restored exactly. The unmodified canonical installer
reaches `publish_mac_publisher_install_provenance_v1`, then its privileged scrubbed subprocess runs:

`env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin "$limactl_path" --version`

Fixed `/usr/local/bin/limactl` 2.1.1 requires HOME even for `--version` and panics when it is absent.
The fixed image was never substituted. No Stage-1, PM, pairing, lifecycle, or retirement ran.

## Objective and exact boundary

Correct only the privileged provenance-version environment. Retain `env -i`, the fixed absolute
no-follow `limactl_path`, the fixed privileged PATH, all image/identity/CDHash/requirement joins, and
fail-closed behavior. Supply a defined root-controlled non-user HOME only for this probe. Do not
forward caller HOME or ambient configuration. `/var/empty` is the preferred candidate after proving
its root-controlled non-writable identity and the exact Lima version command works with it.

Authorized product/test paths:
- `scripts/substrate/dev-install-substrate.sh`
- `tests/installers/dev_install_bash32_fd_regression.sh`
- `tests/mac/lifecycle_r3.sh`
- `tests/mac/dev_install_limactl_provenance_home_r3.sh`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-limactl-home-provenance-correction-review-authority-security.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-limactl-home-provenance-correction-review-lifecycle-convergence.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-limactl-home-provenance-correction-review-allowlist-evidence.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-limactl-home-provenance-correction-review-cycle-record.json`
- `llm-last-mile/runtime-refactor/06-review-finding-inventory.md`

The new focused test path is optional. The finding inventory may change only for a valid accepted
P3/P4. No other path is authorized.

## TDD and proof

1. Run GitNexus impact for `publish_mac_publisher_install_provenance_v1`; if unavailable/degraded,
   record exact UNKNOWN state and perform manual caller/callee/platform analysis before editing.
2. Reproduce read-only: missing HOME fails with the fixed Lima image; the proposed root-controlled
   HOME succeeds. This is not official evidence and must not mutate native state.
3. Add a deterministic RED regression on the exact base. The regression must prove HOME is defined,
   caller/user HOME is not inherited, the environment remains scrubbed, the absolute fixed Lima
   image is retained, and a failure remains fail-closed.
4. Implement the minimum source change. Do not broaden the privileged environment.
5. Run the focused regression plus `/bin/bash tests/installers/dev_install_bash32_fd_regression.sh`,
   `bash tests/mac/lifecycle_r3.sh`, `/bin/bash -n scripts/substrate/dev-install-substrate.sh`,
   `cargo fmt --all -- --check`, and `git diff --check`.
6. Prove exact path/symbol fence, MAC-only branch containment, no Windows/WSL/ordinary-Linux changes,
   no credential material, and no native mutation.

## Causal cascading review

Freeze the product/test subject fingerprint. Run a fresh three-lens discovery review
(authority/security, lifecycle/convergence, allowlist/evidence). Remediate every valid P1/P2 within
this exact fence. Freeze each changed subject and run a different fresh closure reviewer. A finding
directly unmasked by a remediation may use at most two supplemental causal cycles. Do not reset,
clean, or discard work. Validate the final cycle record with
`llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py`. Publication requires
terminal CLEAN and no open P1/P2. Record valid P3/P4 only under the conditional inventory rule.

## Publication and terminal receipt

Immediately before publication run GitNexus detect_changes; use exact manual fallback if the
current worktree index is unavailable. Require live remote still equals the expected base. Publish
one scoped conventional commit as a normal fast-forward to the target ref. Verify remote equality,
ancestry, 0/0 divergence, and clean worktree/index.

Return one `codex.top-level-task-receipt.v1` with exact base/landed commit/tree, changed paths,
subject fingerprint, review record/digest/terminal verdict, checks, change detection, cleanliness,
and `next_increment: EVIDENCE:R3-MAC-IMP-01`. For a legitimate blocker return the exact blocked
status and preserve the worktree. Do not dispatch evidence or any successor. Send the receipt to
meta as the final tool action.
