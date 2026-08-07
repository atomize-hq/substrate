R3 MAC CORRIDOR AMENDMENT 0016 — RESUME EXISTING R2 TASK

Continue in the same task and preserved worktree:

- task_thread_id: 019fdcee-73e6-77a2-8a16-1d1c86223f9c
- task_host_id: local
- exact_worktree: /Users/spensermcconnell/.codex/worktrees/bacb/substrate
- original_dispatch_nonce: 7f33f40d8af93e12561b3b1e23fa38bfe3fca28e15c70f53caf6525dc85d68de
- prior_continuation_nonce: 48c556cf19c9c4ac60b5d8c1ee4adf744184885a66ae7a3c9223e8df7aa96dbe
- continuation_nonce: 2248c338b9bfc2c0dcb702ea5828ab9674436c1fa3f02f740d6b817c897d779c
- expected base/tree: bb098b6e265a4b6cadc4b7ca908c8b6ccf59f2b9 / 688ae2860d99d6d5cf1019e0d3ba27c7cc26b6cc
- expected tracked diff SHA-256: 200f5e0563ce1af63625d0ed33822520d96f9dcadedee6b32a7b843fdfd314f2
- expected untracked test SHA-256: 0a03fde502464ce62bb2e94cec65bd86d98c8117579f5a3ad758bcad4e03ddc3
- live remote must remain the expected base

The user explicitly makes the R3 MAC recovery corridor MAC-only. This amendment supersedes stale
R2-R6 wording that makes Linux or Windows target compilation a blocking gate.

1. Stop all Linux and Windows target work. Do not run another Linux/Windows cross compile, install
   or mutate toolchains, or remediate any of the 20 Windows errors. Leave
   /Users/spensermcconnell/.cache/substrate-r2-cross-toolchains untouched.
2. Linux/Windows target compilation is not a blocking gate for R2-R6. For R2, the already-passing
   Linux result is nonblocking extra evidence. Record the 20 Windows failures in review/receipt
   evidence as observed out-of-corridor baseline debt only.
3. R2 proof is MAC-only: native arm64 macOS and x86_64-apple-darwin checks of the R2-owned surface,
   exact allowlist, byte-identical forbidden Windows/WSL hashes, Linux-body diff proof for the
   shared Linux-named file, and static proof that every new MAC branch uses a positive
   target_os = macos predicate rather than not-linux.
4. One narrow product correction is authorized in
   crates/shell/src/execution/managed_lifecycle/linux_client.rs only: replace the R2-introduced
   cfg(not(target_os = linux)) predicates for socket_type, the post-socket fcntl calls, and
   set_close_on_exec_v1 with cfg(target_os = macos). This is MAC containment, not Windows work.
   No other product or test edit is authorized.
5. Reverify identity/base/tree/remote and exact preserved hashes, make only that correction, rerun
   all MAC-only R2 deterministic checks, then complete the full repository-defined causal
   cascading review. P1/P2 remediation stays inside R2; P3/P4 are recorded without cycles.
6. Do not run npx gitnexus analyze. Use the already-recorded degraded result plus complete manual
   exact-symbol/cfg/caller analysis. Do not begin R3/R4, installation, native evidence, or any
   platform work outside MAC.
7. Publish one scoped fast-forward R2 commit only after terminal CLEAN review and all MAC-only gates.
   Return the receipt under the original dispatch nonce with
   next_increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R3. Do not dispatch the successor yourself.

Use gpt-5.6-terra at Extra High for all subagents and reviewers. Preserve every task/worktree and
return an exact blocker rather than widening if any new issue cannot be resolved inside R2 MAC-only
authority.
