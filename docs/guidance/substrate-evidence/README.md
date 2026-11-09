# Substrate source and evidence index

This index preserves a screened selection from the 2026-11-09 cleanup audit. It is a review candidate: publication, remote verification, cleanup, task acceptance and phase completion are separate events. The original audit was accepted as a plan and local draft, not as a finding of current safe reclaim. [Inventory index](inventory-index.json) maps all 636 historical rows, 194 examined repository groups and 620 record-coverage groups to local custody references; it deliberately omits private path and session metadata. The full frozen Task01 inventory, groups, source-preservation sets, coverage and original records remain in protected local custody. Opaque IDs are references, not URLs or a claim of off-machine backup. The custody mapping must remain available to an authorized local custodian.

[Record manifest](manifest.json) records original, frozen-copy and publication hashes, original labels and redaction reasons for the 12 selected records. Original results, including failed outcomes, are evidence of what happened then; a future build cannot recreate them. The selected strategic discovery is independent **advice to META**, not META's disposition. The original draft's label is retained in the manifest as history; Task01's META disposition accepted the reviewed plan and local bundle, while leaving publication and cleanup open.

## Canonical source and recipe

The source baseline was `3dda500a630e7ab92186788b9a0641dc84a788b0` on `feat/internal-host-orchestrator-world-dispatch-bootstrap` in [Substrate](https://github.com/atomize-hq/substrate/tree/3dda500a630e7ab92186788b9a0641dc84a788b0). `Cargo.lock` SHA-256: `e997cc8ae39cee20b184895ac6ae0f1300fb1b2bb4e95b68d1b104303a8e3467`; `rust-toolchain.toml` SHA-256: `8aa4ad836dc3e6479a7677e869fd3ad6a9314e5624d76b5e7f666fe7820b30c8`. The selected future recipe, contingent on preserved and available source, lock, unique inputs and native dependencies, is:

```sh
cargo +1.89.0 build --locked --workspace --target x86_64-unknown-linux-gnu
```

This selects the default workspace features and dev profile for a new build. It is neither the historical command for every binary nor a guarantee of successful rebuilding. Toolchain components and target prerequisites must be present; identify and preserve exact immutable native/download inputs for any affected subset before retiring it. The owner must verify the exact preservation commit and selected files by retrieval from GitHub immediately before cleanup, not merely rely on a local branch or previous remote observation.

## Full cleanup plan and holds

All allocations in historical records are point-in-time measurements, not current reclaim. Nested rows are nonadditive; hardlinks, live processes, mount boundaries, unique inputs and protected references can reduce reclaim. The audit's approximately 47 GiB canonical-debug historical ceiling is not a current safe-reclaim or capacity promise. Only independent eligible descendants of canonical `target/debug` are considered in this increment:

1. **B1** compiler incremental descendants, after per-subset source, input, record, reference and live preflights.
2. **B2** other eligible debug descendants excluding B1. Hold generated/unique inputs, active executables, protected references, symlinks, external hardlinks and unknown content; avoid counting any inode twice.
3. **B3** integration checkout debug artifacts: later work only, with the frozen actor and all aliases, its World/Lane workflow, enclosing evidence and full dependency closure protected.
4. **B4** isolated inventory target: retain private original proof and source until selected and preserved.
5. **B5** comparison roots: preserve each base/head source and original results; GitHub custody remains unverified.
6. **B6** other repository and worktree candidates: preserve per-repository exact source, changes, inputs and selected records, check ownership and remote restoration; never infer eligibility from age or directory name.
7. **B7** mixed, loose and temporary roots: classify source, results, private configuration, caches and unrelated repositories before any separate disposition. Preserve shared Cargo state, installed files, backups and rollback dependencies. Runtime/session records and protected workflow trees remain local and are not publication payloads.

Four historical lock hashes and two Git metadata probes were unavailable in the accepted audit; affected source groups remain held until resolved. Historical R2 command, cwd/time or outcome metadata is incomplete where the manifest says so. Other original results and private custody remain unselected for many groups. Missing original binary history alone does not prevent eligible cleanup, but a rebuilt artifact is not an original result. Do not remove a path or ancestor containing a protected dependency. Do not use `cargo clean`, `git clean`, pruning or force worktree removal. Full audit and subsequent batches are future work, not permission for this increment.

## Receipt and recovery

[Receipt template](receipt-template.md) distinguishes expected allocation from observed free-space change, source/record reachability, failed checks and exact held paths. Update status only with actual results after remotely verified preservation and a fresh live preflight. Preserve original observations in custody; restore source only from verified remote source and separately preserved inputs. Do not present a new build as historical proof. The plan has 24 GiB persistent, 2 GiB temporary and 512 MiB proof-storage caps, and requires free-space floors of 16 GiB and 8 GiB on their applicable filesystems. No artifact backup is implied.
