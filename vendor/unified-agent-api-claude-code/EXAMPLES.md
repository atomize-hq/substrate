# Claude Code Wrapper Examples vs. Native CLI

Every example under `crates/claude_code/examples/` spawns a real `claude` CLI binary (no stubs). The examples are designed to be copy/paste friendly and to map 1:1 to a native CLI invocation.

The Cargo package name is `unified-agent-api-claude-code`; the Rust library crate remains `claude_code`.

## Common environment variables

- `CLAUDE_BINARY`: Path to the `claude` binary. If unset, examples fall back to a repo-local `./claude-<target>` when present, else `claude` from `PATH`.
- `CLAUDE_HOME`: Wrapper-managed “home root” for Claude CLI state/config (redirects `HOME` + `XDG_*` per subprocess).
- `CLAUDE_EXAMPLE_ISOLATED_HOME=1`: Runs examples with an isolated Claude home under `target/` to avoid touching your real config.
- `CLAUDE_EXAMPLE_LIVE=1`: Enables examples that may require network/auth (e.g. `print_*`, `setup_token_flow`).
- `CLAUDE_EXAMPLE_ALLOW_MUTATION=1`: Enables examples that may mutate local state (e.g. `update`, `plugin_manage`, `mcp_manage`).
- `CLAUDE_SETUP_TOKEN_CODE`: Optional shortcut for `setup_token_flow` to submit the code without prompting.
- `CLAUDE_EXAMPLE_ALLOW_CHROME=1`: Enables `--chrome` / `--no-chrome` examples.
- `CLAUDE_EXAMPLE_ALLOW_IDE=1`: Enables `--ide` example.
- `CLAUDE_EXAMPLE_FROM_PR`: Value for `--from-pr [value]` example.
- `CLAUDE_EXAMPLE_FILE_SPECS`: Space-separated `file_id:relative_path` specs for `--file`.
- `CLAUDE_EXAMPLE_PLUGIN_DIRS`: Space-separated plugin directory paths for `--plugin-dir`.
- `CLAUDE_EXAMPLE_AGENTS_JSON`: JSON object for `--agents`.
- `CLAUDE_EXAMPLE_AGENT`: Agent name for `--agent`.
- `CLAUDE_EXAMPLE_BETAS`: Space-separated beta tokens for `--betas`.
- `CLAUDE_EXAMPLE_MCP_CONFIG`: MCP config file path or JSON string for `--mcp-config`.
- `CLAUDE_EXAMPLE_STREAM_JSON_INPUT`: Stream-json input payload for the `--input-format stream-json` example.

## Basics

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `cargo run -p unified-agent-api-claude-code --example help_version` | `claude --help` and `claude --version` | Safe, non-auth, non-mutating. |
| `cargo run -p unified-agent-api-claude-code --example doctor` | `claude doctor` | Safe, non-auth, non-mutating. |
| `cargo run -p unified-agent-api-claude-code --example claude_home` | `claude --version` | Demonstrates wrapper-managed `CLAUDE_HOME` (isolated CLI state). |
| `cargo run -p unified-agent-api-claude-code --example env_binary` | `claude --version` | Shows how examples resolve the `claude` binary. |
| `cargo run -p unified-agent-api-claude-code --example mirror_output` | `claude --version` | Demonstrates wrapper stdout/stderr mirroring options. |

## Print basics (`--print`)

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `cargo run -p unified-agent-api-claude-code --example print_text -- "hello"` | `claude --print "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1` (auth/network). |
| `cargo run -p unified-agent-api-claude-code --example print_json -- "hello"` | `claude --print --output-format json "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`; prints prettified JSON. |
| `cargo run -p unified-agent-api-claude-code --example print_json_schema -- "hello"` | `claude --print --output-format json --json-schema ... "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`; demonstrates structured output validation. |
| `cargo run -p unified-agent-api-claude-code --example print_stream_json -- "hello"` | `claude --print --output-format stream-json "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`; demonstrates parsing `stream-json`. |
| `cargo run -p unified-agent-api-claude-code --example print_stream_json_extract_text -- "hello"` | `claude --print --output-format stream-json "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`; shows how to reconstruct assistant text. |
| `cargo run -p unified-agent-api-claude-code --example print_stdin_text -- "hello from stdin"` | `echo "..." \| claude --print` | Requires `CLAUDE_EXAMPLE_LIVE=1`; demonstrates supplying input via stdin bytes. |

