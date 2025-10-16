# Linux Button-Up Plan (World, Agent, Replay)

Purpose
- Finalize “Always World” semantics on Linux for both non-PTY and PTY execution paths.
- Tighten the HTTP agent surface (fs_diff over /v1/execute), align replay output, and clean up user-facing messages.
- Deliver crisp validation steps (Podman + native), acceptance criteria, rollback notes, and code pointers.

Status
- Baseline implemented (summary):
  - Shell: default-on world init (Linux), PTY WS via world-agent with auto-start, non-PTY best-effort via agent HTTP.
  - World backend: overlayfs diff for non-PTY, per-world netns + nftables, cgroup v2 helpers; replay uses LinuxLocalBackend(alway_isolate).
  - Docs: WORLD.md, REPLAY.md updated; CI reference job exists.
- Remaining to button up: non-PTY auto-start, fs_diff via agent HTTP, replay scopes in verbose, message clarity, doc polish. Optional: heavy isolation wiring.

Scope (in-scope vs. deferred)
- In-scope: Shell non-PTY agent auto-start parity; agent HTTP returns fs_diff; shell consumes fs_diff; replay prints scopes; warning message clarity; doc updates; tests/CI.
- Deferred: PTY fs_diff strategy; full heavy isolation by default (keep optional/gated);

Prerequisites
- Linux kernel features available (or graceful degradations):
  - cgroup v2: /sys/fs/cgroup/cgroup.controllers exists
  - nft: `nft` available (optional, for netfilter rules/logging)
  - overlayfs and/or fuse-overlayfs + /dev/fuse for fs diffs
- Tooling: Rust stable, Podman (optional), jq, ripgrep; workspace builds.

High-Level Plan (order of execution)
1) Non-PTY agent auto-start parity in shell
2) fs_diff returned via agent HTTP (/v1/execute) and consumed by shell
3) Replay verbose: print scopes_used
4) Warning message clarity (shell vs replay)
5) Docs updates (WORLD.md, REPLAY.md, DEV_PODMAN_LINUX_TESTING.md)
6) Tests and CI updates
7) Optional: heavy isolation wiring (capability-gated)

---

1) Non-PTY Agent Auto-Start Parity

Goal: Ensure non-PTY commands use the world-agent by default when world is enabled, mirroring PTY behavior.

Where
- `crates/shell/src/lib.rs`
  - In `execute_command(...)` before calling `exec_non_pty_via_agent(...)`.
  - Reuse `ensure_world_agent_ready()` (already used for PTY WS) when `SUBSTRATE_WORLD=enabled`.

Change (code sketch)
```rust
// Before:
let world_enabled = env::var("SUBSTRATE_WORLD").unwrap_or_default() == "enabled";
let uds_exists = std::path::Path::new("/run/substrate.sock").exists();
if world_enabled || uds_exists {
    if let Ok((exit_code, stdout, stderr, scopes_used)) = exec_non_pty_via_agent(trimmed) { ... }
    else { eprintln!("substrate: warn: world exec failed, running direct"); }
}

// After:
let world_enabled = env::var("SUBSTRATE_WORLD").unwrap_or_default() == "enabled";
let uds_exists = std::path::Path::new("/run/substrate.sock").exists();
if world_enabled || uds_exists {
    if world_enabled {
        let _ = ensure_world_agent_ready(); // ignore error here to allow fallback
    }
    if let Ok((exit_code, stdout, stderr, scopes_used, fs_diff_opt)) = exec_non_pty_via_agent(trimmed) { /* see step 2 */ }
    else { eprintln!("substrate: warn: shell world-agent exec failed, running direct"); }
}
```

Acceptance
- `substrate -c "echo hi"` on Linux uses world-agent path without manual agent start.
- No spurious fallback warnings when agent binary is available.

Sanity Checks
- Podman: run `scripts/podman/run.sh bash -lc 'cargo build && /src/target/debug/substrate -c "echo ok"'` and confirm no fallback warning.
- Native Linux: same as above.

