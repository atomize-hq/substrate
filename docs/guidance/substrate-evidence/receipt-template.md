# Cleanup receipt template

Template only. No cleanup or remote publication is asserted. Scrub private paths and metadata before publishing a completed receipt; keep raw receipts and custody mapping locally.

- Batch/subset and UTC time: [pending]
- Selected exact paths and held exact paths with reasons: [pending; use opaque references in published receipt]
- Source and record preservation ref, commit, direct remote retrieval command, individual expected/actual hashes and result: [pending]
- Live protection/reference closure, symlinks, mounts, unique inputs, cwd/fd/exe/maps, inode/link counts and errors: [pending]
- Expected allocated bytes by unique `(device,inode)` only if all links selected; separately list non-reclaimable hardlinks, sparse files and directory blocks: [pending]
- Command, cwd, time, exit status and protected hash comparison, raw local output custody reference: [pending]
- Before/after available bytes, free inodes and applicable 16/8 GiB floors: [pending]
- Actual observed free-space change, inodes removed, expected versus actual and concurrent-allocation caveat: [pending]
- Changed or missing paths, failed/unrun checks, remaining holds and recovery instructions: [pending]
- Post-cleanup remote source and record hashes, retained protection hashes, C/W status: [pending]

Reclaimed allocated bytes and filesystem availability are different measures. Never add nested measurements or count an inode twice. A negative free-space delta or a discrepancy does not establish cleanup failure by itself; retain raw observations and investigate before continuing. A receipt is evidence of an action, not a replacement for source or original test results.
