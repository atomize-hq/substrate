use super::*;

#[cfg(unix)]
#[tokio::test]
async fn mcp_list_get_and_add_map_args_and_parse_json() {
    let dir = tempfile::tempdir().unwrap();
    let script_path = write_fake_codex(
        dir.path(),
        r#"#!/usr/bin/env bash
printf "%s\n" "$@" 1>&2
cat <<'JSON'
{"servers":[{"name":"files"}]}
JSON
"#,
    );

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let list = client
        .mcp_list(McpListRequest::new().json(true))
        .await
        .unwrap();
    assert_eq!(list.json, Some(json!({"servers": [{"name": "files"}]})));
    assert_eq!(
        list.stderr.lines().collect::<Vec<_>>(),
        vec!["mcp", "list", "--json"]
    );

    let get = client
        .mcp_get(McpGetRequest::new("files").json(true))
        .await
        .unwrap();
    assert_eq!(get.json, Some(json!({"servers": [{"name": "files"}]})));
    assert_eq!(
        get.stderr.lines().collect::<Vec<_>>(),
        vec!["mcp", "get", "--json", "files"]
    );
}

#[cfg(unix)]
#[tokio::test]
async fn mcp_add_maps_transports_and_validates_required_fields() {
    let dir = tempfile::tempdir().unwrap();
    let script_path = write_fake_codex(
        dir.path(),
        r#"#!/usr/bin/env bash
printf "%s\n" "$@"
"#,
    );

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let stdio = client
        .mcp_add(
            McpAddRequest::stdio("files", vec![OsString::from("node"), OsString::from("srv")])
                .env("TOKEN", "abc"),
        )
        .await
        .unwrap();
    assert_eq!(
        stdio.stdout.lines().collect::<Vec<_>>(),
        vec![
            "mcp",
            "add",
            "files",
            "--env",
            "TOKEN=abc",
            "--",
            "node",
            "srv"
        ]
    );

    let http = client
        .mcp_add(
            McpAddRequest::streamable_http("http", "https://example.test")
                .bearer_token_env_var("TOKEN_ENV"),
        )
        .await
        .unwrap();
    assert_eq!(
        http.stdout.lines().collect::<Vec<_>>(),
        vec![
            "mcp",
            "add",
            "http",
            "--url",
            "https://example.test",
            "--bearer-token-env-var",
            "TOKEN_ENV"
        ]
    );

    let err = client
        .mcp_add(McpAddRequest::stdio("files", Vec::new()))
        .await
        .unwrap_err();
    assert!(matches!(err, CodexError::EmptyMcpCommand));

    let err = client
        .mcp_add(McpAddRequest::streamable_http("bad", "  "))
        .await
        .unwrap_err();
    assert!(matches!(err, CodexError::EmptyMcpUrl));
}
