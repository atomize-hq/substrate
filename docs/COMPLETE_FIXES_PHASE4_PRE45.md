Title: Phase 4 + PRE 4.5 Complete Fixes Plan (Linux)

Notes / Current State (live)
- Implemented (Phase A):
  - Replay execution parity using bash -lc '<raw-cmd>' with correct cwd and minimal env reinjection.
  - --replay-verbose flag with clear headers (span_id, cmd, cwd, mode) and capability warnings when isolation is requested but unavailable.
  - Unit test confirming redirection works under replay’s direct path.
- Implemented (Phase B – first cut):
  - Overlay-backed replay path on Linux: per‑replay overlay root at /var/lib/substrate/overlay/<span_id>, bind lower, mount overlay, execute under merged, compute fs_diff from upper, teardown.
  - Fallback to direct execution (no fs_diff) with verbose warning if overlay fails.
- Observed on Podman VM container: replay succeeds but fs_diff empty for a simple write span. We hardened overlay execution (bind lower + execute inside merged) but still observed empty diff on that VM/container.
- Next adjustments (targeted):
  - Add a tiny capability probe in replay to detect whether kernel overlay captures upper changes; if not, automatically fall back to fuse-overlayfs (present in our image) before degrading further.
  - Optionally add chroot/pivot_root mode (CAP_SYS_CHROOT) when appropriate; continue to cd into merged as a safe default.
  - Emit “[replay] world strategy: overlay|fuse|chroot|direct” when --replay-verbose is on, plus a one‑line upper summary when fs_diff is empty.
- Rationale to test on native Linux (your Manjaro host):
  - Confirms whether the empty fs_diff is a container‑specific quirk or a general issue.
  - Provides a clean baseline for Phase B before we add the fuse-overlayfs fallback and extended strategy selection.

How to Test on Manjaro (Arch family) – Step‑by‑Step
1) Install base tooling (run as root or with sudo):
   - pacman -Syu --needed base-devel git curl jq ripgrep
   - pacman -S --needed nftables conntrack-tools iproute2 iptables iputils
   - pacman -S --needed libseccomp
   - Optional: fuse-overlayfs (only if you want to test the user-space fallback; not required for native host kernel overlay)
   - Optional niceties: fd bat zsh less unzip

2) Install Rust toolchain (user scope):
   - If not installed: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   - source "$HOME/.cargo/env"
   - rustup toolchain install 1.89.0; rustup default 1.89.0

3) Kernel prerequisites (root):
   - Enable user namespaces (often already allowed on Arch/Manjaro):
     - sysctl -n kernel.unprivileged_userns_clone  # expect 1
     - If not 1: echo 'kernel.unprivileged_userns_clone=1' > /etc/sysctl.d/99-userns.conf && sysctl --system
   - Allow dmesg reads for LOG tests (optional but useful):
     - echo 'kernel.dmesg_restrict=0' > /etc/sysctl.d/99-dmesg.conf && sysctl --system
   - Ensure overlayfs module available:
     - modprobe overlay || true
   - Confirm cgroup v2 mounted:
     - test -f /sys/fs/cgroup/cgroup.controllers && echo "cgroup v2 present" || echo "cgroup v2 missing"

4) Build substrate on Manjaro (user):
   - cd /path/to/this/repo
   - cargo build

5) Validate Phase A (replay semantics) on Linux (user):
   - export HOME="$HOME"  # ensure consistency
   - mkdir -p /tmp/phaseA && cd /tmp/phaseA
   - target/debug/substrate -c "bash -lc 'echo hello > out.txt'"
   - target/debug/substrate -c "bash -lc 'echo one | sed s/o/O/g > piped.txt && printf \"b\\nc\\n\" | wc -l > count.txt'"
   - SPAN1=$(tail -n 200 "$HOME/.substrate/trace.jsonl" | jq -r 'select(.event_type=="command_complete") | .span_id' | tail -n 2 | head -n 1)
   - SPAN2=$(tail -n 200 "$HOME/.substrate/trace.jsonl" | jq -r 'select(.event_type=="command_complete") | .span_id' | tail -n 1)
   - target/debug/substrate --replay-verbose --replay "$SPAN1"
   - target/debug/substrate --replay-verbose --replay "$SPAN2"
   - Expect: exit_code=0 for both; out.txt=hello; piped.txt=One; count.txt=2; verbose shows mode “bash -lc” and correct cwd.

