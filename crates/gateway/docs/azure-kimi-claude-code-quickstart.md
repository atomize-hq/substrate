# Azure Kimi + Claude Code Quick Start

This is the shortest working path for running `substrate-gateway` locally and pointing Claude Code at Azure-hosted Kimi deployments.

It assumes:

- you have two Azure deployments ready, one for your default Kimi model and one for the reasoning / thinking model
- you have the Azure OpenAI v1 base URL for that resource
- you already have Claude Code installed locally

## 1. Build the gateway

From the repo root:

```bash
cargo build --manifest-path gateway/Cargo.toml -p substrate-gateway --release
```

The binary will be at:

```text
gateway/target/release/substrate-gateway
```

## 2. Export your Azure credentials

Use the Azure OpenAI v1 base URL, not the old `/chat/completions` request path:

```bash
export AZURE_KIMI_API_KEY="your-azure-api-key"
export AZURE_KIMI_ENDPOINT="https://YOUR-RESOURCE-NAME.openai.azure.com/openai/v1"
```

If your existing endpoint still includes `/openai/v1/chat/completions`, the gateway trims that suffix automatically, but the `/openai/v1` base URL is the preferred shape.

## 3. Create the gateway config

Copy the Azure example into the default runtime location:

```bash
mkdir -p ~/.substrate-gateway
cp gateway/config/default.example.toml ~/.substrate-gateway/config.toml
```

Edit `~/.substrate-gateway/config.toml` and replace the placeholder deployment names in `actual_model`:

```toml
[[models]]
name = "substrate-think"

[[models.mappings]]
actual_model = "your-kimi-thinking-deployment"
priority = 1
provider = "azure-kimi"

[[models]]
name = "substrate-default"

[[models.mappings]]
actual_model = "your-kimi-default-deployment"
priority = 1
provider = "azure-kimi"
```

The rest of the example is already set up for Azure Kimi:

- provider name: `azure-kimi`
- provider type: `azure-openai`
- auth env vars: `AZURE_KIMI_API_KEY` and `AZURE_KIMI_ENDPOINT`
- default route: `substrate-default`
- think route: `substrate-think`

## 4. Start the gateway

Use either the built binary:

```bash
./gateway/target/release/substrate-gateway start
```

Or run it through Cargo:

```bash
cargo run --manifest-path gateway/Cargo.toml -p substrate-gateway -- start
```

By default it listens on:

```text
http://127.0.0.1:13456
```

## 5. Install the Claude Code statusline

This gives you the routed model/provider readout from `~/.substrate-gateway/last_routing.json`:

```bash
./gateway/target/release/substrate-gateway install-statusline
```

The command prints the snippet to add to `~/.claude/settings.json`.

## 6. Point Claude Code at the gateway

Export the Anthropic-compatible gateway URL plus a placeholder API key for local bootstrap:

```bash
export ANTHROPIC_BASE_URL="http://127.0.0.1:13456"
export ANTHROPIC_API_KEY="any-string"
```

Then launch Claude Code:

```bash
claude
```

## 7. How routing works once Claude Code is attached

- regular Claude model requests auto-map to the router default because the gateway's default `auto_map_regex` matches `^claude-`
- the default route goes to `substrate-default`
- Claude Code plan / thinking turns route to `substrate-think`
- both of those public routing labels map internally to your Azure deployment names through `actual_model`

## 8. Quick validation

Check the gateway status:

```bash
./gateway/target/release/substrate-gateway status
```

Check the latest routed model:

```bash
cat ~/.substrate-gateway/last_routing.json
```

Optional direct smoke against the public Anthropic-compatible surface:

```bash
curl -sS http://127.0.0.1:13456/v1/messages \
  -H "Content-Type: application/json" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "claude-sonnet-4-5",
    "max_tokens": 64,
    "messages": [
      { "role": "user", "content": "Reply with one short sentence." }
    ]
  }'
```

If you want to test the thinking path directly, enable `thinking` (same model name):

```bash
curl -sS http://127.0.0.1:13456/v1/messages \
  -H "Content-Type: application/json" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "claude-sonnet-4-5",
    "max_tokens": 64,
    "thinking": { "type": "enabled", "budget_tokens": 1024 },
    "messages": [
      { "role": "user", "content": "Reply with one short sentence." }
    ]
  }'
```

## 9. Common setup mistakes

- `AZURE_KIMI_ENDPOINT` points at the wrong host or path
- `actual_model` does not match the Azure deployment name exactly
- the gateway is running but Claude Code was not launched with `ANTHROPIC_BASE_URL`
- the statusline was not installed, so you cannot quickly confirm which route was chosen
- tracing was enabled for general use instead of only when you need redacted debugging evidence