## Stream-JSON

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `cargo run -p unified-agent-api-claude-code --example print_session_id -- "hello"` | `claude --print --output-format stream-json "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`; prints the discovered `session_id`. |
| `cargo run -p unified-agent-api-claude-code --example print_include_partial_messages -- "hello"` | `claude --print --output-format stream-json --include-partial-messages "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`; prints a type-count summary. |
| `cargo run -p unified-agent-api-claude-code --example print_stream_json_replay_user_messages` | `claude --print --input-format stream-json --output-format stream-json --replay-user-messages` | Requires `CLAUDE_EXAMPLE_LIVE=1`; opt-in via `CLAUDE_EXAMPLE_STREAM_JSON_INPUT`. |

## Tools & permissions

Most live examples set `--dangerously-skip-permissions` by default to avoid headless permission prompts/hangs.

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `cargo run -p unified-agent-api-claude-code --example print_tools_safe_bash` | `claude --print --tools ... --allowedTools ... --add-dir ...` | Requires `CLAUDE_EXAMPLE_LIVE=1`; runs in a temp working dir and restricts tool access to it. |
| `cargo run -p unified-agent-api-claude-code --example print_tools_disallowed` | `claude --print --disallowedTools ...` | Requires `CLAUDE_EXAMPLE_LIVE=1`; demonstrates deny list behavior. |
| `cargo run -p unified-agent-api-claude-code --example print_allow_dangerously_skip_permissions -- "hello"` | `claude --print --allow-dangerously-skip-permissions "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`; demonstrates the opt-in bypass toggle. |

## Multi-turn & sessions

These examples are intentionally run inside a temp working directory so session persistence doesn’t touch your repo checkout.

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `cargo run -p unified-agent-api-claude-code --example multi_turn_resume` | `claude --print --session-id <uuid> ...` then `claude --print --resume <uuid> ...` | Requires `CLAUDE_EXAMPLE_LIVE=1`; demonstrates 2 turns via explicit session ID then resume. |
| `cargo run -p unified-agent-api-claude-code --example multi_turn_fork` | `claude --print --resume <uuid> --fork-session ...` | Requires `CLAUDE_EXAMPLE_LIVE=1`; best-effort check that a new session is created. |
| `cargo run -p unified-agent-api-claude-code --example multi_turn_continue` | `claude --print ...` then `claude --print --continue ...` | Requires `CLAUDE_EXAMPLE_LIVE=1`; continues most recent session in the working dir. |
| `cargo run -p unified-agent-api-claude-code --example multi_turn_no_session_persistence` | `claude --print --no-session-persistence ...` | Requires `CLAUDE_EXAMPLE_LIVE=1`; best-effort demonstration of “cannot resume”. |
| `cargo run -p unified-agent-api-claude-code --example print_from_pr` | `claude --print --from-pr [value]` | Requires `CLAUDE_EXAMPLE_LIVE=1`; opt-in via `CLAUDE_EXAMPLE_FROM_PR`. |

## Debugging

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `cargo run -p unified-agent-api-claude-code --example print_debug_file -- "hello"` | `claude --print --debug --debug-file ... "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`; prints a preview of the debug file. |

