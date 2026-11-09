# Six canonical-swap source checkout recovery

The common source base is commit `b2dae23a8c8700afe6e261bf8c833b312b44efb0`, tree `f99ac84afb40e82d025f5e7c1de18b7803ad4e93`. This commit was retrieved from GitHub as part of the prior source-cohort publication; the GitHub commit API again returned that commit and tree at preparation. The previously published `records/task04-four-repos/public/base-to-c6712d99.patch` supplies the first patch of the second chain; its SHA-256 is `cac32f3f6911b788e608d265339787e8a18fbc3525c76735ba4ae6d2be7ea422`. All other listed patches are in this proposed package. Verify their actual publication, current branch, path, mode and hash before recovery.

Apply each chain in order to the base. Reset the index to the base before starting the second chain. Exact expected commit bodies are held separately; the patch-derived tree, together with its original raw commit body and parent, reconstructs the original commit ID.

| Chain | Patch | Commit | Expected tree | Checkout HEAD |
|---|---|---|---|---|
| A | `to-d91fe0f1.patch` | `d91fe0f12ff01d8ef542d202e367665dd8758af2` | `ae9bbb6d26507ca698c25bc84c3a76e6cc50aeb8` | — |
| A | `to-235ef4cb.patch` | `235ef4cb6f6a63c24d705ad4937dc82722a6f11b` | `aca64fa11b236a2d182e47ee451c608cc060027a` | repo-016 |
| A | `to-968e47cb.patch` | `968e47cb05cc406770cdd62e51b552f8671a9bbb` | `5695b882884935e4ad86b4b4e755f803cc414d0a` | repo-036 |
| A | `to-c2c4038d.patch` | `c2c4038d5b5e0a7230c47a07ed95a7d22e167c84` | `e9fd8741c6d46deb079a5aea448d04068d45a5d4` | repo-017 |
| B | prior published `base-to-c6712d99.patch` | `c6712d99de7e564cbe295493c80f9c3d71602d38` | `b08a918916db1d50a97f2fff9779fc3164de3698` | — |
| B | `to-44c82373.patch` | `44c823739427faa471772f7cfeef3c98209c439c` | `1ff4178dc25919ebd0a283b4403735fd8a7e05cf` | repo-024 |
| B | `to-82cace2a.patch` | `82cace2a4534f89f12917ff882fea71a892ac99e` | `02a7c6ece7d87e7e8f4c5bb78910e0a6e12ad872` | repo-025 |
| B | `to-8c46135c.patch` | `8c46135c861a468dea316cf9fd7d6c6bb15bddac` | `5358497a8baec6f36e15aaef58415a759e64977d` | repo-023 |

For each state, use `git apply --cached --check`, then `git apply --cached`; verify `git write-tree` against the table. Recreate an exact commit object from its original body using `git hash-object -t commit -w --stdin` in a disposable repository and verify the ID. `public/SHA256SUMS` is relative to the parent of `public/`; preserve the layout.

Seven raw commit bodies and six original Git-record archives are in local private custody at `/home/spenser/__Active_code/.worktrees/substrate-runtime-refactor-product-authority-integration/.codex/orchestration/substrate-evidence-consolidation-cleanup-20261109/evidence/task04/six-repos-candidate/private/SHA256SUMS`, SHA-256 `d3fdc3465dcb8503129756318ce4e13e17977d14f18434028b8f4db39dbcff50`. The existing c671 raw commit body is `/home/spenser/__Active_code/.worktrees/substrate-runtime-refactor-product-authority-integration/.codex/orchestration/substrate-evidence-consolidation-cleanup-20261109/evidence/task04/four-repos-candidate/private/c6712d99.commit`, SHA-256 `2b5ebd5787a6f3a8aaf791f52a6362de5bcd11e0cb053e0ff44630893d2629a0`. These records include original refs, reflogs and commit identity; private local custody is not an off-host backup.

Historical local origins form chains repo-017 → repo-036 → repo-016 and repo-023 → repo-025 → repo-024. repo-024's original local origin pointed to a previously removed checkout. If these origins are later needed for fetch, restore the referenced source and private metadata or remap the origin. This is a recovery caveat, not a claim that a live consumer exists. No original binary identity, historic test reproduction or product acceptance is implied.
