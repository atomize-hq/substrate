R2 CONTINUATION 0017 — REMEDIATE P2 MANAGED-COPY RETRY CONVERGENCE

Continue in the same task/worktree:

- task_thread_id: 019fdcee-73e6-77a2-8a16-1d1c86223f9c
- task_host_id: local
- exact_worktree: /Users/spensermcconnell/.codex/worktrees/bacb/substrate
- original_dispatch_nonce: 7f33f40d8af93e12561b3b1e23fa38bfe3fca28e15c70f53caf6525dc85d68de
- prior_continuation_nonce: 2248c338b9bfc2c0dcb702ea5828ab9674436c1fa3f02f740d6b817c897d779c
- continuation_nonce: ab5f7cbe656daeeec75e2e2e96d3f753f34ce266913c42088e7547e291dba1af
- expected base/tree: bb098b6e265a4b6cadc4b7ca908c8b6ccf59f2b9 / 688ae2860d99d6d5cf1019e0d3ba27c7cc26b6cc
- expected tracked diff SHA-256: c24cd01ade9c201cc7cdfc801c1f3c28b445d58adc63247d8e64b96c44add835
- expected untracked compile-test SHA-256: 0a03fde502464ce62bb2e94cec65bd86d98c8117579f5a3ad758bcad4e03ddc3
- expected review record SHA-256: 2863d377948976092c5f50971bc7e5e5b698ef86634396699294a892c1ab1ab7
- live remote must remain the expected base

Authorize one bounded remediation of valid discovery finding P2-R2-LIFECYCLE-001.

Allowed product/test scope:
- scripts/substrate/dev-install-substrate.sh: stage_managed_mac_control_binary_copy and the existing
  fixed two-binary MAC managed-copy call site only.
- tests/installers/dev_install_bash32_fd_regression.sh: existing fixture cases needed to prove
  injected copy/manifest failure and retry convergence.
- Existing R2 review reports/record only for remediation and closure bookkeeping.

Required result:
1. A copy or manifest failure must not strand an unmanifested destination that a retry rejects as
   unmanaged.
2. Temporary binary/manifest artifacts must be cleaned or remain safely recoverable.
3. Failure on either member of the fixed two-binary pair must converge on retry to exactly both
   immutable copied binaries and exactly both manifest entries.
4. An unmanaged pre-existing destination remains fail-closed and is never overwritten.
5. Preserve the exact two named binaries, copy-not-link behavior, executable permissions, and
   positive MAC-only branch. Do not introduce a general transaction framework or new endpoint.
6. Add deterministic failure-injection coverage inside the existing installer fixture. Do not add
   production test-only authority or environment bypasses when shell command/path fixture
   interception can prove the behavior.

Reverify all bound hashes first, then perform required pre-edit impact/manual caller analysis for
the exact helper/call site. Make no other product or test edit. Rerun the complete MAC-only R2
deterministic suite and static platform proof. Do not run Linux/Windows target commands, mutate the
external toolchain cache, or run npx gitnexus analyze.

Review continuation:
- This is remediation after discovery-1, not a new discovery burst.
- Freeze the new subject and run one fresh different closure reviewer.
- A supplemental causal cycle is allowed only for P1/P2 directly caused or unmasked by this
  remediation, with at most two total supplemental cycles.
- CLEAN is terminal; unresolved P3/P4 are recorded without remediation cycles.

Only after terminal CLEAN, validated record, change detection/manual fallback, exact allowlist,
secret/diff checks, and live remote equality may you create one scoped fast-forward R2 commit and
push it. Return the terminal receipt under the original dispatch nonce with
next_increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R3. Do not dispatch R3 yourself.

All prior MAC-only boundaries remain. Use gpt-5.6-terra at Extra High for every subagent/reviewer.
Preserve the worktree and return an exact blocker rather than widening.
