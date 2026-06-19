# In-World Gateway Auth Handoff (Internals)

This document explains the current integrated auth handoff path used when Substrate launches `substrate-gateway` in-world.

This is an implementation note, not the operator contract. For stable external behavior, see:

- `docs/contracts/gateway/policy-evaluation.md`
- `docs/contracts/gateway/runtime-parity.md`
- `crates/gateway/docs/contracts/chatgpt-codex-auth-handoff-contract.md`
- `docs/reference/world/verification/gateway_auth_handoff.md`

## One-sentence model

Host-side policy and credential resolution produces a typed integrated-auth payload; `world-service` converts that payload into a validated JSON auth bundle, writes it into an inherited pipe, passes only `SUBSTRATE_LLM_AUTH_BUNDLE_FD` to the child, and the gateway reads that bundle once during in-world startup.

## Where each stage lives

### 1. Host-side source selection

The host-side gateway entrypoints build `GatewayIntegratedAuthPayloadV1` before calling into `world-service`.

Code pointers:

- `crates/shell/src/builtins/world_gateway.rs`
  - `resolve_integrated_auth_payload`
  - `resolve_cli_codex_integrated_auth`
  - `resolve_api_env_integrated_auth`
- `crates/shell/tests/world_gateway.rs`

Important current rules:

- allowlisted env auth material wins when fully present;
- host credential-file reads are fallback-only;
- partial env auth is an invalid integration error, not a merge case.

The Codex path in `resolve_cli_codex_integrated_auth` currently does this in order:

1. read `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN` and optional `...ACCOUNT_ID` from the host env;
2. enforce `llm.secrets.env_allowed` for the names that were present;
3. if the access token was present, stop there and build the payload from env;
4. if only the account id was present, fail closed as incomplete env auth;
5. otherwise fall back to `~/.codex/auth.json` only when `agents.host_credentials.read.allowed_backends` allows the selected placement-qualified Codex backend such as `cli:codex-host`.

That source-selection policy is documented contractually in `docs/contracts/gateway/policy-evaluation.md`.

### 2. Shared auth-bundle contract

The carrier contract lives in `crates/common/src/gateway_auth_bundle.rs`.

It defines:

- `GatewayAuthBundleV1`
- `SUBSTRATE_LLM_AUTH_BUNDLE_FD`
- backend-specific allowed and required field sets
- schema validation for the JSON payload written through the pipe

Important invariants:

- schema version is currently `1`;
- the bundle must declare a single `backend_id`;
- only backend-allowed field names are accepted;
- required fields are enforced per backend.

Examples:

- `cli:codex-host`
  - allowed: `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID`, `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN`
  - required: `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN`
- `cli:claude_code-host`
  - allowed/required: `SUBSTRATE_LLM_BACKEND_AUTH_API_ANTHROPIC_API_KEY`
- `api:openai`
  - allowed/required: `SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY`

## Launch flow inside `world-service`

The launch-side implementation lives in `crates/world-service/src/gateway_runtime.rs`.

Key functions:

- `prepare_gateway_auth_bundle_handoff`
- `create_inherited_auth_bundle_pipe`
- `resolve_integrated_auth_handoff`
- `resolve_codex_auth_handoff`
- `resolve_api_env_auth_handoff`
- `start_runtime`

### Launch sequence

1. `start_runtime` prepares the runtime directory and writes `config.toml`, `stdout.log`, and `stderr.log`.
2. `prepare_gateway_auth_bundle_handoff`:
   - converts the typed integrated-auth payload into a `GatewayAuthBundleV1`;
   - validates the bundle;
   - serializes it as JSON;
   - creates a pipe with `pipe2(O_CLOEXEC)`;
   - clears `FD_CLOEXEC` on the read side so the child can inherit it;
   - writes the JSON bundle into the write end and closes the write end.
3. `start_runtime` builds the child process environment.
4. `start_runtime` explicitly removes all `KNOWN_GATEWAY_AUTH_ENV_VARS` from the child env.
5. `start_runtime` sets only the pointer env `SUBSTRATE_LLM_AUTH_BUNDLE_FD=<read-fd>` plus non-secret gateway startup env such as mode/config-path.
6. `start_runtime` spawns `substrate-gateway --config <path> start`.
7. the parent drops its copy of the inherited read FD after spawn.

The raw secret env scrub list currently includes:

- `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID`
- `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN`
- `SUBSTRATE_LLM_BACKEND_AUTH_API_ANTHROPIC_API_KEY`
- `SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`
- `OPENAI_API_KEY`

That list is the reason `/proc/<pid>/environ` is a good manual proof surface for “pointer env present, raw secret env absent”.

## Consumer flow inside `substrate-gateway`

The startup-side consumer lives in `crates/gateway/src/server/mod.rs`.

Key functions:

- `IntegratedGatewayAuthContext::from_launch_mode`
- `read_gateway_auth_bundle_from_env`
- `take_auth_bundle_fd_env`

### Consumer sequence