6) Validate Phase B (overlay‑backed fs_diff) on Linux (requires privilege for mounts):
   - These steps may require root or CAP_SYS_ADMIN. If you’re non‑root, prefix with sudo -E to preserve HOME.
   - mkdir -p /tmp/phaseB && cd /tmp/phaseB
   - target/debug/substrate -c "bash -lc 'mkdir -p demo && echo data > demo/file.txt'"
   - SPAN=$(tail -n 200 "$HOME/.substrate/trace.jsonl" | jq -r 'select(.event_type=="command_complete") | select(.cmd | test("mkdir -p demo")) | .span_id' | tail -n 1)
   - export SUBSTRATE_REPLAY_USE_WORLD=1
   - target/debug/substrate --replay-verbose --replay "$SPAN"
   - Expect: "Filesystem changes:" printed with lines like:
     - "  + demo" and "  + demo/file.txt" (writes)
   - If fs_diff is empty or you see an overlay error, please capture the verbose lines printed (there should be a warning in verbose mode) and the output of:
     - grep overlay /proc/filesystems
     - lsmod | grep overlay || true
     - ls -l /var/lib/substrate/overlay | tail -n 10

7) Optional quick nftables sanity (Phase D preview):
   - nft --version
   - sudo RUST_LOG=info cargo test -p world -- --nocapture test_nftables_rules
   - Note: The per‑world rules and LOGs come in later phases; this is a simple availability check.

What happens next (after Manjaro test)
- If fs_diff appears on Manjaro:
  - We’ll add the capability probe to pick overlay vs fuse-overlayfs automatically in containers, and proceed with cgroups (Phase C) and nftables (Phase D).
- If fs_diff is still empty on Manjaro:
  - I’ll add the fuse-overlayfs fallback and re‑test. If both overlay and fuse fail to capture upper writes, we’ll switch to a pivot_root/chroot execution path and verify; worst case, we’ll flag degraded_components and proceed with cgroups/netfilter coverage while we debug the mount specifics.

Objective
- Bring substrate to 100% compliance with Implementation Phase 4 and PRE_PHASE_4_5 hardening on Linux, validated inside the Podman rootful VM/container.
- Address current gaps: replay parsing/quoting, overlayfs fs_diff, per‑world cgroups, per‑world nftables with LOG, trace consolidation, diagnostics, and reliable shim deployment/usage.

Acceptance Criteria (must all pass)
- Replay faithfully re-executes commands (including shell metacharacters) and emits fs_diff for write spans.
- Per‑world cgroup v2 subtree exists during replay with non‑empty cgroup.procs.
- Per‑world nftables tables/chains install; replayed disallowed network attempts are blocked with rate‑limited LOG lines in dmesg.
- A single JSONL trace contains spans with fs_diff and scopes_used.
- “Doctor” or verbose replay prints capability checks and explicit degradation messages when features are unavailable.
- Shims deploy/remove reliably; shim execution path resolves correctly during substrate -c.

Environment (reference)
- Container: Debian 13 (trixie) based Rust image, privileged, with: build-essential, nftables, conntrack, libseccomp-dev, jq, ripgrep, curl, iproute2, etc.
- VM: Podman rootful Fedora CoreOS, userns clone + overlay module + cgroup v2 enabled. Guide: docs/DEV_PODMAN_LINUX_TESTING.md.

Summary of Current Failures (from Codex report)
- Replay errors on quoted/redirection commands; fs_diff is null for write spans.
- No per‑world cgroups created.
- No per‑world nftables tables/rules or LOGs.
- Trace split/duplicated; not all spans consolidated.
- Shims deploy/remove OK but prior runs showed substrate-shim resolution issues in some environments.

Roadmap & Implementation Steps

