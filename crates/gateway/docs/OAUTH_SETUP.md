# OAuth Authentication Setup

Substrate Gateway supports OAuth authentication for Claude Pro/Max subscriptions, allowing you to use your Claude subscription without needing an API key!

## Features

- ✅ **Zero Cost**: Max plan users pay $0 for API calls
- ✅ **PKCE Security**: Secure OAuth 2.0 with PKCE (Proof Key for Code Exchange)
- ✅ **Auto Refresh**: Tokens are automatically refreshed when expired
- ✅ **Persistent Storage**: Tokens stored securely in `~/.substrate-gateway/oauth_tokens.json`

## Quick Start

### 1. Get Authorization URL

```rust
use substrate_gateway::auth::{OAuthClient, OAuthConfig, TokenStore};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize
    let config = OAuthConfig::anthropic();
    let token_store = TokenStore::load_default()?;
    let oauth_client = OAuthClient::new(config, token_store);

    // Get authorization URL
    let auth_url = oauth_client.get_authorization_url();

    println!("Go to: {}", auth_url.url);
    println!();
    println!("After authorization, you'll receive a code.");
    println!("Enter the code here:");

    // Read code from user
    let mut code = String::new();
    std::io::stdin().read_line(&mut code)?;
    let code = code.trim();

    // Exchange code for tokens
    let token = oauth_client.exchange_code(
        code,
        &auth_url.verifier.verifier,
        "anthropic-max"  // Provider ID
    ).await?;

    println!("✅ Authentication successful!");
    println!("Access token expires at: {}", token.expires_at);

    Ok(())
}
```

### 2. Configure Provider

Create `~/.substrate-gateway/config.toml`:

```toml
[server]
host = "127.0.0.1"
port = 13456

[router]
default = "claude-sonnet-4.5"

# OAuth Provider
[[providers]]
name = "claude-max"
provider_type = "anthropic"
auth_type = "oauth"  # Use OAuth instead of api_key
oauth_provider = "anthropic-max"  # Must match the provider_id used in exchange_code
enabled = true
models = []

[[models]]
name = "claude-sonnet-4.5"

[[models.mappings]]
actual_model = "claude-sonnet-4-5-20250929"
priority = 1
provider = "claude-max"
```

### 3. Start Server

```bash
substrate-gateway start
```

## OAuth Configuration Options

### OAuthConfig::anthropic()

For Claude Pro/Max users:

- **Client ID**: `******`
- **Auth URL**: `https://claude.ai/oauth/authorize`
- **Token URL**: `https://console.anthropic.com/v1/oauth/token`
- **Scopes**: `org:create_api_key user:profile user:inference`

### OAuthConfig::anthropic_console()

For creating an API key via OAuth (alternative flow):

- Uses console.anthropic.com for authorization
- Creates an API key automatically after OAuth
- Useful if you want a traditional API key workflow

## Token Storage

Tokens are stored in JSON format at `~/.substrate-gateway/oauth_tokens.json`:

```json
{
  "anthropic-max": {
    "provider_id": "anthropic-max",
    "access_token": "ey...",
    "refresh_token": "rt_...",
    "expires_at": "2025-11-18T15:30:00Z",
    "enterprise_url": null
  }
}
```

File permissions are automatically set to `0600` (owner read/write only) for security.

### ChatGPT Codex handoff note

For integrated ChatGPT Codex deployments, the gateway does not treat local token files as the trust boundary for account identity. The authoritative handoff contract is documented in [`docs/contracts/chatgpt-codex-auth-handoff-contract.md`](contracts/chatgpt-codex-auth-handoff-contract.md).

Standalone local token material, including `~/.substrate-gateway/oauth_tokens.json`, remains a compatibility path only. It is useful for gateway-local OAuth operation, but it must not be read as proof of the integrated Substrate-owned auth boundary.

For the Codex route, account resolution follows a strict order:

1. In integrated mode, use `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID` first.
2. If that field is absent, fall back to the `chatgpt_account_id` claim inside the same Substrate-delivered OAuth access token.
3. In standalone mode, use explicit `account_id` from local Codex auth state first, then the same JWT fallback rule.