---

2) Return fs_diff via Agent HTTP and Consume in Shell

Goal: Make non-PTY agent HTTP path return and persist fs_diff so spans include filesystem changes immediately.

Server-side Changes
- `crates/agent-api-types/src/lib.rs`
  - Add `substrate-common` dep and reuse FsDiff type, mirroring world-api:
  ```toml
  # Cargo.toml
  substrate-common = { path = "../common" }
  ```
  ```rust
  // lib.rs
  pub use substrate_common::FsDiff;
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ExecuteResponse {
      pub exit: i32,
      pub span_id: String,
      pub stdout_b64: String,
      pub stderr_b64: String,
      pub scopes_used: Vec<String>,
      #[serde(skip_serializing_if = "Option::is_none")]
      pub fs_diff: Option<FsDiff>,
  }
  ```

- `crates/world-agent/src/service.rs`
  - Map `result.fs_diff` onto `ExecuteResponse.fs_diff`.
  ```rust
  let result = self.backend.exec(&world, exec_req)?;
  Ok(ExecuteResponse {
      exit: result.exit,
      span_id,
      stdout_b64: base64::engine::general_purpose::STANDARD.encode(result.stdout),
      stderr_b64: base64::engine::general_purpose::STANDARD.encode(result.stderr),
      scopes_used: result.scopes_used,
      fs_diff: result.fs_diff, // NEW
  })
  ```

  JSON Schema (success)
  ```json
  {
    "exit": 0,
    "span_id": "spn_01994...",
    "stdout_b64": "aGVsbG8=",
    "stderr_b64": "",
    "scopes_used": ["tcp:github.com:443"],
    "fs_diff": {
      "writes": ["/tmp/pretest/a.txt"],
      "mods": [],
      "deletes": [],
      "truncated": false
    }
  }
  ```

  Quick local probe (UDS)
  ```bash
  # Capabilities
  curl --unix-socket /run/substrate.sock http://localhost/v1/capabilities | jq .
  # Execute (non-PTY) minimal example
  curl --unix-socket /run/substrate.sock \
    -H 'content-type: application/json' \
    -d '{"cmd":"bash -lc \"echo ok > x.txt\"","cwd":"/tmp","env":{},"pty":false,"agent_id":"human"}' \
    http://localhost/v1/execute | jq .
  ```

Client-side Changes
- `crates/shell/src/lib.rs`
  - Function anchor: `fn exec_non_pty_via_agent(cmd: &str) -> anyhow::Result<(i32, Vec<u8>, Vec<u8>, Vec<String>)>`
    - Change return type to include `Option<substrate_common::FsDiff>`.
    - After parsing `v: serde_json::Value`, if `v.get("fs_diff").is_some()`, deserialize with:
      `let fs_diff: substrate_common::FsDiff = serde_json::from_value(v["fs_diff"].clone())?;`
  - Call site in `execute_command(...)` where agent path succeeds:
    - Use returned `fs_diff_opt` when finishing span and skip `collect_world_telemetry`.

Change (code sketch)
```rust
fn exec_non_pty_via_agent(cmd: &str) -> anyhow::Result<(i32, Vec<u8>, Vec<u8>, Vec<String>, Option<substrate_common::FsDiff>)> {
    // ... existing HTTP-over-UDS code ...
    let v: serde_json::Value = serde_json::from_str(body)?;
    let exit_code = v["exit"].as_i64().unwrap_or(1) as i32;
    let stdout = base64::engine::general_purpose::STANDARD.decode(v["stdout_b64"].as_str().unwrap_or("")).unwrap_or_default();
    let stderr = base64::engine::general_purpose::STANDARD.decode(v["stderr_b64"].as_str().unwrap_or("")).unwrap_or_default();
    let scopes_used = v["scopes_used"].as_array().unwrap_or(&vec![])
        .iter().filter_map(|s| s.as_str().map(|s| s.to_string())).collect();
    let fs_diff = if let Some(fd) = v.get("fs_diff") {
        use substrate_common::FsDiff;
        let mut diff = FsDiff::default();
        for key in ["writes", "mods", "deletes"] { /* map arrays of strings to Paths */ }
        Some(diff)
    } else { None };
    Ok((exit_code, stdout, stderr, scopes_used, fs_diff))
}

// In execute_command(...), when agent path succeeds:
if let Some(active_span) = span { let _ = active_span.finish(exit_code, scopes_used, fs_diff_opt); }
```

