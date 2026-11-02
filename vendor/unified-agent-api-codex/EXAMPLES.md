# Codex Wrapper Examples vs. Native CLI

Every example under `crates/codex/examples/` maps to a `codex` CLI invocation. Wrapper calls (`cargo run -p unified-agent-api-codex --example ...`) run with safe defaults: `--skip-git-repo-check`, temp working dirs unless overridden, 120s timeout, ANSI color disabled, and `RUST_LOG=error` unless set. Select the binary with `CODEX_BINARY` or `.binary(...)`; use `resolve_bundled_binary(...)` when you ship a pinned bundle, set `CODEX_HOME` to keep config/auth/history/logs under an app-scoped directory, and call `CodexHomeLayout::seed_auth_from(...)` to copy auth.json/.credentials.json from a trusted seed home. Examples labeled `--sample` print mocked data (covering `thread/turn/item` events and MCP/app-server notifications) when you do not have a binary handy; streaming/resume/apply fixtures live in `crates/codex/examples/fixtures/*` so docs and samples stay aligned.

The Cargo package name is `unified-agent-api-codex`; the Rust library crate remains `codex`.

## Basics

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `cargo run -p unified-agent-api-codex --example send_prompt -- "List Rust toolchain commands"` | `codex exec "List Rust toolchain commands" --skip-git-repo-check` | Baseline prompt with default timeout/temp dir. |
| `cargo run -p unified-agent-api-codex --example timeout -- "List long-running tasks"` | `codex exec "List long-running tasks" --skip-git-repo-check --timeout 30` | Forces a 30‑second timeout. |
| `cargo run -p unified-agent-api-codex --example timeout_zero -- "Stream until completion"` | `codex exec "Stream until completion" --skip-git-repo-check --timeout 0` | Disables the wrapper timeout. |
| `cargo run -p unified-agent-api-codex --example working_dir -- "C:\\path\\to\\repo" "List files here"` | `codex exec "List files here" --skip-git-repo-check --cd "C:\\path\\to\\repo"` | Run inside a specific directory. |
| `cargo run -p unified-agent-api-codex --example working_dir_json -- "C:\\path\\to\\repo" "Summarize repo status"` | `echo "Summarize repo status" \| codex exec --skip-git-repo-check --json --cd "C:\\path\\to\\repo"` | Combines working dir override with JSON streaming. |
| `cargo run -p unified-agent-api-codex --example select_model -- gpt-5-codex -- "Explain rustfmt defaults"` | `codex exec "Explain rustfmt defaults" --skip-git-repo-check --model gpt-5-codex` | Picks a specific model. |
| `cargo run -p unified-agent-api-codex --example color_always -- "Show colorful output"` | `codex exec "Show colorful output" --skip-git-repo-check --color always` | Forces ANSI color codes. |
| `cargo run -p unified-agent-api-codex --example send_prompt --color never -- "Show monochrome"` | `codex exec "Show monochrome" --skip-git-repo-check --color never` | Color example also works for `auto`/`never`. |
| `cargo run -p unified-agent-api-codex --example image_json -- "C:\\path\\to\\mockup.png" "Describe the screenshot"` | `echo "Describe the screenshot" \| codex exec --skip-git-repo-check --json --image "C:\\path\\to\\mockup.png"` | Attach an image while streaming JSON quietly. |
| `cargo run -p unified-agent-api-codex --example quiet -- "Run without tool noise"` | `codex exec "Run without tool noise" --skip-git-repo-check --quiet` | Suppress stderr mirroring. |
| `cargo run -p unified-agent-api-codex --example no_stdout_mirror -- "Stream quietly"` | `codex exec "Stream quietly" --skip-git-repo-check > out.txt` | Disable stdout mirroring to capture output yourself. |
| `cargo run -p unified-agent-api-codex --example cli_overrides -- "Draft release notes"` | `codex exec "Draft release notes" --skip-git-repo-check --ask-for-approval on-request --sandbox workspace-write --local-provider ollama --oss --enable builder-toggle --disable legacy-flow --config model_verbosity=high --config features.search=true --config model_reasoning_effort=low --enable request-toggle --search [--cd /tmp/repo]` | CLI parity example showing builder safety/config overrides, feature toggles, and per-request search/CD tweaks. |
| `cargo run -p unified-agent-api-codex --example run_sandbox -- linux --full-auto -- echo "hello from sandbox"` | `codex sandbox linux --full-auto -- echo "hello from sandbox"` | Wraps the sandbox helper with platform selection (defaults to host OS), macOS `--log-denials`, and captured stdout/stderr + exit. |
| `cargo run -p unified-agent-api-codex --example features_toggle -- enable unified_exec` | `codex features enable unified_exec` | Enables a named feature key. |
| `cargo run -p unified-agent-api-codex --example features_toggle -- disable unified_exec` | `codex features disable unified_exec` | Disables a named feature key. |
| `cargo run -p unified-agent-api-codex --example debug_cmd -- app-server send-message-v2 "hello"` | `codex debug app-server send-message-v2 "hello"` | Calls the debug app-server shim for smoke testing. |
| `cargo run -p unified-agent-api-codex --example debug_cmd -- models --bundled` | `codex debug models --bundled` | Prints the bundled model inventory when the binary exposes it. |
| `cargo run -p unified-agent-api-codex --example debug_cmd -- prompt-input --image ./diagram.png "summarize this prompt payload"` | `codex debug prompt-input --image ./diagram.png "summarize this prompt payload"` | Echoes prompt-input normalization with optional image attachments. |
| `cargo run -p unified-agent-api-codex --example plugin_cmd -- marketplace add github.com/acme/codex-marketplace --ref main --sparse packages/demo` | `codex plugin marketplace add --ref main --sparse packages/demo github.com/acme/codex-marketplace` | Exercises the 0.125.0 plugin marketplace wrapper family. |
| `cargo run -p unified-agent-api-codex --example update` | `codex update` | Invokes the native self-update flow through the wrapper surface. |

