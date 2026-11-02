# Claude Code Rust Wrapper

Async wrapper around the Claude Code CLI (`claude`) focused on the headless `--print` flow.

- crates.io package: `unified-agent-api-claude-code`
- Rust library crate: `claude_code`

Design goals:
- Non-interactive first: all supported prompting APIs run with `--print`.
- No automatic downloads: this crate never installs Claude Code and never auto-updates it; update only runs when explicitly invoked.
- Parent environment is never mutated; env overrides apply per-spawn only.

## Quickstart

```rust,no_run
use claude_code::{ClaudeClient, ClaudeOutputFormat, ClaudePrintRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClaudeClient::builder().build();
    let req = ClaudePrintRequest::new("Hello from Rust")
        .output_format(ClaudeOutputFormat::Text);
    let res = client.print(req).await?;
    println!("{}", String::from_utf8_lossy(&res.output.stdout));
    Ok(())
}
```

## Examples (real CLI, no stubs)

Examples live under `crates/claude_code/examples/` and always spawn a real `claude` binary.
See `crates/claude_code/EXAMPLES.md` for a 1:1 mapping of wrapper examples to native CLI commands.

Common environment variables:
- `CLAUDE_BINARY`: path to the `claude` binary (otherwise uses repo-local `./claude-<target>` when present, or `claude` from PATH).
- `CLAUDE_HOME`: wrapper-managed “home root” for Claude CLI state/config (similar to `CODEX_HOME` for Codex).
- `CLAUDE_EXAMPLE_ISOLATED_HOME=1`: run examples with an isolated home under `target/`.
- `CLAUDE_EXAMPLE_LIVE=1`: enable examples that may require network/auth (e.g. `print_*`, `setup_token_flow`).
- `CLAUDE_EXAMPLE_ALLOW_MUTATION=1`: enable examples that may mutate local state (e.g. `update`, plugin/MCP management).
CI compiles examples but does not run them; authenticated/networked examples are live-gated for local runs.
See `crates/claude_code/EXAMPLES.md` for additional opt-in environment variables.

## Isolated Claude home (CODEX_HOME parity)

Codex supports `CODEX_HOME` as an app-scoped directory for config/auth/logs/history. Claude Code
does not have a single official `CLAUDE_HOME` knob, so this wrapper provides one:

- `ClaudeClientBuilder::claude_home(...)` redirects `HOME` + `XDG_*` (and Windows equivalents)
  per subprocess so the real `claude` CLI writes state beneath your chosen directory.
- `CLAUDE_HOME=/path/to/home` is also honored when `claude_home(...)` is not set.
- Optional seeding is opt-in:
  - `seed_profile_from(..., MinimalAuth)` copies a small set of CLI-relevant artifacts.
  - `seed_profile_from(..., FullProfile)` may copy large/sensitive app profile data (macOS:
    `~/Library/Application Support/Claude`); use only when needed.

See the `claude_home` example under `crates/claude_code/examples/`.
