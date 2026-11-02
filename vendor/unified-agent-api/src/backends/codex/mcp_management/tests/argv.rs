use std::{collections::BTreeMap, ffi::OsString};

use crate::mcp::AgentWrapperMcpAddTransport;

use super::super::argv::{
    codex_mcp_add_argv, codex_mcp_get_argv, codex_mcp_list_argv, codex_mcp_remove_argv,
};

#[test]
fn codex_mcp_list_argv_is_pinned() {
    assert_eq!(
        codex_mcp_list_argv(),
        vec![
            OsString::from("mcp"),
            OsString::from("list"),
            OsString::from("--json"),
        ]
    );
}

#[test]
fn codex_mcp_get_argv_is_pinned() {
    assert_eq!(
        codex_mcp_get_argv("demo"),
        vec![
            OsString::from("mcp"),
            OsString::from("get"),
            OsString::from("--json"),
            OsString::from("demo"),
        ]
    );
}

#[test]
fn codex_mcp_remove_argv_is_pinned() {
    assert_eq!(
        codex_mcp_remove_argv("demo"),
        vec![
            OsString::from("mcp"),
            OsString::from("remove"),
            OsString::from("demo"),
        ]
    );
}

#[test]
fn codex_mcp_add_argv_maps_stdio_transport_with_sorted_env_and_separator() {
    let transport = AgentWrapperMcpAddTransport::Stdio {
        command: vec!["node".to_string()],
        args: vec!["server.js".to_string(), "--flag".to_string()],
        env: BTreeMap::from([
            ("BETA".to_string(), "two".to_string()),
            ("ALPHA".to_string(), "one".to_string()),
        ]),
    };

    assert_eq!(
        codex_mcp_add_argv("demo", &transport),
        vec![
            OsString::from("mcp"),
            OsString::from("add"),
            OsString::from("demo"),
            OsString::from("--env"),
            OsString::from("ALPHA=one"),
            OsString::from("--env"),
            OsString::from("BETA=two"),
            OsString::from("--"),
            OsString::from("node"),
            OsString::from("server.js"),
            OsString::from("--flag"),
        ]
    );
}

#[test]
fn codex_mcp_add_argv_maps_url_transport() {
    let transport = AgentWrapperMcpAddTransport::Url {
        url: "https://example.test/mcp".to_string(),
        bearer_token_env_var: Some("TOKEN_ENV".to_string()),
    };

    assert_eq!(
        codex_mcp_add_argv("demo", &transport),
        vec![
            OsString::from("mcp"),
            OsString::from("add"),
            OsString::from("demo"),
            OsString::from("--url"),
            OsString::from("https://example.test/mcp"),
            OsString::from("--bearer-token-env-var"),
            OsString::from("TOKEN_ENV"),
        ]
    );
}

#[test]
fn codex_mcp_add_argv_maps_url_transport_without_bearer_env() {
    let transport = AgentWrapperMcpAddTransport::Url {
        url: "https://example.test/mcp".to_string(),
        bearer_token_env_var: None,
    };

    assert_eq!(
        codex_mcp_add_argv("demo", &transport),
        vec![
            OsString::from("mcp"),
            OsString::from("add"),
            OsString::from("demo"),
            OsString::from("--url"),
            OsString::from("https://example.test/mcp"),
        ]
    );
}
