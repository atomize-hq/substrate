# Plan 18 source recovery — proposed

The original checkout is retained. Its source HEAD is [`c7061de60c62f65bfb54771107fdaa19022937a3`](https://github.com/atomize-hq/substrate/tree/c7061de60c62f65bfb54771107fdaa19022937a3), contained in `refs/heads/feat/preserve-task03-ba242549bb10` when last independently fetched. The accompanying `task03-plan18-source-manifest.json` records its tree, three changed files, full filesystem modes and hashes, empty stage, lockfile and patch identities. The zero-context patch avoids unrelated private path text from the original diff context while retaining the exact changed source.

After publication and remote byte verification, create a separate Git repository and display its common directory:

```sh
git init restored-source
cd restored-source
git rev-parse --path-format=absolute --git-common-dir
```

Only if that output resolves to this new repository's `.git` directory, run the following commands. The isolated `-text` override avoids an unrelated pristine-checkout line-ending status mismatch observed in the local rehearsal. Do not write to a shared Git administration directory.

```sh
set -euo pipefail
printf '%s\n' 'docs/project_management/_archived/logs/windows_always_world.md -text' > .git/info/attributes
git fetch --no-tags git@github.com:atomize-hq/substrate.git refs/heads/feat/preserve-task03-ba242549bb10
git merge-base --is-ancestor c7061de60c62f65bfb54771107fdaa19022937a3 FETCH_HEAD
git checkout --detach c7061de60c62f65bfb54771107fdaa19022937a3
gzip -cd /path/to/task03-plan18-zero.patch.gz | git apply --unidiff-zero
git diff --cached --quiet
```

Compare the three restored file hashes and modes and full source status against the manifest. The local isolated rehearsal matched those files, status, empty stage and the regenerated original full patch; an actual published-payload fetch and restoration remain a prerequisite to cleanup.

For a future Linux rebuild, install the platform prerequisites described by the restored `docs/INSTALLATION.md`, then run `cargo +1.89.0 build --workspace --locked --target x86_64-unknown-linux-gnu`. Rust 1.89 is the source MSRV; this HEAD has no `rust-toolchain.toml`. Historical compiler/native dependency identity and the build command that produced the selected metadata are unknown. No build is claimed here.