The JWT claim path is bounded compatibility fallback only. It does not redefine ownership, and the gateway must fail before any upstream Codex request when neither explicit nor JWT-derived `account_id` can be resolved.

For maintenance purposes, keep the Codex auth owner line explicit:

- integrated mode consumes Substrate-delivered auth context first
- explicit `account_id` remains authoritative over JWT-derived fallback
- JWT-derived account identity is bounded compatibility fallback only
- unresolved account identity must fail before any upstream request is sent
- integrated mode must not require direct reads of host-local auth files inside the gateway runtime

Codex auth or route drift should be revalidated against:

- [`chatgpt-codex-auth-handoff-contract.md`](contracts/chatgpt-codex-auth-handoff-contract.md)
- [`chatgpt-codex-route-contract.md`](contracts/chatgpt-codex-route-contract.md)
- [`chatgpt-codex-conformance-and-drift-guard.md`](contracts/chatgpt-codex-conformance-and-drift-guard.md)

The maintenance evidence anchors for that revalidation are:

- `crates/gateway/src/providers/openai.rs`
- `crates/gateway/tests/openai_responses_conformance.rs`
- `crates/gateway/tests/openai_shared_parity.rs`
- `crates/gateway/src/server/openai_conformance_test_support.rs`
- `crates/gateway/tests/fixtures/openai_responses/codex-*.json`
- this guide and `docs/OAUTH_TESTING.md`

Treat the Codex auth guidance as stale and reopen route-specific review when any of the following drift materially:

- Substrate auth-bundle delivery posture, secret-channel semantics, or integrated owner-line assumptions change
- auth field identifiers, `account_id` precedence rules, or JWT fallback constraints change
- the Codex route header contract or pre-upstream auth failure posture changes
- local OAuth or token-storage guidance starts implying integrated trust-boundary authority

## HTTP Endpoints

The gateway exposes these OAuth HTTP endpoints directly:

- `POST /api/oauth/authorize` - Get authorization URL
- `POST /api/oauth/exchange` - Exchange authorization code for tokens
- `GET /api/oauth/tokens` - List all OAuth providers
- `DELETE /api/oauth/tokens/:provider` - Remove OAuth token

## Usage Example

```rust
use substrate_gateway::auth::{OAuthClient, OAuthConfig, TokenStore};

let config = OAuthConfig::anthropic();
let token_store = TokenStore::load_default()?;
let client = OAuthClient::new(config, token_store);

// Get valid token (auto-refreshes if needed)
let access_token = client.get_valid_token("anthropic-max").await?;

// Use in HTTP request
let response = reqwest::Client::new()
    .post("https://api.anthropic.com/v1/messages")
    .header("Authorization", format!("Bearer {}", access_token))
    .header("anthropic-version", "2023-06-01")
    .json(&request_body)
    .send()
    .await?;
```

## Security Notes

1. **Never commit tokens**: The `oauth_tokens.json` file contains sensitive credentials
2. **File permissions**: Always stored with `0600` permissions (Unix)
3. **PKCE**: Uses SHA-256 challenge for additional security
4. **Auto-refresh**: Tokens are refreshed 5 minutes before expiration

## Comparison: API Key vs OAuth

| Feature    | API Key           | OAuth (Max Plan)              |
| ---------- | ----------------- | ----------------------------- |
| Setup      | Simple            | One-time OAuth flow           |
| Cost       | Pay per token     | $0 (included in subscription) |
| Security   | Static key        | Rotating tokens + PKCE        |
| Sharing    | Easy (but unsafe) | Per-user authentication       |
| Expiration | Never             | Auto-refreshes                |

## Troubleshooting

### "Token refresh failed"

- Check your internet connection
- Verify your Max subscription is active
- Re-authenticate: Delete token and run OAuth flow again

### "No token found for provider"

- Run the OAuth authorization flow first
- Check that `oauth_provider` in config matches the provider_id in TokenStore

### "Environment variable not found"

- OAuth doesn't use environment variables
- Make sure `auth_type = "oauth"` is set in provider config

## Related

- [OpenCode Anthropic Auth](https://github.com/sst/opencode-anthropic-auth) - Inspiration for this implementation
- [Anthropic OAuth Docs](https://docs.anthropic.com/claude/reference/oauth)