Exact call-site anchor
- Search for: `if world_enabled || uds_exists {` in `execute_command(...)`
- Immediately before calling `exec_non_pty_via_agent`, insert:
  ```rust
  if world_enabled { let _ = ensure_world_agent_ready(); }
  ```

Acceptance
- A simple write (`echo x > out.txt`) shows fs_diff in the command_complete span when routed via agent HTTP.

Sanity Checks
- Podman: write span then inspect last command_complete in `~/.substrate/trace.jsonl` for fs_diff.
- Doctor: no regressions in PTY WS path.

---

3) Replay Verbose: Print scopes_used

Where
- `crates/replay/src/replay.rs` after `ExecutionResult` is obtained (both world and direct paths).

Change (code sketch)
```rust
if verbose && !result.scopes_used.is_empty() {
    eprintln!("[replay] scopes: {}", result.scopes_used.join(","));
}
```

Acceptance
- With `--replay-verbose`, scopes are printed when any were collected.

---

4) Warning Message Clarity

Where
- `crates/shell/src/lib.rs` non-PTY fallback warning: change to
  `substrate: warn: shell world-agent exec failed, running direct`.
- Replay warnings already use `[replay] warn: ...` prefix (keep as-is).

Acceptance
- Separate origins are clear in logs; fewer user confusions.

---

5) Docs Updates

- `docs/WORLD.md`
  - Note non-PTY auto-start behavior and fallback semantics.
  - Clarify fs_diff availability via agent HTTP.

 - `docs/REPLAY.md`
  - Document scopes printing in verbose mode, example:
    ```
    [replay] scopes: tcp:github.com:443,tcp:registry.npmjs.org:443
    ```

- `docs/DEV_PODMAN_LINUX_TESTING.md`
  - Simplify replay steps (default world on) and add a quick fs_diff check on agent HTTP route.

Acceptance
- Docs reflect current behavior exactly; commands copy/paste clean.

---

6) Tests & CI

Unit/Small Integration
- world-agent service: round-trip ExecuteResponse with fs_diff (serde test).
- shell: exec_non_pty_via_agent parses optional fs_diff correctly (unit parsing test against a fixture JSON string).
- replay: verbose prints scopes when present.

Privileged Integration (best-effort)
- Podman script: run write span -> replay default world -> verify overlay/copy-diff strategy and fs_diff non-empty.
- Non-PTY shell path: ensure agent auto-start then create file; expect fs_diff in span.

CI
- `.github/workflows/pre45.yml`: keep privileged world tests best-effort; add a job that builds agent types and runs serde tests.

Acceptance
- Workspace builds; tests green or skipped appropriately; no regression in PTY path.

File anchors for tests
- world-agent: `crates/world-agent/src/service.rs` module tests and/or new JSON fixture under `crates/world-agent/tests/`.
- shell: add a lightweight unit in `crates/shell` gated behind `#[cfg(test)]` that calls a helper deserializer for `fs_diff` from a JSON snippet.

---

7) Optional: Heavy Isolation Wiring (capability-gated)

Goal
- Offer a mode (env or feature) to apply `LinuxIsolation::apply()` for non-PTY jobs when capabilities allow (userns, mountns, pivot_root). Default remains current model.

