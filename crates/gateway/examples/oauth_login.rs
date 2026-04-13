//! OAuth Login Example
//!
//! This example shows how to authenticate with Claude Pro/Max using OAuth.
//! Run with: cargo run --example oauth_login

use std::io::{self, Write};
use substrate_gateway::auth::{OAuthClient, OAuthConfig, TokenStore};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔐 Substrate Gateway OAuth Authentication");
    println!();
    println!("This will authenticate your Claude Pro/Max account");
    println!("and save the OAuth token for use with substrate-gateway.");
    println!();

    // Initialize OAuth client
    let config = OAuthConfig::anthropic();
    let token_store = TokenStore::load_default()?;
    let oauth_client = OAuthClient::new(config, token_store);

    // Generate authorization URL
    let auth_url = oauth_client.get_authorization_url();

    println!("Step 1: Visit the following URL in your browser:");
    println!();
    println!("  {}", auth_url.url);
    println!();
    println!("Step 2: After authorizing, you'll receive a code.");
    println!();
    print!("Enter the authorization code here: ");
    io::stdout().flush()?;

    // Read code from user
    let mut code = String::new();
    io::stdin().read_line(&mut code)?;
    let code = code.trim();

    // Exchange code for tokens
    println!();
    println!("Exchanging code for tokens...");

    let token = oauth_client
        .exchange_code(code, &auth_url.verifier.verifier, "anthropic-max")
        .await?;

    println!();
    println!("✅ Authentication successful!");
    println!();
    println!("Token details:");
    println!("  Provider ID: {}", token.provider_id);
    println!("  Expires at: {}", token.expires_at);
    println!();
    println!("Your OAuth token has been saved to:");
    println!("  ~/.substrate-gateway/oauth_tokens.json");
    println!();
    println!("You can now use this token in your config:");
    println!();
    println!("  [[providers]]");
    println!("  name = \"claude-max\"");
    println!("  provider_type = \"anthropic\"");
    println!("  auth_type = \"oauth\"");
    println!("  oauth_provider = \"anthropic-max\"");
    println!("  enabled = true");
    println!();

    Ok(())
}
