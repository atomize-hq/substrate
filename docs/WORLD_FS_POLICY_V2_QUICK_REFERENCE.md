# World FS Policy V2 — Quick Reference (Granular Allow/Deny + Strict)

This is a practical “what to set + what to try” guide for the newer `world_fs` policy shape (ADR-0018 / WFGAD*).

## 0) Preconditions (so tests are meaningful)

- The deny/strict behavior is a **Linux** implementation detail:
  - Linux host: world-agent runs on the host.
  - macOS host: world-agent runs inside a **Lima VM** (Linux in VM).
- Verify world backend health before testing:
  - `substrate world doctor`
  - If that fails, run `substrate world enable` (or on macOS: `scripts/mac/lima-warm.sh`).

## 1) Policy shape (what exists now)

At a high level:

- `world_fs` is configured with:
  - `mode`: `read_only` or `writable`
  - `isolation`: `workspace` or `full`
  - `require_world`: `true|false` (denies require `true`)
  - `enforcement`: `strict` or `best_effort` (**only** when any `deny_list` is non-empty)
- Dimensions (optional objects):
  - `world_fs.read`
  - `world_fs.discover` (optional; defaults to `read` semantics when omitted)
  - `world_fs.write`
- Each dimension has:
  - `allow_list`: required, non-empty (when the dimension is present / implied by mode)
  - `deny_list`: optional list of patterns

### Hard rules that commonly bite people

- **Deny lists require full isolation**:
  - If any `deny_list` is non-empty, you must have:
    - `world_fs.isolation=full`
    - `world_fs.require_world=true`
    - `world_fs.enforcement` present (`strict` or `best_effort`)
- `discover` is optional:
  - If `world_fs.discover` is omitted, **discover behaves like read** (no “mystery default”).
- Allow-list patterns are intentionally conservative:
  - `allow_list` rejects wildcards and unsupported metacharacters.
- Deny-list patterns support only `*` and `**` (no `?`, no character classes).
- Legacy keys are **hard errors**:
  - e.g. `world_fs.read_allowlist`, `world_fs.write_allowlist` should fail fast.

## 2) Minimal working examples (CLI)

Create a clean workspace + policy, then apply one of the examples below:

```bash
substrate workspace init --force
substrate policy init --force
```

### A) Read-only, full isolation (allow everything in project)

```bash
substrate policy set \
  'world_fs.mode=read_only' \
  'world_fs.isolation=full' \
  'world_fs.require_world=true' \
  'world_fs.read.allow_list+=.'
```

### B) Read-only with deny masking (best_effort)

```bash
substrate policy set \
  'world_fs.mode=read_only' \
  'world_fs.isolation=full' \
  'world_fs.require_world=true' \
  'world_fs.enforcement=best_effort' \
  'world_fs.read.allow_list+=.' \
  'world_fs.read.deny_list+=./secrets/**'
```

### C) Strict deny lockdown (security boundary)

```bash
substrate policy set \
  'world_fs.mode=read_only' \
  'world_fs.isolation=full' \
  'world_fs.require_world=true' \
  'world_fs.enforcement=strict' \
  'world_fs.read.allow_list+=.' \
  'world_fs.read.deny_list+=./secrets/**'
```

### D) Discover vs read (visible-but-not-readable)

```bash
substrate policy set \
  'world_fs.mode=read_only' \
  'world_fs.isolation=full' \
  'world_fs.require_world=true' \
  'world_fs.enforcement=best_effort' \
  'world_fs.discover.allow_list+=.' \
  'world_fs.read.allow_list+=.' \
  'world_fs.read.deny_list+=./secrets/secret.txt'
```

### E) Writable with write-deny (EROFS)

```bash
substrate policy set \
  'world_fs.mode=writable' \
  'world_fs.isolation=full' \
  'world_fs.require_world=true' \
  'world_fs.enforcement=best_effort' \
  'world_fs.write.allow_list+=.' \
  'world_fs.write.deny_list+=./outputs/private/**'
```

## 3) “Things to try” (behavior checklist)

