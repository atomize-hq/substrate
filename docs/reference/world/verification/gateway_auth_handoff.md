# In-World Gateway Auth Handoff Verification

Use this playbook when you need to verify that a managed in-world `substrate-gateway` runtime is receiving integrated auth material through the auth-bundle FD handoff instead of raw secret-bearing child environment variables.

This is the operator-facing companion to:

- `docs/contracts/gateway/policy-evaluation.md`
- `crates/gateway/docs/contracts/chatgpt-codex-auth-handoff-contract.md`
- `docs/internals/world/gateway_auth_handoff.md`

## What this playbook proves

This procedure gives you three increasingly strong proof levels:

1. **Managed runtime proof**
   - `substrate world gateway status --json` shows the gateway is `available`, wired to loopback, and running with `placement_posture.execution == "in_world"`.
2. **Secure launch-carrier proof**
   - the launched gateway process has `SUBSTRATE_LLM_AUTH_BUNDLE_FD` in its launch environment;
   - the launched gateway process does **not** have raw secret env vars such as `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN`, `OPENAI_API_KEY`, or `ANTHROPIC_API_KEY`.
3. **Env-only source proof**
   - with host credential-file fallback disabled, the gateway still restarts successfully when auth is supplied through allowlisted host env vars.

## What this playbook does **not** prove by itself

`/proc/<pid>/environ` is useful for proving what was present at `exec` time. It is **not** a definitive proof that the running process later removed `SUBSTRATE_LLM_AUTH_BUNDLE_FD` with `env::remove_var(...)`.

