# repo-037 committed source recovery

The source states below were identified in a local canonical-swap experiment. The recovery base is Substrate commit `b2dae23a8c8700afe6e261bf8c833b312b44efb0`, tree `f99ac84afb40e82d025f5e7c1de18b7803ad4e93`. On 2026-11-09 that commit was returned by the origin GitHub commit API and was locally verified as an ancestor of the then-advertised `feat/internal-host-orchestrator-world-dispatch-bootstrap` head `dc06858aa82c649f3d63d0bc86ff9ccbca901806`. Ref availability can change; verify the base and path contents when recovering.

Apply `01-base-to-parent.patch` to the base to produce tree `477932fa466bce2ccd48d6345bc9bc2adab0a158`, corresponding to commit `4cfcabf9d00e20c26bae71618563635e1e0afb61`. From that tree, apply one of the following independent patches:

| Patch | Commit | Expected tree |
|---|---|---|
| `02-parent-to-head.patch` | `733c5799a8ebf097761f960b106a47cb7604f19c` | `839c6a3269c917dfd535b349f721d4ac2cbbfa80` |
| `parent-to-e17d356312746f3737ec1844ac28fb70ef32d06b.patch` | `e17d356312746f3737ec1844ac28fb70ef32d06b` | `7221bac7791372ebe3e704a87c13bebf0c33d216` |
| `parent-to-870919df9227b0494e005adbe72c82657b76457c.patch` | `870919df9227b0494e005adbe72c82657b76457c` | `8a13355200517ecd2ed981c62707c819d48562c4` |
| `parent-to-5c25a8647d0c025f632082e553531ad551bbb374.patch` | `5c25a8647d0c025f632082e553531ad551bbb374` | `990c53f0f0f9877e7aa278b3a43012c6f7b379e2` |
| `parent-to-bc888e40b587511a6c4beae563cbfc8c7ec10b7c.patch` | `bc888e40b587511a6c4beae563cbfc8c7ec10b7c` | `571424fda1beb71ad38c84177b10d061e59b2a96` |
| `parent-to-e1f4954d6e5b6a13117440f19a41ff179ded32cd.patch` | `e1f4954d6e5b6a13117440f19a41ff179ded32cd` | `3f377d361d58337dc121ba3f44531ccdc72afca7` |

These are source patches; exact commit identity additionally requires the corresponding original commit body, including author, committer, parent and message metadata, from the separately controlled private recovery map. The commit bodies are ordinary Git metadata whose preservation is necessary for exact commit IDs; their placement in private local custody avoids publishing personal addresses and local record details without review, and does not establish an off-host backup. Apply patches with `git apply --cached` in a disposable repository and verify each tree with `git write-tree`; use the exact body with `git hash-object -t commit -w --stdin` and verify its commit ID. Reinitialize the index to the parent tree before each independent tail. Do not infer byte-identical historical binaries or successful historical tests from this source recovery.

`SHA256SUMS` binds this proposed package. Publication and independent retrieval of the exact paths, modes, and bytes on the authorized feature branch remain required before corresponding cleanup. Private mapping and original historical records must remain in verified custody outside the selected root.