## Binary & CODEX_HOME

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `$env:CODEX_BINARY="C:\\bin\\codex-nightly.exe"; cargo run -p unified-agent-api-codex --example env_binary -- "Nightly sanity check"` | `C:\\bin\\codex-nightly.exe exec "Nightly sanity check" --skip-git-repo-check` | Honors `CODEX_BINARY` override; default fallback remains `codex` on `PATH`. |
| `CODEX_BUNDLE_ROOT="$HOME/.myapp/codex-bin" CODEX_BUNDLE_VERSION="1.2.3" cargo run -p unified-agent-api-codex --example bundled_binary -- "Quick health check"` | `<bundle_root>/<platform>/1.2.3/codex exec "Quick health check" --skip-git-repo-check` | Uses `resolve_bundled_binary` to pin a versioned app bundle (default platform label); no fallback to `CODEX_BINARY`/`PATH`. |
| `CODEX_BUNDLE_ROOT="$HOME/.myapp/codex-bin" CODEX_BUNDLE_VERSION="1.2.3" CODEX_PROJECT_HOME="$HOME/.myapp/codex-homes/demo" [CODEX_AUTH_SEED_HOME="$HOME/.myapp/codex-auth-seed"] cargo run -p unified-agent-api-codex --example bundled_binary_home -- "Health check prompt"` | `CODEX_HOME="$HOME/.myapp/codex-homes/demo" <bundle_root>/<platform>/1.2.3/codex exec "Health check prompt" --skip-git-repo-check` | Recommended bundled+isolated flow: resolve the pinned binary, pick a per-project `CODEX_HOME`, seed `auth.json`/`.credentials.json` via `CodexHomeLayout::seed_auth_from` (options to require missing files), then use `AuthSessionHelper` for login under the isolated home. |
| `CODEX_HOME=/tmp/codex-demo cargo run -p unified-agent-api-codex --example codex_home -- "Show CODEX_HOME contents"` | `CODEX_HOME=/tmp/codex-demo codex exec "Show CODEX_HOME contents" --skip-git-repo-check` | App-scoped `CODEX_HOME` showing config/auth/history/log paths (pair with a bundled binary or `CODEX_BINARY`). |

