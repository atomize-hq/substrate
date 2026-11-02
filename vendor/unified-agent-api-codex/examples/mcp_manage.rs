//! Demonstrates the `codex mcp` management wrapper APIs (`list/get/add/remove/login/logout`).
//!
//! Note: these commands read/write Codex MCP configuration, so they may require CODEX_HOME to be
//! configured and may prompt or open a browser for OAuth login depending on the server type.
//!
//! Usage:
//! - List servers (JSON):
//!   `cargo run -p unified-agent-api-codex --example mcp_manage -- list --json`
//! - Get a server entry:
//!   `cargo run -p unified-agent-api-codex --example mcp_manage -- get <NAME> --json`
//! - Add (matches `codex mcp add` shape):
//!   - Streamable HTTP:
//!     `cargo run -p unified-agent-api-codex --example mcp_manage -- add <NAME> --url <URL> [--bearer-token-env-var ENV_VAR]`
//!   - Stdio:
//!     `cargo run -p unified-agent-api-codex --example mcp_manage -- add <NAME> [--env KEY=VALUE ...] -- <COMMAND>...`
//! - Roundtrip (isolated home): add -> list -> get -> remove
//!   `cargo run -p unified-agent-api-codex --example mcp_manage -- roundtrip [NAME] [URL]`
//! - (Legacy aliases)
//!   - `add-http <NAME> <URL> [BEARER_TOKEN_ENV_VAR]`
//!   - `add-stdio <NAME> -- <COMMAND>...`
//! - Remove a server:
//!   `cargo run -p unified-agent-api-codex --example mcp_manage -- remove <NAME>`
//! - Logout:
//!   `cargo run -p unified-agent-api-codex --example mcp_manage -- logout <NAME>`
//! - OAuth login (spawns process; stdout/stderr are inherited by the wrapper):
//!   `cargo run -p unified-agent-api-codex --example mcp_manage -- login <NAME> [scope1 scope2 ...]`
//!
//! Isolation:
//! - `--isolated-home` uses a fresh `CODEX_HOME` under `target/` (avoids mutating your real home).

use std::{env, error::Error, ffi::OsString};

