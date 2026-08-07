R2 BOUNDED CONTINUATION AUTHORITY — EXISTING TASK

Continue in your existing task/worktree; do not create or use a fresh task:

- task_thread_id: 019fdcee-73e6-77a2-8a16-1d1c86223f9c
- task_host_id: local
- exact_worktree: /Users/spensermcconnell/.codex/worktrees/bacb/substrate
- original_dispatch_nonce: 7f33f40d8af93e12561b3b1e23fa38bfe3fca28e15c70f53caf6525dc85d68de
- continuation_nonce: 3f7553bcb50a0c0d06373e6a469f8972c9a31a59b06a13eea36a255a66c06760
- expected base/tree: bb098b6e265a4b6cadc4b7ca908c8b6ccf59f2b9 / 688ae2860d99d6d5cf1019e0d3ba27c7cc26b6cc
- expected preserved tracked diff SHA-256: 200f5e0563ce1af63625d0ed33822520d96f9dcadedee6b32a7b843fdfd314f2
- expected untracked compile-test SHA-256: 6f926b83ce7263f0950ecdd98d451a85f80a2a496cb98e31503ce07954255781
- live remote must still equal the expected base

The compile failure is adjudicated as an R2/R4 proof-gate ownership contradiction, not a request to
expand R2. Continue the preserved R2 subject with these exact corrections:

1. R2 proves the named shell/macOS cfg admissions and the installer source shape for exactly
   `substrate-lifecycle-control` and `substrate-lifecycle-macos`.
2. Replace only the compile-surface test invocation that compiles `substrate-lifecycle-macos` with
   native and `x86_64-apple-darwin` checks of the R2-owned shell/ordinary host compile surface plus
   static assertions for the exact installer BUILD_FLAGS and fixed managed-copy branch.
3. Do not edit `src/bin/substrate-lifecycle-macos.rs`, `Cargo.toml`, `Cargo.lock`, AuditToken/API/FFI
   ownership, or any other new path/symbol.
4. Record the existing sha2/AuditToken/pointer-cast failures as a mandatory deferred R4 gate. R4
   cannot land CLEAN until the lifecycle-macos binary compiles after its authorized changes.
5. Keep `npx gitnexus analyze` and tracked context-file injection forbidden. Preserve the prior 05f9
   worktree and the unused 1958 barrier worktree; do not mutate either.
6. Reverify this worktree's exact hashes/base/remote before continuing. Then finish R2 deterministic
   checks, causal review, publication, and receipt under the original dispatch nonce. Use
   `next_increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R3` only on independently CLEAN success.

All other R2 retry contract fences remain unchanged. This continuation authorizes no R3/R4
implementation, native action, new dependency, Windows change, ordinary Linux-host change, or new
source path. Use `gpt-5.6-terra` at Extra High for all subagents/reviewers. On another bounded
same-packet issue, preserve the worktree and return evidence; do not self-widen.