## Streaming & Logging

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `cargo run -p unified-agent-api-codex --example json_stream -- "Summarize repo status"` | `echo "Summarize repo status" \| codex exec --skip-git-repo-check --json` | Enable JSONL streaming; prompt is piped via stdin. |
| `cargo run -p unified-agent-api-codex --example stream_events -- "Summarize repo status"` | `echo "Summarize repo status" \| codex exec --skip-git-repo-check --json --timeout 0` | Typed consumer for `thread/turn/item` events (thread/turn IDs included, item created/updated for agent_message, reasoning, command_execution, file_change, mcp_tool_call, web_search, todo_list) plus `turn.failed`; `--sample` replays bundled events. |
| `cargo run -p unified-agent-api-codex --example stream_last_message -- "Summarize repo status"` | `codex exec --skip-git-repo-check --json --output-last-message <path> --output-schema <path> <<<"Summarize repo status"` | Reads `--output-last-message` + `--output-schema` files; falls back to samples when the binary does not support those flags (e.g., 0.61.x). |
| `CODEX_LOG_PATH=/tmp/codex.log cargo run -p unified-agent-api-codex --example stream_with_log -- "Stream with logging"` | `echo "Stream with logging" \| codex exec --skip-git-repo-check --json` | Mirrors stdout and tees JSONL events to `CODEX_LOG_PATH` (or uses sample events with IDs/status). |
| `cargo run -p unified-agent-api-codex --example parse_rollout_jsonl -- --path <rollout.jsonl>` | `cat <rollout.jsonl>` | Offline parser for saved rollout JSONL logs (`rollout-*.jsonl` under `$CODEX_HOME/sessions`). |
| `cargo run -p unified-agent-api-codex --example filter_rollout_event_msg -- --path <rollout.jsonl> --event-msg-type token_count --response-item-type message` | `cat <rollout.jsonl>` | Offline filter for rollout `event_msg` and `response_item` records by payload type; supports multiple filters and `--list-types`. |

## Resume & Apply/Diff

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `CODEX_CONVERSATION_ID=abc cargo run -p unified-agent-api-codex --example resume_apply` | `codex exec resume --last` then (optionally) `codex apply <task-id>` | Resumes the most recent session (or `--resume-id <id>`) over stdio; if you pass `--task-id`/`CODEX_TASK_ID`, it runs `codex apply` afterward. `--sample` replays bundled resume/apply fixtures; `--no-apply` skips the apply step. |

## MCP + App Server

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `cargo run -p unified-agent-api-codex --example mcp_codex_flow -- "Draft a plan" ["Tighten scope"]` | `codex mcp-server --stdio` then `tools/call` with `name=codex` (or `codex-reply`) | Typed `codex::mcp` helper that streams `codex/event`, supports `$ /cancelRequest`, and chains a follow-up when the first call returns a conversation ID; sends `clientInfo` + `protocolVersion` and uses `tools/call` for 0.61.x MCP compatibility; gate with `feature_detection` if the binary lacks MCP endpoints. |
| `cargo run -p unified-agent-api-codex --example mcp_codex_tool -- "Summarize repo status"` | `codex mcp-server` then send `tools/codex` JSON-RPC call | Streams codex tool notifications (approval/task_complete); `--sample` and optional `CODEX_HOME` for isolation. |
| `CODEX_CONVERSATION_ID=abc123 cargo run -p unified-agent-api-codex --example mcp_codex_reply -- "Continue the prior run"` | `codex mcp-server` then call `tools/codex-reply` with `conversationId=abc123` | Continue a session via `codex-reply`; needs `CODEX_CONVERSATION_ID`/first arg (use the `session_id` from `session_configured`), and the session must still be active in the same `mcp-server` process (0.61.0 does not rehydrate from disk). Use `codex exec resume` or app-server `thread/resume` for cross-process resumes. `--sample` available. |
| `cargo run -p unified-agent-api-codex --example app_server_turns -- "Draft a release note" [thread-id]` | `codex app-server` then `thread/start` or `thread/resume` plus `turn/start` (optional `turn/interrupt`) | Uses the `codex::mcp` app-server client to stream items and task_complete notices, optionally resuming a thread and sending `turn/interrupt` after a delay; pair with `feature_detection` if the binary omits app-server support. |
| `cargo run -p unified-agent-api-codex --example app_server_thread_turn -- "Draft a release note"` | `codex app-server` then send `thread/start` and `turn/start` | App-server thread/turn notifications; supports `--sample` and optional `CODEX_HOME` for state isolation. |
| `cargo run -p unified-agent-api-codex --example app_server_codegen -- ts ./gen/app --prettier ./node_modules/.bin/prettier` | `codex app-server generate-ts --out ./gen/app --prettier ./node_modules/.bin/prettier` | Refresh TypeScript bindings (or `json ./gen/app` for schemas) with shared config/profile flags; ensures the output directory exists first and surfaces non-zero exits as `CodexError::NonZeroExit`. |
| `cargo run -p unified-agent-api-codex --example app_server_codegen -- json ./gen/app --experimental` | `codex app-server generate-json-schema --out ./gen/app --experimental` | Requests experimental codegen surfaces when supported by the binary. |
| `cargo run -p unified-agent-api-codex --example server_helpers -- app-proxy /tmp/codex-app.sock` | `codex app-server proxy --sock /tmp/codex-app.sock` | Starts the app-server proxy helper under host-managed stdio, prints the child pid, then shuts it down. |