Use `--world` runs so you’re exercising the backend enforcement path:

### 3.1 Schema / validation (fast fail)

- Legacy keys should error:
  - `substrate policy set 'world_fs.read_allowlist+=.'`
  - `substrate policy set 'world_fs.write_allowlist+=.'`
- Invalid patterns should error:
  - `substrate policy set 'world_fs.read.allow_list+=../x'`
  - `substrate policy set 'world_fs.read.allow_list+=/abs'`
  - `substrate policy set 'world_fs.read.allow_list+=src/**'`
  - `substrate policy set 'world_fs.read.deny_list+=file?.txt'`

### 3.2 Deny overrides allow (directory deny)

Expected: `EACCES` / “Permission denied” for denied paths; allowed paths still work.

```bash
substrate --world --command 'ls ./secrets'
substrate --world --command 'cat ./secrets/secret.txt'
substrate --world --command 'cat ./docs/public.txt'
```

### 3.3 Strict bypass prevention (mount/umount must not undo denies)

Expected in strict:
- `umount`/`mount` attempts fail with `EPERM` / “Operation not permitted”
- denied reads remain denied

```bash
substrate --world --command 'umount /project/secrets || true'
substrate --world --command 'cat ./secrets/secret.txt'
```

### 3.4 Discover vs read (“listable but not readable”)

Expected:
- `ls` shows the file
- `cat` is denied

```bash
substrate --world --command 'ls ./secrets | grep -qx secret.txt'
substrate --world --command 'cat ./secrets/secret.txt'
```

### 3.5 Wildcard deny snapshot semantics (deny matches at exec start)

Expected:
- wildcard deny blocks matching files that exist at exec start

```bash
substrate --world --command 'cat ./certs/a.pem'
```

### 3.6 Write deny returns EROFS (not EACCES)

Expected: “Read-only file system” / `EROFS` when writing under a denied write subtree.

```bash
substrate --world --command 'mkdir -p ./outputs/private/x'
```

### 3.7 Discover deny makes subtree “invisible”

Expected: listing/reading in the denied discover subtree fails (typically “Permission denied”).

```bash
substrate --world --command 'ls ./secrets'
```

## 4) One-command “known good” smoke (recommended)

These run a curated suite of the above behaviors.

### Linux (local)

```bash
cargo build --bin substrate --bin substrate-shim
export PATH="$PWD/target/debug:$PATH"
SUBSTRATE_SMOKE_SLICE_ID=WFGAD5 bash docs/project_management/next/world-fs-granular-allow-deny/smoke/linux-smoke.sh
```

### macOS (local; Lima-backed Linux world)

```bash
scripts/mac/lima-warm.sh "$PWD"
cargo build --bin substrate --bin substrate-shim
export PATH="$PWD/target/debug:$PATH"
SUBSTRATE_SMOKE_SLICE_ID=WFGAD5 bash docs/project_management/next/world-fs-granular-allow-deny/smoke/macos-smoke.sh
```

## 5) CI / runners (Planning Pack Feature Smoke)

Feature smoke is dispatched via `.github/workflows/feature-smoke.yml`.

- Linux (self-hosted runner with `/run/substrate.sock`): `make feature-smoke PLATFORM=linux RUNNER_KIND=self-hosted ...`
- macOS (self-hosted runner): `make feature-smoke PLATFORM=macos RUNNER_KIND=self-hosted ...`
- macOS **github-hosted runners cannot run Lima** (Virtualization.framework is unavailable), so full isolation smoke must run on self-hosted macOS.

Example:

```bash
FEATURE_DIR="docs/project_management/next/world-fs-granular-allow-deny"
CHECKOUT_SHA="$(git rev-parse HEAD)"

make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=linux RUNNER_KIND=self-hosted WORKFLOW_REF="$(git branch --show-current)" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" SMOKE_SLICE_ID=WFGAD5
make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=macos RUNNER_KIND=self-hosted WORKFLOW_REF="$(git branch --show-current)" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" SMOKE_SLICE_ID=WFGAD5
```

