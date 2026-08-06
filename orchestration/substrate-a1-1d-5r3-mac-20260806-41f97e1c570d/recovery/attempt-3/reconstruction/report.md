# A1.1d-5R3-MAC recovery reconstruction report

## Verdict

`RESTORED_EXACT`

- Restored path: `/Users/spensermcconnell/.codex/worktrees/ace6/substrate`
- Detached HEAD/tree: `270f6e55e1a94b7e2f9b2667e605980d2e50579c` / `acef6844c15d17aba8cfd18d2fc7ef88d9c3420b`
- Status inventory: 11 modified tracked, 8 untracked, 0 staged.
- Recovered-tree SHA-256: `a023bc6e2f4929277468f5f418e954b0d7b96671d2d8c24f8790a9674763ff88`
- Historical subject fingerprint: `sha256:e60963026aa39be67e334119ff7ae61f3f15f19dbfd31539d790f8735b493924`
- Review-cycle-record SHA-256: `bc7258fb432375e50af899f14691c7691980d4f99a8174b8e2d6ee2e1a29e003`

## Preconditions and containment

The independently audited report rehashed to `99869c925b5caeebb9c8fa0f960d621771d8e533b04a7add4b06b6501d83b7c0` and remained `VERIFIED_COMPLETE`; the recovery manifest rehashed to `9edb61f5aab00299706efe3e73050886d81d80d1eedfdbf15c7d8662d595a908`. Before reconstruction, the target path was absent and not registered as a linked worktree. The protected checkout was HEAD/tree `270f6e55e1a94b7e2f9b2667e605980d2e50579c` / `acef6844c15d17aba8cfd18d2fc7ef88d9c3420b` with an empty index, working tree, and untracked inventory; the live `origin/feat/internal-host-orchestrator-world-dispatch-bootstrap` ref was `270f6e55e1a94b7e2f9b2667e605980d2e50579c`.

After reconstruction, the target remains a detached linked worktree with an unchanged base index and exactly the expected 19 porcelain records. All 19 recovered bytes, modes, SHA-256 values, and raw Git blob OIDs matched the manifest. The exact 16-path historical fingerprint includes the unchanged installer-parity fixture and matches the required final subject. The review record is `bounded_stop` / `budget_exhausted`, has two (not three) supplemental causal cycles, and retains the terminal findings. The live remote and protected checkout remained at the exact base and the protected checkout stayed clean.

No implementation test, review, native action, GitNexus operation, staging, commit, push, publication, protected-checkout edit, or successor dispatch was performed.

## Check results

- Total checks: 155
- Passed: 155
- Failed: 0

Revision 2 revalidated the protected `git diff --quiet` and `git diff --cached --quiet` **exit codes** as zero. The first evidence serialization had recorded their blank stdout as the actual value, yielding two false failures; this was an evidence-recording defect only and no target or protected working-tree mutation occurred while correcting the record.

Detailed observations and every check result are in [`receipt.json`](receipt.json). Digest bindings are in [`digests.json`](digests.json).
