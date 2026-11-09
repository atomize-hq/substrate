# Canonical-swap source recovery for repo-009, 010, 021 and 022

The shared source base is commit `b2dae23a8c8700afe6e261bf8c833b312b44efb0`, tree `f99ac84afb40e82d025f5e7c1de18b7803ad4e93`. On 2026-11-09 the GitHub commit API returned that commit and tree; locally the base was verified as an ancestor of the then-advertised feature-branch tip `d656bcdb207a2497e346fb58150b6cb57ab99e64`. Confirm current ref and retrieve exact base objects at recovery time.

Apply one patch to the base in a disposable Git index:

| Repositories | Source patch | Commit | Expected tree |
|---|---|---|---|
| repo-009, repo-010 | `base-to-e7b45801.patch` | `e7b45801d942452539b2f42d717ec3528eb2bdcc` | `1837e0c7346a2cd985349e6512b0b6567832526d` |
| repo-021, repo-022 | `base-to-c6712d99.patch` | `c6712d99de7e564cbe295493c80f9c3d71602d38` | `b08a918916db1d50a97f2fff9779fc3164de3698` |

Each commit is a direct child of the base. The patches are exact source changes; exact commit identity also requires the original raw Git commit body and original per-repository ref and record map held in separate local private custody. Git author and committer metadata is needed to reconstruct exact IDs; this custody is not an off-host backup. Use `git apply --cached --check`, then `git apply --cached`, and verify `git write-tree`; create the exact commit using `git hash-object -t commit -w --stdin` from its original body, then verify its ID. Reset the index to the base before the second patch. repo-009 originally had detached HEAD; the other three used the local feature branch. No original binary equivalence or historical test outcome is asserted.

These files are a proposed package; authorized publication and independent retrieval of exact branch, commit, paths, modes and hashes remain required before affected cleanup. Verify private custody separately.