1) Replay Execution Parity (quoting/pipes/redirection)
- Goal: Execute the original command exactly as it ran.
- Approach: Preserve the raw command string recorded in the span. If it contains shell metacharacters, run via /bin/bash -lc '<raw>' with correct quoting; otherwise exec directly.

Code sketch (Rust): crates/shell/src/lib.rs (replay execution path)
```rust
fn needs_shell(cmd: &str) -> bool {
    let meta = ['|','&',';','<','>','(',')','$','`','"','\'','{','}','*','?','[',']','~'];
    cmd.chars().any(|c| meta.contains(&c))
}

fn run_replay_command(raw_cmd: &str, cwd: &Path) -> anyhow::Result<i32> {
    use std::process::Command;
    let status = if needs_shell(raw_cmd) {
        // Preserve semantics with a login-compatible shell
        Command::new("/bin/bash")
            .arg("-lc").arg(raw_cmd)
            .current_dir(cwd)
            .status()?
    } else {
        // Tokenize minimally: first token is program, rest as one arg string?
        // Prefer executing shell anyway unless you have captured argv vector.
        Command::new("/bin/bash")
            .arg("-lc").arg(raw_cmd)
            .current_dir(cwd)
            .status()?
    };
    Ok(status.code().unwrap_or(128))
}
```

Notes:
- If span already stores argv vector, prefer exact argv exec for the simple case; otherwise default to bash -lc for correctness with redirection/pipes.
- Ensure working directory is the span’s cwd.

Tests:
- Redirection: `echo hi > a`, Append: `echo hi >> a`, Pipe: `echo hi | sed 's/h/H/'`, And/Or: `touch a && ls a`, Subshells, Quotes.

2) Overlayfs-backed Replay + fs_diff Collection
- Goal: Run the replay in an isolated overlay so we can compute non-destructive fs_diff for any command.
- Approach:
  - Build a per‑replay world dir, e.g., `/run/substrate/worlds/<world_id>/` with `upper`, `work`, `merged`.
  - Mount overlay: `mount -t overlay overlay -o lowerdir=<cwd>,upperdir=upper,workdir=work merged`.
  - chroot/pivot_root into `merged` (after making it rprivate) and run the command.
  - After exit, walk `upper` to produce fs_diff (CREATED/MODIFIED/DELETED). Serialize into command_complete span.
  - Teardown: unmount, remove dir.

Code sketch (Rust): crates/world/src/overlayfs.rs
```rust
pub struct OverlayFs { base_dir: PathBuf, world_id: String }

impl OverlayFs {
  pub fn mount_for_cwd(&self, cwd: &Path) -> Result<OverlaySession> { /* create upper/work/merged; mount overlay */ }
}

pub struct OverlaySession { pub merged: PathBuf, pub upper: PathBuf }

impl OverlaySession {
  pub fn compute_diff(&self) -> Result<Vec<FsChange>> { /* walk upper; emit creates/modifies/deletes */ }
  pub fn teardown(self) -> Result<()> { /* umount merged; cleanup */ }
}
```

Integration:
- In replay flow: if `SUBSTRATE_REPLAY_USE_WORLD=1`, create world + overlay session scoped to span.cwd; run command; compute diff; put into span.complete.fs_diff.

Tests:
- Create file/dir; modify file contents; delete file; rename.

3) Per‑World cgroups v2
- Goal: For each replay world, create `/sys/fs/cgroup/substrate/<world_id>` and add the replay process to it.
- Approach:
  - Ensure cgroup v2 mount exists; create root dir `/sys/fs/cgroup/substrate` (once).
  - Create per‑world dir, write child PID to `cgroup.procs` before launching command.
  - On teardown, remove empty dir.

Code sketch:
```rust
fn ensure_cgroup_root() -> Result<()> {
  if !Path::new("/sys/fs/cgroup/cgroup.controllers").exists() {
    anyhow::bail!("cgroup v2 not mounted");
  }
  std::fs::create_dir_all("/sys/fs/cgroup/substrate").ok();
  Ok(())
}