## Proxies & Bridges

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `cargo run -p unified-agent-api-codex --example responses_api_proxy` | `echo "$OPENAI_API_KEY" \| codex responses-api-proxy [--port <PORT>] [--server-info <FILE>] [--http-shutdown] [--upstream-url <URL>]` | Starts the API-key-injecting responses proxy with piped stdin; requires `OPENAI_API_KEY`/`CODEX_API_KEY` for live runs, otherwise the example falls back to a stub `--sample` server-info output. Polls `--server-info` for `{port,pid}` when requested. |
| `cargo run -p unified-agent-api-codex --example server_helpers -- exec-server 127.0.0.1:7777` | `codex exec-server --listen 127.0.0.1:7777` | Starts the exec-server helper under host-managed stdio, prints the child pid, then shuts it down. |
| `cargo run -p unified-agent-api-codex --example stdio_to_uds_live` | `codex stdio-to-uds <temp.sock>` | Unix-only live demo: spawns a temp UDS listener, bridges via `codex stdio-to-uds`, sends `ping`, and prints the echoed `pong`; set `CODEX_BINARY` to pick a specific CLI. |

## Capabilities

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `cargo run -p unified-agent-api-codex --example capability_snapshot -- ./codex ./codex-capabilities.json auto` | `codex --version && codex features list --json` | Persists capability snapshots with fingerprint checks, refresh/backoff guidance, and bypass mode for FUSE/overlay paths. |

## Feature Detection

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `cargo run -p unified-agent-api-codex --example feature_detection` | `codex --version` and `codex features list` | Probes version + feature list (per-binary cache), gates streaming/log-tee/resume/apply/artifact flags, and emits upgrade advisories; falls back to sample data. |

## Ingestion harness

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `cargo run -p ingestion --example ingest_to_codex -- --instructions "Summarize the documents" --model gpt-5-codex --json --include-prompt --image "C:\\Docs\\mockup.png" C:\\Docs\\spec.pdf` | `codex exec --skip-git-repo-check --json --model gpt-5-codex --image "C:\\Docs\\mockup.png" "<constructed prompt covering spec.pdf>"` | Builds a multi-document prompt before calling `codex exec`; supports images and optional prompt echo. |

## Capability TTL helper
`capability_cache_ttl_decision` provides a TTL/backoff wrapper around cached snapshots so hosts know when to reuse, refresh, or bypass:

```rust
use codex::{capability_cache_entry, capability_cache_ttl_decision, CapabilityCachePolicy, CodexClient};
use std::time::{Duration, SystemTime};

async fn decide(client: &CodexClient, binary: &std::path::Path) {
    let cached = capability_cache_entry(binary);
    let decision = capability_cache_ttl_decision(cached.as_ref(), Duration::from_secs(300), SystemTime::now());

    let capabilities = if let Some(snapshot) = cached.filter(|_| !decision.should_probe) {
        snapshot
    } else {
        client.probe_capabilities_with_policy(decision.policy).await
    };

    if decision.policy == CapabilityCachePolicy::Bypass {
        // Metadata missing (FUSE/overlay); stretch the TTL toward 10-15 minutes to reduce probe churn.
    }

    let _ = capabilities;
}
```
- `Refresh` is recommended for hot-swaps that reuse the same path even when fingerprints look unchanged.
- `Bypass` is returned when metadata is missing; avoid cache writes and apply a growing TTL/backoff to avoid hammering the binary.

## Discovering `CODEX_HOME` layout

Use `CodexHomeLayout` to inspect where Codex stores config, credentials, history, conversations, and logs when you set an app-scoped `CODEX_HOME`:

```rust
use codex::CodexHomeLayout;

let layout = CodexHomeLayout::new("/apps/myhub/codex");
println!("Config: {}", layout.config_path().display());
println!("History: {}", layout.history_path().display());
println!("Conversations: {}", layout.conversations_dir().display());
println!("Logs: {}", layout.logs_dir().display());

// Optional: create the CODEX_HOME directories yourself before spawning Codex.
layout.materialize(true).expect("failed to prepare CODEX_HOME");
```

Use these pairs as a checklist when validating parity between the Rust wrapper and the raw Codex CLI.