For one-time consumption proof, run the regression tests listed in [Regression-test proof](#regression-test-proof).

## Prerequisites

- Linux host with `jq` installed.
- `sudo` access for reading `/proc/<pid>/environ` and `/run/substrate/substrate-gateway-runtime/...`.
- A provisioned world backend with a currently available gateway runtime.

Notes:

- On macOS, run the same commands **inside the Lima guest**.
- If you have multiple active gateway backends, do **not** use a plain `find ... | head -n1` and assume you found the right runtime. Match the runtime by port or by backend-specific directory.

## Step 1: prove you are hitting the managed in-world gateway

```bash
~/.substrate/bin/substrate world gateway status --json | jq
```

Expected signals:

- `.status == "available"`
- `.placement_posture.execution == "in_world"`
- `.client_wiring.openai_base_url` points to `http://127.0.0.1:<port>`
- `.identity_tuple.auth_authority` identifies the selected auth authority (for example `codex_subscription`)

This proves you are using the managed in-world runtime, but it does **not** yet prove which auth carrier was used.

## Step 2: locate the exact runtime manifest for the active gateway

### Recommended: match on the active status port

```bash
STATUS_JSON="$(mktemp)"
~/.substrate/bin/substrate world gateway status --json > "$STATUS_JSON"
PORT="$(jq -r '.client_wiring.openai_base_url | capture(":(?<port>[0-9]+)$").port' "$STATUS_JSON")"

RUNTIME_JSON="$({
  find /run/substrate/substrate-gateway-runtime -name runtime.json -print |
    while read -r path; do
      jq -e --argjson port "$PORT" '.port == $port' "$path" >/dev/null && echo "$path"
    done
} | head -n1)"

jq . "$RUNTIME_JSON"
PID="$(jq -r '.pid' "$RUNTIME_JSON")"
echo "$PID"
```

### Backend-specific alternative

If you are validating the host-scoped Codex runtime specifically, inspect the `cli:codex-host` subtree directly:

```bash
CODEX_RUNTIME_JSON="$(find /run/substrate/substrate-gateway-runtime/cli:codex-host -name runtime.json -print | head -n1)"
jq . "$CODEX_RUNTIME_JSON"
CODEX_PID="$(jq -r '.pid' "$CODEX_RUNTIME_JSON")"
echo "$CODEX_PID"

RUNTIME_JSON="$CODEX_RUNTIME_JSON"
PID="$CODEX_PID"
```

Expected manifest fields:

- `pid`
- `runtime_dir`
- `config_path`
- `state == "ready"`

## Step 3: inspect the launched gateway environment

Use `sudo cat`, not `sudo tr ... < /proc/$PID/environ`.

Why: shell redirection happens **before** `sudo` runs. If you redirect from your non-root shell, you will still get `Permission denied`.

```bash
sudo cat "/proc/$PID/environ" | tr '\0' '\n' | sort | grep '^SUBSTRATE_'
```

Then verify that raw secret env vars are absent:

```bash
sudo cat "/proc/$PID/environ" | tr '\0' '\n' | sort | \
  grep -E 'SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN|SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID|SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY|OPENAI_API_KEY|ANTHROPIC_API_KEY' \
  || echo 'OK: no raw secret env vars found in child env'
```

### Pass criteria

Expected **present** entries include:

- `SUBSTRATE_LLM_AUTH_BUNDLE_FD=<n>`
- `SUBSTRATE_LLM_GATEWAY_MODE=in_world`
- `SUBSTRATE_LLM_GATEWAY_CONFIG_PATH=...`
- optionally `SUBSTRATE_LLM_GATEWAY_DISABLE_TOKEN_PERSISTENCE=1`

Expected **absent** entries include:

- `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN`
- `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID`
- `SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY`
- `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`

If the pointer env is present and the raw secret env vars are absent, you have strong manual evidence that:

- the runtime is the managed in-world gateway;
- world-service launched it with the auth-bundle FD pointer;
- world-service scrubbed the known raw secret env vars from the child launch environment.

## Step 4: inspect managed runtime artifacts for leakage

```bash
RUNTIME_DIR="$(dirname "$RUNTIME_JSON")"
sudo ls -la "$RUNTIME_DIR"
sudo sed -n '1,200p' "$RUNTIME_DIR/config.toml"
sudo sed -n '1,200p' "$RUNTIME_DIR/stderr.log"
sudo sed -n '1,200p' "$RUNTIME_DIR/stdout.log"
```

Useful files:

- `runtime.json`
- `config.toml`
- `stdout.log`
- `stderr.log`

Expected result:

- runtime metadata and config are readable for authorized operators;
- raw auth token values do **not** appear in the config or logs.

## Step 5: stronger source proof with env-only auth

Use this when you want to prove not just the **carrier**, but also the **source** of the integrated auth payload.

Goal:

- disable host credential-file fallback;
- allow only the Codex env handoff names;
- restart the gateway in a short-lived shell with those env vars exported;
- confirm the restart succeeds and the child env is still scrubbed.

### 5.1 Back up the current global policy patch

```bash
POLICY_PATH="${SUBSTRATE_HOME:-$HOME/.substrate}/policy.yaml"
BACKUP_PATH="$(mktemp)"
[ -f "$POLICY_PATH" ] && cp "$POLICY_PATH" "$BACKUP_PATH" || : > "$BACKUP_PATH"

echo "policy backup: $BACKUP_PATH"
```

### 5.2 Disable host credential-file fallback and allow env-source auth

```bash
~/.substrate/bin/substrate policy global set 'agents.host_credentials.read.allowed_backends=[]'
~/.substrate/bin/substrate policy global set 'llm.secrets.env_allowed=["SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN","SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID"]'
```

### 5.3 Export auth in a temporary shell and restart the gateway

If you already know the token and account id, use a short-lived subshell:

```bash
OLD_PID="$PID"

(
  export SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN='<codex access token>'
  export SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID='<codex account id>'  # optional

  ~/.substrate/bin/substrate world gateway restart
  ~/.substrate/bin/substrate world gateway status --json | jq
)
```

If you want to extract values from `~/.codex/auth.json`, do **not** assume the token is always at `.access_token`.
First discover likely paths without printing the secret values themselves:

```bash
jq -r 'paths(scalars) | map(tostring) | join(".")' ~/.codex/auth.json | grep -E 'access_token|account_id|token'
```

Then substitute the real `jq` paths you found into a short-lived subshell.

### 5.4 Re-read the runtime manifest and verify the restart

```bash
STATUS_JSON="$(mktemp)"
~/.substrate/bin/substrate world gateway status --json > "$STATUS_JSON"
PORT="$(jq -r '.client_wiring.openai_base_url | capture(":(?<port>[0-9]+)$").port' "$STATUS_JSON")"

RUNTIME_JSON="$({
  find /run/substrate/substrate-gateway-runtime -name runtime.json -print |
    while read -r path; do
      jq -e --argjson port "$PORT" '.port == $port' "$path" >/dev/null && echo "$path"
    done
} | head -n1)"

NEW_PID="$(jq -r '.pid' "$RUNTIME_JSON")"
echo "restart changed pid? $OLD_PID -> $NEW_PID"
jq . "$RUNTIME_JSON"
```

Then repeat the env inspection from [Step 3](#step-3-inspect-the-launched-gateway-environment).

### 5.5 Interpret the result

Strong pass:

- `substrate world gateway restart` succeeds;
- `status --json` still reports `execution == "in_world"`;
- the PID changes;
- raw secret env vars are still absent from the child process environment.

What that means:

- the source was the allowlisted host env vars, not the host auth-file fallback;
- the carrier into the child remained the auth-bundle FD handoff;
- the child still did not receive raw secret env vars.

### 5.6 Restore the original global policy patch

```bash
if [ -s "$BACKUP_PATH" ]; then
  cp "$BACKUP_PATH" "$POLICY_PATH"
else
  rm -f "$POLICY_PATH"
fi

rm -f "$BACKUP_PATH"
```

## Regression-test proof

If you need proof stronger than runtime inspection, run the pinned regression tests.

### Gateway consumer: pointer env is consumed once

```bash
cargo test -p substrate-gateway integrated_gateway_startup_reads_cli_codex_bundle_once_from_pointer_env -- --nocapture
```

This proves the in-world gateway:

- reads the FD pointer env;
- consumes it once;
- fails the second read after the pointer env is removed.

### World-service launcher: sync is idempotent and restart redelivers auth

```bash
cargo test -p world-service gateway_sync_makes_status_available_and_is_idempotent -- --nocapture
cargo test -p world-service gateway_restart_uses_fresh_codex_bundle_after_rotation -- --nocapture
```

These prove the launcher:

- can bring the runtime to `available`;
- does not relaunch on an idempotent `sync`;
- redelivers a fresh auth bundle on restart.

### Optional backend-specific secret-env fallback checks

```bash
cargo test -p substrate-gateway integrated_gateway_startup_reads_openai_bundle_without_secret_env_fallback -- --nocapture
cargo test -p substrate-gateway integrated_gateway_startup_reads_claude_bundle_without_secret_env_fallback -- --nocapture
```

## Troubleshooting

### `Permission denied` reading `/proc/<pid>/environ`

Use:

```bash
sudo cat "/proc/$PID/environ" | tr '\0' '\n' | sort
```

Do **not** use:

```bash
sudo tr '\0' '\n' < "/proc/$PID/environ"
```

The second form still opens `/proc/$PID/environ` from your current non-root shell.

### You inspected the wrong runtime

If multiple backends are active, `find /run/substrate/substrate-gateway-runtime -name runtime.json | head -n1` can point at the wrong runtime.

Match by:

- active port from `status --json`; or
- backend-specific directory such as `.../cli:codex-host/...`.

### The PID did not change after restart

That usually means one of these happened:

- the restart command failed before relaunch;
- the subshell exited before exporting a valid token;
- you re-read the wrong runtime manifest.

### `~/.codex/auth.json` did not contain `.access_token`

That file shape can vary. Discover the actual key path first:

```bash
jq -r 'paths(scalars) | map(tostring) | join(".")' ~/.codex/auth.json | grep -E 'access_token|account_id|token'
```

Then use the discovered path in your extraction command or export the values manually in a throwaway shell.