1. startup selects integrated vs standalone mode from the launch posture;
2. in `in_world` mode, `read_gateway_auth_bundle_from_env` calls `take_auth_bundle_fd_env`;
3. `take_auth_bundle_fd_env` reads `SUBSTRATE_LLM_AUTH_BUNDLE_FD` and immediately calls `env::remove_var(SUBSTRATE_LLM_AUTH_BUNDLE_FD)`;
4. the gateway converts that FD into an owned file handle and reads the JSON bundle body;
5. the decoded `GatewayAuthBundleV1` is validated again on the gateway side;
6. the gateway derives `IntegratedGatewayAuthContext` from the bundle and applies it to config/provider setup.

Why the pointer env may still appear in `/proc/<pid>/environ`:

- the Linux `/proc/<pid>/environ` view is useful for launch-time proof, but it is not a reliable proof of post-startup mutation inside the running process;
- therefore, the one-time-consumption behavior is pinned by tests rather than by `/proc` inspection.

## Runtime artifacts and ownership

`world-service` stores managed gateway runtime artifacts under:

- `/run/substrate/substrate-gateway-runtime/<backend-id>/<world-id>/`

Current artifacts include:

- `runtime.json`
- `config.toml`
- `stdout.log`
- `stderr.log`
- `home/`

Current mode expectations in `gateway_runtime.rs`:

- runtime directories: `0750`
- runtime files: `0640`

The runtime manifest records:

- `world_id`
- `backend_id`
- `pid`
- `pid_start_time_ticks`
- `port`
- `runtime_dir`
- `config_path`
- `state`

That manifest is what the operator playbook uses to locate the exact running child process.

## Why the manual checks work

### `status --json`

This proves you are talking to the managed Substrate runtime surface rather than an upstream provider URL. It does **not** by itself prove the auth carrier.

### `/proc/<pid>/environ`

This is the best manual proof that the child was launched with:

- `SUBSTRATE_LLM_AUTH_BUNDLE_FD`; and
- no raw secret env vars from the known scrub list.

This directly reflects the `Command::env_remove(...)` and `.env(SUBSTRATE_LLM_AUTH_BUNDLE_FD, ...)` behavior in `start_runtime`.

### Env-only restart experiment

Disabling `agents.host_credentials.read.allowed_backends` while allowing the Codex env names isolates the **source** side of the handoff. If restart still succeeds, the host payload had to come from allowlisted env material rather than the host auth-file fallback.

### Regression tests

These pin the parts that manual inspection cannot prove reliably.

## Important proof surfaces

### Shell/source-selection tests

`crates/shell/tests/world_gateway.rs` includes source-precedence coverage such as:

- `world_gateway_sync_builds_integrated_auth_payload_from_host_auth_file`
- `world_gateway_status_builds_integrated_auth_payload_from_allowed_env_override`
- `world_gateway_status_prefers_allowed_env_auth_over_host_auth_file`
- `world_gateway_env_auth_blocked_by_policy_denies_without_file_fallback`

### World-service/carrier tests

`crates/world-service/tests/gateway_runtime_parity.rs` includes launcher/carrier coverage such as:

- `gateway_sync_makes_status_available_and_is_idempotent`
- `gateway_restart_uses_fresh_codex_bundle_after_rotation`
- `gateway_openai_sync_makes_status_available_and_is_idempotent`

The parity harness uses a fake gateway binary that asserts:

- `SUBSTRATE_LLM_AUTH_BUNDLE_FD` is present;
- forbidden secret env vars are absent;
- the received JSON bundle contains the expected backend-specific fields.

### Gateway/consumer tests

`crates/gateway/tests/openai_shared_parity.rs` includes consumer-side coverage such as:

- `integrated_gateway_startup_reads_cli_codex_bundle_once_from_pointer_env`
- `integrated_gateway_startup_reads_openai_bundle_without_secret_env_fallback`
- `integrated_gateway_startup_reads_claude_bundle_without_secret_env_fallback`

These tests pin:

- one-time pointer-env consumption;
- bundle decoding and validation;
- no dependency on raw secret env fallback inside the gateway process.

## Common operator gotchas reflected in the internals

### `find ... | head -n1` can select the wrong runtime

The runtime root is keyed by backend id and world id. If multiple backends are live, a naive `find ... | head -n1` can easily return `cli:claude_code-host` when you meant `cli:codex-host`.

Prefer matching by:

- the active loopback port from `substrate world gateway status --json`; or
- the backend-specific runtime subtree.

### `sudo` with shell redirection still fails

`sudo tr '\0' '\n' < /proc/<pid>/environ` fails because the redirect is opened by the current shell before `sudo` runs. That is why the operator playbook uses `sudo cat /proc/<pid>/environ | ...`.

### `~/.codex/auth.json` shape is not the carrier contract

The host-side Codex file is just one fallback source. The durable carrier contract between `world-service` and `substrate-gateway` is the validated auth bundle over `SUBSTRATE_LLM_AUTH_BUNDLE_FD`.
