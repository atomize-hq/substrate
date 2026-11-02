# Codex Rust Wrapper

Async wrapper around the Codex CLI focused on the headless `codex exec` flow. The client shells out to the bundled or system Codex binary, mirrors stdout/stderr when asked, and keeps the parent process environment untouched.

- crates.io package: `unified-agent-api-codex`
- Rust library crate: `codex`

## Capability + versioning release notes (Workstream F)
- Capability probes now capture `codex --version`, `codex features list` (`--json` when available), and `--help` hints, storing results as `CodexCapabilities` snapshots with `collected_at` timestamps and `BinaryFingerprint` metadata keyed by canonical binary path.
- Guard helpers (`guard_output_schema`, `guard_add_dir`, `guard_mcp_login`, `guard_features_list`) keep optional flags off when support is unknown; surface `CapabilityGuard.notes` to operators instead of passing flags blindly.
- Cache controls: configure `CapabilityCachePolicy::{PreferCache, Refresh, Bypass}` via `capability_cache_policy` or `bypass_capability_cache`. Use `Refresh` for TTL/backoff windows or hot-swaps that reuse the same path; use `Bypass` when metadata is missing (FUSE/overlay filesystems) or when you need an isolated probe that skips cache reads/writes.
- TTL/backoff helper: `capability_cache_ttl_decision` inspects `collected_at` and fingerprint presence to recommend `Refresh` vs `Bypass` for hot-swaps or metadata-missing paths (FUSE/overlay); start with a ~5 minute TTL and back off toward 10-15 minutes when metadata keeps failing.
- Overrides + persistence: `capability_snapshot` / `capability_overrides` accept manual snapshots and feature/version hints; `write_capabilities_snapshot`, `read_capabilities_snapshot`, and `capability_snapshot_matches_binary` let hosts reuse snapshots across processes while avoiding stale data when fingerprints diverge.
- Update advisories stay offline: supply `CodexLatestReleases` and call `update_advisory_from_capabilities` to prompt upgrades without this crate performing network I/O.

## Snapshot reuse + cache policy quickstart
Run the snapshot example to see disk reuse with fingerprint checks plus cache policy guidance:

```
cargo run -p unified-agent-api-codex --example capability_snapshot -- ./codex ./codex-capabilities.json auto
```

- The example loads a prior snapshot when the fingerprint matches, falls back to `CapabilityCachePolicy::Refresh` after a TTL or hot-swap, and drops to `CapabilityCachePolicy::Bypass` when metadata is missing (typical on some FUSE/overlay mounts) to avoid persisting snapshots that cannot be validated.
- Refresh vs. Bypass: use `Refresh` to re-probe while still writing back to the cache (good for TTL/backoff windows or deployments that reuse the same path); use `Bypass` for one-off probes that should not read or write cache entries when metadata is unreliable.

See `crates/codex/examples/capability_snapshot.rs` for the full flow, including fingerprint validation and snapshot persistence helpers.

## TTL/backoff helper
Use `capability_cache_ttl_decision` to decide whether to reuse a cached snapshot or force a probe with the right cache policy:

```rust
use codex::{
    capability_cache_entry, capability_cache_ttl_decision, CapabilityCachePolicy, CodexClient,
};
use std::{path::Path, time::{Duration, SystemTime}};

async fn refresh_capabilities(client: &CodexClient, binary: &Path) {
    let cached = capability_cache_entry(binary);
    let ttl = Duration::from_secs(300); // start with ~5 minutes for binaries with fingerprints
    let decision = capability_cache_ttl_decision(cached.as_ref(), ttl, SystemTime::now());

    let capabilities = if let Some(snapshot) = cached.filter(|_| !decision.should_probe) {
        snapshot
    } else {
        client.probe_capabilities_with_policy(decision.policy).await
    };

    if decision.policy == CapabilityCachePolicy::Bypass {
        // FUSE/overlay path; back off toward 10-15 minutes to avoid hammering probes.
    }

    let _ = capabilities; // reuse, refresh, or bypass based on the helper decision
}
```
- `Refresh` covers hot-swaps that reuse the same binary path even when fingerprints look unchanged.
- `Bypass` is returned when metadata is missing; avoid cache writes and increase the TTL/backoff window to reduce probe churn.

## Binary and `CODEX_HOME` isolation

- Point the wrapper at a bundled Codex binary via [`CodexClientBuilder::binary`]; if unset, it honors `CODEX_BINARY` or falls back to `codex` on `PATH`.
- Apply an app-scoped home with [`CodexClientBuilder::codex_home`]. The resolved binary is mirrored into `CODEX_BINARY`, and the provided home is exported as `CODEX_HOME` for every spawn site (exec/login/status/logout). The parent environment is never mutated.
- Use [`CodexClientBuilder::create_home_dirs`] to control whether `CODEX_HOME`, `conversations/`, and `logs/` are created up front (defaults to `true` when a home is set). `RUST_LOG` defaults to `error` if you have not set it.

