R2 CONTINUATION 0018 — ALIGN STALE MAC STATIC PROOF

Continue in the same task/worktree:

- task_thread_id: 019fdcee-73e6-77a2-8a16-1d1c86223f9c
- task_host_id: local
- exact_worktree: /Users/spensermcconnell/.codex/worktrees/bacb/substrate
- original_dispatch_nonce: 7f33f40d8af93e12561b3b1e23fa38bfe3fca28e15c70f53caf6525dc85d68de
- prior_continuation_nonce: ab5f7cbe656daeeec75e2e2e96d3f753f34ce266913c42088e7547e291dba1af
- continuation_nonce: 247ed8fe255205152bfe548529a6796e6532f91748f7e4cea5764829dfe9ddae
- expected base/tree: bb098b6e265a4b6cadc4b7ca908c8b6ccf59f2b9 / 688ae2860d99d6d5cf1019e0d3ba27c7cc26b6cc
- expected tracked diff SHA-256: 1a202492a960baa181a1012f439f558e61c596676fe17c4290c2e7b0959a0074
- expected untracked compile-test SHA-256: 0a03fde502464ce62bb2e94cec65bd86d98c8117579f5a3ad758bcad4e03ddc3
- expected review record SHA-256: 2863d377948976092c5f50971bc7e5e5b698ef86634396699294a892c1ab1ab7
- live remote must remain the expected base

The stale static assertion is adjudicated as a test-only consequence of the authorized P2 fix.

Authorize exactly one edit surface: the stage_managed_mac_control_binary_copy required-invariant
block in tests/mac/dev_install_compile_surface_r3.sh.

Replace the old generic tmp assertions with strict assertions that:
1. binary_tmp and manifest_tmp are distinct helper temporaries;
2. source copy and chmod target binary_tmp;
3. manifest creation and the destination entry target manifest_tmp;
4. failure cleanup covers both temporary paths where applicable;
5. mv of manifest_tmp to manifest_path occurs before mv of binary_tmp to dest;
6. existing unmanaged-destination fail-closed, exact MAC BUILD_FLAGS, exact two-binary call-site,
   native arm64, and x86_64-apple-darwin assertions remain intact.

Do not merely weaken or delete the helper assertions. No production change, other test change, or
new path is authorized.

Reverify all hashes first, update only that static block, then rerun the entire MAC-only R2
deterministic suite and static platform proof. Do not run Linux/Windows target commands, mutate the
external toolchain cache, or run npx gitnexus analyze.

After gates pass, freeze the new subject and perform the required fresh different closure review
against discovery-1. CLEAN is terminal. Use a supplemental causal cycle only for P1/P2 directly
caused or unmasked by the remediation, with no more than two. Then run review validation,
change-detection/manual fallback, exact allowlist, secret/diff checks, and live-remote verification.

Publish one scoped fast-forward R2 commit only after terminal CLEAN. Return the receipt under the
original dispatch nonce with next_increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R3. Do not dispatch R3.

All MAC-only and preservation boundaries remain. Use gpt-5.6-terra at Extra High for all
subagents/reviewers.