fn attach_pid(world_id: &str, pid: i32) -> Result<()> {
  let dir = format!("/sys/fs/cgroup/substrate/{}", world_id);
  std::fs::create_dir_all(&dir)?;
  std::fs::write(format!("{}/cgroup.procs", dir), pid.to_string())?;
  Ok(())
}
```

4) Per‑World nftables with LOG + drop
- Goal: Install per‑world nftables rules that allow an allowlist set and LOG+drop the rest; uninstall on teardown.
- Approach: Use `nft` programmatically during world setup/teardown.

Example rules (inet table):
```bash
nft add table inet substrate_${WORLD}
nft add set inet substrate_${WORLD} allowed4 { type ipv4_addr; flags interval; }
nft add set inet substrate_${WORLD} allowed6 { type ipv6_addr; flags interval; }
nft add chain inet substrate_${WORLD} egress { type filter hook output priority 0; }
nft add rule inet substrate_${WORLD} egress ip daddr @allowed4 accept
nft add rule inet substrate_${WORLD} egress ip6 daddr @allowed6 accept
nft add rule inet substrate_${WORLD} egress limit rate 10/second log prefix "substrate-dropped-${WORLD} " drop
```

Rust integration:
```rust
fn nft(args: &[&str]) -> Result<()> { let s = Command::new("nft").args(args).status()?; anyhow::ensure!(s.success(), "nft failed"); Ok(()) }
```

Tests:
- Install/remove rules for a dummy world.
- Replay curl with empty allowlist → blocked; `dmesg -T | grep substrate-dropped-` shows lines.

5) Trace Consolidation
- Goal: All spans (start/complete) live in a single JSONL: `~/.substrate/trace.jsonl`.
- Approach: Route all trace writes through `substrate_trace::append_to_trace`. Deprecate any separate `.trace_shell.jsonl` and ensure shell layer calls into common tracer.

6) Diagnostics & Verbose Replay
- Add a `substrate world doctor` or `substrate --doctor` that prints:
  - Kernel, userns availability, overlay mount test, cgroup v2 check, nft presence, dmesg_restrict value.
- Add `--replay-verbose` to log world setup steps and failures (userns, mount ns, pivot_root, netns, cgroup, nft install).

7) Shim Deployment & Resolution
- Validate shim deploy at runtime and improve resolution:
  - On `substrate --shim-status`, compare shim symlinks (e.g., curl → substrate-shim) to expected binary path.
  - If not present or version mismatch, print clear guidance to run `substrate --shim-deploy`.
  - During shim invocation, set/propagate `SHIM_ORIGINAL_PATH` and fall back to absolute target path when PATH lookup fails.
- Ensure build produces `target/<profile>/substrate-shim` and that deploy uses that path for symlink targets.

Code touch points
- crates/shell/src/lib.rs (replay execution + trace writing)
- crates/world/src/overlayfs.rs (mount, diff)
- crates/world/src/netfilter.rs (nft rules install/remove)
- crates/world/src/cgroups.rs (ensure root, per‑world attach)
- crates/trace (append_to_trace API; include fs_diff/scopes_used)
- crates/shell/src/shim_deploy.rs (status/deploy/remove improvements)
- bin CLI: add `--doctor` and/or `--replay-verbose`

Tests (add under appropriate crates; skip/require root)
- world::test_overlayfs_diff (root only): write/modify/delete, assert fs_diff.
- world::test_nftables_rules (root only): installs/removes, grep ruleset.
- world::test_cgroups_v2 (root only): creates per-world dir, cgroup.procs non-empty while process runs.
- shell::test_replay_quoting (unit): various commands render via bash -lc and succeed.

Operational Validation (scripted)
1) Build & run in Podman container (see DEV_PODMAN_LINUX_TESTING.md).
2) Run world tests: `RUST_LOG=info cargo test -p world -- --nocapture`.
3) Run Codex prompt in docs/CODEX_PROMPT_PHASE4_VALIDATION.md and capture artifacts.
4) Verify:
   - fs_diff present for write spans
   - nft LOG lines in dmesg on blocked curl
   - per‑world cgroups directory and cgroup.procs shows PIDs
   - replay of redirection/pipes succeeds
   - trace.jsonl contains consolidated spans

Rollout Plan
- Phase A (parsing & overlay): Implement replay bash -lc execution + overlay fs, fs_diff emitted. Add unit tests.
- Phase B (cgroups & nftables): Implement per‑world cgroup attach + nft rules install/remove. Add integration tests.
- Phase C (trace & diagnostics): Consolidate trace + add doctor and verbose replay.
- Phase D (shims): Harden deploy/status and resolution during execution.
- Phase E (docs/CI): Update HOWTO_PRIVILEGED_TESTS and add a CI job that runs doctor + a minimal privileged test in Podman.

Notes
- All isolation steps should fail loud in verbose mode and surface a clear “graceful degradation” message otherwise.
- Keep world IDs consistent across cgroup and nft naming to simplify diagnostics.

Enhancements and Missing Considerations (addendum)
- Non‑goals/assumptions: privileged Linux environment available (Podman rootful VM or Linux host); replay equivalence defined on stdout/stderr/exit_code and scoped side effects; default deny egress during replay unless specified.
- Replay env/umask: reinject minimal env (PATH, HOME, SHELL, LANG/LC_ALL, RUST_LOG) and honor umask if captured; default to 022.
- Replay coverage: add tests for here‑docs, input redirection, Unicode/locale edge cases.
- FsDiff spec and limits: define JSON fields (path, kind, mode, uid, gid, size, mtime, file_type, link_target); map overlay whiteouts to deletions; cap entries (e.g., 10k) and set fs_diff_truncated with counts.
- Cgroups controllers: enable required controllers in cgroup.subtree_control (e.g., +pids +cpu +memory); handle races and busy teardown with GC.
- Netfilter edges: skip IPv6 when disabled; prefer nft JSON input; apply rules in per‑world netns if used; print explicit degradation when dmesg LOGs not observable; robust teardown tolerating ENOENT.
- Trace schema/versioning: add trace_schema_version and fs_diff_schema_version; provide a trace compaction/validation tool.
- Diagnostics: add `substrate world doctor --json` and `substrate world gc`; include a verbose replay mode printing every isolation step and failure.
- Shims hardening: ensure ~/.substrate/shims is first in PATH when SUBSTRATE_WORLD=enabled; persist SHIM_ORIGINAL_PATH; make deploy idempotent/atomic; store/version shim metadata and trigger redeploy on drift.
- Concurrency/orchestration: globally unique world IDs (UUIDv7); per‑world roots under /run/substrate/worlds/<world>; file locks to prevent reuse; configurable concurrency limits.
- Fallback contract: on overlay/cgroup/nft/dmesg failures, emit explicit stderr lines and record degraded_components in spans; fs_diff may be null only with explicit overlay degradation marker.
- Security: document SELinux/AppArmor handling; our run scripts already disable SELinux labels in container and unconfine seccomp.
- Cross‑arch/platform: document aarch64 vs amd64 implications (BUILD_ARCH=amd64 option), Debian vs Fedora package name differences.
- Observability: logging categories (world,netfilter,cgroups,overlay); rate limit nft LOG; metrics for worlds created/failed, overlay mounts, nft failures, fs_diff sizes.
- CI & repro: add a smoke test that runs doctor in Podman VM and gates privileged integration tests; provide a make target to run the validation playbook non‑interactively.
- Data model stability: ship JSON Schemas for trace and fs_diff; add a small validator to catch schema regressions.
- Known limitations: extremely hardened kernels may block overlay/nft; doctor reports and tests skip with explicit reason; dmesg may be restricted; IPv6 may be disabled.
- Example doctor output (target):
  {
    "kernel": "6.15.9-201.fc42.aarch64",
    "cgroup_v2": true,
    "overlay_mount_ok": true,
    "userns": "enabled",
    "nft": {"present": true, "version": "1.1.3"},
    "dmesg_restrict": 0,
    "selinux": "disabled-in-container",
    "capabilities": ["CAP_SYS_ADMIN","CAP_NET_ADMIN"]
  }

Deep Technical Details and Edge Cases (must-read for implementers)
- World setup sequence (recommended order):
  1) Make mounts private: `mount --make-rprivate /`.
  2) Create per-world runtime root: `/run/substrate/worlds/<world_id>`.
  3) Bind-mount the original cwd to `<root>/lower` to ensure lowerdir shares FS with `upper/work`.
  4) Create `<root>/{upper,work,merged}` and mount overlay: `mount -t overlay overlay -o lowerdir=<root>/lower,upperdir=<root>/upper,workdir=<root>/work <root>/merged`.
  5) Prepare pivot_root: bind-mount `<root>/merged` to itself, make `old_root=<root>/merged/.old_root`, then `pivot_root <root>/merged <root>/merged/.old_root`.
  6) `chdir("/")`; mount minimal `proc`: `mount -t proc proc /proc`. (Optionally mount tmpfs on /tmp.)
  7) Detach old root: `umount2("/.old_root", MNT_DETACH)`; `rmdir("/.old_root")`.
  8) Prepare netns (optional): if creating a per-world netns, `unshare(CLONE_NEWNET)`, bring up loopback (`ip link set lo up`).
  9) Install nftables rules (inet table) inside the target netns.
  10) Create cgroup dir and attach PIDs before launching the command.
  11) Launch command with sanitized env/umask. Ensure `close_fds` or CLOEXEC to avoid fd leaks.
  12) On teardown: kill remaining PIDs in cgroup (best-effort), remove nftables table, unmount overlay, GC directories.

- Signal handling & timeouts:
  - Run child in its own process group; propagate SIGINT/SIGTERM; on timeout, send SIGTERM then SIGKILL to the group; always run teardown.
  - Use `prctl(PR_SET_CHILD_SUBREAPER)` or equivalent to prevent zombie leakage in the supervisor.

- Environment & security hygiene:
  - Sanitize PATH; prepend the shim dir when `SUBSTRATE_WORLD=enabled`.
  - Preserve `SHIM_ORIGINAL_PATH` and fall back to absolute tool paths when shims fail.
  - Optionally clear sensitive vars (AWS_*, GCP_*, SSH_AUTH_SOCK) unless explicitly allowed.

- DNS and allowlist policy:
  - Map domain allowlist → IP sets before replay; support both A and AAAA; respect TTLs and refresh policy.
  - Always allow loopback (127.0.0.1/8, ::1) rules.
  - Consider blocking DNS egress too unless explicitly allowed (document policy).

- nftables naming & limits:
  - Sanitize world table name: `substrate_<short_id>` where `<short_id>` is a short hash to keep names < 64 chars; only `[A-Za-z0-9_ ]`.
  - Rate-limit LOG rules (e.g., 10/second) to avoid log flooding; include `world_id` prefix.

- Overlayfs constraints:
  - `upper` and `work` must be on the same filesystem; binding `cwd` into `<root>/lower` ensures `lower` is on that FS too.
  - Handle whiteouts translating to `deleted` entries in fs_diff; include `type_change` for file→dir, dir→file.

- /proc and /dev considerations:
  - Minimal `/proc` usually suffices; some tools may require `/dev/null` and `devpts`. Mount only if necessary and document.

- cgroups controllers:
  - Enable `+pids +cpu +memory` in `cgroup.subtree_control` under `/sys/fs/cgroup/substrate` where permitted; if denied, log explicit degradation and proceed without limits.
  - Kill-on-teardown: iterate `cgroup.procs` and send SIGKILL to remaining PIDs to avoid residuals.

- Close-on-exec and fd leaks:
  - Ensure `close_fds(true)` (std::process::Command) and/or set FD_CLOEXEC on open descriptors prior to exec; avoid leaking host fds into world.

- Trace growth & rotation:
  - Add rotation/compaction to prevent unbounded growth; document defaults (e.g., rotate at 100MB, keep last N files).

- Parallel worlds & interference:
  - Use unique IDs and per-world resources; serialize nft table names; ensure teardown runs even on failures; provide `world gc` to reclaim leftovers.

- IPv6 disabled kernels:
  - Detect `/proc/sys/net/ipv6/conf/all/disable_ipv6=1` and skip v6 constructs; mark in diagnostics.

- Journald vs dmesg:
  - When `kernel.dmesg_restrict=1`, prefer `journalctl -k` to observe LOG lines; document this fallback.
