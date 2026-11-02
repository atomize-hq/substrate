use claude_code::{
    PluginDisableRequest, PluginEnableRequest, PluginInstallRequest, PluginListRequest,
    PluginManifestMarketplaceRequest, PluginManifestRequest, PluginMarketplaceAddRequest,
    PluginMarketplaceListRequest, PluginMarketplaceRemoveRequest, PluginMarketplaceRepoRequest,
    PluginMarketplaceRequest, PluginMarketplaceUpdateRequest, PluginRequest,
    PluginUninstallRequest, PluginUpdateRequest, PluginValidateRequest,
};

#[test]
fn plugin_root_argv() {
    let argv = PluginRequest::new().into_command().argv();
    assert_eq!(argv, ["plugin"]);

    let argv = PluginMarketplaceRequest::new().into_command().argv();
    assert_eq!(argv, ["plugin", "marketplace"]);
}

#[test]
fn plugin_manifest_argv() {
    let argv = PluginManifestRequest::new().into_command().argv();
    assert_eq!(argv, ["plugin", "manifest"]);

    let argv = PluginManifestMarketplaceRequest::new()
        .into_command()
        .argv();
    assert_eq!(argv, ["plugin", "manifest", "marketplace"]);
}

#[test]
fn plugin_marketplace_repo_argv() {
    let argv = PluginMarketplaceRepoRequest::new().into_command().argv();
    assert_eq!(argv, ["plugin", "marketplace", "repo"]);
}

#[test]
fn plugin_marketplace_add_argv() {
    let argv = PluginMarketplaceAddRequest::new("https://example.com")
        .into_command()
        .argv();
    assert_eq!(
        argv,
        ["plugin", "marketplace", "add", "https://example.com"]
    );
}

#[test]
fn plugin_enable_orders_scope_before_plugin_name() {
    let argv = PluginEnableRequest::new("my-plugin")
        .scope("project")
        .into_command()
        .argv();
    assert_eq!(
        argv,
        ["plugin", "enable", "--scope", "project", "my-plugin"]
    );
}

#[test]
fn plugin_disable_orders_flags_before_positionals() {
    let argv = PluginDisableRequest::new()
        .all(true)
        .scope("user")
        .into_command()
        .argv();
    assert_eq!(argv, ["plugin", "disable", "--all", "--scope", "user"]);
}

#[test]
fn plugin_list_orders_flags_deterministically() {
    let argv = PluginListRequest::new()
        .available(true)
        .json(true)
        .into_command()
        .argv();
    assert_eq!(argv, ["plugin", "list", "--available", "--json"]);
}

#[test]
fn plugin_install_uninstall_order_scope_flag() {
    let argv = PluginInstallRequest::new()
        .scope("project")
        .into_command()
        .argv();
    assert_eq!(argv, ["plugin", "install", "--scope", "project"]);

    let argv = PluginUninstallRequest::new()
        .scope("project")
        .into_command()
        .argv();
    assert_eq!(argv, ["plugin", "uninstall", "--scope", "project"]);
}

#[test]
fn plugin_update_includes_scope_before_plugin_name() {
    let argv = PluginUpdateRequest::new("my-plugin")
        .scope("project")
        .into_command()
        .argv();
    assert_eq!(
        argv,
        ["plugin", "update", "--scope", "project", "my-plugin"]
    );
}

#[test]
fn plugin_validate_includes_path_positional() {
    let argv = PluginValidateRequest::new("path/to/plugin")
        .into_command()
        .argv();
    assert_eq!(argv, ["plugin", "validate", "path/to/plugin"]);
}

#[test]
fn plugin_marketplace_list_includes_json_flag() {
    let argv = PluginMarketplaceListRequest::new()
        .json(true)
        .into_command()
        .argv();
    assert_eq!(argv, ["plugin", "marketplace", "list", "--json"]);
}

#[test]
fn plugin_marketplace_remove_update_argv() {
    let argv = PluginMarketplaceRemoveRequest::new().into_command().argv();
    assert_eq!(argv, ["plugin", "marketplace", "remove"]);

    let argv = PluginMarketplaceUpdateRequest::new().into_command().argv();
    assert_eq!(argv, ["plugin", "marketplace", "update"]);
}
