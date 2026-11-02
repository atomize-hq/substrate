//! Demonstrates `codex plugin ...` marketplace helpers via the wrapper.
//!
//! Usage:
//! - `cargo run -p unified-agent-api-codex --example plugin_cmd -- root`
//! - `cargo run -p unified-agent-api-codex --example plugin_cmd -- help [COMMAND ...]`
//! - `cargo run -p unified-agent-api-codex --example plugin_cmd -- marketplace`
//! - `cargo run -p unified-agent-api-codex --example plugin_cmd -- marketplace help [COMMAND ...]`
//! - `cargo run -p unified-agent-api-codex --example plugin_cmd -- marketplace add <SOURCE> [--ref <REF>] [--sparse <PATH>]`
//! - `cargo run -p unified-agent-api-codex --example plugin_cmd -- marketplace remove <NAME>`
//! - `cargo run -p unified-agent-api-codex --example plugin_cmd -- marketplace upgrade [NAME]`
//!
//! Environment:
//! - `CODEX_BINARY` (optional): path to the `codex` CLI binary (defaults to `codex` in PATH).
//! - `CODEX_HOME` (optional): Codex home directory for config/auth.

use codex::{
    PluginCommandRequest, PluginHelpRequest, PluginMarketplaceAddRequest,
    PluginMarketplaceCommandRequest, PluginMarketplaceHelpRequest, PluginMarketplaceRemoveRequest,
    PluginMarketplaceUpgradeRequest,
};

#[path = "support/real_cli.rs"]
mod real_cli;

fn usage() {
    eprintln!("usage:");
    eprintln!("  plugin_cmd root");
    eprintln!("  plugin_cmd help [COMMAND ...]");
    eprintln!("  plugin_cmd marketplace");
    eprintln!("  plugin_cmd marketplace help [COMMAND ...]");
    eprintln!("  plugin_cmd marketplace add <SOURCE> [--ref <REF>] [--sparse <PATH>]");
    eprintln!("  plugin_cmd marketplace remove <NAME>");
    eprintln!("  plugin_cmd marketplace upgrade [NAME]");
}

fn print_output(output: codex::ApplyDiffArtifacts) {
    print!("{}", output.stdout);
    if !output.stderr.is_empty() {
        eprintln!("{}", output.stderr);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage();
        return Ok(());
    }

    let client = real_cli::default_client();

    match args[0].as_str() {
        "root" => {
            let output = client.plugin(PluginCommandRequest::new()).await?;
            print_output(output);
        }
        "help" => {
            let output = client
                .plugin_help(PluginHelpRequest::new().command(args.drain(1..)))
                .await?;
            print_output(output);
        }
        "marketplace" => {
            if args.len() == 1 {
                let output = client
                    .plugin_marketplace(PluginMarketplaceCommandRequest::new())
                    .await?;
                print_output(output);
                return Ok(());
            }

            match args[1].as_str() {
                "help" => {
                    let output = client
                        .plugin_marketplace_help(
                            PluginMarketplaceHelpRequest::new().command(args.drain(2..)),
                        )
                        .await?;
                    print_output(output);
                }
                "add" => {
                    if args.len() < 3 {
                        usage();
                        return Ok(());
                    }
                    let mut idx = 2;
                    let source = args[idx].clone();
                    idx += 1;
                    let mut request = PluginMarketplaceAddRequest::new(source);
                    while idx < args.len() {
                        match args[idx].as_str() {
                            "--ref" => {
                                idx += 1;
                                if idx >= args.len() {
                                    usage();
                                    return Ok(());
                                }
                                request = request.source_ref(args[idx].clone());
                            }
                            "--sparse" => {
                                idx += 1;
                                if idx >= args.len() {
                                    usage();
                                    return Ok(());
                                }
                                request = request.sparse_path(args[idx].clone());
                            }
                            _ => {
                                usage();
                                return Ok(());
                            }
                        }
                        idx += 1;
                    }
                    let output = client.plugin_marketplace_add(request).await?;
                    print_output(output);
                }
                "remove" => {
                    let Some(name) = args.get(2) else {
                        usage();
                        return Ok(());
                    };
                    let output = client
                        .plugin_marketplace_remove(PluginMarketplaceRemoveRequest::new(name))
                        .await?;
                    print_output(output);
                }
                "upgrade" => {
                    let mut request = PluginMarketplaceUpgradeRequest::new();
                    if let Some(name) = args.get(2) {
                        request = request.marketplace_name(name.clone());
                    }
                    let output = client.plugin_marketplace_upgrade(request).await?;
                    print_output(output);
                }
                _ => usage(),
            }
        }
        _ => usage(),
    }

    Ok(())
}