Where
- `crates/world/src/session.rs` during setup. Add a guarded call:
```rust
if std::env::var("SUBSTRATE_HEAVY_ISOLATION").as_deref() == Ok("1") {
    let iso = crate::isolation::LinuxIsolation::new(&self.spec);
    let _ = iso.apply(&self.root_dir, &self.project_dir, &self.cgroup_path);
}
```

Risks
- Kernel variance; permission errors. Keep best-effort logging only; never fail user workloads.

Acceptance
- When enabled in privileged env, /proc is re-mounted, system dirs RO bound, pivot_root works; otherwise graceful no-op.

Note
- Keep this path strictly best-effort with clear warnings. Do not enable by default. Gate via env var only.

---

Validation Playbook (Copy/Paste)

Podman
```bash
bash scripts/podman/build.sh
scripts/podman/run.sh bash -lc '
  cargo build -q
  # Kernel and tools sanity
  bash scripts/check-container-prereqs.sh || true

  # non-PTY auto-start + fs_diff
  mkdir -p /tmp/t && cd /tmp/t
  /src/target/debug/substrate -c "bash -lc 'echo data > a.txt'"
  tail -n 50 ~/.substrate/trace.jsonl | jq -r "select(.event_type==\"command_complete\") | [.span_id, (.fs_diff.writes|tostring)] | @tsv" | tail -n1

  # agent UDS presence and capabilities
  ls -l /run/substrate.sock
  curl --unix-socket /run/substrate.sock http://localhost/v1/capabilities | jq .

  # direct agent HTTP exec: response should include fs_diff
  curl --unix-socket /run/substrate.sock \
    -H "content-type: application/json" \
    -d "{\"cmd\":\"bash -lc \\\"echo more >> a.txt\\\"\",\"cwd\":\"/tmp/t\",\"env\":{},\"pty\":false,\"agent_id\":\"human\"}" \
    http://localhost/v1/execute | jq .

  # replay default world + scopes
  SPAN=$(tail -n 200 ~/.substrate/trace.jsonl | jq -r "select(.event_type==\"command_complete\") | .span_id" | tail -n1)
  /src/target/debug/substrate --replay-verbose --replay "$SPAN"

  # netfilter sanity (best-effort)
  /src/target/debug/substrate -c "bash -lc 'curl -m2 http://example.com || true'"
  dmesg -T | rg "substrate-dropped-" | tail -n 5 || true
'
```

Native Linux
```bash
cargo build
target/debug/substrate -c "bash -lc 'mkdir -p d && echo x > d/f'"
SPAN=$(tail -n 200 ~/.substrate/trace.jsonl | jq -r 'select(.event_type=="command_complete") | .span_id' | tail -n1)
target/debug/substrate --replay-verbose --replay "$SPAN"
ls -l /run/substrate.sock && curl --unix-socket /run/substrate.sock http://localhost/v1/capabilities | jq .
```

Acceptance Checklist
- Non-PTY shell commands use world with no manual agent start (Linux).
- Agent HTTP responses include fs_diff; shell spans show fs_diff for non-PTY.
- Replay verbose prints world strategy and scopes when present.
- Clear, differentiated warnings; no confusing cross-path messages.
- CI builds; tests pass or skip appropriately.

Rollback Strategy
- Revert agent-api-types change if downstream impact is high; the shell can continue telemetry fallback (pre-change behavior) while we iterate.
- Keep PTY WS path unaffected; changes must be modular to non-PTY HTTP route only.

Risks & Mitigations
- Kernel features missing: maintain degrade paths with explicit, rate-limited warnings.
- API evolution: adding fs_diff is additive; guard shell parsing to treat absence as None.
- Performance: fs_diff payload volume—truncate or summarize at large counts (future enhancement).

Timeline (estimate)
- Auto-start (non-PTY): 0.5 day
- fs_diff via HTTP + shell consumption: 0.5–1 day
- Replay scopes + message clarity + docs: 0.5 day
- Tests/CI polish: 0.5 day
- Optional heavy isolation wiring: 1–1.5 days