## Settings / model / config

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `cargo run -p unified-agent-api-claude-code --example print_settings_sources -- "hello"` | `claude --print --setting-sources ... --settings ... "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`; demonstrates settings sources + inline JSON settings. |
| `cargo run -p unified-agent-api-claude-code --example print_model_fallback_budget -- "hello"` | `claude --print --model ... --fallback-model ... --max-budget-usd ... "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`; model/fallback/budget knobs. |
| `cargo run -p unified-agent-api-claude-code --example print_mcp_config -- "hello"` | `claude --print --mcp-config ... [--strict-mcp-config] [--mcp-debug] "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`; opt-in via `CLAUDE_EXAMPLE_MCP_CONFIG`. |
| `cargo run -p unified-agent-api-claude-code --example print_system_prompts -- "hello"` | `claude --print --system-prompt ... --append-system-prompt ... "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`; system prompt composition. |
| `cargo run -p unified-agent-api-claude-code --example print_disable_slash_commands -- "hello"` | `claude --print --disable-slash-commands "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`. |
| `cargo run -p unified-agent-api-claude-code --example print_verbose -- "hello"` | `claude --print --verbose "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`. |
| `cargo run -p unified-agent-api-claude-code --example print_chrome_flags -- chrome -- "hello"` | `claude --print --chrome "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`; opt-in via `CLAUDE_EXAMPLE_ALLOW_CHROME`. |
| `cargo run -p unified-agent-api-claude-code --example print_ide -- "hello"` | `claude --print --ide "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`; opt-in via `CLAUDE_EXAMPLE_ALLOW_IDE`. |
| `cargo run -p unified-agent-api-claude-code --example print_plugin_dirs -- "hello"` | `claude --print --plugin-dir ... "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`; opt-in via `CLAUDE_EXAMPLE_PLUGIN_DIRS`. |
| `cargo run -p unified-agent-api-claude-code --example print_file_resources -- "hello"` | `claude --print --file ... "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`; opt-in via `CLAUDE_EXAMPLE_FILE_SPECS`. |
| `cargo run -p unified-agent-api-claude-code --example print_agents -- "hello"` | `claude --print --agent ...` / `--agents ...` | Requires `CLAUDE_EXAMPLE_LIVE=1`; opt-in via `CLAUDE_EXAMPLE_AGENT` / `CLAUDE_EXAMPLE_AGENTS_JSON`. |
| `cargo run -p unified-agent-api-claude-code --example print_betas -- "hello"` | `claude --print --betas ... "hello"` | Requires `CLAUDE_EXAMPLE_LIVE=1`; opt-in via `CLAUDE_EXAMPLE_BETAS`. |

## Auth & setup-token

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `cargo run -p unified-agent-api-claude-code --example setup_token_flow` | `claude setup-token` | Requires `CLAUDE_EXAMPLE_LIVE=1`; interactive auth flow; submits code if prompted. |

## MCP / plugins / update (mutation-gated)

| Wrapper example | Native command | Notes |
| --- | --- | --- |
| `cargo run -p unified-agent-api-claude-code --example update` | `claude update` | Mutating; requires `CLAUDE_EXAMPLE_ALLOW_MUTATION=1`. |
| `cargo run -p unified-agent-api-claude-code --example mcp_list` | `claude mcp list` and `claude mcp reset-project-choices` | Safe-ish but can affect local MCP state; see source for behavior. |
| `cargo run -p unified-agent-api-claude-code --example mcp_manage -- <subcommand>` | `claude mcp ...` | Mutating; requires `CLAUDE_EXAMPLE_ALLOW_MUTATION=1`. Platform support may vary. |
| `cargo run -p unified-agent-api-claude-code --example plugin_manage -- <subcommand>` | `claude plugin ...` | Mutating; requires `CLAUDE_EXAMPLE_ALLOW_MUTATION=1`. Platform support may vary. |

## Drift prevention (coverage gates)

- Command coverage: `crates/claude_code/examples/examples_manifest.json` and `crates/claude_code/tests/examples_manifest.rs`.
  - Ensures every `CoverageLevel::Explicit` command path (excluding root) has at least one example.
- Print-flow coverage: `crates/claude_code/examples/print_flows_manifest.json` and `crates/claude_code/tests/print_flows_manifest.rs`.
  - Ensures multi-turn/session + stream-json flows keep examples as wrapper capabilities evolve.
- Print-flag coverage: `crates/claude_code/examples/print_flags_manifest.json` and `crates/claude_code/tests/print_flags_manifest.rs`.
  - Ensures every explicit root flag declared by the wrapper has at least one example mapping.