```rust
use codex::{CodexClient, CodexHomeLayout};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let binary = "/opt/myapp/bin/codex";
    let codex_home = "/var/lib/myapp/codex";

    // Discover (and optionally create) the CODEX_HOME layout.
    let layout = CodexHomeLayout::new(codex_home);
    layout.materialize(true)?;
    println!("Logs live at {}", layout.logs_dir().display());

    let client = CodexClient::builder()
        .binary(binary)
        .codex_home(codex_home)
        .create_home_dirs(true)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let reply = client.send_prompt("Health check").await?;
    println!("{reply}");
    Ok(())
}
```

## `CODEX_HOME` layout helper

`CodexHomeLayout` documents where Codex stores state under an app-scoped home:

- `config.toml`
- `auth.json`
- `.credentials.json`
- `history.jsonl`
- `conversations/` for transcript JSONL files
- `logs/` for `codex-*.log` files

Call [`CodexHomeLayout::materialize`] to create the root, `conversations/`, and `logs/` directories before spawning Codex.

## Stream JSONL events

Use the streaming surface to consume `codex exec --json` output as it arrives. Disable stdout mirroring so you control the console, and set an idle timeout to fail fast if the CLI stalls.

```rust
use codex::{CodexClient, ExecStreamRequest, ThreadEvent};
use futures_util::StreamExt;
use std::{path::PathBuf, time::Duration};

# async fn demo() -> Result<(), Box<dyn std::error::Error>> {
let client = CodexClient::builder()
    .json(true)
    .quiet(true)
    .mirror_stdout(false)
    .json_event_log("logs/codex_events.log")
    .build();

let mut stream = client
    .stream_exec(ExecStreamRequest {
        prompt: "List repo files".into(),
        idle_timeout: Some(Duration::from_secs(30)),
        output_last_message: Some(PathBuf::from("last_message.txt")),
        output_schema: None,
        json_event_log: None, // override per request if desired
    })
    .await?;

while let Some(event) = stream.events.next().await {
    match event {
        Ok(ThreadEvent::ItemDelta(delta)) => println!("delta: {:?}", delta.delta),
        Ok(other) => println!("event: {other:?}"),
        Err(err) => {
            eprintln!("stream error: {err}");
            break;
        }
    }
}

let completion = stream.completion.await?;
println!("codex exited with {}", completion.status);
if let Some(path) = completion.last_message_path {
    println!("last message saved to {}", path.display());
}
# Ok(()) }
```

## Log the raw JSON stream

Set `json_event_log` on the builder or per request to tee every raw JSONL line to disk before parsing:

- The log is appended to (existing files are preserved) and flushed per line.
- Parent directories are created automatically.
- An empty string is ignored; set a real path or leave `None` to disable.
- The per-request `json_event_log` overrides the builder default for that run.

Events still flow to your `events` stream even when teeing is enabled.

## Apply or inspect task diffs

`CodexClient::apply_task` wraps `codex apply <TASK_ID>`, and `CodexClient::cloud_diff_task` wraps `codex cloud diff <TASK_ID>` when supported by the binary. `CodexClient::apply`/`CodexClient::diff` are convenience helpers that will append `<TASK_ID>` from `CODEX_TASK_ID` when set.

All of these capture stdout/stderr and return the exit status via [`ApplyDiffArtifacts`](crates/codex/src/lib.rs). They honor the builder flags you already use for streaming:

- `mirror_stdout` controls whether stdout is echoed while still being captured.
- `quiet` suppresses stderr mirroring (stderr is always returned in the artifacts).
- `RUST_LOG` defaults to `error` for these subcommands when the environment is unset; set `RUST_LOG=info` (or higher) to inspect codex internals.

```rust
use codex::CodexClient;

# async fn demo() -> Result<(), Box<dyn std::error::Error>> {
let client = CodexClient::builder()
    .mirror_stdout(false) // silence stdout while capturing
    .quiet(true)          // silence stderr while capturing
    .build();

let apply = client.apply_task("t-123").await?;
println!("exit: {}", apply.status);
println!("stdout: {}", apply.stdout);
println!("stderr: {}", apply.stderr);
# Ok(()) }
```

## RUST_LOG defaults

If `RUST_LOG` is unset, the wrapper injects `RUST_LOG=error` for spawned commands to silence verbose upstream tracing. Any existing `RUST_LOG` value is respected.

## MCP + app-server helpers

