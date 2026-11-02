use claude_code::{
    McpAddFromClaudeDesktopRequest, McpAddJsonRequest, McpAddRequest, McpRemoveRequest, McpScope,
    McpServeRequest, McpTransport,
};

#[test]
fn mcp_add_argv_orders_options_before_positionals() {
    let req = McpAddRequest::new("my-server", "npx")
        .scope(McpScope::User)
        .transport(McpTransport::Stdio)
        .env(["KEY=value"])
        .headers(["X-Test: 1"])
        .args(["--foo", "bar"]);

    let argv = req.into_command().argv();
    assert!(argv.starts_with(&[
        "mcp".to_string(),
        "add".to_string(),
        "--scope".to_string(),
        "user".to_string(),
        "--transport".to_string(),
        "stdio".to_string(),
        "--env".to_string(),
        "KEY=value".to_string(),
        "--header".to_string(),
        "X-Test: 1".to_string(),
        "my-server".to_string(),
        "npx".to_string(),
        "--foo".to_string(),
        "bar".to_string(),
    ]));
}

#[test]
fn mcp_remove_includes_scope_when_set() {
    let req = McpRemoveRequest::new("my-server").scope(McpScope::Project);
    let argv = req.into_command().argv();
    assert_eq!(
        argv,
        vec![
            "mcp".to_string(),
            "remove".to_string(),
            "--scope".to_string(),
            "project".to_string(),
            "my-server".to_string(),
        ]
    );
}

#[test]
fn mcp_add_json_includes_scope_when_set() {
    let req =
        McpAddJsonRequest::new("my-server", r#"{"transport":"stdio"}"#).scope(McpScope::Local);
    let argv = req.into_command().argv();
    assert_eq!(
        argv,
        vec![
            "mcp".to_string(),
            "add-json".to_string(),
            "--scope".to_string(),
            "local".to_string(),
            "my-server".to_string(),
            r#"{"transport":"stdio"}"#.to_string(),
        ]
    );
}

#[test]
fn mcp_serve_argv_allows_extra_args() {
    let argv = McpServeRequest::new()
        .args(["--foo", "bar"])
        .into_command()
        .argv();
    assert_eq!(argv, ["mcp", "serve", "--foo", "bar"]);
}

#[test]
fn mcp_add_from_claude_desktop_orders_scope_flag() {
    let argv = McpAddFromClaudeDesktopRequest::new()
        .scope(McpScope::User)
        .into_command()
        .argv();
    assert_eq!(argv, ["mcp", "add-from-claude-desktop", "--scope", "user"]);
}
