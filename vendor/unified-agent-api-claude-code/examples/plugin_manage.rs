//! Demonstrates plugin management commands that are upstream-gated to Windows (`win32-x64`).
//!
//! Usage (Windows only):
//! - `cargo run -p unified-agent-api-claude-code --example plugin_manage -- list`
//! - Mutating commands require: `CLAUDE_EXAMPLE_ALLOW_MUTATION=1`
//!   - `... -- enable <PLUGIN>`
//!   - `... -- disable`
//!   - `... -- install`
//!   - `... -- uninstall`
//!   - `... -- update <PLUGIN>`

use std::{env, error::Error};

use claude_code::{
    PluginDisableRequest, PluginEnableRequest, PluginInstallRequest, PluginListRequest,
    PluginManifestMarketplaceRequest, PluginManifestRequest, PluginMarketplaceAddRequest,
    PluginMarketplaceListRequest, PluginMarketplaceRemoveRequest, PluginMarketplaceRepoRequest,
    PluginMarketplaceRequest, PluginMarketplaceUpdateRequest, PluginRequest,
    PluginUninstallRequest, PluginUpdateRequest, PluginValidateRequest,
};

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !cfg!(windows) {
        eprintln!(
            "skipped plugin_manage: upstream plugin management subcommands are win32-x64 only"
        );
        return Ok(());
    }

    let client = real_cli::maybe_isolated_client("plugin_manage")?;
    let mut args = env::args().skip(1);
    let sub = args.next().unwrap_or_else(|| "list".to_string());

    match sub.as_str() {
        "root" => {
            let out = client.plugin(PluginRequest::new()).await?;
            println!("exit: {}", out.status);
        }
        "list" => {
            let out = client
                .plugin_list(PluginListRequest::new().available(true).json(true))
                .await?;
            println!("exit: {}", out.status);
            print!("{}", String::from_utf8_lossy(&out.stdout));
        }
        "enable" => {
            real_cli::require_mutation("plugin_manage enable")?;
            if !real_cli::mutation_enabled() {
                return Ok(());
            }
            let plugin = args.next().ok_or("usage: enable <PLUGIN>")?;
            let out = client
                .plugin_enable(PluginEnableRequest::new(plugin).scope("user"))
                .await?;
            println!("exit: {}", out.status);
        }
        "disable" => {
            real_cli::require_mutation("plugin_manage disable")?;
            if !real_cli::mutation_enabled() {
                return Ok(());
            }
            let out = client
                .plugin_disable(PluginDisableRequest::new().all(true).scope("user"))
                .await?;
            println!("exit: {}", out.status);
        }
        "install" => {
            real_cli::require_mutation("plugin_manage install")?;
            if !real_cli::mutation_enabled() {
                return Ok(());
            }
            let out = client
                .plugin_install(PluginInstallRequest::new().scope("user"))
                .await?;
            println!("exit: {}", out.status);
        }
        "uninstall" => {
            real_cli::require_mutation("plugin_manage uninstall")?;
            if !real_cli::mutation_enabled() {
                return Ok(());
            }
            let out = client
                .plugin_uninstall(PluginUninstallRequest::new().scope("user"))
                .await?;
            println!("exit: {}", out.status);
        }
        "update" => {
            real_cli::require_mutation("plugin_manage update")?;
            if !real_cli::mutation_enabled() {
                return Ok(());
            }
            let plugin = args.next().ok_or("usage: update <PLUGIN>")?;
            let out = client
                .plugin_update(PluginUpdateRequest::new(plugin).scope("user"))
                .await?;
            println!("exit: {}", out.status);
        }
        "validate" => {
            let path = args.next().ok_or("usage: validate <PATH>")?;
            let out = client
                .plugin_validate(PluginValidateRequest::new(path))
                .await?;
            println!("exit: {}", out.status);
        }
        "manifest" => {
            let out = client.plugin_manifest(PluginManifestRequest::new()).await?;
            println!("exit: {}", out.status);
            print!("{}", String::from_utf8_lossy(&out.stdout));
        }
        "manifest-marketplace" => {
            let out = client
                .plugin_manifest_marketplace(PluginManifestMarketplaceRequest::new())
                .await?;
            println!("exit: {}", out.status);
            print!("{}", String::from_utf8_lossy(&out.stdout));
        }
        "marketplace" => {
            let out = client
                .plugin_marketplace(PluginMarketplaceRequest::new())
                .await?;
            println!("exit: {}", out.status);
        }
        "marketplace-add" => {
            real_cli::require_mutation("plugin_manage marketplace-add")?;
            if !real_cli::mutation_enabled() {
                return Ok(());
            }
            let source = args.next().ok_or("usage: marketplace-add <SOURCE>")?;
            let out = client
                .plugin_marketplace_add(PluginMarketplaceAddRequest::new(source))
                .await?;
            println!("exit: {}", out.status);
        }
        "marketplace-list" => {
            let out = client
                .plugin_marketplace_list(PluginMarketplaceListRequest::new().json(true))
                .await?;
            println!("exit: {}", out.status);
            print!("{}", String::from_utf8_lossy(&out.stdout));
        }
        "marketplace-remove" => {
            real_cli::require_mutation("plugin_manage marketplace-remove")?;
            if !real_cli::mutation_enabled() {
                return Ok(());
            }
            let out = client
                .plugin_marketplace_remove(PluginMarketplaceRemoveRequest::new())
                .await?;
            println!("exit: {}", out.status);
        }
        "marketplace-repo" => {
            let out = client
                .plugin_marketplace_repo(PluginMarketplaceRepoRequest::new())
                .await?;
            println!("exit: {}", out.status);
            print!("{}", String::from_utf8_lossy(&out.stdout));
        }
        "marketplace-update" => {
            real_cli::require_mutation("plugin_manage marketplace-update")?;
            if !real_cli::mutation_enabled() {
                return Ok(());
            }
            let out = client
                .plugin_marketplace_update(PluginMarketplaceUpdateRequest::new())
                .await?;
            println!("exit: {}", out.status);
        }
        other => {
            eprintln!("unknown subcommand: {other}");
        }
    }

    Ok(())
}
