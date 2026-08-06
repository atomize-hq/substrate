# A1.1d-5R3-MAC attempt-3 deterministic replay plan

1. Verify the source transcript SHA-256 and read-only exact base commit/tree.
2. Create a non-Git scratch directory from `git archive 270f6e55e1a94b7e2f9b2667e605980d2e50579c`; do not create a worktree.
3. Replay the ordered product mutation calls recorded in `mutation-events.jsonl` using their transcript `call_id` and input payload. Keep proof/test/native commands disabled; run only deterministic byte transformations and Rust formatting needed by those transformations.
4. Materialize all 19 final subject paths under `recovered-files/`, preserving the listed modes.
5. For `scripts/mac/lima-stop.sh` and `scripts/mac/lima-warm.sh`, require the historical final subject manifest blob OIDs, materialize those existing objects read-only, and verify their OIDs. This resolves the missing-worktree terminal-byte ambiguity without inventing content.
6. Reproduce the historical fingerprint from the transcript algorithm: sort `/tmp/a1-r3-mac-subject-paths`; prepend `expected_base=<commit>`; append `<path>\t<base mode or NEW>\t<git hash-object>`; include the unchanged installer fixture and exclude review-control metadata. The result must be `sha256:e60963026aa39be67e334119ff7ae61f3f15f19dbfd31539d790f8735b493924`.
7. Independently SHA-256 the final review cycle record and require `bc7258fb432375e50af899f14691c7691980d4f99a8174b8e2d6ee2e1a29e003`.
8. Verify exactly 19 recovered relative paths and recompute `bundle-digests.json` tree rows. Do not stage, commit, publish, run native evidence, or change any orchestration receipt/state.
