R2 CONTINUATION 0019 — REMEDIATE CAUSALLY UNMASKED MANIFEST READ FAILURE

Continue in the same task/worktree:

- task_thread_id: 019fdcee-73e6-77a2-8a16-1d1c86223f9c
- task_host_id: local
- exact_worktree: /Users/spensermcconnell/.codex/worktrees/bacb/substrate
- original_dispatch_nonce: 7f33f40d8af93e12561b3b1e23fa38bfe3fca28e15c70f53caf6525dc85d68de
- prior_continuation_nonce: 247ed8fe255205152bfe548529a6796e6532f91748f7e4cea5764829dfe9ddae
- continuation_nonce: 353fae0cab2c3d5475c33214236731e3547e44a4a1f1b7f8420aba8a85272056
- expected base/tree: bb098b6e265a4b6cadc4b7ca908c8b6ccf59f2b9 / 688ae2860d99d6d5cf1019e0d3ba27c7cc26b6cc
- expected tracked diff SHA-256: 1a202492a960baa181a1012f439f558e61c596676fe17c4290c2e7b0959a0074
- expected untracked compile-test SHA-256: 62c47ff28d3bfbb0b27ee204258dc3d52340e59a427118ab21abba18c921e170
- expected review record SHA-256: 74fc39e47b461c901e67dd7a8f9521faf148615308130d770ada9ced1b2bfcf0
- live remote must remain the expected base

P2-R2-LIFECYCLE-002 is accepted as directly unmasked by the immediately preceding
P2-R2-LIFECYCLE-001 remediation. Authorize this exact MAC-only correction:

1. In stage_managed_mac_control_binary_copy only, capture grep -Fxv exit status. Status 0 and normal
   no-match status 1 may continue. Status greater than 1 must clean binary_tmp and manifest_tmp and
   return failure before either manifest or destination publication. Do not suppress real read,
   permission, or I/O errors.
2. In tests/installers/dev_install_bash32_fd_regression.sh only, inject an actual read failure for
   an existing MAC control manifest. Prove the prior binary pair and manifest remain intact, no
   temporary artifact survives, and a later retry converges to exactly both binaries and entries.
3. Add no path, general transaction framework, production test-only bypass, or unrelated behavior.

Reverify the bound hashes, run exact impact/manual caller analysis, make only those edits, and rerun
the complete MAC-only R2 deterministic suite and static platform proof. Do not run Linux/Windows
target commands, mutate the external toolchain cache, or run npx gitnexus analyze.

Then freeze the changed subject and run supplemental-causal-1:
- kind: supplemental_causal
- trigger cycle: closure-1
- trigger finding: P2-R2-LIFECYCLE-002
- include concrete causal evidence connecting it to the immediately preceding remediation
- use a fresh independent read-only reviewer
- CLEAN is terminal
One supplemental cycle remains only if another P1/P2 is directly caused or unmasked by this
remediation under unchanged authority/risk. P3/P4 are recorded without remediation cycles.

After terminal CLEAN, validate the record, run change-detection/manual fallback, allowlist,
secret/diff, and live-remote gates, then publish one scoped fast-forward R2 commit. Return the
receipt under the original dispatch nonce with next_increment:
AUX-R3-MAC-EVIDENCE-RECOVERY-R3. Do not dispatch R3.

All prior MAC-only and preservation boundaries remain. Use gpt-5.6-terra at Extra High for every
subagent and reviewer.