- `codex::mcp` offers typed clients for `codex mcp-server --stdio` and `codex app-server --stdio`, along with config managers for `[mcp_servers]` and `[app_runtimes]` plus launcher helpers when you want to spawn from saved config.
- Use `CodexClient::spawn_mcp_login_process` (capability-guarded) when you need an interactive bearer token for HTTP transports before persisting it via `McpConfigManager::login`.
- Examples: `mcp_codex_flow` (typed `tools/call` for `codex` + `codex-reply` with optional cancellation), `mcp_codex_tool`/`mcp_codex_reply` (raw tool calls with `--sample` payloads; use the `session_id` from `session_configured` as the `conversationId`, and note that `codex-reply` requires the session to remain active inside the same `mcp-server` process on 0.61.0), and `app_server_turns`/`app_server_thread_turn` (thread start/resume + optional interrupt). Pair these with `feature_detection` if the binary may be missing server endpoints.
- MCP `codex-reply` does **not** rehydrate conversations from disk on 0.61.0; follow-up calls only work while the original `mcp-server` process is still running. For cross-process resumes, use `codex exec resume` (CLI) or the app-server `thread/resume` path instead.

## Runtime definitions and env prep
- `[mcp_servers]` and `[app_runtimes]` live in `config.toml`; `McpConfigManager` reads/writes them.
- `StdioServerConfig` should be built with the Workstream A env prep (binary path, `CODEX_HOME`, base env, timeouts). Runtime entries layer env/timeout overrides on top of those defaults, and `CODEX_HOME` is injected when `code_home` is set.
- Resolution through the runtime/app APIs is read-only: stored config and metadata are not mutated.

## MCP runtime API (read-only)
- `McpRuntimeApi::from_config(&manager, &defaults)` loads launch-ready stdio configs or HTTP connectors from stored runtimes.
- `available` returns `McpRuntimeSummary` entries (description/tags/tool hints + transport kind).
- `launcher`, `stdio_launcher`, and `http_connector` hand back launchers/connectors without side effects; HTTP connectors resolve bearer tokens from env without overwriting existing `Authorization` headers.
- `prepare` spawns stdio runtimes or hands back HTTP connectors with tool hints preserved; use `ManagedStdioRuntime::stop` to shut down processes (drop is best-effort kill).
- Use `McpRuntimeManager` directly when you already have launchers and only need spawn/connector plumbing.

## App runtime API (read-only)
- `AppRuntimeApi::from_config(&manager, &defaults)` merges stored `[app_runtimes]` entries with defaults (binary/path/env/timeout) while keeping metadata/resume hints intact.
- `available` lists stored runtimes and metadata; `prepare`/`stdio_config` return merged stdio configs without launching.
- `start` launches an app-server and returns `ManagedAppRuntime` (metadata + merged env + `CodexAppServer` handle). Calls leave stored definitions untouched and preserve metadata for future starts.

## Pooled app runtimes
- `AppRuntimePoolApi::from_config(&manager, &defaults)` (or `AppRuntimeApi::pool_api`) wraps the pool that reuses running runtimes by name.
- `available` lists stored entries; `running` lists active runtimes; `start` reuses an existing process if one is already running; `stop`/`stop_all` clean up without altering stored definitions or metadata/resume hints.
- Pool handles still expose stdio configs via `launcher`/`prepare` so callers can inspect launch parameters without starting a process.

## Examples and tests
- `examples/mcp_codex_flow.rs`: starts `codex mcp-server`, streams `codex/event`, supports `$ /cancelRequest` and follow-up `codex/codex-reply` via `tools/call`; respects `CODEX_BINARY`/`CODEX_HOME` and does not touch stored `[mcp_servers]`.
- `examples/app_server_turns.rs`: starts/resumes `codex app-server` threads, streams items/task_complete, and can issue `turn/interrupt` after the first item; metadata/thread IDs come from server responses and are not persisted by the wrapper.
- `examples/responses_api_proxy.rs`: launches `codex responses-api-proxy` with an API key piped on stdin; falls back to a stub `--sample` path when no `OPENAI_API_KEY`/`CODEX_API_KEY` is available and polls `--server-info` for `{port,pid}`.
- `examples/stdio_to_uds_live.rs`: Unix-only live bridge that spins up a temp Unix socket listener, runs `codex stdio-to-uds <socket>`, sends `ping`, and prints the echoed `pong`.
- `cargo test -p unified-agent-api-codex` exercises env merging and non-destructive behavior (`runtime_api_*`, `app_runtime_*`, `app_runtime_pool_*` cover listing/prepare/start/stop without writing config or altering metadata).
- See `crates/codex/EXAMPLES.md` for one-to-one CLI parity examples, including `bundled_binary_home` to run Codex from an embedded binary with isolated state.

## Integration notes

- For a practical integration pattern in an async shell/orchestrator (Substrate), see `docs/integrations/substrate.md`.
