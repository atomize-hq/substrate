# Three retained source snapshots

The source package was published and independently retrieved at `041e8951cd075e7d3efb3f55e406e79085f498d4`. The source manifest lists exact HEADs, file SHA256 hashes and modes, original and proposed patch hashes, and the one untracked source file. All three HEADs were fetched as ancestors of `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap` from `git@github.com:atomize-hq/substrate.git`. No assertion is made about other remote refs. No changes are staged. Each original source checkout remains retained.

For a future restoration, restore each in a separate new repository with its own Git administration. Fetch the containing branch, confirm the specified commit is an ancestor of the fetched ref, then check out the commit detached:

```sh
git init restored-source
cd restored-source
git fetch --no-tags git@github.com:atomize-hq/substrate.git refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
git merge-base --is-ancestor COMMIT FETCH_HEAD
git checkout --detach COMMIT
git apply /path/to/task03-LABEL.patch
```

Use `COMMIT=33ebdf0d6a277f45dbc5603063bbcdbe521252ce` and `LABEL=t13` or `t11`; use `COMMIT=29cf8e8223c9d166263e56de219428d2b3858aa9` and `LABEL=t10`. Substitute those values in the commands. For T11, additionally run `install -D -m 0644 /path/to/task03-t11-native-launch-context.rs crates/shell/src/execution/agent_runtime/native_launch_context.rs`. Verify every path's bytes and filesystem mode against the manifest, tracked modifications and the T11 untracked path against its stated status, and that `git diff --cached --quiet` succeeds. Actual GitHub payload retrieval and three serial isolated physical restorations matched all 21 changed or untracked source files, full modes, regenerated patch bytes, source status and empty stage. The untracked T11 payload was explicitly installed with its documented 0644 mode; other changed modes came from native checkout/apply under umask 022.

For a future Linux rebuild, install Rust 1.89.0 and the platform prerequisites described by the restored `docs/INSTALLATION.md`, then run `cargo +1.89.0 build --workspace --locked --target x86_64-unknown-linux-gnu` in the restored source root. The manifest identifies the lockfile and toolchain bytes. No build is claimed here; historical compiler/native dependency identity is not fully known, and original native outputs are not reproduced by assertion. A retired GitNexus database and parse cache must be rebuilt before use, even if local metadata or registry pointers remain.

The 38 exact generated index/cache leaves were removed after fresh guards. All three source roots, targets/private inputs, historical originals, registries and lightweight index metadata remain. Retained index pointers are stale; rebuild before querying. The [four-subject receipt](cleanup-receipt-task03-remaining-four.json) records the exact relative leaf membership and actual allocation.