use codex::{
    McpAddRequest, McpGetRequest, McpListRequest, McpLogoutRequest, McpOauthLoginRequest,
    McpRemoveRequest,
};

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        eprintln!("missing subcommand (list/get/add-http/add-stdio/remove/logout/login)");
        return Ok(());
    }

    let use_isolated_home = args.iter().any(|arg| arg == "--isolated-home");
    args.retain(|arg| arg != "--isolated-home");

    let subcommand = args.remove(0);
    let client = if use_isolated_home || real_cli::wants_isolated_home() {
        let home = real_cli::isolated_home_root("mcp_manage");
        real_cli::build_client_with_home(&home)
    } else {
        real_cli::default_client()
    };

    match subcommand.as_str() {
        "overview" => {
            let output = client.mcp_overview(Default::default()).await?;
            print!("{}", output.stdout);
        }
        "list" => {
            let json = args.first().is_some_and(|v| v == "--json");
            let output = client.mcp_list(McpListRequest::new().json(json)).await?;
            if let Some(value) = output.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
                );
            } else {
                print!("{}", output.stdout);
            }
        }
        "get" => {
            let name = args
                .first()
                .ok_or("usage: mcp_manage get <NAME> [--json]")?;
            let json = args.iter().any(|v| v == "--json");
            let output = client
                .mcp_get(McpGetRequest::new(name.to_string()).json(json))
                .await?;
            if let Some(value) = output.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
                );
            } else {
                print!("{}", output.stdout);
            }
        }
        "roundtrip" => {
            // Exercises add -> list -> get -> remove using an isolated CODEX_HOME to avoid mutating
            // the user's real MCP config.
            let home = real_cli::isolated_home_root("mcp_manage_roundtrip");
            let client = real_cli::build_client_with_home(&home);

            let name = args
                .first()
                .cloned()
                .unwrap_or_else(|| "example".to_string());
            let url = args
                .get(1)
                .cloned()
                .unwrap_or_else(|| "https://mcp.deepwiki.com/mcp".to_string());

            client
                .mcp_add(McpAddRequest::streamable_http(name.clone(), url))
                .await?;

            let _ = client.mcp_list(McpListRequest::new().json(false)).await?;
            let _ = client
                .mcp_get(McpGetRequest::new(name.clone()).json(false))
                .await?;
            let _ = client
                .mcp_remove(McpRemoveRequest::new(name.clone()))
                .await?;
        }
        "add" => {
            let name = args
                .first()
                .ok_or("usage: mcp_manage add <NAME> (--url <URL> | -- <COMMAND>...)")?
                .to_string();
            let mut tail = args.into_iter().skip(1).collect::<Vec<_>>();

            let mut url: Option<String> = None;
            let mut bearer_token_env_var: Option<String> = None;
            let mut env_pairs: Vec<(String, String)> = Vec::new();

            while !tail.is_empty() {
                match tail[0].as_str() {
                    "--url" => {
                        tail.remove(0);
                        url = tail.first().cloned();
                        if url.is_some() {
                            tail.remove(0);
                        }
                    }
                    "--bearer-token-env-var" => {
                        tail.remove(0);
                        bearer_token_env_var = tail.first().cloned();
                        if bearer_token_env_var.is_some() {
                            tail.remove(0);
                        }
                    }
                    "--env" => {
                        tail.remove(0);
                        if let Some(kv) = tail.first().cloned() {
                            tail.remove(0);
                            if let Some((k, v)) = kv.split_once('=') {
                                env_pairs.push((k.to_string(), v.to_string()));
                            } else {
                                return Err(format!("--env expects KEY=VALUE, got {kv:?}").into());
                            }
                        }
                    }
                    "--" => break,
                    _ => break,
                }
            }

            let output = if let Some(url) = url {
                let mut request = McpAddRequest::streamable_http(name, url);
                if let Some(env_var) = bearer_token_env_var {
                    request = request.bearer_token_env_var(env_var);
                }
                client.mcp_add(request).await?
            } else {
                let delimiter = tail
                    .iter()
                    .position(|v| v == "--")
                    .ok_or("usage: mcp_manage add <NAME> [--env KEY=VALUE ...] -- <COMMAND>...")?;
                let command = tail
                    .iter()
                    .skip(delimiter + 1)
                    .map(OsString::from)
                    .collect::<Vec<_>>();

                let mut request = McpAddRequest::stdio(name, command);
                for (key, value) in env_pairs {
                    request = request.env(key, value);
                }
                client.mcp_add(request).await?
            };

            print!("{}", output.stdout);
        }
        "add-http" => {
            let name = args
                .first()
                .ok_or("usage: mcp_manage add-http <NAME> <URL> [TOKEN_ENV]")?;
            let url = args
                .get(1)
                .ok_or("usage: mcp_manage add-http <NAME> <URL> [TOKEN_ENV]")?;
            let token_env = args.get(2).cloned();

            let mut request = McpAddRequest::streamable_http(name.to_string(), url.to_string());
            if let Some(token_env) = token_env {
                request = request.bearer_token_env_var(token_env);
            }

            let output = client.mcp_add(request).await?;
            print!("{}", output.stdout);
        }
        "add-stdio" => {
            let name = args
                .first()
                .ok_or("usage: mcp_manage add-stdio <NAME> -- <COMMAND>...")?;
            let delimiter = args
                .iter()
                .position(|v| v == "--")
                .ok_or("usage: mcp_manage add-stdio <NAME> -- <COMMAND>...")?;
            let command = args
                .iter()
                .skip(delimiter + 1)
                .map(OsString::from)
                .collect::<Vec<_>>();
            let output = client
                .mcp_add(McpAddRequest::stdio(name.to_string(), command))
                .await?;
            print!("{}", output.stdout);
        }
        "remove" => {
            let name = args.first().ok_or("usage: mcp_manage remove <NAME>")?;
            let output = client
                .mcp_remove(McpRemoveRequest::new(name.to_string()))
                .await?;
            print!("{}", output.stdout);
        }
        "logout" => {
            let name = args.first().ok_or("usage: mcp_manage logout <NAME>")?;
            let output = client
                .mcp_logout(McpLogoutRequest::new(name.to_string()))
                .await?;
            print!("{}", output.stdout);
        }
        "login" => {
            let name = args
                .first()
                .ok_or("usage: mcp_manage login <NAME> [scopes...]")?;
            let scopes = args.iter().skip(1).cloned().collect::<Vec<_>>();
            let mut request = McpOauthLoginRequest::new(name.to_string());
            if !scopes.is_empty() {
                request = request.scopes(scopes);
            }

            let mut child = client.spawn_mcp_oauth_login_process(request)?;
            let status = child.wait().await?;
            if !status.success() {
                return Err(format!("mcp login exited with {status:?}").into());
            }
        }
        other => {
            eprintln!("unknown subcommand: {other}");
        }
    }

    Ok(())
}
